/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! An implementation of ecma262's [LoadRequestedModules](https://tc39.es/ecma262/#sec-LoadRequestedModules)
//! Partly inspired by mozjs implementation: <https://searchfox.org/firefox-main/source/js/src/vm/Modules.cpp#1450>
//! Since we can't access ModuleObject internals (eg. ModuleRequest records), we deviate from the spec in some aspects.

#![expect(unsafe_code)]

use std::cell::{Cell, RefCell};
use std::collections::HashSet;
use std::rc::Rc;

use js::conversions::jsstr_to_string;
use js::jsapi::{HandleValue as RawHandleValue, IsCyclicModule, JSObject, ModuleType};
use js::jsval::{ObjectValue, UndefinedValue};
use js::realm::{AutoRealm, CurrentRealm};
use js::rust::wrappers2::{
    GetModuleNamespace, GetRequestedModuleSpecifier, GetRequestedModuleType,
    GetRequestedModulesCount, JS_GetModulePrivate, ModuleEvaluate, ModuleLink,
};
use js::rust::{HandleValue, IntoHandle};
use net_traits::request::{Destination, Referrer};
use script_bindings::settings_stack::run_a_callback;
use script_bindings::str::DOMString;
use servo_url::ServoUrl;

use crate::DomTypeHolder;
use crate::dom::bindings::error::Error;
use crate::dom::bindings::refcounted::Trusted;
use crate::dom::bindings::reflector::DomObject;
use crate::dom::bindings::root::DomRoot;
use crate::dom::globalscope::GlobalScope;
use crate::dom::promise::Promise;
use crate::dom::promisenativehandler::{Callback, PromiseNativeHandler};
use crate::realms::{InRealm, enter_auto_realm};
use crate::script_module::{
    ModuleHandler, ModuleObject, ModuleOwner, ModuleTree, RethrowError, ScriptFetchOptions,
    fetch_a_single_module_script, gen_type_error, module_script_from_reference_private,
};
use crate::script_runtime::{CanGc, IntroductionType};

#[derive(JSTraceable, MallocSizeOf)]
struct OnRejectedHandler {
    #[conditional_malloc_size_of]
    promise: Rc<Promise>,
}

impl Callback for OnRejectedHandler {
    fn callback(&self, cx: &mut CurrentRealm, v: HandleValue) {
        // a. Perform ! Call(promiseCapability.[[Reject]], undefined, « reason »).
        self.promise.reject(cx.into(), v, CanGc::from_cx(cx));
    }
}

pub(crate) enum Payload {
    GraphRecord(Rc<GraphLoadingState>),
    PromiseRecord(Rc<Promise>),
}

#[derive(JSTraceable)]
pub(crate) struct LoadState {
    pub(crate) error_to_rethrow: RefCell<Option<RethrowError>>,
    #[no_trace]
    pub(crate) destination: Destination,
    pub(crate) fetch_client: ModuleOwner,
}

/// <https://tc39.es/ecma262/#graphloadingstate-record>
pub(crate) struct GraphLoadingState {
    /// [[PromiseCapability]]
    promise: Rc<Promise>,
    /// [[IsLoading]]
    is_loading: Cell<bool>,
    /// [[PendingModulesCount]]
    pending_modules_count: Cell<u32>,
    /// [[Visited]]
    visited: RefCell<HashSet<ServoUrl>>,
    /// [[HostDefined]]
    load_state: Option<Rc<LoadState>>,
}

/// <https://tc39.es/ecma262/#sec-LoadRequestedModules>
pub(crate) fn load_requested_modules(
    cx: &mut CurrentRealm,
    module: Rc<ModuleTree>,
    load_state: Option<Rc<LoadState>>,
) -> Rc<Promise> {
    // Step 1. If hostDefined is not present, let hostDefined be empty.
    //
    // Not required, since we implement it as an `Option`

    // Step 2. Let pc be ! NewPromiseCapability(%Promise%).
    let mut realm = CurrentRealm::assert(cx);
    let promise = Promise::new_in_realm(&mut realm);

    // Step 3. Let state be the GraphLoadingState Record
    // { [[IsLoading]]: true, [[PendingModulesCount]]: 1, [[Visited]]: « », [[PromiseCapability]]: pc, [[HostDefined]]: hostDefined }.
    let state = GraphLoadingState {
        promise: promise.clone(),
        is_loading: Cell::new(true),
        pending_modules_count: Cell::new(1),
        visited: RefCell::new(HashSet::new()),
        load_state,
    };

    // Step 4. Perform InnerModuleLoading(state, module).
    inner_module_loading(cx, &Rc::new(state), module);

    // Step 5. Return pc.[[Promise]].
    promise
}

/// <https://tc39.es/ecma262/#sec-InnerModuleLoading>
fn inner_module_loading(
    cx: &mut CurrentRealm,
    state: &Rc<GraphLoadingState>,
    module: Rc<ModuleTree>,
) {
    // Step 1. Assert: state.[[IsLoading]] is true.
    assert!(state.is_loading.get());

    let module_handle = module.get_record().map(|module| module.handle()).unwrap();

    let module_url = module.get_url();
    let visited_contains_module = state.visited.borrow().contains(&module_url);

    // Step 2. If module is a Cyclic Module Record, module.[[Status]] is new, and state.[[Visited]] does not contain module, then
    // Note: mozjs doesn't expose a way to check the ModuleStatus of a ModuleObject.
    if unsafe { IsCyclicModule(module_handle.get()) } && !visited_contains_module {
        // a. Append module to state.[[Visited]].
        state.visited.borrow_mut().insert(module_url);

        // b. Let requestedModulesCount be the number of elements in module.[[RequestedModules]].
        let requested_modules_count = unsafe { GetRequestedModulesCount(cx, module_handle) };

        // c. Set state.[[PendingModulesCount]] to state.[[PendingModulesCount]] + requestedModulesCount.
        let pending_modules_count = state.pending_modules_count.get();
        state
            .pending_modules_count
            .set(pending_modules_count + requested_modules_count);

        // d. For each ModuleRequest Record request of module.[[RequestedModules]], do
        for index in 0..requested_modules_count {
            // i. If AllImportAttributesSupported(request.[[Attributes]]) is false, then
            // Note: Gecko will call hasFirstUnsupportedAttributeKey on each module request,
            // GetRequestedModuleSpecifier will do it for us.
            // In addition it will also check if specifier has an unknown module type.
            let jsstr = unsafe { GetRequestedModuleSpecifier(cx, module_handle, index) };

            if jsstr.is_null() {
                // 1. Let error be ThrowCompletion(a newly created SyntaxError object).
                let error = RethrowError::from_pending_exception(cx.into());

                // See Step 7. of `host_load_imported_module`.
                state.load_state.as_ref().inspect(|load_state| {
                    load_state
                        .error_to_rethrow
                        .borrow_mut()
                        .get_or_insert(error.clone());
                });

                // 2. Perform ContinueModuleLoading(state, error).
                continue_module_loading(cx, state, Err(error));
            } else {
                let specifier =
                    unsafe { jsstr_to_string(cx.raw_cx(), std::ptr::NonNull::new(jsstr).unwrap()) };
                let module_type = unsafe { GetRequestedModuleType(cx, module_handle, index) };

                let realm = CurrentRealm::assert(cx);
                let global = GlobalScope::from_current_realm(&realm);

                // ii. Else if module.[[LoadedModules]] contains a LoadedModuleRequest Record record
                // such that ModuleRequestsEqual(record, request) is true, then
                let loaded_module =
                    module.find_descendant_inside_module_map(&global, &specifier, module_type);

                match loaded_module {
                    // 1. Perform InnerModuleLoading(state, record.[[Module]]).
                    Some(module) => inner_module_loading(cx, state, module),
                    // iii. Else,
                    None => {
                        rooted!(&in(cx) let mut referrer = UndefinedValue());
                        unsafe { JS_GetModulePrivate(module_handle.get(), referrer.handle_mut()) };

                        // 1. Perform HostLoadImportedModule(module, request, state.[[HostDefined]], state).
                        host_load_imported_module(
                            cx,
                            Some(module.clone()),
                            referrer.handle().into_handle(),
                            specifier,
                            module_type,
                            state.load_state.clone(),
                            Payload::GraphRecord(state.clone()),
                        );
                    },
                }
            }

            // iv. If state.[[IsLoading]] is false, return unused.
            if !state.is_loading.get() {
                return;
            }
        }
    }

    // Step 3. Assert: state.[[PendingModulesCount]] ≥ 1.
    assert!(state.pending_modules_count.get() >= 1);

    // Step 4. Set state.[[PendingModulesCount]] to state.[[PendingModulesCount]] - 1.
    let pending_modules_count = state.pending_modules_count.get();
    state.pending_modules_count.set(pending_modules_count - 1);

    // Step 5. If state.[[PendingModulesCount]] = 0, then
    if state.pending_modules_count.get() == 0 {
        // a. Set state.[[IsLoading]] to false.
        state.is_loading.set(false);

        // b. For each Cyclic Module Record loaded of state.[[Visited]], do
        // i. If loaded.[[Status]] is new, set loaded.[[Status]] to unlinked.
        // Note: mozjs defaults to the unlinked status.

        // c. Perform ! Call(state.[[PromiseCapability]].[[Resolve]], undefined, « undefined »).
        state.promise.resolve_native(&(), CanGc::from_cx(cx));
    }

    // Step 6. Return unused.
}

/// <https://tc39.es/ecma262/#sec-ContinueModuleLoading>
fn continue_module_loading(
    cx: &mut CurrentRealm,
    state: &Rc<GraphLoadingState>,
    module_completion: Result<Rc<ModuleTree>, RethrowError>,
) {
    // Step 1. If state.[[IsLoading]] is false, return unused.
    if !state.is_loading.get() {
        return;
    }

    match module_completion {
        // Step 2. If moduleCompletion is a normal completion, then
        // a. Perform InnerModuleLoading(state, moduleCompletion.[[Value]]).
        Ok(module) => inner_module_loading(cx, state, module),

        // Step 3. Else,
        Err(exception) => {
            // a. Set state.[[IsLoading]] to false.
            state.is_loading.set(false);

            // b. Perform ! Call(state.[[PromiseCapability]].[[Reject]], undefined, « moduleCompletion.[[Value]] »).
            state
                .promise
                .reject(cx.into(), exception.handle(), CanGc::from_cx(cx));
        },
    }

    // Step 4. Return unused.
}

/// <https://tc39.es/ecma262/#sec-FinishLoadingImportedModule>
fn finish_loading_imported_module(
    cx: &mut CurrentRealm,
    referrer_module: Option<Rc<ModuleTree>>,
    module_request_specifier: String,
    payload: Payload,
    result: Result<Rc<ModuleTree>, RethrowError>,
) {
    match payload {
        // Step 2. If payload is a GraphLoadingState Record, then
        Payload::GraphRecord(state) => {
            let module_tree =
                referrer_module.expect("Module must not be None in non dynamic imports");

            // Step 1. If result is a normal completion, then
            if let Ok(ref module) = result {
                module_tree.insert_module_dependency(module, module_request_specifier);
            }

            // a. Perform ContinueModuleLoading(payload, result).
            continue_module_loading(cx, &state, result);
        },

        // Step 3. Else,
        // a. Perform ContinueDynamicImport(payload, result).
        Payload::PromiseRecord(promise) => continue_dynamic_import(cx, promise, result),
    }

    // 4. Return unused.
}

/// <https://tc39.es/ecma262/#sec-ContinueDynamicImport>
fn continue_dynamic_import(
    cx: &mut CurrentRealm,
    promise: Rc<Promise>,
    module_completion: Result<Rc<ModuleTree>, RethrowError>,
) {
    // Step 1. If moduleCompletion is an abrupt completion, then
    if let Err(exception) = module_completion {
        // a. Perform ! Call(promiseCapability.[[Reject]], undefined, « moduleCompletion.[[Value]] »).
        promise.reject(cx.into(), exception.handle(), CanGc::from_cx(cx));

        // b. Return unused.
        return;
    }

    let realm = CurrentRealm::assert(cx);
    let global = GlobalScope::from_current_realm(&realm);

    // Step 2. Let module be moduleCompletion.[[Value]].
    let module = module_completion.unwrap();
    let record = ModuleObject::new(module.get_record().map(|module| module.handle()).unwrap());

    // Step 3. Let loadPromise be module.LoadRequestedModules().
    let load_promise = load_requested_modules(cx, module, None);

    // Step 4. Let rejectedClosure be a new Abstract Closure with parameters (reason)
    // that captures promiseCapability and performs the following steps when called:
    // Step 5. Let onRejected be CreateBuiltinFunction(rejectedClosure, 1, "", « »).
    // Note: implemented by OnRejectedHandler.

    let global_scope = global.clone();
    let inner_promise = promise.clone();
    let fulfilled_promise = promise.clone();

    // Step 6. Let linkAndEvaluateClosure be a new Abstract Closure with no parameters that captures
    // module, promiseCapability, and onRejected and performs the following steps when called:
    // Step 7. Let linkAndEvaluate be CreateBuiltinFunction(linkAndEvaluateClosure, 0, "", « »).
    let link_and_evaluate = ModuleHandler::new_boxed(Box::new(
        task!(link_and_evaluate: |cx, global_scope: DomRoot<GlobalScope>, inner_promise: Rc<Promise>, record: ModuleObject| {
            let mut realm = AutoRealm::new(
                cx,
                std::ptr::NonNull::new(global_scope.reflector().get_jsobject().get()).unwrap(),
            );
            let in_realm_proof = (&mut realm.current_realm()).into();
            let cx = &mut *realm;
            // a. Let link be Completion(module.Link()).
            let link = unsafe { ModuleLink(cx, record.handle()) };

            // b. If link is an abrupt completion, then
            if !link {
                // i. Perform ! Call(promiseCapability.[[Reject]], undefined, « link.[[Value]] »).
                let exception = RethrowError::from_pending_exception(cx.into());
                inner_promise.reject(cx.into(), exception.handle(), CanGc::from_cx(cx));

                // ii. Return NormalCompletion(undefined).
                return;
            }

            rooted!(&in(cx) let mut rval = UndefinedValue());
            rooted!(&in(cx) let mut evaluate_promise = std::ptr::null_mut::<JSObject>());

            // c. Let evaluatePromise be module.Evaluate().
            assert!(unsafe { ModuleEvaluate(cx, record.handle(), rval.handle_mut()) });

            if !rval.is_object() {
                let error = RethrowError::from_pending_exception(cx.into());
                return inner_promise.reject(cx.into(), error.handle(), CanGc::from_cx(cx));
            }
            evaluate_promise.set(rval.to_object());
            let evaluate_promise = Promise::new_with_js_promise(evaluate_promise.handle(), cx.into());

            // d. Let fulfilledClosure be a new Abstract Closure with no parameters that captures
            // module and promiseCapability and performs the following steps when called:
            // e. Let onFulfilled be CreateBuiltinFunction(fulfilledClosure, 0, "", « »).
            let on_fulfilled = ModuleHandler::new_boxed(Box::new(
                task!(on_fulfilled: |cx, fulfilled_promise: Rc<Promise>, record: ModuleObject| {
                    rooted!(&in(cx) let mut rval: *mut JSObject = std::ptr::null_mut());
                    rooted!(&in(cx) let mut namespace = UndefinedValue());

                    // i. Let namespace be GetModuleNamespace(module).
                    rval.set(unsafe { GetModuleNamespace(cx, record.handle()) });
                    namespace.handle_mut().set(ObjectValue(rval.get()));

                    // ii. Perform ! Call(promiseCapability.[[Resolve]], undefined, « namespace »).
                    fulfilled_promise.resolve(cx.into(), namespace.handle(), CanGc::from_cx(cx));

                    // iii. Return NormalCompletion(undefined).
            })));

            // f. Perform PerformPromiseThen(evaluatePromise, onFulfilled, onRejected).
            let handler = PromiseNativeHandler::new(
                &global_scope,
                Some(on_fulfilled),
                Some(Box::new(OnRejectedHandler {
                    promise: inner_promise.clone(),
                })),
                CanGc::note(),
            );
            let in_realm = InRealm::Already(&in_realm_proof);
            evaluate_promise.append_native_handler(&handler, in_realm, CanGc::from_cx(cx));

            // g. Return unused.
        }),
    ));

    let mut realm = enter_auto_realm(cx, &*global);
    let mut realm = realm.current_realm();
    run_a_callback::<DomTypeHolder, _>(&*global, || {
        // Step 8. Perform PerformPromiseThen(loadPromise, linkAndEvaluate, onRejected).
        let handler = PromiseNativeHandler::new(
            &global,
            Some(link_and_evaluate),
            Some(Box::new(OnRejectedHandler {
                promise: promise.clone(),
            })),
            CanGc::from_cx(&mut realm),
        );
        let in_realm_proof = (&mut realm).into();
        let in_realm = InRealm::Already(&in_realm_proof);
        load_promise.append_native_handler(&handler, in_realm, CanGc::from_cx(&mut realm));
    });
    // Step 9. Return unused.
}

/// <https://html.spec.whatwg.org/multipage/#hostloadimportedmodule>
pub(crate) fn host_load_imported_module(
    cx: &mut CurrentRealm,
    referrer_module: Option<Rc<ModuleTree>>,
    referrer: RawHandleValue,
    specifier: String,
    module_type: ModuleType,
    load_state: Option<Rc<LoadState>>,
    payload: Payload,
) {
    // Step 1. Let settingsObject be the current settings object.
    let realm = CurrentRealm::assert(cx);
    let global_scope = GlobalScope::from_current_realm(&realm);

    // TODO Step 2. If settingsObject's global object implements WorkletGlobalScope or ServiceWorkerGlobalScope and loadState is undefined, then:

    // Step 3. Let referencingScript be null.
    // Step 6.1. Set referencingScript to referrer.[[HostDefined]].
    let referencing_script = unsafe { module_script_from_reference_private(&referrer) };

    // Step 6. If referrer is a Script Record or a Cyclic Module Record, then:
    let (original_fetch_options, fetch_referrer) = match referencing_script {
        Some(module) => (
            // Step 6.4. Set originalFetchOptions to referencingScript's fetch options.
            module.options.clone(),
            // Step 6.3. Set fetchReferrer to referencingScript's base URL.
            Referrer::ReferrerUrl(module.base_url.clone()),
        ),
        None => (
            // Step 4. Let originalFetchOptions be the default script fetch options.
            ScriptFetchOptions::default_classic_script(&global_scope),
            // Step 5. Let fetchReferrer be "client".
            global_scope.get_referrer(),
        ),
    };

    // Step 6.2. Set settingsObject to referencingScript's settings object.
    // Note: We later set fetchClient to the `ModuleOwner` provided by loadState,
    // which provides the `GlobalScope` that we will use for fetching.

    // Step 7 If referrer is a Cyclic Module Record and moduleRequest is equal to the first element of referrer.[[RequestedModules]], then:
    // Note: These substeps are implemented by `GetRequestedModuleSpecifier`,
    // setting loadState.[[ErrorToRethrow]] is done by `inner_module_loading`.

    // Step 8 Let url be the result of resolving a module specifier given referencingScript and moduleRequest.[[Specifier]],
    // catching any exceptions. If they throw an exception, let resolutionError be the thrown exception.
    let url = ModuleTree::resolve_module_specifier(
        &global_scope,
        referencing_script,
        DOMString::from_string(specifier.clone()),
    );

    // Step 9 If the previous step threw an exception, then:
    if let Err(error) = url {
        let resolution_error = gen_type_error(&global_scope, error, CanGc::from_cx(cx));

        // Step 9.1. If loadState is not undefined and loadState.[[ErrorToRethrow]] is null,
        // set loadState.[[ErrorToRethrow]] to resolutionError.
        load_state.as_ref().inspect(|load_state| {
            load_state
                .error_to_rethrow
                .borrow_mut()
                .get_or_insert(resolution_error.clone());
        });

        // Step 9.2. Perform FinishLoadingImportedModule(referrer, moduleRequest, payload, ThrowCompletion(resolutionError)).
        finish_loading_imported_module(
            cx,
            referrer_module,
            specifier,
            payload,
            Err(resolution_error),
        );

        // Step 9.3. Return.
        return;
    };

    let url = url.unwrap();

    // Step 10. Let fetchOptions be the result of getting the descendant script fetch options given
    // originalFetchOptions, url, and settingsObject.
    let fetch_options = original_fetch_options.descendant_fetch_options(&url, &global_scope);

    // Step 13. If loadState is not undefined, then:
    // Note: loadState is undefined only in dynamic imports
    let (destination, fetch_client) = match load_state.as_ref() {
        // Step 13.1. Set destination to loadState.[[Destination]].
        // Step 13.2. Set fetchClient to loadState.[[FetchClient]].
        Some(load_state) => (load_state.destination, load_state.fetch_client.clone()),
        None => (
            // Step 11. Let destination be "script".
            Destination::Script,
            // Step 12. Let fetchClient be settingsObject.
            ModuleOwner::DynamicModule(Trusted::new(&global_scope)),
        ),
    };

    let on_single_fetch_complete = move |module_tree: Option<Rc<ModuleTree>>| {
        let mut cx = unsafe { script_bindings::script_runtime::temp_cx() };
        let mut realm = CurrentRealm::assert(&mut cx);
        let cx = &mut realm;

        // Step 1. Let completion be null.
        let completion = match module_tree {
            // Step 2. If moduleScript is null, then set completion to ThrowCompletion(a new TypeError).
            None => Err(gen_type_error(
                &global_scope,
                Error::Type(c"Module fetching failed".to_owned()),
                CanGc::from_cx(cx),
            )),
            Some(module_tree) => {
                // Step 3. Otherwise, if moduleScript's parse error is not null, then:
                // Step 3.1 Let parseError be moduleScript's parse error.
                if let Some(parse_error) = module_tree.get_parse_error() {
                    // Step 3.3 If loadState is not undefined and loadState.[[ErrorToRethrow]] is null,
                    // set loadState.[[ErrorToRethrow]] to parseError.
                    load_state.as_ref().inspect(|load_state| {
                        load_state
                            .error_to_rethrow
                            .borrow_mut()
                            .get_or_insert(parse_error.clone());
                    });

                    // Step 3.2 Set completion to ThrowCompletion(parseError).
                    Err(parse_error.clone())
                } else {
                    // Step 4. Otherwise, set completion to NormalCompletion(moduleScript's record).
                    Ok(module_tree)
                }
            },
        };

        // Step 5. Perform FinishLoadingImportedModule(referrer, moduleRequest, payload, completion).
        finish_loading_imported_module(cx, referrer_module, specifier, payload, completion);
    };

    // Step 14 Fetch a single imported module script given url, fetchClient, destination, fetchOptions, settingsObject,
    // fetchReferrer, moduleRequest, and onSingleFetchComplete as defined below.
    // If loadState is not undefined and loadState.[[PerformFetch]] is not null, pass loadState.[[PerformFetch]] along as well.
    // Note: we don't have access to the requested `ModuleObject`, so we pass only its type.
    fetch_a_single_imported_module_script(
        url,
        fetch_client,
        destination,
        fetch_options,
        fetch_referrer,
        module_type,
        on_single_fetch_complete,
    );
}

/// <https://html.spec.whatwg.org/multipage/#fetch-a-single-imported-module-script>
fn fetch_a_single_imported_module_script(
    url: ServoUrl,
    owner: ModuleOwner,
    destination: Destination,
    options: ScriptFetchOptions,
    referrer: Referrer,
    module_type: ModuleType,
    on_complete: impl FnOnce(Option<Rc<ModuleTree>>) + 'static,
) {
    // TODO Step 1. Assert: moduleRequest.[[Attributes]] does not contain any Record entry such that entry.[[Key]] is not "type",
    // because we only asked for "type" attributes in HostGetSupportedImportAttributes.

    // TODO Step 2. Let moduleType be the result of running the module type from module request steps given moduleRequest.

    // Step 3. If the result of running the module type allowed steps given moduleType and settingsObject is false,
    // then run onComplete given null, and return.
    match module_type {
        ModuleType::Unknown => return on_complete(None),
        ModuleType::JavaScript | ModuleType::JSON => (),
    }

    // Step 4. Fetch a single module script given url, fetchClient, destination, options, settingsObject, referrer,
    // moduleRequest, false, and onComplete. If performFetch was given, pass it along as well.
    fetch_a_single_module_script(
        url,
        owner,
        destination,
        options,
        referrer,
        Some(module_type),
        false,
        Some(IntroductionType::IMPORTED_MODULE),
        on_complete,
    );
}

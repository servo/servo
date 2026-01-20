/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! An implementation of ecma262's [LoadRequestedModules](https://tc39.es/ecma262/#sec-LoadRequestedModules)
//! Partly inspired by mozjs implementation. Due to the inability to access ModuleObject internals (eg. ModuleRequest records),
//! this implementation deviates from the spec in some aspects.

#![expect(non_snake_case, unsafe_code)]

use std::cell::{Cell, RefCell};
use std::collections::HashSet;
use std::rc::Rc;

use js::conversions::jsstr_to_string;
use js::jsapi::{
    GetModuleNamespace, GetRequestedModuleSpecifier, GetRequestedModulesCount,
    HandleValue as RawHandleValue, IsCyclicModule, JSObject, ModuleEvaluate,
};
use js::jsval::{ObjectValue, UndefinedValue};
use js::realm::CurrentRealm;
use js::rust::wrappers::JS_GetModulePrivate;
use js::rust::{HandleObject, HandleValue, IntoHandle};
use net_traits::request::{Destination, Referrer};
use script_bindings::str::DOMString;
use servo_url::ServoUrl;

use crate::dom::bindings::error::Error;
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::settings_stack::AutoIncumbentScript;
use crate::dom::globalscope::GlobalScope;
use crate::dom::promise::Promise;
use crate::dom::promisenativehandler::{Callback, PromiseNativeHandler};
use crate::realms::{AlreadyInRealm, InRealm, enter_realm};
use crate::script_module::{
    ModuleHandler, ModuleObject, ModuleOwner, ModuleTree, RethrowError, ScriptFetchOptions,
    fetch_a_single_module_script, gen_type_error, module_script_from_reference_private,
};
use crate::script_runtime::{CanGc, IntroductionType, JSContext as SafeJSContext};

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
    promise: Rc<Promise>,
    is_loading: Cell<bool>,
    pending_modules_count: Cell<u32>,
    visited: RefCell<HashSet<ServoUrl>>,
    load_state: Option<Rc<LoadState>>,
}

/// <https://tc39.es/ecma262/#sec-LoadRequestedModules>
pub(crate) fn LoadRequestedModules(
    global: &GlobalScope,
    module: Rc<ModuleTree>,
    load_state: Option<Rc<LoadState>>,
) -> Rc<Promise> {
    // Step 1. If hostDefined is not present, let hostDefined be empty.

    // Step 2. Let pc be ! NewPromiseCapability(%Promise%).
    let promise = Promise::new(global, CanGc::note());

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
    InnerModuleLoading(global, &Rc::new(state), module);

    // Step 5. Return pc.[[Promise]].
    promise
}

/// <https://tc39.es/ecma262/#sec-InnerModuleLoading>
fn InnerModuleLoading(global: &GlobalScope, state: &Rc<GraphLoadingState>, module: Rc<ModuleTree>) {
    let cx = GlobalScope::get_cx();

    // Step 1. Assert: state.[[IsLoading]] is true.
    assert!(state.is_loading.get());

    let module_handle = module
        .get_record()
        .borrow()
        .as_ref()
        .map(|module| module.handle())
        .unwrap();

    let module_url = module.get_url();
    let visited_contains_module = state.visited.borrow().contains(&module_url);

    // Step 2. If module is a Cyclic Module Record, module.[[Status]] is new, and state.[[Visited]] does not contain module, then
    if unsafe { IsCyclicModule(module_handle.get()) } && !visited_contains_module {
        // a. Append module to state.[[Visited]].
        state.visited.borrow_mut().insert(module_url);

        // b. Let requestedModulesCount be the number of elements in module.[[RequestedModules]].
        let requested_modules_count = unsafe { GetRequestedModulesCount(*cx, module_handle) };

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
            let jsstr = unsafe { GetRequestedModuleSpecifier(*cx, module_handle, index) };

            if jsstr.is_null() {
                // 1. Let error be ThrowCompletion(a newly created SyntaxError object).
                let error = RethrowError::from_pending_exception(cx);

                // 2. Perform ContinueModuleLoading(state, error).
                ContinueModuleLoading(global, state, Err(error));
            } else {
                let specifier =
                    unsafe { jsstr_to_string(*cx, std::ptr::NonNull::new(jsstr).unwrap()) };

                // ii. Else if module.[[LoadedModules]] contains a LoadedModuleRequest Record record
                // such that ModuleRequestsEqual(record, request) is true, then
                let loaded_module = module.find_descendant_inside_module_map(global, &specifier);

                match loaded_module {
                    // 1. Perform InnerModuleLoading(state, record.[[Module]]).
                    Some(module) => InnerModuleLoading(global, state, module),
                    // 1. Perform HostLoadImportedModule(module, request, state.[[HostDefined]], state).
                    None => {
                        rooted!(in(*cx) let mut referrer = UndefinedValue());
                        unsafe { JS_GetModulePrivate(module_handle.get(), referrer.handle_mut()) };

                        HostLoadImportedModule(
                            cx,
                            Some(module.clone()),
                            referrer.handle().into_handle(),
                            specifier,
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

        // c. Perform ! Call(state.[[PromiseCapability]].[[Resolve]], undefined, « undefined »).
        state.promise.resolve_native(&(), CanGc::note());
    }

    // Step 6. Return unused.
}

/// <https://tc39.es/ecma262/#sec-ContinueModuleLoading>
fn ContinueModuleLoading(
    global: &GlobalScope,
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
        Ok(module) => InnerModuleLoading(global, state, module),

        // Step 3. Else,
        Err(exception) => {
            // a. Set state.[[IsLoading]] to false.
            state.is_loading.set(false);

            // b. Perform ! Call(state.[[PromiseCapability]].[[Reject]], undefined, « moduleCompletion.[[Value]] »).
            state
                .promise
                .reject(GlobalScope::get_cx(), exception.handle(), CanGc::note());
        },
    }

    // Step 4. Return unused.
}

/// <https://tc39.es/ecma262/#sec-FinishLoadingImportedModule>
fn FinishLoadingImportedModule(
    global: &GlobalScope,
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
                // a. If referrer.[[LoadedModules]] contains a LoadedModuleRequest Record record such that
                // ModuleRequestsEqual(record, moduleRequest) is true, then
                module_tree.insert_module_dependency(module, module_request_specifier);
            }

            // a. Perform ContinueModuleLoading(payload, result).
            ContinueModuleLoading(global, &state, result);
        },

        // Step 3. Else,
        // a. Perform ContinueDynamicImport(payload, result).
        Payload::PromiseRecord(promise) => ContinueDynamicImport(global, promise, result),
    }

    // 4. Return unused.
}

/// <https://tc39.es/ecma262/#sec-ContinueDynamicImport>
fn ContinueDynamicImport(
    global: &GlobalScope,
    promise: Rc<Promise>,
    module_completion: Result<Rc<ModuleTree>, RethrowError>,
) {
    let cx = GlobalScope::get_cx();

    // Step 1. If moduleCompletion is an abrupt completion, then
    if let Err(exception) = module_completion {
        // a. Perform ! Call(promiseCapability.[[Reject]], undefined, « moduleCompletion.[[Value]] »).
        promise.reject(cx, exception.handle(), CanGc::note());

        // b. Return unused.
        return;
    }

    // Step 2. Let module be moduleCompletion.[[Value]].
    let module = module_completion.unwrap();
    let module_handle = module
        .get_record()
        .borrow()
        .as_ref()
        .map(|module| module.handle())
        .unwrap();

    // Step 3. Let loadPromise be module.LoadRequestedModules().
    let load_promise = LoadRequestedModules(global, module, None);

    let realm = enter_realm(global);
    let comp = InRealm::Entered(&realm);
    let _ais = AutoIncumbentScript::new(global);

    // Step 4. Let rejectedClosure be a new Abstract Closure with parameters (reason)
    // that captures promiseCapability and performs the following steps when called:
    // Step 5. Let onRejected be CreateBuiltinFunction(rejectedClosure, 1, "", « »).
    // Note: implemented by OnRejectedHandler.

    let global_scope = DomRoot::from_ref(global);
    let inner_promise = promise.clone();
    let fulfilled_promise = promise.clone();
    let record = ModuleObject::new(unsafe { HandleObject::from_raw(module_handle) });

    // Step 6. Let linkAndEvaluateClosure be a new Abstract Closure with no parameters that captures
    // module, promiseCapability, and onRejected and performs the following steps when called:
    // Step 7. Let linkAndEvaluate be CreateBuiltinFunction(linkAndEvaluateClosure, 0, "", « »).
    let link_and_evaluate = ModuleHandler::new_boxed(Box::new(
        task!(link_and_evaluate: |global_scope: DomRoot<GlobalScope>, inner_promise: Rc<Promise>, record: ModuleObject| {
            let cx = GlobalScope::get_cx();

            // a. Let link be Completion(module.Link()).
            let link = ModuleTree::instantiate_module_tree(&global_scope, record.handle());

            // b. If link is an abrupt completion, then
            if let Err(exception) = link {
                // i. Perform ! Call(promiseCapability.[[Reject]], undefined, « link.[[Value]] »).
                inner_promise.reject(cx, exception.handle(), CanGc::note());

                // ii. Return NormalCompletion(undefined).
                return;
            }

            rooted!(in(*cx) let mut rval = UndefinedValue());
            rooted!(in(*cx) let mut evaluate_promise = std::ptr::null_mut::<JSObject>());

            // c. Let evaluatePromise be module.Evaluate().
            assert!(unsafe { ModuleEvaluate(*cx, record.handle(), rval.handle_mut().into()) });

            if !rval.is_object() {
                let error = RethrowError::from_pending_exception(cx);
                return inner_promise.reject(cx, error.handle(), CanGc::note());
            }
            evaluate_promise.set(rval.to_object());
            let evaluate_promise = Promise::new_with_js_promise(evaluate_promise.handle(), cx);

            // d. Let fulfilledClosure be a new Abstract Closure with no parameters that captures
            // module and promiseCapability and performs the following steps when called:
            // e. Let onFulfilled be CreateBuiltinFunction(fulfilledClosure, 0, "", « »).
            let on_fulfilled = ModuleHandler::new_boxed(Box::new(
                task!(on_fulfilled: |fulfilled_promise: Rc<Promise>, record: ModuleObject| {
                    let cx = GlobalScope::get_cx();
                    rooted!(in(*cx) let mut rval: *mut JSObject = std::ptr::null_mut());
                    rooted!(in(*cx) let mut namespace = UndefinedValue());

                    // i. Let namespace be GetModuleNamespace(module).
                    rval.set(unsafe { GetModuleNamespace(*cx, record.handle()) });
                    namespace.handle_mut().set(ObjectValue(rval.get()));

                    // ii. Perform ! Call(promiseCapability.[[Resolve]], undefined, « namespace »).
                    fulfilled_promise.resolve(cx, namespace.handle(), CanGc::note());

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
            let realm = enter_realm(&*global_scope);
            let comp = InRealm::Entered(&realm);
            evaluate_promise.append_native_handler(&handler, comp, CanGc::note());

            // g. Return unused.
        }),
    ));

    // Step 8. Perform PerformPromiseThen(loadPromise, linkAndEvaluate, onRejected).
    let handler = PromiseNativeHandler::new(
        global,
        Some(link_and_evaluate),
        Some(Box::new(OnRejectedHandler {
            promise: promise.clone(),
        })),
        CanGc::note(),
    );
    load_promise.append_native_handler(&handler, comp, CanGc::note());

    // Step 9. Return unused.
}

/// <https://html.spec.whatwg.org/multipage/#hostloadimportedmodule>
pub(crate) fn HostLoadImportedModule(
    cx: SafeJSContext,
    referrer_module: Option<Rc<ModuleTree>>,
    referrer: RawHandleValue,
    specifier: String,
    load_state: Option<Rc<LoadState>>,
    payload: Payload,
) {
    // Step 1. Let settingsObject be the current settings object.
    let in_realm_proof = AlreadyInRealm::assert_for_cx(cx);
    let global_scope = unsafe { GlobalScope::from_context(*cx, InRealm::Already(&in_realm_proof)) };

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

    // TODO It seems that Gecko doesn't implement this step, and currently we don't handle module types.
    // Step 7 If referrer is a Cyclic Module Record and moduleRequest is equal to the first element of referrer.[[RequestedModules]], then:

    // Step 8 Let url be the result of resolving a module specifier given referencingScript and moduleRequest.[[Specifier]],
    // catching any exceptions. If they throw an exception, let resolutionError be the thrown exception.
    let url = ModuleTree::resolve_module_specifier(
        &global_scope,
        referencing_script,
        DOMString::from_string(specifier.clone()),
        CanGc::note(),
    );

    // Step 9 If the previous step threw an exception, then:
    if let Err(error) = url {
        let resolution_error = gen_type_error(&global_scope, error, CanGc::note());

        // Step 9.1. If loadState is not undefined and loadState.[[ErrorToRethrow]] is null,
        // set loadState.[[ErrorToRethrow]] to resolutionError.
        if let Some(load_state) = load_state {
            load_state
                .error_to_rethrow
                .borrow_mut()
                .get_or_insert(resolution_error.clone());
        }

        // Step 9.2. Perform FinishLoadingImportedModule(referrer, moduleRequest, payload, ThrowCompletion(resolutionError)).
        FinishLoadingImportedModule(
            &global_scope,
            referrer_module,
            specifier,
            payload,
            Err(resolution_error),
        );

        // Step 9.3. Return.
        return;
    };

    // Step 10. Let fetchOptions be the result of getting the descendant script fetch options given
    // originalFetchOptions, url, and settingsObject.
    let fetch_options = original_fetch_options.descendant_fetch_options();

    // Step 11. Let destination be "script".
    // Step 12. Let fetchClient be settingsObject.
    // Step 13. If loadState is not undefined, then:
    let (destination, fetch_client) = match load_state.as_ref() {
        // Step 13.1. Set destination to loadState.[[Destination]].
        // Step 13.2. Set fetchClient to loadState.[[FetchClient]].
        Some(load_state) => (load_state.destination, load_state.fetch_client.clone()),
        None => (
            Destination::Script,
            ModuleOwner::new_dynamic(&global_scope, CanGc::note()),
        ),
    };

    let on_single_fetch_complete = move |global: &GlobalScope, module_tree: Rc<ModuleTree>| {
        // Step 1. Let completion be null.
        // Step 2. If moduleScript is null, then set completion to ThrowCompletion(a new TypeError).
        let completion = if module_tree.get_network_error().borrow().is_some() {
            Err(gen_type_error(
                global,
                Error::Type("Module fetching failed".to_string()),
                CanGc::note(),
            ))
        } else {
            // Step 3. Otherwise, if moduleScript's parse error is not null, then:
            // Step 3.1 Let parseError be moduleScript's parse error.
            if let Some(parse_error) = module_tree.get_parse_error().borrow().as_ref() {
                // Step 3.3 If loadState is not undefined and loadState.[[ErrorToRethrow]] is null,
                // set loadState.[[ErrorToRethrow]] to parseError.
                if let Some(load_state) = load_state {
                    load_state
                        .error_to_rethrow
                        .borrow_mut()
                        .get_or_insert(parse_error.clone());
                }

                // Step 3.2 Set completion to ThrowCompletion(parseError).
                Err(parse_error.clone())
            } else {
                assert!(
                    module_tree
                        .get_record()
                        .borrow()
                        .as_ref()
                        .is_some_and(|record| !record.handle().is_null())
                );
                // Step 4. Otherwise, set completion to NormalCompletion(moduleScript's record).
                Ok(module_tree)
            }
        };

        // Step 5. Perform FinishLoadingImportedModule(referrer, moduleRequest, payload, completion).
        FinishLoadingImportedModule(global, referrer_module, specifier, payload, completion);
    };

    // Step 14 Fetch a single imported module script given url, fetchClient, destination, fetchOptions, settingsObject,
    // fetchReferrer, moduleRequest, and onSingleFetchComplete as defined below.
    // If loadState is not undefined and loadState.[[PerformFetch]] is not null, pass loadState.[[PerformFetch]] along as well.
    fetch_a_single_imported_module_script(
        url.unwrap(),
        fetch_client,
        destination,
        fetch_options,
        fetch_referrer,
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
    on_complete: impl FnOnce(&GlobalScope, Rc<ModuleTree>) + 'static,
) {
    // TODO Step 1. Assert: moduleRequest.[[Attributes]] does not contain any Record entry such that entry.[[Key]] is not "type",
    // because we only asked for "type" attributes in HostGetSupportedImportAttributes.

    // TODO Step 2. Let moduleType be the result of running the module type from module request steps given moduleRequest.

    // TODO Step 3. If the result of running the module type allowed steps given moduleType and settingsObject is false,
    // then run onComplete given null, and return.

    // Step 4. Fetch a single module script given url, fetchClient, destination, options, settingsObject, referrer,
    // moduleRequest, false, and onComplete. If performFetch was given, pass it along as well.
    fetch_a_single_module_script(
        url,
        owner,
        destination,
        options,
        referrer,
        false,
        Some(IntroductionType::IMPORTED_MODULE),
        on_complete,
    );
}

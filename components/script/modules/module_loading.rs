/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! An implementation of ecma262's [LoadRequestedModules](https://tc39.es/ecma262/#sec-LoadRequestedModules)
//! Partly inspired by mozjs implementation: <https://searchfox.org/firefox-main/source/js/src/vm/Modules.cpp#1450>
//! Since we can't access ModuleObject internals (eg. ModuleRequest records), we deviate from the spec in some aspects.

#![expect(unsafe_code)]

use std::cell::RefCell;
use std::ffi::c_void;
use std::ptr;
use std::rc::Rc;

use js::context::{JSContext, RawJSContext};
use js::jsapi::{
    CallArgs, GetErrorType, GetFunctionNativeReserved, Heap, JS_GetFunctionObject, JSExnType,
    JSObject, JSScript, ModuleType, SetFunctionNativeReserved,
};
use js::jsval::{JSVal, ObjectValue, PrivateValue, UndefinedValue};
use js::realm::CurrentRealm;
use js::rust::Handle;
use js::rust::wrappers2::{
    AddPromiseReactions, FinishLoadingDynamicImportedModule, FinishLoadingImportedModule,
    FinishLoadingImportedModuleFailed, GetModuleNamespace, GetModuleRequestType, IsPromiseObject,
    JS_GetScriptPrivate, LoadRequestedModules1, ModuleEvaluate, ModuleLink,
    NewFunctionWithReserved,
};
use net_traits::blob_url_store::UrlWithBlobClaim;
use net_traits::request::{Destination, Referrer, RequestClient};
use script_bindings::cell::DomRefCell;
use script_bindings::settings_stack::run_a_callback;

use crate::DomTypeHolder;
use crate::dom::bindings::error::Error;
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::trace::RootedTraceableBox;
use crate::dom::globalscope::GlobalScope;
use crate::dom::promise::Promise;
use crate::dom::promise::promisenativehandler::{Callback, PromiseNativeHandler};
use crate::modules::script_module::{
    ModuleHandler, ModuleObject, ModuleTree, RethrowError, ScriptFetchOptions,
    fetch_a_single_module_script, gen_type_error, module_script_from_reference_private,
};
use crate::realms::enter_auto_realm;
use crate::runtime::script_runtime::IntroductionType;
use crate::url::ensure_blob_referenced_by_url_is_kept_alive;

#[derive(JSTraceable, MallocSizeOf)]
struct OnRejectedHandler {
    #[conditional_malloc_size_of]
    promise: Rc<Promise>,
}

impl Callback for OnRejectedHandler {
    fn callback(&self, cx: &mut CurrentRealm, v: Handle<JSVal>) {
        // a. Perform ! Call(promiseCapability.[[Reject]], undefined, « reason »).
        self.promise.reject(cx, v);
    }
}

#[derive(JSTraceable, MallocSizeOf)]
pub(crate) struct LoadState {
    #[ignore_malloc_size_of = "mozjs"]
    pub(crate) error_to_rethrow: RefCell<Option<RethrowError>>,
    #[no_trace]
    pub(crate) destination: Destination,
    #[no_trace]
    pub(crate) fetch_client: RequestClient,
    #[conditional_malloc_size_of]
    pub(crate) module_script: DomRefCell<Option<Rc<ModuleTree>>>,
    #[ignore_malloc_size_of = "Measuring trait objects is hard"]
    #[no_trace]
    #[allow(clippy::type_complexity)]
    pub(crate) on_complete:
        DomRefCell<Option<Box<dyn FnOnce(&mut JSContext, Option<Rc<ModuleTree>>)>>>,
}

const LOAD_REACTION_HOST_DEFINED_SLOT: usize = 0;

fn take_state_from_reserved_slot(cx: &mut JSContext, args: &CallArgs) -> Box<LoadState> {
    rooted!(&in(cx) let host_defined = unsafe { *GetFunctionNativeReserved(args.callee(), LOAD_REACTION_HOST_DEFINED_SLOT) });
    unsafe {
        SetFunctionNativeReserved(
            args.callee(),
            LOAD_REACTION_HOST_DEFINED_SLOT,
            &UndefinedValue(),
        )
    };
    assert!(!host_defined.get().is_undefined());
    unsafe { Box::from_raw((*host_defined).to_private() as *mut LoadState) }
}

unsafe extern "C" fn on_load_requested_modules_resolved(
    cx: *mut RawJSContext,
    argc: u32,
    vp: *mut JSVal,
) -> bool {
    // SAFETY: it is safe to construct a JSContext from engine hook.
    let mut cx = unsafe { JSContext::from_ptr(ptr::NonNull::new(cx).unwrap()) };
    let mut realm = CurrentRealm::assert(&mut cx);
    let cx = &mut realm;

    let args = unsafe { CallArgs::from_vp(vp, argc) };

    let state = take_state_from_reserved_slot(cx, &args);

    let on_complete = state.on_complete.safe_borrow_mut(cx).take().unwrap();
    let module_script = state.module_script.safe_borrow_mut(cx).take().unwrap();

    let record = module_script
        .get_record()
        .map(|module| module.handle())
        .unwrap();

    // https://html.spec.whatwg.org/multipage/#fetch-the-descendants-of-and-link-a-module-script
    // Step 6. Upon fulfillment of loadingPromise, run the following steps:

    // Step 6.1. Perform record.Link().
    let link = unsafe { ModuleLink(cx, record) };

    // If this throws an exception, catch it, and set moduleScript's error to rethrow to that exception.
    if !link {
        let exception = RethrowError::from_pending_exception(cx);
        module_script.set_rethrow_error(exception);
    }

    // Step 6.2. Run onComplete given moduleScript.
    on_complete(cx, Some(module_script));

    true
}

unsafe extern "C" fn on_load_requested_modules_rejected(
    cx: *mut RawJSContext,
    argc: u32,
    vp: *mut JSVal,
) -> bool {
    // SAFETY: it is safe to construct a JSContext from engine hook.
    let mut cx = unsafe { JSContext::from_ptr(ptr::NonNull::new(cx).unwrap()) };
    let mut realm = CurrentRealm::assert(&mut cx);
    let cx = &mut realm;

    let args = unsafe { CallArgs::from_vp(vp, argc) };

    let state = take_state_from_reserved_slot(cx, &args);

    let error = unsafe { Handle::from_raw(args.get(0)) };

    let on_complete = state.on_complete.safe_borrow_mut(cx).take().unwrap();
    let module_script = state.module_script.safe_borrow_mut(cx).take().unwrap();

    // https://html.spec.whatwg.org/multipage/#fetch-the-descendants-of-and-link-a-module-script
    // Step 7. Upon rejection of loadingPromise, run the following steps:

    // Note: When the error is thrown on the SpiderMonkey side, which happens when encountering
    // an unsupported attribute inside `InnerModuleLoading`, state.[[ErrorToRethrow]] doesn't
    // contain the expected error, we need to use the value passed down.
    if unsafe { GetErrorType(error.as_ref(cx)) } == JSExnType::JSEXN_SYNTAXERR {
        module_script.set_rethrow_error(RethrowError::new(Heap::boxed(*error.as_ref(cx))));
        on_complete(cx, Some(module_script));
        return true;
    }

    // Step 7.1. If state.[[ErrorToRethrow]] is not null, set moduleScript's error to rethrow to
    // state.[[ErrorToRethrow]] and run onComplete given moduleScript.
    let error_to_rethrow = state.error_to_rethrow.borrow().as_ref().cloned();
    if let Some(error) = error_to_rethrow {
        module_script.set_rethrow_error(error);
        on_complete(cx, Some(module_script));
    } else {
        // Step 7.2. Otherwise, run onComplete given null.
        on_complete(cx, None);
    }

    true
}

struct ImportRequest {
    referrer: RootedTraceableBox<Heap<*mut JSScript>>,
    module_request: RootedTraceableBox<Heap<*mut JSObject>>,
    payload: RootedTraceableBox<Heap<JSVal>>,
    host_defined: RootedTraceableBox<Heap<JSVal>>,
}

fn load_state_from_handle_value<'a>(reference_private: Handle<'a, JSVal>) -> Option<&'a LoadState> {
    if reference_private.get().is_undefined() {
        return None;
    }
    unsafe { (reference_private.get().to_private() as *const LoadState).as_ref() }
}

pub(crate) fn load_requested_modules(
    cx: &mut CurrentRealm,
    module: Handle<*mut JSObject>,
    load_state: Box<LoadState>,
) {
    rooted!(&in(cx) let host_defined = PrivateValue(Box::into_raw(load_state) as *const _ as *const c_void));

    unsafe {
        let on_resolved = NewFunctionWithReserved(
            cx,
            Some(on_load_requested_modules_resolved),
            0,
            0,
            ptr::null(),
        );
        let on_rejected = NewFunctionWithReserved(
            cx,
            Some(on_load_requested_modules_rejected),
            1,
            0,
            ptr::null(),
        );

        rooted!(&in(cx) let resolved_function_object = JS_GetFunctionObject(on_resolved));
        SetFunctionNativeReserved(
            resolved_function_object.get(),
            LOAD_REACTION_HOST_DEFINED_SLOT,
            host_defined.handle().as_ref(cx),
        );

        rooted!(&in(cx) let rejected_function_object = JS_GetFunctionObject(on_rejected));
        SetFunctionNativeReserved(
            rejected_function_object.get(),
            LOAD_REACTION_HOST_DEFINED_SLOT,
            host_defined.handle().as_ref(cx),
        );

        rooted!(&in(cx) let mut promise_obj = ptr::null_mut::<JSObject>());
        assert!(LoadRequestedModules1(
            cx,
            module,
            host_defined.handle(),
            promise_obj.handle_mut(),
        ));

        AddPromiseReactions(
            cx,
            promise_obj.handle(),
            resolved_function_object.handle(),
            rejected_function_object.handle(),
        );
    }
}

/// <https://tc39.es/ecma262/#sec-FinishLoadingImportedModule>
fn finish_loading_imported_module(
    cx: &mut CurrentRealm,
    referrer: Handle<*mut JSScript>,
    module_request: Handle<*mut JSObject>,
    payload: Handle<JSVal>,
    result: Result<Rc<ModuleTree>, RethrowError>,
) {
    match result {
        Ok(module_tree) => {
            let module_handle = module_tree
                .get_record()
                .map(|module| module.handle())
                .unwrap();

            if payload.is_object() {
                rooted!(&in(cx) let object = payload.to_object());
                let is_promise = unsafe { IsPromiseObject(object.handle()) };

                if is_promise {
                    unsafe {
                        FinishLoadingDynamicImportedModule(
                            cx,
                            referrer,
                            module_request,
                            payload,
                            module_handle,
                        )
                    };
                    let promise = Promise::new_with_js_promise(cx, object.handle());
                    let record = ModuleObject::new(module_handle);
                    return continue_dynamic_import(cx, promise, record);
                }
            }

            assert!(unsafe {
                FinishLoadingImportedModule(
                    cx,
                    referrer,
                    module_request,
                    payload,
                    module_handle,
                    true,
                )
            });
        },
        Err(error) => {
            unsafe { FinishLoadingImportedModuleFailed(cx, payload, error.handle()) };
        },
    }
}

/// <https://tc39.es/ecma262/#sec-ContinueDynamicImport>
fn continue_dynamic_import(realm: &mut CurrentRealm, promise: Rc<Promise>, module: ModuleObject) {
    // Step 1. If moduleCompletion is an abrupt completion, then
    // a. Perform ! Call(promiseCapability.[[Reject]], undefined, « moduleCompletion.[[Value]] »).
    // b. Return unused.
    // Note: Done inside `finish_loading_imported_module`

    let global = GlobalScope::from_current_realm(realm);

    // Step 2. Let module be moduleCompletion.[[Value]].

    rooted!(&in(*realm) let host_defined = UndefinedValue());
    rooted!(&in(*realm) let mut promise_obj = ptr::null_mut::<JSObject>());

    // Step 3. Let loadPromise be module.LoadRequestedModules().
    unsafe {
        LoadRequestedModules1(
            realm,
            module.handle(),
            host_defined.handle(),
            promise_obj.handle_mut(),
        )
    };

    let load_promise = Promise::new_with_js_promise(realm, promise_obj.handle());

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
        task!(link_and_evaluate: |cx, global_scope: DomRoot<GlobalScope>, inner_promise: Rc<Promise>, module: ModuleObject| {
            let mut realm = enter_auto_realm(cx, &*global_scope);
            let cx = &mut realm.current_realm();

            // a. Let link be Completion(module.Link()).
            let link = unsafe { ModuleLink(cx, module.handle()) };

            // b. If link is an abrupt completion, then
            if !link {
                // i. Perform ! Call(promiseCapability.[[Reject]], undefined, « link.[[Value]] »).
                let exception = RethrowError::from_pending_exception(cx);
                inner_promise.reject(cx, exception.handle());

                // ii. Return NormalCompletion(undefined).
                return;
            }

            rooted!(&in(cx) let mut rval = UndefinedValue());

            // c. Let evaluatePromise be module.Evaluate().
            assert!(unsafe { ModuleEvaluate(cx, module.handle(), rval.handle_mut()) });

            if !rval.is_object() {
                let error = RethrowError::from_pending_exception(cx);
                return inner_promise.reject(cx, error.handle());
            }

            rooted!(&in(cx) let evaluate_promise = rval.to_object());
            let evaluate_promise = Promise::new_with_js_promise(cx, evaluate_promise.handle());

            // d. Let fulfilledClosure be a new Abstract Closure with no parameters that captures
            // module and promiseCapability and performs the following steps when called:
            // e. Let onFulfilled be CreateBuiltinFunction(fulfilledClosure, 0, "", « »).
            let on_fulfilled = ModuleHandler::new_boxed(Box::new(
                task!(on_fulfilled: |cx, fulfilled_promise: Rc<Promise>, module: ModuleObject| {

                    // i. Let namespace be GetModuleNamespace(module).
                    rooted!(&in(cx) let rval = unsafe { GetModuleNamespace(cx, module.handle()) });
                    rooted!(&in(cx) let namespace = ObjectValue(rval.get()));

                    // ii. Perform ! Call(promiseCapability.[[Resolve]], undefined, « namespace »).
                    fulfilled_promise.resolve(cx, namespace.handle());

                    // iii. Return NormalCompletion(undefined).
            })));

            // f. Perform PerformPromiseThen(evaluatePromise, onFulfilled, onRejected).
            let handler = PromiseNativeHandler::new(
                cx,
                &global_scope,
                Some(on_fulfilled),
                Some(Box::new(OnRejectedHandler { promise: inner_promise }))
            );
            evaluate_promise.append_native_handler(cx, &handler);

            // g. Return unused.
        }),
    ));

    run_a_callback::<DomTypeHolder, _>(&*global, || {
        // Step 8. Perform PerformPromiseThen(loadPromise, linkAndEvaluate, onRejected).
        let handler = PromiseNativeHandler::new(
            realm,
            &global,
            Some(link_and_evaluate),
            Some(Box::new(OnRejectedHandler { promise })),
        );
        load_promise.append_native_handler(realm, &handler);
    });
    // Step 9. Return unused.
}

/// <https://html.spec.whatwg.org/multipage/#hostloadimportedmodule>
pub(crate) fn host_load_imported_module(
    cx: &mut CurrentRealm,
    referrer: Handle<*mut JSScript>,
    module_request: Handle<*mut JSObject>,
    specifier: String,
    host_defined: Handle<JSVal>,
    payload: Handle<JSVal>,
) {
    // Step 1. Let settingsObject be the current settings object.
    let mut realm = CurrentRealm::assert(cx);
    let mut global_scope = GlobalScope::from_current_realm(&mut realm);

    let load_state = load_state_from_handle_value(host_defined);

    // TODO Step 2. If settingsObject's global object implements WorkletGlobalScope or ServiceWorkerGlobalScope and loadState is undefined, then:

    // Step 3. Let referencingScript be null.
    rooted!(&in(cx) let mut script_private = UndefinedValue());

    // Step 6.1. Set referencingScript to referrer.[[HostDefined]].
    unsafe { JS_GetScriptPrivate(*referrer.as_ref(cx), script_private.handle_mut()) };
    let referencing_script =
        unsafe { module_script_from_reference_private(script_private.handle()) };

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
            ScriptFetchOptions::default_classic_script(),
            // Step 5. Let fetchReferrer be "client".
            global_scope.get_referrer(),
        ),
    };

    // TODO: investigate providing a `ModuleOwner` to classic scripts.
    let script_owner = referencing_script.and_then(|script| script.owner.clone());

    // Step 6.2. Set settingsObject to referencingScript's settings object.
    if let Some(ref owner) = script_owner {
        global_scope = owner.root();
    }

    let global = &global_scope.clone();

    // Step 7. If referrer is a Cyclic Module Record and moduleRequest is equal to the first element of referrer.[[RequestedModules]], then:
    // Note: Spidermonkey removed the API for iterating through a module's requested modules,
    // preventing upfront validation.
    // Additionally, we skip step 7.1.1 (handled internally by Spidermonkey in `InnerModuleLoading`),
    // as well as steps 7.1.2 and 7.1.3 (executed later in steps 8 and 9).

    // Step 7.1.4. Let moduleType be the result of running the module type from module request steps given requested.
    let module_type = unsafe { GetModuleRequestType(cx, module_request) };

    // Step 7.1.5. If the result of running the module type allowed steps given moduleType and settingsObject is false:
    if let ModuleType::Unknown = module_type {
        // Step 7.1.5.1. Let error be a new TypeError exception.
        let error = gen_type_error(
            cx,
            global,
            Error::Type(c"Found invalid module type attribute".to_owned()),
        );

        // Step 7.1.5.2. If loadState is not undefined and loadState.[[ErrorToRethrow]] is null, set
        // loadState.[[ErrorToRethrow]] to error.
        if let Some(load_state) = load_state {
            load_state
                .error_to_rethrow
                .borrow_mut()
                .get_or_insert(error.clone());
        }

        // Step 7.1.5.3. Perform FinishLoadingImportedModule(referrer, moduleRequest, payload, ThrowCompletion(error)).
        finish_loading_imported_module(cx, referrer, module_request, payload, Err(error));

        // Step 7.1.5.4. Return.
        return;
    }

    // Step 8 Let url be the result of resolving a module specifier given referencingScript and moduleRequest.[[Specifier]],
    // catching any exceptions. If they throw an exception, let resolutionError be the thrown exception.
    let url = ModuleTree::resolve_module_specifier(global, referencing_script, specifier);

    // Step 9 If the previous step threw an exception, then:
    if let Err(error) = url {
        let resolution_error = gen_type_error(cx, &global_scope, error);

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
            referrer,
            module_request,
            payload,
            Err(resolution_error),
        );

        // Step 9.3. Return.
        return;
    };

    let url = ensure_blob_referenced_by_url_is_kept_alive(global, url.unwrap());

    // Step 10. Let fetchOptions be the result of getting the descendant script fetch options given
    // originalFetchOptions, url, and settingsObject.
    let fetch_options = original_fetch_options.descendant_fetch_options(&url.url(), &global_scope);

    // Step 13. If loadState is not undefined, then:
    // Note: loadState is undefined only in dynamic imports
    let (destination, fetch_client) = match load_state {
        // Step 13.1. Set destination to loadState.[[Destination]].
        // Step 13.2. Set fetchClient to loadState.[[FetchClient]].
        Some(load_state) => (load_state.destination, load_state.fetch_client.clone()),
        None => (
            // Step 11. Let destination be "script".
            Destination::Script,
            // Step 12. Let fetchClient be settingsObject.
            global_scope.request_client(Some(cx.no_gc())),
        ),
    };

    let request = ImportRequest {
        referrer: RootedTraceableBox::from_box(Heap::boxed(*referrer.as_ref(cx))),
        module_request: RootedTraceableBox::from_box(Heap::boxed(*module_request.as_ref(cx))),
        payload: RootedTraceableBox::from_box(Heap::boxed(*payload.as_ref(cx))),
        host_defined: RootedTraceableBox::from_box(Heap::boxed(*host_defined.as_ref(cx))),
    };

    let on_single_fetch_complete =
        move |cx: &mut JSContext, module_tree: Option<Rc<ModuleTree>>| {
            let mut realm = CurrentRealm::assert(cx);
            let cx = &mut realm;

            // Step 1. Let completion be null.
            let completion = match module_tree {
                // Step 2. If moduleScript is null, then set completion to ThrowCompletion(a new TypeError).
                None => Err(gen_type_error(
                    cx,
                    &global_scope,
                    Error::Type(c"Module fetching failed".to_owned()),
                )),
                Some(module_tree) => {
                    // Step 3. Otherwise, if moduleScript's parse error is not null, then:
                    // Step 3.1 Let parseError be moduleScript's parse error.
                    if let Some(parse_error) = module_tree.get_parse_error() {
                        // Step 3.3 If loadState is not undefined and loadState.[[ErrorToRethrow]] is null,
                        // set loadState.[[ErrorToRethrow]] to parseError.
                        let load_state =
                            load_state_from_handle_value(request.host_defined.handle());
                        load_state.inspect(|load_state| {
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
            finish_loading_imported_module(
                cx,
                request.referrer.handle(),
                request.module_request.handle(),
                request.payload.handle(),
                completion,
            );
        };

    // Step 14 Fetch a single imported module script given url, fetchClient, destination, fetchOptions, settingsObject,
    // fetchReferrer, moduleRequest, and onSingleFetchComplete as defined below.
    // If loadState is not undefined and loadState.[[PerformFetch]] is not null, pass loadState.[[PerformFetch]] along as well.
    // Note: we don't have access to the requested `ModuleObject`, so we pass only its type.
    fetch_a_single_imported_module_script(
        cx,
        url,
        fetch_client,
        global,
        destination,
        fetch_options,
        fetch_referrer,
        module_type,
        on_single_fetch_complete,
    );
}

/// <https://html.spec.whatwg.org/multipage/#fetch-a-single-imported-module-script>
#[expect(clippy::too_many_arguments)]
fn fetch_a_single_imported_module_script(
    cx: &mut JSContext,
    url: UrlWithBlobClaim,
    fetch_client: RequestClient,
    global: &GlobalScope,
    destination: Destination,
    options: ScriptFetchOptions,
    referrer: Referrer,
    module_type: ModuleType,
    on_complete: impl FnOnce(&mut JSContext, Option<Rc<ModuleTree>>) + 'static,
) {
    // TODO Step 1. Assert: moduleRequest.[[Attributes]] does not contain any Record entry such that entry.[[Key]] is not "type",
    // because we only asked for "type" attributes in HostGetSupportedImportAttributes.

    // TODO Step 2. Let moduleType be the result of running the module type from module request steps given moduleRequest.

    // Step 3. If the result of running the module type allowed steps given moduleType and settingsObject is false,
    // then run onComplete given null, and return.
    match module_type {
        ModuleType::Unknown | ModuleType::Bytes | ModuleType::Text | ModuleType::CSS => {
            return on_complete(cx, None);
        },
        ModuleType::JavaScript | ModuleType::JSON => (),
    }

    // Step 4. Fetch a single module script given url, fetchClient, destination, options, settingsObject, referrer,
    // moduleRequest, false, and onComplete. If performFetch was given, pass it along as well.
    fetch_a_single_module_script(
        cx,
        url,
        fetch_client,
        global,
        destination,
        options,
        referrer,
        Some(module_type),
        false,
        Some(IntroductionType::IMPORTED_MODULE),
        on_complete,
    );
}

/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#![expect(non_snake_case, unsafe_code)]

use std::cell::{Cell, RefCell};
use std::ffi::CStr;
use std::rc::Rc;

use indexmap::IndexMap;
use js::conversions::jsstr_to_string;
use js::jsapi::{
    GetModuleRequestSpecifier, GetRequestedModuleSpecifier, GetRequestedModulesCount, Handle,
    HandleObject, HandleValue, Heap, JSObject,
};
use js::jsval::JSVal;
use net_traits::http_status::HttpStatus;
use net_traits::request::{Destination, Referrer, RequestBuilder, RequestId, RequestMode};
use net_traits::{FetchMetadata, Metadata, NetworkError, ResourceFetchTiming};
use script_bindings::codegen::GenericBindings::WindowBinding::WindowMethods;
use script_bindings::str::DOMString;
use servo_url::ServoUrl;

use crate::document_loader::LoadType;
use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::refcounted::Trusted;
use crate::dom::bindings::root::DomRoot;
use crate::dom::csp::{GlobalCspReporting, Violation};
use crate::dom::globalscope::GlobalScope;
use crate::dom::performanceresourcetiming::InitiatorType;
use crate::dom::promise::Promise;
use crate::dom::window::Window;
use crate::network_listener::{FetchResponseListener, NetworkListener, ResourceTimingListener};
use crate::realms::{AlreadyInRealm, InRealm};
use crate::script_module::{
    ModuleTree, RethrowError, ScriptFetchOptions, gen_type_error,
    module_script_from_reference_private,
};
use crate::script_runtime::{CanGc, IntroductionType};

#[derive(PartialEq, Debug)]
struct ModuleObject(Box<Heap<*mut JSObject>>);

impl ModuleObject {
    fn new(obj: HandleObject) -> ModuleObject {
        ModuleObject(Heap::boxed(obj.get()))
    }

    pub(crate) fn handle(&self) -> HandleObject {
        unsafe { self.0.handle().into() }
    }
}

unsafe fn private_module_data_from_reference(
    reference_private: &Handle<JSVal>,
) -> Option<&PrivateModuleData> {
    if reference_private.get().is_undefined() {
        return None;
    }
    unsafe { (reference_private.get().to_private() as *const PrivateModuleData).as_ref() }
}

struct PrivateModuleData {
    loaded_modules: DomRefCell<IndexMap<String, ModuleObject>>,
}

/// <https://tc39.es/ecma262/#graphloadingstate-record>
struct GraphLoadingState {
    promise: Rc<Promise>,
    is_loading: Cell<bool>,
    pending_modules_count: Cell<u32>,
    visited: RefCell<Vec<ModuleObject>>,
}

/// <https://tc39.es/ecma262/#sec-LoadRequestedModules>
fn LoadRequestedModules(global: &GlobalScope, module: ModuleObject) {
    // Step 1. If hostDefined is not present, let hostDefined be empty.

    // Step 2. Let pc be ! NewPromiseCapability(%Promise%).
    let promise = Promise::new(global, CanGc::note());

    // Step 3. Let state be the GraphLoadingState Record
    // { [[IsLoading]]: true, [[PendingModulesCount]]: 1, [[Visited]]: « », [[PromiseCapability]]: pc, [[HostDefined]]: hostDefined }.
    let state = GraphLoadingState {
        promise,
        is_loading: Cell::new(true),
        pending_modules_count: Cell::new(1),
        visited: RefCell::new(Vec::new()),
    };

    // Step 4. Perform InnerModuleLoading(state, module).
    InnerModuleLoading(&state, module);

    // Step 5. Return pc.[[Promise]].
}

/// <https://tc39.es/ecma262/#sec-InnerModuleLoading>
fn InnerModuleLoading(state: &GraphLoadingState, module: ModuleObject) {
    let cx = GlobalScope::get_cx();
    let module_handle = module.handle();

    // Step 1. Assert: state.[[IsLoading]] is true.
    assert!(state.is_loading.get());

    let visited_contains_module = state.visited.borrow().contains(&module);

    // Step 2. If module is a Cyclic Module Record, module.[[Status]] is new, and state.[[Visited]] does not contain module, then
    if !visited_contains_module {
        // a. Append module to state.[[Visited]].
        state.visited.borrow_mut().push(module);

        // b. Let requestedModulesCount be the number of elements in module.[[RequestedModules]].
        let requested_modules_count = unsafe { GetRequestedModulesCount(*cx, module_handle) };

        // c. Set state.[[PendingModulesCount]] to state.[[PendingModulesCount]] + requestedModulesCount.
        state
            .pending_modules_count
            .update(|count| count + requested_modules_count);

        // d. For each ModuleRequest Record request of module.[[RequestedModules]], do
        for index in 0..requested_modules_count {
            // Here Gecko will call hasFirstUnsupportedAttributeKey on each module request,
            // GetRequestedModuleSpecifier will do it for us.
            let jsstr = unsafe { GetRequestedModuleSpecifier(*cx, module_handle, index) };

            if jsstr.is_null() {
                // Step 1. Let error be ThrowCompletion(a newly created SyntaxError object).
                let error = RethrowError::from_pending_exception(cx);

                // Step 2. Perform ContinueModuleLoading(state, error).
                ContinueModuleLoading(state, Err(error));
            } else if false {
                // ii. Else if module.[[LoadedModules]] contains a LoadedModuleRequest Record record
                // such that ModuleRequestsEqual(record, request) is true, then
                // 1. Perform InnerModuleLoading(state, record.[[Module]]).
            } else {
                // 1. Perform HostLoadImportedModule(module, request, state.[[HostDefined]], state).
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
    state.pending_modules_count.update(|count| count - 1);

    // Step 5. If state.[[PendingModulesCount]] = 0, then
    if state.pending_modules_count.get() == 0 {
        // a. Set state.[[IsLoading]] to false.
        state.is_loading.set(false);

        // b. For each Cyclic Module Record loaded of state.[[Visited]], do
        // for loaded in state.visited {
        // i. If loaded.[[Status]] is new, set loaded.[[Status]] to unlinked.
        // }

        // c. Perform ! Call(state.[[PromiseCapability]].[[Resolve]], undefined, « undefined »).
    }

    // Step 6. Return unused.
}

/// <https://tc39.es/ecma262/#sec-ContinueModuleLoading>
fn ContinueModuleLoading(
    state: &GraphLoadingState,
    module_completion: Result<ModuleObject, RethrowError>,
) {
    // Step 1. If state.[[IsLoading]] is false, return unused.
    if !state.is_loading.get() {
        return;
    }

    // TODO Pass a result with module or error
    match module_completion {
        // Step 2. If moduleCompletion is a normal completion, then
        // a. Perform InnerModuleLoading(state, moduleCompletion.[[Value]]).
        Ok(module) => InnerModuleLoading(state, module),

        // Step 3. Else,
        Err(_) => {
            // a. Set state.[[IsLoading]] to false.
            state.is_loading.set(false);

            // TODO b. Perform ! Call(state.[[PromiseCapability]].[[Reject]], undefined, « moduleCompletion.[[Value]] »).
        },
    }
    // Step 4. Return unused.
}

/// <https://tc39.es/ecma262/#sec-FinishLoadingImportedModule>
/// We should use a map whose keys are module requests, for now we use module request's specifier
fn FinishLoadingImportedModule(
    referrer: HandleValue,
    module_request_specifier: String,
    payload: GraphLoadingState,
    result: Result<ModuleObject, RethrowError>,
) {
    // Step 1. If result is a normal completion, then
    if let Ok(ref module) = result {
        if let Some(private_data) = unsafe { private_module_data_from_reference(&referrer) } {
            let mut loaded_modules = private_data.loaded_modules.borrow_mut();

            // a. If referrer.[[LoadedModules]] contains a LoadedModuleRequest Record record such that
            // ModuleRequestsEqual(record, moduleRequest) is true, then
            loaded_modules
                .get(&module_request_specifier)
                // i. Assert: record.[[Module]] and result.[[Value]] are the same Module Record.
                .map(|record| assert_eq!(record.handle(), module.handle()))
                // b. Else,
                .unwrap_or_else(|| {
                    // i. Append the LoadedModuleRequest Record { [[Specifier]]: moduleRequest.[[Specifier]],
                    // [[Attributes]]: moduleRequest.[[Attributes]], [[Module]]: result.[[Value]] } to referrer.[[LoadedModules]].
                    loaded_modules
                        .insert(module_request_specifier, ModuleObject::new(module.handle()));
                });
        }
    }

    // Step 2. If payload is a GraphLoadingState Record, then
    // a. Perform ContinueModuleLoading(payload, result).
    ContinueModuleLoading(&payload, result);

    // TODO Step 3. Else,
    // a. Perform ContinueDynamicImport(payload, result).

    // 4. Return unused.
}

/// <https://html.spec.whatwg.org/multipage/webappapis.html#hostloadimportedmodule>
fn HostLoadImportedModule(
    referrer: HandleValue,
    module_request: Handle<*mut JSObject>,
    /* loadState, */ payload: GraphLoadingState,
) {
    let cx = GlobalScope::get_cx();
    let in_realm_proof = AlreadyInRealm::assert_for_cx(cx);
    let global_scope = unsafe { GlobalScope::from_context(*cx, InRealm::Already(&in_realm_proof)) };

    // Step 1. Let settingsObject be the current settings object.
    // Step 2. If settingsObject's global object implements WorkletGlobalScope or ServiceWorkerGlobalScope and loadState is undefined, then:

    // Step 3. Let referencingScript be null.
    let referencing_script = unsafe { module_script_from_reference_private(&referrer) };

    // Step 4. Let originalFetchOptions be the default script fetch options.

    // Step 5. Let fetchReferrer be "client".

    // Step 6. If referrer is a Script Record or a Cyclic Module Record, then:
    let (original_fetch_options, fetch_referrer) = match referencing_script {
        Some(module) => (
            module.options.clone(),
            Referrer::ReferrerUrl(module.base_url.clone()),
        ),
        None => (
            ScriptFetchOptions::default_classic_script(&global_scope),
            global_scope.get_referrer(),
        ),
    };

    // TODO It seems that Gecko doesn't implement this step, and currently we don't handle module types.
    // Step 7 If referrer is a Cyclic Module Record and moduleRequest is equal to the first element of referrer.[[RequestedModules]], then:

    let specifier = unsafe {
        let jsstr = std::ptr::NonNull::new(GetModuleRequestSpecifier(*cx, module_request)).unwrap();
        jsstr_to_string(*cx, jsstr)
    };

    // Step 8 Let url be the result of resolving a module specifier given referencingScript and moduleRequest.[[Specifier]],
    // catching any exceptions. If they throw an exception, let resolutionError be the thrown exception.
    let url = ModuleTree::resolve_module_specifier(
        &global_scope,
        referencing_script,
        DOMString::from_string(specifier.clone()),
        CanGc::note(),
    );

    // Step 9 If the previous step threw an exception, then:
    let Ok(url) = url else {
        // TODO Step 9.1. If loadState is not undefined and loadState.[[ErrorToRethrow]] is null,
        // set loadState.[[ErrorToRethrow]] to resolutionError.

        let resolution_error = gen_type_error(
            &global_scope,
            "Wrong module specifier".to_string(),
            CanGc::note(),
        );

        // Step 9.2. Perform FinishLoadingImportedModule(referrer, moduleRequest, payload, ThrowCompletion(resolutionError)).
        FinishLoadingImportedModule(referrer, specifier, payload, Err(resolution_error));

        // Step 9.3. Return.
        return;
    };

    // Step 10. Let fetchOptions be the result of getting the descendant script fetch options given
    // originalFetchOptions, url, and settingsObject.
    let fetch_options = original_fetch_options.descendant_fetch_options();

    // Step 11. Let destination be "script".
    let destination = Destination::Script;

    // Step 12. Let fetchClient be settingsObject.

    // TODO Step 13. If loadState is not undefined, then:

    // Step 14 Fetch a single imported module script given url, fetchClient, destination, fetchOptions, settingsObject,
    // fetchReferrer, moduleRequest, and onSingleFetchComplete as defined below.
    // If loadState is not undefined and loadState.[[PerformFetch]] is not null, pass loadState.[[PerformFetch]] along as well.
    fetch_a_single_imported_module_script(
        url,
        &global_scope,
        destination,
        fetch_options,
        fetch_referrer,
    );
}

/// <https://html.spec.whatwg.org/multipage/webappapis.html#fetch-a-single-imported-module-script>
fn fetch_a_single_imported_module_script(
    url: ServoUrl,
    global_scope: &GlobalScope,
    destination: Destination,
    options: ScriptFetchOptions,
    referrer: Referrer,
) {
    // Step 1. Assert: moduleRequest.[[Attributes]] does not contain any Record entry such that entry.[[Key]] is not "type",
    // because we only asked for "type" attributes in HostGetSupportedImportAttributes.

    // Step 2. Let moduleType be the result of running the module type from module request steps given moduleRequest.

    // Step 3. If the result of running the module type allowed steps given moduleType and settingsObject is false,
    // then run onComplete given null, and return.

    // Step 4. Fetch a single module script given url, fetchClient, destination, options, settingsObject, referrer,
    // moduleRequest, false, and onComplete. If performFetch was given, pass it along as well.
    fetch_a_single_module_script(
        url,
        global_scope,
        destination,
        options,
        referrer,
        false,
        Some(IntroductionType::IMPORTED_MODULE),
    );
}

/// <https://html.spec.whatwg.org/multipage/webappapis.html#fetch-a-single-module-script>
fn fetch_a_single_module_script(
    url: ServoUrl,
    global: &GlobalScope,
    destination: Destination,
    options: ScriptFetchOptions,
    referrer: Referrer,
    is_top_level: bool,
    introduction_type: Option<&'static CStr>,
) {
    // Step 1. Let moduleType be "javascript-or-wasm".

    // Step 2. If moduleRequest was given, then set moduleType to the result of running the
    // module type from module request steps given moduleRequest.

    // Step 3. Assert: the result of running the module type allowed steps given moduleType and settingsObject is true.
    // Otherwise, we would not have reached this point because a failure would have been raised
    // when inspecting moduleRequest.[[Attributes]] in HostLoadImportedModule or fetch a single imported module script.

    // Step 4. Let moduleMap be settingsObject's module map.

    // Step 5. If moduleMap[(url, moduleType)] is "fetching", wait in parallel until that entry's value changes,
    // then queue a task on the networking task source to proceed with running the following steps.

    // Step 6. If moduleMap[(url, moduleType)] exists, run onComplete given moduleMap[(url, moduleType)], and return.

    // Step 7. Set moduleMap[(url, moduleType)] to "fetching".

    // Step 8. Let request be a new request whose URL is url, mode is "cors", referrer is referrer, and client is fetchClient.

    // Step 9. Set request's destination to the result of running the fetch destination from module type steps given destination and moduleType.

    // Step 10. If destination is "worker", "sharedworker", or "serviceworker", and isTopLevel is true, then set request's mode to "same-origin".
    let mode = match destination {
        Destination::Worker | Destination::SharedWorker if is_top_level => RequestMode::SameOrigin,
        _ => RequestMode::CorsMode,
    };

    // Step 11. Set request's initiator type to "script".

    // Step 12. Set up the module script request given request and options.
    let request = RequestBuilder::new(None, url.clone(), referrer)
        .destination(destination)
        .origin(global.origin().immutable().clone())
        .parser_metadata(options.parser_metadata)
        .integrity_metadata(options.integrity_metadata.clone())
        .credentials_mode(options.credentials_mode)
        .referrer_policy(options.referrer_policy)
        .mode(mode)
        .insecure_requests_policy(global.insecure_requests_policy())
        .has_trustworthy_ancestor_origin(global.has_trustworthy_ancestor_origin())
        .policy_container(global.policy_container().to_owned())
        .cryptographic_nonce_metadata(options.cryptographic_nonce.clone());

    let context = ModuleContext {
        global: Trusted::new(global),
        data: vec![],
        metadata: None,
        url: url.clone(),
        destination,
        options,
        status: Ok(()),
        introduction_type,
    };

    let network_listener = NetworkListener::new(
        context,
        global.task_manager().networking_task_source().to_sendable(),
    );
    global.fetch_with_network_listener(request, network_listener);
}

struct ModuleContext {
    global: Trusted<GlobalScope>,
    /// The response body received to date.
    data: Vec<u8>,
    /// The response metadata received to date.
    metadata: Option<Metadata>,
    /// The initial URL requested.
    url: ServoUrl,
    /// Destination of current module context
    destination: Destination,
    /// Options for the current script fetch
    options: ScriptFetchOptions,
    /// Indicates whether the request failed, and why
    status: Result<(), NetworkError>,
    /// `introductionType` value to set in the `CompileOptionsWrapper`.
    introduction_type: Option<&'static CStr>,
}

impl FetchResponseListener for ModuleContext {
    // TODO(cybai): Perhaps add custom steps to perform fetch here?
    fn process_request_body(&mut self, _: RequestId) {}

    // TODO(cybai): Perhaps add custom steps to perform fetch here?
    fn process_request_eof(&mut self, _: RequestId) {}

    fn process_response(&mut self, _: RequestId, metadata: Result<FetchMetadata, NetworkError>) {
        self.metadata = metadata.ok().map(|meta| match meta {
            FetchMetadata::Unfiltered(m) => m,
            FetchMetadata::Filtered { unsafe_, .. } => unsafe_,
        });

        let status = self
            .metadata
            .as_ref()
            .map(|m| m.status.clone())
            .unwrap_or_else(HttpStatus::new_error);

        self.status = {
            if status.is_error() {
                Err(NetworkError::Internal(
                    "No http status code received".to_owned(),
                ))
            } else if status.is_success() {
                Ok(())
            } else {
                Err(NetworkError::Internal(format!(
                    "HTTP error code {}",
                    status.code()
                )))
            }
        };
    }

    fn process_response_chunk(&mut self, _: RequestId, mut chunk: Vec<u8>) {
        if self.status.is_ok() {
            self.data.append(&mut chunk);
        }
    }

    /// <https://html.spec.whatwg.org/multipage/#fetch-a-single-module-script>
    /// Step 9-12
    fn process_response_eof(
        self,
        _: RequestId,
        response: Result<ResourceFetchTiming, NetworkError>,
    ) {
        let global = self.global.root();

        if let Some(window) = DomRoot::downcast::<Window>(global) {
            window
                .Document()
                .finish_load(LoadType::Script(self.url.clone()), CanGc::note());
        }
    }

    fn process_csp_violations(&mut self, _request_id: RequestId, violations: Vec<Violation>) {
        let global = &self.resource_timing_global();
        global.report_csp_violations(violations, None, None);
    }
}

impl ResourceTimingListener for ModuleContext {
    fn resource_timing_information(&self) -> (InitiatorType, ServoUrl) {
        let initiator_type = InitiatorType::LocalName("module".to_string());
        (initiator_type, self.url.clone())
    }

    fn resource_timing_global(&self) -> DomRoot<GlobalScope> {
        self.global.root()
    }
}

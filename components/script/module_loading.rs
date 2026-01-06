/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#![expect(non_snake_case, unsafe_code)]

use std::cell::{Cell, RefCell};
use std::collections::HashSet;
use std::ffi::CStr;
use std::rc::Rc;

use encoding_rs::UTF_8;
use headers::{HeaderMapExt, ReferrerPolicy as ReferrerPolicyHeader};
use hyper_serde::Serde;
use indexmap::IndexMap;
use js::conversions::jsstr_to_string;
use js::jsapi::{
    GetRequestedModuleSpecifier, GetRequestedModulesCount, Handle, HandleObject, Heap, JSObject,
};
use js::jsval::{JSVal, ObjectValue, UndefinedValue};
use js::rust::IntoHandle;
use mime::Mime;
use net_traits::http_status::HttpStatus;
use net_traits::request::{Destination, Referrer, RequestBuilder, RequestId, RequestMode};
use net_traits::{FetchMetadata, Metadata, NetworkError, ReferrerPolicy, ResourceFetchTiming};
use script_bindings::codegen::GenericBindings::WindowBinding::WindowMethods;
use script_bindings::str::DOMString;
use servo_url::ServoUrl;

use crate::document_loader::LoadType;
use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::refcounted::Trusted;
use crate::dom::bindings::root::DomRoot;
use crate::dom::csp::{GlobalCspReporting, Violation};
use crate::dom::globalscope::GlobalScope;
use crate::dom::htmlscriptelement::{SCRIPT_JS_MIMES, substitute_with_local_script};
use crate::dom::performanceresourcetiming::InitiatorType;
use crate::dom::promise::Promise;
use crate::dom::window::Window;
use crate::network_listener::{
    self, FetchResponseListener, NetworkListener, ResourceTimingListener,
};
use crate::script_module::{
    ModuleObject as Module, ModuleTree, RethrowError, ScriptFetchOptions,
    create_a_javascript_module_script, gen_type_error, module_script_from_reference_private,
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
    InnerModuleLoading(global, &state, module);

    // Step 5. Return pc.[[Promise]].
}

/// <https://tc39.es/ecma262/#sec-InnerModuleLoading>
fn InnerModuleLoading(global: &GlobalScope, state: &GraphLoadingState, module: ModuleObject) {
    let cx = GlobalScope::get_cx();
    let module_handle = module.handle();

    rooted!(&in(cx) let mut referrer = UndefinedValue());
    referrer.handle_mut().set(ObjectValue(module.0.get()));
    let referrer_handle = referrer.handle().into_handle();

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
                ContinueModuleLoading(global, state, Err(error));
            } else if let Some(private_data) =
                unsafe { private_module_data_from_reference(&referrer_handle) }
            {
                let specifier =
                    unsafe { jsstr_to_string(*cx, std::ptr::NonNull::new(jsstr).unwrap()) };

                // ii. Else if module.[[LoadedModules]] contains a LoadedModuleRequest Record record
                // such that ModuleRequestsEqual(record, request) is true, then
                if let Some(module) = private_data.loaded_modules.borrow().get(&specifier) {
                    // 1. Perform InnerModuleLoading(state, record.[[Module]]).
                    InnerModuleLoading(global, state, ModuleObject::new(module.handle()));
                } else {
                    // 1. Perform HostLoadImportedModule(module, request, state.[[HostDefined]], state).
                    HostLoadImportedModule(global, referrer_handle, specifier, state);
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
    global: &GlobalScope,
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
        Ok(module) => InnerModuleLoading(global, state, module),

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
    global: &GlobalScope,
    referrer: Handle<JSVal>,
    module_request_specifier: String,
    payload: &GraphLoadingState,
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
    ContinueModuleLoading(global, payload, result);

    // TODO Step 3. Else,
    // a. Perform ContinueDynamicImport(payload, result).

    // 4. Return unused.
}

/// <https://html.spec.whatwg.org/multipage/webappapis.html#hostloadimportedmodule>
fn HostLoadImportedModule(
    global_scope: &GlobalScope,
    referrer: Handle<JSVal>,
    specifier: String,
    /* loadState, */
    payload: &GraphLoadingState,
) {
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

    // Step 8 Let url be the result of resolving a module specifier given referencingScript and moduleRequest.[[Specifier]],
    // catching any exceptions. If they throw an exception, let resolutionError be the thrown exception.
    let url = ModuleTree::resolve_module_specifier(
        global_scope,
        referencing_script,
        DOMString::from_string(specifier.clone()),
        CanGc::note(),
    );

    // Step 9 If the previous step threw an exception, then:
    let Ok(url) = url else {
        // TODO Step 9.1. If loadState is not undefined and loadState.[[ErrorToRethrow]] is null,
        // set loadState.[[ErrorToRethrow]] to resolutionError.

        let resolution_error = gen_type_error(
            global_scope,
            "Wrong module specifier".to_string(),
            CanGc::note(),
        );

        // Step 9.2. Perform FinishLoadingImportedModule(referrer, moduleRequest, payload, ThrowCompletion(resolutionError)).
        FinishLoadingImportedModule(
            global_scope,
            referrer,
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
    let destination = Destination::Script;

    // Step 12. Let fetchClient be settingsObject.

    // TODO Step 13. If loadState is not undefined, then:

    let on_single_fetch_complete = |module_tree: Option<&Rc<ModuleTree>>| {
        let completion = match module_tree {
            None => Err(gen_type_error(
                global_scope,
                "Module fetcing failed".to_string(),
                CanGc::note(),
            )),
            Some(tree) => {
                let parse_error = tree.get_rethrow_error().borrow().as_ref().cloned();

                if let Some(error) = parse_error {
                    Err(error)
                } else {
                    let module_handle = tree.get_record().borrow().as_ref().unwrap().handle();
                    Ok(ModuleObject::new(module_handle))
                }
            },
        };

        FinishLoadingImportedModule(global_scope, referrer, specifier, payload, completion);
    };

    // Step 14 Fetch a single imported module script given url, fetchClient, destination, fetchOptions, settingsObject,
    // fetchReferrer, moduleRequest, and onSingleFetchComplete as defined below.
    // If loadState is not undefined and loadState.[[PerformFetch]] is not null, pass loadState.[[PerformFetch]] along as well.
    fetch_a_single_imported_module_script(
        url,
        global_scope,
        destination,
        fetch_options,
        fetch_referrer,
        on_single_fetch_complete,
    );
}

/// <https://html.spec.whatwg.org/multipage/webappapis.html#fetch-a-single-imported-module-script>
fn fetch_a_single_imported_module_script(
    url: ServoUrl,
    global_scope: &GlobalScope,
    destination: Destination,
    options: ScriptFetchOptions,
    referrer: Referrer,
    on_complete: impl FnOnce(Option<&Rc<ModuleTree>>),
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
        on_complete,
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
    on_complete: impl FnOnce(Option<&Rc<ModuleTree>>),
) {
    // Step 1. Let moduleType be "javascript-or-wasm".

    // Step 2. If moduleRequest was given, then set moduleType to the result of running the
    // module type from module request steps given moduleRequest.

    // Step 3. Assert: the result of running the module type allowed steps given moduleType and settingsObject is true.
    // Otherwise, we would not have reached this point because a failure would have been raised
    // when inspecting moduleRequest.[[Attributes]] in HostLoadImportedModule or fetch a single imported module script.

    {
        // Step 4. Let moduleMap be settingsObject's module map.
        let mut module_map = global.get_module_map().borrow_mut();

        if let Some(module_tree) = module_map.get(&url) {
            if module_tree.get_record().borrow().is_some() {
                return on_complete(Some(module_tree));
            }
            // TODO Step 5. If moduleMap[(url, moduleType)] is "fetching", wait in parallel until that entry's value changes,
            // then queue a task on the networking task source to proceed with running the following steps.
        } else {
            // Step 7. Set moduleMap[(url, moduleType)] to "fetching".
            let module_tree = ModuleTree::new(url.clone(), true, HashSet::new());
            module_map.insert(url.clone(), Rc::new(module_tree));
        }
    }

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

    /// <https://html.spec.whatwg.org/multipage/webappapis.html#fetch-a-single-module-script>
    /// Step 13
    fn process_response_eof(
        mut self,
        _: RequestId,
        response: Result<ResourceFetchTiming, NetworkError>,
    ) {
        let global = self.global.root();

        if let Some(window) = global.downcast::<Window>() {
            window
                .Document()
                .finish_load(LoadType::Script(self.url.clone()), CanGc::note());
        }

        if let Ok(response) = &response {
            network_listener::submit_timing(&self, response, CanGc::note());
        }

        // Step 1. If any of the following are true: bodyBytes is null or failure; or response's status is not an ok status,
        // then set moduleMap[(url, moduleType)] to null, run onComplete given null, and abort these steps.
        if let (Err(error), _) | (_, Err(error)) = (response.as_ref(), self.status.as_ref()) {
            error!("Fetching module script failed {:?}", error);
            return;
        }

        let metadata = self.metadata.take().unwrap();
        let final_url = metadata.final_url;

        // Step 2. Let mimeType be the result of extracting a MIME type from response's header list.
        let mime_type: Option<Mime> = metadata.content_type.map(Serde::into_inner).map(Into::into);

        // Step 3. Let moduleScript be null.

        // Step 4. Let referrerPolicy be the result of parsing the `Referrer-Policy` header given response. [REFERRERPOLICY]
        let referrer_policy = metadata
            .headers
            .and_then(|headers| headers.typed_get::<ReferrerPolicyHeader>())
            .into();

        // Step 5. If referrerPolicy is not the empty string, set options's referrer policy to referrerPolicy.
        if referrer_policy != ReferrerPolicy::EmptyString {
            self.options.referrer_policy = referrer_policy;
        }

        // TODO Step 6. If mimeType's essence is "application/wasm" and moduleType is "javascript-or-wasm", then set
        // moduleScript to the result of creating a WebAssembly module script given bodyBytes, settingsObject, response's URL, and options.

        // TODO handle CSS and JSON module scripts

        // Step 7.1 Let sourceText be the result of UTF-8 decoding bodyBytes.
        let (mut source_text, _, _) = UTF_8.decode(&self.data);

        // Step 7.2 If mimeType is a JavaScript MIME type and moduleType is "javascript-or-wasm", then set moduleScript
        // to the result of creating a JavaScript module script given sourceText, settingsObject, response's URL, and options.
        if mime_type.is_some_and(|mime| SCRIPT_JS_MIMES.contains(&mime.essence_str())) {
            if let Some(window) = global.downcast::<Window>() {
                substitute_with_local_script(window, &mut source_text, final_url.clone());
            }

            let cx = GlobalScope::get_cx();
            rooted!(in(*cx) let mut compiled_module: *mut JSObject = std::ptr::null_mut());
            let result = create_a_javascript_module_script(
                &source_text,
                final_url,
                self.options,
                compiled_module.handle_mut(),
                self.introduction_type,
            );

            // Step 8. Set moduleMap[(url, moduleType)] to moduleScript, and run onComplete given moduleScript.
            global
                .get_module_map()
                .borrow()
                .get(&self.url)
                .map(|module_tree| {
                    if let Err(error) = result {
                        module_tree.set_rethrow_error(error);
                    } else {
                        module_tree.set_record(Module::new(compiled_module.handle()));
                    }
                });
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

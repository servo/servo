/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! The script module mod contains common traits and structs
//! related to `type=module` for script thread or worker threads.

use std::borrow::Cow;
use std::cell::{OnceCell, RefCell};
use std::collections::hash_map::Entry;
use std::ffi::CStr;
use std::fmt::Debug;
use std::ptr::NonNull;
use std::rc::Rc;
use std::{mem, ptr};

use encoding_rs::UTF_8;
use headers::{HeaderMapExt, ReferrerPolicy as ReferrerPolicyHeader};
use hyper_serde::Serde;
use indexmap::IndexMap;
use indexmap::map::Entry as IndexMapEntry;
use js::context::JSContext;
use js::conversions::{ToJSValConvertible, jsstr_to_string};
use js::gc::{HandleObject, MutableHandleValue};
use js::jsapi::{
    CallArgs, ExceptionStackBehavior, GetFunctionNativeReserved, GetModuleResolveHook,
    Handle as RawHandle, HandleValue as RawHandleValue, Heap, JS_GetFunctionObject,
    JSContext as RawJSContext, JSObject, JSPROP_ENUMERATE, JSRuntime, ModuleErrorBehaviour,
    ModuleType, SetFunctionNativeReserved, SetModuleDynamicImportHook, SetModuleMetadataHook,
    SetModulePrivate, SetModuleResolveHook, SetScriptPrivateReferenceHooks, Value,
};
use js::jsval::{JSVal, PrivateValue, UndefinedValue};
use js::realm::{AutoRealm, CurrentRealm};
use js::rust::wrappers2::{
    CompileJsonModule1, CompileModule1, DefineFunctionWithReserved, GetModuleRequestSpecifier,
    GetModuleRequestType, JS_ClearPendingException, JS_DefineProperty4, JS_GetPendingException,
    JS_NewStringCopyN, JS_SetPendingException, ModuleEvaluate, ModuleLink,
    ThrowOnModuleEvaluationFailure,
};
use js::rust::{Handle, HandleValue, ToString, transform_str_to_source_text};
use mime::Mime;
use net_traits::blob_url_store::UrlWithBlobClaim;
use net_traits::http_status::HttpStatus;
use net_traits::mime_classifier::MimeClassifier;
use net_traits::policy_container::PolicyContainer;
use net_traits::request::{
    CredentialsMode, Destination, ParserMetadata, Referrer, RequestBuilder, RequestClient,
    RequestId, RequestMode,
};
use net_traits::{FetchMetadata, Metadata, NetworkError, ReferrerPolicy, ResourceFetchTiming};
use script_bindings::cell::DomRefCell;
use script_bindings::error::Fallible;
use script_bindings::reflector::DomObject;
use script_bindings::settings_stack::run_a_callback;
use script_bindings::trace::CustomTraceable;
use servo_config::pref;
use servo_url::ServoUrl;

use crate::DomTypeHolder;
use crate::dom::bindings::error::{Error, ErrorToJsval, report_pending_exception};
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::refcounted::Trusted;
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::DOMString;
use crate::dom::bindings::trace::RootedTraceableBox;
use crate::dom::csp::{GlobalCspReporting, Violation};
use crate::dom::globalscope::GlobalScope;
use crate::dom::globalscope::script_execution::{ErrorReporting, fill_compile_options};
use crate::dom::html::htmlscriptelement::{SCRIPT_JS_MIMES, substitute_with_local_script};
use crate::dom::performance::performanceresourcetiming::InitiatorType;
use crate::dom::promise::Promise;
use crate::dom::promisenativehandler::{Callback, PromiseNativeHandler};
use crate::dom::types::{
    DedicatedWorkerGlobalScope, SharedWorkerGlobalScope, WorkerGlobalScope, WorkletGlobalScope,
};
use crate::dom::window::Window;
use crate::fetch::network_listener::{self, FetchResponseListener, ResourceTimingListener};
use crate::modules::import_map::{ModuleSpecifierMap, resolve_url_like_module_specifier};
use crate::modules::module_loading::{
    LoadState, Payload, host_load_imported_module, load_requested_modules,
};
use crate::realms::enter_auto_realm;
use crate::script_runtime::IntroductionType;
use crate::tasks::task::NonSendTaskBox;
use crate::unminify::{ScriptSource, unminify_js};

pub(crate) fn gen_type_error(
    cx: &mut JSContext,
    global: &GlobalScope,
    error: Error,
) -> RethrowError {
    rooted!(&in(cx) let mut thrown = UndefinedValue());
    error.to_jsval(cx, global, thrown.handle_mut());

    RethrowError(RootedTraceableBox::from_box(Heap::boxed(thrown.get())))
}

#[derive(JSTraceable)]
pub(crate) struct ModuleObject(RootedTraceableBox<Heap<*mut JSObject>>);

impl ModuleObject {
    pub(crate) fn new(obj: HandleObject) -> ModuleObject {
        ModuleObject(RootedTraceableBox::from_box(Heap::boxed(obj.get())))
    }

    pub(crate) fn handle(&'_ self) -> HandleObject<'_> {
        self.0.handle()
    }
}

#[derive(JSTraceable)]
pub(crate) struct RethrowError(RootedTraceableBox<Heap<JSVal>>);

impl RethrowError {
    pub(crate) fn new(val: Box<Heap<JSVal>>) -> Self {
        Self(RootedTraceableBox::from_box(val))
    }

    #[expect(unsafe_code)]
    pub(crate) fn from_pending_exception(cx: &mut JSContext) -> Self {
        rooted!(&in(cx) let mut exception = UndefinedValue());
        assert!(unsafe { JS_GetPendingException(cx, exception.handle_mut()) });
        unsafe { JS_ClearPendingException(cx) };

        Self::new(Heap::boxed(exception.get()))
    }

    pub(crate) fn handle(&self) -> Handle<'_, JSVal> {
        self.0.handle()
    }
}

impl Debug for RethrowError {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        "RethrowError(...)".fmt(fmt)
    }
}

impl Clone for RethrowError {
    fn clone(&self) -> Self {
        Self(RootedTraceableBox::from_box(Heap::boxed(self.0.get())))
    }
}

pub(crate) struct ModuleScript {
    pub(crate) base_url: ServoUrl,
    pub(crate) options: ScriptFetchOptions,
    pub(crate) owner: Option<Trusted<GlobalScope>>,
}

impl ModuleScript {
    pub(crate) fn new(
        base_url: ServoUrl,
        options: ScriptFetchOptions,
        owner: Option<Trusted<GlobalScope>>,
    ) -> Self {
        ModuleScript {
            base_url,
            options,
            owner,
        }
    }
}

pub(crate) type ModuleRequest = (ServoUrl, ModuleType);

#[derive(JSTraceable)]
pub(crate) enum ModuleStatus {
    #[expect(clippy::type_complexity)]
    Fetching(#[no_trace] Vec<Box<dyn FnOnce(&mut JSContext, Option<Rc<ModuleTree>>)>>),
    Loaded(Rc<ModuleTree>),
}

#[derive(JSTraceable, MallocSizeOf)]
pub(crate) struct ModuleTree {
    #[no_trace]
    url: ServoUrl,
    #[ignore_malloc_size_of = "mozjs"]
    record: OnceCell<ModuleObject>,
    #[ignore_malloc_size_of = "mozjs"]
    parse_error: OnceCell<RethrowError>,
    #[ignore_malloc_size_of = "mozjs"]
    rethrow_error: DomRefCell<Option<RethrowError>>,
    #[no_trace]
    loaded_modules: DomRefCell<IndexMap<String, ServoUrl>>,
}

impl ModuleTree {
    pub(crate) fn get_url(&self) -> ServoUrl {
        self.url.clone()
    }

    pub(crate) fn get_record(&self) -> Option<&ModuleObject> {
        self.record.get()
    }

    pub(crate) fn get_parse_error(&self) -> Option<&RethrowError> {
        self.parse_error.get()
    }

    pub(crate) fn get_rethrow_error(&self) -> &DomRefCell<Option<RethrowError>> {
        &self.rethrow_error
    }

    pub(crate) fn set_rethrow_error(&self, rethrow_error: RethrowError) {
        *self.rethrow_error.borrow_mut() = Some(rethrow_error);
    }

    pub(crate) fn find_descendant_inside_module_map(
        &self,
        global: &GlobalScope,
        specifier: &String,
        module_type: ModuleType,
    ) -> Option<Rc<ModuleTree>> {
        self.loaded_modules
            .borrow()
            .get(specifier)
            .and_then(|url| global.module_tree_for_request_if_loaded(&(url.clone(), module_type)))
    }

    pub(crate) fn insert_module_dependency(
        &self,
        module: &Rc<ModuleTree>,
        module_request_specifier: String,
    ) {
        // Store the url which is used to retrieve the module from module map when needed.
        let url = module.url.clone();
        match self
            .loaded_modules
            .borrow_mut()
            .entry(module_request_specifier)
        {
            // a. If referrer.[[LoadedModules]] contains a LoadedModuleRequest Record record such that
            // ModuleRequestsEqual(record, moduleRequest) is true, then
            IndexMapEntry::Occupied(entry) => {
                // i. Assert: record.[[Module]] and result.[[Value]] are the same Module Record.
                assert_eq!(*entry.get(), url);
            },
            // b. Else,
            IndexMapEntry::Vacant(entry) => {
                // i. Append the LoadedModuleRequest Record { [[Specifier]]: moduleRequest.[[Specifier]],
                // [[Attributes]]: moduleRequest.[[Attributes]], [[Module]]: result.[[Value]] } to referrer.[[LoadedModules]].
                entry.insert(url);
            },
        }
    }
}

impl ModuleTree {
    #[expect(unsafe_code)]
    #[expect(clippy::too_many_arguments)]
    /// <https://html.spec.whatwg.org/multipage/#creating-a-javascript-module-script>
    fn create_a_javascript_module_script(
        cx: &mut JSContext,
        source: Cow<'_, str>,
        global: &GlobalScope,
        url: &ServoUrl,
        options: ScriptFetchOptions,
        external: bool,
        line_number: u32,
        introduction_type: Option<&'static CStr>,
    ) -> Self {
        let mut realm = AutoRealm::new(
            cx,
            NonNull::new(global.reflector().get_jsobject().get()).unwrap(),
        );
        let cx = &mut *realm;

        let owner = Trusted::new(global);

        // Step 2. Let script be a new module script that this algorithm will subsequently initialize.
        // Step 6. Set script's parse error and error to rethrow to null.
        let module = ModuleTree {
            url: url.clone(),
            record: OnceCell::new(),
            parse_error: OnceCell::new(),
            rethrow_error: DomRefCell::new(None),
            loaded_modules: DomRefCell::new(IndexMap::new()),
        };

        let compile_options = fill_compile_options(
            cx,
            url.as_str(),
            introduction_type,
            ErrorReporting::Unmuted,
            true, // noScriptRval
            line_number,
        );

        let mut source = if let Some(unminified_js_dir) = global.unminified_js_dir() {
            let mut module_source = ScriptSource {
                source,
                external,
                url,
            };
            unminify_js(&mut module_source, unminified_js_dir);
            transform_str_to_source_text(&module_source.source)
        } else {
            transform_str_to_source_text(&source)
        };

        unsafe {
            // Step 7. Let result be ParseModule(source, settings's realm, script).
            rooted!(&in(cx) let mut module_script: *mut JSObject = std::ptr::null_mut());
            module_script.set(CompileModule1(cx, compile_options.ptr, &mut source));

            // Step 8. If result is a list of errors, then:
            if module_script.is_null() {
                warn!("fail to compile module script of {}", url);

                // Step 8.1. Set script's parse error to result[0].
                let _ = module
                    .parse_error
                    .set(RethrowError::from_pending_exception(cx));

                // Step 8.2. Return script.
                return module;
            }

            // Step 3. Set script's settings object to settings.
            // Step 4. Set script's base URL to baseURL.
            // Step 5. Set script's fetch options to options.
            let module_script_data = Rc::new(ModuleScript::new(url.clone(), options, Some(owner)));

            SetModulePrivate(
                module_script.get(),
                &PrivateValue(Rc::into_raw(module_script_data) as *const _),
            );

            // Step 9. Set script's record to result.
            let _ = module.record.set(ModuleObject::new(module_script.handle()));
        }

        // Step 10. Return script.
        module
    }

    #[expect(unsafe_code)]
    /// <https://html.spec.whatwg.org/multipage/#creating-a-json-module-script>
    fn create_a_json_module_script(
        cx: &mut JSContext,
        source: &str,
        global: &GlobalScope,
        url: &ServoUrl,
        introduction_type: Option<&'static CStr>,
    ) -> Self {
        let mut realm = AutoRealm::new(
            cx,
            NonNull::new(global.reflector().get_jsobject().get()).unwrap(),
        );
        let cx = &mut *realm;

        // Step 1. Let script be a new module script that this algorithm will subsequently initialize.
        // Step 4. Set script's parse error and error to rethrow to null.
        let module = ModuleTree {
            url: url.clone(),
            record: OnceCell::new(),
            parse_error: OnceCell::new(),
            rethrow_error: DomRefCell::new(None),
            loaded_modules: DomRefCell::new(IndexMap::new()),
        };

        // Step 2. Set script's settings object to settings.
        // Step 3. Set script's base URL and fetch options to null.
        // Note: We don't need to call `SetModulePrivate` for json scripts

        let compile_options = fill_compile_options(
            cx,
            url.as_str(),
            introduction_type,
            ErrorReporting::Unmuted,
            true, // noScriptRval
            1,    // lineno
        );

        rooted!(&in(cx) let mut module_script: *mut JSObject = std::ptr::null_mut());

        unsafe {
            // Step 5. Let result be ParseJSONModule(source).
            module_script.set(CompileJsonModule1(
                cx,
                compile_options.ptr,
                &mut transform_str_to_source_text(source),
            ));
        }

        // If this throws an exception, catch it, and set script's parse error to that exception, and return script.
        if module_script.is_null() {
            warn!("fail to compile module script of {}", url);

            let _ = module
                .parse_error
                .set(RethrowError::from_pending_exception(cx));
            return module;
        }

        // Step 6. Set script's record to result.
        let _ = module.record.set(ModuleObject::new(module_script.handle()));

        // Step 7. Return script.
        module
    }

    /// Execute the provided module, storing the evaluation return value in the provided
    /// mutable handle.
    #[expect(unsafe_code)]
    pub(crate) fn execute_module(
        &self,
        cx: &mut JSContext,
        global: &GlobalScope,
        module_record: HandleObject,
        mut eval_result: MutableHandleValue,
    ) -> Result<(), RethrowError> {
        let mut realm = AutoRealm::new(
            cx,
            NonNull::new(global.reflector().get_jsobject().get()).unwrap(),
        );
        let cx = &mut *realm;

        unsafe {
            let ok = ModuleEvaluate(cx, module_record, eval_result.reborrow());
            assert!(ok, "module evaluation failed");

            rooted!(&in(cx) let mut evaluation_promise = ptr::null_mut::<JSObject>());
            if eval_result.is_object() {
                evaluation_promise.set(eval_result.to_object());
            }

            let throw_result = ThrowOnModuleEvaluationFailure(
                cx,
                evaluation_promise.handle(),
                ModuleErrorBehaviour::ThrowModuleErrorsSync,
            );
            if !throw_result {
                warn!("fail to evaluate module");

                Err(RethrowError::from_pending_exception(cx))
            } else {
                debug!("module evaluated successfully");
                Ok(())
            }
        }
    }

    #[expect(unsafe_code)]
    pub(crate) fn report_error(&self, cx: &mut JSContext, global: &GlobalScope) {
        let module_error = self.rethrow_error.borrow();

        if let Some(exception) = &*module_error {
            let mut realm = enter_auto_realm(cx, global);
            let cx = &mut realm.current_realm();

            unsafe {
                JS_SetPendingException(cx, exception.handle(), ExceptionStackBehavior::Capture);
            }
            report_pending_exception(cx);
        }
    }

    /// <https://html.spec.whatwg.org/multipage/#resolve-a-module-specifier>
    pub(crate) fn resolve_module_specifier(
        global: &GlobalScope,
        script: Option<&ModuleScript>,
        specifier: DOMString,
    ) -> Fallible<ServoUrl> {
        // Step 1~3 to get settingsObject and baseURL
        let script_global = script.and_then(|s| s.owner.as_ref().map(|o| o.root()));
        // Step 1. Let settingsObject and baseURL be null.
        let (global, base_url): (&GlobalScope, &ServoUrl) = match script {
            // Step 2. If referringScript is not null, then:
            // Set settingsObject to referringScript's settings object.
            // Set baseURL to referringScript's base URL.
            Some(s) => (script_global.as_ref().map_or(global, |g| g), &s.base_url),
            // Step 3. Otherwise:
            // Set settingsObject to the current settings object.
            // Set baseURL to settingsObject's API base URL.
            // FIXME(#37553): Is this the correct current settings object?
            None => (global, &global.api_base_url()),
        };

        // Step 4. Let importMap be an empty import map.
        // Step 5. If settingsObject's global object implements Window, then set importMap to settingsObject's
        // global object's import map.
        let import_map = if global.is::<Window>() {
            Some(global.import_map())
        } else {
            None
        };
        let specifier = &specifier.str();

        // Step 6. Let serializedBaseURL be baseURL, serialized.
        let serialized_base_url = base_url.as_str();
        // Step 7. Let asURL be the result of resolving a URL-like module specifier given specifier and baseURL.
        let as_url = resolve_url_like_module_specifier(specifier, base_url);
        // Step 8. Let normalizedSpecifier be the serialization of asURL, if asURL is non-null;
        // otherwise, specifier.
        let normalized_specifier = match &as_url {
            Some(url) => url.as_str(),
            None => specifier,
        };

        // Step 9. Let result be a URL-or-null, initially null.
        let mut result = None;
        if let Some(map) = import_map {
            // Step 10. For each scopePrefix → scopeImports of importMap's scopes:
            for (prefix, imports) in &map.scopes {
                // Step 10.1 If scopePrefix is serializedBaseURL, or if scopePrefix ends with U+002F (/)
                // and scopePrefix is a code unit prefix of serializedBaseURL, then:
                let prefix = prefix.as_str();
                if prefix == serialized_base_url ||
                    (serialized_base_url.starts_with(prefix) && prefix.ends_with('\u{002f}'))
                {
                    // Step 10.1.1 Let scopeImportsMatch be the result of resolving an imports match
                    // given normalizedSpecifier, asURL, and scopeImports.
                    let scope_imports_match =
                        resolve_imports_match(normalized_specifier, as_url.as_ref(), imports)?;

                    // Step 10.1.2 If scopeImportsMatch is not null, then set result to scopeImportsMatch, and break.
                    if scope_imports_match.is_some() {
                        result = scope_imports_match;
                        break;
                    }
                }
            }

            // Step 11. If result is null, set result to the result of resolving an imports match given
            // normalizedSpecifier, asURL, and importMap's imports.
            if result.is_none() {
                result =
                    resolve_imports_match(normalized_specifier, as_url.as_ref(), &map.imports)?;
            }
        }

        // Step 12. If result is null, set it to asURL.
        if result.is_none() {
            result = as_url.clone();
        }

        // Step 13. If result is not null, then:
        match result {
            Some(result) => {
                // Step 13.1 Add module to resolved module set given settingsObject, serializedBaseURL,
                // normalizedSpecifier, and asURL.
                global.add_module_to_resolved_module_set(
                    serialized_base_url,
                    normalized_specifier,
                    as_url.clone(),
                );
                // Step 13.2 Return result.
                Ok(result)
            },
            // Step 14. Throw a TypeError indicating that specifier was a bare specifier,
            // but was not remapped to anything by importMap.
            None => Err(Error::Type(
                c"Specifier was a bare specifier, but was not remapped to anything by importMap."
                    .to_owned(),
            )),
        }
    }
}

#[derive(JSTraceable, MallocSizeOf)]
pub(crate) struct ModuleHandler {
    #[ignore_malloc_size_of = "Measuring trait objects is hard"]
    task: DomRefCell<Option<Box<dyn NonSendTaskBox>>>,
}

impl ModuleHandler {
    pub(crate) fn new_boxed(task: Box<dyn NonSendTaskBox>) -> Box<dyn Callback> {
        Box::new(Self {
            task: DomRefCell::new(Some(task)),
        })
    }
}

impl Callback for ModuleHandler {
    fn callback(&self, cx: &mut CurrentRealm, _v: HandleValue) {
        let task = self.task.borrow_mut().take().unwrap();
        task.run_box(cx);
    }
}

/// The context required for asynchronously loading an external module script source.
struct ModuleContext {
    /// The owner of the module that initiated the request.
    owner: Trusted<GlobalScope>,
    /// The response body received to date.
    data: Vec<u8>,
    /// The response metadata received to date.
    metadata: Option<Metadata>,
    /// Url and type of the requested module.
    module_request: ModuleRequest,
    /// Options for the current script fetch
    options: ScriptFetchOptions,
    /// Indicates whether the request failed, and why
    status: Result<(), NetworkError>,
    /// `introductionType` value to set in the `CompileOptionsWrapper`.
    introduction_type: Option<&'static CStr>,
    /// <https://html.spec.whatwg.org/multipage/#policy-container>
    policy_container: Option<PolicyContainer>,
}

impl FetchResponseListener for ModuleContext {
    // TODO(cybai): Perhaps add custom steps to perform fetch here?
    fn process_request_body(&mut self, _: RequestId) {}

    fn process_response(
        &mut self,
        _: &mut js::context::JSContext,
        _: RequestId,
        metadata: Result<FetchMetadata, NetworkError>,
    ) {
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
                Err(NetworkError::ResourceLoadError(
                    "No http status code received".to_owned(),
                ))
            } else if status.is_success() {
                Ok(())
            } else {
                Err(NetworkError::ResourceLoadError(format!(
                    "HTTP error code {}",
                    status.code()
                )))
            }
        };
    }

    fn process_response_chunk(
        &mut self,
        _: &mut js::context::JSContext,
        _: RequestId,
        mut chunk: Vec<u8>,
    ) {
        if self.status.is_ok() {
            self.data.append(&mut chunk);
        }
    }

    /// <https://html.spec.whatwg.org/multipage/#fetch-a-single-module-script>
    /// Step 13
    fn process_response_eof(
        mut self,
        cx: &mut js::context::JSContext,
        _: RequestId,
        response: Result<(), NetworkError>,
        timing: ResourceFetchTiming,
    ) {
        let global = self.owner.root();
        let (_url, module_type) = &self.module_request;

        if !global.is::<WorkletGlobalScope>() {
            network_listener::submit_timing(cx, &self, &response, &timing);
        }

        let module_map = global.module_map();

        // Step 1. If any of the following are true: bodyBytes is null or failure; or response's
        // status is not an ok status, then:
        if let (Err(error), _) | (_, Err(error)) = (response.as_ref(), self.status.as_ref()) {
            error!("Fetching module script failed {:?}", error);
            // Step 1.1. Let callbacks be moduleMap[(url, moduleType)].
            // Step 1.2. Remove moduleMap[(url, moduleType)].
            let Some(ModuleStatus::Fetching(callbacks)) =
                module_map.safe_borrow_mut(cx).remove(&self.module_request)
            else {
                return error!("Processing response for a non pending module request");
            };

            // Step 1.3. For each callback of callbacks: run callback given null.
            for callback in callbacks {
                (callback)(cx, None);
            }

            // Step 1.4. Return.
            return;
        }

        let metadata = self.metadata.take().unwrap();

        // The processResponseConsumeBody steps defined inside
        // [run a worker](https://html.spec.whatwg.org/multipage/#run-a-worker)
        if let Some(policy_container) = self.policy_container {
            let workerscope = global.downcast::<WorkerGlobalScope>().expect(
                "We only need a policy container when initializing a worker's globalscope.",
            );
            workerscope.process_response_for_workerscope(&metadata, &policy_container);
        }

        let final_url = metadata.final_url;

        // Step 2. Let mimeType be the result of extracting a MIME type from response's header list.
        let mime_type: Option<Mime> = metadata.content_type.map(Serde::into_inner).map(Into::into);

        // Step 3. Let moduleScript be null.
        let mut module_script = None;

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

        // TODO handle CSS module scripts on the next mozjs ESR bump.

        if let Some(mime) = mime_type {
            // Step 7.1 Let sourceText be the result of UTF-8 decoding bodyBytes.
            let (mut source_text, _) = UTF_8.decode_with_bom_removal(&self.data);

            // Step 7.2 If mimeType is a JavaScript MIME type and moduleType is "javascript-or-wasm", then set moduleScript
            // to the result of creating a JavaScript module script given sourceText, settingsObject, response's URL, and options.
            if SCRIPT_JS_MIMES.contains(&mime.essence_str()) &&
                matches!(module_type, ModuleType::JavaScript)
            {
                if let Some(window) = global.downcast::<Window>() &&
                    let Some(script_souce) = window.local_script_source()
                {
                    substitute_with_local_script(script_souce, &mut source_text, &final_url);
                }

                let module_tree = Rc::new(ModuleTree::create_a_javascript_module_script(
                    cx,
                    source_text,
                    &global,
                    &final_url,
                    self.options,
                    true,
                    1,
                    self.introduction_type,
                ));
                module_script = Some(module_tree);
            } else if MimeClassifier::is_json(&mime) && matches!(module_type, ModuleType::JSON) {
                // Step 7.4 If mimeType is a JSON MIME type and moduleType is "json",
                // then set moduleScript to the result of creating a JSON module script given sourceText and settingsObject.
                let module_tree = Rc::new(ModuleTree::create_a_json_module_script(
                    cx,
                    &source_text,
                    &global,
                    &final_url,
                    self.introduction_type,
                ));
                module_script = Some(module_tree);
            }
        }

        let callbacks = match module_map
            .safe_borrow_mut(cx)
            .entry(self.module_request.clone())
        {
            Entry::Occupied(mut entry) => {
                // Step 9. If moduleScript is null, then remove moduleMap[(url, moduleType)];
                // otherwise set moduleMap[(url, moduleType)] to moduleScript.
                let old_value = match module_script.as_ref() {
                    None => entry.remove(),
                    Some(module_script) => {
                        entry.insert(ModuleStatus::Loaded(module_script.clone()))
                    },
                };

                match old_value {
                    ModuleStatus::Loaded(_) => {
                        return error!("Processing response for a non pending module request");
                    },
                    ModuleStatus::Fetching(callbacks) => callbacks,
                }
            },
            Entry::Vacant(_) => {
                return error!("Processing response for a non pending module request");
            },
        };

        // Step 10. For each callback of callbacks: run callback given moduleScript.
        for callback in callbacks {
            (callback)(cx, module_script.clone());
        }
    }

    fn process_csp_violations(
        &mut self,
        cx: &mut js::context::JSContext,
        _request_id: RequestId,
        violations: Vec<Violation>,
    ) {
        let global = self.owner.root();
        if let Some(scope) = global.downcast::<DedicatedWorkerGlobalScope>() {
            scope.report_csp_violations(violations);
        } else if let Some(scope) = global.downcast::<SharedWorkerGlobalScope>() {
            scope.report_csp_violations(violations);
        } else {
            global.report_csp_violations(cx, violations, None, None);
        }
    }

    fn process_content_length(&mut self, _request_id: RequestId, size: usize) {
        self.data.reserve(size - self.data.len());
    }
}

impl ResourceTimingListener for ModuleContext {
    fn resource_timing_information(&self) -> (InitiatorType, ServoUrl) {
        let initiator_type = InitiatorType::LocalName("module".to_string());
        let (url, _) = &self.module_request;
        (initiator_type, url.clone())
    }

    fn resource_timing_global(&self) -> DomRoot<GlobalScope> {
        self.owner.root()
    }
}

#[expect(unsafe_code)]
#[expect(non_snake_case)]
/// A function to register module hooks (e.g. listening on resolving modules,
/// getting module metadata, getting script private reference and resolving dynamic import)
pub(crate) unsafe fn EnsureModuleHooksInitialized(rt: *mut JSRuntime) {
    unsafe {
        if GetModuleResolveHook(rt).is_some() {
            return;
        }

        SetModuleResolveHook(rt, Some(HostResolveImportedModule));
        SetModuleMetadataHook(rt, Some(HostPopulateImportMeta));
        SetScriptPrivateReferenceHooks(
            rt,
            Some(host_add_ref_top_level_script),
            Some(host_release_top_level_script),
        );
        SetModuleDynamicImportHook(rt, Some(host_import_module_dynamically));
    }
}

#[expect(unsafe_code)]
unsafe extern "C" fn host_add_ref_top_level_script(value: *const Value) {
    let val = unsafe { Rc::from_raw((*value).to_private() as *const ModuleScript) };
    mem::forget(val.clone());
    mem::forget(val);
}

#[expect(unsafe_code)]
unsafe extern "C" fn host_release_top_level_script(value: *const Value) {
    let _val = unsafe { Rc::from_raw((*value).to_private() as *const ModuleScript) };
}

#[expect(unsafe_code)]
/// <https://tc39.es/ecma262/#sec-hostimportmoduledynamically>
/// <https://html.spec.whatwg.org/multipage/#hostimportmoduledynamically(referencingscriptormodule,-specifier,-promisecapability)>
pub(crate) unsafe extern "C" fn host_import_module_dynamically(
    cx: *mut RawJSContext,
    reference_private: RawHandleValue,
    specifier: RawHandle<*mut JSObject>,
    promise: RawHandle<*mut JSObject>,
) -> bool {
    // SAFETY: it is safe to construct a JSContext from engine hook.
    let mut cx = unsafe { JSContext::from_ptr(NonNull::new(cx).unwrap()) };
    let cx = &mut cx;
    let promise = Promise::new_with_js_promise(cx, unsafe { Handle::from_raw(promise) });

    let jsstr = unsafe { GetModuleRequestSpecifier(cx, Handle::from_raw(specifier)) };
    let module_type = unsafe { GetModuleRequestType(cx, Handle::from_raw(specifier)) };
    let specifier = unsafe { jsstr_to_string(cx, NonNull::new(jsstr).unwrap()) };

    let mut realm = CurrentRealm::assert(cx);
    let payload = Payload::PromiseRecord(promise);
    host_load_imported_module(
        &mut realm,
        None,
        reference_private,
        specifier,
        module_type,
        None,
        payload,
    );

    true
}

#[derive(Clone, Debug, JSTraceable, MallocSizeOf)]
/// <https://html.spec.whatwg.org/multipage/#script-fetch-options>
pub(crate) struct ScriptFetchOptions {
    pub(crate) integrity_metadata: String,
    #[no_trace]
    pub(crate) credentials_mode: CredentialsMode,
    pub(crate) cryptographic_nonce: String,
    #[no_trace]
    pub(crate) parser_metadata: ParserMetadata,
    #[no_trace]
    pub(crate) referrer_policy: ReferrerPolicy,
    /// <https://html.spec.whatwg.org/multipage/#concept-script-fetch-options-render-blocking>
    /// The boolean value of render-blocking used for the initial fetch and for fetching any imported modules.
    /// Unless otherwise stated, its value is false.
    pub(crate) render_blocking: bool,
}

impl ScriptFetchOptions {
    /// <https://html.spec.whatwg.org/multipage/#default-classic-script-fetch-options>
    pub(crate) fn default_classic_script() -> ScriptFetchOptions {
        Self {
            cryptographic_nonce: String::new(),
            integrity_metadata: String::new(),
            parser_metadata: ParserMetadata::NotParserInserted,
            credentials_mode: CredentialsMode::CredentialsSameOrigin,
            referrer_policy: ReferrerPolicy::EmptyString,
            render_blocking: false,
        }
    }

    /// <https://html.spec.whatwg.org/multipage/#descendant-script-fetch-options>
    pub(crate) fn descendant_fetch_options(
        &self,
        url: &ServoUrl,
        global: &GlobalScope,
    ) -> ScriptFetchOptions {
        // Step 2. Let integrity be the result of resolving a module integrity metadata with url and settingsObject.
        let integrity = global.import_map().resolve_a_module_integrity_metadata(url);

        // Step 1. Let newOptions be a copy of originalOptions.
        // TODO Step 4. Set newOptions's fetch priority to "auto".
        Self {
            // Step 3. Set newOptions's integrity metadata to integrity.
            integrity_metadata: integrity,
            cryptographic_nonce: self.cryptographic_nonce.clone(),
            credentials_mode: self.credentials_mode,
            parser_metadata: self.parser_metadata,
            referrer_policy: self.referrer_policy,
            render_blocking: self.render_blocking,
        }
    }
}

#[expect(unsafe_code)]
pub(crate) unsafe fn module_script_from_reference_private(
    reference_private: &RawHandle<JSVal>,
) -> Option<&ModuleScript> {
    if reference_private.get().is_undefined() {
        return None;
    }
    unsafe { (reference_private.get().to_private() as *const ModuleScript).as_ref() }
}

#[expect(unsafe_code)]
#[expect(non_snake_case)]
/// <https://tc39.es/ecma262/#sec-HostLoadImportedModule>
/// <https://html.spec.whatwg.org/multipage/#hostloadimportedmodule>
unsafe extern "C" fn HostResolveImportedModule(
    cx: *mut RawJSContext,
    reference_private: RawHandleValue,
    specifier: RawHandle<*mut JSObject>,
) -> *mut JSObject {
    // SAFETY: it is safe to construct a JSContext from engine hook.
    let mut cx = unsafe { JSContext::from_ptr(NonNull::new(cx).unwrap()) };
    let mut realm = CurrentRealm::assert(&mut cx);
    let global_scope = GlobalScope::from_current_realm(&mut realm);

    let cx = &mut realm;

    // Step 5.
    let module_data = unsafe { module_script_from_reference_private(&reference_private) };
    let jsstr = unsafe { GetModuleRequestSpecifier(cx, Handle::from_raw(specifier)) };
    let module_type = unsafe { GetModuleRequestType(cx, Handle::from_raw(specifier)) };

    let specifier = unsafe { jsstr_to_string(cx, NonNull::new(jsstr).unwrap()) };
    let url = ModuleTree::resolve_module_specifier(
        &global_scope,
        module_data,
        DOMString::from(specifier),
    );

    // Step 6.
    assert!(url.is_ok());

    let parsed_url = url.unwrap();

    // Step 4 & 7.
    let module_tree = global_scope.module_tree_for_request_if_loaded(&(parsed_url, module_type));

    let module = module_tree.expect("Attempted to link a module not found inside module map");

    let fetched_module_object = module.get_record();

    // Step 8.
    assert!(fetched_module_object.is_some());

    // Step 10.
    if let Some(record) = fetched_module_object {
        return record.handle().get();
    }

    unreachable!()
}

// https://searchfox.org/firefox-esr140/rev/3fccb0ec900b931a1a752b02eafab1fb9652d9b9/js/loader/ModuleLoaderBase.h#560
const SLOT_MODULEPRIVATE: usize = 0;

#[expect(unsafe_code)]
#[expect(non_snake_case)]
/// <https://tc39.es/ecma262/#sec-hostgetimportmetaproperties>
/// <https://html.spec.whatwg.org/multipage/#hostgetimportmetaproperties>
unsafe extern "C" fn HostPopulateImportMeta(
    cx: *mut RawJSContext,
    reference_private: RawHandleValue,
    meta_object: RawHandle<*mut JSObject>,
) -> bool {
    // SAFETY: it is safe to construct a JSContext from engine hook.
    let mut cx = unsafe { JSContext::from_ptr(NonNull::new(cx).unwrap()) };
    let mut realm = CurrentRealm::assert(&mut cx);
    let global_scope = GlobalScope::from_current_realm(&mut realm);

    // Step 2.
    let base_url = match unsafe { module_script_from_reference_private(&reference_private) } {
        Some(module_data) => module_data.base_url.clone(),
        None => global_scope.api_base_url(),
    };

    unsafe {
        let url_string = JS_NewStringCopyN(
            &mut cx,
            base_url.as_str().as_ptr() as *const _,
            base_url.as_str().len(),
        );
        rooted!(&in(cx) let url_string = url_string);

        // Step 3.
        if !JS_DefineProperty4(
            &mut cx,
            Handle::from_raw(meta_object),
            c"url".as_ptr(),
            url_string.handle(),
            JSPROP_ENUMERATE.into(),
        ) {
            return false;
        }

        // Step 5. Let resolveFunction be ! CreateBuiltinFunction(steps, 1, "resolve", « »).
        let resolve_function = DefineFunctionWithReserved(
            &mut cx,
            meta_object.get(),
            c"resolve".as_ptr(),
            Some(import_meta_resolve),
            1,
            JSPROP_ENUMERATE.into(),
        );

        rooted!(&in(cx) let obj = JS_GetFunctionObject(resolve_function));
        assert!(!obj.is_null());
        SetFunctionNativeReserved(
            obj.get(),
            SLOT_MODULEPRIVATE,
            &reference_private.get() as *const _,
        );
    }

    true
}

#[expect(unsafe_code)]
unsafe extern "C" fn import_meta_resolve(cx: *mut RawJSContext, argc: u32, vp: *mut JSVal) -> bool {
    // SAFETY: it is safe to construct a JSContext from engine hook.
    let mut cx = unsafe { JSContext::from_ptr(ptr::NonNull::new(cx).unwrap()) };
    let mut realm = CurrentRealm::assert(&mut cx);
    let global_scope = GlobalScope::from_current_realm(&mut realm);

    let cx = &mut realm;

    let args = unsafe { CallArgs::from_vp(vp, argc) };

    rooted!(&in(cx) let module_private = unsafe { *GetFunctionNativeReserved(args.callee(), SLOT_MODULEPRIVATE) });
    let reference_private = module_private.handle().into();
    let module_data = unsafe { module_script_from_reference_private(&reference_private) };

    // https://html.spec.whatwg.org/multipage/#hostgetimportmetaproperties

    // Step 4.1. Set specifier to ? ToString(specifier).
    let specifier = unsafe {
        let value = HandleValue::from_raw(args.get(0));

        match NonNull::new(ToString(cx, value)) {
            Some(jsstr) => jsstr_to_string(cx, jsstr).into(),
            None => return false,
        }
    };

    // Step 4.2. Let url be the result of resolving a module specifier given moduleScript and specifier.
    let url = ModuleTree::resolve_module_specifier(&global_scope, module_data, specifier);

    match url {
        Ok(url) => {
            // Step 4.3. Return the serialization of url.
            url.as_str()
                .safe_to_jsval(cx, unsafe { MutableHandleValue::from_raw(args.rval()) });
            true
        },
        Err(error) => {
            let resolution_error = gen_type_error(cx, &global_scope, error);

            unsafe {
                JS_SetPendingException(
                    cx,
                    resolution_error.handle(),
                    ExceptionStackBehavior::Capture,
                );
            }
            false
        },
    }
}

#[expect(clippy::too_many_arguments)]
/// <https://html.spec.whatwg.org/multipage/#fetch-a-module-worker-script-tree>
/// <https://html.spec.whatwg.org/multipage/#fetch-a-worklet/module-worker-script-graph>
pub(crate) fn fetch_a_module_script_graph(
    cx: &mut JSContext,
    global: &GlobalScope,
    url: UrlWithBlobClaim,
    fetch_client: RequestClient,
    destination: Destination,
    referrer: Referrer,
    credentials_mode: CredentialsMode,
    introduction_type: Option<&'static CStr>,
    on_complete: impl FnOnce(&mut JSContext, Option<Rc<ModuleTree>>) + Clone + 'static,
) {
    let global_scope = DomRoot::from_ref(global);

    // Step 1. Let options be a script fetch options whose cryptographic nonce
    // is the empty string, integrity metadata is the empty string, parser
    // metadata is "not-parser-inserted", credentials mode is credentialsMode,
    // referrer policy is the empty string, and fetch priority is "auto".
    let options = ScriptFetchOptions {
        integrity_metadata: "".into(),
        credentials_mode,
        cryptographic_nonce: "".into(),
        parser_metadata: ParserMetadata::NotParserInserted,
        referrer_policy: ReferrerPolicy::EmptyString,
        render_blocking: false,
    };

    // Step 2. Fetch a single module script given url, fetchClient, destination, options,
    // settingsObject, "client", true, and onSingleFetchComplete as defined below.
    fetch_a_single_module_script(
        cx,
        url,
        fetch_client.clone(),
        global,
        destination,
        options,
        referrer,
        None,
        true,
        introduction_type,
        move |cx, module_tree| {
            let Some(module) = module_tree else {
                // Step 1.1. If result is null, run onComplete given null, and abort these steps.
                return on_complete(cx, None);
            };

            // Step 1.2. Fetch the descendants of and link result given fetchClient, destination,
            // and onComplete.
            fetch_the_descendants_and_link_module_script(
                cx,
                &global_scope,
                module,
                fetch_client,
                destination,
                on_complete,
            );
        },
    );
}

/// <https://html.spec.whatwg.org/multipage/#fetch-a-module-script-tree>
pub(crate) fn fetch_an_external_module_script(
    cx: &mut JSContext,
    url: UrlWithBlobClaim,
    global: &GlobalScope,
    options: ScriptFetchOptions,
    on_complete: impl FnOnce(&mut JSContext, Option<Rc<ModuleTree>>) + Clone + 'static,
) {
    let referrer = global.get_referrer();
    let fetch_client = global.request_client(Some(cx.no_gc()));
    let global_scope = DomRoot::from_ref(global);

    // Step 1. Fetch a single module script given url, settingsObject, "script", options, settingsObject, "client", true,
    // and with the following steps given result:
    fetch_a_single_module_script(
        cx,
        url,
        fetch_client.clone(),
        global,
        Destination::Script,
        options,
        referrer,
        None,
        true,
        Some(IntroductionType::SRC_SCRIPT),
        move |cx, module_tree| {
            let Some(module) = module_tree else {
                // Step 1.1. If result is null, run onComplete given null, and abort these steps.
                return on_complete(cx, None);
            };

            // Step 1.2. Fetch the descendants of and link result given settingsObject, "script", and onComplete.
            fetch_the_descendants_and_link_module_script(
                cx,
                &global_scope,
                module,
                fetch_client,
                Destination::Script,
                on_complete,
            );
        },
    );
}

/// <https://html.spec.whatwg.org/multipage/#fetch-a-modulepreload-module-script-graph>
pub(crate) fn fetch_a_modulepreload_module(
    cx: &mut JSContext,
    url: UrlWithBlobClaim,
    destination: Destination,
    global: &GlobalScope,
    options: ScriptFetchOptions,
    on_complete: impl FnOnce(&mut JSContext, bool) + 'static,
) {
    let referrer = global.get_referrer();
    let fetch_client = global.request_client(Some(cx.no_gc()));
    let global_scope = DomRoot::from_ref(global);

    // Note: There is a specification inconsistency, `fetch_a_single_module_script` doesn't allow
    // fetching top level JSON/CSS module scripts, but should be possible when preloading.
    let module_type = if let Destination::Json = destination {
        Some(ModuleType::JSON)
    } else {
        None
    };

    // Step 1. Fetch a single module script given url, settingsObject, destination, options, settingsObject,
    // "client", true, and with the following steps given result:
    fetch_a_single_module_script(
        cx,
        url,
        fetch_client.clone(),
        global,
        destination,
        options,
        referrer,
        module_type,
        true,
        Some(IntroductionType::SRC_SCRIPT),
        move |cx, result| {
            // Step 1. Run onComplete given result.
            on_complete(cx, result.is_none());

            // Step 2. Assert: settingsObject's global object implements Window.
            assert!(global_scope.is::<Window>());

            // Step 3. If result is not null, optionally fetch the descendants of and link result
            // given settingsObject, destination, and an empty algorithm.
            if pref!(dom_allow_preloading_module_descendants) &&
                let Some(module) = result
            {
                fetch_the_descendants_and_link_module_script(
                    cx,
                    &global_scope,
                    module,
                    fetch_client,
                    destination,
                    |_, _| {},
                );
            }
        },
    );
}

#[expect(clippy::too_many_arguments)]
/// <https://html.spec.whatwg.org/multipage/#fetch-an-inline-module-script-graph>
pub(crate) fn fetch_inline_module_script(
    cx: &mut JSContext,
    global: &GlobalScope,
    module_script_text: Cow<'_, str>,
    url: ServoUrl,
    options: ScriptFetchOptions,
    line_number: u32,
    introduction_type: Option<&'static CStr>,
    on_complete: impl FnOnce(&mut JSContext, Option<Rc<ModuleTree>>) + Clone + 'static,
) {
    // Step 1. Let script be the result of creating a JavaScript module script using sourceText, settingsObject, baseURL, and options.
    let module_tree = Rc::new(ModuleTree::create_a_javascript_module_script(
        cx,
        module_script_text,
        global,
        &url,
        options,
        false,
        line_number,
        introduction_type,
    ));
    let fetch_client = global.request_client(Some(cx.no_gc()));

    // Step 2. Fetch the descendants of and link script, given settingsObject, "script", and onComplete.
    fetch_the_descendants_and_link_module_script(
        cx,
        global,
        module_tree,
        fetch_client,
        Destination::Script,
        on_complete,
    );
}

#[expect(unsafe_code)]
/// <https://html.spec.whatwg.org/multipage/#fetch-the-descendants-of-and-link-a-module-script>
fn fetch_the_descendants_and_link_module_script(
    cx: &mut JSContext,
    global: &GlobalScope,
    module_script: Rc<ModuleTree>,
    fetch_client: RequestClient,
    destination: Destination,
    on_complete: impl FnOnce(&mut JSContext, Option<Rc<ModuleTree>>) + Clone + 'static,
) {
    // Step 1. Let record be moduleScript's record.
    // Step 2. If record is null, then:
    if module_script.get_record().is_none() {
        let parse_error = module_script.get_parse_error().cloned();

        // Step 2.1. Set moduleScript's error to rethrow to moduleScript's parse error.
        module_script.set_rethrow_error(parse_error.unwrap());

        // Step 2.2. Run onComplete given moduleScript.
        on_complete(cx, Some(module_script));

        // Step 2.3. Return.
        return;
    }

    // Step 3. Let state be Record
    // { [[ErrorToRethrow]]: null, [[Destination]]: destination, [[PerformFetch]]: null, [[FetchClient]]: fetchClient }.
    let state = Rc::new(LoadState {
        error_to_rethrow: RefCell::new(None),
        destination,
        fetch_client,
    });

    // TODO Step 4. If performFetch was given, set state.[[PerformFetch]] to performFetch.

    let mut realm = enter_auto_realm(cx, global);
    let cx = &mut realm.current_realm();

    // Step 5. Let loadingPromise be record.LoadRequestedModules(state).
    let loading_promise = load_requested_modules(cx, module_script.clone(), Some(state.clone()));

    let global_scope = DomRoot::from_ref(global);
    let fulfilled_module = module_script.clone();
    let fulfilled_on_complete = on_complete.clone();

    // Step 6. Upon fulfillment of loadingPromise, run the following steps:
    let loading_promise_fulfillment = ModuleHandler::new_boxed(Box::new(
        task!(fulfilled_steps: |cx, global_scope: DomRoot<GlobalScope>| {
            let mut realm = AutoRealm::new(
                cx,
                NonNull::new(global_scope.reflector().get_jsobject().get()).unwrap(),
            );
            let cx = &mut *realm;

            let handle = fulfilled_module.get_record().map(|module| module.handle()).unwrap();

            // Step 6.1. Perform record.Link().
            let link = unsafe { ModuleLink(cx, handle) };

            // If this throws an exception, catch it, and set moduleScript's error to rethrow to that exception.
            if !link {
                let exception = RethrowError::from_pending_exception(cx);
                fulfilled_module.set_rethrow_error(exception);
            }

            // Step 6.2. Run onComplete given moduleScript.
            fulfilled_on_complete(cx, Some(fulfilled_module));
        }),
    ));

    // Step 7. Upon rejection of loadingPromise, run the following steps:
    let loading_promise_rejection =
        ModuleHandler::new_boxed(Box::new(task!(rejected_steps: |cx, state: Rc<LoadState>| {
            // Step 7.1. If state.[[ErrorToRethrow]] is not null, set moduleScript's error to rethrow to state.[[ErrorToRethrow]]
            // and run onComplete given moduleScript.
            if let Some(error) = state.error_to_rethrow.borrow().as_ref() {
                module_script.set_rethrow_error(error.clone());
                on_complete(cx, Some(module_script));
            } else {
                // Step 7.2. Otherwise, run onComplete given null.
                on_complete(cx, None);
            }
        })));

    let handler = PromiseNativeHandler::new(
        cx,
        global,
        Some(loading_promise_fulfillment),
        Some(loading_promise_rejection),
    );

    run_a_callback::<DomTypeHolder, _>(global, || {
        loading_promise.append_native_handler(cx, &handler);
    });
}

/// <https://html.spec.whatwg.org/multipage/#fetch-a-single-module-script>
#[expect(clippy::too_many_arguments)]
pub(crate) fn fetch_a_single_module_script(
    cx: &mut JSContext,
    url: UrlWithBlobClaim,
    fetch_client: RequestClient,
    global: &GlobalScope,
    destination: Destination,
    options: ScriptFetchOptions,
    referrer: Referrer,
    module_type: Option<ModuleType>,
    is_top_level: bool,
    introduction_type: Option<&'static CStr>,
    on_complete: impl FnOnce(&mut JSContext, Option<Rc<ModuleTree>>) + 'static,
) {
    // Step 1. Let moduleType be "javascript-or-wasm".
    // Step 2. If moduleRequest was given, then set moduleType to the result of running the
    // module type from module request steps given moduleRequest.
    let module_type = module_type.unwrap_or(ModuleType::JavaScript);

    // TODO Step 3. Assert: the result of running the module type allowed steps given moduleType and settingsObject is true.
    // Otherwise, we would not have reached this point because a failure would have been raised
    // when inspecting moduleRequest.[[Attributes]] in HostLoadImportedModule or fetch a single imported module script.

    let module_request = (url.url(), module_type);

    // Step 4. Let moduleMap be settingsObject's module map.
    let module_map = global.module_map();
    let mut module_map_borrow = module_map.safe_borrow_mut(cx);

    let entry = module_map_borrow.entry(module_request.clone());

    match entry {
        Entry::Occupied(mut entry) => match entry.get_mut() {
            // Step 5. If moduleMap[(url, moduleType)] is a module script, run onComplete given
            // moduleMap[(url, moduleType)], and return.
            ModuleStatus::Loaded(module_tree) => {
                let module = module_tree.clone();
                drop(module_map_borrow);
                return on_complete(cx, Some(module));
            },
            // Step 6. If moduleMap[(url, moduleType)] is a list, append onComplete to
            // moduleMap[(url, moduleType)], and return.
            ModuleStatus::Fetching(callbacks) => return callbacks.push(Box::new(on_complete)),
        },
        // Step 7. Set moduleMap[(url, moduleType)] to « onComplete ».
        Entry::Vacant(entry) => {
            entry.insert(ModuleStatus::Fetching(vec![Box::new(on_complete)]));
        },
    }

    // We only need a policy container when fetching the root of a module worker.
    let policy_container = (is_top_level && global.is::<WorkerGlobalScope>())
        .then(|| fetch_client.policy_container.clone());

    // Step 8. Let request be a new request whose URL is url, mode is "cors", referrer is referrer, and client is fetchClient.

    // Step 10. If destination is "worker", "sharedworker", or "serviceworker", and isTopLevel is true,
    // then set request's mode to "same-origin".
    let mode = match destination {
        Destination::Worker | Destination::SharedWorker if is_top_level => RequestMode::SameOrigin,
        _ => RequestMode::CorsMode,
    };

    // Step 9. Set request's destination to the result of running the
    // fetch destination from module type steps given destination and moduleType.
    let destination = match module_type {
        ModuleType::JSON => Destination::Json,
        ModuleType::JavaScript | ModuleType::Unknown => destination,
    };

    // TODO Step 11. Set request's initiator type to "script".

    // Step 12. Set up the module script request given request and options.
    let request = RequestBuilder::new(global.webview_id(), url, referrer)
        .destination(destination)
        .parser_metadata(options.parser_metadata)
        .integrity_metadata(options.integrity_metadata.clone())
        .credentials_mode(options.credentials_mode)
        .referrer_policy(options.referrer_policy)
        .mode(mode)
        .cryptographic_nonce_metadata(options.cryptographic_nonce.clone())
        .client(fetch_client)
        .pipeline_id(Some(global.pipeline_id()));

    let context = ModuleContext {
        owner: Trusted::new(global),
        data: vec![],
        metadata: None,
        module_request,
        options,
        status: Ok(()),
        introduction_type,
        policy_container,
    };

    let task_source = global.task_manager().networking_task_source().to_sendable();
    global.fetch(request, context, task_source);
}

/// <https://html.spec.whatwg.org/multipage/#specifier-resolution-record>
#[derive(Default, Eq, Hash, JSTraceable, MallocSizeOf, PartialEq)]
pub(crate) struct ResolvedModule {
    /// <https://html.spec.whatwg.org/multipage/#specifier-resolution-record-serialized-base-url>
    pub(crate) base_url: String,
    /// <https://html.spec.whatwg.org/multipage/#specifier-resolution-record-specifier>
    pub(crate) specifier: String,
    /// <https://html.spec.whatwg.org/multipage/#specifier-resolution-record-as-url>
    #[no_trace]
    pub(crate) specifier_url: Option<ServoUrl>,
}

impl ResolvedModule {
    pub(crate) fn new(
        base_url: String,
        specifier: String,
        specifier_url: Option<ServoUrl>,
    ) -> Self {
        Self {
            base_url,
            specifier,
            specifier_url,
        }
    }
}

/// <https://html.spec.whatwg.org/multipage/#resolving-an-imports-match>
///
/// When the error is thrown, it will terminate the entire resolve a module specifier algorithm
/// without any further fallbacks.
fn resolve_imports_match(
    normalized_specifier: &str,
    as_url: Option<&ServoUrl>,
    specifier_map: &ModuleSpecifierMap,
) -> Fallible<Option<ServoUrl>> {
    // Step 1. For each specifierKey → resolutionResult of specifierMap:
    for (specifier_key, resolution_result) in specifier_map {
        // Step 1.1 If specifierKey is normalizedSpecifier, then:
        if specifier_key == normalized_specifier {
            if let Some(resolution_result) = resolution_result {
                // Step 1.1.2 Assert: resolutionResult is a URL.
                // This is checked by Url type already.
                // Step 1.1.3 Return resolutionResult.
                return Ok(Some(resolution_result.clone()));
            } else {
                // Step 1.1.1 If resolutionResult is null, then throw a TypeError.
                return Err(Error::Type(
                    c"Resolution of specifierKey was blocked by a null entry.".to_owned(),
                ));
            }
        }

        // Step 1.2 If all of the following are true:
        // - specifierKey ends with U+002F (/)
        // - specifierKey is a code unit prefix of normalizedSpecifier
        // - either asURL is null, or asURL is special, then:
        if specifier_key.ends_with('\u{002f}') &&
            normalized_specifier.starts_with(specifier_key) &&
            (as_url.is_none() || as_url.is_some_and(|u| u.is_special_scheme()))
        {
            // Step 1.2.1 If resolutionResult is null, then throw a TypeError.
            // Step 1.2.2 Assert: resolutionResult is a URL.
            let Some(resolution_result) = resolution_result else {
                return Err(Error::Type(
                    c"Resolution of specifierKey was blocked by a null entry.".to_owned(),
                ));
            };

            // Step 1.2.3 Let afterPrefix be the portion of normalizedSpecifier after the initial specifierKey prefix.
            let after_prefix = normalized_specifier
                .strip_prefix(specifier_key)
                .expect("specifier_key should be the prefix of normalized_specifier");

            // Step 1.2.4 Assert: resolutionResult, serialized, ends with U+002F (/), as enforced during parsing.
            debug_assert!(resolution_result.as_str().ends_with('\u{002f}'));

            // Step 1.2.5 Let url be the result of URL parsing afterPrefix with resolutionResult.
            let url = ServoUrl::parse_with_base(Some(resolution_result), after_prefix);

            // Step 1.2.6 If url is failure, then throw a TypeError
            // Step 1.2.7 Assert: url is a URL.
            let Ok(url) = url else {
                return Err(Error::Type(
                    c"Resolution of normalizedSpecifier was blocked since
                    the afterPrefix portion could not be URL-parsed relative to
                    the resolutionResult mapped to by the specifierKey prefix."
                        .to_owned(),
                ));
            };

            // Step 1.2.8 If the serialization of resolutionResult is not
            // a code unit prefix of the serialization of url, then throw a TypeError
            if !url.as_str().starts_with(resolution_result.as_str()) {
                return Err(Error::Type(
                    c"Resolution of normalizedSpecifier was blocked due to
                    it backtracking above its prefix specifierKey."
                        .to_owned(),
                ));
            }

            // Step 1.2.9 Return url.
            return Ok(Some(url));
        }
    }

    // Step 2. Return null.
    Ok(None)
}

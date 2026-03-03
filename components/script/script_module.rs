/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! The script module mod contains common traits and structs
//! related to `type=module` for script thread or worker threads.

use std::cell::{OnceCell, RefCell};
use std::ffi::CStr;
use std::fmt::Debug;
use std::ptr::NonNull;
use std::rc::Rc;
use std::{mem, ptr};

use encoding_rs::UTF_8;
use headers::{HeaderMapExt, ReferrerPolicy as ReferrerPolicyHeader};
use html5ever::local_name;
use hyper_serde::Serde;
use indexmap::IndexMap;
use indexmap::map::Entry;
use js::context::JSContext;
use js::conversions::jsstr_to_string;
use js::gc::MutableHandleValue;
use js::jsapi::{
    CallArgs, CompileJsonModule1, CompileModule1, ExceptionStackBehavior,
    GetFunctionNativeReserved, GetModuleResolveHook, Handle as RawHandle,
    HandleValue as RawHandleValue, Heap, JS_ClearPendingException, JS_GetFunctionObject,
    JSAutoRealm, JSContext as RawJSContext, JSObject, JSPROP_ENUMERATE, JSRuntime,
    ModuleErrorBehaviour, ModuleType, SetFunctionNativeReserved, SetModuleDynamicImportHook,
    SetModuleMetadataHook, SetModulePrivate, SetModuleResolveHook, SetScriptPrivateReferenceHooks,
    ThrowOnModuleEvaluationFailure, Value,
};
use js::jsval::{JSVal, PrivateValue, UndefinedValue};
use js::realm::{AutoRealm, CurrentRealm};
use js::rust::wrappers::{JS_GetPendingException, JS_SetPendingException, ModuleEvaluate};
use js::rust::wrappers2::{
    DefineFunctionWithReserved, GetModuleRequestSpecifier, GetModuleRequestType,
    JS_DefineProperty4, JS_NewStringCopyN, ModuleLink,
};
use js::rust::{
    CompileOptionsWrapper, Handle, HandleObject as RustHandleObject, HandleValue, ToString,
    transform_str_to_source_text,
};
use mime::Mime;
use net_traits::http_status::HttpStatus;
use net_traits::mime_classifier::MimeClassifier;
use net_traits::request::{
    CredentialsMode, Destination, ParserMetadata, Referrer, RequestBuilder, RequestId, RequestMode,
};
use net_traits::{FetchMetadata, Metadata, NetworkError, ReferrerPolicy, ResourceFetchTiming};
use script_bindings::cformat;
use script_bindings::domstring::BytesView;
use script_bindings::error::Fallible;
use script_bindings::settings_stack::run_a_callback;
use script_bindings::trace::CustomTraceable;
use serde_json::{Map as JsonMap, Value as JsonValue};
use servo_url::ServoUrl;

use crate::DomTypeHolder;
use crate::document_loader::LoadType;
use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::WindowBinding::Window_Binding::WindowMethods;
use crate::dom::bindings::conversions::SafeToJSValConvertible;
use crate::dom::bindings::error::{
    Error, ErrorToJsval, report_pending_exception, throw_dom_exception,
};
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::refcounted::Trusted;
use crate::dom::bindings::reflector::{DomGlobal, DomObject};
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::DOMString;
use crate::dom::bindings::trace::RootedTraceableBox;
use crate::dom::csp::{GlobalCspReporting, Violation};
use crate::dom::document::Document;
use crate::dom::element::Element;
use crate::dom::globalscope::GlobalScope;
use crate::dom::html::htmlscriptelement::{HTMLScriptElement, SCRIPT_JS_MIMES, Script};
use crate::dom::htmlscriptelement::substitute_with_local_script;
use crate::dom::node::NodeTraits;
use crate::dom::performance::performanceresourcetiming::InitiatorType;
use crate::dom::promise::Promise;
use crate::dom::promisenativehandler::{Callback, PromiseNativeHandler};
use crate::dom::types::Console;
use crate::dom::window::Window;
use crate::dom::worker::TrustedWorkerAddress;
use crate::fetch::RequestWithGlobalScope;
use crate::module_loading::{
    LoadState, Payload, host_load_imported_module, load_requested_modules,
};
use crate::network_listener::{
    self, FetchResponseListener, NetworkListener, ResourceTimingListener,
};
use crate::realms::{InRealm, enter_realm};
use crate::script_runtime::{CanGc, IntroductionType, JSContext as SafeJSContext};
use crate::task::NonSendTaskBox;

pub(crate) fn gen_type_error(global: &GlobalScope, error: Error, can_gc: CanGc) -> RethrowError {
    rooted!(in(*GlobalScope::get_cx()) let mut thrown = UndefinedValue());
    error.to_jsval(GlobalScope::get_cx(), global, thrown.handle_mut(), can_gc);

    RethrowError(RootedTraceableBox::from_box(Heap::boxed(thrown.get())))
}

#[derive(JSTraceable)]
pub(crate) struct ModuleObject(RootedTraceableBox<Heap<*mut JSObject>>);

impl ModuleObject {
    pub(crate) fn new(obj: RustHandleObject) -> ModuleObject {
        ModuleObject(RootedTraceableBox::from_box(Heap::boxed(obj.get())))
    }

    pub(crate) fn handle(&'_ self) -> js::gc::HandleObject<'_> {
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
    pub(crate) fn from_pending_exception(cx: SafeJSContext) -> Self {
        rooted!(in(*cx) let mut exception = UndefinedValue());
        assert!(unsafe { JS_GetPendingException(*cx, exception.handle_mut()) });
        unsafe { JS_ClearPendingException(*cx) };

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
    owner: Option<ModuleOwner>,
}

impl ModuleScript {
    pub(crate) fn new(
        base_url: ServoUrl,
        options: ScriptFetchOptions,
        owner: Option<ModuleOwner>,
    ) -> Self {
        ModuleScript {
            base_url,
            options,
            owner,
        }
    }
}

pub(crate) type ModuleRequest = (ServoUrl, ModuleType);

#[derive(Clone, JSTraceable)]
pub(crate) enum ModuleStatus {
    Fetching(DomRefCell<Option<Rc<Promise>>>),
    Loaded(Option<Rc<ModuleTree>>),
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
            .and_then(|url| global.get_module_map_entry(&(url.clone(), module_type)))
            .and_then(|status| match status {
                ModuleStatus::Fetching(_) => None,
                ModuleStatus::Loaded(module_tree) => module_tree,
            })
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
            Entry::Occupied(entry) => {
                // i. Assert: record.[[Module]] and result.[[Value]] are the same Module Record.
                assert_eq!(*entry.get(), url);
            },
            // b. Else,
            Entry::Vacant(entry) => {
                // i. Append the LoadedModuleRequest Record { [[Specifier]]: moduleRequest.[[Specifier]],
                // [[Attributes]]: moduleRequest.[[Attributes]], [[Module]]: result.[[Value]] } to referrer.[[LoadedModules]].
                entry.insert(url);
            },
        }
    }
}

pub(crate) struct ModuleSource {
    pub source: Rc<DOMString>,
    pub unminified_dir: Option<String>,
    pub external: bool,
    pub url: ServoUrl,
}

impl crate::unminify::ScriptSource for ModuleSource {
    fn unminified_dir(&self) -> Option<String> {
        self.unminified_dir.clone()
    }

    fn extract_bytes(&self) -> BytesView<'_> {
        self.source.as_bytes()
    }

    fn rewrite_source(&mut self, source: Rc<DOMString>) {
        self.source = source;
    }

    fn url(&self) -> ServoUrl {
        self.url.clone()
    }

    fn is_external(&self) -> bool {
        self.external
    }
}

impl ModuleTree {
    #[expect(unsafe_code)]
    #[expect(clippy::too_many_arguments)]
    /// <https://html.spec.whatwg.org/multipage/#creating-a-javascript-module-script>
    /// Although the CanGc argument appears unused, it represents the GC operations that
    /// can occur as part of compiling a script.
    fn create_a_javascript_module_script(
        source: Rc<DOMString>,
        owner: ModuleOwner,
        url: &ServoUrl,
        options: ScriptFetchOptions,
        external: bool,
        line_number: u32,
        introduction_type: Option<&'static CStr>,
        _can_gc: CanGc,
    ) -> Self {
        let cx = GlobalScope::get_cx();
        let global = owner.global();
        let _ac = JSAutoRealm::new(*cx, *global.reflector().get_jsobject());

        // Step 2. Let script be a new module script that this algorithm will subsequently initialize.
        // Step 6. Set script's parse error and error to rethrow to null.
        let module = ModuleTree {
            url: url.clone(),
            record: OnceCell::new(),
            parse_error: OnceCell::new(),
            rethrow_error: DomRefCell::new(None),
            loaded_modules: DomRefCell::new(IndexMap::new()),
        };

        let c_url = cformat!("{url}");
        let mut compile_options =
            unsafe { CompileOptionsWrapper::new_raw(*cx, c_url, line_number) };
        if let Some(introduction_type) = introduction_type {
            compile_options.set_introduction_type(introduction_type);
        }
        let mut module_source = ModuleSource {
            source,
            unminified_dir: global.unminified_js_dir(),
            external,
            url: url.clone(),
        };
        crate::unminify::unminify_js(&mut module_source);

        unsafe {
            // Step 7. Let result be ParseModule(source, settings's realm, script).
            rooted!(in(*cx) let mut module_script: *mut JSObject = std::ptr::null_mut());
            module_script.set(CompileModule1(
                *cx,
                compile_options.ptr,
                &mut transform_str_to_source_text(&module_source.source.str()),
            ));

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
    /// Although the CanGc argument appears unused, it represents the GC operations that
    /// can occur as part of compiling a script.
    fn crate_a_json_module_script(
        source: &str,
        global: &GlobalScope,
        url: &ServoUrl,
        introduction_type: Option<&'static CStr>,
        _can_gc: CanGc,
    ) -> Self {
        let cx = GlobalScope::get_cx();
        let _ac = JSAutoRealm::new(*cx, *global.reflector().get_jsobject());

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

        let c_url = cformat!("{url}");
        let mut compile_options = unsafe { CompileOptionsWrapper::new_raw(*cx, c_url, 1) };
        if let Some(introduction_type) = introduction_type {
            compile_options.set_introduction_type(introduction_type);
        }

        rooted!(in(*cx) let mut module_script: *mut JSObject = std::ptr::null_mut());

        unsafe {
            // Step 5. Let result be ParseJSONModule(source).
            module_script.set(CompileJsonModule1(
                *cx,
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
    /// mutable handle. Although the CanGc appears unused, it represents the GC operations
    /// possible when evluating arbitrary JS.
    #[expect(unsafe_code)]
    pub(crate) fn execute_module(
        &self,
        global: &GlobalScope,
        module_record: js::gc::HandleObject,
        mut eval_result: MutableHandleValue,
        _can_gc: CanGc,
    ) -> Result<(), RethrowError> {
        let cx = GlobalScope::get_cx();
        let _ac = JSAutoRealm::new(*cx, *global.reflector().get_jsobject());

        unsafe {
            let ok = ModuleEvaluate(*cx, module_record, eval_result.reborrow());
            assert!(ok, "module evaluation failed");

            rooted!(in(*cx) let mut evaluation_promise = ptr::null_mut::<JSObject>());
            if eval_result.is_object() {
                evaluation_promise.set(eval_result.to_object());
            }

            let throw_result = ThrowOnModuleEvaluationFailure(
                *cx,
                evaluation_promise.handle().into(),
                ModuleErrorBehaviour::ThrowModuleErrorsSync,
            );
            if !throw_result {
                warn!("fail to evaluate module");

                rooted!(in(*cx) let mut exception = UndefinedValue());
                assert!(JS_GetPendingException(*cx, exception.handle_mut()));
                JS_ClearPendingException(*cx);

                Err(RethrowError(RootedTraceableBox::from_box(Heap::boxed(
                    exception.get(),
                ))))
            } else {
                debug!("module evaluated successfully");
                Ok(())
            }
        }
    }

    #[expect(unsafe_code)]
    pub(crate) fn report_error(&self, global: &GlobalScope, can_gc: CanGc) {
        let module_error = self.rethrow_error.borrow();

        if let Some(exception) = &*module_error {
            let ar = enter_realm(global);
            unsafe {
                JS_SetPendingException(
                    *GlobalScope::get_cx(),
                    exception.handle(),
                    ExceptionStackBehavior::Capture,
                );
            }
            report_pending_exception(GlobalScope::get_cx(), true, InRealm::Entered(&ar), can_gc);
        }
    }

    /// <https://html.spec.whatwg.org/multipage/#resolve-a-module-specifier>
    pub(crate) fn resolve_module_specifier(
        global: &GlobalScope,
        script: Option<&ModuleScript>,
        specifier: DOMString,
    ) -> Fallible<ServoUrl> {
        // Step 1~3 to get settingsObject and baseURL
        let script_global = script.and_then(|s| s.owner.as_ref().map(|o| o.global()));
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
        let as_url = Self::resolve_url_like_module_specifier(specifier, base_url);
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

    /// <https://html.spec.whatwg.org/multipage/#resolving-a-url-like-module-specifier>
    fn resolve_url_like_module_specifier(specifier: &str, base_url: &ServoUrl) -> Option<ServoUrl> {
        // Step 1. If specifier starts with "/", "./", or "../", then:
        if specifier.starts_with('/') || specifier.starts_with("./") || specifier.starts_with("../")
        {
            // Step 1.1. Let url be the result of URL parsing specifier with baseURL.
            return ServoUrl::parse_with_base(Some(base_url), specifier).ok();
        }
        // Step 2. Let url be the result of URL parsing specifier (with no base URL).
        ServoUrl::parse(specifier).ok()
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

/// The owner of the module
/// It can be `worker` or `script` element
#[derive(Clone, JSTraceable)]
pub(crate) enum ModuleOwner {
    #[expect(dead_code)]
    Worker(TrustedWorkerAddress),
    Window(Trusted<HTMLScriptElement>),
    DynamicModule(Trusted<GlobalScope>),
}

impl ModuleOwner {
    pub(crate) fn global(&self) -> DomRoot<GlobalScope> {
        match &self {
            ModuleOwner::Worker(worker) => (*worker.root().clone()).global(),
            ModuleOwner::Window(script) => (*script.root()).global(),
            ModuleOwner::DynamicModule(dynamic_module) => (*dynamic_module.root()).global(),
        }
    }

    fn notify_owner_to_finish(&self, module_tree: Option<Rc<ModuleTree>>, can_gc: CanGc) {
        match &self {
            ModuleOwner::Worker(_) => unimplemented!(),
            ModuleOwner::DynamicModule(_) => unimplemented!(),
            ModuleOwner::Window(script) => {
                let document = script.root().owner_document();

                let load = match module_tree {
                    Some(module_tree) => Ok(Script::Module(module_tree)),
                    None => Err(()),
                };

                let asynch = script
                    .root()
                    .upcast::<Element>()
                    .has_attribute(&local_name!("async"));

                if !asynch && (*script.root()).get_parser_inserted() {
                    document.deferred_script_loaded(&script.root(), load, can_gc);
                } else if !asynch && !(*script.root()).get_non_blocking() {
                    document.asap_in_order_script_loaded(&script.root(), load, can_gc);
                } else {
                    document.asap_script_loaded(&script.root(), load, can_gc);
                };
            },
        }
    }
}

/// The context required for asynchronously loading an external module script source.
struct ModuleContext {
    /// The owner of the module that initiated the request.
    owner: ModuleOwner,
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

    fn process_response_chunk(&mut self, _: RequestId, mut chunk: Vec<u8>) {
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
        let global = self.owner.global();
        let (url, module_type) = &self.module_request;

        if let Some(window) = global.downcast::<Window>() {
            window
                .Document()
                .finish_load(LoadType::Script(url.clone()), cx);
        }

        network_listener::submit_timing(&self, &response, &timing, CanGc::from_cx(cx));

        let Some(ModuleStatus::Fetching(pending)) =
            global.get_module_map_entry(&self.module_request)
        else {
            return error!("Processing response for a non pending module request");
        };
        let promise = pending
            .borrow_mut()
            .take()
            .expect("Need promise to process response");

        // Step 1. If any of the following are true: bodyBytes is null or failure; or response's status is not an ok status,
        // then set moduleMap[(url, moduleType)] to null, run onComplete given null, and abort these steps.
        if let (Err(error), _) | (_, Err(error)) = (response.as_ref(), self.status.as_ref()) {
            error!("Fetching module script failed {:?}", error);
            global.set_module_map(self.module_request, ModuleStatus::Loaded(None));
            return promise.resolve_native(&(), CanGc::from_cx(cx));
        }

        let metadata = self.metadata.take().unwrap();
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
                if let Some(window) = global.downcast::<Window>() {
                    substitute_with_local_script(window, &mut source_text, final_url.clone());
                }

                let module_tree = Rc::new(ModuleTree::create_a_javascript_module_script(
                    Rc::new(DOMString::from(source_text.clone())),
                    self.owner.clone(),
                    &final_url,
                    self.options,
                    true,
                    1,
                    self.introduction_type,
                    CanGc::from_cx(cx),
                ));
                module_script = Some(module_tree);
            }

            // Step 7.4 If mimeType is a JSON MIME type and moduleType is "json",
            // then set moduleScript to the result of creating a JSON module script given sourceText and settingsObject.
            if MimeClassifier::is_json(&mime) && matches!(module_type, ModuleType::JSON) {
                let module_tree = Rc::new(ModuleTree::crate_a_json_module_script(
                    &source_text,
                    &global,
                    &final_url,
                    self.introduction_type,
                    CanGc::from_cx(cx),
                ));
                module_script = Some(module_tree);
            }
        }
        // Step 8. Set moduleMap[(url, moduleType)] to moduleScript, and run onComplete given moduleScript.
        global.set_module_map(self.module_request, ModuleStatus::Loaded(module_script));
        promise.resolve_native(&(), CanGc::from_cx(cx));
    }

    fn process_csp_violations(&mut self, _request_id: RequestId, violations: Vec<Violation>) {
        let global = &self.resource_timing_global();
        global.report_csp_violations(violations, None, None);
    }
}

impl ResourceTimingListener for ModuleContext {
    fn resource_timing_information(&self) -> (InitiatorType, ServoUrl) {
        let initiator_type = InitiatorType::LocalName("module".to_string());
        let (url, _) = &self.module_request;
        (initiator_type, url.clone())
    }

    fn resource_timing_global(&self) -> DomRoot<GlobalScope> {
        self.owner.global()
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
    // Safety: it is safe to construct a JSContext from engine hook.
    let mut cx = unsafe { JSContext::from_ptr(NonNull::new(cx).unwrap()) };
    let cx = &mut cx;
    let promise = Promise::new_with_js_promise(unsafe { Handle::from_raw(promise) }, cx.into());

    let jsstr = unsafe { GetModuleRequestSpecifier(cx, Handle::from_raw(specifier)) };
    let module_type = unsafe { GetModuleRequestType(cx, Handle::from_raw(specifier)) };
    let specifier = unsafe { jsstr_to_string(cx.raw_cx(), NonNull::new(jsstr).unwrap()) };

    let mut realm = CurrentRealm::assert(cx);
    let payload = Payload::PromiseRecord(promise.clone());
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
    #[no_trace]
    pub(crate) referrer: Referrer,
    pub(crate) integrity_metadata: String,
    #[no_trace]
    pub(crate) credentials_mode: CredentialsMode,
    pub(crate) cryptographic_nonce: String,
    #[no_trace]
    pub(crate) parser_metadata: ParserMetadata,
    #[no_trace]
    pub(crate) referrer_policy: ReferrerPolicy,
}

impl ScriptFetchOptions {
    /// <https://html.spec.whatwg.org/multipage/#default-classic-script-fetch-options>
    pub(crate) fn default_classic_script(global: &GlobalScope) -> ScriptFetchOptions {
        Self {
            cryptographic_nonce: String::new(),
            integrity_metadata: String::new(),
            referrer: global.get_referrer(),
            parser_metadata: ParserMetadata::NotParserInserted,
            credentials_mode: CredentialsMode::CredentialsSameOrigin,
            referrer_policy: ReferrerPolicy::EmptyString,
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
            referrer: self.referrer.clone(),
            // Step 3. Set newOptions's integrity metadata to integrity.
            integrity_metadata: integrity,
            cryptographic_nonce: self.cryptographic_nonce.clone(),
            credentials_mode: self.credentials_mode,
            parser_metadata: self.parser_metadata,
            referrer_policy: self.referrer_policy,
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
    // Safety: it is safe to construct a JSContext from engine hook.
    let mut cx = unsafe { JSContext::from_ptr(NonNull::new(cx).unwrap()) };
    let mut realm = CurrentRealm::assert(&mut cx);
    let global_scope = GlobalScope::from_current_realm(&realm);

    let cx = &mut realm;

    // Step 5.
    let module_data = unsafe { module_script_from_reference_private(&reference_private) };
    let jsstr = unsafe { GetModuleRequestSpecifier(cx, Handle::from_raw(specifier)) };
    let module_type = unsafe { GetModuleRequestType(cx, Handle::from_raw(specifier)) };

    let specifier = unsafe { jsstr_to_string(cx.raw_cx(), NonNull::new(jsstr).unwrap()) };
    let url = ModuleTree::resolve_module_specifier(
        &global_scope,
        module_data,
        DOMString::from(specifier),
    );

    // Step 6.
    assert!(url.is_ok());

    let parsed_url = url.unwrap();

    // Step 4 & 7.
    let module = global_scope.get_module_map_entry(&(parsed_url, module_type));

    // Step 9.
    assert!(module.as_ref().is_some_and(
        |status| matches!(status, ModuleStatus::Loaded(module_tree) if module_tree.is_some())
    ));

    let ModuleStatus::Loaded(Some(module_tree)) = module.unwrap() else {
        unreachable!()
    };

    let fetched_module_object = module_tree.get_record();

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
    // Safety: it is safe to construct a JSContext from engine hook.
    let mut cx = unsafe { JSContext::from_ptr(NonNull::new(cx).unwrap()) };
    let realm = CurrentRealm::assert(&mut cx);
    let global_scope = GlobalScope::from_current_realm(&realm);

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
    // Safety: it is safe to construct a JSContext from engine hook.
    let mut cx = unsafe { JSContext::from_ptr(ptr::NonNull::new(cx).unwrap()) };
    let mut realm = CurrentRealm::assert(&mut cx);
    let global_scope = GlobalScope::from_current_realm(&realm);

    let cx = &mut realm;

    let args = unsafe { CallArgs::from_vp(vp, argc) };

    rooted!(&in(cx) let module_private = unsafe { *GetFunctionNativeReserved(args.callee(), SLOT_MODULEPRIVATE) });
    let reference_private = module_private.handle().into();
    let module_data = unsafe { module_script_from_reference_private(&reference_private) };

    // https://html.spec.whatwg.org/multipage/#hostgetimportmetaproperties

    // Step 4.1. Set specifier to ? ToString(specifier).
    let specifier = unsafe {
        let value = HandleValue::from_raw(args.get(0));

        match NonNull::new(ToString(cx.raw_cx(), value)) {
            Some(jsstr) => DOMString::from_string(jsstr_to_string(cx.raw_cx(), jsstr)),
            None => return false,
        }
    };

    // Step 4.2. Let url be the result of resolving a module specifier given moduleScript and specifier.
    let url = ModuleTree::resolve_module_specifier(&global_scope, module_data, specifier);

    match url {
        Ok(url) => {
            // Step 4.3. Return the serialization of url.
            url.as_str().safe_to_jsval(
                cx.into(),
                unsafe { MutableHandleValue::from_raw(args.rval()) },
                CanGc::from_cx(cx),
            );
            true
        },
        Err(error) => {
            let resolution_error = gen_type_error(&global_scope, error, CanGc::from_cx(cx));

            unsafe {
                JS_SetPendingException(
                    cx.raw_cx(),
                    resolution_error.handle(),
                    ExceptionStackBehavior::Capture,
                );
            }
            false
        },
    }
}

/// <https://html.spec.whatwg.org/multipage/#fetch-a-module-script-tree>
pub(crate) fn fetch_an_external_module_script(
    url: ServoUrl,
    owner: ModuleOwner,
    options: ScriptFetchOptions,
    can_gc: CanGc,
) {
    let referrer = owner.global().get_referrer();

    // Step 1. Fetch a single module script given url, settingsObject, "script", options, settingsObject, "client", true,
    // and with the following steps given result:
    fetch_a_single_module_script(
        url,
        owner.clone(),
        Destination::Script,
        options,
        referrer,
        None,
        true,
        Some(IntroductionType::SRC_SCRIPT),
        move |module_tree| {
            let Some(module) = module_tree else {
                // Step 1.1. If result is null, run onComplete given null, and abort these steps.
                return owner.notify_owner_to_finish(None, can_gc);
            };

            // Step 1.2. Fetch the descendants of and link result given settingsObject, "script", and onComplete.
            fetch_the_descendants_and_link_module_script(module, Destination::Script, owner);
        },
    );
}

/// <https://html.spec.whatwg.org/multipage/#fetch-an-inline-module-script-graph>
pub(crate) fn fetch_inline_module_script(
    owner: ModuleOwner,
    module_script_text: Rc<DOMString>,
    url: ServoUrl,
    options: ScriptFetchOptions,
    line_number: u32,
    can_gc: CanGc,
) {
    // Step 1. Let script be the result of creating a JavaScript module script using sourceText, settingsObject, baseURL, and options.
    let module_tree = Rc::new(ModuleTree::create_a_javascript_module_script(
        module_script_text,
        owner.clone(),
        &url,
        options,
        false,
        line_number,
        Some(IntroductionType::INLINE_SCRIPT),
        can_gc,
    ));

    // Step 2. Fetch the descendants of and link script, given settingsObject, "script", and onComplete.
    fetch_the_descendants_and_link_module_script(module_tree, Destination::Script, owner);
}

#[expect(unsafe_code)]
/// <https://html.spec.whatwg.org/multipage/#fetch-the-descendants-of-and-link-a-module-script>
fn fetch_the_descendants_and_link_module_script(
    module_script: Rc<ModuleTree>,
    destination: Destination,
    owner: ModuleOwner,
) {
    let mut cx = unsafe { script_bindings::script_runtime::temp_cx() };
    let mut realm = CurrentRealm::assert(&mut cx);
    let cx = &mut realm;

    let global = owner.global();

    // Step 1. Let record be moduleScript's record.
    // Step 2. If record is null, then:
    if module_script.get_record().is_none() {
        let parse_error = module_script.get_parse_error().cloned();

        // Step 2.1. Set moduleScript's error to rethrow to moduleScript's parse error.
        module_script.set_rethrow_error(parse_error.unwrap());

        // Step 2.2. Run onComplete given moduleScript.
        owner.notify_owner_to_finish(Some(module_script), CanGc::from_cx(cx));

        // Step 2.3. Return.
        return;
    }

    // Step 3. Let state be Record
    // { [[ErrorToRethrow]]: null, [[Destination]]: destination, [[PerformFetch]]: null, [[FetchClient]]: fetchClient }.
    let state = Rc::new(LoadState {
        error_to_rethrow: RefCell::new(None),
        destination,
        fetch_client: owner.clone(),
    });

    // TODO Step 4. If performFetch was given, set state.[[PerformFetch]] to performFetch.

    // Step 5. Let loadingPromise be record.LoadRequestedModules(state).
    let loading_promise =
        load_requested_modules(cx, module_script.clone(), Some(Rc::clone(&state)));

    let fulfillment_owner = owner.clone();
    let fulfilled_module = module_script.clone();

    // Step 6. Upon fulfillment of loadingPromise, run the following steps:
    let loading_promise_fulfillment = ModuleHandler::new_boxed(Box::new(
        task!(fulfilled_steps: |cx, fulfillment_owner: ModuleOwner| {
            let global = fulfillment_owner.global();
            let mut realm = AutoRealm::new(
                cx,
                NonNull::new(global.reflector().get_jsobject().get()).unwrap(),
            );
            let cx = &mut *realm;

            let handle = fulfilled_module.get_record().map(|module| module.handle()).unwrap();

            // Step 6.1. Perform record.Link().
            let link = unsafe { ModuleLink(cx, handle) };

            // If this throws an exception, catch it, and set moduleScript's error to rethrow to that exception.
            if !link {
                let exception = RethrowError::from_pending_exception(cx.into());
                fulfilled_module.set_rethrow_error(exception);
            }

            // Step 6.2. Run onComplete given moduleScript.
            fulfillment_owner.notify_owner_to_finish(Some(fulfilled_module), CanGc::from_cx(cx));
        }),
    ));

    let rejection_owner = owner.clone();
    let rejected_module = module_script.clone();

    // Step 7. Upon rejection of loadingPromise, run the following steps:
    let loading_promise_rejection = ModuleHandler::new_boxed(Box::new(
        task!(rejected_steps: |rejection_owner: ModuleOwner, state: Rc<LoadState>| {
            // Step 7.1. If state.[[ErrorToRethrow]] is not null, set moduleScript's error to rethrow to state.[[ErrorToRethrow]]
            // and run onComplete given moduleScript.
            if let Some(error) = state.error_to_rethrow.borrow().as_ref() {
                rejected_module.set_rethrow_error(error.clone());
                rejection_owner.notify_owner_to_finish(Some(rejected_module), CanGc::note());
            } else {
                // Step 7.2. Otherwise, run onComplete given null.
                rejection_owner.notify_owner_to_finish(None, CanGc::note());
            }
        }),
    ));

    let handler = PromiseNativeHandler::new(
        &global,
        Some(loading_promise_fulfillment),
        Some(loading_promise_rejection),
        CanGc::from_cx(cx),
    );

    let realm = enter_realm(&*global);
    let comp = InRealm::Entered(&realm);
    run_a_callback::<DomTypeHolder, _>(&global, || {
        loading_promise.append_native_handler(&handler, comp, CanGc::from_cx(cx));
    });
}

/// <https://html.spec.whatwg.org/multipage/#fetch-a-single-module-script>
#[expect(clippy::too_many_arguments)]
pub(crate) fn fetch_a_single_module_script(
    url: ServoUrl,
    owner: ModuleOwner,
    destination: Destination,
    options: ScriptFetchOptions,
    referrer: Referrer,
    module_type: Option<ModuleType>,
    is_top_level: bool,
    introduction_type: Option<&'static CStr>,
    on_complete: impl FnOnce(Option<Rc<ModuleTree>>) + 'static,
) {
    let global = owner.global();

    // Step 1. Let moduleType be "javascript-or-wasm".
    // Step 2. If moduleRequest was given, then set moduleType to the result of running the
    // module type from module request steps given moduleRequest.
    let module_type = module_type.unwrap_or(ModuleType::JavaScript);

    // TODO Step 3. Assert: the result of running the module type allowed steps given moduleType and settingsObject is true.
    // Otherwise, we would not have reached this point because a failure would have been raised
    // when inspecting moduleRequest.[[Attributes]] in HostLoadImportedModule or fetch a single imported module script.

    // Step 4. Let moduleMap be settingsObject's module map.
    let module_request = (url.clone(), module_type);
    let entry = global.get_module_map_entry(&module_request);

    let pending = match entry {
        Some(ModuleStatus::Fetching(pending)) => pending,
        // Step 6. If moduleMap[(url, moduleType)] exists, run onComplete given moduleMap[(url, moduleType)], and return.
        Some(ModuleStatus::Loaded(module_tree)) => {
            return on_complete(module_tree);
        },
        None => DomRefCell::new(None),
    };

    let global_scope = DomRoot::from_ref(&*global);
    let module_map_key = module_request.clone();
    let handler = ModuleHandler::new_boxed(Box::new(
        task!(fetch_completed: |global_scope: DomRoot<GlobalScope>| {
            let key = module_map_key;
            let module = global_scope.get_module_map_entry(&key);

            if let Some(ModuleStatus::Loaded(module_tree)) = module {
                on_complete(module_tree);
            }
        }),
    ));

    let handler = PromiseNativeHandler::new(&global, Some(handler), None, CanGc::note());

    let realm = enter_realm(&*global);
    let comp = InRealm::Entered(&realm);
    run_a_callback::<DomTypeHolder, _>(&global, || {
        let has_pending_fetch = pending.borrow().is_some();
        pending
            .borrow_mut()
            .get_or_insert_with(|| Promise::new_in_current_realm(comp, CanGc::note()))
            .append_native_handler(&handler, comp, CanGc::note());

        // Step 5. If moduleMap[(url, moduleType)] is "fetching", wait in parallel until that entry's value changes,
        // then queue a task on the networking task source to proceed with running the following steps.
        if has_pending_fetch {
            return;
        }

        // Step 7. Set moduleMap[(url, moduleType)] to "fetching".
        global.set_module_map(module_request.clone(), ModuleStatus::Fetching(pending));

        let document: Option<DomRoot<Document>> = match &owner {
            ModuleOwner::Worker(_) | ModuleOwner::DynamicModule(_) => None,
            ModuleOwner::Window(script) => Some(script.root().owner_document()),
        };
        let webview_id = document.as_ref().map(|document| document.webview_id());

        // Step 8. Let request be a new request whose URL is url, mode is "cors", referrer is referrer, and client is fetchClient.

        // Step 10. If destination is "worker", "sharedworker", or "serviceworker", and isTopLevel is true,
        // then set request's mode to "same-origin".
        let mode = match destination {
            Destination::Worker | Destination::SharedWorker if is_top_level => {
                RequestMode::SameOrigin
            },
            _ => RequestMode::CorsMode,
        };

        // Step 9. Set request's destination to the result of running the fetch destination from module type steps given destination and moduleType.
        let destination = match module_type {
            ModuleType::JSON => Destination::Json,
            ModuleType::JavaScript | ModuleType::Unknown => destination,
        };

        // TODO Step 11. Set request's initiator type to "script".

        // Step 12. Set up the module script request given request and options.
        let request = RequestBuilder::new(webview_id, url.clone(), referrer)
            .destination(destination)
            .parser_metadata(options.parser_metadata)
            .integrity_metadata(options.integrity_metadata.clone())
            .credentials_mode(options.credentials_mode)
            .referrer_policy(options.referrer_policy)
            .mode(mode)
            .with_global_scope(&global)
            .cryptographic_nonce_metadata(options.cryptographic_nonce.clone());

        let context = ModuleContext {
            owner,
            data: vec![],
            metadata: None,
            module_request,
            options,
            status: Ok(()),
            introduction_type,
        };

        let network_listener = NetworkListener::new(
            context,
            global.task_manager().networking_task_source().to_sendable(),
        );
        match document {
            Some(document) => {
                document.loader_mut().fetch_async_with_callback(
                    LoadType::Script(url),
                    request,
                    network_listener.into_callback(),
                );
            },
            None => global.fetch_with_network_listener(request, network_listener),
        };
    })
}

pub(crate) type ModuleSpecifierMap = IndexMap<String, Option<ServoUrl>>;
pub(crate) type ModuleIntegrityMap = IndexMap<ServoUrl, String>;

/// <https://html.spec.whatwg.org/multipage/#specifier-resolution-record>
#[derive(Default, Eq, Hash, JSTraceable, MallocSizeOf, PartialEq)]
pub(crate) struct ResolvedModule {
    /// <https://html.spec.whatwg.org/multipage/#specifier-resolution-record-serialized-base-url>
    base_url: String,
    /// <https://html.spec.whatwg.org/multipage/#specifier-resolution-record-specifier>
    specifier: String,
    /// <https://html.spec.whatwg.org/multipage/#specifier-resolution-record-as-url>
    #[no_trace]
    specifier_url: Option<ServoUrl>,
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

/// <https://html.spec.whatwg.org/multipage/#import-map-processing-model>
#[derive(Default, JSTraceable, MallocSizeOf)]
pub(crate) struct ImportMap {
    #[no_trace]
    imports: ModuleSpecifierMap,
    #[no_trace]
    scopes: IndexMap<ServoUrl, ModuleSpecifierMap>,
    #[no_trace]
    integrity: ModuleIntegrityMap,
}

impl ImportMap {
    /// <https://html.spec.whatwg.org/multipage/#resolving-a-module-integrity-metadata>
    pub(crate) fn resolve_a_module_integrity_metadata(&self, url: &ServoUrl) -> String {
        // Step 1. Let map be settingsObject's global object's import map.

        // Step 2. If map's integrity[url] does not exist, then return the empty string.
        // Step 3. Return map's integrity[url].
        self.integrity.get(url).cloned().unwrap_or_default()
    }
}

/// <https://html.spec.whatwg.org/multipage/#register-an-import-map>
pub(crate) fn register_import_map(
    global: &GlobalScope,
    result: Fallible<ImportMap>,
    can_gc: CanGc,
) {
    match result {
        Ok(new_import_map) => {
            // Step 2. Merge existing and new import maps, given global and result's import map.
            merge_existing_and_new_import_maps(global, new_import_map, can_gc);
        },
        Err(exception) => {
            // Step 1. If result's error to rethrow is not null, then report
            // an exception given by result's error to rethrow for global and return.
            throw_dom_exception(GlobalScope::get_cx(), global, exception.clone(), can_gc);
        },
    }
}

/// <https://html.spec.whatwg.org/multipage/#merge-existing-and-new-import-maps>
fn merge_existing_and_new_import_maps(
    global: &GlobalScope,
    new_import_map: ImportMap,
    can_gc: CanGc,
) {
    // Step 1. Let newImportMapScopes be a deep copy of newImportMap's scopes.
    let new_import_map_scopes = new_import_map.scopes;

    // Step 2. Let oldImportMap be global's import map.
    let mut old_import_map = global.import_map_mut();

    // Step 3. Let newImportMapImports be a deep copy of newImportMap's imports.
    let mut new_import_map_imports = new_import_map.imports;

    let resolved_module_set = global.resolved_module_set();
    // Step 4. For each scopePrefix → scopeImports of newImportMapScopes:
    for (scope_prefix, mut scope_imports) in new_import_map_scopes {
        // Step 4.1. For each record of global's resolved module set:
        for record in resolved_module_set.iter() {
            // If scopePrefix is record's serialized base URL, or if scopePrefix ends with
            // U+002F (/) and scopePrefix is a code unit prefix of record's serialized base URL, then:
            let prefix = scope_prefix.as_str();
            if prefix == record.base_url ||
                (record.base_url.starts_with(prefix) && prefix.ends_with('\u{002f}'))
            {
                // For each specifierKey → resolutionResult of scopeImports:
                scope_imports.retain(|key, val| {
                    // If specifierKey is record's specifier, or if all of the following conditions are true:
                    // specifierKey ends with U+002F (/);
                    // specifierKey is a code unit prefix of record's specifier;
                    // either record's specifier as a URL is null or is special,
                    if *key == record.specifier ||
                        (key.ends_with('\u{002f}') &&
                            record.specifier.starts_with(key) &&
                            (record.specifier_url.is_none() ||
                                record
                                    .specifier_url
                                    .as_ref()
                                    .is_some_and(|u| u.is_special_scheme())))
                    {
                        // The user agent may report a warning to the console indicating the ignored rule.
                        // They may choose to avoid reporting if the rule is identical to an existing one.
                        Console::internal_warn(
                            global,
                            DOMString::from(format!("Ignored rule: {key} -> {val:?}.")),
                        );
                        // Remove scopeImports[specifierKey].
                        false
                    } else {
                        true
                    }
                })
            }
        }

        // Step 4.2 If scopePrefix exists in oldImportMap's scopes
        if old_import_map.scopes.contains_key(&scope_prefix) {
            // set oldImportMap's scopes[scopePrefix] to the result of
            // merging module specifier maps, given scopeImports and oldImportMap's scopes[scopePrefix].
            let merged_module_specifier_map = merge_module_specifier_maps(
                global,
                scope_imports,
                &old_import_map.scopes[&scope_prefix],
                can_gc,
            );
            old_import_map
                .scopes
                .insert(scope_prefix, merged_module_specifier_map);
        } else {
            // Step 4.3 Otherwise, set oldImportMap's scopes[scopePrefix] to scopeImports.
            old_import_map.scopes.insert(scope_prefix, scope_imports);
        }
    }

    // Step 5. For each url → integrity of newImportMap's integrity:
    for (url, integrity) in &new_import_map.integrity {
        // Step 5.1 If url exists in oldImportMap's integrity, then:
        if old_import_map.integrity.contains_key(url) {
            // Step 5.1.1 The user agent may report a warning to the console indicating the ignored rule.
            // They may choose to avoid reporting if the rule is identical to an existing one.
            Console::internal_warn(
                global,
                DOMString::from(format!("Ignored rule: {url} -> {integrity}.")),
            );
            // Step 5.1.2 Continue.
            continue;
        }

        // Step 5.2 Set oldImportMap's integrity[url] to integrity.
        old_import_map
            .integrity
            .insert(url.clone(), integrity.clone());
    }

    // Step 6. For each record of global's resolved module set:
    for record in resolved_module_set.iter() {
        // For each specifier → url of newImportMapImports:
        new_import_map_imports.retain(|specifier, val| {
            // If specifier starts with record's specifier, then:
            //
            // Note: Spec is wrong, we need to check if record's specifier starts with specifier
            // See: https://github.com/whatwg/html/issues/11875
            if record.specifier.starts_with(specifier) {
                // The user agent may report a warning to the console indicating the ignored rule.
                // They may choose to avoid reporting if the rule is identical to an existing one.
                Console::internal_warn(
                    global,
                    DOMString::from(format!("Ignored rule: {specifier} -> {val:?}.")),
                );
                // Remove newImportMapImports[specifier].
                false
            } else {
                true
            }
        });
    }

    // Step 7. Set oldImportMap's imports to the result of merge module specifier maps,
    // given newImportMapImports and oldImportMap's imports.
    let merged_module_specifier_map = merge_module_specifier_maps(
        global,
        new_import_map_imports,
        &old_import_map.imports,
        can_gc,
    );
    old_import_map.imports = merged_module_specifier_map;

    // https://html.spec.whatwg.org/multipage/#the-resolution-algorithm
    // Sort scopes to ensure entries are visited from most-specific to least-specific.
    old_import_map
        .scopes
        .sort_by(|a_key, _, b_key, _| b_key.cmp(a_key));
}

/// <https://html.spec.whatwg.org/multipage/#merge-module-specifier-maps>
fn merge_module_specifier_maps(
    global: &GlobalScope,
    new_map: ModuleSpecifierMap,
    old_map: &ModuleSpecifierMap,
    _can_gc: CanGc,
) -> ModuleSpecifierMap {
    // Step 1. Let mergedMap be a deep copy of oldMap.
    let mut merged_map = old_map.clone();

    // Step 2. For each specifier → url of newMap:
    for (specifier, url) in new_map {
        // Step 2.1 If specifier exists in oldMap, then:
        if old_map.contains_key(&specifier) {
            // Step 2.1.1 The user agent may report a warning to the console indicating the ignored rule.
            // They may choose to avoid reporting if the rule is identical to an existing one.
            Console::internal_warn(
                global,
                DOMString::from(format!("Ignored rule: {specifier} -> {url:?}.")),
            );

            // Step 2.1.2 Continue.
            continue;
        }

        // Step 2.2 Set mergedMap[specifier] to url.
        merged_map.insert(specifier, url);
    }

    merged_map
}

/// <https://html.spec.whatwg.org/multipage/#parse-an-import-map-string>
pub(crate) fn parse_an_import_map_string(
    module_owner: ModuleOwner,
    input: Rc<DOMString>,
    base_url: ServoUrl,
) -> Fallible<ImportMap> {
    // Step 1. Let parsed be the result of parsing a JSON string to an Infra value given input.
    let parsed: JsonValue = serde_json::from_str(&input.str())
        .map_err(|_| Error::Type(c"The value needs to be a JSON object.".to_owned()))?;
    // Step 2. If parsed is not an ordered map, then throw a TypeError indicating that the
    // top-level value needs to be a JSON object.
    let JsonValue::Object(mut parsed) = parsed else {
        return Err(Error::Type(
            c"The top-level value needs to be a JSON object.".to_owned(),
        ));
    };

    // Step 3. Let sortedAndNormalizedImports be an empty ordered map.
    let mut sorted_and_normalized_imports = ModuleSpecifierMap::new();
    // Step 4. If parsed["imports"] exists, then:
    if let Some(imports) = parsed.get("imports") {
        // Step 4.1 If parsed["imports"] is not an ordered map, then throw a TypeError
        // indicating that the value for the "imports" top-level key needs to be a JSON object.
        let JsonValue::Object(imports) = imports else {
            return Err(Error::Type(
                c"The \"imports\" top-level value needs to be a JSON object.".to_owned(),
            ));
        };
        // Step 4.2 Set sortedAndNormalizedImports to the result of sorting and
        // normalizing a module specifier map given parsed["imports"] and baseURL.
        sorted_and_normalized_imports =
            sort_and_normalize_module_specifier_map(&module_owner.global(), imports, &base_url);
    }

    // Step 5. Let sortedAndNormalizedScopes be an empty ordered map.
    let mut sorted_and_normalized_scopes: IndexMap<ServoUrl, ModuleSpecifierMap> = IndexMap::new();
    // Step 6. If parsed["scopes"] exists, then:
    if let Some(scopes) = parsed.get("scopes") {
        // Step 6.1 If parsed["scopes"] is not an ordered map, then throw a TypeError
        // indicating that the value for the "scopes" top-level key needs to be a JSON object.
        let JsonValue::Object(scopes) = scopes else {
            return Err(Error::Type(
                c"The \"scopes\" top-level value needs to be a JSON object.".to_owned(),
            ));
        };
        // Step 6.2 Set sortedAndNormalizedScopes to the result of sorting and
        // normalizing scopes given parsed["scopes"] and baseURL.
        sorted_and_normalized_scopes =
            sort_and_normalize_scopes(&module_owner.global(), scopes, &base_url)?;
    }

    // Step 7. Let normalizedIntegrity be an empty ordered map.
    let mut normalized_integrity = ModuleIntegrityMap::new();
    // Step 8. If parsed["integrity"] exists, then:
    if let Some(integrity) = parsed.get("integrity") {
        // Step 8.1 If parsed["integrity"] is not an ordered map, then throw a TypeError
        // indicating that the value for the "integrity" top-level key needs to be a JSON object.
        let JsonValue::Object(integrity) = integrity else {
            return Err(Error::Type(
                c"The \"integrity\" top-level value needs to be a JSON object.".to_owned(),
            ));
        };
        // Step 8.2 Set normalizedIntegrity to the result of normalizing
        // a module integrity map given parsed["integrity"] and baseURL.
        normalized_integrity =
            normalize_module_integrity_map(&module_owner.global(), integrity, &base_url);
    }

    // Step 9. If parsed's keys contains any items besides "imports", "scopes", or "integrity",
    // then the user agent should report a warning to the console indicating that an invalid
    // top-level key was present in the import map.
    parsed.retain(|k, _| !matches!(k.as_str(), "imports" | "scopes" | "integrity"));
    if !parsed.is_empty() {
        Console::internal_warn(
            &module_owner.global(),
            DOMString::from(
                "Invalid top-level key was present in the import map.
                Only \"imports\", \"scopes\", and \"integrity\" are allowed.",
            ),
        );
    }

    // Step 10. Return an import map
    Ok(ImportMap {
        imports: sorted_and_normalized_imports,
        scopes: sorted_and_normalized_scopes,
        integrity: normalized_integrity,
    })
}

/// <https://html.spec.whatwg.org/multipage/#sorting-and-normalizing-a-module-specifier-map>
fn sort_and_normalize_module_specifier_map(
    global: &GlobalScope,
    original_map: &JsonMap<String, JsonValue>,
    base_url: &ServoUrl,
) -> ModuleSpecifierMap {
    // Step 1. Let normalized be an empty ordered map.
    let mut normalized = ModuleSpecifierMap::new();

    // Step 2. For each specifier_key -> value in originalMap
    for (specifier_key, value) in original_map {
        // Step 2.1 Let normalized_specifier_key be the result of
        // normalizing a specifier key given specifier_key and base_url.
        let Some(normalized_specifier_key) =
            normalize_specifier_key(global, specifier_key, base_url)
        else {
            // Step 2.2 If normalized_specifier_key is null, then continue.
            continue;
        };

        // Step 2.3 If value is not a string, then:
        let JsonValue::String(value) = value else {
            // Step 2.3.1 The user agent may report a warning to the console
            // indicating that addresses need to be strings.
            Console::internal_warn(global, DOMString::from("Addresses need to be strings."));

            // Step 2.3.2 Set normalized[normalized_specifier_key] to null.
            normalized.insert(normalized_specifier_key, None);
            // Step 2.3.3 Continue.
            continue;
        };

        // Step 2.4. Let address_url be the result of resolving a URL-like module specifier given value and baseURL.
        let Some(address_url) =
            ModuleTree::resolve_url_like_module_specifier(value.as_str(), base_url)
        else {
            // Step 2.5 If address_url is null, then:
            // Step 2.5.1. The user agent may report a warning to the console
            // indicating that the address was invalid.
            Console::internal_warn(
                global,
                DOMString::from(format!(
                    "Value failed to resolve to module specifier: {value}"
                )),
            );

            // Step 2.5.2 Set normalized[normalized_specifier_key] to null.
            normalized.insert(normalized_specifier_key, None);
            // Step 2.5.3 Continue.
            continue;
        };

        // Step 2.6 If specifier_key ends with U+002F (/), and the serialization of
        // address_url does not end with U+002F (/), then:
        if specifier_key.ends_with('\u{002f}') && !address_url.as_str().ends_with('\u{002f}') {
            // step 2.6.1. The user agent may report a warning to the console
            // indicating that an invalid address was given for the specifier key specifierKey;
            // since specifierKey ends with a slash, the address needs to as well.
            Console::internal_warn(
                global,
                DOMString::from(format!(
                    "Invalid address for specifier key '{specifier_key}': {address_url}.
                    Since specifierKey ends with a slash, the address needs to as well."
                )),
            );

            // Step 2.6.2 Set normalized[normalized_specifier_key] to null.
            normalized.insert(normalized_specifier_key, None);
            // Step 2.6.3 Continue.
            continue;
        }

        // Step 2.7 Set normalized[normalized_specifier_key] to address_url.
        normalized.insert(normalized_specifier_key, Some(address_url));
    }

    // Step 3. Return the result of sorting in descending order normalized
    // with an entry a being less than an entry b if a's key is code unit less than b's key.
    normalized.sort_by(|a_key, _, b_key, _| b_key.cmp(a_key));
    normalized
}

/// <https://html.spec.whatwg.org/multipage/#sorting-and-normalizing-scopes>
fn sort_and_normalize_scopes(
    global: &GlobalScope,
    original_map: &JsonMap<String, JsonValue>,
    base_url: &ServoUrl,
) -> Fallible<IndexMap<ServoUrl, ModuleSpecifierMap>> {
    // Step 1. Let normalized be an empty ordered map.
    let mut normalized: IndexMap<ServoUrl, ModuleSpecifierMap> = IndexMap::new();

    // Step 2. For each scopePrefix → potentialSpecifierMap of originalMap:
    for (scope_prefix, potential_specifier_map) in original_map {
        // Step 2.1 If potentialSpecifierMap is not an ordered map, then throw a TypeError indicating
        // that the value of the scope with prefix scopePrefix needs to be a JSON object.
        let JsonValue::Object(potential_specifier_map) = potential_specifier_map else {
            return Err(Error::Type(
                c"The value of the scope with prefix scopePrefix needs to be a JSON object."
                    .to_owned(),
            ));
        };

        // Step 2.2 Let scopePrefixURL be the result of URL parsing scopePrefix with baseURL.
        let Ok(scope_prefix_url) = ServoUrl::parse_with_base(Some(base_url), scope_prefix) else {
            // Step 2.3 If scopePrefixURL is failure, then:
            // Step 2.3.1 The user agent may report a warning
            // to the console that the scope prefix URL was not parseable.
            Console::internal_warn(
                global,
                DOMString::from(format!(
                    "Scope prefix URL was not parseable: {scope_prefix}"
                )),
            );
            // Step 2.3.2 Continue.
            continue;
        };

        // Step 2.4 Let normalizedScopePrefix be the serialization of scopePrefixURL.
        let normalized_scope_prefix = scope_prefix_url;

        // Step 2.5 Set normalized[normalizedScopePrefix] to the result of sorting and
        // normalizing a module specifier map given potentialSpecifierMap and baseURL.
        let normalized_specifier_map =
            sort_and_normalize_module_specifier_map(global, potential_specifier_map, base_url);
        normalized.insert(normalized_scope_prefix, normalized_specifier_map);
    }

    // Step 3. Return the result of sorting in descending order normalized,
    // with an entry a being less than an entry b if a's key is code unit less than b's key.
    normalized.sort_by(|a_key, _, b_key, _| b_key.cmp(a_key));
    Ok(normalized)
}

/// <https://html.spec.whatwg.org/multipage/#normalizing-a-module-integrity-map>
fn normalize_module_integrity_map(
    global: &GlobalScope,
    original_map: &JsonMap<String, JsonValue>,
    base_url: &ServoUrl,
) -> ModuleIntegrityMap {
    // Step 1. Let normalized be an empty ordered map.
    let mut normalized = ModuleIntegrityMap::new();

    // Step 2. For each key → value of originalMap:
    for (key, value) in original_map {
        // Step 2.1 Let resolvedURL be the result of
        // resolving a URL-like module specifier given key and baseURL.
        let Some(resolved_url) =
            ModuleTree::resolve_url_like_module_specifier(key.as_str(), base_url)
        else {
            // Step 2.2 If resolvedURL is null, then:
            // Step 2.2.1 The user agent may report a warning
            // to the console indicating that the key failed to resolve.
            Console::internal_warn(
                global,
                DOMString::from(format!("Key failed to resolve to module specifier: {key}")),
            );
            // Step 2.2.2 Continue.
            continue;
        };

        // Step 2.3 If value is not a string, then:
        let JsonValue::String(value) = value else {
            // Step 2.3.1 The user agent may report a warning
            // to the console indicating that integrity metadata values need to be strings.
            Console::internal_warn(
                global,
                DOMString::from("Integrity metadata values need to be strings."),
            );
            // Step 2.3.2 Continue.
            continue;
        };

        // Step 2.4 Set normalized[resolvedURL] to value.
        normalized.insert(resolved_url, value.clone());
    }

    // Step 3. Return normalized.
    normalized
}

/// <https://html.spec.whatwg.org/multipage/#normalizing-a-specifier-key>
fn normalize_specifier_key(
    global: &GlobalScope,
    specifier_key: &str,
    base_url: &ServoUrl,
) -> Option<String> {
    // step 1. If specifierKey is the empty string, then:
    if specifier_key.is_empty() {
        // step 1.1 The user agent may report a warning to the console
        // indicating that specifier keys may not be the empty string.
        Console::internal_warn(
            global,
            DOMString::from("Specifier keys may not be the empty string."),
        );
        // step 1.2 Return null.
        return None;
    }
    // step 2. Let url be the result of resolving a URL-like module specifier, given specifierKey and baseURL.
    let url = ModuleTree::resolve_url_like_module_specifier(specifier_key, base_url);

    // step 3. If url is not null, then return the serialization of url.
    if let Some(url) = url {
        return Some(url.into_string());
    }

    // step 4. Return specifierKey.
    Some(specifier_key.to_string())
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

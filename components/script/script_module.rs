/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! The script module mod contains common traits and structs
//! related to `type=module` for script thread or worker threads.

use std::collections::{HashMap, HashSet};
use std::rc::Rc;
use std::str::FromStr;
use std::sync::{Arc, Mutex};
use std::{mem, ptr};

use encoding_rs::UTF_8;
use headers::{HeaderMapExt, ReferrerPolicy as ReferrerPolicyHeader};
use html5ever::local_name;
use hyper_serde::Serde;
use indexmap::IndexSet;
use js::jsapi::{
    CompileModule1, ExceptionStackBehavior, FinishDynamicModuleImport, GetModuleRequestSpecifier,
    GetModuleResolveHook, GetRequestedModuleSpecifier, GetRequestedModulesCount,
    Handle as RawHandle, HandleObject, HandleValue as RawHandleValue, Heap, JSAutoRealm, JSContext,
    JSObject, JSRuntime, JSString, JS_ClearPendingException, JS_DefineProperty4,
    JS_IsExceptionPending, JS_NewStringCopyN, ModuleErrorBehaviour, ModuleEvaluate, ModuleLink,
    MutableHandleValue, SetModuleDynamicImportHook, SetModuleMetadataHook, SetModulePrivate,
    SetModuleResolveHook, SetScriptPrivateReferenceHooks, ThrowOnModuleEvaluationFailure, Value,
    JSPROP_ENUMERATE,
};
use js::jsval::{JSVal, PrivateValue, UndefinedValue};
use js::rust::jsapi_wrapped::JS_GetPendingException;
use js::rust::wrappers::JS_SetPendingException;
use js::rust::{
    transform_str_to_source_text, CompileOptionsWrapper, Handle, HandleObject as RustHandleObject,
    HandleValue, IntoHandle, MutableHandleObject as RustMutableHandleObject,
};
use mime::Mime;
use net_traits::http_status::HttpStatus;
use net_traits::request::{
    CredentialsMode, Destination, ParserMetadata, Referrer, RequestBuilder, RequestId, RequestMode,
};
use net_traits::{
    FetchMetadata, FetchResponseListener, Metadata, NetworkError, ReferrerPolicy,
    ResourceFetchTiming, ResourceTimingType,
};
use servo_url::ServoUrl;
use url::ParseError as UrlParseError;
use uuid::Uuid;

use crate::document_loader::LoadType;
use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::WindowBinding::Window_Binding::WindowMethods;
use crate::dom::bindings::conversions::jsstring_to_str;
use crate::dom::bindings::error::{report_pending_exception, Error};
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::refcounted::Trusted;
use crate::dom::bindings::reflector::{DomGlobal, DomObject};
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::settings_stack::AutoIncumbentScript;
use crate::dom::bindings::str::DOMString;
use crate::dom::bindings::trace::RootedTraceableBox;
use crate::dom::document::Document;
use crate::dom::dynamicmoduleowner::{DynamicModuleId, DynamicModuleOwner};
use crate::dom::element::Element;
use crate::dom::globalscope::GlobalScope;
use crate::dom::htmlscriptelement::{
    HTMLScriptElement, ScriptId, ScriptOrigin, ScriptType, SCRIPT_JS_MIMES,
};
use crate::dom::node::NodeTraits;
use crate::dom::performanceresourcetiming::InitiatorType;
use crate::dom::promise::Promise;
use crate::dom::promisenativehandler::{Callback, PromiseNativeHandler};
use crate::dom::window::Window;
use crate::dom::worker::TrustedWorkerAddress;
use crate::network_listener::{self, NetworkListener, PreInvoke, ResourceTimingListener};
use crate::realms::{enter_realm, AlreadyInRealm, InRealm};
use crate::script_runtime::{CanGc, JSContext as SafeJSContext};
use crate::task::TaskBox;

fn gen_type_error(global: &GlobalScope, string: String) -> RethrowError {
    rooted!(in(*GlobalScope::get_cx()) let mut thrown = UndefinedValue());
    Error::Type(string).to_jsval(GlobalScope::get_cx(), global, thrown.handle_mut());

    RethrowError(RootedTraceableBox::from_box(Heap::boxed(thrown.get())))
}

#[derive(JSTraceable)]
pub(crate) struct ModuleObject(Box<Heap<*mut JSObject>>);

impl ModuleObject {
    fn new(obj: RustHandleObject) -> ModuleObject {
        ModuleObject(Heap::boxed(obj.get()))
    }

    #[allow(unsafe_code)]
    pub(crate) fn handle(&self) -> HandleObject {
        unsafe { self.0.handle() }
    }
}

#[derive(JSTraceable)]
pub(crate) struct RethrowError(RootedTraceableBox<Heap<JSVal>>);

impl RethrowError {
    fn handle(&self) -> Handle<JSVal> {
        self.0.handle()
    }
}

impl Clone for RethrowError {
    fn clone(&self) -> Self {
        Self(RootedTraceableBox::from_box(Heap::boxed(self.0.get())))
    }
}

pub(crate) struct ModuleScript {
    base_url: ServoUrl,
    options: ScriptFetchOptions,
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

/// Identity for a module which will be
/// used to retrieve the module when we'd
/// like to get it from module map.
///
/// For example, we will save module parents with
/// module identity so that we can get module tree
/// from a descendant no matter the parent is an
/// inline script or a external script
#[derive(Clone, Debug, Eq, Hash, JSTraceable, PartialEq)]
pub(crate) enum ModuleIdentity {
    ScriptId(ScriptId),
    ModuleUrl(#[no_trace] ServoUrl),
}

impl ModuleIdentity {
    pub(crate) fn get_module_tree(&self, global: &GlobalScope) -> Rc<ModuleTree> {
        match self {
            ModuleIdentity::ModuleUrl(url) => {
                let module_map = global.get_module_map().borrow();
                module_map.get(&url.clone()).unwrap().clone()
            },
            ModuleIdentity::ScriptId(script_id) => {
                let inline_module_map = global.get_inline_module_map().borrow();
                inline_module_map.get(script_id).unwrap().clone()
            },
        }
    }
}

#[derive(JSTraceable)]
pub(crate) struct ModuleTree {
    #[no_trace]
    url: ServoUrl,
    text: DomRefCell<Rc<DOMString>>,
    record: DomRefCell<Option<ModuleObject>>,
    status: DomRefCell<ModuleStatus>,
    // The spec maintains load order for descendants, so we use an indexset for descendants and
    // parents. This isn't actually necessary for parents however the IndexSet APIs don't
    // interop with HashSet, and IndexSet isn't very expensive
    // (https://github.com/bluss/indexmap/issues/110)
    //
    // By default all maps in web specs are ordered maps
    // (https://infra.spec.whatwg.org/#ordered-map), however we can usually get away with using
    // stdlib maps and sets because we rarely iterate over them.
    #[custom_trace]
    parent_identities: DomRefCell<IndexSet<ModuleIdentity>>,
    #[no_trace]
    descendant_urls: DomRefCell<IndexSet<ServoUrl>>,
    // A set to memoize which descendants are under fetching
    #[no_trace]
    incomplete_fetch_urls: DomRefCell<IndexSet<ServoUrl>>,
    #[no_trace]
    visited_urls: DomRefCell<HashSet<ServoUrl>>,
    rethrow_error: DomRefCell<Option<RethrowError>>,
    #[no_trace]
    network_error: DomRefCell<Option<NetworkError>>,
    // A promise for owners to execute when the module tree
    // is finished
    promise: DomRefCell<Option<Rc<Promise>>>,
    external: bool,
}

impl ModuleTree {
    pub(crate) fn new(url: ServoUrl, external: bool, visited_urls: HashSet<ServoUrl>) -> Self {
        ModuleTree {
            url,
            text: DomRefCell::new(Rc::new(DOMString::new())),
            record: DomRefCell::new(None),
            status: DomRefCell::new(ModuleStatus::Initial),
            parent_identities: DomRefCell::new(IndexSet::new()),
            descendant_urls: DomRefCell::new(IndexSet::new()),
            incomplete_fetch_urls: DomRefCell::new(IndexSet::new()),
            visited_urls: DomRefCell::new(visited_urls),
            rethrow_error: DomRefCell::new(None),
            network_error: DomRefCell::new(None),
            promise: DomRefCell::new(None),
            external,
        }
    }

    pub(crate) fn get_status(&self) -> ModuleStatus {
        *self.status.borrow()
    }

    pub(crate) fn set_status(&self, status: ModuleStatus) {
        *self.status.borrow_mut() = status;
    }

    pub(crate) fn get_record(&self) -> &DomRefCell<Option<ModuleObject>> {
        &self.record
    }

    pub(crate) fn set_record(&self, record: ModuleObject) {
        *self.record.borrow_mut() = Some(record);
    }

    pub(crate) fn get_rethrow_error(&self) -> &DomRefCell<Option<RethrowError>> {
        &self.rethrow_error
    }

    pub(crate) fn set_rethrow_error(&self, rethrow_error: RethrowError) {
        *self.rethrow_error.borrow_mut() = Some(rethrow_error);
    }

    pub(crate) fn get_network_error(&self) -> &DomRefCell<Option<NetworkError>> {
        &self.network_error
    }

    pub(crate) fn set_network_error(&self, network_error: NetworkError) {
        *self.network_error.borrow_mut() = Some(network_error);
    }

    pub(crate) fn get_text(&self) -> &DomRefCell<Rc<DOMString>> {
        &self.text
    }

    pub(crate) fn set_text(&self, module_text: Rc<DOMString>) {
        *self.text.borrow_mut() = module_text;
    }

    pub(crate) fn get_incomplete_fetch_urls(&self) -> &DomRefCell<IndexSet<ServoUrl>> {
        &self.incomplete_fetch_urls
    }

    pub(crate) fn get_descendant_urls(&self) -> &DomRefCell<IndexSet<ServoUrl>> {
        &self.descendant_urls
    }

    pub(crate) fn get_parent_urls(&self) -> IndexSet<ServoUrl> {
        let parent_identities = self.parent_identities.borrow();

        parent_identities
            .iter()
            .filter_map(|parent_identity| match parent_identity {
                ModuleIdentity::ScriptId(_) => None,
                ModuleIdentity::ModuleUrl(url) => Some(url.clone()),
            })
            .collect()
    }

    pub(crate) fn insert_parent_identity(&self, parent_identity: ModuleIdentity) {
        self.parent_identities.borrow_mut().insert(parent_identity);
    }

    pub(crate) fn insert_incomplete_fetch_url(&self, dependency: &ServoUrl) {
        self.incomplete_fetch_urls
            .borrow_mut()
            .insert(dependency.clone());
    }

    pub(crate) fn remove_incomplete_fetch_url(&self, dependency: &ServoUrl) {
        self.incomplete_fetch_urls
            .borrow_mut()
            .shift_remove(dependency);
    }

    /// recursively checks if all of the transitive descendants are
    /// in the FetchingDescendants or later status
    fn recursive_check_descendants(
        module_tree: &ModuleTree,
        module_map: &HashMap<ServoUrl, Rc<ModuleTree>>,
        discovered_urls: &mut HashSet<ServoUrl>,
    ) -> bool {
        discovered_urls.insert(module_tree.url.clone());

        let descendant_urls = module_tree.descendant_urls.borrow();

        for descendant_url in descendant_urls.iter() {
            match module_map.get(&descendant_url.clone()) {
                None => return false,
                Some(descendant_module) => {
                    if discovered_urls.contains(&descendant_module.url) {
                        continue;
                    }

                    let descendant_status = descendant_module.get_status();
                    if descendant_status < ModuleStatus::FetchingDescendants {
                        return false;
                    }

                    let all_ready_descendants = ModuleTree::recursive_check_descendants(
                        descendant_module,
                        module_map,
                        discovered_urls,
                    );

                    if !all_ready_descendants {
                        return false;
                    }
                },
            }
        }

        true
    }

    fn has_all_ready_descendants(&self, global: &GlobalScope) -> bool {
        let module_map = global.get_module_map().borrow();
        let mut discovered_urls = HashSet::new();

        ModuleTree::recursive_check_descendants(self, &module_map.0, &mut discovered_urls)
    }

    // We just leverage the power of Promise to run the task for `finish` the owner.
    // Thus, we will always `resolve` it and no need to register a callback for `reject`
    fn append_handler(
        &self,
        owner: ModuleOwner,
        module_identity: ModuleIdentity,
        fetch_options: ScriptFetchOptions,
        can_gc: CanGc,
    ) {
        let this = owner.clone();
        let identity = module_identity.clone();
        let options = fetch_options.clone();

        let handler = PromiseNativeHandler::new(
            &owner.global(),
            Some(ModuleHandler::new_boxed(Box::new(
                task!(fetched_resolve: move || {
                    this.notify_owner_to_finish(identity, options, CanGc::note());
                }),
            ))),
            None,
            can_gc,
        );

        let realm = enter_realm(&*owner.global());
        let comp = InRealm::Entered(&realm);
        let _ais = AutoIncumbentScript::new(&owner.global());

        if let Some(promise) = self.promise.borrow().as_ref() {
            promise.append_native_handler(&handler, comp, can_gc);
            return;
        }

        let new_promise = Promise::new_in_current_realm(comp, can_gc);
        new_promise.append_native_handler(&handler, comp, can_gc);
        *self.promise.borrow_mut() = Some(new_promise);
    }

    fn append_dynamic_module_handler(
        &self,
        owner: ModuleOwner,
        module_identity: ModuleIdentity,
        dynamic_module: RootedTraceableBox<DynamicModule>,
        can_gc: CanGc,
    ) {
        let this = owner.clone();
        let identity = module_identity.clone();

        let module_id = owner.global().dynamic_module_list().push(dynamic_module);

        let handler = PromiseNativeHandler::new(
            &owner.global(),
            Some(ModuleHandler::new_boxed(Box::new(
                task!(fetched_resolve: move || {
                    this.finish_dynamic_module(identity, module_id, CanGc::note());
                }),
            ))),
            None,
            can_gc,
        );

        let realm = enter_realm(&*owner.global());
        let comp = InRealm::Entered(&realm);
        let _ais = AutoIncumbentScript::new(&owner.global());

        if let Some(promise) = self.promise.borrow().as_ref() {
            promise.append_native_handler(&handler, comp, can_gc);
            return;
        }

        let new_promise = Promise::new_in_current_realm(comp, can_gc);
        new_promise.append_native_handler(&handler, comp, can_gc);
        *self.promise.borrow_mut() = Some(new_promise);
    }
}

#[derive(Clone, Copy, Debug, JSTraceable, PartialEq, PartialOrd)]
pub(crate) enum ModuleStatus {
    Initial,
    Fetching,
    FetchingDescendants,
    Finished,
}

struct ModuleSource {
    source: Rc<DOMString>,
    unminified_dir: Option<String>,
    external: bool,
    url: ServoUrl,
}

impl crate::unminify::ScriptSource for ModuleSource {
    fn unminified_dir(&self) -> Option<String> {
        self.unminified_dir.clone()
    }

    fn extract_bytes(&self) -> &[u8] {
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
    #[allow(unsafe_code, clippy::too_many_arguments)]
    /// <https://html.spec.whatwg.org/multipage/#creating-a-module-script>
    /// Step 7-11.
    /// Although the CanGc argument appears unused, it represents the GC operations that
    /// can occur as part of compiling a script.
    fn compile_module_script(
        &self,
        global: &GlobalScope,
        owner: ModuleOwner,
        module_script_text: Rc<DOMString>,
        url: &ServoUrl,
        options: ScriptFetchOptions,
        mut module_script: RustMutableHandleObject,
        inline: bool,
        _can_gc: CanGc,
    ) -> Result<(), RethrowError> {
        let cx = GlobalScope::get_cx();
        let _ac = JSAutoRealm::new(*cx, *global.reflector().get_jsobject());

        let compile_options = unsafe { CompileOptionsWrapper::new(*cx, url.as_str(), 1) };
        let mut module_source = ModuleSource {
            source: module_script_text,
            unminified_dir: global.unminified_js_dir(),
            external: !inline,
            url: url.clone(),
        };
        crate::unminify::unminify_js(&mut module_source);

        unsafe {
            module_script.set(CompileModule1(
                *cx,
                compile_options.ptr,
                &mut transform_str_to_source_text(&module_source.source),
            ));

            if module_script.is_null() {
                warn!("fail to compile module script of {}", url);

                rooted!(in(*cx) let mut exception = UndefinedValue());
                assert!(JS_GetPendingException(*cx, &mut exception.handle_mut()));
                JS_ClearPendingException(*cx);

                return Err(RethrowError(RootedTraceableBox::from_box(Heap::boxed(
                    exception.get(),
                ))));
            }

            let module_script_data = Rc::new(ModuleScript::new(url.clone(), options, Some(owner)));

            SetModulePrivate(
                module_script.get(),
                &PrivateValue(Rc::into_raw(module_script_data) as *const _),
            );

            debug!("module script of {} compile done", url);

            self.resolve_requested_module_specifiers(
                global,
                module_script.handle().into_handle(),
                url,
            )
            .map(|_| ())
        }
    }

    #[allow(unsafe_code)]
    /// <https://html.spec.whatwg.org/multipage/#fetch-the-descendants-of-and-link-a-module-script>
    /// Step 5-2.
    pub(crate) fn instantiate_module_tree(
        &self,
        global: &GlobalScope,
        module_record: HandleObject,
    ) -> Result<(), RethrowError> {
        let cx = GlobalScope::get_cx();
        let _ac = JSAutoRealm::new(*cx, *global.reflector().get_jsobject());

        unsafe {
            if !ModuleLink(*cx, module_record) {
                warn!("fail to link & instantiate module");

                rooted!(in(*cx) let mut exception = UndefinedValue());
                assert!(JS_GetPendingException(*cx, &mut exception.handle_mut()));
                JS_ClearPendingException(*cx);

                Err(RethrowError(RootedTraceableBox::from_box(Heap::boxed(
                    exception.get(),
                ))))
            } else {
                debug!("module instantiated successfully");

                Ok(())
            }
        }
    }

    /// Execute the provided module, storing the evaluation return value in the provided
    /// mutable handle. Although the CanGc appears unused, it represents the GC operations
    /// possible when evluating arbitrary JS.
    #[allow(unsafe_code)]
    pub(crate) fn execute_module(
        &self,
        global: &GlobalScope,
        module_record: HandleObject,
        eval_result: MutableHandleValue,
        _can_gc: CanGc,
    ) -> Result<(), RethrowError> {
        let cx = GlobalScope::get_cx();
        let _ac = JSAutoRealm::new(*cx, *global.reflector().get_jsobject());

        unsafe {
            let ok = ModuleEvaluate(*cx, module_record, eval_result);
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
                assert!(JS_GetPendingException(*cx, &mut exception.handle_mut()));
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

    #[allow(unsafe_code)]
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

    #[allow(unsafe_code)]
    fn resolve_requested_module_specifiers(
        &self,
        global: &GlobalScope,
        module_object: HandleObject,
        base_url: &ServoUrl,
    ) -> Result<IndexSet<ServoUrl>, RethrowError> {
        let cx = GlobalScope::get_cx();
        let _ac = JSAutoRealm::new(*cx, *global.reflector().get_jsobject());

        let mut specifier_urls = IndexSet::new();

        unsafe {
            let length = GetRequestedModulesCount(*cx, module_object);

            for index in 0..length {
                rooted!(in(*cx) let specifier = GetRequestedModuleSpecifier(
                    *cx, module_object, index
                ));

                let url = ModuleTree::resolve_module_specifier(
                    *cx,
                    base_url,
                    specifier.handle().into_handle(),
                );

                if url.is_err() {
                    let specifier_error =
                        gen_type_error(global, "Wrong module specifier".to_owned());

                    return Err(specifier_error);
                }

                specifier_urls.insert(url.unwrap());
            }
        }

        Ok(specifier_urls)
    }

    /// The following module specifiers are allowed by the spec:
    ///  - a valid absolute URL
    ///  - a valid relative URL that starts with "/", "./" or "../"
    ///
    /// Bareword module specifiers are currently disallowed as these may be given
    /// special meanings in the future.
    /// <https://html.spec.whatwg.org/multipage/#resolve-a-module-specifier>
    #[allow(unsafe_code)]
    fn resolve_module_specifier(
        cx: *mut JSContext,
        url: &ServoUrl,
        specifier: RawHandle<*mut JSString>,
    ) -> Result<ServoUrl, UrlParseError> {
        let specifier_str = unsafe { jsstring_to_str(cx, ptr::NonNull::new(*specifier).unwrap()) };

        // Step 1.
        if let Ok(specifier_url) = ServoUrl::parse(&specifier_str) {
            return Ok(specifier_url);
        }

        // Step 2.
        if !specifier_str.starts_with('/') &&
            !specifier_str.starts_with("./") &&
            !specifier_str.starts_with("../")
        {
            return Err(UrlParseError::InvalidDomainCharacter);
        }

        // Step 3.
        ServoUrl::parse_with_base(Some(url), &specifier_str)
    }

    /// <https://html.spec.whatwg.org/multipage/#finding-the-first-parse-error>
    fn find_first_parse_error(
        &self,
        global: &GlobalScope,
        discovered_urls: &mut HashSet<ServoUrl>,
    ) -> (Option<NetworkError>, Option<RethrowError>) {
        // 3.
        discovered_urls.insert(self.url.clone());

        // 4.
        let record = self.get_record().borrow();
        if record.is_none() {
            return (
                self.network_error.borrow().clone(),
                self.rethrow_error.borrow().clone(),
            );
        }

        let module_map = global.get_module_map().borrow();
        let mut parse_error: Option<RethrowError> = None;

        // 5-6.
        let descendant_urls = self.descendant_urls.borrow();
        for descendant_module in descendant_urls
            .iter()
            // 7.
            .filter_map(|url| module_map.get(&url.clone()))
        {
            // 8-2.
            if discovered_urls.contains(&descendant_module.url) {
                continue;
            }

            // 8-3.
            let (child_network_error, child_parse_error) =
                descendant_module.find_first_parse_error(global, discovered_urls);

            // Due to network error's priority higher than parse error,
            // we will return directly when we meet a network error.
            if child_network_error.is_some() {
                return (child_network_error, None);
            }

            // 8-4.
            //
            // In case of having any network error in other descendants,
            // we will store the "first" parse error and keep running this
            // loop to ensure we don't have any network error.
            if child_parse_error.is_some() && parse_error.is_none() {
                parse_error = child_parse_error;
            }
        }

        // Step 9.
        (None, parse_error)
    }

    #[allow(unsafe_code)]
    /// <https://html.spec.whatwg.org/multipage/#fetch-the-descendants-of-a-module-script>
    fn fetch_module_descendants(
        &self,
        owner: &ModuleOwner,
        destination: Destination,
        options: &ScriptFetchOptions,
        parent_identity: ModuleIdentity,
        can_gc: CanGc,
    ) {
        debug!("Start to load dependencies of {}", self.url);

        let global = owner.global();

        self.set_status(ModuleStatus::FetchingDescendants);

        let specifier_urls = {
            let raw_record = self.record.borrow();
            match raw_record.as_ref() {
                // Step 1.
                None => {
                    self.set_status(ModuleStatus::Finished);
                    debug!(
                        "Module {} doesn't have module record but tried to load descendants.",
                        self.url
                    );
                    return;
                },
                // Step 5.
                Some(raw_record) => self.resolve_requested_module_specifiers(
                    &global,
                    raw_record.handle(),
                    &self.url,
                ),
            }
        };

        match specifier_urls {
            // Step 3.
            Ok(valid_specifier_urls) if valid_specifier_urls.is_empty() => {
                debug!("Module {} doesn't have any dependencies.", self.url);
                self.advance_finished_and_link(&global, can_gc);
            },
            Ok(valid_specifier_urls) => {
                self.descendant_urls
                    .borrow_mut()
                    .extend(valid_specifier_urls.clone());

                let urls_to_fetch = {
                    let mut urls = IndexSet::new();
                    let mut visited_urls = self.visited_urls.borrow_mut();

                    for parsed_url in &valid_specifier_urls {
                        // Step 5-3.
                        if !visited_urls.contains(parsed_url) {
                            // Step 5-3-1.
                            urls.insert(parsed_url.clone());
                            // Step 5-3-2.
                            visited_urls.insert(parsed_url.clone());

                            self.insert_incomplete_fetch_url(parsed_url);
                        }
                    }
                    urls
                };

                // Step 3.
                if urls_to_fetch.is_empty() {
                    debug!(
                        "After checking with visited urls, module {} doesn't have dependencies to load.",
                        &self.url
                    );
                    self.advance_finished_and_link(&global, can_gc);
                    return;
                }

                // Step 8.

                let visited_urls = self.visited_urls.borrow().clone();
                let options = options.descendant_fetch_options();

                for url in urls_to_fetch {
                    // https://html.spec.whatwg.org/multipage/#internal-module-script-graph-fetching-procedure
                    // Step 1.
                    assert!(self.visited_urls.borrow().contains(&url));

                    // Step 2.
                    fetch_single_module_script(
                        owner.clone(),
                        url,
                        visited_urls.clone(),
                        destination,
                        options.clone(),
                        Some(parent_identity.clone()),
                        false,
                        None,
                        can_gc,
                    );
                }
            },
            Err(error) => {
                self.set_rethrow_error(error);
                self.advance_finished_and_link(&global, can_gc);
            },
        }
    }

    /// <https://html.spec.whatwg.org/multipage/#fetch-the-descendants-of-and-link-a-module-script>
    /// step 4-7.
    fn advance_finished_and_link(&self, global: &GlobalScope, can_gc: CanGc) {
        {
            if !self.has_all_ready_descendants(global) {
                return;
            }
        }

        self.set_status(ModuleStatus::Finished);

        debug!("Going to advance and finish for: {}", self.url);

        {
            // Notify parents of this module to finish
            //
            // Before notifying, if the parent module has already had zero incomplete
            // fetches, then it means we don't need to notify it.
            let parent_identities = self.parent_identities.borrow();
            for parent_identity in parent_identities.iter() {
                let parent_tree = parent_identity.get_module_tree(global);

                let incomplete_count_before_remove = {
                    let incomplete_urls = parent_tree.get_incomplete_fetch_urls().borrow();
                    incomplete_urls.len()
                };

                if incomplete_count_before_remove > 0 {
                    parent_tree.remove_incomplete_fetch_url(&self.url);
                    parent_tree.advance_finished_and_link(global, can_gc);
                }
            }
        }

        let mut discovered_urls: HashSet<ServoUrl> = HashSet::new();
        let (network_error, rethrow_error) =
            self.find_first_parse_error(global, &mut discovered_urls);

        match (network_error, rethrow_error) {
            (Some(network_error), _) => {
                self.set_network_error(network_error);
            },
            (None, None) => {
                let module_record = self.get_record().borrow();
                if let Some(record) = &*module_record {
                    let instantiated = self.instantiate_module_tree(global, record.handle());

                    if let Err(exception) = instantiated {
                        self.set_rethrow_error(exception);
                    }
                }
            },
            (None, Some(error)) => {
                self.set_rethrow_error(error);
            },
        }

        let promise = self.promise.borrow();
        if let Some(promise) = promise.as_ref() {
            promise.resolve_native(&(), can_gc);
        }
    }
}

#[derive(JSTraceable, MallocSizeOf)]
struct ModuleHandler {
    #[ignore_malloc_size_of = "Measuring trait objects is hard"]
    task: DomRefCell<Option<Box<dyn TaskBox>>>,
}

impl ModuleHandler {
    pub(crate) fn new_boxed(task: Box<dyn TaskBox>) -> Box<dyn Callback> {
        Box::new(Self {
            task: DomRefCell::new(Some(task)),
        })
    }
}

impl Callback for ModuleHandler {
    fn callback(&self, _cx: SafeJSContext, _v: HandleValue, _realm: InRealm, _can_gc: CanGc) {
        let task = self.task.borrow_mut().take().unwrap();
        task.run_box();
    }
}

/// The owner of the module
/// It can be `worker` or `script` element
#[derive(Clone)]
pub(crate) enum ModuleOwner {
    #[allow(dead_code)]
    Worker(TrustedWorkerAddress),
    Window(Trusted<HTMLScriptElement>),
    DynamicModule(Trusted<DynamicModuleOwner>),
}

impl ModuleOwner {
    pub(crate) fn global(&self) -> DomRoot<GlobalScope> {
        match &self {
            ModuleOwner::Worker(worker) => (*worker.root().clone()).global(),
            ModuleOwner::Window(script) => (*script.root()).global(),
            ModuleOwner::DynamicModule(dynamic_module) => (*dynamic_module.root()).global(),
        }
    }

    pub(crate) fn notify_owner_to_finish(
        &self,
        module_identity: ModuleIdentity,
        fetch_options: ScriptFetchOptions,
        can_gc: CanGc,
    ) {
        match &self {
            ModuleOwner::Worker(_) => unimplemented!(),
            ModuleOwner::DynamicModule(_) => unimplemented!(),
            ModuleOwner::Window(script) => {
                let global = self.global();

                let document = script.root().owner_document();
                let load = {
                    let module_tree = module_identity.get_module_tree(&global);

                    let network_error = module_tree.get_network_error().borrow();
                    match network_error.as_ref() {
                        Some(network_error) => Err(network_error.clone().into()),
                        None => match module_identity {
                            ModuleIdentity::ModuleUrl(script_src) => Ok(ScriptOrigin::external(
                                Rc::clone(&module_tree.get_text().borrow()),
                                script_src.clone(),
                                fetch_options,
                                ScriptType::Module,
                                global.unminified_js_dir(),
                            )),
                            ModuleIdentity::ScriptId(_) => Ok(ScriptOrigin::internal(
                                Rc::clone(&module_tree.get_text().borrow()),
                                document.base_url().clone(),
                                fetch_options,
                                ScriptType::Module,
                                global.unminified_js_dir(),
                            )),
                        },
                    }
                };

                let asynch = script
                    .root()
                    .upcast::<Element>()
                    .has_attribute(&local_name!("async"));

                if !asynch && (*script.root()).get_parser_inserted() {
                    document.deferred_script_loaded(&script.root(), load);
                } else if !asynch && !(*script.root()).get_non_blocking() {
                    document.asap_in_order_script_loaded(&script.root(), load, can_gc);
                } else {
                    document.asap_script_loaded(&script.root(), load, can_gc);
                };
            },
        }
    }

    #[allow(unsafe_code)]
    /// <https://html.spec.whatwg.org/multipage/#hostimportmoduledynamically(referencingscriptormodule,-specifier,-promisecapability):fetch-an-import()-module-script-graph>
    /// Step 6-9
    fn finish_dynamic_module(
        &self,
        module_identity: ModuleIdentity,
        dynamic_module_id: DynamicModuleId,
        can_gc: CanGc,
    ) {
        let global = self.global();

        let module = global.dynamic_module_list().remove(dynamic_module_id);

        let cx = GlobalScope::get_cx();
        let module_tree = module_identity.get_module_tree(&global);

        // In the timing of executing this `finish_dynamic_module` function,
        // we've run `find_first_parse_error` which means we've had the highest
        // priority error in the tree. So, we can just get both `network_error` and
        // `rethrow_error` directly here.
        let network_error = module_tree.get_network_error().borrow().as_ref().cloned();
        let existing_rethrow_error = module_tree.get_rethrow_error().borrow().as_ref().cloned();

        rooted!(in(*cx) let mut rval = UndefinedValue());
        if network_error.is_none() && existing_rethrow_error.is_none() {
            let record = module_tree
                .get_record()
                .borrow()
                .as_ref()
                .map(|record| record.handle());

            if let Some(record) = record {
                let evaluated = module_tree
                    .execute_module(&global, record, rval.handle_mut().into(), can_gc)
                    .err();

                if let Some(exception) = evaluated.clone() {
                    module_tree.set_rethrow_error(exception);
                }
            }
        }

        // Ensure any failures related to importing this dynamic module are immediately reported.
        match (network_error, existing_rethrow_error) {
            (Some(_), _) => unsafe {
                let err = gen_type_error(&global, "Dynamic import failed".to_owned());
                JS_SetPendingException(*cx, err.handle(), ExceptionStackBehavior::Capture);
            },
            (None, Some(rethrow_error)) => unsafe {
                JS_SetPendingException(
                    *cx,
                    rethrow_error.handle(),
                    ExceptionStackBehavior::Capture,
                );
            },
            // do nothing if there's no errors
            (None, None) => {},
        };

        debug!("Finishing dynamic import for {:?}", module_identity);

        rooted!(in(*cx) let mut evaluation_promise = ptr::null_mut::<JSObject>());
        if rval.is_object() {
            evaluation_promise.set(rval.to_object());
        }

        unsafe {
            let ok = FinishDynamicModuleImport(
                *cx,
                evaluation_promise.handle().into(),
                module.referencing_private.handle(),
                module.specifier.handle(),
                module.promise.reflector().get_jsobject().into_handle(),
            );
            if ok {
                assert!(!JS_IsExceptionPending(*cx));
            } else {
                warn!("failed to finish dynamic module import");
            }
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
    /// The initial URL requested.
    url: ServoUrl,
    /// Destination of current module context
    destination: Destination,
    /// Options for the current script fetch
    options: ScriptFetchOptions,
    /// Indicates whether the request failed, and why
    status: Result<(), NetworkError>,
    /// Timing object for this resource
    resource_timing: ResourceFetchTiming,
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
    #[allow(unsafe_code)]
    fn process_response_eof(
        &mut self,
        _: RequestId,
        response: Result<ResourceFetchTiming, NetworkError>,
    ) {
        let global = self.owner.global();

        if let Some(window) = global.downcast::<Window>() {
            window
                .Document()
                .finish_load(LoadType::Script(self.url.clone()), CanGc::note());
        }

        // Step 9-1 & 9-2.
        let load = response.and(self.status.clone()).and_then(|_| {
            // Step 9-3.
            let meta = self.metadata.take().unwrap();

            if let Some(content_type) = meta.content_type.map(Serde::into_inner) {
                if let Ok(content_type) = Mime::from_str(&content_type.to_string()) {
                    let essence_mime = content_type.essence_str();

                    if !SCRIPT_JS_MIMES.contains(&essence_mime) {
                        return Err(NetworkError::Internal(format!(
                            "Invalid MIME type: {}",
                            essence_mime
                        )));
                    }
                } else {
                    return Err(NetworkError::Internal(format!(
                        "Failed to parse MIME type: {}",
                        content_type
                    )));
                }
            } else {
                return Err(NetworkError::Internal("No MIME type".into()));
            }

            // Step 13.4: Let referrerPolicy be the result of parsing the `Referrer-Policy` header
            // given response.
            let referrer_policy = meta
                .headers
                .and_then(|headers| headers.typed_get::<ReferrerPolicyHeader>())
                .into();

            // Step 13.5: If referrerPolicy is not the empty string, set options's referrer policy
            // to referrerPolicy.
            if referrer_policy != ReferrerPolicy::EmptyString {
                self.options.referrer_policy = referrer_policy;
            }

            // Step 10.
            let (source_text, _, _) = UTF_8.decode(&self.data);
            Ok(ScriptOrigin::external(
                Rc::new(DOMString::from(source_text)),
                meta.final_url,
                self.options.clone(),
                ScriptType::Module,
                global.unminified_js_dir(),
            ))
        });

        let module_tree = {
            let module_map = global.get_module_map().borrow();
            module_map.get(&self.url).unwrap().clone()
        };

        module_tree.remove_incomplete_fetch_url(&self.url);

        // Step 12.
        match load {
            Err(err) => {
                error!("Failed to fetch {} with error {:?}", &self.url, err);
                module_tree.set_network_error(err);
                module_tree.advance_finished_and_link(&global, CanGc::note());
            },
            Ok(ref resp_mod_script) => {
                module_tree.set_text(resp_mod_script.text());

                let cx = GlobalScope::get_cx();
                rooted!(in(*cx) let mut compiled_module: *mut JSObject = ptr::null_mut());
                let compiled_module_result = module_tree.compile_module_script(
                    &global,
                    self.owner.clone(),
                    resp_mod_script.text(),
                    &self.url,
                    self.options.clone(),
                    compiled_module.handle_mut(),
                    false,
                    CanGc::note(),
                );

                match compiled_module_result {
                    Err(exception) => {
                        module_tree.set_rethrow_error(exception);
                        module_tree.advance_finished_and_link(&global, CanGc::note());
                    },
                    Ok(_) => {
                        module_tree.set_record(ModuleObject::new(compiled_module.handle()));

                        module_tree.fetch_module_descendants(
                            &self.owner,
                            self.destination,
                            &self.options,
                            ModuleIdentity::ModuleUrl(self.url.clone()),
                            CanGc::note(),
                        );
                    },
                }
            },
        }
    }

    fn resource_timing_mut(&mut self) -> &mut ResourceFetchTiming {
        &mut self.resource_timing
    }

    fn resource_timing(&self) -> &ResourceFetchTiming {
        &self.resource_timing
    }

    fn submit_resource_timing(&mut self) {
        network_listener::submit_timing(self, CanGc::note())
    }
}

impl ResourceTimingListener for ModuleContext {
    fn resource_timing_information(&self) -> (InitiatorType, ServoUrl) {
        let initiator_type = InitiatorType::LocalName("module".to_string());
        (initiator_type, self.url.clone())
    }

    fn resource_timing_global(&self) -> DomRoot<GlobalScope> {
        self.owner.global()
    }
}

impl PreInvoke for ModuleContext {}

#[allow(unsafe_code, non_snake_case)]
/// A function to register module hooks (e.g. listening on resolving modules,
/// getting module metadata, getting script private reference and resolving dynamic import)
pub(crate) unsafe fn EnsureModuleHooksInitialized(rt: *mut JSRuntime) {
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

#[allow(unsafe_code)]
unsafe extern "C" fn host_add_ref_top_level_script(value: *const Value) {
    let val = Rc::from_raw((*value).to_private() as *const ModuleScript);
    mem::forget(val.clone());
    mem::forget(val);
}

#[allow(unsafe_code)]
unsafe extern "C" fn host_release_top_level_script(value: *const Value) {
    let _val = Rc::from_raw((*value).to_private() as *const ModuleScript);
}

#[allow(unsafe_code)]
/// <https://html.spec.whatwg.org/multipage/#hostimportmoduledynamically(referencingscriptormodule,-specifier,-promisecapability)>
pub(crate) unsafe extern "C" fn host_import_module_dynamically(
    cx: *mut JSContext,
    reference_private: RawHandleValue,
    specifier: RawHandle<*mut JSObject>,
    promise: RawHandle<*mut JSObject>,
) -> bool {
    // Step 1.
    let cx = SafeJSContext::from_ptr(cx);
    let in_realm_proof = AlreadyInRealm::assert_for_cx(cx);
    let global_scope = GlobalScope::from_context(*cx, InRealm::Already(&in_realm_proof));

    // Step 2.
    let mut base_url = global_scope.api_base_url();

    // Step 3.
    let mut options = ScriptFetchOptions::default_classic_script(&global_scope);

    // Step 4.
    let module_data = module_script_from_reference_private(&reference_private);
    if let Some(data) = module_data {
        base_url = data.base_url.clone();
        options = data.options.descendant_fetch_options();
    }

    let promise = Promise::new_with_js_promise(Handle::from_raw(promise), cx);

    //Step 5 & 6.
    if let Err(e) = fetch_an_import_module_script_graph(
        &global_scope,
        specifier,
        reference_private,
        base_url,
        options,
        promise,
        CanGc::note(),
    ) {
        JS_SetPendingException(*cx, e.handle(), ExceptionStackBehavior::Capture);
        return false;
    }

    true
}

#[derive(Clone, JSTraceable, MallocSizeOf)]
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
    fn descendant_fetch_options(&self) -> ScriptFetchOptions {
        Self {
            referrer: self.referrer.clone(),
            integrity_metadata: String::new(),
            cryptographic_nonce: self.cryptographic_nonce.clone(),
            credentials_mode: self.credentials_mode,
            parser_metadata: self.parser_metadata,
            referrer_policy: self.referrer_policy,
        }
    }
}

#[allow(unsafe_code)]
unsafe fn module_script_from_reference_private(
    reference_private: &RawHandle<JSVal>,
) -> Option<&ModuleScript> {
    if reference_private.get().is_undefined() {
        return None;
    }
    (reference_private.get().to_private() as *const ModuleScript).as_ref()
}

/// <https://html.spec.whatwg.org/multipage/#fetch-an-import()-module-script-graph>
#[allow(unsafe_code)]
fn fetch_an_import_module_script_graph(
    global: &GlobalScope,
    module_request: RawHandle<*mut JSObject>,
    reference_private: RawHandleValue,
    base_url: ServoUrl,
    options: ScriptFetchOptions,
    promise: Rc<Promise>,
    can_gc: CanGc,
) -> Result<(), RethrowError> {
    // Step 1.
    let cx = GlobalScope::get_cx();
    rooted!(in(*cx) let specifier = unsafe { GetModuleRequestSpecifier(*cx, module_request) });
    let url = ModuleTree::resolve_module_specifier(*cx, &base_url, specifier.handle().into());

    // Step 2.
    if url.is_err() {
        let specifier_error = gen_type_error(global, "Wrong module specifier".to_owned());
        return Err(specifier_error);
    }

    let dynamic_module_id = DynamicModuleId(Uuid::new_v4());

    // Step 3.
    let owner = match unsafe { module_script_from_reference_private(&reference_private) } {
        Some(module_data) if module_data.owner.is_some() => module_data.owner.clone().unwrap(),
        _ => ModuleOwner::DynamicModule(Trusted::new(&DynamicModuleOwner::new(
            global,
            promise.clone(),
            dynamic_module_id,
            can_gc,
        ))),
    };

    let dynamic_module = RootedTraceableBox::new(DynamicModule {
        promise,
        specifier: Heap::default(),
        referencing_private: Heap::default(),
        id: dynamic_module_id,
    });
    dynamic_module.specifier.set(module_request.get());
    dynamic_module
        .referencing_private
        .set(reference_private.get());

    let url = url.unwrap();

    let mut visited_urls = HashSet::new();
    visited_urls.insert(url.clone());

    fetch_single_module_script(
        owner,
        url,
        visited_urls,
        Destination::Script,
        options,
        None,
        true,
        Some(dynamic_module),
        can_gc,
    );
    Ok(())
}

#[allow(unsafe_code, non_snake_case)]
/// <https://tc39.github.io/ecma262/#sec-hostresolveimportedmodule>
/// <https://html.spec.whatwg.org/multipage/#hostresolveimportedmodule(referencingscriptormodule%2C-specifier)>
unsafe extern "C" fn HostResolveImportedModule(
    cx: *mut JSContext,
    reference_private: RawHandleValue,
    specifier: RawHandle<*mut JSObject>,
) -> *mut JSObject {
    let in_realm_proof = AlreadyInRealm::assert_for_cx(SafeJSContext::from_ptr(cx));
    let global_scope = GlobalScope::from_context(cx, InRealm::Already(&in_realm_proof));

    // Step 2.
    let mut base_url = global_scope.api_base_url();

    // Step 3.
    let module_data = module_script_from_reference_private(&reference_private);
    if let Some(data) = module_data {
        base_url = data.base_url.clone();
    }

    // Step 5.
    rooted!(in(*GlobalScope::get_cx()) let specifier = GetModuleRequestSpecifier(cx, specifier));
    let url = ModuleTree::resolve_module_specifier(
        *GlobalScope::get_cx(),
        &base_url,
        specifier.handle().into(),
    );

    // Step 6.
    assert!(url.is_ok());

    let parsed_url = url.unwrap();

    // Step 4 & 7.
    let module_map = global_scope.get_module_map().borrow();

    let module_tree = module_map.get(&parsed_url);

    // Step 9.
    assert!(module_tree.is_some());

    let fetched_module_object = module_tree.unwrap().get_record().borrow();

    // Step 8.
    assert!(fetched_module_object.is_some());

    // Step 10.
    if let Some(record) = &*fetched_module_object {
        return record.handle().get();
    }

    unreachable!()
}

#[allow(unsafe_code, non_snake_case)]
/// <https://tc39.es/ecma262/#sec-hostgetimportmetaproperties>
/// <https://html.spec.whatwg.org/multipage/#hostgetimportmetaproperties>
unsafe extern "C" fn HostPopulateImportMeta(
    cx: *mut JSContext,
    reference_private: RawHandleValue,
    meta_object: RawHandle<*mut JSObject>,
) -> bool {
    let in_realm_proof = AlreadyInRealm::assert_for_cx(SafeJSContext::from_ptr(cx));
    let global_scope = GlobalScope::from_context(cx, InRealm::Already(&in_realm_proof));

    // Step 2.
    let base_url = match module_script_from_reference_private(&reference_private) {
        Some(module_data) => module_data.base_url.clone(),
        None => global_scope.api_base_url(),
    };

    rooted!(in(cx) let url_string = JS_NewStringCopyN(
        cx,
        base_url.as_str().as_ptr() as *const _,
        base_url.as_str().len()
    ));

    // Step 3.
    JS_DefineProperty4(
        cx,
        meta_object,
        c"url".as_ptr(),
        url_string.handle().into_handle(),
        JSPROP_ENUMERATE.into(),
    )
}

/// <https://html.spec.whatwg.org/multipage/#fetch-a-module-script-tree>
pub(crate) fn fetch_external_module_script(
    owner: ModuleOwner,
    url: ServoUrl,
    destination: Destination,
    options: ScriptFetchOptions,
    can_gc: CanGc,
) {
    let mut visited_urls = HashSet::new();
    visited_urls.insert(url.clone());

    // Step 1.
    fetch_single_module_script(
        owner,
        url,
        visited_urls,
        destination,
        options,
        None,
        true,
        None,
        can_gc,
    )
}

#[derive(JSTraceable, MallocSizeOf)]
#[cfg_attr(crown, crown::unrooted_must_root_lint::must_root)]
pub(crate) struct DynamicModuleList {
    requests: Vec<RootedTraceableBox<DynamicModule>>,

    #[ignore_malloc_size_of = "Define in uuid"]
    next_id: DynamicModuleId,
}

impl DynamicModuleList {
    pub(crate) fn new() -> Self {
        Self {
            requests: vec![],
            next_id: DynamicModuleId(Uuid::new_v4()),
        }
    }

    fn push(&mut self, mut module: RootedTraceableBox<DynamicModule>) -> DynamicModuleId {
        let id = self.next_id;
        self.next_id = DynamicModuleId(Uuid::new_v4());
        module.id = id;
        self.requests.push(module);
        id
    }

    fn remove(&mut self, id: DynamicModuleId) -> RootedTraceableBox<DynamicModule> {
        let index = self
            .requests
            .iter()
            .position(|module| module.id == id)
            .expect("missing dynamic module");
        self.requests.remove(index)
    }
}

#[cfg_attr(crown, crown::unrooted_must_root_lint::must_root)]
#[derive(JSTraceable, MallocSizeOf)]
struct DynamicModule {
    #[ignore_malloc_size_of = "Rc is hard"]
    promise: Rc<Promise>,
    #[ignore_malloc_size_of = "GC types are hard"]
    specifier: Heap<*mut JSObject>,
    #[ignore_malloc_size_of = "GC types are hard"]
    referencing_private: Heap<JSVal>,
    #[ignore_malloc_size_of = "Defined in uuid"]
    id: DynamicModuleId,
}

/// <https://html.spec.whatwg.org/multipage/#fetch-a-single-module-script>
#[allow(clippy::too_many_arguments)]
fn fetch_single_module_script(
    owner: ModuleOwner,
    url: ServoUrl,
    visited_urls: HashSet<ServoUrl>,
    destination: Destination,
    options: ScriptFetchOptions,
    parent_identity: Option<ModuleIdentity>,
    top_level_module_fetch: bool,
    dynamic_module: Option<RootedTraceableBox<DynamicModule>>,
    can_gc: CanGc,
) {
    {
        // Step 1.
        let global = owner.global();
        let module_map = global.get_module_map().borrow();

        debug!("Start to fetch {}", url);

        if let Some(module_tree) = module_map.get(&url.clone()) {
            let status = module_tree.get_status();

            debug!("Meet a fetched url {} and its status is {:?}", url, status);

            match dynamic_module {
                Some(module) => module_tree.append_dynamic_module_handler(
                    owner.clone(),
                    ModuleIdentity::ModuleUrl(url.clone()),
                    module,
                    can_gc,
                ),
                None if top_level_module_fetch => module_tree.append_handler(
                    owner.clone(),
                    ModuleIdentity::ModuleUrl(url.clone()),
                    options,
                    can_gc,
                ),
                // do nothing if it's neither a dynamic module nor a top level module
                None => {},
            }

            if let Some(parent_identity) = parent_identity {
                module_tree.insert_parent_identity(parent_identity);
            }

            match status {
                ModuleStatus::Initial => unreachable!(
                    "We have the module in module map so its status should not be `initial`"
                ),
                // Step 2.
                ModuleStatus::Fetching => {},
                // Step 3.
                ModuleStatus::FetchingDescendants | ModuleStatus::Finished => {
                    module_tree.advance_finished_and_link(&global, can_gc);
                },
            }

            return;
        }
    }

    let global = owner.global();
    let is_external = true;
    let module_tree = ModuleTree::new(url.clone(), is_external, visited_urls);
    module_tree.set_status(ModuleStatus::Fetching);

    match dynamic_module {
        Some(module) => module_tree.append_dynamic_module_handler(
            owner.clone(),
            ModuleIdentity::ModuleUrl(url.clone()),
            module,
            can_gc,
        ),
        None if top_level_module_fetch => module_tree.append_handler(
            owner.clone(),
            ModuleIdentity::ModuleUrl(url.clone()),
            options.clone(),
            can_gc,
        ),
        // do nothing if it's neither a dynamic module nor a top level module
        None => {},
    }

    if let Some(parent_identity) = parent_identity {
        module_tree.insert_parent_identity(parent_identity);
    }

    module_tree.insert_incomplete_fetch_url(&url);

    // Step 4.
    global.set_module_map(url.clone(), module_tree);

    // Step 5-6.
    let mode = match destination {
        Destination::Worker | Destination::SharedWorker if top_level_module_fetch => {
            RequestMode::SameOrigin
        },
        _ => RequestMode::CorsMode,
    };

    let document: Option<DomRoot<Document>> = match &owner {
        ModuleOwner::Worker(_) | ModuleOwner::DynamicModule(_) => None,
        ModuleOwner::Window(script) => Some(script.root().owner_document()),
    };
    let webview_id = document.as_ref().map(|document| document.webview_id());

    // Step 7-8.
    let request = RequestBuilder::new(webview_id, url.clone(), global.get_referrer())
        .destination(destination)
        .origin(global.origin().immutable().clone())
        .parser_metadata(options.parser_metadata)
        .integrity_metadata(options.integrity_metadata.clone())
        .credentials_mode(options.credentials_mode)
        .referrer_policy(options.referrer_policy)
        .mode(mode);

    let context = Arc::new(Mutex::new(ModuleContext {
        owner,
        data: vec![],
        metadata: None,
        url: url.clone(),
        destination,
        options,
        status: Ok(()),
        resource_timing: ResourceFetchTiming::new(ResourceTimingType::Resource),
    }));

    let network_listener = NetworkListener {
        context,
        task_source: global.task_manager().networking_task_source().to_sendable(),
    };
    match document {
        Some(document) => {
            let request = document.prepare_request(request);
            document.loader_mut().fetch_async_with_callback(
                LoadType::Script(url),
                request,
                network_listener.into_callback(),
            );
        },
        None => global.fetch_with_network_listener(request, network_listener),
    }
}

#[allow(unsafe_code)]
/// <https://html.spec.whatwg.org/multipage/#fetch-an-inline-module-script-graph>
pub(crate) fn fetch_inline_module_script(
    owner: ModuleOwner,
    module_script_text: Rc<DOMString>,
    url: ServoUrl,
    script_id: ScriptId,
    options: ScriptFetchOptions,
    can_gc: CanGc,
) {
    let global = owner.global();
    let is_external = false;
    let module_tree = ModuleTree::new(url.clone(), is_external, HashSet::new());

    let cx = GlobalScope::get_cx();
    rooted!(in(*cx) let mut compiled_module: *mut JSObject = ptr::null_mut());
    let compiled_module_result = module_tree.compile_module_script(
        &global,
        owner.clone(),
        module_script_text,
        &url,
        options.clone(),
        compiled_module.handle_mut(),
        true,
        can_gc,
    );

    match compiled_module_result {
        Ok(_) => {
            module_tree.append_handler(
                owner.clone(),
                ModuleIdentity::ScriptId(script_id),
                options.clone(),
                can_gc,
            );
            module_tree.set_record(ModuleObject::new(compiled_module.handle()));

            // We need to set `module_tree` into inline module map in case
            // of that the module descendants finished right after the
            // fetch module descendants step.
            global.set_inline_module_map(script_id, module_tree);

            // Due to needed to set `module_tree` to inline module_map first,
            // we will need to retrieve it again so that we can do the fetch
            // module descendants step.
            let inline_module_map = global.get_inline_module_map().borrow();
            let module_tree = inline_module_map.get(&script_id).unwrap().clone();

            module_tree.fetch_module_descendants(
                &owner,
                Destination::Script,
                &options,
                ModuleIdentity::ScriptId(script_id),
                can_gc,
            );
        },
        Err(exception) => {
            module_tree.set_rethrow_error(exception);
            module_tree.set_status(ModuleStatus::Finished);
            global.set_inline_module_map(script_id, module_tree);
            owner.notify_owner_to_finish(ModuleIdentity::ScriptId(script_id), options, can_gc);
        },
    }
}

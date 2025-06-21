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

use content_security_policy as csp;
use encoding_rs::UTF_8;
use headers::{HeaderMapExt, ReferrerPolicy as ReferrerPolicyHeader};
use html5ever::local_name;
use hyper_serde::Serde;
use indexmap::{IndexMap, IndexSet};
use js::jsapi::{
    CompileModule1, ExceptionStackBehavior, FinishDynamicModuleImport, GetModuleRequestSpecifier,
    GetModuleResolveHook, GetRequestedModuleSpecifier, GetRequestedModulesCount,
    Handle as RawHandle, HandleObject, HandleValue as RawHandleValue, Heap,
    JS_ClearPendingException, JS_DefineProperty4, JS_IsExceptionPending, JS_NewStringCopyN,
    JSAutoRealm, JSContext, JSObject, JSPROP_ENUMERATE, JSRuntime, ModuleErrorBehaviour,
    ModuleEvaluate, ModuleLink, MutableHandleValue, SetModuleDynamicImportHook,
    SetModuleMetadataHook, SetModulePrivate, SetModuleResolveHook, SetScriptPrivateReferenceHooks,
    ThrowOnModuleEvaluationFailure, Value,
};
use js::jsval::{JSVal, PrivateValue, UndefinedValue};
use js::rust::wrappers::{JS_GetModulePrivate, JS_GetPendingException, JS_SetPendingException};
use js::rust::{
    CompileOptionsWrapper, Handle, HandleObject as RustHandleObject, HandleValue, IntoHandle,
    MutableHandleObject as RustMutableHandleObject, transform_str_to_source_text,
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
use script_bindings::error::Fallible;
use serde_json::{Map as JsonMap, Value as JsonValue};
use servo_url::ServoUrl;
use uuid::Uuid;

use crate::document_loader::LoadType;
use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::WindowBinding::Window_Binding::WindowMethods;
use crate::dom::bindings::conversions::jsstring_to_str;
use crate::dom::bindings::error::{
    Error, ErrorToJsval, report_pending_exception, throw_dom_exception,
};
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
    HTMLScriptElement, SCRIPT_JS_MIMES, ScriptId, ScriptOrigin, ScriptType,
};
use crate::dom::node::NodeTraits;
use crate::dom::performanceresourcetiming::InitiatorType;
use crate::dom::promise::Promise;
use crate::dom::promisenativehandler::{Callback, PromiseNativeHandler};
use crate::dom::types::Console;
use crate::dom::window::Window;
use crate::dom::worker::TrustedWorkerAddress;
use crate::network_listener::{self, NetworkListener, PreInvoke, ResourceTimingListener};
use crate::realms::{AlreadyInRealm, InRealm, enter_realm};
use crate::script_runtime::{CanGc, JSContext as SafeJSContext};
use crate::task::TaskBox;

fn gen_type_error(global: &GlobalScope, string: String, can_gc: CanGc) -> RethrowError {
    rooted!(in(*GlobalScope::get_cx()) let mut thrown = UndefinedValue());
    Error::Type(string).to_jsval(GlobalScope::get_cx(), global, thrown.handle_mut(), can_gc);

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
        can_gc: CanGc,
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
                assert!(JS_GetPendingException(*cx, exception.handle_mut()));
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
                can_gc,
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
                assert!(JS_GetPendingException(*cx, exception.handle_mut()));
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
        can_gc: CanGc,
    ) -> Result<IndexSet<ServoUrl>, RethrowError> {
        let cx = GlobalScope::get_cx();
        let _ac = JSAutoRealm::new(*cx, *global.reflector().get_jsobject());

        let mut specifier_urls = IndexSet::new();

        unsafe {
            let length = GetRequestedModulesCount(*cx, module_object);

            for index in 0..length {
                let specifier = jsstring_to_str(
                    *cx,
                    ptr::NonNull::new(GetRequestedModuleSpecifier(*cx, module_object, index))
                        .unwrap(),
                );

                rooted!(in(*cx) let mut private = UndefinedValue());
                JS_GetModulePrivate(module_object.get(), private.handle_mut());
                let private = private.handle().into_handle();
                let script = module_script_from_reference_private(&private);
                let url = ModuleTree::resolve_module_specifier(global, script, specifier, can_gc);

                if url.is_err() {
                    let specifier_error =
                        gen_type_error(global, "Wrong module specifier".to_owned(), can_gc);

                    return Err(specifier_error);
                }

                specifier_urls.insert(url.unwrap());
            }
        }

        Ok(specifier_urls)
    }

    /// <https://html.spec.whatwg.org/multipage/#resolve-a-module-specifier>
    #[allow(unsafe_code)]
    fn resolve_module_specifier(
        global: &GlobalScope,
        script: Option<&ModuleScript>,
        specifier: DOMString,
        can_gc: CanGc,
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

        // Step 6. Let serializedBaseURL be baseURL, serialized.
        let serialized_base_url = base_url.as_str();
        // Step 7. Let asURL be the result of resolving a URL-like module specifier given specifier and baseURL.
        let as_url = Self::resolve_url_like_module_specifier(&specifier, base_url);
        // Step 8. Let normalizedSpecifier be the serialization of asURL, if asURL is non-null;
        // otherwise, specifier.
        let normalized_specifier = match &as_url {
            Some(url) => url.as_str(),
            None => &specifier,
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
                    // Step 10.1.2 If scopeImportsMatch is not null, then set result to scopeImportsMatch,
                    // and break.
                    result = resolve_imports_match(
                        normalized_specifier,
                        as_url.as_ref(),
                        imports,
                        can_gc,
                    )?;
                    break;
                }
            }

            // Step 11. If result is null, set result to the result of resolving an imports match given
            // normalizedSpecifier, asURL, and importMap's imports.
            if result.is_none() {
                result = resolve_imports_match(
                    normalized_specifier,
                    as_url.as_ref(),
                    &map.imports,
                    can_gc,
                )?;
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
                "Specifier was a bare specifier, but was not remapped to anything by importMap."
                    .to_owned(),
            )),
        }
    }

    /// <https://html.spec.whatwg.org/multipage/#resolving-a-url-like-module-specifier>
    fn resolve_url_like_module_specifier(
        specifier: &DOMString,
        base_url: &ServoUrl,
    ) -> Option<ServoUrl> {
        // Step 1. If specifier starts with "/", "./", or "../", then:
        if specifier.starts_with('/') || specifier.starts_with("./") || specifier.starts_with("../")
        {
            // Step 1.1. Let url be the result of URL parsing specifier with baseURL.
            return ServoUrl::parse_with_base(Some(base_url), specifier).ok();
        }
        // Step 2. Let url be the result of URL parsing specifier (with no base URL).
        ServoUrl::parse(specifier).ok()
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
                Some(raw_record) => {
                    self.resolve_requested_module_specifiers(&global, raw_record.handle(), can_gc)
                },
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
                                Err(Error::NotFound),
                            )),
                        },
                    }
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
                let err = gen_type_error(&global, "Dynamic import failed".to_owned(), can_gc);
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

    fn process_csp_violations(&mut self, _request_id: RequestId, violations: Vec<csp::Violation>) {
        let global = &self.resource_timing_global();
        global.report_csp_violations(violations, None);
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
/// <https://tc39.es/ecma262/#sec-hostimportmoduledynamically>
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
    let promise = Promise::new_with_js_promise(Handle::from_raw(promise), cx);

    //Step 5 & 6.
    if let Err(e) = fetch_an_import_module_script_graph(
        &global_scope,
        specifier,
        reference_private,
        promise,
        CanGc::note(),
    ) {
        JS_SetPendingException(*cx, e.handle(), ExceptionStackBehavior::Capture);
        return false;
    }

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
    promise: Rc<Promise>,
    can_gc: CanGc,
) -> Result<(), RethrowError> {
    // Step 1.
    let cx = GlobalScope::get_cx();
    let specifier = unsafe {
        jsstring_to_str(
            *cx,
            ptr::NonNull::new(GetModuleRequestSpecifier(*cx, module_request)).unwrap(),
        )
    };
    let mut options = ScriptFetchOptions::default_classic_script(global);
    let module_data = unsafe { module_script_from_reference_private(&reference_private) };
    if let Some(data) = module_data {
        options = data.options.descendant_fetch_options();
    }
    let url = ModuleTree::resolve_module_specifier(global, module_data, specifier, can_gc);

    // Step 2.
    if url.is_err() {
        let specifier_error = gen_type_error(global, "Wrong module specifier".to_owned(), can_gc);
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
/// <https://tc39.es/ecma262/#sec-HostLoadImportedModule>
/// <https://html.spec.whatwg.org/multipage/#hostloadimportedmodule>
unsafe extern "C" fn HostResolveImportedModule(
    cx: *mut JSContext,
    reference_private: RawHandleValue,
    specifier: RawHandle<*mut JSObject>,
) -> *mut JSObject {
    let in_realm_proof = AlreadyInRealm::assert_for_cx(SafeJSContext::from_ptr(cx));
    let global_scope = GlobalScope::from_context(cx, InRealm::Already(&in_realm_proof));

    // Step 5.
    let module_data = module_script_from_reference_private(&reference_private);
    let specifier = jsstring_to_str(
        cx,
        ptr::NonNull::new(GetModuleRequestSpecifier(cx, specifier)).unwrap(),
    );
    let url =
        ModuleTree::resolve_module_specifier(&global_scope, module_data, specifier, CanGc::note());

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
        .mode(mode)
        .insecure_requests_policy(global.insecure_requests_policy())
        .has_trustworthy_ancestor_origin(global.has_trustworthy_ancestor_origin())
        .policy_container(global.policy_container().to_owned())
        .cryptographic_nonce_metadata(options.cryptographic_nonce.clone());

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
                                    .map(|u| u.is_special_scheme())
                                    .unwrap_or_default()))
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
            if specifier.starts_with(&record.specifier) {
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
    can_gc: CanGc,
) -> Fallible<ImportMap> {
    // Step 1. Let parsed be the result of parsing a JSON string to an Infra value given input.
    let parsed: JsonValue = serde_json::from_str(input.str())
        .map_err(|_| Error::Type("The value needs to be a JSON object.".to_owned()))?;
    // Step 2. If parsed is not an ordered map, then throw a TypeError indicating that the
    // top-level value needs to be a JSON object.
    let JsonValue::Object(mut parsed) = parsed else {
        return Err(Error::Type(
            "The top-level value needs to be a JSON object.".to_owned(),
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
                "The \"imports\" top-level value needs to be a JSON object.".to_owned(),
            ));
        };
        // Step 4.2 Set sortedAndNormalizedImports to the result of sorting and
        // normalizing a module specifier map given parsed["imports"] and baseURL.
        sorted_and_normalized_imports = sort_and_normalize_module_specifier_map(
            &module_owner.global(),
            imports,
            &base_url,
            can_gc,
        );
    }

    // Step 5. Let sortedAndNormalizedScopes be an empty ordered map.
    let mut sorted_and_normalized_scopes: IndexMap<ServoUrl, ModuleSpecifierMap> = IndexMap::new();
    // Step 6. If parsed["scopes"] exists, then:
    if let Some(scopes) = parsed.get("scopes") {
        // Step 6.1 If parsed["scopes"] is not an ordered map, then throw a TypeError
        // indicating that the value for the "scopes" top-level key needs to be a JSON object.
        let JsonValue::Object(scopes) = scopes else {
            return Err(Error::Type(
                "The \"scopes\" top-level value needs to be a JSON object.".to_owned(),
            ));
        };
        // Step 6.2 Set sortedAndNormalizedScopes to the result of sorting and
        // normalizing scopes given parsed["scopes"] and baseURL.
        sorted_and_normalized_scopes =
            sort_and_normalize_scopes(&module_owner.global(), scopes, &base_url, can_gc)?;
    }

    // Step 7. Let normalizedIntegrity be an empty ordered map.
    let mut normalized_integrity = ModuleIntegrityMap::new();
    // Step 8. If parsed["integrity"] exists, then:
    if let Some(integrity) = parsed.get("integrity") {
        // Step 8.1 If parsed["integrity"] is not an ordered map, then throw a TypeError
        // indicating that the value for the "integrity" top-level key needs to be a JSON object.
        let JsonValue::Object(integrity) = integrity else {
            return Err(Error::Type(
                "The \"integrity\" top-level value needs to be a JSON object.".to_owned(),
            ));
        };
        // Step 8.2 Set normalizedIntegrity to the result of normalizing
        // a module integrity map given parsed["integrity"] and baseURL.
        normalized_integrity =
            normalize_module_integrity_map(&module_owner.global(), integrity, &base_url, can_gc);
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
#[allow(unsafe_code)]
fn sort_and_normalize_module_specifier_map(
    global: &GlobalScope,
    original_map: &JsonMap<String, JsonValue>,
    base_url: &ServoUrl,
    can_gc: CanGc,
) -> ModuleSpecifierMap {
    // Step 1. Let normalized be an empty ordered map.
    let mut normalized = ModuleSpecifierMap::new();

    // Step 2. For each specifier_key -> value in originalMap
    for (specifier_key, value) in original_map {
        // Step 2.1 Let normalized_specifier_key be the result of
        // normalizing a specifier key given specifier_key and base_url.
        let Some(normalized_specifier_key) =
            normalize_specifier_key(global, specifier_key, base_url, can_gc)
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
        let value = DOMString::from(value.as_str());
        let Some(address_url) = ModuleTree::resolve_url_like_module_specifier(&value, base_url)
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
    can_gc: CanGc,
) -> Fallible<IndexMap<ServoUrl, ModuleSpecifierMap>> {
    // Step 1. Let normalized be an empty ordered map.
    let mut normalized: IndexMap<ServoUrl, ModuleSpecifierMap> = IndexMap::new();

    // Step 2. For each scopePrefix → potentialSpecifierMap of originalMap:
    for (scope_prefix, potential_specifier_map) in original_map {
        // Step 2.1 If potentialSpecifierMap is not an ordered map, then throw a TypeError indicating
        // that the value of the scope with prefix scopePrefix needs to be a JSON object.
        let JsonValue::Object(potential_specifier_map) = potential_specifier_map else {
            return Err(Error::Type(
                "The value of the scope with prefix scopePrefix needs to be a JSON object."
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
        let normalized_specifier_map = sort_and_normalize_module_specifier_map(
            global,
            potential_specifier_map,
            base_url,
            can_gc,
        );
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
    _can_gc: CanGc,
) -> ModuleIntegrityMap {
    // Step 1. Let normalized be an empty ordered map.
    let mut normalized = ModuleIntegrityMap::new();

    // Step 2. For each key → value of originalMap:
    for (key, value) in original_map {
        // Step 2.1 Let resolvedURL be the result of
        // resolving a URL-like module specifier given key and baseURL.
        let Some(resolved_url) =
            ModuleTree::resolve_url_like_module_specifier(&DOMString::from(key.as_str()), base_url)
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
    _can_gc: CanGc,
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
    let url =
        ModuleTree::resolve_url_like_module_specifier(&DOMString::from(specifier_key), base_url);

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
pub(crate) fn resolve_imports_match(
    normalized_specifier: &str,
    as_url: Option<&ServoUrl>,
    specifier_map: &ModuleSpecifierMap,
    _can_gc: CanGc,
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
                    "Resolution of specifierKey was blocked by a null entry.".to_owned(),
                ));
            }
        }

        // Step 1.2 If all of the following are true:
        // - specifierKey ends with U+002F (/)
        // - specifierKey is a code unit prefix of normalizedSpecifier
        // - either asURL is null, or asURL is special, then:
        if specifier_key.ends_with('\u{002f}') &&
            normalized_specifier.starts_with(specifier_key) &&
            (as_url.is_none() || as_url.map(|u| u.is_special_scheme()).unwrap_or_default())
        {
            // Step 1.2.1 If resolutionResult is null, then throw a TypeError.
            // Step 1.2.2 Assert: resolutionResult is a URL.
            let Some(resolution_result) = resolution_result else {
                return Err(Error::Type(
                    "Resolution of specifierKey was blocked by a null entry.".to_owned(),
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
                    "Resolution of normalizedSpecifier was blocked since
                    the afterPrefix portion could not be URL-parsed relative to
                    the resolutionResult mapped to by the specifierKey prefix."
                        .to_owned(),
                ));
            };

            // Step 1.2.8 If the serialization of resolutionResult is not
            // a code unit prefix of the serialization of url, then throw a TypeError
            if !url.as_str().starts_with(resolution_result.as_str()) {
                return Err(Error::Type(
                    "Resolution of normalizedSpecifier was blocked due to
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

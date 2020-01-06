/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! The script module mod contains common traits and structs
//! related to `type=module` for script thread or worker threads.

use crate::compartments::{enter_realm, AlreadyInCompartment, InCompartment};
use crate::document_loader::LoadType;
use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::WindowBinding::WindowBinding::WindowMethods;
use crate::dom::bindings::conversions::jsstring_to_str;
use crate::dom::bindings::error::report_pending_exception;
use crate::dom::bindings::error::Error;
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::refcounted::{Trusted, TrustedPromise};
use crate::dom::bindings::reflector::DomObject;
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::settings_stack::AutoIncumbentScript;
use crate::dom::bindings::str::DOMString;
use crate::dom::bindings::trace::RootedTraceableBox;
use crate::dom::document::Document;
use crate::dom::element::Element;
use crate::dom::globalscope::GlobalScope;
use crate::dom::htmlscriptelement::{HTMLScriptElement, ScriptId};
use crate::dom::htmlscriptelement::{ScriptOrigin, ScriptType, SCRIPT_JS_MIMES};
use crate::dom::node::document_from_node;
use crate::dom::performanceresourcetiming::InitiatorType;
use crate::dom::promise::Promise;
use crate::dom::promisenativehandler::{Callback, PromiseNativeHandler};
use crate::dom::window::Window;
use crate::dom::worker::TrustedWorkerAddress;
use crate::network_listener::{self, NetworkListener};
use crate::network_listener::{PreInvoke, ResourceTimingListener};
use crate::task::TaskBox;
use crate::task_source::TaskSourceName;
use encoding_rs::UTF_8;
use hyper_serde::Serde;
use ipc_channel::ipc;
use ipc_channel::router::ROUTER;
use js::glue::{AppendToAutoObjectVector, CreateAutoObjectVector};
use js::jsapi::Handle as RawHandle;
use js::jsapi::HandleObject;
use js::jsapi::HandleValue as RawHandleValue;
use js::jsapi::{AutoObjectVector, JSAutoRealm, JSObject, JSString};
use js::jsapi::{GetModuleResolveHook, JSRuntime, SetModuleResolveHook};
use js::jsapi::{GetRequestedModules, SetModuleMetadataHook};
use js::jsapi::{GetWaitForAllPromise, ModuleEvaluate, ModuleInstantiate, SourceText};
use js::jsapi::{Heap, JSContext, JS_ClearPendingException, SetModulePrivate};
use js::jsapi::{SetModuleDynamicImportHook, SetScriptPrivateReferenceHooks};
use js::jsval::{JSVal, PrivateValue, UndefinedValue};
use js::rust::jsapi_wrapped::{CompileModule, JS_GetArrayLength, JS_GetElement};
use js::rust::jsapi_wrapped::{GetRequestedModuleSpecifier, JS_GetPendingException};
use js::rust::wrappers::JS_SetPendingException;
use js::rust::CompileOptionsWrapper;
use js::rust::IntoHandle;
use js::rust::{Handle, HandleValue};
use net_traits::request::{CredentialsMode, Destination, ParserMetadata};
use net_traits::request::{Referrer, RequestBuilder, RequestMode};
use net_traits::{FetchMetadata, Metadata};
use net_traits::{FetchResponseListener, NetworkError};
use net_traits::{ResourceFetchTiming, ResourceTimingType};
use servo_url::ServoUrl;
use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};
use std::ffi;
use std::marker::PhantomData;
use std::ptr;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use url::ParseError as UrlParseError;

use indexmap::IndexSet;

pub fn get_source_text(source: &[u16]) -> SourceText<u16> {
    SourceText {
        units_: source.as_ptr() as *const _,
        length_: source.len() as u32,
        ownsUnits_: false,
        _phantom_0: PhantomData,
    }
}

#[allow(unsafe_code)]
unsafe fn gen_type_error(global: &GlobalScope, string: String) -> ModuleError {
    rooted!(in(*global.get_cx()) let mut thrown = UndefinedValue());
    Error::Type(string).to_jsval(*global.get_cx(), &global, thrown.handle_mut());

    return ModuleError::RawException(RootedTraceableBox::from_box(Heap::boxed(thrown.get())));
}

#[derive(JSTraceable)]
pub struct ModuleObject(Box<Heap<*mut JSObject>>);

impl ModuleObject {
    #[allow(unsafe_code)]
    pub fn handle(&self) -> HandleObject {
        unsafe { self.0.handle() }
    }
}

#[derive(JSTraceable)]
pub enum ModuleError {
    Network(NetworkError),
    RawException(RootedTraceableBox<Heap<JSVal>>),
}

impl Eq for ModuleError {}

impl PartialEq for ModuleError {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Network(_), Self::RawException(_)) |
            (Self::RawException(_), Self::Network(_)) => false,
            _ => true,
        }
    }
}

impl Ord for ModuleError {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self, other) {
            (Self::Network(_), Self::RawException(_)) => Ordering::Greater,
            (Self::RawException(_), Self::Network(_)) => Ordering::Less,
            _ => Ordering::Equal,
        }
    }
}

impl PartialOrd for ModuleError {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl ModuleError {
    #[allow(unsafe_code)]
    pub fn handle(&self) -> Handle<JSVal> {
        match self {
            Self::Network(_) => unreachable!(),
            Self::RawException(exception) => exception.handle(),
        }
    }
}

impl Clone for ModuleError {
    fn clone(&self) -> Self {
        match self {
            Self::Network(network_error) => Self::Network(network_error.clone()),
            Self::RawException(exception) => Self::RawException(RootedTraceableBox::from_box(
                Heap::boxed(exception.get().clone()),
            )),
        }
    }
}

struct ModuleScript {
    base_url: ServoUrl,
}

#[derive(JSTraceable)]
pub struct ModuleTree {
    url: ServoUrl,
    text: DomRefCell<DOMString>,
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
    parent_urls: DomRefCell<IndexSet<ServoUrl>>,
    descendant_urls: DomRefCell<IndexSet<ServoUrl>>,
    visited_urls: DomRefCell<HashSet<ServoUrl>>,
    error: DomRefCell<Option<ModuleError>>,
    promise: DomRefCell<Option<Rc<Promise>>>,
}

impl ModuleTree {
    pub fn new(url: ServoUrl) -> Self {
        ModuleTree {
            url,
            text: DomRefCell::new(DOMString::new()),
            record: DomRefCell::new(None),
            status: DomRefCell::new(ModuleStatus::Initial),
            parent_urls: DomRefCell::new(IndexSet::new()),
            descendant_urls: DomRefCell::new(IndexSet::new()),
            visited_urls: DomRefCell::new(HashSet::new()),
            error: DomRefCell::new(None),
            promise: DomRefCell::new(None),
        }
    }

    pub fn get_promise(&self) -> &DomRefCell<Option<Rc<Promise>>> {
        &self.promise
    }

    pub fn set_promise(&self, promise: Rc<Promise>) {
        *self.promise.borrow_mut() = Some(promise);
    }

    pub fn get_status(&self) -> ModuleStatus {
        self.status.borrow().clone()
    }

    pub fn set_status(&self, status: ModuleStatus) {
        *self.status.borrow_mut() = status;
    }

    pub fn get_record(&self) -> &DomRefCell<Option<ModuleObject>> {
        &self.record
    }

    pub fn set_record(&self, record: ModuleObject) {
        *self.record.borrow_mut() = Some(record);
    }

    pub fn get_error(&self) -> &DomRefCell<Option<ModuleError>> {
        &self.error
    }

    pub fn set_error(&self, error: Option<ModuleError>) {
        *self.error.borrow_mut() = error;
    }

    pub fn get_text(&self) -> &DomRefCell<DOMString> {
        &self.text
    }

    pub fn set_text(&self, module_text: DOMString) {
        *self.text.borrow_mut() = module_text;
    }

    pub fn get_parent_urls(&self) -> &DomRefCell<IndexSet<ServoUrl>> {
        &self.parent_urls
    }

    pub fn insert_parent_url(&self, parent_url: ServoUrl) {
        self.parent_urls.borrow_mut().insert(parent_url);
    }

    pub fn append_parent_urls(&self, parent_urls: IndexSet<ServoUrl>) {
        self.parent_urls.borrow_mut().extend(parent_urls);
    }

    pub fn get_descendant_urls(&self) -> &DomRefCell<IndexSet<ServoUrl>> {
        &self.descendant_urls
    }

    pub fn append_descendant_urls(&self, descendant_urls: IndexSet<ServoUrl>) {
        self.descendant_urls.borrow_mut().extend(descendant_urls);
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

        for descendant_module in descendant_urls
            .iter()
            .filter_map(|url| module_map.get(&url.clone()))
        {
            if discovered_urls.contains(&descendant_module.url) {
                continue;
            }

            let descendant_status = descendant_module.get_status();
            if descendant_status < ModuleStatus::FetchingDescendants {
                return false;
            }

            let all_ready_descendants = ModuleTree::recursive_check_descendants(
                &descendant_module,
                module_map,
                discovered_urls,
            );

            if !all_ready_descendants {
                return false;
            }
        }

        return true;
    }

    fn has_all_ready_descendants(&self, module_map: &HashMap<ServoUrl, Rc<ModuleTree>>) -> bool {
        let mut discovered_urls = HashSet::new();

        return ModuleTree::recursive_check_descendants(&self, module_map, &mut discovered_urls);
    }

    pub fn get_visited_urls(&self) -> &DomRefCell<HashSet<ServoUrl>> {
        &self.visited_urls
    }

    pub fn append_handler(&self, owner: ModuleOwner, module_url: ServoUrl, is_top_level: bool) {
        let promise = self.promise.borrow();

        let resolve_this = owner.clone();
        let reject_this = owner.clone();

        let resolved_url = module_url.clone();
        let rejected_url = module_url.clone();

        let handler = PromiseNativeHandler::new(
            &owner.global(),
            Some(ModuleHandler::new(Box::new(
                task!(fetched_resolve: move || {
                    resolve_this.finish_module_load(Some(resolved_url), is_top_level);
                }),
            ))),
            Some(ModuleHandler::new(Box::new(
                task!(failure_reject: move || {
                    reject_this.finish_module_load(Some(rejected_url), is_top_level);
                }),
            ))),
        );

        let _compartment = enter_realm(&*owner.global());
        AlreadyInCompartment::assert(&*owner.global());
        let _ais = AutoIncumbentScript::new(&*owner.global());

        let promise = promise.as_ref().unwrap();

        promise.append_native_handler(&handler);
    }
}

#[derive(Clone, Copy, Debug, JSTraceable, PartialEq, PartialOrd)]
pub enum ModuleStatus {
    Initial,
    Fetching,
    FetchingDescendants,
    FetchFailed,
    Ready,
    Finished,
}

impl ModuleTree {
    #[allow(unsafe_code)]
    /// https://html.spec.whatwg.org/multipage/#creating-a-module-script
    /// Step 7-11.
    fn compile_module_script(
        &self,
        global: &GlobalScope,
        module_script_text: DOMString,
        url: ServoUrl,
    ) -> Result<ModuleObject, ModuleError> {
        let module: Vec<u16> = module_script_text.encode_utf16().collect();

        let url_cstr = ffi::CString::new(url.as_str().as_bytes()).unwrap();

        let _ac = JSAutoRealm::new(*global.get_cx(), *global.reflector().get_jsobject());

        let compile_options = CompileOptionsWrapper::new(*global.get_cx(), url_cstr.as_ptr(), 1);

        rooted!(in(*global.get_cx()) let mut module_script = ptr::null_mut::<JSObject>());

        let mut source = get_source_text(&module);

        unsafe {
            if !CompileModule(
                *global.get_cx(),
                compile_options.ptr,
                &mut source,
                &mut module_script.handle_mut(),
            ) {
                warn!("fail to compile module script of {}", url);

                rooted!(in(*global.get_cx()) let mut exception = UndefinedValue());
                assert!(JS_GetPendingException(
                    *global.get_cx(),
                    &mut exception.handle_mut()
                ));
                JS_ClearPendingException(*global.get_cx());

                return Err(ModuleError::RawException(RootedTraceableBox::from_box(
                    Heap::boxed(exception.get()),
                )));
            }

            let module_script_data = Box::new(ModuleScript {
                base_url: url.clone(),
            });

            SetModulePrivate(
                module_script.get(),
                &PrivateValue(Box::into_raw(module_script_data) as *const _),
            );
        }

        debug!("module script of {} compile done", url);

        self.resolve_requested_module_specifiers(
            &global,
            module_script.handle().into_handle(),
            url.clone(),
        )
        .map(|_| ModuleObject(Heap::boxed(*module_script)))
    }

    #[allow(unsafe_code)]
    pub fn instantiate_module_tree(
        &self,
        global: &GlobalScope,
        module_record: HandleObject,
    ) -> Result<(), ModuleError> {
        let _ac = JSAutoRealm::new(*global.get_cx(), *global.reflector().get_jsobject());

        unsafe {
            if !ModuleInstantiate(*global.get_cx(), module_record) {
                warn!("fail to instantiate module");

                rooted!(in(*global.get_cx()) let mut exception = UndefinedValue());
                assert!(JS_GetPendingException(
                    *global.get_cx(),
                    &mut exception.handle_mut()
                ));
                JS_ClearPendingException(*global.get_cx());

                Err(ModuleError::RawException(RootedTraceableBox::from_box(
                    Heap::boxed(exception.get()),
                )))
            } else {
                debug!("module instantiated successfully");

                Ok(())
            }
        }
    }

    #[allow(unsafe_code)]
    pub fn execute_module(
        &self,
        global: &GlobalScope,
        module_record: HandleObject,
    ) -> Result<(), ModuleError> {
        let _ac = JSAutoRealm::new(*global.get_cx(), *global.reflector().get_jsobject());

        unsafe {
            if !ModuleEvaluate(*global.get_cx(), module_record) {
                warn!("fail to evaluate module");

                rooted!(in(*global.get_cx()) let mut exception = UndefinedValue());
                assert!(JS_GetPendingException(
                    *global.get_cx(),
                    &mut exception.handle_mut()
                ));
                JS_ClearPendingException(*global.get_cx());

                Err(ModuleError::RawException(RootedTraceableBox::from_box(
                    Heap::boxed(exception.get()),
                )))
            } else {
                debug!("module evaluated successfully");
                Ok(())
            }
        }
    }

    #[allow(unsafe_code)]
    pub fn report_error(&self, global: &GlobalScope) {
        let module_error = self.error.borrow();

        if let Some(exception) = &*module_error {
            unsafe {
                JS_SetPendingException(*global.get_cx(), exception.handle());
                report_pending_exception(*global.get_cx(), true);
            }
        }
    }

    /// https://html.spec.whatwg.org/multipage/#fetch-the-descendants-of-a-module-script
    /// Step 5.
    pub fn resolve_requested_modules(
        &self,
        global: &GlobalScope,
    ) -> Result<IndexSet<ServoUrl>, ModuleError> {
        let status = self.get_status();

        assert_ne!(status, ModuleStatus::Initial);
        assert_ne!(status, ModuleStatus::Fetching);

        let record = self.record.borrow();

        if let Some(raw_record) = &*record {
            let valid_specifier_urls = self.resolve_requested_module_specifiers(
                &global,
                raw_record.handle(),
                self.url.clone(),
            );

            return valid_specifier_urls.map(|parsed_urls| {
                parsed_urls
                    .iter()
                    .filter_map(|parsed_url| {
                        let mut visited = self.visited_urls.borrow_mut();

                        if !visited.contains(&parsed_url) {
                            visited.insert(parsed_url.clone());

                            Some(parsed_url.clone())
                        } else {
                            None
                        }
                    })
                    .collect::<IndexSet<ServoUrl>>()
            });
        }

        unreachable!("Didn't have record while resolving its requested module")
    }

    #[allow(unsafe_code)]
    fn resolve_requested_module_specifiers(
        &self,
        global: &GlobalScope,
        module_object: HandleObject,
        base_url: ServoUrl,
    ) -> Result<IndexSet<ServoUrl>, ModuleError> {
        let _ac = JSAutoRealm::new(*global.get_cx(), *global.reflector().get_jsobject());

        let mut specifier_urls = IndexSet::new();

        unsafe {
            rooted!(in(*global.get_cx()) let requested_modules = GetRequestedModules(*global.get_cx(), module_object));

            let mut length = 0;

            if !JS_GetArrayLength(*global.get_cx(), requested_modules.handle(), &mut length) {
                let module_length_error =
                    gen_type_error(&global, "Wrong length of requested modules".to_owned());

                return Err(module_length_error);
            }

            for index in 0..length {
                rooted!(in(*global.get_cx()) let mut element = UndefinedValue());

                if !JS_GetElement(
                    *global.get_cx(),
                    requested_modules.handle(),
                    index,
                    &mut element.handle_mut(),
                ) {
                    let get_element_error =
                        gen_type_error(&global, "Failed to get requested module".to_owned());

                    return Err(get_element_error);
                }

                rooted!(in(*global.get_cx()) let specifier = GetRequestedModuleSpecifier(
                    *global.get_cx(), element.handle()
                ));

                let url = ModuleTree::resolve_module_specifier(
                    *global.get_cx(),
                    &base_url,
                    specifier.handle().into_handle(),
                );

                if url.is_err() {
                    let specifier_error =
                        gen_type_error(&global, "Wrong module specifier".to_owned());

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
    /// https://html.spec.whatwg.org/multipage/#resolve-a-module-specifier
    #[allow(unsafe_code)]
    fn resolve_module_specifier(
        cx: *mut JSContext,
        url: &ServoUrl,
        specifier: RawHandle<*mut JSString>,
    ) -> Result<ServoUrl, UrlParseError> {
        let specifier_str = unsafe { jsstring_to_str(cx, *specifier) };

        // Step 1.
        if let Ok(specifier_url) = ServoUrl::parse(&specifier_str) {
            return Ok(specifier_url);
        }

        // Step 2.
        if !specifier_str.starts_with("/") &&
            !specifier_str.starts_with("./") &&
            !specifier_str.starts_with("../")
        {
            return Err(UrlParseError::InvalidDomainCharacter);
        }

        // Step 3.
        return ServoUrl::parse_with_base(Some(url), &specifier_str.clone());
    }

    /// https://html.spec.whatwg.org/multipage/#finding-the-first-parse-error
    fn find_first_parse_error(
        global: &GlobalScope,
        module_tree: &ModuleTree,
        discovered_urls: &mut HashSet<ServoUrl>,
    ) -> Option<ModuleError> {
        // 3.
        discovered_urls.insert(module_tree.url.clone());

        // 4.
        let module_map = global.get_module_map().borrow();
        let record = module_tree.get_record().borrow();
        if record.is_none() {
            let module_error = module_tree.get_error().borrow();

            return module_error.clone();
        }

        // 5-6.
        let descendant_urls = module_tree.get_descendant_urls().borrow();

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
            let child_parse_error =
                ModuleTree::find_first_parse_error(&global, &descendant_module, discovered_urls);

            // 8-4.
            if child_parse_error.is_some() {
                return child_parse_error;
            }
        }

        // Step 9.
        return None;
    }
}

#[derive(JSTraceable, MallocSizeOf)]
struct ModuleHandler {
    #[ignore_malloc_size_of = "Measuring trait objects is hard"]
    task: DomRefCell<Option<Box<dyn TaskBox>>>,
}

impl ModuleHandler {
    pub fn new(task: Box<dyn TaskBox>) -> Box<dyn Callback> {
        Box::new(Self {
            task: DomRefCell::new(Some(task)),
        })
    }
}

impl Callback for ModuleHandler {
    fn callback(&self, _cx: *mut JSContext, _v: HandleValue) {
        let task = self.task.borrow_mut().take().unwrap();
        task.run_box();
    }
}

/// The owner of the module
/// It can be `worker` or `script` element
#[derive(Clone)]
pub enum ModuleOwner {
    #[allow(dead_code)]
    Worker(TrustedWorkerAddress),
    Window(Trusted<HTMLScriptElement>),
}

impl ModuleOwner {
    pub fn global(&self) -> DomRoot<GlobalScope> {
        match &self {
            ModuleOwner::Worker(worker) => (*worker.root().clone()).global(),
            ModuleOwner::Window(script) => (*script.root()).global(),
        }
    }

    fn gen_promise_with_final_handler(
        &self,
        module_url: Option<ServoUrl>,
        is_top_level: bool,
    ) -> Rc<Promise> {
        let resolve_this = self.clone();
        let reject_this = self.clone();

        let resolved_url = module_url.clone();
        let rejected_url = module_url.clone();

        let handler = PromiseNativeHandler::new(
            &self.global(),
            Some(ModuleHandler::new(Box::new(
                task!(fetched_resolve: move || {
                    resolve_this.finish_module_load(resolved_url, is_top_level);
                }),
            ))),
            Some(ModuleHandler::new(Box::new(
                task!(failure_reject: move || {
                    reject_this.finish_module_load(rejected_url, is_top_level);
                }),
            ))),
        );

        let compartment = enter_realm(&*self.global());
        let comp = InCompartment::Entered(&compartment);
        let _ais = AutoIncumbentScript::new(&*self.global());

        let promise = Promise::new_in_current_compartment(&self.global(), comp);

        promise.append_native_handler(&handler);

        promise
    }

    /// https://html.spec.whatwg.org/multipage/#fetch-the-descendants-of-and-link-a-module-script
    /// step 4-7.
    pub fn finish_module_load(&self, module_url: Option<ServoUrl>, is_top_level: bool) {
        match &self {
            ModuleOwner::Worker(_) => unimplemented!(),
            ModuleOwner::Window(script) => {
                let global = self.global();

                let document = document_from_node(&*script.root());

                let module_map = global.get_module_map().borrow();

                let (module_tree, mut load) = if let Some(script_src) = module_url.clone() {
                    let module_tree = module_map.get(&script_src.clone()).unwrap().clone();

                    let load = Ok(ScriptOrigin::external(
                        module_tree.get_text().borrow().clone(),
                        script_src.clone(),
                        ScriptType::Module,
                    ));

                    debug!(
                        "Going to finish external script from {}",
                        script_src.clone()
                    );

                    (module_tree, load)
                } else {
                    let module_tree = {
                        let inline_module_map = global.get_inline_module_map().borrow();
                        inline_module_map
                            .get(&script.root().get_script_id())
                            .unwrap()
                            .clone()
                    };

                    let base_url = document.base_url();

                    let load = Ok(ScriptOrigin::internal(
                        module_tree.get_text().borrow().clone(),
                        base_url.clone(),
                        ScriptType::Module,
                    ));

                    debug!("Going to finish internal script from {}", base_url.clone());

                    (module_tree, load)
                };

                module_tree.set_status(ModuleStatus::Finished);

                if !module_tree.has_all_ready_descendants(&module_map) {
                    return;
                }

                let parent_urls = module_tree.get_parent_urls().borrow();
                let parent_all_ready = parent_urls
                    .iter()
                    .filter_map(|parent_url| module_map.get(&parent_url.clone()))
                    .all(|parent_tree| parent_tree.has_all_ready_descendants(&module_map));

                if !parent_all_ready {
                    return;
                }

                parent_urls
                    .iter()
                    .filter_map(|parent_url| module_map.get(&parent_url.clone()))
                    .for_each(|parent_tree| {
                        let parent_promise = parent_tree.get_promise().borrow();
                        if let Some(promise) = parent_promise.as_ref() {
                            promise.resolve_native(&());
                        }
                    });

                let mut discovered_urls: HashSet<ServoUrl> = HashSet::new();
                let module_error =
                    ModuleTree::find_first_parse_error(&global, &module_tree, &mut discovered_urls);

                match module_error {
                    None => {
                        let module_record = module_tree.get_record().borrow();
                        if let Some(record) = &*module_record {
                            let instantiated =
                                module_tree.instantiate_module_tree(&global, record.handle());

                            if let Err(exception) = instantiated {
                                module_tree.set_error(Some(exception.clone()));
                            }
                        }
                    },
                    Some(ModuleError::RawException(exception)) => {
                        module_tree.set_error(Some(ModuleError::RawException(exception)));
                    },
                    Some(ModuleError::Network(network_error)) => {
                        module_tree.set_error(Some(ModuleError::Network(network_error.clone())));

                        // Change the `result` load of the script into `network` error
                        load = Err(network_error);
                    },
                };

                if is_top_level {
                    let r#async = script
                        .root()
                        .upcast::<Element>()
                        .has_attribute(&local_name!("async"));

                    if !r#async && (&*script.root()).get_parser_inserted() {
                        document.deferred_script_loaded(&*script.root(), load);
                    } else if !r#async && !(&*script.root()).get_non_blocking() {
                        document.asap_in_order_script_loaded(&*script.root(), load);
                    } else {
                        document.asap_script_loaded(&*script.root(), load);
                    };
                }
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
    /// The initial URL requested.
    url: ServoUrl,
    /// Destination of current module context
    destination: Destination,
    /// Credentials Mode of current module context
    credentials_mode: CredentialsMode,
    /// Indicates whether the request failed, and why
    status: Result<(), NetworkError>,
    /// Timing object for this resource
    resource_timing: ResourceFetchTiming,
}

impl FetchResponseListener for ModuleContext {
    fn process_request_body(&mut self) {} // TODO(cybai): Perhaps add custom steps to perform fetch here?

    fn process_request_eof(&mut self) {} // TODO(cybai): Perhaps add custom steps to perform fetch here?

    fn process_response(&mut self, metadata: Result<FetchMetadata, NetworkError>) {
        self.metadata = metadata.ok().map(|meta| match meta {
            FetchMetadata::Unfiltered(m) => m,
            FetchMetadata::Filtered { unsafe_, .. } => unsafe_,
        });

        let status_code = self
            .metadata
            .as_ref()
            .and_then(|m| match m.status {
                Some((c, _)) => Some(c),
                _ => None,
            })
            .unwrap_or(0);

        self.status = match status_code {
            0 => Err(NetworkError::Internal(
                "No http status code received".to_owned(),
            )),
            200..=299 => Ok(()), // HTTP ok status codes
            _ => Err(NetworkError::Internal(format!(
                "HTTP error code {}",
                status_code
            ))),
        };
    }

    fn process_response_chunk(&mut self, mut chunk: Vec<u8>) {
        if self.status.is_ok() {
            self.data.append(&mut chunk);
        }
    }

    /// <https://html.spec.whatwg.org/multipage/#fetch-a-single-module-script>
    /// Step 9-12
    #[allow(unsafe_code)]
    fn process_response_eof(&mut self, response: Result<ResourceFetchTiming, NetworkError>) {
        let global = self.owner.global();

        if let Some(window) = global.downcast::<Window>() {
            window
                .Document()
                .finish_load(LoadType::Script(self.url.clone()));
        }

        // Step 9-1 & 9-2.
        let load = response.and(self.status.clone()).and_then(|_| {
            // Step 9-3.
            let meta = self.metadata.take().unwrap();

            if let Some(content_type) = meta.content_type.map(Serde::into_inner) {
                let c = content_type.to_string();
                // The MIME crate includes params (e.g. charset=utf8) in the to_string
                // https://github.com/hyperium/mime/issues/120
                if let Some(ty) = c.split(';').next() {
                    if !SCRIPT_JS_MIMES.contains(&ty) {
                        return Err(NetworkError::Internal(format!("Invalid MIME type: {}", ty)));
                    }
                } else {
                    return Err(NetworkError::Internal("Empty MIME type".into()));
                }
            } else {
                return Err(NetworkError::Internal("No MIME type".into()));
            }

            // Step 10.
            let (source_text, _, _) = UTF_8.decode(&self.data);
            Ok(ScriptOrigin::external(
                DOMString::from(source_text),
                meta.final_url,
                ScriptType::Module,
            ))
        });

        if let Err(err) = load {
            // Step 9.
            error!("Failed to fetch {} with error {:?}", self.url.clone(), err);
            let module_tree = {
                let module_map = global.get_module_map().borrow();
                module_map.get(&self.url.clone()).unwrap().clone()
            };

            module_tree.set_status(ModuleStatus::FetchFailed);

            module_tree.set_error(Some(ModuleError::Network(err)));

            let promise = module_tree.get_promise().borrow();
            promise.as_ref().unwrap().resolve_native(&());

            return;
        }

        // Step 12.
        if let Ok(ref resp_mod_script) = load {
            let module_tree = {
                let module_map = global.get_module_map().borrow();
                module_map.get(&self.url.clone()).unwrap().clone()
            };

            module_tree.set_text(resp_mod_script.text());

            let compiled_module = module_tree.compile_module_script(
                &global,
                resp_mod_script.text(),
                self.url.clone(),
            );

            match compiled_module {
                Err(exception) => {
                    module_tree.set_error(Some(exception));

                    let promise = module_tree.get_promise().borrow();
                    promise.as_ref().unwrap().resolve_native(&());

                    return;
                },
                Ok(record) => {
                    module_tree.set_record(record);

                    {
                        let mut visited = module_tree.get_visited_urls().borrow_mut();
                        visited.insert(self.url.clone());
                    }

                    let descendant_results = fetch_module_descendants_and_link(
                        &self.owner,
                        &module_tree,
                        self.destination.clone(),
                        self.credentials_mode.clone(),
                    );

                    // Resolve the request of this module tree promise directly
                    // when there's no descendant
                    if descendant_results.is_none() {
                        module_tree.set_status(ModuleStatus::Ready);

                        let promise = module_tree.get_promise().borrow();
                        promise.as_ref().unwrap().resolve_native(&());
                    }
                },
            }
        }
    }

    fn resource_timing_mut(&mut self) -> &mut ResourceFetchTiming {
        &mut self.resource_timing
    }

    fn resource_timing(&self) -> &ResourceFetchTiming {
        &self.resource_timing
    }

    fn submit_resource_timing(&mut self) {
        network_listener::submit_timing(self)
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

#[allow(unsafe_code)]
/// A function to register module hooks (e.g. listening on resolving modules,
/// getting module metadata, getting script private reference and resolving dynamic import)
pub unsafe fn EnsureModuleHooksInitialized(rt: *mut JSRuntime) {
    if GetModuleResolveHook(rt).is_some() {
        return;
    }

    SetModuleResolveHook(rt, Some(HostResolveImportedModule));
    SetModuleMetadataHook(rt, None);
    SetScriptPrivateReferenceHooks(rt, None, None);

    SetModuleDynamicImportHook(rt, None);
}

#[allow(unsafe_code)]
/// https://tc39.github.io/ecma262/#sec-hostresolveimportedmodule
/// https://html.spec.whatwg.org/multipage/#hostresolveimportedmodule(referencingscriptormodule%2C-specifier)
unsafe extern "C" fn HostResolveImportedModule(
    cx: *mut JSContext,
    reference_private: RawHandleValue,
    specifier: RawHandle<*mut JSString>,
) -> *mut JSObject {
    let global_scope = GlobalScope::from_context(cx);

    // Step 2.
    let mut base_url = global_scope.api_base_url();

    // Step 3.
    let module_data = (reference_private.to_private() as *const ModuleScript).as_ref();
    if let Some(data) = module_data {
        base_url = data.base_url.clone();
    }

    // Step 5.
    let url = ModuleTree::resolve_module_specifier(*global_scope.get_cx(), &base_url, specifier);

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

/// https://html.spec.whatwg.org/multipage/#fetch-a-module-script-tree
pub fn fetch_external_module_script(
    owner: ModuleOwner,
    url: ServoUrl,
    destination: Destination,
    integrity_metadata: String,
    credentials_mode: CredentialsMode,
) -> Rc<Promise> {
    // Step 1.
    fetch_single_module_script(
        owner,
        url,
        destination,
        Referrer::Client,
        ParserMetadata::NotParserInserted,
        integrity_metadata,
        credentials_mode,
        None,
        true,
    )
}

/// https://html.spec.whatwg.org/multipage/#fetch-a-single-module-script
pub fn fetch_single_module_script(
    owner: ModuleOwner,
    url: ServoUrl,
    destination: Destination,
    referrer: Referrer,
    parser_metadata: ParserMetadata,
    integrity_metadata: String,
    credentials_mode: CredentialsMode,
    parent_url: Option<ServoUrl>,
    top_level_module_fetch: bool,
) -> Rc<Promise> {
    {
        // Step 1.
        let global = owner.global();
        let module_map = global.get_module_map().borrow();

        debug!("Start to fetch {}", url);

        if let Some(module_tree) = module_map.get(&url.clone()) {
            let status = module_tree.get_status();

            let promise = module_tree.get_promise().borrow();

            debug!("Meet a fetched url {} and its status is {:?}", url, status);

            assert!(promise.is_some());

            module_tree.append_handler(owner.clone(), url.clone(), top_level_module_fetch);

            let promise = promise.as_ref().unwrap();

            match status {
                ModuleStatus::Initial => unreachable!(
                    "We have the module in module map so its status should not be `initial`"
                ),
                // Step 2.
                ModuleStatus::Fetching => return promise.clone(),
                ModuleStatus::FetchingDescendants => {
                    if module_tree.has_all_ready_descendants(&module_map) {
                        promise.resolve_native(&());
                    }
                },
                // Step 3.
                ModuleStatus::FetchFailed | ModuleStatus::Ready | ModuleStatus::Finished => {
                    promise.resolve_native(&());
                },
            }

            return promise.clone();
        }
    }

    let global = owner.global();

    let module_tree = ModuleTree::new(url.clone());
    module_tree.set_status(ModuleStatus::Fetching);

    let promise = owner.gen_promise_with_final_handler(Some(url.clone()), top_level_module_fetch);

    module_tree.set_promise(promise.clone());
    if let Some(parent_url) = parent_url {
        module_tree.insert_parent_url(parent_url);
    }

    // Step 4.
    global.set_module_map(url.clone(), module_tree);

    // Step 5-6.
    let mode = match destination.clone() {
        Destination::Worker | Destination::SharedWorker if top_level_module_fetch => {
            RequestMode::SameOrigin
        },
        _ => RequestMode::CorsMode,
    };

    let document: Option<DomRoot<Document>> = match &owner {
        ModuleOwner::Worker(_) => None,
        ModuleOwner::Window(script) => Some(document_from_node(&*script.root())),
    };

    // Step 7-8.
    let request = RequestBuilder::new(url.clone())
        .destination(destination.clone())
        .origin(global.origin().immutable().clone())
        .referrer(Some(referrer))
        .parser_metadata(parser_metadata)
        .integrity_metadata(integrity_metadata.clone())
        .credentials_mode(credentials_mode)
        .mode(mode);

    let context = Arc::new(Mutex::new(ModuleContext {
        owner,
        data: vec![],
        metadata: None,
        url: url.clone(),
        destination: destination.clone(),
        credentials_mode: credentials_mode.clone(),
        status: Ok(()),
        resource_timing: ResourceFetchTiming::new(ResourceTimingType::Resource),
    }));

    let (action_sender, action_receiver) = ipc::channel().unwrap();

    let listener = NetworkListener {
        context,
        task_source: global.networking_task_source(),
        canceller: Some(global.task_canceller(TaskSourceName::Networking)),
    };

    ROUTER.add_route(
        action_receiver.to_opaque(),
        Box::new(move |message| {
            listener.notify_fetch(message.to().unwrap());
        }),
    );

    if let Some(doc) = document {
        doc.fetch_async(LoadType::Script(url), request, action_sender);
    }

    promise
}

#[allow(unsafe_code)]
/// https://html.spec.whatwg.org/multipage/#fetch-an-inline-module-script-graph
pub fn fetch_inline_module_script(
    owner: ModuleOwner,
    module_script_text: DOMString,
    url: ServoUrl,
    script_id: ScriptId,
    credentials_mode: CredentialsMode,
) {
    let global = owner.global();

    let module_tree = ModuleTree::new(url.clone());

    let promise = owner.gen_promise_with_final_handler(None, true);

    module_tree.set_promise(promise.clone());

    let compiled_module =
        module_tree.compile_module_script(&global, module_script_text, url.clone());

    match compiled_module {
        Ok(record) => {
            module_tree.set_record(record);

            let descendant_results = fetch_module_descendants_and_link(
                &owner,
                &module_tree,
                Destination::Script,
                credentials_mode,
            );

            global.set_inline_module_map(script_id, module_tree);

            if descendant_results.is_none() {
                promise.resolve_native(&());
            }
        },
        Err(exception) => {
            module_tree.set_status(ModuleStatus::Ready);
            module_tree.set_error(Some(exception));
            global.set_inline_module_map(script_id, module_tree);
            promise.resolve_native(&());
        },
    }
}

/// https://html.spec.whatwg.org/multipage/#fetch-the-descendants-of-and-link-a-module-script
/// Step 1-3.
#[allow(unsafe_code)]
fn fetch_module_descendants_and_link(
    owner: &ModuleOwner,
    module_tree: &ModuleTree,
    destination: Destination,
    credentials_mode: CredentialsMode,
) -> Option<Rc<Promise>> {
    let descendant_results =
        fetch_module_descendants(owner, module_tree, destination, credentials_mode);

    match descendant_results {
        Ok(descendants) => {
            if descendants.len() > 0 {
                unsafe {
                    let global = owner.global();

                    let _compartment = enter_realm(&*global);
                    AlreadyInCompartment::assert(&*global);
                    let _ais = AutoIncumbentScript::new(&*global);

                    let abv = CreateAutoObjectVector(*global.get_cx());

                    for descendant in descendants {
                        assert!(AppendToAutoObjectVector(
                            abv as *mut AutoObjectVector,
                            descendant.promise_obj().get()
                        ));
                    }

                    rooted!(in(*global.get_cx()) let raw_promise_all = GetWaitForAllPromise(*global.get_cx(), abv));

                    let promise_all =
                        Promise::new_with_js_promise(raw_promise_all.handle(), global.get_cx());

                    let promise = module_tree.get_promise().borrow();
                    let promise = promise.as_ref().unwrap().clone();

                    let resolve_promise = TrustedPromise::new(promise.clone());
                    let reject_promise = TrustedPromise::new(promise.clone());

                    let handler = PromiseNativeHandler::new(
                        &global,
                        Some(ModuleHandler::new(Box::new(
                            task!(all_fetched_resolve: move || {
                                let promise = resolve_promise.root();
                                promise.resolve_native(&());
                            }),
                        ))),
                        Some(ModuleHandler::new(Box::new(
                            task!(all_failure_reject: move || {
                                let promise = reject_promise.root();
                                promise.reject_native(&());
                            }),
                        ))),
                    );

                    promise_all.append_native_handler(&handler);

                    return Some(promise_all);
                }
            }
        },
        Err(err) => {
            module_tree.set_error(Some(err));
        },
    }

    None
}

#[allow(unsafe_code)]
/// https://html.spec.whatwg.org/multipage/#fetch-the-descendants-of-a-module-script
fn fetch_module_descendants(
    owner: &ModuleOwner,
    module_tree: &ModuleTree,
    destination: Destination,
    credentials_mode: CredentialsMode,
) -> Result<Vec<Rc<Promise>>, ModuleError> {
    debug!("Start to load dependencies of {}", module_tree.url.clone());

    let global = owner.global();

    module_tree.set_status(ModuleStatus::FetchingDescendants);

    module_tree
        .resolve_requested_modules(&global)
        .map(|requested_urls| {
            module_tree.append_descendant_urls(requested_urls.clone());

            let parent_urls = module_tree.get_parent_urls().borrow();

            if parent_urls.intersection(&requested_urls).count() > 0 {
                return Vec::new();
            }

            requested_urls
                .iter()
                .map(|requested_url| {
                    // https://html.spec.whatwg.org/multipage/#internal-module-script-graph-fetching-procedure
                    // Step 1.
                    {
                        let visited = module_tree.get_visited_urls().borrow();
                        assert!(visited.get(&requested_url).is_some());
                    }

                    // Step 2.
                    fetch_single_module_script(
                        owner.clone(),
                        requested_url.clone(),
                        destination.clone(),
                        Referrer::Client,
                        ParserMetadata::NotParserInserted,
                        "".to_owned(),
                        credentials_mode.clone(),
                        Some(module_tree.url.clone()),
                        false,
                    )
                })
                .collect()
        })
}

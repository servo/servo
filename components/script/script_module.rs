/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! The script module mod contains common traits and structs
//! related to `type=module` for script thread or worker threads.

use crate::compartments::{enter_realm, AlreadyInCompartment, InCompartment};
use crate::document_loader::LoadType;
use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::conversions::jsstring_to_str;
use crate::dom::bindings::error::Error;
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::refcounted::Trusted;
use crate::dom::bindings::reflector::DomObject;
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::settings_stack::AutoIncumbentScript;
use crate::dom::bindings::str::DOMString;
use crate::dom::bindings::trace::RootedTraceableBox;
use crate::dom::document::Document;
use crate::dom::element::Element;
use crate::dom::globalscope::GlobalScope;
use crate::dom::htmlscriptelement::{HTMLScriptElement, ScriptOrigin, ScriptType};
use crate::dom::node::document_from_node;
use crate::dom::performanceresourcetiming::InitiatorType;
use crate::dom::promise::Promise;
use crate::dom::promisenativehandler::{Callback, PromiseNativeHandler};
use crate::dom::worker::TrustedWorkerAddress;
use crate::network_listener::{self, NetworkListener};
use crate::network_listener::{PreInvoke, ResourceTimingListener};
use crate::task::TaskBox;
use crate::task_source::TaskSourceName;
use encoding_rs::{Encoding, UTF_8};
use ipc_channel::ipc;
use ipc_channel::router::ROUTER;
use js::jsapi::HandleObject;
use js::jsapi::HandleValue as RawHandleValue;
use js::jsapi::{GetModuleResolveHook, JSRuntime, SetModuleResolveHook};
use js::jsapi::{GetRequestedModules, SetModuleMetadataHook};
use js::jsapi::{Handle, JSAutoRealm, JSObject, JSString};
use js::jsapi::{Heap, JSContext, JS_ClearPendingException};
use js::jsapi::{ModuleEvaluate, ModuleInstantiate, SourceText};
use js::jsapi::{SetModuleDynamicImportHook, SetScriptPrivateReferenceHooks};
use js::jsval::{JSVal, UndefinedValue};
use js::rust::jsapi_wrapped::{CompileModule, JS_GetArrayLength, JS_GetElement};
use js::rust::jsapi_wrapped::{GetRequestedModuleSpecifier, JS_GetPendingException};
use js::rust::CompileOptionsWrapper;
use js::rust::HandleValue;
use js::rust::IntoHandle;
use net_traits::request::{Destination, ParserMetadata, Referrer, RequestBuilder, RequestMode};
use net_traits::{FetchMetadata, Metadata};
use net_traits::{FetchResponseListener, NetworkError};
use net_traits::{ResourceFetchTiming, ResourceTimingType};
use servo_url::ServoUrl;
use std::collections::HashSet;
use std::ffi;
use std::marker::PhantomData;
use std::ptr;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use url::ParseError as UrlParseError;

pub fn get_source_text(source: Vec<u16>) -> SourceText<u16> {
    SourceText {
        units_: source.as_ptr() as *const _,
        length_: source.len() as u32,
        ownsUnits_: false,
        _phantom_0: PhantomData,
    }
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
pub struct ModuleException(RootedTraceableBox<Heap<JSVal>>);

impl ModuleException {
    #[allow(unsafe_code)]
    pub fn handle(&self) -> Handle<JSVal> {
        self.0.handle().into_handle()
    }
}

#[derive(JSTraceable)]
pub struct ModuleTree {
    pub url: ServoUrl,
    text: DomRefCell<DOMString>,
    record: DomRefCell<Option<ModuleObject>>,
    status: DomRefCell<ModuleStatus>,
    descendant_urls: HashSet<ServoUrl>,
    visited_urls: HashSet<ServoUrl>,
    error: DomRefCell<Option<ModuleException>>,
    promise: DomRefCell<Option<Rc<Promise>>>,
}

impl ModuleTree {
    pub fn new(url: ServoUrl) -> Self {
        ModuleTree {
            url,
            text: DomRefCell::new(DOMString::new()),
            record: DomRefCell::new(None),
            status: DomRefCell::new(ModuleStatus::Initial),
            descendant_urls: HashSet::new(),
            visited_urls: HashSet::new(),
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

    pub fn get_error(&self) -> &DomRefCell<Option<ModuleException>> {
        &self.error
    }

    pub fn set_error(&self, error: ModuleException) {
        *self.error.borrow_mut() = Some(error);
    }

    pub fn get_text(&self) -> &DomRefCell<DOMString> {
        &self.text
    }

    pub fn set_text(&self, module_text: DOMString) {
        *self.text.borrow_mut() = module_text;
    }

    pub fn append_handler(&self, owner: ModuleOwner) {
        let promise = self.promise.borrow();

        let this = owner.clone();

        let handler = PromiseNativeHandler::new(
            &owner.global(),
            Some(ModuleHandler::new(Box::new(
                task!(fetched_resolve: move || {
                    println!("fetched");
                    this.finish_module_load();
                }),
            ))),
            Some(ModuleHandler::new(Box::new(
                task!(failure_reject: move || {
                    println!("failed");
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

#[derive(Copy, Clone, Debug, JSTraceable, PartialEq, PartialOrd)]
pub enum ModuleStatus {
    Initial,
    Fetching,
    FetchingDescendants,
    Finished,
}

impl ModuleTree {
    #[allow(unsafe_code)]
    /// https://html.spec.whatwg.org/multipage/#creating-a-javascript-module-script
    /// Step 7-11.
    fn compile_module_script(
        &self,
        global: &GlobalScope,
        module_script_text: DOMString,
        url: ServoUrl,
    ) -> Result<ModuleObject, ModuleException> {
        let module: Vec<u16> = module_script_text.encode_utf16().collect();

        let url_cstr = ffi::CString::new(url.as_str().as_bytes()).unwrap();

        let _ac = JSAutoRealm::new(*global.get_cx(), *global.reflector().get_jsobject());

        let compile_options = CompileOptionsWrapper::new(*global.get_cx(), url_cstr.as_ptr(), 1);

        rooted!(in(*global.get_cx()) let mut module_script = ptr::null_mut::<JSObject>());

        let mut source = get_source_text(module);

        unsafe {
            if !CompileModule(
                *global.get_cx(),
                compile_options.ptr,
                &mut source,
                &mut module_script.handle_mut(),
            ) {
                println!("fail to compile module script of {}", url);

                rooted!(in(*global.get_cx()) let mut exception = UndefinedValue());
                assert!(JS_GetPendingException(
                    *global.get_cx(),
                    &mut exception.handle_mut()
                ));
                JS_ClearPendingException(*global.get_cx());

                return Err(ModuleException(RootedTraceableBox::from_box(Heap::boxed(
                    exception.get(),
                ))));
            }
        }

        println!("module script of {} compile done", url);

        if self
            .resolve_requested_module_specifiers(
                &global,
                module_script.handle().into_handle(),
                global.api_base_url().clone(),
            )
            .is_err()
        {
            unsafe {
                rooted!(in(*global.get_cx()) let mut thrown = UndefinedValue());
                Error::Type("Wrong module specifier".to_owned()).to_jsval(
                    *global.get_cx(),
                    &global,
                    thrown.handle_mut(),
                );

                return Err(ModuleException(RootedTraceableBox::from_box(Heap::boxed(
                    thrown.get(),
                ))));
            }
        }

        Ok(ModuleObject(Heap::boxed(*module_script)))
    }

    #[allow(unsafe_code)]
    pub fn instantiate_module_tree(
        &self,
        global: &GlobalScope,
        module_record: HandleObject,
    ) -> Result<(), ModuleException> {
        let _ac = JSAutoRealm::new(*global.get_cx(), *global.reflector().get_jsobject());

        unsafe {
            if !ModuleInstantiate(*global.get_cx(), module_record) {
                println!("fail to instantiate module");

                rooted!(in(*global.get_cx()) let mut exception = UndefinedValue());
                assert!(JS_GetPendingException(
                    *global.get_cx(),
                    &mut exception.handle_mut()
                ));
                JS_ClearPendingException(*global.get_cx());

                Err(ModuleException(RootedTraceableBox::from_box(Heap::boxed(
                    exception.get(),
                ))))
            } else {
                println!("module instantiated successfully");

                Ok(())
            }
        }
    }

    #[allow(unsafe_code)]
    pub fn execute_module(
        &self,
        global: &GlobalScope,
        module_record: HandleObject,
    ) -> Result<(), ModuleException> {
        let _ac = JSAutoRealm::new(*global.get_cx(), *global.reflector().get_jsobject());

        unsafe {
            if !ModuleEvaluate(*global.get_cx(), module_record) {
                println!("fail to evaluate module");

                rooted!(in(*global.get_cx()) let mut exception = UndefinedValue());
                assert!(JS_GetPendingException(
                    *global.get_cx(),
                    &mut exception.handle_mut()
                ));
                JS_ClearPendingException(*global.get_cx());

                Err(ModuleException(RootedTraceableBox::from_box(Heap::boxed(
                    exception.get(),
                ))))
            } else {
                println!("module evaluated successfully");
                Ok(())
            }
        }
    }

    pub fn resolve_requested_modules(
        global: &GlobalScope,
        parent_module_url: ServoUrl,
        visited: &mut HashSet<ServoUrl>,
    ) -> Result<HashSet<ServoUrl>, ()> {
        let module_map = global.get_module_map().borrow();

        let module_tree = module_map.get(&parent_module_url);

        if let Some(module_tree) = module_tree {
            assert_ne!(module_tree.get_status(), ModuleStatus::Initial);
            assert_ne!(module_tree.get_status(), ModuleStatus::Fetching);

            let mut requested_urls: HashSet<ServoUrl> = HashSet::new();

            let record = module_tree.get_record().borrow();

            if let Some(raw_record) = &*record {
                let valid_specifier_urls = module_tree.resolve_requested_module_specifiers(
                    &global,
                    raw_record.handle(),
                    parent_module_url.clone(),
                );

                if valid_specifier_urls.is_err() {
                    return Err(());
                }

                for parsed_url in valid_specifier_urls.unwrap().iter() {
                    if !visited.contains(&parsed_url) {
                        requested_urls.insert(parsed_url.clone());

                        visited.insert(parsed_url.clone());
                    }
                }

                return Ok(requested_urls);
            }
        }

        Err(())
    }

    #[allow(unsafe_code)]
    fn resolve_requested_module_specifiers(
        &self,
        global: &GlobalScope,
        module_object: HandleObject,
        base_url: ServoUrl,
    ) -> Result<HashSet<ServoUrl>, ()> {
        let _ac = JSAutoRealm::new(*global.get_cx(), *global.reflector().get_jsobject());

        let mut specifier_urls = HashSet::new();

        unsafe {
            rooted!(in(*global.get_cx()) let requested_modules = GetRequestedModules(*global.get_cx(), module_object));

            let mut length = 0;

            if !JS_GetArrayLength(*global.get_cx(), requested_modules.handle(), &mut length) {
                println!("Wrong length of requested modules");
                return Err(());
            }

            for index in 0..length {
                rooted!(in(*global.get_cx()) let mut element = UndefinedValue());

                if !JS_GetElement(
                    *global.get_cx(),
                    requested_modules.handle(),
                    index,
                    &mut element.handle_mut(),
                ) {
                    return Err(());
                }

                rooted!(in(*global.get_cx()) let specifier = GetRequestedModuleSpecifier(*global.get_cx(), element.handle()));

                let url = ModuleTree::resolve_module_specifier(
                    *global.get_cx(),
                    &base_url,
                    specifier.handle().into_handle(),
                );

                if url.is_err() {
                    return Err(());
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
        specifier: Handle<*mut JSString>,
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

    /// https://html.spec.whatwg.org/multipage/#creating-a-javascript-module-script
    #[allow(unused)]
    fn create_javascript_module_script(
        &self,
        global: &GlobalScope,
        module_script_text: DOMString,
        url: ServoUrl,
    ) {
        // TODO: Step 1. If scripting is disabled in responsible browsing context, set `source` to be empty string
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
            ModuleOwner::Window(script) => document_from_node(&*script.root()).global(),
        }
    }

    fn gen_promise_with_final_handler(&self) -> Rc<Promise> {
        let this = self.clone();

        let handler = PromiseNativeHandler::new(
            &self.global(),
            Some(ModuleHandler::new(Box::new(
                task!(fetched_resolve: move || {
                    println!("fetched");
                    this.finish_module_load();
                }),
            ))),
            Some(ModuleHandler::new(Box::new(
                task!(failure_reject: move || {
                    println!("failed");
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

    pub fn finish_module_load(&self) {
        match &self {
            ModuleOwner::Worker(_) => unimplemented!(),
            ModuleOwner::Window(script) => {
                let global = self.global();

                let document = document_from_node(&*script.root());

                let base_url = document.base_url();

                if let Some(script_src) = script
                    .root()
                    .upcast::<Element>()
                    .get_attribute(&ns!(), &local_name!("src"))
                    .map(|attr| base_url.join(&attr.value()).ok())
                    .unwrap_or(None)
                {
                    let module_map = global.get_module_map().borrow();
                    let module_tree = module_map.get(&script_src.clone()).unwrap();
                    let source_text = module_tree.get_text().borrow();

                    let load = Ok(ScriptOrigin::external(
                        source_text.clone(),
                        script_src.clone(),
                        ScriptType::Module,
                    ));

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

                    document.finish_load(LoadType::Script(script_src.clone()));
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
    /// Step 8-14
    #[allow(unsafe_code)]
    fn process_response_eof(&mut self, response: Result<ResourceFetchTiming, NetworkError>) {
        let global = self.owner.global();

        let load = response.and(self.status.clone()).map(|_| {
            let meta = self.metadata.take().unwrap();

            let encoding = meta
                .charset
                .and_then(|encoding| Encoding::for_label(encoding.as_bytes()))
                .unwrap_or(UTF_8);

            // Step 12-1 & 13-1.
            let (source_text, _, _) = encoding.decode(&self.data);
            ScriptOrigin::external(
                DOMString::from(source_text),
                meta.final_url,
                ScriptType::Module,
            )
        });

        if load.is_err() {
            // Step 9.
            let module_tree = ModuleTree::new(self.url.clone());
            module_tree.set_status(ModuleStatus::Finished);

            global.set_module_map(self.url.clone(), module_tree);

            return;
        }

        // TODO: Step 12 & 13. HANDLE MIME TYPE CHECKING

        // Step 14.
        if let Ok(ref resp_mod_script) = load {
            let module_map = global.get_module_map().borrow();
            let module_tree = module_map.get(&self.url.clone()).unwrap();

            module_tree.set_text(resp_mod_script.text());

            let compiled_module = module_tree.compile_module_script(
                &global,
                resp_mod_script.text(),
                self.url.clone(),
            );

            match compiled_module {
                Err(exception) => {
                    module_tree.set_error(exception);

                    let promise = module_tree.get_promise().borrow();
                    promise.as_ref().unwrap().reject_native(&());

                    return;
                },
                Ok(record) => {
                    module_tree.set_record(record);

                    let mut visited = HashSet::new();
                    visited.insert(self.url.clone());

                    let _descendant_result = fetch_module_descendants(
                        &self.owner,
                        true,
                        self.url.clone(),
                        self.destination.clone(),
                        visited,
                    );

                    // if let Ok(len) = descendant_result {
                    //     if len != 0 {
                    //         return;
                    //     }
                    // }

                    match &self.owner {
                        ModuleOwner::Worker(_) => unimplemented!(),
                        ModuleOwner::Window(script) => {
                            let document = document_from_node(&*script.root());

                            let base_url = document.base_url();

                            if let Some(script_src) = script
                                .root()
                                .upcast::<Element>()
                                .get_attribute(&ns!(), &local_name!("src"))
                                .map(|attr| base_url.join(&attr.value()).ok())
                                .unwrap_or(None)
                            {
                                if self.url.clone() == script_src {
                                    let promise = module_tree.get_promise().borrow();
                                    promise.as_ref().unwrap().resolve_native(&());
                                }
                            }
                        },
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
    _reference_private: RawHandleValue,
    specifier: Handle<*mut JSString>,
) -> *mut JSObject {
    let global_scope = GlobalScope::from_context(cx);

    // Step 2.
    let base_url = global_scope.api_base_url();

    // Step 5.
    let url = ModuleTree::resolve_module_specifier(*global_scope.get_cx(), &base_url, specifier);

    // Step 6.
    assert!(url.is_ok());

    let parsed_url = url.unwrap();

    println!("In HostResolveImportedModule: {}", parsed_url);

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
) -> Rc<Promise> {
    // Step 1.
    fetch_single_module_script(
        owner,
        url,
        destination,
        Referrer::Client,
        ParserMetadata::NotParserInserted,
        true,
    )
}

/// https://html.spec.whatwg.org/multipage/#internal-module-script-graph-fetching-procedure
pub fn perform_internal_module_script_fetch(
    owner: ModuleOwner,
    url: ServoUrl,
    destination: Destination,
    visited: HashSet<ServoUrl>,
    referrer: Referrer,
    parser_metadata: ParserMetadata,
    top_level_module_fetch: bool,
) -> Rc<Promise> {
    // Step 1.
    assert!(visited.get(&url).is_some());

    // Step 2.
    fetch_single_module_script(
        owner,
        url,
        destination,
        referrer,
        parser_metadata,
        top_level_module_fetch,
    )
}

/// https://html.spec.whatwg.org/multipage/#fetch-a-single-module-script
pub fn fetch_single_module_script(
    owner: ModuleOwner,
    url: ServoUrl,
    destination: Destination,
    referrer: Referrer,
    parser_metadata: ParserMetadata,
    top_level_module_fetch: bool,
) -> Rc<Promise> {
    {
        // Step 1.
        let global = owner.global();
        let module_map = global.get_module_map().borrow();

        if let Some(module_tree) = module_map.get(&url.clone()) {
            let status = module_tree.get_status();

            let promise = module_tree.get_promise().borrow();

            println!(
                "Meet a fetched url: {:?}, {}, {:?}",
                status,
                module_tree.url,
                promise.is_none()
            );

            assert!(promise.is_some());

            module_tree.append_handler(owner.clone());

            // Step 2.
            if status == ModuleStatus::Fetching || status == ModuleStatus::FetchingDescendants {
                // TODO: queue a network task ?
                return promise.as_ref().unwrap().clone();
            }

            // Step 3.
            if status == ModuleStatus::Finished {
                //  asynchronously complete this algorithm with moduleMap[url], and abort these steps
                let promise = promise.as_ref().unwrap();
                // promise.resolve_native(&());
                return promise.clone();
            }
        }
    }

    let global = owner.global();

    let module_tree = ModuleTree::new(url.clone());
    module_tree.set_status(ModuleStatus::Fetching);

    let promise = owner.gen_promise_with_final_handler();

    module_tree.set_promise(promise.clone());

    // Step 4.
    global.set_module_map(url.clone(), module_tree);

    // Step 5-6.
    let mode = match destination.clone() {
        Destination::Worker | Destination::SharedWorker if top_level_module_fetch => {
            RequestMode::SameOrigin
        },
        _ => RequestMode::NoCors,
    };

    let document: Option<DomRoot<Document>> = match &owner {
        ModuleOwner::Worker(_) => None,
        ModuleOwner::Window(script) => Some(document_from_node(&*script.root())),
    };

    // Step 7-8.
    let request = RequestBuilder::new(url.clone())
        .destination(destination.clone())
        .referrer(Some(referrer))
        .parser_metadata(parser_metadata)
        .mode(mode);

    let context = Arc::new(Mutex::new(ModuleContext {
        owner,
        data: vec![],
        metadata: None,
        url: url.clone(),
        destination: destination.clone(),
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
) {
    let global = owner.global();

    let module_tree = ModuleTree::new(url.clone());

    let compiled_module =
        module_tree.compile_module_script(&global, module_script_text, url.clone());

    if let Ok(record) = compiled_module {
        let _descendant_result =
            fetch_inline_module_descendants(&owner, url.clone(), Destination::Script);

        let _instantiated = module_tree.instantiate_module_tree(&global, record.handle());
        let _evaluated = module_tree.execute_module(&global, record.handle());

        return;
    }
}

/// https://html.spec.whatwg.org/multipage/#fetch-the-descendants-of-and-link-a-module-script
fn fetch_inline_module_descendants(
    owner: &ModuleOwner,
    url: ServoUrl,
    destination: Destination,
) -> Result<u32, ()> {
    fetch_module_descendants(owner, false, url, destination, HashSet::new())
}

#[allow(unsafe_code)]
/// https://html.spec.whatwg.org/multipage/#fetch-the-descendants-of-a-module-script
fn fetch_module_descendants(
    owner: &ModuleOwner,
    external: bool,
    parent_module_url: ServoUrl,
    destination: Destination,
    mut visited: HashSet<ServoUrl>,
) -> Result<u32, ()> {
    println!(
        "Start to load dependencies of {:?}",
        parent_module_url.clone()
    );

    // return a Promise from `fetch_module_descendants`
    // and then append a native handler to resolve so that
    // we can do this asynchronously

    let global = owner.global();

    let module_map = global.get_module_map().borrow();

    if external {
        let module_tree = module_map.get(&parent_module_url.clone()).unwrap();
        module_tree.set_status(ModuleStatus::FetchingDescendants);
    }

    if let Ok(requested_urls) =
        ModuleTree::resolve_requested_modules(&global, parent_module_url.clone(), &mut visited)
    {
        for url in requested_urls
            .iter()
            .filter(|url| module_map.get(&url).is_none())
        {
            perform_internal_module_script_fetch(
                owner.clone(),
                url.clone(),
                destination.clone(),
                visited.clone(),
                Referrer::Client,
                ParserMetadata::NotParserInserted,
                true,
            );
        }

        return Ok(requested_urls.len() as u32);
    }

    Err(())
}

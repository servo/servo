/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! The script module mod contains common traits and structs
//! related to `type=module` for script thread or worker threads.

use crate::document_loader::LoadType;
use crate::dom::bindings::conversions::jsstring_to_str;
use crate::dom::bindings::error::report_pending_exception;
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::refcounted::Trusted;
use crate::dom::bindings::reflector::DomObject;
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::DOMString;
use crate::dom::document::Document;
use crate::dom::element::Element;
use crate::dom::globalscope::GlobalScope;
use crate::dom::htmlscriptelement::{HTMLScriptElement, ScriptOrigin, ScriptType};
use crate::dom::node::document_from_node;
use crate::dom::performanceresourcetiming::InitiatorType;
use crate::dom::worker::TrustedWorkerAddress;
use crate::network_listener::{self, NetworkListener};
use crate::network_listener::{PreInvoke, ResourceTimingListener};
use crate::task_source::TaskSourceName;
use encoding_rs::{Encoding, UTF_8};
use ipc_channel::ipc;
use ipc_channel::router::ROUTER;
use js::jsapi::HandleObject;
use js::jsapi::SetModuleMetadataHook;
use js::jsapi::{GetModuleResolveHook, JSRuntime, SetModuleResolveHook};
use js::jsapi::{Handle, JSAutoRealm, JSObject, JSString};
use js::jsapi::{HandleValue, Heap, JSContext};
use js::jsapi::{ModuleEvaluate, ModuleInstantiate, SourceText};
use js::jsapi::{SetModuleDynamicImportHook, SetScriptPrivateReferenceHooks};
use js::panic::maybe_resume_unwind;
use js::rust::jsapi_wrapped::CompileModule;
use js::rust::CompileOptionsWrapper;
use net_traits::request::{Destination, ParserMetadata, Referrer, RequestBuilder, RequestMode};
use net_traits::{FetchMetadata, Metadata};
use net_traits::{FetchResponseListener, NetworkError};
use net_traits::{ResourceFetchTiming, ResourceTimingType};
use servo_url::ServoUrl;
use std::collections::HashSet;
use std::ffi;
use std::marker::PhantomData;
use std::ptr;
use std::sync::{Arc, Mutex};
use url::ParseError as UrlParseError;

#[derive(JSTraceable, PartialEq)]
pub enum ModuleObject {
    Fetching,
    Fetched(Option<Box<Heap<*mut JSObject>>>),
}

impl ModuleObject {
    fn get_fetched_object(&self) -> Option<&Box<Heap<*mut JSObject>>> {
        match self {
            ModuleObject::Fetching | ModuleObject::Fetched(None) => None,
            ModuleObject::Fetched(Some(module)) => Some(module),
        }
    }
}

/// The owner of the module
/// It can be `worker` or `script` element
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
    /// Indicates whether the request failed, and why
    status: Result<(), NetworkError>,
    /// Timing object for this resource
    resource_timing: ResourceFetchTiming,
}

impl FetchResponseListener for ModuleContext {
    fn process_request_body(&mut self) {} // TODO(cybai): Perhaps add custom steps to perform fetch here?

    fn process_request_eof(&mut self) {} // TODO(cybai): Perhaps add custom steps to perform fetch here?

    fn process_response(&mut self, metadata: Result<FetchMetadata, NetworkError>) {
        // https://html.spec.whatwg.org/multipage/#fetch-a-single-module-script
        // Step 4.
        let global = self.owner.global();
        global.set_module_map(self.url.clone(), ModuleObject::Fetching);

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
            200...299 => Ok(()), // HTTP ok status codes
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

        // TODO: HANDLE MIME TYPE CHECKING

        match load {
            Err(_) => {
                // https://html.spec.whatwg.org/multipage/#fetch-a-single-module-script
                // Step 9.
                global.set_module_map(self.url.clone(), ModuleObject::Fetched(None));
            },
            Ok(resp_mod_script) => {
                let compiled_module =
                    compile_module_script(resp_mod_script.text(), self.url.clone(), &global);

                let compiled = compiled_module.ok().map(|compiled| Heap::boxed(compiled));

                global.set_module_map(self.url.clone(), ModuleObject::Fetched(compiled));

                match &self.owner {
                    ModuleOwner::Worker(_) => unimplemented!(),
                    ModuleOwner::Window(script) => {
                        let document = document_from_node(&*script.root());

                        let r#async = script
                            .root()
                            .upcast::<Element>()
                            .has_attribute(&local_name!("async"));

                        if r#async {
                            // document.asap_module_script_loaded(script.root(), load);
                        } else {
                            document
                                .deferred_module_script_loaded(&*script.root(), self.url.clone());
                        }

                        document.finish_load(LoadType::Script(self.url.clone()));
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

pub fn get_source_text(source: Vec<u16>) -> SourceText<u16> {
    SourceText {
        units_: source.as_ptr() as *const _,
        length_: source.len() as u32,
        ownsUnits_: false,
        _phantom_0: PhantomData,
    }
}

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

/// https://html.spec.whatwg.org/multipage/#fetch-a-module-script-tree
pub fn fetch_module_script_graph(
    owner: ModuleOwner,
    url: ServoUrl,
    // fetch_client_settings,
    destination: Destination,
    // options
) {
    // Step 1.
    let mut visited: HashSet<ServoUrl> = HashSet::new();
    visited.insert(url.clone());

    let global_scope = owner.global();

    // Step 2-3.
    create_module_script(
        owner,
        url,
        // fetch_client_settings,
        &global_scope,
        destination,
        // options,
        // module map settings
        visited,
        Referrer::Client,
        ParserMetadata::NotParserInserted,
        true,
    );
}

/// https://html.spec.whatwg.org/multipage/#internal-module-script-graph-fetching-procedure
pub fn create_module_script(
    owner: ModuleOwner,
    url: ServoUrl,
    // fetch_client_settings,
    global_scope: &GlobalScope,
    destination: Destination,
    // options,
    // module map settings
    visited: HashSet<ServoUrl>,
    referrer: Referrer,
    parser_metadata: ParserMetadata,
    top_level_module_fetch: bool,
) {
    // Step 1.
    assert!(visited.get(&url).is_some());

    // Step 2.
    fetch_single_module_script(
        owner,
        url,
        // fetch_client_settings
        global_scope,
        destination,
        // options,
        // module_map_settings,
        referrer,
        parser_metadata,
        top_level_module_fetch,
    );
}

/// https://html.spec.whatwg.org/multipage/#fetch-a-single-module-script
pub fn fetch_single_module_script(
    owner: ModuleOwner,
    url: ServoUrl,
    // fetch_client_settings,
    global_scope: &GlobalScope,
    destination: Destination,
    // options,
    // module_map_settings,
    referrer: Referrer,
    parser_metadata: ParserMetadata,
    top_level_module_fetch: bool,
) {
    // Step 1.
    let module_map = global_scope.get_module_map().borrow();
    let module_object = module_map.get(&url.clone());

    // Step 2.
    if let Some(ModuleObject::Fetching) = module_object {
        return;
    }

    // Step 3.
    if let Some(ModuleObject::Fetched(Some(_))) = module_object {
        return;
    }

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
        status: Ok(()),
        resource_timing: ResourceFetchTiming::new(ResourceTimingType::Resource),
    }));

    let (action_sender, action_receiver) = ipc::channel().unwrap();

    let listener = NetworkListener {
        context,
        task_source: global_scope.networking_task_source(),
        canceller: Some(global_scope.task_canceller(TaskSourceName::Networking)),
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
}

#[allow(unsafe_code)]
/// https://html.spec.whatwg.org/multipage/#creating-a-javascript-module-script
/// Step 7.
fn compile_module_script(
    module_script_text: DOMString,
    url: ServoUrl,
    global: &GlobalScope,
) -> Result<*mut JSObject, ()> {
    let module: Vec<u16> = module_script_text.encode_utf16().collect();

    let url_cstr = ffi::CString::new(url.as_str().as_bytes()).unwrap();

    let _ac = JSAutoRealm::new(global.get_cx(), *global.reflector().get_jsobject());

    let compile_options = CompileOptionsWrapper::new(global.get_cx(), url_cstr.as_ptr(), 1);

    rooted!(in(global.get_cx()) let mut module_script = ptr::null_mut::<JSObject>());

    let mut source = get_source_text(module);

    unsafe {
        if !CompileModule(
            global.get_cx(),
            compile_options.ptr,
            &mut source,
            &mut module_script.handle_mut(),
        ) {
            println!("fail to compile module script of {}", url);

            report_pending_exception(global.get_cx(), true);
            maybe_resume_unwind();

            Err(())
        } else {
            println!("module script of {} compile done", url);

            Ok(*module_script)
        }
    }
}

#[allow(unsafe_code)]
pub fn instantiate_module_tree(
    global: &GlobalScope,
    module_record: HandleObject,
) -> Result<(), ()> {
    let _ac = JSAutoRealm::new(global.get_cx(), *global.reflector().get_jsobject());

    unsafe {
        if !ModuleInstantiate(global.get_cx(), module_record) {
            println!("fail to instantiate module");

            report_pending_exception(global.get_cx(), true);
            maybe_resume_unwind();

            Err(())
        } else {
            println!("module instantiated successfully");

            Ok(())
        }
    }
}

#[allow(unsafe_code)]
pub fn execute_module(global: &GlobalScope, module_record: HandleObject) -> Result<(), ()> {
    let _ac = JSAutoRealm::new(global.get_cx(), *global.reflector().get_jsobject());

    unsafe {
        if !ModuleEvaluate(global.get_cx(), module_record) {
            println!("fail to evaluate module");

            report_pending_exception(global.get_cx(), true);
            maybe_resume_unwind();

            Err(())
        } else {
            println!("module evaluated successfully");
            Ok(())
        }
    }
}

#[allow(unsafe_code)]
/// https://tc39.github.io/ecma262/#sec-hostresolveimportedmodule
/// https://html.spec.whatwg.org/multipage/#hostresolveimportedmodule(referencingscriptormodule%2C-specifier)
unsafe extern "C" fn HostResolveImportedModule(
    cx: *mut JSContext,
    _reference_private: HandleValue,
    specifier: Handle<*mut JSString>,
) -> *mut JSObject {
    let global_scope = GlobalScope::from_context(cx);

    // Step 2.
    let base_url = global_scope.api_base_url();

    // Step 5.
    let url = resolve_module_specifier(global_scope.get_cx(), &base_url, specifier);

    // Step 6.
    assert!(url.is_ok());

    let parsed_url = url.unwrap();

    // Step 4 & 7.
    let module_map = global_scope.get_module_map().borrow();

    let module = module_map.get(&parsed_url);

    // Step 9.
    assert!(module.is_some());

    let fetched_module_object = module.unwrap().get_fetched_object();

    // Step 8.
    assert!(fetched_module_object.is_some());

    // Step 10.
    return fetched_module_object.unwrap().handle().get();
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

/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use core::ffi::c_void;
use std::cell::Cell;
use std::fs::{create_dir_all, read_to_string, File};
use std::io::{Read, Seek, Write};
use std::path::PathBuf;
use std::process::Command;
use std::ptr;
use std::rc::Rc;
use std::sync::{Arc, Mutex};

use base::id::PipelineId;
use content_security_policy as csp;
use dom_struct::dom_struct;
use encoding_rs::Encoding;
use html5ever::{local_name, namespace_url, ns, LocalName, Prefix};
use ipc_channel::ipc;
use ipc_channel::router::ROUTER;
use js::jsapi::{CanCompileOffThread, CompileToStencilOffThread1, OffThreadToken};
use js::jsval::UndefinedValue;
use js::rust::{
    transform_str_to_source_text, CompileOptionsWrapper, FinishOffThreadStencil, HandleObject,
    Stencil,
};
use net_traits::request::{
    CorsSettings, CredentialsMode, Destination, ParserMetadata, RequestBuilder,
};
use net_traits::{
    FetchMetadata, FetchResponseListener, Metadata, NetworkError, ResourceFetchTiming,
    ResourceTimingType,
};
use servo_atoms::Atom;
use servo_config::pref;
use servo_url::{ImmutableOrigin, ServoUrl};
use style::str::{StaticStringVec, HTML_SPACE_CHARACTERS};
use uuid::Uuid;

use crate::document_loader::LoadType;
use crate::dom::attr::Attr;
use crate::dom::bindings::codegen::Bindings::DocumentBinding::DocumentMethods;
use crate::dom::bindings::codegen::Bindings::HTMLScriptElementBinding::HTMLScriptElementMethods;
use crate::dom::bindings::codegen::Bindings::NodeBinding::NodeMethods;
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::refcounted::Trusted;
use crate::dom::bindings::reflector::DomObject;
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::bindings::settings_stack::AutoEntryScript;
use crate::dom::bindings::str::{DOMString, USVString};
use crate::dom::bindings::trace::NoTrace;
use crate::dom::document::Document;
use crate::dom::element::{
    cors_setting_for_element, referrer_policy_for_element, reflect_cross_origin_attribute,
    reflect_referrer_policy_attribute, set_cross_origin_attribute, AttributeMutation, Element,
    ElementCreator,
};
use crate::dom::event::{Event, EventBubbles, EventCancelable, EventStatus};
use crate::dom::globalscope::GlobalScope;
use crate::dom::htmlelement::HTMLElement;
use crate::dom::node::{
    document_from_node, window_from_node, BindContext, ChildrenMutation, CloneChildrenFlag, Node,
};
use crate::dom::performanceresourcetiming::InitiatorType;
use crate::dom::virtualmethods::VirtualMethods;
use crate::fetch::create_a_potential_cors_request;
use crate::network_listener::{self, NetworkListener, PreInvoke, ResourceTimingListener};
use crate::realms::enter_realm;
use crate::script_module::{
    fetch_external_module_script, fetch_inline_module_script, ModuleOwner, ScriptFetchOptions,
};
use crate::task::TaskCanceller;
use crate::task_source::dom_manipulation::DOMManipulationTaskSource;
use crate::task_source::{TaskSource, TaskSourceName};

pub struct OffThreadCompilationContext {
    script_element: Trusted<HTMLScriptElement>,
    script_kind: ExternalScriptKind,
    final_url: ServoUrl,
    url: ServoUrl,
    task_source: DOMManipulationTaskSource,
    canceller: TaskCanceller,
    script_text: String,
    fetch_options: ScriptFetchOptions,
}

/// A wrapper to mark OffThreadToken as Send,
/// which should be safe according to
/// mozjs/js/public/OffThreadScriptCompilation.h
struct OffThreadCompilationToken(*mut OffThreadToken);

#[allow(unsafe_code)]
unsafe impl Send for OffThreadCompilationToken {}

#[allow(unsafe_code)]
unsafe extern "C" fn off_thread_compilation_callback(
    token: *mut OffThreadToken,
    callback_data: *mut c_void,
) {
    let mut context = Box::from_raw(callback_data as *mut OffThreadCompilationContext);
    let token = OffThreadCompilationToken(token);

    let url = context.url.clone();
    let final_url = context.final_url.clone();
    let script_element = context.script_element.clone();
    let script_kind = context.script_kind;
    let script = std::mem::take(&mut context.script_text);
    let fetch_options = context.fetch_options.clone();

    // Continue with <https://html.spec.whatwg.org/multipage/#fetch-a-classic-script>
    let _ = context.task_source.queue_with_canceller(
        task!(off_thread_compile_continue: move || {
            let elem = script_element.root();
            let global = elem.global();
            let cx = GlobalScope::get_cx();
            let _ar = enter_realm(&*global);

            // TODO: This is necessary because the rust compiler will otherwise try to move the *mut
            // OffThreadToken directly, which isn't marked as Send. The correct fix is that this
            // type is marked as Send in mozjs.
            let used_token = token;

            let compiled_script = FinishOffThreadStencil(*cx, used_token.0, ptr::null_mut());
            let load = if compiled_script.is_null() {
                Err(NoTrace(NetworkError::Internal(
                    "Off-thread compilation failed.".into(),
                )))
            } else {
                let script_text = DOMString::from(script);
                let code = SourceCode::Compiled(CompiledSourceCode {
                    source_code: compiled_script,
                    original_text: Rc::new(script_text),
                });

                Ok(ScriptOrigin {
                    code,
                    url: final_url,
                    external: true,
                    fetch_options,
                    type_: ScriptType::Classic,
                })
            };

            finish_fetching_a_classic_script(&elem, script_kind, url, load);
        }),
        &context.canceller,
    );
}

/// An unique id for script element.
#[derive(Clone, Copy, Debug, Eq, Hash, JSTraceable, PartialEq)]
pub struct ScriptId(#[no_trace] Uuid);

#[dom_struct]
pub struct HTMLScriptElement {
    htmlelement: HTMLElement,

    /// <https://html.spec.whatwg.org/multipage/#already-started>
    already_started: Cell<bool>,

    /// <https://html.spec.whatwg.org/multipage/#parser-inserted>
    parser_inserted: Cell<bool>,

    /// <https://html.spec.whatwg.org/multipage/#non-blocking>
    ///
    /// (currently unused)
    non_blocking: Cell<bool>,

    /// Document of the parser that created this element
    parser_document: Dom<Document>,

    /// Track line line_number
    line_number: u64,

    /// Unique id for each script element
    #[ignore_malloc_size_of = "Defined in uuid"]
    id: ScriptId,
}

impl HTMLScriptElement {
    fn new_inherited(
        local_name: LocalName,
        prefix: Option<Prefix>,
        document: &Document,
        creator: ElementCreator,
    ) -> HTMLScriptElement {
        HTMLScriptElement {
            id: ScriptId(Uuid::new_v4()),
            htmlelement: HTMLElement::new_inherited(local_name, prefix, document),
            already_started: Cell::new(false),
            parser_inserted: Cell::new(creator.is_parser_created()),
            non_blocking: Cell::new(!creator.is_parser_created()),
            parser_document: Dom::from_ref(document),
            line_number: creator.return_line_number(),
        }
    }

    #[allow(crown::unrooted_must_root)]
    pub fn new(
        local_name: LocalName,
        prefix: Option<Prefix>,
        document: &Document,
        proto: Option<HandleObject>,
        creator: ElementCreator,
    ) -> DomRoot<HTMLScriptElement> {
        Node::reflect_node_with_proto(
            Box::new(HTMLScriptElement::new_inherited(
                local_name, prefix, document, creator,
            )),
            document,
            proto,
        )
    }

    pub fn get_script_id(&self) -> ScriptId {
        self.id
    }
}

/// Supported script types as defined by
/// <https://html.spec.whatwg.org/multipage/#javascript-mime-type>.
pub static SCRIPT_JS_MIMES: StaticStringVec = &[
    "application/ecmascript",
    "application/javascript",
    "application/x-ecmascript",
    "application/x-javascript",
    "text/ecmascript",
    "text/javascript",
    "text/javascript1.0",
    "text/javascript1.1",
    "text/javascript1.2",
    "text/javascript1.3",
    "text/javascript1.4",
    "text/javascript1.5",
    "text/jscript",
    "text/livescript",
    "text/x-ecmascript",
    "text/x-javascript",
];

#[derive(Clone, Copy, JSTraceable, MallocSizeOf, PartialEq)]
pub enum ScriptType {
    Classic,
    Module,
}

#[derive(JSTraceable, MallocSizeOf)]
pub struct CompiledSourceCode {
    #[ignore_malloc_size_of = "SM handles JS values"]
    pub source_code: Stencil,
    #[ignore_malloc_size_of = "Rc is hard"]
    pub original_text: Rc<DOMString>,
}

#[derive(JSTraceable)]
pub enum SourceCode {
    Text(Rc<DOMString>),
    Compiled(CompiledSourceCode),
}

#[derive(JSTraceable, MallocSizeOf)]
pub struct ScriptOrigin {
    #[ignore_malloc_size_of = "Rc is hard"]
    code: SourceCode,
    #[no_trace]
    url: ServoUrl,
    external: bool,
    fetch_options: ScriptFetchOptions,
    type_: ScriptType,
}

impl ScriptOrigin {
    pub fn internal(
        text: Rc<DOMString>,
        url: ServoUrl,
        fetch_options: ScriptFetchOptions,
        type_: ScriptType,
    ) -> ScriptOrigin {
        ScriptOrigin {
            code: SourceCode::Text(text),
            url,
            external: false,
            fetch_options,
            type_,
        }
    }

    pub fn external(
        text: Rc<DOMString>,
        url: ServoUrl,
        fetch_options: ScriptFetchOptions,
        type_: ScriptType,
    ) -> ScriptOrigin {
        ScriptOrigin {
            code: SourceCode::Text(text),
            url,
            external: true,
            fetch_options,
            type_,
        }
    }

    pub fn text(&self) -> Rc<DOMString> {
        match &self.code {
            SourceCode::Text(text) => Rc::clone(text),
            SourceCode::Compiled(compiled_script) => Rc::clone(&compiled_script.original_text),
        }
    }
}

/// Final steps of <https://html.spec.whatwg.org/multipage/#fetch-a-classic-script>
fn finish_fetching_a_classic_script(
    elem: &HTMLScriptElement,
    script_kind: ExternalScriptKind,
    url: ServoUrl,
    load: ScriptResult,
) {
    // Step 11, Asynchronously complete this algorithm with script,
    // which refers to step 26.6 "When the chosen algorithm asynchronously completes",
    // of https://html.spec.whatwg.org/multipage/#prepare-a-script
    let document = document_from_node(elem);

    match script_kind {
        ExternalScriptKind::Asap => document.asap_script_loaded(elem, load),
        ExternalScriptKind::AsapInOrder => document.asap_in_order_script_loaded(elem, load),
        ExternalScriptKind::Deferred => document.deferred_script_loaded(elem, load),
        ExternalScriptKind::ParsingBlocking => {
            document.pending_parsing_blocking_script_loaded(elem, load)
        },
    }

    document.finish_load(LoadType::Script(url));
}

pub type ScriptResult = Result<ScriptOrigin, NoTrace<NetworkError>>;

/// The context required for asynchronously loading an external script source.
struct ClassicContext {
    /// The element that initiated the request.
    elem: Trusted<HTMLScriptElement>,
    /// The kind of external script.
    kind: ExternalScriptKind,
    /// The (fallback) character encoding argument to the "fetch a classic
    /// script" algorithm.
    character_encoding: &'static Encoding,
    /// The response body received to date.
    data: Vec<u8>,
    /// The response metadata received to date.
    metadata: Option<Metadata>,
    /// The initial URL requested.
    url: ServoUrl,
    /// Indicates whether the request failed, and why
    status: Result<(), NetworkError>,
    /// The fetch options of the script
    fetch_options: ScriptFetchOptions,
    /// Timing object for this resource
    resource_timing: ResourceFetchTiming,
}

impl FetchResponseListener for ClassicContext {
    fn process_request_body(&mut self) {} // TODO(KiChjang): Perhaps add custom steps to perform fetch here?

    fn process_request_eof(&mut self) {} // TODO(KiChjang): Perhaps add custom steps to perform fetch here?

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

    /// <https://html.spec.whatwg.org/multipage/#fetch-a-classic-script>
    /// step 4-9
    #[allow(unsafe_code)]
    fn process_response_eof(&mut self, response: Result<ResourceFetchTiming, NetworkError>) {
        let (source_text, final_url) = match (response.as_ref(), self.status.as_ref()) {
            (Err(err), _) | (_, Err(err)) => {
                // Step 6, response is an error.
                finish_fetching_a_classic_script(
                    &self.elem.root(),
                    self.kind,
                    self.url.clone(),
                    Err(NoTrace(err.clone())),
                );
                return;
            },
            (Ok(_), Ok(_)) => {
                let metadata = self.metadata.take().unwrap();

                // Step 7.
                let encoding = metadata
                    .charset
                    .and_then(|encoding| Encoding::for_label(encoding.as_bytes()))
                    .unwrap_or(self.character_encoding);

                // Step 8.
                let (source_text, _, _) = encoding.decode(&self.data);
                (source_text, metadata.final_url)
            },
        };

        let elem = self.elem.root();
        let global = elem.global();
        let cx = GlobalScope::get_cx();
        let _ar = enter_realm(&*global);

        let options = unsafe { CompileOptionsWrapper::new(*cx, final_url.as_str(), 1) };

        let can_compile_off_thread = pref!(dom.script.asynch) &&
            unsafe { CanCompileOffThread(*cx, options.ptr as *const _, source_text.len()) };

        if can_compile_off_thread {
            let source_string = source_text.to_string();

            let context = Box::new(OffThreadCompilationContext {
                script_element: self.elem.clone(),
                script_kind: self.kind,
                final_url,
                url: self.url.clone(),
                task_source: global.dom_manipulation_task_source(),
                canceller: global.task_canceller(TaskSourceName::DOMManipulation),
                script_text: source_string,
                fetch_options: self.fetch_options.clone(),
            });

            unsafe {
                assert!(!CompileToStencilOffThread1(
                    *cx,
                    options.ptr as *const _,
                    &mut transform_str_to_source_text(&context.script_text) as *mut _,
                    Some(off_thread_compilation_callback),
                    Box::into_raw(context) as *mut c_void,
                )
                .is_null());
            }
        } else {
            let load = ScriptOrigin::external(
                Rc::new(DOMString::from(source_text)),
                final_url.clone(),
                self.fetch_options.clone(),
                ScriptType::Classic,
            );
            finish_fetching_a_classic_script(&elem, self.kind, self.url.clone(), Ok(load));
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

impl ResourceTimingListener for ClassicContext {
    fn resource_timing_information(&self) -> (InitiatorType, ServoUrl) {
        let initiator_type = InitiatorType::LocalName(
            self.elem
                .root()
                .upcast::<Element>()
                .local_name()
                .to_string(),
        );
        (initiator_type, self.url.clone())
    }

    fn resource_timing_global(&self) -> DomRoot<GlobalScope> {
        document_from_node(&*self.elem.root()).global()
    }
}

impl PreInvoke for ClassicContext {}

/// Steps 1-2 of <https://html.spec.whatwg.org/multipage/#fetch-a-classic-script>
// This function is also used to prefetch a script in `script::dom::servoparser::prefetch`.
pub(crate) fn script_fetch_request(
    url: ServoUrl,
    cors_setting: Option<CorsSettings>,
    origin: ImmutableOrigin,
    pipeline_id: PipelineId,
    options: ScriptFetchOptions,
) -> RequestBuilder {
    // We intentionally ignore options' credentials_mode member for classic scripts.
    // The mode is initialized by create_a_potential_cors_request.
    create_a_potential_cors_request(
        url,
        Destination::Script,
        cors_setting,
        None,
        options.referrer,
    )
    .origin(origin)
    .pipeline_id(Some(pipeline_id))
    .parser_metadata(options.parser_metadata)
    .integrity_metadata(options.integrity_metadata.clone())
    .referrer_policy(options.referrer_policy)
}

/// <https://html.spec.whatwg.org/multipage/#fetch-a-classic-script>
fn fetch_a_classic_script(
    script: &HTMLScriptElement,
    kind: ExternalScriptKind,
    url: ServoUrl,
    cors_setting: Option<CorsSettings>,
    options: ScriptFetchOptions,
    character_encoding: &'static Encoding,
) {
    let doc = document_from_node(script);

    // Step 1, 2.
    let request = script_fetch_request(
        url.clone(),
        cors_setting,
        doc.origin().immutable().clone(),
        script.global().pipeline_id(),
        options.clone(),
    );

    // TODO: Step 3, Add custom steps to perform fetch

    let context = Arc::new(Mutex::new(ClassicContext {
        elem: Trusted::new(script),
        kind,
        character_encoding,
        data: vec![],
        metadata: None,
        url: url.clone(),
        status: Ok(()),
        fetch_options: options,
        resource_timing: ResourceFetchTiming::new(ResourceTimingType::Resource),
    }));

    let (action_sender, action_receiver) = ipc::channel().unwrap();
    let (task_source, canceller) = doc
        .window()
        .task_manager()
        .networking_task_source_with_canceller();
    let listener = NetworkListener {
        context,
        task_source,
        canceller: Some(canceller),
    };

    ROUTER.add_route(
        action_receiver.to_opaque(),
        Box::new(move |message| {
            listener.notify_fetch(message.to().unwrap());
        }),
    );
    doc.fetch_async(LoadType::Script(url), request, action_sender);
}

impl HTMLScriptElement {
    /// <https://html.spec.whatwg.org/multipage/#prepare-a-script>
    pub fn prepare(&self) {
        // Step 1.
        if self.already_started.get() {
            return;
        }

        // Step 2.
        let was_parser_inserted = self.parser_inserted.get();
        self.parser_inserted.set(false);

        // Step 4.
        let element = self.upcast::<Element>();
        let asynch = element.has_attribute(&local_name!("async"));
        // Note: confusingly, this is done if the element does *not* have an "async" attribute.
        if was_parser_inserted && !asynch {
            self.non_blocking.set(true);
        }

        // Step 5-6.
        let text = self.Text();
        if text.is_empty() && !element.has_attribute(&local_name!("src")) {
            return;
        }

        // Step 7.
        if !self.upcast::<Node>().is_connected() {
            return;
        }

        let script_type = if let Some(ty) = self.get_script_type() {
            ty
        } else {
            // Step 7.
            return;
        };

        // Step 8.
        if was_parser_inserted {
            self.parser_inserted.set(true);
            self.non_blocking.set(false);
        }

        // Step 10.
        self.already_started.set(true);

        // Step 12.
        let doc = document_from_node(self);
        if self.parser_inserted.get() && *self.parser_document != *doc {
            return;
        }

        // Step 13.
        if !doc.is_scripting_enabled() {
            return;
        }

        // Step 14
        if element.has_attribute(&local_name!("nomodule")) && script_type == ScriptType::Classic {
            return;
        }

        // Step 15.
        if !element.has_attribute(&local_name!("src")) &&
            doc.should_elements_inline_type_behavior_be_blocked(
                element,
                csp::InlineCheckType::Script,
                &text,
            ) == csp::CheckResult::Blocked
        {
            warn!("Blocking inline script due to CSP");
            return;
        }

        // Step 16.
        if script_type == ScriptType::Classic {
            let for_attribute = element.get_attribute(&ns!(), &local_name!("for"));
            let event_attribute = element.get_attribute(&ns!(), &local_name!("event"));
            if let (Some(ref for_attribute), Some(ref event_attribute)) =
                (for_attribute, event_attribute)
            {
                let for_value = for_attribute.value().to_ascii_lowercase();
                let for_value = for_value.trim_matches(HTML_SPACE_CHARACTERS);
                if for_value != "window" {
                    return;
                }

                let event_value = event_attribute.value().to_ascii_lowercase();
                let event_value = event_value.trim_matches(HTML_SPACE_CHARACTERS);
                if event_value != "onload" && event_value != "onload()" {
                    return;
                }
            }
        }

        // Step 17.
        let encoding = element
            .get_attribute(&ns!(), &local_name!("charset"))
            .and_then(|charset| Encoding::for_label(charset.value().as_bytes()))
            .unwrap_or_else(|| doc.encoding());

        // Step 18.
        let cors_setting = cors_setting_for_element(element);

        // Step 19.
        let module_credentials_mode = match script_type {
            ScriptType::Classic => CredentialsMode::CredentialsSameOrigin,
            ScriptType::Module => reflect_cross_origin_attribute(element).map_or(
                CredentialsMode::CredentialsSameOrigin,
                |attr| match &*attr {
                    "use-credentials" => CredentialsMode::Include,
                    "anonymous" => CredentialsMode::CredentialsSameOrigin,
                    _ => CredentialsMode::CredentialsSameOrigin,
                },
            ),
        };

        // TODO: Step 20: Nonce.

        // Step 21: Integrity metadata.
        let im_attribute = element.get_attribute(&ns!(), &local_name!("integrity"));
        let integrity_val = im_attribute.as_ref().map(|a| a.value());
        let integrity_metadata = match integrity_val {
            Some(ref value) => &***value,
            None => "",
        };

        // TODO: Step 22: referrer policy

        // Step 23
        let parser_metadata = if self.parser_inserted.get() {
            ParserMetadata::ParserInserted
        } else {
            ParserMetadata::NotParserInserted
        };

        // Step 24.
        let options = ScriptFetchOptions {
            cryptographic_nonce: "".into(),
            integrity_metadata: integrity_metadata.to_owned(),
            parser_metadata,
            referrer: self.global().get_referrer(),
            referrer_policy: referrer_policy_for_element(self.upcast::<Element>()),
            credentials_mode: module_credentials_mode,
        };

        // TODO: Step 23: environment settings object.

        let base_url = doc.base_url();
        if let Some(src) = element.get_attribute(&ns!(), &local_name!("src")) {
            // Step 26.

            // Step 26.1.
            let src = src.value();

            // Step 26.2.
            if src.is_empty() {
                self.queue_error_event();
                return;
            }

            // Step 26.3: The "from an external file"" flag is stored in ScriptOrigin.

            // Step 26.4-26.5.
            let url = match base_url.join(&src) {
                Ok(url) => url,
                Err(_) => {
                    warn!("error parsing URL for script {}", &**src);
                    self.queue_error_event();
                    return;
                },
            };

            // Step 26.6.
            match script_type {
                ScriptType::Classic => {
                    // Preparation for step 26.
                    let kind = if element.has_attribute(&local_name!("defer")) &&
                        was_parser_inserted &&
                        !asynch
                    {
                        // Step 26.a: classic, has src, has defer, was parser-inserted, is not async.
                        ExternalScriptKind::Deferred
                    } else if was_parser_inserted && !asynch {
                        // Step 26.c: classic, has src, was parser-inserted, is not async.
                        ExternalScriptKind::ParsingBlocking
                    } else if !asynch && !self.non_blocking.get() {
                        // Step 26.d: classic, has src, is not async, is not non-blocking.
                        ExternalScriptKind::AsapInOrder
                    } else {
                        // Step 26.f: classic, has src.
                        ExternalScriptKind::Asap
                    };

                    // Step 24.6.
                    fetch_a_classic_script(self, kind, url, cors_setting, options, encoding);

                    // Step 23.
                    match kind {
                        ExternalScriptKind::Deferred => doc.add_deferred_script(self),
                        ExternalScriptKind::ParsingBlocking => {
                            doc.set_pending_parsing_blocking_script(self, None)
                        },
                        ExternalScriptKind::AsapInOrder => doc.push_asap_in_order_script(self),
                        ExternalScriptKind::Asap => doc.add_asap_script(self),
                    }
                },
                ScriptType::Module => {
                    fetch_external_module_script(
                        ModuleOwner::Window(Trusted::new(self)),
                        url.clone(),
                        Destination::Script,
                        options,
                    );

                    if !asynch && was_parser_inserted {
                        doc.add_deferred_script(self);
                    } else if !asynch && !self.non_blocking.get() {
                        doc.push_asap_in_order_script(self);
                    } else {
                        doc.add_asap_script(self);
                    };
                },
            }
        } else {
            // Step 27.
            assert!(!text.is_empty());

            let text_rc = Rc::new(text);

            // Step 27-1. & 27-2.
            let result = Ok(ScriptOrigin::internal(
                Rc::clone(&text_rc),
                base_url.clone(),
                options.clone(),
                script_type,
            ));

            // Step 27-2.
            match script_type {
                ScriptType::Classic => {
                    if was_parser_inserted &&
                        doc.get_current_parser()
                            .map_or(false, |parser| parser.script_nesting_level() <= 1) &&
                        doc.get_script_blocking_stylesheets_count() > 0
                    {
                        // Step 27.h: classic, has no src, was parser-inserted, is blocked on stylesheet.
                        doc.set_pending_parsing_blocking_script(self, Some(result));
                    } else {
                        // Step 27.i: otherwise.
                        self.execute(result);
                    }
                },
                ScriptType::Module => {
                    // We should add inline module script elements
                    // into those vectors in case that there's no
                    // descendants in the inline module script.
                    if !asynch && was_parser_inserted {
                        doc.add_deferred_script(self);
                    } else if !asynch && !self.non_blocking.get() {
                        doc.push_asap_in_order_script(self);
                    } else {
                        doc.add_asap_script(self);
                    };

                    fetch_inline_module_script(
                        ModuleOwner::Window(Trusted::new(self)),
                        text_rc,
                        base_url.clone(),
                        self.id,
                        options,
                    );
                },
            }
        }
    }

    fn unminify_js(&self, script: &mut ScriptOrigin) {
        if !self.parser_document.window().unminify_js() {
            return;
        }

        // Write the minified code to a temporary file and pass its path as an argument
        // to js-beautify to read from. Meanwhile, redirect the process' stdout into
        // another temporary file and read that into a string. This avoids some hangs
        // observed on macOS when using direct input/output pipes with very large
        // unminified content.
        let (input, output) = (tempfile::NamedTempFile::new(), tempfile::tempfile());
        if let (Ok(mut input), Ok(mut output)) = (input, output) {
            match &script.code {
                SourceCode::Text(text) => {
                    input.write_all(text.as_bytes()).unwrap();
                },
                SourceCode::Compiled(compiled_source_code) => {
                    input
                        .write_all(compiled_source_code.original_text.as_bytes())
                        .unwrap();
                },
            }
            match Command::new("js-beautify")
                .arg(input.path())
                .stdout(output.try_clone().unwrap())
                .status()
            {
                Ok(status) if status.success() => {
                    let mut script_content = String::new();
                    output.seek(std::io::SeekFrom::Start(0)).unwrap();
                    output.read_to_string(&mut script_content).unwrap();
                    script.code = SourceCode::Text(Rc::new(DOMString::from(script_content)));
                },
                _ => {
                    warn!("Failed to execute js-beautify. Will store unmodified script");
                },
            }
        } else {
            warn!("Error creating input and output files for unminify");
        }

        let path = match window_from_node(self).unminified_js_dir() {
            Some(unminified_js_dir) => PathBuf::from(unminified_js_dir),
            None => {
                warn!("Unminified script directory not found");
                return;
            },
        };

        let (base, has_name) = match script.url.as_str().ends_with('/') {
            true => (
                path.join(&script.url[url::Position::BeforeHost..])
                    .as_path()
                    .to_owned(),
                false,
            ),
            false => (
                path.join(&script.url[url::Position::BeforeHost..])
                    .parent()
                    .unwrap()
                    .to_owned(),
                true,
            ),
        };
        match create_dir_all(base.clone()) {
            Ok(()) => debug!("Created base dir: {:?}", base),
            Err(e) => {
                debug!("Failed to create base dir: {:?}, {:?}", base, e);
                return;
            },
        }
        let path = if script.external && has_name {
            // External script.
            path.join(&script.url[url::Position::BeforeHost..])
        } else {
            // Inline script or url ends with '/'
            base.join(Uuid::new_v4().to_string())
        };

        debug!("script will be stored in {:?}", path);

        match File::create(&path) {
            Ok(mut file) => match &script.code {
                SourceCode::Text(text) => file.write_all(text.as_bytes()).unwrap(),
                SourceCode::Compiled(compiled_source_code) => {
                    file.write_all(compiled_source_code.original_text.as_bytes())
                        .unwrap();
                },
            },
            Err(why) => warn!("Could not store script {:?}", why),
        }
    }

    fn substitute_with_local_script(&self, script: &mut ScriptOrigin) {
        if self
            .parser_document
            .window()
            .local_script_source()
            .is_none() ||
            !script.external
        {
            return;
        }
        let mut path = PathBuf::from(
            self.parser_document
                .window()
                .local_script_source()
                .clone()
                .unwrap(),
        );
        path = path.join(&script.url[url::Position::BeforeHost..]);
        debug!("Attempting to read script stored at: {:?}", path);
        match read_to_string(path.clone()) {
            Ok(local_script) => {
                debug!("Found script stored at: {:?}", path);
                script.code = SourceCode::Text(Rc::new(DOMString::from(local_script)));
            },
            Err(why) => warn!("Could not restore script from file {:?}", why),
        }
    }

    /// <https://html.spec.whatwg.org/multipage/#execute-the-script-block>
    pub fn execute(&self, result: ScriptResult) {
        // Step 1.
        let doc = document_from_node(self);
        if self.parser_inserted.get() && *doc != *self.parser_document {
            return;
        }

        let mut script = match result {
            // Step 2.
            Err(e) => {
                warn!("error loading script {:?}", e);
                self.dispatch_error_event();
                return;
            },

            Ok(script) => script,
        };

        if script.type_ == ScriptType::Classic {
            self.unminify_js(&mut script);
            self.substitute_with_local_script(&mut script);
        }

        // Step 3.
        let neutralized_doc = if script.external || script.type_ == ScriptType::Module {
            debug!("loading external script, url = {}", script.url);
            let doc = document_from_node(self);
            doc.incr_ignore_destructive_writes_counter();
            Some(doc)
        } else {
            None
        };

        // Step 4.
        let document = document_from_node(self);
        let old_script = document.GetCurrentScript();

        match script.type_ {
            ScriptType::Classic => document.set_current_script(Some(self)),
            ScriptType::Module => document.set_current_script(None),
        }

        match script.type_ {
            ScriptType::Classic => {
                self.run_a_classic_script(&script);
                document.set_current_script(old_script.as_deref());
            },
            ScriptType::Module => {
                assert!(document.GetCurrentScript().is_none());
                self.run_a_module_script(&script, false);
            },
        }

        // Step 5.
        if let Some(doc) = neutralized_doc {
            doc.decr_ignore_destructive_writes_counter();
        }

        // Step 6.
        if script.external {
            self.dispatch_load_event();
        }
    }

    // https://html.spec.whatwg.org/multipage/#run-a-classic-script
    pub fn run_a_classic_script(&self, script: &ScriptOrigin) {
        // TODO use a settings object rather than this element's document/window
        // Step 2
        let document = document_from_node(self);
        if !document.is_fully_active() || !document.is_scripting_enabled() {
            return;
        }

        // Steps 4-10
        let window = window_from_node(self);
        let line_number = if script.external {
            1
        } else {
            self.line_number as u32
        };
        rooted!(in(*GlobalScope::get_cx()) let mut rval = UndefinedValue());
        let global = window.upcast::<GlobalScope>();
        global.evaluate_script_on_global_with_result(
            &script.code,
            script.url.as_str(),
            rval.handle_mut(),
            line_number,
            script.fetch_options.clone(),
            script.url.clone(),
        );
    }

    #[allow(unsafe_code)]
    /// <https://html.spec.whatwg.org/multipage/#run-a-module-script>
    pub fn run_a_module_script(&self, script: &ScriptOrigin, _rethrow_errors: bool) {
        // TODO use a settings object rather than this element's document/window
        // Step 2
        let document = document_from_node(self);
        if !document.is_fully_active() || !document.is_scripting_enabled() {
            return;
        }

        // Step 4
        let window = window_from_node(self);
        let global = window.upcast::<GlobalScope>();
        let _aes = AutoEntryScript::new(global);

        let tree = if script.external {
            global.get_module_map().borrow().get(&script.url).cloned()
        } else {
            global
                .get_inline_module_map()
                .borrow()
                .get(&self.id.clone())
                .cloned()
        };

        if let Some(module_tree) = tree {
            // Step 6.
            {
                let module_error = module_tree.get_rethrow_error().borrow();
                let network_error = module_tree.get_network_error().borrow();
                if module_error.is_some() && network_error.is_none() {
                    module_tree.report_error(global);
                    return;
                }
            }

            let record = module_tree
                .get_record()
                .borrow()
                .as_ref()
                .map(|record| record.handle());

            if let Some(record) = record {
                rooted!(in(*GlobalScope::get_cx()) let mut rval = UndefinedValue());
                let evaluated =
                    module_tree.execute_module(global, record, rval.handle_mut().into());

                if let Err(exception) = evaluated {
                    module_tree.set_rethrow_error(exception);
                    module_tree.report_error(global);
                }
            }
        }
    }

    pub fn queue_error_event(&self) {
        let window = window_from_node(self);
        window
            .task_manager()
            .dom_manipulation_task_source()
            .queue_simple_event(self.upcast(), atom!("error"), &window);
    }

    pub fn dispatch_load_event(&self) {
        self.dispatch_event(
            atom!("load"),
            EventBubbles::DoesNotBubble,
            EventCancelable::NotCancelable,
        );
    }

    pub fn dispatch_error_event(&self) {
        self.dispatch_event(
            atom!("error"),
            EventBubbles::DoesNotBubble,
            EventCancelable::NotCancelable,
        );
    }

    // https://html.spec.whatwg.org/multipage/#prepare-a-script Step 7.
    pub fn get_script_type(&self) -> Option<ScriptType> {
        let element = self.upcast::<Element>();

        let type_attr = element.get_attribute(&ns!(), &local_name!("type"));
        let language_attr = element.get_attribute(&ns!(), &local_name!("language"));

        let script_type = match (
            type_attr.as_ref().map(|t| t.value()),
            language_attr.as_ref().map(|l| l.value()),
        ) {
            (Some(ref ty), _) if ty.is_empty() => {
                debug!("script type empty, inferring js");
                Some(ScriptType::Classic)
            },
            (None, Some(ref lang)) if lang.is_empty() => {
                debug!("script type empty, inferring js");
                Some(ScriptType::Classic)
            },
            (None, None) => {
                debug!("script type empty, inferring js");
                Some(ScriptType::Classic)
            },
            (None, Some(ref lang)) => {
                debug!("script language={}", &***lang);
                let language = format!("text/{}", &***lang);

                if SCRIPT_JS_MIMES.contains(&language.to_ascii_lowercase().as_str()) {
                    Some(ScriptType::Classic)
                } else {
                    None
                }
            },
            (Some(ref ty), _) => {
                debug!("script type={}", &***ty);

                if ty.to_ascii_lowercase().trim_matches(HTML_SPACE_CHARACTERS) == "module" {
                    return Some(ScriptType::Module);
                }

                if SCRIPT_JS_MIMES
                    .contains(&ty.to_ascii_lowercase().trim_matches(HTML_SPACE_CHARACTERS))
                {
                    Some(ScriptType::Classic)
                } else {
                    None
                }
            },
        };

        // https://github.com/rust-lang/rust/issues/21114
        script_type
    }

    pub fn set_parser_inserted(&self, parser_inserted: bool) {
        self.parser_inserted.set(parser_inserted);
    }

    pub fn get_parser_inserted(&self) -> bool {
        self.parser_inserted.get()
    }

    pub fn set_already_started(&self, already_started: bool) {
        self.already_started.set(already_started);
    }

    pub fn get_non_blocking(&self) -> bool {
        self.non_blocking.get()
    }

    fn dispatch_event(
        &self,
        type_: Atom,
        bubbles: EventBubbles,
        cancelable: EventCancelable,
    ) -> EventStatus {
        let window = window_from_node(self);
        let event = Event::new(window.upcast(), type_, bubbles, cancelable);
        event.fire(self.upcast())
    }
}

impl VirtualMethods for HTMLScriptElement {
    fn super_type(&self) -> Option<&dyn VirtualMethods> {
        Some(self.upcast::<HTMLElement>() as &dyn VirtualMethods)
    }

    fn attribute_mutated(&self, attr: &Attr, mutation: AttributeMutation) {
        self.super_type().unwrap().attribute_mutated(attr, mutation);
        if *attr.local_name() == local_name!("src") {
            if let AttributeMutation::Set(_) = mutation {
                if !self.parser_inserted.get() && self.upcast::<Node>().is_connected() {
                    self.prepare();
                }
            }
        }
    }

    fn children_changed(&self, mutation: &ChildrenMutation) {
        if let Some(s) = self.super_type() {
            s.children_changed(mutation);
        }
        if !self.parser_inserted.get() && self.upcast::<Node>().is_connected() {
            self.prepare();
        }
    }

    fn bind_to_tree(&self, context: &BindContext) {
        if let Some(s) = self.super_type() {
            s.bind_to_tree(context);
        }

        if context.tree_connected && !self.parser_inserted.get() {
            let script = Trusted::new(self);
            document_from_node(self).add_delayed_task(task!(ScriptDelayedInitialize: move || {
                script.root().prepare();
            }));
        }
    }

    fn cloning_steps(
        &self,
        copy: &Node,
        maybe_doc: Option<&Document>,
        clone_children: CloneChildrenFlag,
    ) {
        if let Some(s) = self.super_type() {
            s.cloning_steps(copy, maybe_doc, clone_children);
        }

        // https://html.spec.whatwg.org/multipage/#already-started
        if self.already_started.get() {
            copy.downcast::<HTMLScriptElement>()
                .unwrap()
                .set_already_started(true);
        }
    }
}

impl HTMLScriptElementMethods for HTMLScriptElement {
    // https://html.spec.whatwg.org/multipage/#dom-script-src
    make_url_getter!(Src, "src");

    // https://html.spec.whatwg.org/multipage/#dom-script-src
    make_url_setter!(SetSrc, "src");

    // https://html.spec.whatwg.org/multipage/#dom-script-type
    make_getter!(Type, "type");
    // https://html.spec.whatwg.org/multipage/#dom-script-type
    make_setter!(SetType, "type");

    // https://html.spec.whatwg.org/multipage/#dom-script-charset
    make_getter!(Charset, "charset");
    // https://html.spec.whatwg.org/multipage/#dom-script-charset
    make_setter!(SetCharset, "charset");

    // https://html.spec.whatwg.org/multipage/#dom-script-async
    fn Async(&self) -> bool {
        self.non_blocking.get() ||
            self.upcast::<Element>()
                .has_attribute(&local_name!("async"))
    }

    // https://html.spec.whatwg.org/multipage/#dom-script-async
    fn SetAsync(&self, value: bool) {
        self.non_blocking.set(false);
        self.upcast::<Element>()
            .set_bool_attribute(&local_name!("async"), value);
    }

    // https://html.spec.whatwg.org/multipage/#dom-script-defer
    make_bool_getter!(Defer, "defer");
    // https://html.spec.whatwg.org/multipage/#dom-script-defer
    make_bool_setter!(SetDefer, "defer");

    // https://html.spec.whatwg.org/multipage/#dom-script-nomodule
    make_bool_getter!(NoModule, "nomodule");
    // https://html.spec.whatwg.org/multipage/#dom-script-nomodule
    make_bool_setter!(SetNoModule, "nomodule");

    // https://html.spec.whatwg.org/multipage/#dom-script-integrity
    make_getter!(Integrity, "integrity");
    // https://html.spec.whatwg.org/multipage/#dom-script-integrity
    make_setter!(SetIntegrity, "integrity");

    // https://html.spec.whatwg.org/multipage/#dom-script-event
    make_getter!(Event, "event");
    // https://html.spec.whatwg.org/multipage/#dom-script-event
    make_setter!(SetEvent, "event");

    // https://html.spec.whatwg.org/multipage/#dom-script-htmlfor
    make_getter!(HtmlFor, "for");
    // https://html.spec.whatwg.org/multipage/#dom-script-htmlfor
    make_setter!(SetHtmlFor, "for");

    // https://html.spec.whatwg.org/multipage/#dom-script-crossorigin
    fn GetCrossOrigin(&self) -> Option<DOMString> {
        reflect_cross_origin_attribute(self.upcast::<Element>())
    }

    // https://html.spec.whatwg.org/multipage/#dom-script-crossorigin
    fn SetCrossOrigin(&self, value: Option<DOMString>) {
        set_cross_origin_attribute(self.upcast::<Element>(), value);
    }

    // https://html.spec.whatwg.org/multipage/#dom-script-referrerpolicy
    fn ReferrerPolicy(&self) -> DOMString {
        reflect_referrer_policy_attribute(self.upcast::<Element>())
    }

    // https://html.spec.whatwg.org/multipage/#dom-script-referrerpolicy
    make_setter!(SetReferrerPolicy, "referrerpolicy");

    // https://html.spec.whatwg.org/multipage/#dom-script-text
    fn Text(&self) -> DOMString {
        self.upcast::<Node>().child_text_content()
    }

    // https://html.spec.whatwg.org/multipage/#dom-script-text
    fn SetText(&self, value: DOMString) {
        self.upcast::<Node>().SetTextContent(Some(value))
    }
}

#[derive(Clone, Copy)]
enum ExternalScriptKind {
    Deferred,
    ParsingBlocking,
    AsapInOrder,
    Asap,
}

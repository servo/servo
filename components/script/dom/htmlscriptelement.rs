/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

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
use crate::dom::document::Document;
use crate::dom::element::{
    cors_setting_for_element, reflect_cross_origin_attribute, set_cross_origin_attribute,
};
use crate::dom::element::{AttributeMutation, Element, ElementCreator};
use crate::dom::event::{Event, EventBubbles, EventCancelable, EventStatus};
use crate::dom::globalscope::GlobalScope;
use crate::dom::htmlelement::HTMLElement;
use crate::dom::node::{document_from_node, window_from_node};
use crate::dom::node::{BindContext, ChildrenMutation, CloneChildrenFlag, Node};
use crate::dom::performanceresourcetiming::InitiatorType;
use crate::dom::virtualmethods::VirtualMethods;
use crate::fetch::create_a_potential_cors_request;
use crate::network_listener::{self, NetworkListener, PreInvoke, ResourceTimingListener};
use crate::script_module::fetch_inline_module_script;
use crate::script_module::{fetch_external_module_script, ModuleOwner};
use content_security_policy as csp;
use dom_struct::dom_struct;
use encoding_rs::Encoding;
use html5ever::{LocalName, Prefix};
use ipc_channel::ipc;
use ipc_channel::router::ROUTER;
use js::jsval::UndefinedValue;
use msg::constellation_msg::PipelineId;
use net_traits::request::{CorsSettings, CredentialsMode, Destination, Referrer, RequestBuilder};
use net_traits::ReferrerPolicy;
use net_traits::{FetchMetadata, FetchResponseListener, Metadata, NetworkError};
use net_traits::{ResourceFetchTiming, ResourceTimingType};
use servo_atoms::Atom;
use servo_url::ImmutableOrigin;
use servo_url::ServoUrl;
use std::cell::Cell;
use std::fs::File;
use std::io::{Read, Write};
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};
use style::str::{StaticStringVec, HTML_SPACE_CHARACTERS};
use uuid::Uuid;

/// An unique id for script element.
#[derive(Clone, Copy, Debug, Eq, Hash, JSTraceable, PartialEq)]
pub struct ScriptId(Uuid);

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

    #[allow(unrooted_must_root)]
    pub fn new(
        local_name: LocalName,
        prefix: Option<Prefix>,
        document: &Document,
        creator: ElementCreator,
    ) -> DomRoot<HTMLScriptElement> {
        Node::reflect_node(
            Box::new(HTMLScriptElement::new_inherited(
                local_name, prefix, document, creator,
            )),
            document,
        )
    }

    pub fn get_script_id(&self) -> ScriptId {
        self.id.clone()
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
pub struct ScriptOrigin {
    text: DOMString,
    url: ServoUrl,
    external: bool,
    type_: ScriptType,
}

impl ScriptOrigin {
    pub fn internal(text: DOMString, url: ServoUrl, type_: ScriptType) -> ScriptOrigin {
        ScriptOrigin {
            text: text,
            url: url,
            external: false,
            type_,
        }
    }

    pub fn external(text: DOMString, url: ServoUrl, type_: ScriptType) -> ScriptOrigin {
        ScriptOrigin {
            text: text,
            url: url,
            external: true,
            type_,
        }
    }

    pub fn text(&self) -> DOMString {
        self.text.clone()
    }
}

pub type ScriptResult = Result<ScriptOrigin, NetworkError>;

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
    fn process_response_eof(&mut self, response: Result<ResourceFetchTiming, NetworkError>) {
        // Step 5.
        let load = response.and(self.status.clone()).map(|_| {
            let metadata = self.metadata.take().unwrap();

            // Step 6.
            let encoding = metadata
                .charset
                .and_then(|encoding| Encoding::for_label(encoding.as_bytes()))
                .unwrap_or(self.character_encoding);

            // Step 7.
            let (source_text, _, _) = encoding.decode(&self.data);
            ScriptOrigin::external(
                DOMString::from(source_text),
                metadata.final_url,
                ScriptType::Classic,
            )
        });

        // Step 9.
        // https://html.spec.whatwg.org/multipage/#prepare-a-script
        // Step 18.6 (When the chosen algorithm asynchronously completes).
        let elem = self.elem.root();
        let document = document_from_node(&*elem);

        match self.kind {
            ExternalScriptKind::Asap => document.asap_script_loaded(&elem, load),
            ExternalScriptKind::AsapInOrder => document.asap_in_order_script_loaded(&elem, load),
            ExternalScriptKind::Deferred => document.deferred_script_loaded(&elem, load),
            ExternalScriptKind::ParsingBlocking => {
                document.pending_parsing_blocking_script_loaded(&elem, load)
            },
        }

        document.finish_load(LoadType::Script(self.url.clone()));
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
    referrer: Referrer,
    referrer_policy: Option<ReferrerPolicy>,
    integrity_metadata: String,
) -> RequestBuilder {
    create_a_potential_cors_request(url, Destination::Script, cors_setting, None)
        .origin(origin)
        .pipeline_id(Some(pipeline_id))
        .referrer(Some(referrer))
        .referrer_policy(referrer_policy)
        .integrity_metadata(integrity_metadata)
}

/// <https://html.spec.whatwg.org/multipage/#fetch-a-classic-script>
fn fetch_a_classic_script(
    script: &HTMLScriptElement,
    kind: ExternalScriptKind,
    url: ServoUrl,
    cors_setting: Option<CorsSettings>,
    integrity_metadata: String,
    character_encoding: &'static Encoding,
) {
    let doc = document_from_node(script);

    // Step 1, 2.
    let request = script_fetch_request(
        url.clone(),
        cors_setting,
        doc.origin().immutable().clone(),
        script.global().pipeline_id(),
        Referrer::ReferrerUrl(doc.url()),
        doc.get_referrer_policy(),
        integrity_metadata,
    );

    // TODO: Step 3, Add custom steps to perform fetch

    let context = Arc::new(Mutex::new(ClassicContext {
        elem: Trusted::new(script),
        kind: kind,
        character_encoding: character_encoding,
        data: vec![],
        metadata: None,
        url: url.clone(),
        status: Ok(()),
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

        // Step 3.
        let element = self.upcast::<Element>();
        let r#async = element.has_attribute(&local_name!("async"));
        // Note: confusingly, this is done if the element does *not* have an "async" attribute.
        if was_parser_inserted && !r#async {
            self.non_blocking.set(true);
        }

        // Step 4-5.
        let text = self.Text();
        if text.is_empty() && !element.has_attribute(&local_name!("src")) {
            return;
        }

        // Step 6.
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

        // Step 9.
        self.already_started.set(true);

        // Step 10.
        let doc = document_from_node(self);
        if self.parser_inserted.get() && &*self.parser_document != &*doc {
            return;
        }

        // Step 11.
        if !doc.is_scripting_enabled() {
            return;
        }

        // Step 12
        if element.has_attribute(&local_name!("nomodule")) && script_type == ScriptType::Classic {
            return;
        }

        // Step 13.
        if !element.has_attribute(&local_name!("src")) &&
            doc.should_elements_inline_type_behavior_be_blocked(
                &element,
                csp::InlineCheckType::Script,
                &text,
            ) == csp::CheckResult::Blocked
        {
            return;
        }

        // Step 14.
        if script_type == ScriptType::Classic {
            let for_attribute = element.get_attribute(&ns!(), &local_name!("for"));
            let event_attribute = element.get_attribute(&ns!(), &local_name!("event"));
            match (for_attribute, event_attribute) {
                (Some(ref for_attribute), Some(ref event_attribute)) => {
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
                },
                (_, _) => (),
            }
        }

        // Step 15.
        let encoding = element
            .get_attribute(&ns!(), &local_name!("charset"))
            .and_then(|charset| Encoding::for_label(charset.value().as_bytes()))
            .unwrap_or_else(|| doc.encoding());

        // Step 16.
        let cors_setting = cors_setting_for_element(element);

        // Step 17.
        let credentials_mode = match script_type {
            ScriptType::Classic => None,
            ScriptType::Module => Some(reflect_cross_origin_attribute(element).map_or(
                CredentialsMode::CredentialsSameOrigin,
                |attr| match &*attr {
                    "use-credentials" => CredentialsMode::Include,
                    "anonymous" => CredentialsMode::CredentialsSameOrigin,
                    _ => CredentialsMode::CredentialsSameOrigin,
                },
            )),
        };

        // TODO: Step 18: Nonce.

        // Step 19: Integrity metadata.
        let im_attribute = element.get_attribute(&ns!(), &local_name!("integrity"));
        let integrity_val = im_attribute.as_ref().map(|a| a.value());
        let integrity_metadata = match integrity_val {
            Some(ref value) => &***value,
            None => "",
        };

        // TODO: Step 20: referrer policy

        // TODO: Step 21: parser state.

        // TODO: Step 22: Fetch options

        // TODO: Step 23: environment settings object.

        let base_url = doc.base_url();
        if let Some(src) = element.get_attribute(&ns!(), &local_name!("src")) {
            // Step 24.

            // Step 24.1.
            let src = src.value();

            // Step 24.2.
            if src.is_empty() {
                self.queue_error_event();
                return;
            }

            // Step 24.3: The "from an external file"" flag is stored in ScriptOrigin.

            // Step 24.4-24.5.
            let url = match base_url.join(&src) {
                Ok(url) => url,
                Err(_) => {
                    warn!("error parsing URL for script {}", &**src);
                    self.queue_error_event();
                    return;
                },
            };

            // Step 24.6.
            match script_type {
                ScriptType::Classic => {
                    // Preparation for step 26.
                    let kind = if element.has_attribute(&local_name!("defer")) &&
                        was_parser_inserted &&
                        !r#async
                    {
                        // Step 26.a: classic, has src, has defer, was parser-inserted, is not async.
                        ExternalScriptKind::Deferred
                    } else if was_parser_inserted && !r#async {
                        // Step 26.c: classic, has src, was parser-inserted, is not async.
                        ExternalScriptKind::ParsingBlocking
                    } else if !r#async && !self.non_blocking.get() {
                        // Step 26.d: classic, has src, is not async, is not non-blocking.
                        ExternalScriptKind::AsapInOrder
                    } else {
                        // Step 26.f: classic, has src.
                        ExternalScriptKind::Asap
                    };

                    // Step 24.6.
                    fetch_a_classic_script(
                        self,
                        kind,
                        url,
                        cors_setting,
                        integrity_metadata.to_owned(),
                        encoding,
                    );

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
                        integrity_metadata.to_owned(),
                        credentials_mode.unwrap(),
                    );

                    if !r#async && was_parser_inserted {
                        doc.add_deferred_script(self);
                    } else if !r#async && !self.non_blocking.get() {
                        doc.push_asap_in_order_script(self);
                    } else {
                        doc.add_asap_script(self);
                    };
                },
            }
        } else {
            // Step 25.
            assert!(!text.is_empty());

            // Step 25-1. & 25-2.
            let result = Ok(ScriptOrigin::internal(
                text.clone(),
                base_url.clone(),
                script_type.clone(),
            ));

            // Step 25-2.
            match script_type {
                ScriptType::Classic => {
                    if was_parser_inserted &&
                        doc.get_current_parser()
                            .map_or(false, |parser| parser.script_nesting_level() <= 1) &&
                        doc.get_script_blocking_stylesheets_count() > 0
                    {
                        // Step 26.h: classic, has no src, was parser-inserted, is blocked on stylesheet.
                        doc.set_pending_parsing_blocking_script(self, Some(result));
                    } else {
                        // Step 26.i: otherwise.
                        self.execute(result);
                    }
                },
                ScriptType::Module => {
                    // We should add inline module script elements
                    // into those vectors in case that there's no
                    // descendants in the inline module script.
                    if !r#async && was_parser_inserted {
                        doc.add_deferred_script(self);
                    } else if !r#async && !self.non_blocking.get() {
                        doc.push_asap_in_order_script(self);
                    } else {
                        doc.add_asap_script(self);
                    };

                    fetch_inline_module_script(
                        ModuleOwner::Window(Trusted::new(self)),
                        text.clone(),
                        base_url.clone(),
                        self.id.clone(),
                        credentials_mode.unwrap(),
                    );
                },
            }
        }
    }

    fn unminify_js(&self, script: &mut ScriptOrigin) {
        if !self.parser_document.window().unminify_js() {
            return;
        }

        match Command::new("js-beautify")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
        {
            Err(_) => {
                warn!("Failed to execute js-beautify. Will store unmodified script");
            },
            Ok(process) => {
                let mut script_content = String::from(script.text.clone());
                let _ = process.stdin.unwrap().write_all(script_content.as_bytes());
                script_content.clear();
                let _ = process.stdout.unwrap().read_to_string(&mut script_content);

                script.text = DOMString::from(script_content);
            },
        }

        let path;
        match window_from_node(self).unminified_js_dir() {
            Some(unminified_js_dir) => path = PathBuf::from(unminified_js_dir),
            None => {
                warn!("Unminified script directory not found");
                return;
            },
        }

        let path = if script.external {
            // External script.
            let path_parts = script.url.path_segments().unwrap();
            match path_parts.last() {
                Some(script_name) => path.join(script_name),
                None => path.join(Uuid::new_v4().to_string()),
            }
        } else {
            // Inline script.
            path.join(Uuid::new_v4().to_string())
        };

        debug!("script will be stored in {:?}", path);

        match File::create(&path) {
            Ok(mut file) => file.write_all(script.text.as_bytes()).unwrap(),
            Err(why) => warn!("Could not store script {:?}", why),
        }
    }

    /// <https://html.spec.whatwg.org/multipage/#execute-the-script-block>
    pub fn execute(&self, result: ScriptResult) {
        // Step 1.
        let doc = document_from_node(self);
        if self.parser_inserted.get() && &*doc != &*self.parser_document {
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
            ScriptType::Classic => {
                document.set_current_script(Some(self));
                self.run_a_classic_script(&script);
                document.set_current_script(old_script.as_deref());
            },
            ScriptType::Module => {
                assert!(old_script.is_none());
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
        rooted!(in(*window.get_cx()) let mut rval = UndefinedValue());
        let global = window.upcast::<GlobalScope>();
        global.evaluate_script_on_global_with_result(
            &script.text,
            script.url.as_str(),
            rval.handle_mut(),
            line_number,
        );
    }

    #[allow(unsafe_code)]
    /// https://html.spec.whatwg.org/multipage/#run-a-module-script
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
        let _aes = AutoEntryScript::new(&global);

        if script.external {
            let module_map = global.get_module_map().borrow();

            if let Some(module_tree) = module_map.get(&script.url) {
                // Step 6.
                {
                    let module_error = module_tree.get_error().borrow();
                    if module_error.is_some() {
                        module_tree.report_error(&global);
                        return;
                    }
                }

                let module_record = module_tree.get_record().borrow();
                if let Some(record) = &*module_record {
                    let evaluated = module_tree.execute_module(global, record.handle());

                    if let Err(exception) = evaluated {
                        module_tree.set_error(Some(exception.clone()));
                        module_tree.report_error(&global);
                        return;
                    }
                }
            }
        } else {
            let inline_module_map = global.get_inline_module_map().borrow();

            if let Some(module_tree) = inline_module_map.get(&self.id.clone()) {
                // Step 6.
                {
                    let module_error = module_tree.get_error().borrow();
                    if module_error.is_some() {
                        module_tree.report_error(&global);
                        return;
                    }
                }

                let module_record = module_tree.get_record().borrow();
                if let Some(record) = &*module_record {
                    let evaluated = module_tree.execute_module(global, record.handle());

                    if let Err(exception) = evaluated {
                        module_tree.set_error(Some(exception.clone()));
                        module_tree.report_error(&global);
                        return;
                    }
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

                if &***ty == String::from("module") {
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
        match *attr.local_name() {
            local_name!("src") => {
                if let AttributeMutation::Set(_) = mutation {
                    if !self.parser_inserted.get() && self.upcast::<Node>().is_connected() {
                        self.prepare();
                    }
                }
            },
            _ => {},
        }
    }

    fn children_changed(&self, mutation: &ChildrenMutation) {
        if let Some(ref s) = self.super_type() {
            s.children_changed(mutation);
        }
        if !self.parser_inserted.get() && self.upcast::<Node>().is_connected() {
            self.prepare();
        }
    }

    fn bind_to_tree(&self, context: &BindContext) {
        if let Some(ref s) = self.super_type() {
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
        if let Some(ref s) = self.super_type() {
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

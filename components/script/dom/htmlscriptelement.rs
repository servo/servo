/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use document_loader::LoadType;
use dom::attr::Attr;
use dom::bindings::codegen::Bindings::DocumentBinding::DocumentMethods;
use dom::bindings::codegen::Bindings::HTMLScriptElementBinding;
use dom::bindings::codegen::Bindings::HTMLScriptElementBinding::HTMLScriptElementMethods;
use dom::bindings::codegen::Bindings::NodeBinding::NodeMethods;
use dom::bindings::inheritance::Castable;
use dom::bindings::js::{JS, Root};
use dom::bindings::js::RootedReference;
use dom::bindings::refcounted::Trusted;
use dom::bindings::reflector::DomObject;
use dom::bindings::str::DOMString;
use dom::document::Document;
use dom::element::{AttributeMutation, Element, ElementCreator};
use dom::element::{cors_setting_for_element, reflect_cross_origin_attribute, set_cross_origin_attribute};
use dom::event::{Event, EventBubbles, EventCancelable, EventStatus};
use dom::globalscope::GlobalScope;
use dom::htmlelement::HTMLElement;
use dom::node::{ChildrenMutation, CloneChildrenFlag, Node};
use dom::node::{document_from_node, window_from_node};
use dom::virtualmethods::VirtualMethods;
use dom_struct::dom_struct;
use encoding::label::encoding_from_whatwg_label;
use encoding::types::{DecoderTrap, EncodingRef};
use html5ever::{LocalName, Prefix};
use ipc_channel::ipc;
use ipc_channel::router::ROUTER;
use js::jsval::UndefinedValue;
use net_traits::{FetchMetadata, FetchResponseListener, Metadata, NetworkError};
use net_traits::request::{CorsSettings, CredentialsMode, Destination, RequestInit, RequestMode, Type as RequestType};
use network_listener::{NetworkListener, PreInvoke};
use servo_atoms::Atom;
use servo_config::opts;
use servo_url::ServoUrl;
use std::ascii::AsciiExt;
use std::cell::Cell;
use std::fs::File;
use std::io::{Read, Write};
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};
use style::str::{HTML_SPACE_CHARACTERS, StaticStringVec};
use uuid::Uuid;

#[dom_struct]
pub struct HTMLScriptElement {
    htmlelement: HTMLElement,

    /// https://html.spec.whatwg.org/multipage/#already-started
    already_started: Cell<bool>,

    /// https://html.spec.whatwg.org/multipage/#parser-inserted
    parser_inserted: Cell<bool>,

    /// https://html.spec.whatwg.org/multipage/#non-blocking
    ///
    /// (currently unused)
    non_blocking: Cell<bool>,

    /// Document of the parser that created this element
    parser_document: JS<Document>,

    /// Track line line_number
    line_number: u64,
}

impl HTMLScriptElement {
    fn new_inherited(local_name: LocalName, prefix: Option<Prefix>, document: &Document,
                     creator: ElementCreator) -> HTMLScriptElement {
        HTMLScriptElement {
            htmlelement:
                HTMLElement::new_inherited(local_name, prefix, document),
            already_started: Cell::new(false),
            parser_inserted: Cell::new(creator.is_parser_created()),
            non_blocking: Cell::new(!creator.is_parser_created()),
            parser_document: JS::from_ref(document),
            line_number: creator.return_line_number(),
        }
    }

    #[allow(unrooted_must_root)]
    pub fn new(local_name: LocalName, prefix: Option<Prefix>, document: &Document,
               creator: ElementCreator) -> Root<HTMLScriptElement> {
        Node::reflect_node(box HTMLScriptElement::new_inherited(local_name, prefix, document, creator),
                           document,
                           HTMLScriptElementBinding::Wrap)
    }
}


/// Supported script types as defined by
/// <https://html.spec.whatwg.org/multipage/#javascript-mime-type>.
static SCRIPT_JS_MIMES: StaticStringVec = &[
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

#[derive(HeapSizeOf, JSTraceable)]
pub struct ClassicScript {
    text: DOMString,
    url: ServoUrl,
    external: bool,
}

impl ClassicScript {
    fn internal(text: DOMString, url: ServoUrl) -> ClassicScript {
        ClassicScript {
            text: text,
            url: url,
            external: false,
        }
    }

    fn external(text: DOMString, url: ServoUrl) -> ClassicScript {
        ClassicScript {
            text: text,
            url: url,
            external: true,
        }
    }
}

pub type ScriptResult = Result<ClassicScript, NetworkError>;

/// The context required for asynchronously loading an external script source.
struct ScriptContext {
    /// The element that initiated the request.
    elem: Trusted<HTMLScriptElement>,
    /// The kind of external script.
    kind: ExternalScriptKind,
    /// The (fallback) character encoding argument to the "fetch a classic
    /// script" algorithm.
    character_encoding: EncodingRef,
    /// The response body received to date.
    data: Vec<u8>,
    /// The response metadata received to date.
    metadata: Option<Metadata>,
    /// The initial URL requested.
    url: ServoUrl,
    /// Indicates whether the request failed, and why
    status: Result<(), NetworkError>
}

impl FetchResponseListener for ScriptContext {
    fn process_request_body(&mut self) {} // TODO(KiChjang): Perhaps add custom steps to perform fetch here?

    fn process_request_eof(&mut self) {} // TODO(KiChjang): Perhaps add custom steps to perform fetch here?

    fn process_response(&mut self,
                        metadata: Result<FetchMetadata, NetworkError>) {
        self.metadata = metadata.ok().map(|meta| match meta {
            FetchMetadata::Unfiltered(m) => m,
            FetchMetadata::Filtered { unsafe_, .. } => unsafe_
        });

        let status_code = self.metadata.as_ref().and_then(|m| {
            match m.status {
                Some((c, _)) => Some(c),
                _ => None,
            }
        }).unwrap_or(0);

        self.status = match status_code {
            0 => Err(NetworkError::Internal("No http status code received".to_owned())),
            200...299 => Ok(()), // HTTP ok status codes
            _ => Err(NetworkError::Internal(format!("HTTP error code {}", status_code)))
        };
    }

    fn process_response_chunk(&mut self, mut chunk: Vec<u8>) {
        if self.status.is_ok() {
            self.data.append(&mut chunk);
        }
    }

    /// https://html.spec.whatwg.org/multipage/#fetch-a-classic-script
    /// step 4-9
    fn process_response_eof(&mut self, response: Result<(), NetworkError>) {
        // Step 5.
        let load = response.and(self.status.clone()).map(|_| {
            let metadata = self.metadata.take().unwrap();

            // Step 6.
            let encoding = metadata.charset
                .and_then(|encoding| encoding_from_whatwg_label(&encoding))
                .unwrap_or(self.character_encoding);

            // Step 7.
            let source_text = encoding.decode(&self.data, DecoderTrap::Replace).unwrap();
            ClassicScript::external(DOMString::from(source_text), metadata.final_url)
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
            ExternalScriptKind::ParsingBlocking => document.pending_parsing_blocking_script_loaded(&elem, load),
        }

        document.finish_load(LoadType::Script(self.url.clone()));
    }
}

impl PreInvoke for ScriptContext {}

/// https://html.spec.whatwg.org/multipage/#fetch-a-classic-script
fn fetch_a_classic_script(script: &HTMLScriptElement,
                          kind: ExternalScriptKind,
                          url: ServoUrl,
                          cors_setting: Option<CorsSettings>,
                          integrity_metadata: String,
                          character_encoding: EncodingRef) {
    let doc = document_from_node(script);

    // Step 1, 2.
    let request = RequestInit {
        url: url.clone(),
        type_: RequestType::Script,
        destination: Destination::Script,
        // https://html.spec.whatwg.org/multipage/#create-a-potential-cors-request
        // Step 1
        mode: match cors_setting {
            Some(_) => RequestMode::CorsMode,
            None => RequestMode::NoCors,
        },
        // https://html.spec.whatwg.org/multipage/#create-a-potential-cors-request
        // Step 3-4
        credentials_mode: match cors_setting {
            Some(CorsSettings::Anonymous) => CredentialsMode::CredentialsSameOrigin,
            _ => CredentialsMode::Include,
        },
        origin: doc.url(),
        pipeline_id: Some(script.global().pipeline_id()),
        referrer_url: Some(doc.url()),
        referrer_policy: doc.get_referrer_policy(),
        integrity_metadata: integrity_metadata,
        .. RequestInit::default()
    };

    // TODO: Step 3, Add custom steps to perform fetch

    let context = Arc::new(Mutex::new(ScriptContext {
        elem: Trusted::new(script),
        kind: kind,
        character_encoding: character_encoding,
        data: vec!(),
        metadata: None,
        url: url.clone(),
        status: Ok(())
    }));

    let (action_sender, action_receiver) = ipc::channel().unwrap();
    let listener = NetworkListener {
        context: context,
        task_source: doc.window().networking_task_source(),
        wrapper: Some(doc.window().get_runnable_wrapper())
    };

    ROUTER.add_route(action_receiver.to_opaque(), box move |message| {
        listener.notify_fetch(message.to().unwrap());
    });
    doc.fetch_async(LoadType::Script(url), request, action_sender);
}

impl HTMLScriptElement {
    /// https://html.spec.whatwg.org/multipage/#prepare-a-script
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
        let async = element.has_attribute(&local_name!("async"));
        // Note: confusingly, this is done if the element does *not* have an "async" attribute.
        if was_parser_inserted && !async {
            self.non_blocking.set(true);
        }

        // Step 4.
        let text = self.Text();
        if text.is_empty() && !element.has_attribute(&local_name!("src")) {
            return;
        }

        // Step 5.
        if !self.upcast::<Node>().is_in_doc() {
            return;
        }

        // Step 6.
        if !self.is_javascript() {
            return;
        }

        // Step 7.
        if was_parser_inserted {
            self.parser_inserted.set(true);
            self.non_blocking.set(false);
        }

        // Step 8.
        self.already_started.set(true);

        // Step 9.
        let doc = document_from_node(self);
        if self.parser_inserted.get() && &*self.parser_document != &*doc {
            return;
        }

        // Step 10.
        if !doc.is_scripting_enabled() {
            return;
        }

        // TODO(#4577): Step 11: CSP.

        // Step 12.
        let for_attribute = element.get_attribute(&ns!(), &local_name!("for"));
        let event_attribute = element.get_attribute(&ns!(), &local_name!("event"));
        match (for_attribute.r(), event_attribute.r()) {
            (Some(for_attribute), Some(event_attribute)) => {
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

        // Step 13.
        let encoding = element.get_attribute(&ns!(), &local_name!("charset"))
                              .and_then(|charset| encoding_from_whatwg_label(&charset.value()))
                              .unwrap_or_else(|| doc.encoding());

        // Step 14.
        let cors_setting = cors_setting_for_element(element);

        // TODO: Step 15: Module script credentials mode.

        // TODO: Step 16: Nonce.

        // Step 17: Integrity metadata.
        let im_attribute = element.get_attribute(&ns!(), &local_name!("integrity"));
        let integrity_val = im_attribute.r().map(|a| a.value());
        let integrity_metadata = match integrity_val {
            Some(ref value) => &***value,
            None => "",
        };

        // TODO: Step 18: parser state.

        // TODO: Step 19: environment settings object.

        let base_url = doc.base_url();
        if let Some(src) = element.get_attribute(&ns!(), &local_name!("src")) {
            // Step 20.

            // Step 20.1.
            let src = src.value();

            // Step 20.2.
            if src.is_empty() {
                self.queue_error_event();
                return;
            }

            // Step 20.3: The "from an external file"" flag is stored in ClassicScript.

            // Step 20.4-20.5.
            let url = match base_url.join(&src) {
                Ok(url) => url,
                Err(_) => {
                    warn!("error parsing URL for script {}", &**src);
                    self.queue_error_event();
                    return;
                },
            };

            // Preparation for step 22.
            let kind = if element.has_attribute(&local_name!("defer")) && was_parser_inserted && !async {
                // Step 22.a: classic, has src, has defer, was parser-inserted, is not async.
                ExternalScriptKind::Deferred
            } else if was_parser_inserted && !async {
                // Step 22.b: classic, has src, was parser-inserted, is not async.
                ExternalScriptKind::ParsingBlocking
            } else if !async && !self.non_blocking.get() {
                // Step 22.c: classic, has src, is not async, is not non-blocking.
                ExternalScriptKind::AsapInOrder
            } else {
                // Step 22.d: classic, has src.
                ExternalScriptKind::Asap
            };

            // Step 20.6.
            fetch_a_classic_script(self, kind, url, cors_setting, integrity_metadata.to_owned(), encoding);

            // Step 22.
            match kind {
                ExternalScriptKind::Deferred => doc.add_deferred_script(self),
                ExternalScriptKind::ParsingBlocking => doc.set_pending_parsing_blocking_script(self, None),
                ExternalScriptKind::AsapInOrder => doc.push_asap_in_order_script(self),
                ExternalScriptKind::Asap => doc.add_asap_script(self),
            }
        } else {
            // Step 21.
            assert!(!text.is_empty());
            let result = Ok(ClassicScript::internal(text, base_url));

            // Step 22.
            if was_parser_inserted &&
               doc.get_current_parser().map_or(false, |parser| parser.script_nesting_level() <= 1) &&
               doc.get_script_blocking_stylesheets_count() > 0 {
                // Step 22.e: classic, has no src, was parser-inserted, is blocked on stylesheet.
                doc.set_pending_parsing_blocking_script(self, Some(result));
            } else {
                // Step 22.f: otherwise.
                self.execute(result);
            }
        }
    }

    fn unminify_js(&self, script: &mut ClassicScript) {
        if !opts::get().unminify_js {
            return;
        }

        match Command::new("js-beautify")
                      .stdin(Stdio::piped())
                      .stdout(Stdio::piped())
                      .spawn() {
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

        let path = PathBuf::from(window_from_node(self).unminified_js_dir().unwrap());
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

    /// https://html.spec.whatwg.org/multipage/#execute-the-script-block
    pub fn execute(&self, result: Result<ClassicScript, NetworkError>) {
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
            }

            Ok(script) => script,
        };

        self.unminify_js(&mut script);

        // Step 3.
        let neutralized_doc = if script.external {
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

        // Step 5.a.1.
        document.set_current_script(Some(self));

        // Step 5.a.2.
        self.run_a_classic_script(&script);

        // Step 6.
        document.set_current_script(old_script.r());

        // Step 7.
        if let Some(doc) = neutralized_doc {
            doc.decr_ignore_destructive_writes_counter();
        }

        // Step 8.
        if script.external {
            self.dispatch_load_event();
        }
    }

    // https://html.spec.whatwg.org/multipage/#run-a-classic-script
    pub fn run_a_classic_script(&self, script: &ClassicScript) {
        // TODO use a settings object rather than this element's document/window
        // Step 2
        let document = document_from_node(self);
        if !document.is_fully_active() || !document.is_scripting_enabled() {
            return;
        }

        // Steps 4-10
        let window = window_from_node(self);
        let line_number = if script.external { 1 } else { self.line_number as u32 };
        rooted!(in(window.get_cx()) let mut rval = UndefinedValue());
        let global = window.upcast::<GlobalScope>();
        global.evaluate_script_on_global_with_result(
            &script.text, script.url.as_str(), rval.handle_mut(), line_number);
    }

    pub fn queue_error_event(&self) {
        let window = window_from_node(self);
        window.dom_manipulation_task_source().queue_simple_event(self.upcast(), atom!("error"), &window);
    }

    pub fn dispatch_load_event(&self) {
        self.dispatch_event(atom!("load"),
                            EventBubbles::DoesNotBubble,
                            EventCancelable::NotCancelable);
    }

    pub fn dispatch_error_event(&self) {
        self.dispatch_event(atom!("error"),
                            EventBubbles::DoesNotBubble,
                            EventCancelable::NotCancelable);
    }

    pub fn is_javascript(&self) -> bool {
        let element = self.upcast::<Element>();
        let type_attr = element.get_attribute(&ns!(), &local_name!("type"));
        let is_js = match type_attr.as_ref().map(|s| s.value()) {
            Some(ref s) if s.is_empty() => {
                // type attr exists, but empty means js
                debug!("script type empty, inferring js");
                true
            },
            Some(s) => {
                debug!("script type={}", &**s);
                SCRIPT_JS_MIMES.contains(&s.to_ascii_lowercase().trim_matches(HTML_SPACE_CHARACTERS))
            },
            None => {
                debug!("no script type");
                let language_attr = element.get_attribute(&ns!(), &local_name!("language"));
                let is_js = match language_attr.as_ref().map(|s| s.value()) {
                    Some(ref s) if s.is_empty() => {
                        debug!("script language empty, inferring js");
                        true
                    },
                    Some(s) => {
                        debug!("script language={}", &**s);
                        let mut language = format!("text/{}", &**s);
                        language.make_ascii_lowercase();
                        SCRIPT_JS_MIMES.contains(&&*language)
                    },
                    None => {
                        debug!("no script type or language, inferring js");
                        true
                    }
                };
                // https://github.com/rust-lang/rust/issues/21114
                is_js
            }
        };
        // https://github.com/rust-lang/rust/issues/21114
        is_js
    }

    pub fn set_parser_inserted(&self, parser_inserted: bool) {
        self.parser_inserted.set(parser_inserted);
    }

    pub fn set_already_started(&self, already_started: bool) {
        self.already_started.set(already_started);
    }

    fn dispatch_event(&self,
                      type_: Atom,
                      bubbles: EventBubbles,
                      cancelable: EventCancelable) -> EventStatus {
        let window = window_from_node(self);
        let event = Event::new(window.upcast(), type_, bubbles, cancelable);
        event.fire(self.upcast())
    }
}

impl VirtualMethods for HTMLScriptElement {
    fn super_type(&self) -> Option<&VirtualMethods> {
        Some(self.upcast::<HTMLElement>() as &VirtualMethods)
    }

    fn attribute_mutated(&self, attr: &Attr, mutation: AttributeMutation) {
        self.super_type().unwrap().attribute_mutated(attr, mutation);
        match *attr.local_name() {
            local_name!("src") => {
                if let AttributeMutation::Set(_) = mutation {
                    if !self.parser_inserted.get() && self.upcast::<Node>().is_in_doc() {
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
        if !self.parser_inserted.get() && self.upcast::<Node>().is_in_doc() {
            self.prepare();
        }
    }

    fn bind_to_tree(&self, tree_in_doc: bool) {
        if let Some(ref s) = self.super_type() {
            s.bind_to_tree(tree_in_doc);
        }

        if tree_in_doc && !self.parser_inserted.get() {
            self.prepare();
        }
    }

    fn cloning_steps(&self, copy: &Node, maybe_doc: Option<&Document>,
                     clone_children: CloneChildrenFlag) {
        if let Some(ref s) = self.super_type() {
            s.cloning_steps(copy, maybe_doc, clone_children);
        }

        // https://html.spec.whatwg.org/multipage/#already-started
        if self.already_started.get() {
            copy.downcast::<HTMLScriptElement>().unwrap().set_already_started(true);
        }
    }
}

impl HTMLScriptElementMethods for HTMLScriptElement {
    // https://html.spec.whatwg.org/multipage/#dom-script-src
    make_url_getter!(Src, "src");
    // https://html.spec.whatwg.org/multipage/#dom-script-src
    make_setter!(SetSrc, "src");

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
        self.non_blocking.get() || self.upcast::<Element>().has_attribute(&local_name!("async"))
    }

    // https://html.spec.whatwg.org/multipage/#dom-script-async
    fn SetAsync(&self, value: bool) {
        self.non_blocking.set(false);
        self.upcast::<Element>().set_bool_attribute(&local_name!("async"), value);
    }

    // https://html.spec.whatwg.org/multipage/#dom-script-defer
    make_bool_getter!(Defer, "defer");
    // https://html.spec.whatwg.org/multipage/#dom-script-defer
    make_bool_setter!(SetDefer, "defer");

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

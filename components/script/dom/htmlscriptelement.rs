/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use document_loader::LoadType;
use dom::attr::Attr;
use dom::bindings::cell::DOMRefCell;
use dom::bindings::codegen::Bindings::AttrBinding::AttrMethods;
use dom::bindings::codegen::Bindings::DocumentBinding::DocumentMethods;
use dom::bindings::codegen::Bindings::HTMLScriptElementBinding;
use dom::bindings::codegen::Bindings::HTMLScriptElementBinding::HTMLScriptElementMethods;
use dom::bindings::codegen::Bindings::NodeBinding::NodeMethods;
use dom::bindings::global::GlobalRef;
use dom::bindings::inheritance::Castable;
use dom::bindings::js::RootedReference;
use dom::bindings::js::{JS, Root};
use dom::bindings::refcounted::Trusted;
use dom::bindings::str::DOMString;
use dom::document::Document;
use dom::element::{AttributeMutation, Element, ElementCreator};
use dom::event::{Event, EventBubbles, EventCancelable};
use dom::htmlelement::HTMLElement;
use dom::node::{ChildrenMutation, CloneChildrenFlag, Node};
use dom::node::{document_from_node, window_from_node};
use dom::virtualmethods::VirtualMethods;
use dom::window::ScriptHelpers;
use encoding::label::encoding_from_whatwg_label;
use encoding::types::{DecoderTrap, EncodingRef};
use html5ever::tree_builder::NextParserState;
use hyper::http::RawStatus;
use ipc_channel::ipc;
use ipc_channel::router::ROUTER;
use js::jsval::UndefinedValue;
use net_traits::{AsyncResponseListener, AsyncResponseTarget, Metadata, NetworkError};
use network_listener::{NetworkListener, PreInvoke};
use std::ascii::AsciiExt;
use std::cell::Cell;
use std::mem;
use std::sync::{Arc, Mutex};
use string_cache::Atom;
use style::str::{HTML_SPACE_CHARACTERS, StaticStringVec};
use url::Url;

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

    /// https://html.spec.whatwg.org/multipage/#ready-to-be-parser-executed
    ready_to_be_parser_executed: Cell<bool>,

    /// Document of the parser that created this element
    parser_document: JS<Document>,

    /// The source this script was loaded from
    load: DOMRefCell<Option<ScriptOrigin>>,

    #[ignore_heap_size_of = "Defined in rust-encoding"]
    /// https://html.spec.whatwg.org/multipage/#concept-script-encoding
    block_character_encoding: Cell<Option<EncodingRef>>,
}

impl HTMLScriptElement {
    fn new_inherited(localName: Atom, prefix: Option<DOMString>, document: &Document,
                     creator: ElementCreator) -> HTMLScriptElement {
        HTMLScriptElement {
            htmlelement:
                HTMLElement::new_inherited(localName, prefix, document),
            already_started: Cell::new(false),
            parser_inserted: Cell::new(creator == ElementCreator::ParserCreated),
            non_blocking: Cell::new(creator != ElementCreator::ParserCreated),
            ready_to_be_parser_executed: Cell::new(false),
            parser_document: JS::from_ref(document),
            load: DOMRefCell::new(None),
            block_character_encoding: Cell::new(None),
        }
    }

    #[allow(unrooted_must_root)]
    pub fn new(localName: Atom, prefix: Option<DOMString>, document: &Document,
               creator: ElementCreator) -> Root<HTMLScriptElement> {
        let element = HTMLScriptElement::new_inherited(localName, prefix, document, creator);
        Node::reflect_node(box element, document, HTMLScriptElementBinding::Wrap)
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
pub enum ScriptOrigin {
    Internal(DOMString, Url),
    External(Result<(Metadata, Vec<u8>), NetworkError>),
}

/// The context required for asynchronously loading an external script source.
struct ScriptContext {
    /// The element that initiated the request.
    elem: Trusted<HTMLScriptElement>,
    /// The response body received to date.
    data: Vec<u8>,
    /// The response metadata received to date.
    metadata: Option<Metadata>,
    /// The initial URL requested.
    url: Url,
    /// Indicates whether the request failed, and why
    status: Result<(), NetworkError>
}

impl AsyncResponseListener for ScriptContext {
    fn headers_available(&mut self, metadata: Result<Metadata, NetworkError>) {
        self.metadata = metadata.ok();

        let status_code = self.metadata.as_ref().and_then(|m| {
            match m.status {
                Some(RawStatus(c, _)) => Some(c),
                _ => None,
            }
        }).unwrap_or(0);

        self.status = match status_code {
            0 => Err(NetworkError::Internal("No http status code received".to_owned())),
            200...299 => Ok(()), // HTTP ok status codes
            _ => Err(NetworkError::Internal(format!("HTTP error code {}", status_code)))
        };
    }

    fn data_available(&mut self, payload: Vec<u8>) {
        if self.status.is_ok() {
            let mut payload = payload;
            self.data.append(&mut payload);
        }
    }

    fn response_complete(&mut self, status: Result<(), NetworkError>) {
        let load = status.and(self.status.clone()).map(|_| {
            let data = mem::replace(&mut self.data, vec!());
            let metadata = self.metadata.take().unwrap();
            (metadata, data)
        });
        let elem = self.elem.root();
        // TODO: maybe set this to None again after script execution to save memory.
        *elem.load.borrow_mut() = Some(ScriptOrigin::External(load));
        elem.ready_to_be_parser_executed.set(true);

        let document = document_from_node(elem.r());
        document.finish_load(LoadType::Script(self.url.clone()));
    }
}

impl PreInvoke for ScriptContext {}

impl HTMLScriptElement {
    /// https://html.spec.whatwg.org/multipage/#prepare-a-script
    pub fn prepare(&self) -> NextParserState {
        // Step 1.
        if self.already_started.get() {
            return NextParserState::Continue;
        }

        // Step 2.
        let was_parser_inserted = self.parser_inserted.get();
        self.parser_inserted.set(false);

        // Step 3.
        let element = self.upcast::<Element>();
        let async = element.has_attribute(&atom!("async"));
        // Note: confusingly, this is done if the element does *not* have an "async" attribute.
        if was_parser_inserted && !async {
            self.non_blocking.set(true);
        }

        // Step 4.
        let text = self.Text();
        if text.is_empty() && !element.has_attribute(&atom!("src")) {
            return NextParserState::Continue;
        }

        // Step 5.
        if !self.upcast::<Node>().is_in_doc() {
            return NextParserState::Continue;
        }

        // Step 6.
        if !self.is_javascript() {
            return NextParserState::Continue;
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
        let document_from_node_ref = doc.r();
        if self.parser_inserted.get() && &*self.parser_document != document_from_node_ref {
            return NextParserState::Continue;
        }

        // Step 10.
        if !document_from_node_ref.is_scripting_enabled() {
            return NextParserState::Continue;
        }

        // TODO(#4577): Step 11: CSP.

        // Step 12.
        let for_attribute = element.get_attribute(&ns!(), &atom!("for"));
        let event_attribute = element.get_attribute(&ns!(), &atom!("event"));
        match (for_attribute.r(), event_attribute.r()) {
            (Some(for_attribute), Some(event_attribute)) => {
                let for_value = for_attribute.value().to_ascii_lowercase();
                let for_value = for_value.trim_matches(HTML_SPACE_CHARACTERS);
                if for_value != "window" {
                    return NextParserState::Continue;
                }

                let event_value = event_attribute.value().to_ascii_lowercase();
                let event_value = event_value.trim_matches(HTML_SPACE_CHARACTERS);
                if event_value != "onload" && event_value != "onload()" {
                    return NextParserState::Continue;
                }
            },
            (_, _) => (),
        }

        // Step 13.
        if let Some(ref charset) = element.get_attribute(&ns!(), &atom!("charset")) {
            if let Some(encodingRef) = encoding_from_whatwg_label(&charset.Value()) {
                self.block_character_encoding.set(Some(encodingRef));
            }
        }

        // TODO: Step 14: CORS.

        // TODO: Step 15: environment settings object.

        let base_url = doc.base_url();
        let is_external = match element.get_attribute(&ns!(), &atom!("src")) {
            // Step 16.
            Some(ref src) => {
                // Step 16.1.
                let src = src.value();

                // Step 16.2.
                if src.is_empty() {
                    self.queue_error_event();
                    return NextParserState::Continue;
                }

                // Step 16.4-16.5.
                let url = match base_url.join(&src) {
                    Err(_) => {
                        error!("error parsing URL for script {}", &**src);
                        self.queue_error_event();
                        return NextParserState::Continue;
                    }
                    Ok(url) => url,
                };

                // Step 16.6.
                // TODO(#9186): use the fetch infrastructure.
                let elem = Trusted::new(self);

                let context = Arc::new(Mutex::new(ScriptContext {
                    elem: elem,
                    data: vec!(),
                    metadata: None,
                    url: url.clone(),
                    status: Ok(())
                }));

                let (action_sender, action_receiver) = ipc::channel().unwrap();
                let listener = NetworkListener {
                    context: context,
                    script_chan: doc.window().networking_task_source(),
                    wrapper: Some(doc.window().get_runnable_wrapper()),
                };
                let response_target = AsyncResponseTarget {
                    sender: action_sender,
                };
                ROUTER.add_route(action_receiver.to_opaque(), box move |message| {
                    listener.notify_action(message.to().unwrap());
                });

                doc.load_async(LoadType::Script(url), response_target);
                true
            },
            None => false,
        };

        // Step 18.
        let deferred = element.has_attribute(&atom!("defer"));
        // Step 18.a: has src, has defer, was parser-inserted, is not async.
        if is_external &&
           deferred &&
           was_parser_inserted &&
           !async {
            doc.add_deferred_script(self);
            // Second part implemented in Document::process_deferred_scripts.
            return NextParserState::Continue;
        // Step 18.b: has src, was parser-inserted, is not async.
        } else if is_external &&
                  was_parser_inserted &&
                  !async {
            doc.set_pending_parsing_blocking_script(Some(self));
            // Second part implemented in the load result handler.
        // Step 18.c: has src, isn't async, isn't non-blocking.
        } else if is_external &&
                  !async &&
                  !self.non_blocking.get() {
            doc.push_asap_in_order_script(self);
            // Second part implemented in Document::process_asap_scripts.
        // Step 18.d: has src.
        } else if is_external {
            doc.add_asap_script(self);
            // Second part implemented in Document::process_asap_scripts.
        // Step 18.e: doesn't have src, was parser-inserted, is blocked on stylesheet.
        } else if !is_external &&
                  was_parser_inserted &&
                  // TODO: check for script nesting levels.
                  doc.get_script_blocking_stylesheets_count() > 0 {
            doc.set_pending_parsing_blocking_script(Some(self));
            *self.load.borrow_mut() = Some(ScriptOrigin::Internal(text, base_url));
            self.ready_to_be_parser_executed.set(true);
        // Step 18.f: otherwise.
        } else {
            assert!(!text.is_empty());
            self.ready_to_be_parser_executed.set(true);
            *self.load.borrow_mut() = Some(ScriptOrigin::Internal(text, base_url));
            self.execute();
            return NextParserState::Continue;
        }

        // TODO: make this suspension happen automatically.
        if was_parser_inserted {
            if let Some(parser) = doc.get_current_parser() {
                parser.r().suspend();
            }
        }
        NextParserState::Suspend
    }

    pub fn is_ready_to_be_executed(&self) -> bool {
        self.ready_to_be_parser_executed.get()
    }

    /// https://html.spec.whatwg.org/multipage/#execute-the-script-block
    pub fn execute(&self) {
        assert!(self.ready_to_be_parser_executed.get());

        // Step 1.
        let doc = document_from_node(self);
        if self.parser_inserted.get() && &*doc != &*self.parser_document {
            return;
        }

        let load = self.load.borrow_mut().take().unwrap();

        // Step 2.
        let (source, external, url) = match load {
            // Step 2.a.
            ScriptOrigin::External(Err(e)) => {
                error!("error loading script {:?}", e);
                self.dispatch_error_event();
                return;
            }

            // Step 2.b.1.a.
            ScriptOrigin::External(Ok((metadata, bytes))) => {
                debug!("loading external script, url = {}", metadata.final_url);

                let encoding = metadata.charset
                    .and_then(|encoding| encoding_from_whatwg_label(&encoding))
                    .or_else(|| self.block_character_encoding.get())
                    .unwrap_or_else(|| self.parser_document.encoding());

                (DOMString::from(encoding.decode(&*bytes, DecoderTrap::Replace).unwrap()),
                    true,
                    metadata.final_url)
            },

            // Step 2.b.1.c.
            ScriptOrigin::Internal(text, url) => {
                (text, false, url)
            }
        };

        // Step 2.b.2.
        if !self.dispatch_before_script_execute_event() {
            return;
        }

        // Step 2.b.3.
        // TODO: If the script is from an external file, then increment the
        // ignore-destructive-writes counter of the script element's node
        // document. Let neutralised doc be that Document.

        // Step 2.b.4.
        let document = document_from_node(self);
        let document = document.r();
        let old_script = document.GetCurrentScript();

        // Step 2.b.5.
        document.set_current_script(Some(self));

        // Step 2.b.6.
        // TODO: Create a script...
        let window = window_from_node(self);
        rooted!(in(window.get_cx()) let mut rval = UndefinedValue());
        window.evaluate_script_on_global_with_result(&*source,
                                                         url.as_str(),
                                                         rval.handle_mut());

        // Step 2.b.7.
        document.set_current_script(old_script.r());

        // Step 2.b.8.
        // TODO: Decrement the ignore-destructive-writes counter of neutralised
        // doc, if it was incremented in the earlier step.

        // Step 2.b.9.
        self.dispatch_after_script_execute_event();

        // Step 2.b.10.
        if external {
            self.dispatch_load_event();
        } else {
            window.dom_manipulation_task_source().queue_simple_event(self.upcast(), atom!("load"));
        }
    }

    pub fn queue_error_event(&self) {
        window_from_node(self).dom_manipulation_task_source().queue_simple_event(self.upcast(), atom!("error"));
    }

    pub fn dispatch_before_script_execute_event(&self) -> bool {
        self.dispatch_event(atom!("beforescriptexecute"),
                            EventBubbles::Bubbles,
                            EventCancelable::Cancelable)
    }

    pub fn dispatch_after_script_execute_event(&self) {
        self.dispatch_event(atom!("afterscriptexecute"),
                            EventBubbles::Bubbles,
                            EventCancelable::NotCancelable);
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
        let type_attr = element.get_attribute(&ns!(), &atom!("type"));
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
                let language_attr = element.get_attribute(&ns!(), &atom!("language"));
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

    pub fn set_already_started(&self, already_started: bool) {
        self.already_started.set(already_started);
    }

    fn dispatch_event(&self,
                      type_: Atom,
                      bubbles: EventBubbles,
                      cancelable: EventCancelable) -> bool {
        let window = window_from_node(self);
        let window = window.r();
        let event = Event::new(GlobalRef::Window(window), type_, bubbles, cancelable);
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
            atom!("src") => {
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

    // https://html.spec.whatwg.org/multipage/#dom-script-defer
    make_bool_getter!(Defer, "defer");
    // https://html.spec.whatwg.org/multipage/#dom-script-defer
    make_bool_setter!(SetDefer, "defer");

    // https://html.spec.whatwg.org/multipage/#dom-script-event
    make_getter!(Event, "event");
    // https://html.spec.whatwg.org/multipage/#dom-script-event
    make_setter!(SetEvent, "event");

    // https://html.spec.whatwg.org/multipage/#dom-script-htmlfor
    make_getter!(HtmlFor, "for");
    // https://html.spec.whatwg.org/multipage/#dom-script-htmlfor
    make_setter!(SetHtmlFor, "for");

    // https://html.spec.whatwg.org/multipage/#dom-script-text
    fn Text(&self) -> DOMString {
        Node::collect_text_contents(self.upcast::<Node>().children())
    }

    // https://html.spec.whatwg.org/multipage/#dom-script-text
    fn SetText(&self, value: DOMString) {
        self.upcast::<Node>().SetTextContent(Some(value))
    }
}

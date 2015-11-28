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
use dom::bindings::trace::JSTraceable;
use dom::document::Document;
use dom::element::{AttributeMutation, Element, ElementCreator};
use dom::event::{Event, EventBubbles, EventCancelable};
use dom::htmlelement::HTMLElement;
use dom::node::{ChildrenMutation, CloneChildrenFlag, Node};
use dom::node::{document_from_node, window_from_node};
use dom::virtualmethods::VirtualMethods;
use dom::window::ScriptHelpers;
use encoding::all::UTF_8;
use encoding::label::encoding_from_whatwg_label;
use encoding::types::{DecoderTrap, Encoding, EncodingRef};
use html5ever::tree_builder::NextParserState;
use ipc_channel::ipc;
use ipc_channel::router::ROUTER;
use js::jsapi::RootedValue;
use js::jsval::UndefinedValue;
use net_traits::{AsyncResponseListener, AsyncResponseTarget, Metadata};
use network_listener::{NetworkListener, PreInvoke};
use script_task::ScriptTaskEventCategory::ScriptEvent;
use script_task::{CommonScriptMsg, Runnable, ScriptChan};
use std::ascii::AsciiExt;
use std::cell::Cell;
use std::mem;
use std::sync::{Arc, Mutex};
use url::{Url, UrlParser};
use util::str::{DOMString, HTML_SPACE_CHARACTERS, StaticStringVec};

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
    block_character_encoding: DOMRefCell<EncodingRef>,
}

impl HTMLScriptElement {
    fn new_inherited(localName: DOMString, prefix: Option<DOMString>, document: &Document,
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
            block_character_encoding: DOMRefCell::new(UTF_8 as EncodingRef),
        }
    }

    #[allow(unrooted_must_root)]
    pub fn new(localName: DOMString, prefix: Option<DOMString>, document: &Document,
               creator: ElementCreator) -> Root<HTMLScriptElement> {
        let element = HTMLScriptElement::new_inherited(localName, prefix, document, creator);
        Node::reflect_node(box element, document, HTMLScriptElementBinding::Wrap)
    }
}


/// Supported script types as defined by
/// <https://html.spec.whatwg.org/multipage/#support-the-scripting-language>.
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
    External(Result<(Metadata, Vec<u8>), String>),
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
}

impl AsyncResponseListener for ScriptContext {
    fn headers_available(&mut self, metadata: Metadata) {
        self.metadata = Some(metadata);
    }

    fn data_available(&mut self, payload: Vec<u8>) {
        let mut payload = payload;
        self.data.append(&mut payload);
    }

    fn response_complete(&mut self, status: Result<(), String>) {
        let load = status.map(|_| {
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
        // Step 6, 7.
        if !self.is_javascript() {
            return NextParserState::Continue;
        }
        // Step 8.
        if was_parser_inserted {
            self.parser_inserted.set(true);
            self.non_blocking.set(false);
        }
        // Step 9.
        self.already_started.set(true);

        // Step 10.
        let doc = document_from_node(self);
        let document_from_node_ref = doc.r();
        if self.parser_inserted.get() && &*self.parser_document != document_from_node_ref {
            return NextParserState::Continue;
        }

        // Step 11.
        if !document_from_node_ref.is_scripting_enabled() {
            return NextParserState::Continue;
        }

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
                *self.block_character_encoding.borrow_mut() = encodingRef;
            }
        }

        // Step 14.
        let window = window_from_node(self);
        let window = window.r();
        let base_url = window.get_url();
        let deferred = element.has_attribute(&atom!("defer"));

        let is_external = match element.get_attribute(&ns!(), &atom!("src")) {
            // Step 14.
            Some(ref src) => {
                // Step 14.1
                let src = src.value();

                // Step 14.2
                if src.is_empty() {
                    self.queue_error_event();
                    return NextParserState::Continue;
                }

                // Step 14.3
                match UrlParser::new().base_url(&base_url).parse(&src) {
                    Err(_) => {
                        // Step 14.4
                        error!("error parsing URL for script {}", &**src);
                        self.queue_error_event();
                        return NextParserState::Continue;
                    }
                    Ok(url) => {
                        // Step 14.5
                        // TODO: Do a potentially CORS-enabled fetch with the mode being the current
                        // state of the element's `crossorigin` content attribute, the origin being
                        // the origin of the script element's node document, and the default origin
                        // behaviour set to taint.
                        let script_chan = window.script_chan();
                        let elem = Trusted::new(window.get_cx(), self, script_chan.clone());

                        let context = Arc::new(Mutex::new(ScriptContext {
                            elem: elem,
                            data: vec!(),
                            metadata: None,
                            url: url.clone(),
                        }));

                        let (action_sender, action_receiver) = ipc::channel().unwrap();
                        let listener = box NetworkListener {
                            context: context,
                            script_chan: script_chan,
                        };
                        let response_target = AsyncResponseTarget {
                            sender: action_sender,
                        };
                        ROUTER.add_route(action_receiver.to_opaque(), box move |message| {
                            listener.notify(message.to().unwrap());
                        });

                        doc.load_async(LoadType::Script(url), response_target);
                    }
                }
                true
            },
            None => false,
        };

        // Step 15.
        // Step 15.a, has src, has defer, was parser-inserted, is not async.
        if is_external &&
           deferred &&
           was_parser_inserted &&
           !async {
            doc.add_deferred_script(self);
            // Second part implemented in Document::process_deferred_scripts.
            return NextParserState::Continue;
        // Step 15.b, has src, was parser-inserted, is not async.
        } else if is_external &&
                  was_parser_inserted &&
                  !async {
            doc.set_pending_parsing_blocking_script(Some(self));
            // Second part implemented in the load result handler.
        // Step 15.c, doesn't have src, was parser-inserted, is blocked on stylesheet.
        } else if !is_external &&
                  was_parser_inserted &&
                  // TODO: check for script nesting levels.
                  doc.get_script_blocking_stylesheets_count() > 0 {
            doc.set_pending_parsing_blocking_script(Some(self));
            *self.load.borrow_mut() = Some(ScriptOrigin::Internal(text, base_url));
            self.ready_to_be_parser_executed.set(true);
        // Step 15.d, has src, isn't async, isn't non-blocking.
        } else if is_external &&
                  !async &&
                  !self.non_blocking.get() {
            doc.push_asap_in_order_script(self);
            // Second part implemented in Document::process_asap_scripts.
        // Step 15.e, has src.
        } else if is_external {
            doc.add_asap_script(self);
            // Second part implemented in Document::process_asap_scripts.
        // Step 15.f, otherwise.
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
                parser.suspend();
            }
        }
        return NextParserState::Suspend;
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
                error!("error loading script {}", e);
                self.dispatch_error_event();
                return;
            }

            // Step 2.b.1.a.
            ScriptOrigin::External(Ok((metadata, bytes))) => {
                // Step 1.
                // TODO: If the resource's Content Type metadata, if any,
                // specifies a character encoding, and the user agent supports
                // that encoding, then let character encoding be that encoding,
                // and jump to the bottom step in this series of steps.

                // Step 2.
                // TODO: If the algorithm above set the script block's
                // character encoding, then let character encoding be that
                // encoding, and jump to the bottom step in this series of
                // steps.

                // Step 3.
                // TODO: Let character encoding be the script block's fallback
                // character encoding.

                // Step 4.
                // TODO: Otherwise, decode the file to Unicode, using character
                // encoding as the fallback encoding.

                (DOMString::from(UTF_8.decode(&*bytes, DecoderTrap::Replace).unwrap()),
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
        let mut rval = RootedValue::new(window.get_cx(), UndefinedValue());
        window.evaluate_script_on_global_with_result(&*source,
                                                         &*url.serialize(),
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
            let chan = window.script_chan();
            let handler = Trusted::new(window.get_cx(), self, chan.clone());
            let dispatcher = box EventDispatcher {
                element: handler,
                is_error: false,
            };
            chan.send(CommonScriptMsg::RunnableMsg(ScriptEvent, dispatcher)).unwrap();
        }
    }

    pub fn queue_error_event(&self) {
        let window = window_from_node(self);
        let window = window.r();
        let chan = window.script_chan();
        let handler = Trusted::new(window.get_cx(), self, chan.clone());
        let dispatcher = box EventDispatcher {
            element: handler,
            is_error: true,
        };
        chan.send(CommonScriptMsg::RunnableMsg(ScriptEvent, dispatcher)).unwrap();
    }

    pub fn dispatch_before_script_execute_event(&self) -> bool {
        self.dispatch_event("beforescriptexecute".to_owned(),
                            EventBubbles::Bubbles,
                            EventCancelable::Cancelable)
    }

    pub fn dispatch_after_script_execute_event(&self) {
        self.dispatch_event("afterscriptexecute".to_owned(),
                            EventBubbles::Bubbles,
                            EventCancelable::NotCancelable);
    }

    pub fn dispatch_load_event(&self) {
        self.dispatch_event("load".to_owned(),
                            EventBubbles::DoesNotBubble,
                            EventCancelable::NotCancelable);
    }

    pub fn dispatch_error_event(&self) {
        self.dispatch_event("error".to_owned(),
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

    pub fn mark_already_started(&self) {
        self.already_started.set(true);
    }

    fn dispatch_event(&self,
                      type_: String,
                      bubbles: EventBubbles,
                      cancelable: EventCancelable) -> bool {
        let window = window_from_node(self);
        let window = window.r();
        let event = Event::new(GlobalRef::Window(window),
                               DOMString::from(type_),
                               bubbles,
                               cancelable);
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
            copy.downcast::<HTMLScriptElement>().unwrap().mark_already_started();
        }
    }
}

impl HTMLScriptElementMethods for HTMLScriptElement {
    // https://html.spec.whatwg.org/multipage/#dom-script-src
    make_url_getter!(Src, "src");
    // https://html.spec.whatwg.org/multipage/#dom-script-src
    make_setter!(SetSrc, "src");

    // https://html.spec.whatwg.org/multipage/#dom-script-text
    fn Text(&self) -> DOMString {
        Node::collect_text_contents(self.upcast::<Node>().children())
    }

    // https://html.spec.whatwg.org/multipage/#dom-script-text
    fn SetText(&self, value: DOMString) {
        self.upcast::<Node>().SetTextContent(Some(value))
    }
}

struct EventDispatcher {
    element: Trusted<HTMLScriptElement>,
    is_error: bool,
}

impl Runnable for EventDispatcher {
    fn handler(self: Box<EventDispatcher>) {
        let target = self.element.root();
        if self.is_error {
            target.dispatch_error_event();
        } else {
            target.dispatch_load_event();
        }
    }
}

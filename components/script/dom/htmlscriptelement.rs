/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::ascii::AsciiExt;

use document_loader::LoadType;
use dom::attr::Attr;
use dom::bindings::cell::DOMRefCell;
use dom::bindings::codegen::Bindings::AttrBinding::AttrMethods;
use dom::bindings::codegen::Bindings::DocumentBinding::DocumentMethods;
use dom::bindings::codegen::Bindings::HTMLScriptElementBinding;
use dom::bindings::codegen::Bindings::HTMLScriptElementBinding::HTMLScriptElementMethods;
use dom::bindings::codegen::Bindings::NodeBinding::NodeMethods;
use dom::bindings::codegen::InheritTypes::EventTargetCast;
use dom::bindings::codegen::InheritTypes::{ElementCast, HTMLElementCast, NodeCast};
use dom::bindings::codegen::InheritTypes::{HTMLScriptElementDerived, HTMLScriptElementCast};
use dom::bindings::global::GlobalRef;
use dom::bindings::js::RootedReference;
use dom::bindings::js::{JS, Root};
use dom::bindings::refcounted::Trusted;
use dom::bindings::trace::JSTraceable;
use dom::document::Document;
use dom::element::{ElementCreator, ElementTypeId};
use dom::event::{Event, EventBubbles, EventCancelable};
use dom::eventtarget::{EventTarget, EventTargetTypeId};
use dom::htmlelement::{HTMLElement, HTMLElementTypeId};
use dom::node::{ChildrenMutation, CloneChildrenFlag, Node};
use dom::node::{NodeTypeId, document_from_node, window_from_node};
use dom::virtualmethods::VirtualMethods;
use dom::window::ScriptHelpers;
use js::jsapi::RootedValue;
use js::jsval::UndefinedValue;
use network_listener::{NetworkListener, PreInvoke};
use script_task::{ScriptChan, Runnable, CommonScriptMsg};

use encoding::all::UTF_8;
use encoding::label::encoding_from_whatwg_label;
use encoding::types::{Encoding, EncodingRef, DecoderTrap};
use html5ever::tree_builder::NextParserState;
use ipc_channel::ipc;
use ipc_channel::router::ROUTER;
use net_traits::{Metadata, AsyncResponseListener, AsyncResponseTarget};
use std::cell::{RefCell, Cell};
use std::mem;
use std::sync::{Arc, Mutex};
use string_cache::Atom;
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
    ///
    /// (currently unused)
    ready_to_be_parser_executed: Cell<bool>,

    /// Document of the parser that created this element
    parser_document: JS<Document>,

    #[ignore_heap_size_of = "Defined in rust-encoding"]
    /// https://html.spec.whatwg.org/multipage/#concept-script-encoding
    block_character_encoding: DOMRefCell<EncodingRef>,
}

impl HTMLScriptElementDerived for EventTarget {
    fn is_htmlscriptelement(&self) -> bool {
        *self.type_id() ==
            EventTargetTypeId::Node(
                NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLScriptElement)))
    }
}

impl HTMLScriptElement {
    fn new_inherited(localName: DOMString, prefix: Option<DOMString>, document: &Document,
                     creator: ElementCreator) -> HTMLScriptElement {
        HTMLScriptElement {
            htmlelement:
                HTMLElement::new_inherited(HTMLElementTypeId::HTMLScriptElement, localName, prefix, document),
            already_started: Cell::new(false),
            parser_inserted: Cell::new(creator == ElementCreator::ParserCreated),
            non_blocking: Cell::new(creator != ElementCreator::ParserCreated),
            ready_to_be_parser_executed: Cell::new(false),
            parser_document: JS::from_ref(document),
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
/// <https://whatwg.org/html/#support-the-scripting-language>.
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

#[derive(HeapSizeOf)]
pub enum ScriptOrigin {
    Internal(String, Url),
    External(Result<(Metadata, Vec<u8>), String>),
}

/// The context required for asynchronously loading an external script source.
struct ScriptContext {
    /// The element that initiated the request.
    elem: Trusted<HTMLScriptElement>,
    /// The response body received to date.
    data: RefCell<Vec<u8>>,
    /// The response metadata received to date.
    metadata: RefCell<Option<Metadata>>,
    /// Whether the owning document's parser should resume once the response completes.
    resume_on_completion: bool,
    /// The initial URL requested.
    url: Url,
}

impl AsyncResponseListener for ScriptContext {
    fn headers_available(&self, metadata: Metadata) {
        *self.metadata.borrow_mut() = Some(metadata);
    }

    fn data_available(&self, payload: Vec<u8>) {
        let mut payload = payload;
        self.data.borrow_mut().append(&mut payload);
    }

    fn response_complete(&self, status: Result<(), String>) {
        let load = status.map(|_| {
            let data = mem::replace(&mut *self.data.borrow_mut(), vec!());
            let metadata = self.metadata.borrow_mut().take().unwrap();
            (metadata, data)
        });
        let elem = self.elem.root();

        elem.r().execute(ScriptOrigin::External(load));

        let document = document_from_node(elem.r());
        document.r().finish_load(LoadType::Script(self.url.clone()));

        if self.resume_on_completion {
            document.r().get_current_parser().unwrap().r().resume();
        }
    }
}

impl PreInvoke for ScriptContext {}

impl HTMLScriptElement {
    pub fn prepare(&self) -> NextParserState {
        // https://html.spec.whatwg.org/multipage/#prepare-a-script
        // Step 1.
        if self.already_started.get() {
            return NextParserState::Continue;
        }
        // Step 2.
        let was_parser_inserted = self.parser_inserted.get();
        self.parser_inserted.set(false);

        // Step 3.
        let element = ElementCast::from_ref(self);
        if was_parser_inserted && element.has_attribute(&atom!("async")) {
            self.non_blocking.set(true);
        }
        // Step 4.
        let text = self.Text();
        if text.is_empty() && !element.has_attribute(&atom!("src")) {
            return NextParserState::Continue;
        }
        // Step 5.
        let node = NodeCast::from_ref(self);
        if !node.is_in_doc() {
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
        let document_from_node_ref = document_from_node(self);
        let document_from_node_ref = document_from_node_ref.r();
        if self.parser_inserted.get() && self.parser_document.root().r() != document_from_node_ref {
            return NextParserState::Continue;
        }

        // Step 11.
        if !document_from_node_ref.is_scripting_enabled() {
            return NextParserState::Continue;
        }

        // Step 12.
        let for_attribute = element.get_attribute(&ns!(""), &atom!("for"));
        let event_attribute = element.get_attribute(&ns!(""), &Atom::from_slice("event"));
        match (for_attribute.r(), event_attribute.r()) {
            (Some(for_attribute), Some(event_attribute)) => {
                let for_value = for_attribute.Value()
                                             .to_ascii_lowercase();
                let for_value = for_value.trim_matches(HTML_SPACE_CHARACTERS);
                if for_value != "window" {
                    return NextParserState::Continue;
                }

                let event_value = event_attribute.Value().to_ascii_lowercase();
                let event_value = event_value.trim_matches(HTML_SPACE_CHARACTERS);
                if event_value != "onload" && event_value != "onload()" {
                    return NextParserState::Continue;
                }
            },
            (_, _) => (),
        }

        // Step 13.
        if let Some(ref charset) = element.get_attribute(&ns!(""), &Atom::from_slice("charset")) {
            if let Some(encodingRef) = encoding_from_whatwg_label(&charset.r().Value()) {
                *self.block_character_encoding.borrow_mut() = encodingRef;
            }
        }

        // Step 14.
        let window = window_from_node(self);
        let window = window.r();
        let base_url = window.get_url();

        let load = match element.get_attribute(&ns!(""), &atom!("src")) {
            // Step 14.
            Some(ref src) => {
                // Step 14.1
                let src = src.r().Value();

                // Step 14.2
                if src.is_empty() {
                    self.queue_error_event();
                    return NextParserState::Continue;
                }

                // Step 14.3
                match UrlParser::new().base_url(&base_url).parse(&*src) {
                    Err(_) => {
                        // Step 14.4
                        error!("error parsing URL for script {}", src);
                        self.queue_error_event();
                        return NextParserState::Continue;
                    }
                    Ok(url) => {
                        // Step 14.5
                        // TODO: Do a potentially CORS-enabled fetch with the mode being the current
                        // state of the element's `crossorigin` content attribute, the origin being
                        // the origin of the script element's node document, and the default origin
                        // behaviour set to taint.
                        let doc = document_from_node(self);

                        let script_chan = window.script_chan();
                        let elem = Trusted::new(window.get_cx(), self, script_chan.clone());

                        let context = Arc::new(Mutex::new(ScriptContext {
                            elem: elem,
                            data: RefCell::new(vec!()),
                            metadata: RefCell::new(None),
                            resume_on_completion: self.parser_inserted.get(),
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

                        doc.r().load_async(LoadType::Script(url), response_target);

                        if self.parser_inserted.get() {
                            doc.r().get_current_parser().unwrap().r().suspend();
                        }
                        return NextParserState::Suspend;
                    }
                }
            },
            None => ScriptOrigin::Internal(text, base_url),
        };

        // Step 15.
        // TODO: Add support for the `defer` and `async` attributes.  (For now, we fetch all
        // scripts synchronously and execute them immediately.)
        self.execute(load);
        NextParserState::Continue
    }

    pub fn execute(&self, load: ScriptOrigin) {
        // Step 1.
        // TODO: If the element is flagged as "parser-inserted", but the
        // element's node document is not the Document of the parser that
        // created the element, then abort these steps.

        // Step 2.
        let (source, external, url) = match load {
            // Step 2.a.
            ScriptOrigin::External(Err(e)) => {
                error!("error loading script {}", e);
                self.queue_error_event();
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

                (UTF_8.decode(&*bytes, DecoderTrap::Replace).unwrap(), true,
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
        let mut rval = RootedValue::new(window.r().get_cx(), UndefinedValue());
        window.r().evaluate_script_on_global_with_result(&*source,
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
            let chan = window.r().script_chan();
            let handler = Trusted::new(window.r().get_cx(), self, chan.clone());
            let dispatcher = box EventDispatcher {
                element: handler,
                is_error: false,
            };
            chan.send(CommonScriptMsg::RunnableMsg(dispatcher)).unwrap();
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
        chan.send(CommonScriptMsg::RunnableMsg(dispatcher)).unwrap();
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
        let element = ElementCast::from_ref(self);
        match element.get_attribute(&ns!(""), &atom!("type")).map(|s| s.r().Value()) {
            Some(ref s) if s.is_empty() => {
                // type attr exists, but empty means js
                debug!("script type empty, inferring js");
                true
            },
            Some(ref s) => {
                debug!("script type={}", *s);
                SCRIPT_JS_MIMES.contains(&s.to_ascii_lowercase().trim_matches(HTML_SPACE_CHARACTERS))
            },
            None => {
                debug!("no script type");
                match element.get_attribute(&ns!(""), &atom!("language"))
                             .map(|s| s.r().Value()) {
                    Some(ref s) if s.is_empty() => {
                        debug!("script language empty, inferring js");
                        true
                    },
                    Some(ref s) => {
                        debug!("script language={}", *s);
                        SCRIPT_JS_MIMES.contains(&&*format!("text/{}", s).to_ascii_lowercase())
                    },
                    None => {
                        debug!("no script type or language, inferring js");
                        true
                    }
                }
            }
        }
    }

    pub fn mark_already_started(&self) {
        self.already_started.set(true);
    }
}


impl HTMLScriptElement {
    fn dispatch_event(&self,
                      type_: DOMString,
                      bubbles: EventBubbles,
                      cancelable: EventCancelable) -> bool {
        let window = window_from_node(self);
        let window = window.r();
        let event = Event::new(GlobalRef::Window(window),
                               type_,
                               bubbles,
                               cancelable);
        let event = event.r();
        let target = EventTargetCast::from_ref(self);
        event.fire(target)
    }
}

impl VirtualMethods for HTMLScriptElement {
    fn super_type<'b>(&'b self) -> Option<&'b VirtualMethods> {
        let htmlelement: &HTMLElement = HTMLElementCast::from_ref(self);
        Some(htmlelement as &VirtualMethods)
    }

    fn after_set_attr(&self, attr: &Attr) {
        if let Some(ref s) = self.super_type() {
            s.after_set_attr(attr);
        }
        let node = NodeCast::from_ref(self);
        if attr.local_name() == &atom!("src") && !self.parser_inserted.get() && node.is_in_doc() {
            self.prepare();
        }
    }

    fn children_changed(&self, mutation: &ChildrenMutation) {
        if let Some(ref s) = self.super_type() {
            s.children_changed(mutation);
        }
        let node = NodeCast::from_ref(self);
        if !self.parser_inserted.get() && node.is_in_doc() {
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

        // https://whatwg.org/html/#already-started
        if self.already_started.get() {
            let copy_elem = HTMLScriptElementCast::to_ref(copy).unwrap();
            copy_elem.mark_already_started();
        }
    }
}

impl HTMLScriptElementMethods for HTMLScriptElement {
    make_url_getter!(Src);

    make_setter!(SetSrc, "src");

    // https://www.whatwg.org/html/#dom-script-text
    fn Text(&self) -> DOMString {
        Node::collect_text_contents(NodeCast::from_ref(self).children())
    }

    // https://www.whatwg.org/html/#dom-script-text
    fn SetText(&self, value: DOMString) {
        let node = NodeCast::from_ref(self);
        node.SetTextContent(Some(value))
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
            target.r().dispatch_error_event();
        } else {
            target.r().dispatch_load_event();
        }
    }
}

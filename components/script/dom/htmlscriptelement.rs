/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::ascii::AsciiExt;

use dom::attr::Attr;
use dom::attr::AttrHelpers;
use dom::bindings::codegen::Bindings::AttrBinding::AttrMethods;
use dom::bindings::codegen::Bindings::HTMLScriptElementBinding;
use dom::bindings::codegen::Bindings::HTMLScriptElementBinding::HTMLScriptElementMethods;
use dom::bindings::codegen::Bindings::NodeBinding::NodeMethods;
use dom::bindings::codegen::InheritTypes::{HTMLScriptElementDerived, HTMLScriptElementCast};
use dom::bindings::codegen::InheritTypes::{ElementCast, HTMLElementCast, NodeCast};
use dom::bindings::codegen::InheritTypes::EventTargetCast;
use dom::bindings::global::GlobalRef;
use dom::bindings::js::{JSRef, Temporary, OptionalRootable};
use dom::bindings::refcounted::Trusted;
use dom::document::Document;
use dom::element::{Element, AttributeHandlers, ElementCreator};
use dom::eventtarget::{EventTarget, EventTargetTypeId};
use dom::event::{Event, EventBubbles, EventCancelable, EventHelpers};
use dom::element::ElementTypeId;
use dom::htmlelement::{HTMLElement, HTMLElementTypeId};
use dom::node::{Node, NodeHelpers, NodeTypeId, window_from_node, CloneChildrenFlag};
use dom::virtualmethods::VirtualMethods;
use dom::window::ScriptHelpers;
use script_task::{ScriptMsg, Runnable};

use encoding::all::UTF_8;
use encoding::types::{Encoding, DecoderTrap};
use net::resource_task::load_whole_resource;
use util::str::{DOMString, HTML_SPACE_CHARACTERS, StaticStringVec};
use std::borrow::ToOwned;
use std::cell::Cell;
use string_cache::Atom;
use url::UrlParser;

#[dom_struct]
pub struct HTMLScriptElement {
    htmlelement: HTMLElement,

    /// https://html.spec.whatwg.org/multipage/scripting.html#already-started
    already_started: Cell<bool>,

    /// https://html.spec.whatwg.org/multipage/scripting.html#parser-inserted
    parser_inserted: Cell<bool>,

    /// https://html.spec.whatwg.org/multipage/scripting.html#non-blocking
    ///
    /// (currently unused)
    non_blocking: Cell<bool>,

    /// https://html.spec.whatwg.org/multipage/scripting.html#ready-to-be-parser-executed
    ///
    /// (currently unused)
    ready_to_be_parser_executed: Cell<bool>,
}

impl HTMLScriptElementDerived for EventTarget {
    fn is_htmlscriptelement(&self) -> bool {
        *self.type_id() == EventTargetTypeId::Node(NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLScriptElement)))
    }
}

impl HTMLScriptElement {
    fn new_inherited(localName: DOMString, prefix: Option<DOMString>, document: JSRef<Document>,
                     creator: ElementCreator) -> HTMLScriptElement {
        HTMLScriptElement {
            htmlelement: HTMLElement::new_inherited(HTMLElementTypeId::HTMLScriptElement, localName, prefix, document),
            already_started: Cell::new(false),
            parser_inserted: Cell::new(creator == ElementCreator::ParserCreated),
            non_blocking: Cell::new(creator != ElementCreator::ParserCreated),
            ready_to_be_parser_executed: Cell::new(false),
        }
    }

    #[allow(unrooted_must_root)]
    pub fn new(localName: DOMString, prefix: Option<DOMString>, document: JSRef<Document>,
               creator: ElementCreator) -> Temporary<HTMLScriptElement> {
        let element = HTMLScriptElement::new_inherited(localName, prefix, document, creator);
        Node::reflect_node(box element, document, HTMLScriptElementBinding::Wrap)
    }
}

pub trait HTMLScriptElementHelpers {
    /// Prepare a script (<http://www.whatwg.org/html/#prepare-a-script>)
    fn prepare(self);

    /// Prepare a script, steps 6 and 7.
    fn is_javascript(self) -> bool;

    /// Set the "already started" flag (<https://whatwg.org/html/#already-started>)
    fn mark_already_started(self);

    /// Dispatch load event.
    fn dispatch_load_event(self);
}

/// Supported script types as defined by
/// <http://whatwg.org/html/#support-the-scripting-language>.
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

enum ScriptOrigin {
    Internal,
    External,
}

impl<'a> HTMLScriptElementHelpers for JSRef<'a, HTMLScriptElement> {
    fn prepare(self) {
        // https://html.spec.whatwg.org/multipage/scripting.html#prepare-a-script
        // Step 1.
        if self.already_started.get() {
            return;
        }
        // Step 2.
        let was_parser_inserted = self.parser_inserted.get();
        self.parser_inserted.set(false);

        // Step 3.
        let element: JSRef<Element> = ElementCast::from_ref(self);
        if was_parser_inserted && element.has_attribute(&atom!("async")) {
            self.non_blocking.set(true);
        }
        // Step 4.
        let text = self.Text();
        if text.len() == 0 && !element.has_attribute(&atom!("src")) {
            return;
        }
        // Step 5.
        let node: JSRef<Node> = NodeCast::from_ref(self);
        if !node.is_in_doc() {
            return;
        }
        // Step 6, 7.
        if !self.is_javascript() {
            return;
        }
        // Step 8.
        if was_parser_inserted {
            self.parser_inserted.set(true);
            self.non_blocking.set(false);
        }
        // Step 9.
        self.already_started.set(true);

        // Step 10.
        // TODO: If the element is flagged as "parser-inserted", but the element's node document is
        // not the Document of the parser that created the element, then abort these steps.

        // Step 11.
        // TODO: If scripting is disabled for the script element, then the user agent must abort
        // these steps at this point. The script is not executed.

        // Step 12.
        // TODO: If the script element has an `event` attribute and a `for` attribute, then run
        // these substeps...

        // Step 13.
        // TODO: If the script element has a `charset` attribute, then let the script block's
        // character encoding for this script element be the result of getting an encoding from the
        // value of the `charset` attribute.

        // Step 14 and 15.
        // TODO: Add support for the `defer` and `async` attributes.  (For now, we fetch all
        // scripts synchronously and execute them immediately.)
        let window = window_from_node(self).root();
        let window = window.r();
        let page = window.page();
        let base_url = page.get_url();

        let (origin, source, url) = match element.get_attribute(ns!(""), &atom!("src")).root() {
            Some(src) => {
                if src.r().Value().is_empty() {
                    // TODO: queue a task to fire a simple event named `error` at the element
                    return;
                }
                match UrlParser::new().base_url(&base_url).parse(src.r().Value().as_slice()) {
                    Ok(url) => {
                        // TODO: Do a potentially CORS-enabled fetch with the mode being the current
                        // state of the element's `crossorigin` content attribute, the origin being
                        // the origin of the script element's node document, and the default origin
                        // behaviour set to taint.
                        match load_whole_resource(&page.resource_task, url) {
                            Ok((metadata, bytes)) => {
                                // TODO: use the charset from step 13.
                                let source = UTF_8.decode(bytes.as_slice(), DecoderTrap::Replace).unwrap();
                                (ScriptOrigin::External, source, metadata.final_url)
                            }
                            Err(_) => {
                                error!("error loading script {}", src.r().Value());
                                return;
                            }
                        }
                    }
                    Err(_) => {
                        // TODO: queue a task to fire a simple event named `error` at the element
                        error!("error parsing URL for script {}", src.r().Value());
                        return;
                    }
                }
            }
            None => (ScriptOrigin::Internal, text, base_url)
        };

        window.evaluate_script_on_global_with_result(source.as_slice(), url.serialize().as_slice());

        // https://html.spec.whatwg.org/multipage/scripting.html#execute-the-script-block
        // step 2, substep 10
        match origin {
            ScriptOrigin::External => {
                self.dispatch_load_event();
            }
            ScriptOrigin::Internal => {
                let chan = window.script_chan();
                let handler = Trusted::new(window.get_cx(), self, chan.clone());
                chan.send(ScriptMsg::RunnableMsg(box handler));
            }
        }
    }

    fn dispatch_load_event(self) {
        let window = window_from_node(self).root();
        let window = window.r();
        let event = Event::new(GlobalRef::Window(window),
                               "load".to_owned(),
                               EventBubbles::DoesNotBubble,
                               EventCancelable::NotCancelable).root();
        let target: JSRef<EventTarget> = EventTargetCast::from_ref(self);
        event.r().fire(target);
    }

    fn is_javascript(self) -> bool {
        let element: JSRef<Element> = ElementCast::from_ref(self);
        match element.get_attribute(ns!(""), &atom!("type")).root().map(|s| s.r().Value()) {
            Some(ref s) if s.is_empty() => {
                // type attr exists, but empty means js
                debug!("script type empty, inferring js");
                true
            },
            Some(ref s) => {
                debug!("script type={}", *s);
                SCRIPT_JS_MIMES.contains(&s.to_ascii_lowercase().as_slice().trim_matches(HTML_SPACE_CHARACTERS))
            },
            None => {
                debug!("no script type");
                match element.get_attribute(ns!(""), &atom!("language"))
                             .root()
                             .map(|s| s.r().Value()) {
                    Some(ref s) if s.is_empty() => {
                        debug!("script language empty, inferring js");
                        true
                    },
                    Some(ref s) => {
                        debug!("script language={}", *s);
                        SCRIPT_JS_MIMES.contains(&format!("text/{}", s).to_ascii_lowercase().as_slice())
                    },
                    None => {
                        debug!("no script type or language, inferring js");
                        true
                    }
                }
            }
        }
    }

    fn mark_already_started(self) {
        self.already_started.set(true);
    }
}

impl<'a> VirtualMethods for JSRef<'a, HTMLScriptElement> {
    fn super_type<'b>(&'b self) -> Option<&'b VirtualMethods> {
        let htmlelement: &JSRef<HTMLElement> = HTMLElementCast::from_borrowed_ref(self);
        Some(htmlelement as &VirtualMethods)
    }

    fn after_set_attr(&self, attr: JSRef<Attr>) {
        match self.super_type() {
            Some(ref s) => s.after_set_attr(attr),
            _ => (),
        }
        let node: JSRef<Node> = NodeCast::from_ref(*self);
        if attr.local_name() == &atom!("src") && !self.parser_inserted.get() && node.is_in_doc() {
            self.prepare();
        }
    }

    fn child_inserted(&self, child: JSRef<Node>) {
        match self.super_type() {
            Some(ref s) => s.child_inserted(child),
            _ => (),
        }
        let node: JSRef<Node> = NodeCast::from_ref(*self);
        if !self.parser_inserted.get() && node.is_in_doc() {
            self.prepare();
        }
    }

    fn bind_to_tree(&self, tree_in_doc: bool) {
        match self.super_type() {
            Some(ref s) => s.bind_to_tree(tree_in_doc),
            _ => ()
        }

        if tree_in_doc && !self.parser_inserted.get() {
            self.prepare();
        }
    }

    fn cloning_steps(&self, copy: JSRef<Node>, maybe_doc: Option<JSRef<Document>>,
                     clone_children: CloneChildrenFlag) {
        match self.super_type() {
            Some(ref s) => s.cloning_steps(copy, maybe_doc, clone_children),
            _ => (),
        }

        // https://whatwg.org/html/#already-started
        if self.already_started.get() {
            let copy_elem: JSRef<HTMLScriptElement> = HTMLScriptElementCast::to_ref(copy).unwrap();
            copy_elem.mark_already_started();
        }
    }
}

impl<'a> HTMLScriptElementMethods for JSRef<'a, HTMLScriptElement> {
    make_url_getter!(Src);

    make_setter!(SetSrc, "src");

    // http://www.whatwg.org/html/#dom-script-text
    fn Text(self) -> DOMString {
        let node: JSRef<Node> = NodeCast::from_ref(self);
        Node::collect_text_contents(node.children())
    }

    // http://www.whatwg.org/html/#dom-script-text
    fn SetText(self, value: DOMString) {
        let node: JSRef<Node> = NodeCast::from_ref(self);
        node.SetTextContent(Some(value))
    }
}

impl Runnable for Trusted<HTMLScriptElement> {
    fn handler(self: Box<Trusted<HTMLScriptElement>>) {
        let target = self.to_temporary().root();
        target.r().dispatch_load_event();
    }
}

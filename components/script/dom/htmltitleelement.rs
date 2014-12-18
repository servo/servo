/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::HTMLTitleElementBinding;
use dom::bindings::codegen::Bindings::HTMLTitleElementBinding::HTMLTitleElementMethods;
use dom::bindings::codegen::Bindings::NodeBinding::NodeMethods;
use dom::bindings::codegen::InheritTypes::{HTMLElementCast, HTMLTitleElementDerived, NodeCast};
use dom::bindings::codegen::InheritTypes::{TextCast};
use dom::bindings::js::{JSRef, Temporary};
use dom::bindings::utils::{Reflectable, Reflector};
use dom::document::{Document, DocumentHelpers};
use dom::element::ElementTypeId;
use dom::eventtarget::{EventTarget, EventTargetTypeId};
use dom::htmlelement::HTMLElement;
use dom::node::{Node, NodeHelpers, NodeTypeId};
use dom::text::Text;
use dom::virtualmethods::VirtualMethods;
use servo_util::str::DOMString;

#[dom_struct]
pub struct HTMLTitleElement {
    htmlelement: HTMLElement,
}

impl HTMLTitleElementDerived for EventTarget {
    fn is_htmltitleelement(&self) -> bool {
        *self.type_id() == EventTargetTypeId::Node(NodeTypeId::Element(ElementTypeId::HTMLTitleElement))
    }
}

impl HTMLTitleElement {
    fn new_inherited(localName: DOMString, prefix: Option<DOMString>, document: JSRef<Document>) -> HTMLTitleElement {
        HTMLTitleElement {
            htmlelement: HTMLElement::new_inherited(ElementTypeId::HTMLTitleElement, localName, prefix, document)
        }
    }

    #[allow(unrooted_must_root)]
    pub fn new(localName: DOMString, prefix: Option<DOMString>, document: JSRef<Document>) -> Temporary<HTMLTitleElement> {
        let element = HTMLTitleElement::new_inherited(localName, prefix, document);
        Node::reflect_node(box element, document, HTMLTitleElementBinding::Wrap)
    }
}

impl<'a> HTMLTitleElementMethods for JSRef<'a, HTMLTitleElement> {
    // http://www.whatwg.org/html/#dom-title-text
    fn Text(self) -> DOMString {
        let node: JSRef<Node> = NodeCast::from_ref(self);
        let mut content = String::new();
        for child in node.children() {
            let text: Option<JSRef<Text>> = TextCast::to_ref(child);
            match text {
                Some(text) => content.push_str(text.characterdata().data().as_slice()),
                None => (),
            }
        }
        content
    }

    // http://www.whatwg.org/html/#dom-title-text
    fn SetText(self, value: DOMString) {
        let node: JSRef<Node> = NodeCast::from_ref(self);
        node.SetTextContent(Some(value))
    }
}

impl Reflectable for HTMLTitleElement {
    fn reflector<'a>(&'a self) -> &'a Reflector {
        self.htmlelement.reflector()
    }
}

impl<'a> VirtualMethods for JSRef<'a, HTMLTitleElement> {
    fn super_type<'a>(&'a self) -> Option<&'a VirtualMethods> {
        let htmlelement: &JSRef<HTMLElement> = HTMLElementCast::from_borrowed_ref(self);
        Some(htmlelement as &VirtualMethods)
    }

    fn bind_to_tree(&self, is_in_doc: bool) {
        let node: JSRef<Node> = NodeCast::from_ref(*self);
        if is_in_doc {
            let document = node.owner_doc().root();
            document.send_title_to_compositor()
        }
    }
}


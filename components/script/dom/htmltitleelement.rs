/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::HTMLTitleElementBinding;
use dom::bindings::codegen::Bindings::HTMLTitleElementBinding::HTMLTitleElementMethods;
use dom::bindings::codegen::Bindings::NodeBinding::NodeMethods;
use dom::bindings::codegen::InheritTypes::{CharacterDataCast, TextCast};
use dom::bindings::codegen::InheritTypes::{HTMLElementCast, HTMLTitleElementDerived, NodeCast};
use dom::bindings::js::Root;
use dom::characterdata::CharacterDataHelpers;
use dom::document::{Document, DocumentHelpers};
use dom::element::ElementTypeId;
use dom::eventtarget::{EventTarget, EventTargetTypeId};
use dom::htmlelement::{HTMLElement, HTMLElementTypeId};
use dom::node::{ChildrenMutation, Node, NodeHelpers, NodeTypeId};
use dom::text::Text;
use dom::virtualmethods::VirtualMethods;
use util::str::DOMString;

#[dom_struct]
#[derive(HeapSizeOf)]
pub struct HTMLTitleElement {
    htmlelement: HTMLElement,
}

impl HTMLTitleElementDerived for EventTarget {
    fn is_htmltitleelement(&self) -> bool {
        *self.type_id() ==
            EventTargetTypeId::Node(
                NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLTitleElement)))
    }
}

impl HTMLTitleElement {
    fn new_inherited(localName: DOMString, prefix: Option<DOMString>, document: &Document) -> HTMLTitleElement {
        HTMLTitleElement {
            htmlelement: HTMLElement::new_inherited(HTMLElementTypeId::HTMLTitleElement, localName, prefix, document)
        }
    }

    #[allow(unrooted_must_root)]
    pub fn new(localName: DOMString,
               prefix: Option<DOMString>,
               document: &Document) -> Root<HTMLTitleElement> {
        let element = HTMLTitleElement::new_inherited(localName, prefix, document);
        Node::reflect_node(box element, document, HTMLTitleElementBinding::Wrap)
    }
}

impl<'a> HTMLTitleElementMethods for &'a HTMLTitleElement {
    // https://www.whatwg.org/html/#dom-title-text
    fn Text(self) -> DOMString {
        let node = NodeCast::from_ref(self);
        let mut content = String::new();
        for child in node.children() {
            let text: Option<&Text> = TextCast::to_ref(child.r());
            match text {
                Some(text) => content.push_str(&CharacterDataCast::from_ref(text).data()),
                None => (),
            }
        }
        content
    }

    // https://www.whatwg.org/html/#dom-title-text
    fn SetText(self, value: DOMString) {
        let node = NodeCast::from_ref(self);
        node.SetTextContent(Some(value))
    }
}

impl<'a> VirtualMethods for &'a HTMLTitleElement {
    fn super_type<'b>(&'b self) -> Option<&'b VirtualMethods> {
        let htmlelement: &&HTMLElement = HTMLElementCast::from_borrowed_ref(self);
        Some(htmlelement as &VirtualMethods)
    }

    fn children_changed(&self, mutation: &ChildrenMutation) {
        if let Some(ref s) = self.super_type() {
            s.children_changed(mutation);
        }
        let node = NodeCast::from_ref(*self);
        if node.is_in_doc() {
            node.owner_doc().title_changed();
        }
    }

    fn bind_to_tree(&self, is_in_doc: bool) {
        let node = NodeCast::from_ref(*self);
        if is_in_doc {
            let document = node.owner_doc();
            document.r().title_changed();
        }
    }
}

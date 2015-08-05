/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::HTMLHeadElementBinding;
use dom::bindings::codegen::InheritTypes::{HTMLElementCast, HTMLHeadElementDerived};
use dom::bindings::js::Root;
use dom::document::Document;
use dom::eventtarget::{EventTarget, EventTargetTypeId};
use dom::element::ElementTypeId;
use dom::htmlelement::{HTMLElement, HTMLElementTypeId};
use dom::node::{Node, NodeTypeId};
use dom::userscripts::load_script;
use dom::virtualmethods::VirtualMethods;
use util::str::DOMString;

#[dom_struct]
#[derive(HeapSizeOf)]
pub struct HTMLHeadElement {
    htmlelement: HTMLElement
}

impl HTMLHeadElementDerived for EventTarget {
    fn is_htmlheadelement(&self) -> bool {
        *self.type_id() ==
            EventTargetTypeId::Node(
                NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLHeadElement)))
    }
}

impl HTMLHeadElement {
    fn new_inherited(localName: DOMString,
                     prefix: Option<DOMString>,
                     document: &Document) -> HTMLHeadElement {
        HTMLHeadElement {
            htmlelement: HTMLElement::new_inherited(HTMLElementTypeId::HTMLHeadElement, localName, prefix, document)
        }
    }

    #[allow(unrooted_must_root)]
    pub fn new(localName: DOMString,
               prefix: Option<DOMString>,
               document: &Document) -> Root<HTMLHeadElement> {
        let element = HTMLHeadElement::new_inherited(localName, prefix, document);
        Node::reflect_node(box element, document, HTMLHeadElementBinding::Wrap)
    }
}

impl<'a> VirtualMethods for &'a HTMLHeadElement {
    fn super_type<'b>(&'b self) -> Option<&'b VirtualMethods> {
        let htmlelement: &&HTMLElement = HTMLElementCast::from_borrowed_ref(self);
        Some(htmlelement as &VirtualMethods)
    }
    fn bind_to_tree(&self, _tree_in_doc: bool) {
        load_script(*self);
    }
}

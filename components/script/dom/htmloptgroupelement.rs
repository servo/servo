/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::attr::Attr;
use dom::attr::AttrHelpers;
use dom::bindings::codegen::Bindings::HTMLOptGroupElementBinding;
use dom::bindings::codegen::Bindings::HTMLOptGroupElementBinding::HTMLOptGroupElementMethods;
use dom::bindings::codegen::InheritTypes::{HTMLElementCast, NodeCast};
use dom::bindings::codegen::InheritTypes::{HTMLOptGroupElementDerived, HTMLOptionElementDerived};
use dom::bindings::js::{JSRef, Rootable, Temporary};
use dom::document::Document;
use dom::element::AttributeHandlers;
use dom::eventtarget::{EventTarget, EventTargetTypeId};
use dom::element::ElementTypeId;
use dom::htmlelement::{HTMLElement, HTMLElementTypeId};
use dom::node::{DisabledStateHelpers, Node, NodeHelpers, NodeTypeId};
use dom::virtualmethods::VirtualMethods;

use util::str::DOMString;

#[dom_struct]
pub struct HTMLOptGroupElement {
    htmlelement: HTMLElement
}

impl HTMLOptGroupElementDerived for EventTarget {
    fn is_htmloptgroupelement(&self) -> bool {
        *self.type_id() ==
            EventTargetTypeId::Node(
                NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLOptGroupElement)))
    }
}

impl HTMLOptGroupElement {
    fn new_inherited(localName: DOMString,
                     prefix: Option<DOMString>,
                     document: JSRef<Document>) -> HTMLOptGroupElement {
        HTMLOptGroupElement {
            htmlelement:
                HTMLElement::new_inherited(HTMLElementTypeId::HTMLOptGroupElement, localName, prefix, document)
        }
    }

    #[allow(unrooted_must_root)]
    pub fn new(localName: DOMString,
               prefix: Option<DOMString>,
               document: JSRef<Document>) -> Temporary<HTMLOptGroupElement> {
        let element = HTMLOptGroupElement::new_inherited(localName, prefix, document);
        Node::reflect_node(box element, document, HTMLOptGroupElementBinding::Wrap)
    }
}

impl<'a> HTMLOptGroupElementMethods for JSRef<'a, HTMLOptGroupElement> {
    // https://www.whatwg.org/html#dom-optgroup-disabled
    make_bool_getter!(Disabled);

    // https://www.whatwg.org/html#dom-optgroup-disabled
    make_bool_setter!(SetDisabled, "disabled");
}

impl<'a> VirtualMethods for JSRef<'a, HTMLOptGroupElement> {
    fn super_type<'b>(&'b self) -> Option<&'b VirtualMethods> {
        let htmlelement: &JSRef<HTMLElement> = HTMLElementCast::from_borrowed_ref(self);
        Some(htmlelement as &VirtualMethods)
    }

    fn after_set_attr(&self, attr: JSRef<Attr>) {
        if let Some(ref s) = self.super_type() {
            s.after_set_attr(attr);
        }

        match attr.local_name() {
            &atom!("disabled") => {
                let node: JSRef<Node> = NodeCast::from_ref(*self);
                node.set_disabled_state(true);
                node.set_enabled_state(false);
                for child in node.children() {
                    let child = child.root();
                    if child.r().is_htmloptionelement() {
                        child.r().set_disabled_state(true);
                        child.r().set_enabled_state(false);
                    }
                }
            },
            _ => (),
        }
    }

    fn before_remove_attr(&self, attr: JSRef<Attr>) {
        if let Some(ref s) = self.super_type() {
            s.before_remove_attr(attr);
        }

        match attr.local_name() {
            &atom!("disabled") => {
                let node: JSRef<Node> = NodeCast::from_ref(*self);
                node.set_disabled_state(false);
                node.set_enabled_state(true);
                for child in node.children() {
                    let child = child.root();
                    if child.r().is_htmloptionelement() {
                        child.r().check_disabled_attribute();
                    }
                }
            },
            _ => ()
        }
    }
}


/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::HTMLLinkElementBinding;
use dom::bindings::codegen::InheritTypes::HTMLLinkElementDerived;
use dom::bindings::codegen::InheritTypes::{HTMLElementCast, NodeCast};
use dom::bindings::js::{JSRef, Temporary};
use dom::bindings::utils::{Reflectable, Reflector};
use dom::document::Document;
use dom::element::HTMLLinkElementTypeId;
use dom::eventtarget::{EventTarget, NodeTargetTypeId};
use dom::htmlelement::HTMLElement;
use dom::node::{Node, NodeHelpers, ElementNodeTypeId};
use dom::virtualmethods::VirtualMethods;

use servo_util::atom::Atom;
use servo_util::str::DOMString;

#[deriving(Encodable)]
pub struct HTMLLinkElement {
    pub htmlelement: HTMLElement,
}

impl HTMLLinkElementDerived for EventTarget {
    fn is_htmllinkelement(&self) -> bool {
        self.type_id == NodeTargetTypeId(ElementNodeTypeId(HTMLLinkElementTypeId))
    }
}

impl HTMLLinkElement {
    pub fn new_inherited(localName: DOMString, document: &JSRef<Document>) -> HTMLLinkElement {
        HTMLLinkElement {
            htmlelement: HTMLElement::new_inherited(HTMLLinkElementTypeId, localName, document)
        }
    }

    pub fn new(localName: DOMString, document: &JSRef<Document>) -> Temporary<HTMLLinkElement> {
        let element = HTMLLinkElement::new_inherited(localName, document);
        Node::reflect_node(box element, document, HTMLLinkElementBinding::Wrap)
    }
}

impl<'a> VirtualMethods for JSRef<'a, HTMLLinkElement> {
    fn super_type<'a>(&'a self) -> Option<&'a VirtualMethods> {
        let htmlelement: &JSRef<HTMLElement> = HTMLElementCast::from_ref(self);
        Some(htmlelement as &VirtualMethods)
    }

    fn after_set_attr(&self, name: &Atom, value: DOMString) {
        match self.super_type() {
            Some(ref s) => s.after_set_attr(name, value.clone()),
            _ => (),
        }

        let node: &JSRef<Node> = NodeCast::from_ref(self);
        match name.as_slice() {
            "href" => node.set_enabled_state(true),
            _ => ()
        }
    }

    fn before_remove_attr(&self, name: &Atom, value: DOMString) {
        match self.super_type() {
            Some(ref s) => s.before_remove_attr(name, value.clone()),
            _ => (),
        }

        let node: &JSRef<Node> = NodeCast::from_ref(self);
        match name.as_slice() {
            "href" => node.set_enabled_state(false),
            _ => ()
        }
    }
}

impl Reflectable for HTMLLinkElement {
    fn reflector<'a>(&'a self) -> &'a Reflector {
        self.htmlelement.reflector()
    }
}

/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::HTMLOptGroupElementBinding;
use dom::bindings::codegen::Bindings::HTMLOptGroupElementBinding::HTMLOptGroupElementMethods;
use dom::bindings::codegen::InheritTypes::{ElementCast, HTMLElementCast, NodeCast};
use dom::bindings::codegen::InheritTypes::{HTMLOptGroupElementDerived, HTMLOptionElementDerived};
use dom::bindings::js::{JSRef, Temporary};
use dom::bindings::utils::{Reflectable, Reflector};
use dom::document::Document;
use dom::element::{AttributeHandlers, Element, HTMLOptGroupElementTypeId};
use dom::eventtarget::{EventTarget, NodeTargetTypeId};
use dom::htmlelement::HTMLElement;
use dom::node::{DisabledStateHelpers, Node, NodeHelpers, ElementNodeTypeId};
use dom::virtualmethods::VirtualMethods;

use servo_util::str::DOMString;
use string_cache::Atom;

#[jstraceable]
#[must_root]
pub struct HTMLOptGroupElement {
    pub htmlelement: HTMLElement
}

impl HTMLOptGroupElementDerived for EventTarget {
    fn is_htmloptgroupelement(&self) -> bool {
        self.type_id == NodeTargetTypeId(ElementNodeTypeId(HTMLOptGroupElementTypeId))
    }
}

impl HTMLOptGroupElement {
    fn new_inherited(localName: DOMString, prefix: Option<DOMString>, document: JSRef<Document>) -> HTMLOptGroupElement {
        HTMLOptGroupElement {
            htmlelement: HTMLElement::new_inherited(HTMLOptGroupElementTypeId, localName, prefix, document)
        }
    }

    #[allow(unrooted_must_root)]
    pub fn new(localName: DOMString, prefix: Option<DOMString>, document: JSRef<Document>) -> Temporary<HTMLOptGroupElement> {
        let element = HTMLOptGroupElement::new_inherited(localName, prefix, document);
        Node::reflect_node(box element, document, HTMLOptGroupElementBinding::Wrap)
    }
}

impl<'a> HTMLOptGroupElementMethods for JSRef<'a, HTMLOptGroupElement> {
    // http://www.whatwg.org/html#dom-optgroup-disabled
    make_bool_getter!(Disabled)

    // http://www.whatwg.org/html#dom-optgroup-disabled
    fn SetDisabled(self, disabled: bool) {
        let elem: JSRef<Element> = ElementCast::from_ref(self);
        elem.set_bool_attribute("disabled", disabled)
    }
}

impl<'a> VirtualMethods for JSRef<'a, HTMLOptGroupElement> {
    fn super_type<'a>(&'a self) -> Option<&'a VirtualMethods> {
        let htmlelement: &JSRef<HTMLElement> = HTMLElementCast::from_borrowed_ref(self);
        Some(htmlelement as &VirtualMethods)
    }

    fn after_set_attr(&self, name: &Atom, value: DOMString) {
        match self.super_type() {
            Some(ref s) => s.after_set_attr(name, value.clone()),
            _ => (),
        }

        let node: JSRef<Node> = NodeCast::from_ref(*self);
        match name.as_slice() {
            "disabled" => {
                node.set_disabled_state(true);
                node.set_enabled_state(false);
                for child in node.children().filter(|child| child.is_htmloptionelement()) {
                    child.set_disabled_state(true);
                    child.set_enabled_state(false);
                }
            },
            _ => ()
        }
    }

    fn before_remove_attr(&self, name: &Atom, value: DOMString) {
        match self.super_type() {
            Some(ref s) => s.before_remove_attr(name, value),
            _ => (),
        }

        let node: JSRef<Node> = NodeCast::from_ref(*self);
        match name.as_slice() {
            "disabled" => {
                node.set_disabled_state(false);
                node.set_enabled_state(true);
                for child in node.children().filter(|child| child.is_htmloptionelement()) {
                    child.check_disabled_attribute();
                }
            },
            _ => ()
        }
    }
}

impl Reflectable for HTMLOptGroupElement {
    fn reflector<'a>(&'a self) -> &'a Reflector {
        self.htmlelement.reflector()
    }
}

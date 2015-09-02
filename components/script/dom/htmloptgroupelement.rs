/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::attr::Attr;
use dom::bindings::codegen::Bindings::HTMLOptGroupElementBinding;
use dom::bindings::codegen::Bindings::HTMLOptGroupElementBinding::HTMLOptGroupElementMethods;
use dom::bindings::codegen::InheritTypes::{HTMLElementCast, NodeCast};
use dom::bindings::codegen::InheritTypes::{HTMLOptGroupElementDerived, HTMLOptionElementDerived};
use dom::bindings::js::Root;
use dom::document::Document;
use dom::element::{AttributeMutation, ElementTypeId};
use dom::eventtarget::{EventTarget, EventTargetTypeId};
use dom::htmlelement::{HTMLElement, HTMLElementTypeId};
use dom::node::{Node, NodeTypeId};
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
                     document: &Document) -> HTMLOptGroupElement {
        HTMLOptGroupElement {
            htmlelement:
                HTMLElement::new_inherited(HTMLElementTypeId::HTMLOptGroupElement, localName, prefix, document)
        }
    }

    #[allow(unrooted_must_root)]
    pub fn new(localName: DOMString,
               prefix: Option<DOMString>,
               document: &Document) -> Root<HTMLOptGroupElement> {
        let element = HTMLOptGroupElement::new_inherited(localName, prefix, document);
        Node::reflect_node(box element, document, HTMLOptGroupElementBinding::Wrap)
    }
}

impl HTMLOptGroupElementMethods for HTMLOptGroupElement {
    // https://www.whatwg.org/html#dom-optgroup-disabled
    make_bool_getter!(Disabled);

    // https://www.whatwg.org/html#dom-optgroup-disabled
    make_bool_setter!(SetDisabled, "disabled");
}

impl VirtualMethods for HTMLOptGroupElement {
    fn super_type<'b>(&'b self) -> Option<&'b VirtualMethods> {
        let htmlelement: &HTMLElement = HTMLElementCast::from_ref(self);
        Some(htmlelement as &VirtualMethods)
    }

    fn attribute_mutated(&self, attr: &Attr, mutation: AttributeMutation) {
        self.super_type().unwrap().attribute_mutated(attr, mutation);
        match attr.local_name() {
            &atom!(disabled) => {
                let disabled_state = match mutation {
                    AttributeMutation::Set(None) => true,
                    AttributeMutation::Set(Some(_)) => {
                        // Option group was already disabled.
                        return;
                    },
                    AttributeMutation::Removed => false,
                };
                let node = NodeCast::from_ref(self);
                node.set_disabled_state(disabled_state);
                node.set_enabled_state(!disabled_state);
                let options = node.children().filter(|child| {
                    child.is_htmloptionelement()
                });
                if disabled_state {
                    for option in options {
                        option.set_disabled_state(true);
                        option.set_enabled_state(false);
                    }
                } else {
                    for option in options {
                        option.check_disabled_attribute();
                    }
                }
            },
            _ => {},
        }
    }
}

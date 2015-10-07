/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::attr::Attr;
use dom::bindings::codegen::Bindings::HTMLOptGroupElementBinding;
use dom::bindings::codegen::Bindings::HTMLOptGroupElementBinding::HTMLOptGroupElementMethods;
use dom::bindings::conversions::Castable;
use dom::bindings::js::Root;
use dom::document::Document;
use dom::element::AttributeMutation;
use dom::htmlelement::HTMLElement;
use dom::htmloptionelement::HTMLOptionElement;
use dom::node::{IN_ENABLED_STATE, Node, NodeFlags};
use dom::virtualmethods::VirtualMethods;
use util::str::DOMString;

#[dom_struct]
pub struct HTMLOptGroupElement {
    htmlelement: HTMLElement
}

impl HTMLOptGroupElement {
    fn new_inherited(localName: DOMString,
                     prefix: Option<DOMString>,
                     document: &Document) -> HTMLOptGroupElement {
        HTMLOptGroupElement {
            htmlelement:
                HTMLElement::new_inherited_with_flags(NodeFlags::new() | IN_ENABLED_STATE,
                                                      localName, prefix, document)
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
    // https://html.spec.whatwg.org/multipage/#dom-optgroup-disabled
    make_bool_getter!(Disabled);

    // https://html.spec.whatwg.org/multipage/#dom-optgroup-disabled
    make_bool_setter!(SetDisabled, "disabled");
}

impl VirtualMethods for HTMLOptGroupElement {
    fn super_type<'b>(&'b self) -> Option<&'b VirtualMethods> {
        Some(self.upcast::<HTMLElement>() as &VirtualMethods)
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
                let node = self.upcast::<Node>();
                node.set_disabled_state(disabled_state);
                node.set_enabled_state(!disabled_state);
                let options = node.children().filter(|child| {
                    child.is::<HTMLOptionElement>()
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

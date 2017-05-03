/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::attr::Attr;
use dom::bindings::codegen::Bindings::HTMLOptGroupElementBinding;
use dom::bindings::codegen::Bindings::HTMLOptGroupElementBinding::HTMLOptGroupElementMethods;
use dom::bindings::inheritance::Castable;
use dom::bindings::js::Root;
use dom::document::Document;
use dom::element::{AttributeMutation, Element};
use dom::htmlelement::HTMLElement;
use dom::htmloptionelement::HTMLOptionElement;
use dom::node::Node;
use dom::virtualmethods::VirtualMethods;
use dom_struct::dom_struct;
use html5ever::{LocalName, Prefix};
use style::element_state::*;

#[dom_struct]
pub struct HTMLOptGroupElement {
    htmlelement: HTMLElement
}

impl HTMLOptGroupElement {
    fn new_inherited(local_name: LocalName,
                     prefix: Option<Prefix>,
                     document: &Document) -> HTMLOptGroupElement {
        HTMLOptGroupElement {
            htmlelement:
                HTMLElement::new_inherited_with_state(IN_ENABLED_STATE,
                                                      local_name, prefix, document)
        }
    }

    #[allow(unrooted_must_root)]
    pub fn new(local_name: LocalName,
               prefix: Option<Prefix>,
               document: &Document) -> Root<HTMLOptGroupElement> {
        Node::reflect_node(box HTMLOptGroupElement::new_inherited(local_name, prefix, document),
                           document,
                           HTMLOptGroupElementBinding::Wrap)
    }
}

impl HTMLOptGroupElementMethods for HTMLOptGroupElement {
    // https://html.spec.whatwg.org/multipage/#dom-optgroup-disabled
    make_bool_getter!(Disabled, "disabled");

    // https://html.spec.whatwg.org/multipage/#dom-optgroup-disabled
    make_bool_setter!(SetDisabled, "disabled");
}

impl VirtualMethods for HTMLOptGroupElement {
    fn super_type(&self) -> Option<&VirtualMethods> {
        Some(self.upcast::<HTMLElement>() as &VirtualMethods)
    }

    fn attribute_mutated(&self, attr: &Attr, mutation: AttributeMutation) {
        self.super_type().unwrap().attribute_mutated(attr, mutation);
        match attr.local_name() {
            &local_name!("disabled") => {
                let disabled_state = match mutation {
                    AttributeMutation::Set(None) => true,
                    AttributeMutation::Set(Some(_)) => {
                        // Option group was already disabled.
                        return;
                    },
                    AttributeMutation::Removed => false,
                };
                let el = self.upcast::<Element>();
                el.set_disabled_state(disabled_state);
                el.set_enabled_state(!disabled_state);
                let options = el.upcast::<Node>().children().filter(|child| {
                    child.is::<HTMLOptionElement>()
                }).map(|child| Root::from_ref(child.downcast::<HTMLOptionElement>().unwrap()));
                if disabled_state {
                    for option in options {
                        let el = option.upcast::<Element>();
                        el.set_disabled_state(true);
                        el.set_enabled_state(false);
                    }
                } else {
                    for option in options {
                        let el = option.upcast::<Element>();
                        el.check_disabled_attribute();
                    }
                }
            },
            _ => {},
        }
    }
}

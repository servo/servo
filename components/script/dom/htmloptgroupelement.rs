/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::attr::Attr;
use dom::bindings::codegen::Bindings::HTMLOptGroupElementBinding;
use dom::bindings::codegen::Bindings::HTMLOptGroupElementBinding::HTMLOptGroupElementMethods;
use dom::bindings::inheritance::Castable;
use dom::bindings::root::DomRoot;
use dom::document::Document;
use dom::element::{AttributeMutation, Element};
use dom::htmlelement::HTMLElement;
use dom::htmloptionelement::HTMLOptionElement;
use dom::node::Node;
use dom::virtualmethods::VirtualMethods;
use dom_struct::dom_struct;
use html5ever::{LocalName, Prefix};
use style::element_state::ElementState;
use typeholder::TypeHolderTrait;

#[dom_struct]
pub struct HTMLOptGroupElement<TH: TypeHolderTrait> {
    htmlelement: HTMLElement<TH>
}

impl<TH: TypeHolderTrait> HTMLOptGroupElement<TH> {
    fn new_inherited(local_name: LocalName,
                     prefix: Option<Prefix>,
                     document: &Document<TH>) -> HTMLOptGroupElement<TH> {
        HTMLOptGroupElement {
            htmlelement:
                HTMLElement::new_inherited_with_state(ElementState::IN_ENABLED_STATE,
                                                      local_name, prefix, document)
        }
    }

    #[allow(unrooted_must_root)]
    pub fn new(local_name: LocalName,
               prefix: Option<Prefix>,
               document: &Document<TH>) -> DomRoot<HTMLOptGroupElement<TH>> {
        Node::<TH>::reflect_node(Box::new(HTMLOptGroupElement::new_inherited(local_name, prefix, document)),
                           document,
                           HTMLOptGroupElementBinding::Wrap)
    }
}

impl<TH: TypeHolderTrait> HTMLOptGroupElementMethods for HTMLOptGroupElement<TH> {
    // https://html.spec.whatwg.org/multipage/#dom-optgroup-disabled
    make_bool_getter!(Disabled, "disabled");

    // https://html.spec.whatwg.org/multipage/#dom-optgroup-disabled
    make_bool_setter!(SetDisabled, "disabled");
}

impl<TH: TypeHolderTrait> VirtualMethods<TH> for HTMLOptGroupElement<TH> {
    fn super_type(&self) -> Option<&VirtualMethods<TH>> {
        Some(self.upcast::<HTMLElement<TH>>() as &VirtualMethods<TH>)
    }

    fn attribute_mutated(&self, attr: &Attr<TH>, mutation: AttributeMutation<TH>) {
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
                    AttributeMutation::_p(_) => unreachable!(),
                };
                let el = self.upcast::<Element<TH>>();
                el.set_disabled_state(disabled_state);
                el.set_enabled_state(!disabled_state);
                let options = el.upcast::<Node<TH>>().children().filter(|child| {
                    child.is::<HTMLOptionElement<TH>>()
                }).map(|child| DomRoot::from_ref(child.downcast::<HTMLOptionElement<TH>>().unwrap()));
                if disabled_state {
                    for option in options {
                        let el = option.upcast::<Element<TH>>();
                        el.set_disabled_state(true);
                        el.set_enabled_state(false);
                    }
                } else {
                    for option in options {
                        let el = option.upcast::<Element<TH>>();
                        el.check_disabled_attribute();
                    }
                }
            },
            _ => {},
        }
    }
}

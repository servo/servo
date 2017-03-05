/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::attr::Attr;
use dom::bindings::codegen::Bindings::HTMLFieldSetElementBinding;
use dom::bindings::codegen::Bindings::HTMLFieldSetElementBinding::HTMLFieldSetElementMethods;
use dom::bindings::inheritance::{Castable, ElementTypeId, HTMLElementTypeId, NodeTypeId};
use dom::bindings::js::Root;
use dom::bindings::str::DOMString;
use dom::document::Document;
use dom::element::{AttributeMutation, Element};
use dom::htmlcollection::{CollectionFilter, HTMLCollection};
use dom::htmlelement::HTMLElement;
use dom::htmlformelement::{FormControl, HTMLFormElement};
use dom::htmllegendelement::HTMLLegendElement;
use dom::node::{Node, window_from_node};
use dom::validitystate::ValidityState;
use dom::virtualmethods::VirtualMethods;
use dom_struct::dom_struct;
use html5ever_atoms::LocalName;
use style::element_state::*;

#[dom_struct]
pub struct HTMLFieldSetElement {
    htmlelement: HTMLElement
}

impl HTMLFieldSetElement {
    fn new_inherited(local_name: LocalName,
                     prefix: Option<DOMString>,
                     document: &Document) -> HTMLFieldSetElement {
        HTMLFieldSetElement {
            htmlelement:
                HTMLElement::new_inherited_with_state(IN_ENABLED_STATE,
                                                      local_name, prefix, document)
        }
    }

    #[allow(unrooted_must_root)]
    pub fn new(local_name: LocalName,
               prefix: Option<DOMString>,
               document: &Document) -> Root<HTMLFieldSetElement> {
        Node::reflect_node(box HTMLFieldSetElement::new_inherited(local_name, prefix, document),
                           document,
                           HTMLFieldSetElementBinding::Wrap)
    }
}

impl HTMLFieldSetElementMethods for HTMLFieldSetElement {
    // https://html.spec.whatwg.org/multipage/#dom-fieldset-elements
    fn Elements(&self) -> Root<HTMLCollection> {
        #[derive(JSTraceable, HeapSizeOf)]
        struct ElementsFilter;
        impl CollectionFilter for ElementsFilter {
            fn filter<'a>(&self, elem: &'a Element, _root: &'a Node) -> bool {
                elem.downcast::<HTMLElement>()
                    .map_or(false, HTMLElement::is_listed_element)
            }
        }
        let filter = box ElementsFilter;
        let window = window_from_node(self);
        HTMLCollection::create(&window, self.upcast(), filter)
    }

    // https://html.spec.whatwg.org/multipage/#dom-cva-validity
    fn Validity(&self) -> Root<ValidityState> {
        let window = window_from_node(self);
        ValidityState::new(&window, self.upcast())
    }

    // https://html.spec.whatwg.org/multipage/#dom-fieldset-disabled
    make_bool_getter!(Disabled, "disabled");

    // https://html.spec.whatwg.org/multipage/#dom-fieldset-disabled
    make_bool_setter!(SetDisabled, "disabled");

    // https://html.spec.whatwg.org/multipage/#dom-fae-form
    fn GetForm(&self) -> Option<Root<HTMLFormElement>> {
        self.form_owner()
    }
}

impl VirtualMethods for HTMLFieldSetElement {
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
                        // Fieldset was already disabled before.
                        return;
                    },
                    AttributeMutation::Removed => false,
                };
                let node = self.upcast::<Node>();
                let el = self.upcast::<Element>();
                el.set_disabled_state(disabled_state);
                el.set_enabled_state(!disabled_state);
                let mut found_legend = false;
                let children = node.children().filter(|node| {
                    if found_legend {
                        true
                    } else if node.is::<HTMLLegendElement>() {
                        found_legend = true;
                        false
                    } else {
                        true
                    }
                });
                let fields = children.flat_map(|child| {
                    child.traverse_preorder().filter(|descendant| {
                        match descendant.type_id() {
                            NodeTypeId::Element(
                                    ElementTypeId::HTMLElement(
                                        HTMLElementTypeId::HTMLButtonElement)) |
                            NodeTypeId::Element(
                                    ElementTypeId::HTMLElement(
                                        HTMLElementTypeId::HTMLInputElement)) |
                            NodeTypeId::Element(
                                    ElementTypeId::HTMLElement(
                                        HTMLElementTypeId::HTMLSelectElement)) |
                            NodeTypeId::Element(
                                    ElementTypeId::HTMLElement(
                                        HTMLElementTypeId::HTMLTextAreaElement)) => {
                                true
                            },
                            _ => false,
                        }
                    })
                });
                if disabled_state {
                    for field in fields {
                        let el = field.downcast::<Element>().unwrap();
                        el.set_disabled_state(true);
                        el.set_enabled_state(false);
                    }
                } else {
                    for field in fields {
                        let el = field.downcast::<Element>().unwrap();
                        el.check_disabled_attribute();
                        el.check_ancestors_disabled_state_for_form_control();
                    }
                }
            },
            _ => {},
        }
    }
}

impl FormControl for HTMLFieldSetElement {}

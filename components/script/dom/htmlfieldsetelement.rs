/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::attr::Attr;
use dom::bindings::codegen::Bindings::HTMLFieldSetElementBinding;
use dom::bindings::codegen::Bindings::HTMLFieldSetElementBinding::HTMLFieldSetElementMethods;
use dom::bindings::inheritance::{Castable, ElementTypeId, HTMLElementTypeId, NodeTypeId};
use dom::bindings::root::{DomRoot, MutNullableDom};
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
use html5ever::{LocalName, Prefix};
use std::default::Default;
use std::marker::PhantomData;
use style::element_state::ElementState;
use typeholder::TypeHolderTrait;

#[dom_struct]
pub struct HTMLFieldSetElement<TH: TypeHolderTrait> {
    htmlelement: HTMLElement<TH>,
    form_owner: MutNullableDom<HTMLFormElement<TH>>,
}

impl<TH: TypeHolderTrait> HTMLFieldSetElement<TH> {
    fn new_inherited(local_name: LocalName,
                     prefix: Option<Prefix>,
                     document: &Document<TH>) -> HTMLFieldSetElement<TH> {
        HTMLFieldSetElement {
            htmlelement:
                HTMLElement::new_inherited_with_state(ElementState::IN_ENABLED_STATE,
                                                      local_name, prefix, document),
            form_owner: Default::default(),
        }
    }

    #[allow(unrooted_must_root)]
    pub fn new(local_name: LocalName,
               prefix: Option<Prefix>,
               document: &Document<TH>) -> DomRoot<HTMLFieldSetElement<TH>> {
        Node::<TH>::reflect_node(Box::new(HTMLFieldSetElement::new_inherited(local_name, prefix, document)),
                           document,
                           HTMLFieldSetElementBinding::Wrap)
    }
}

impl<TH: TypeHolderTrait> HTMLFieldSetElementMethods<TH> for HTMLFieldSetElement<TH> {
    // https://html.spec.whatwg.org/multipage/#dom-fieldset-elements
    fn Elements(&self) -> DomRoot<HTMLCollection<TH>> {
        #[derive(JSTraceable, MallocSizeOf)]
        struct ElementsFilter<THH: TypeHolderTrait + 'static>(PhantomData<THH>);
        impl<THH: TypeHolderTrait> CollectionFilter<THH> for ElementsFilter<THH> {
            fn filter<'a>(&self, elem: &'a Element<THH>, _root: &'a Node<THH>) -> bool {
                elem.downcast::<HTMLElement<THH>>()
                    .map_or(false, HTMLElement::is_listed_element)
            }
        }
        let filter = Box::new(ElementsFilter(Default::default()));
        let window = window_from_node(self);
        HTMLCollection::create(&window, self.upcast(), filter)
    }

    // https://html.spec.whatwg.org/multipage/#dom-cva-validity
    fn Validity(&self) -> DomRoot<ValidityState<TH>> {
        let window = window_from_node(self);
        ValidityState::new(&window, self.upcast())
    }

    // https://html.spec.whatwg.org/multipage/#dom-fieldset-disabled
    make_bool_getter!(Disabled, "disabled");

    // https://html.spec.whatwg.org/multipage/#dom-fieldset-disabled
    make_bool_setter!(SetDisabled, "disabled");

    // https://html.spec.whatwg.org/multipage/#dom-fae-form
    fn GetForm(&self) -> Option<DomRoot<HTMLFormElement<TH>>> {
        self.form_owner()
    }
}

impl<TH: TypeHolderTrait> VirtualMethods<TH> for HTMLFieldSetElement<TH> {
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
                        // Fieldset was already disabled before.
                        return;
                    },
                    AttributeMutation::Removed => false,
                    AttributeMutation::_p(_) => unreachable!(),
                };
                let node = self.upcast::<Node<TH>>();
                let el = self.upcast::<Element<TH>>();
                el.set_disabled_state(disabled_state);
                el.set_enabled_state(!disabled_state);
                let mut found_legend = false;
                let children = node.children().filter(|node| {
                    if found_legend {
                        true
                    } else if node.is::<HTMLLegendElement<TH>>() {
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
                        let el = field.downcast::<Element<TH>>().unwrap();
                        el.set_disabled_state(true);
                        el.set_enabled_state(false);
                    }
                } else {
                    for field in fields {
                        let el = field.downcast::<Element<TH>>().unwrap();
                        el.check_disabled_attribute();
                        el.check_ancestors_disabled_state_for_form_control();
                    }
                }
            },
            &local_name!("form") => {
                self.form_attribute_mutated(mutation);
            },
            _ => {},
        }
    }
}

impl<TH: TypeHolderTrait> FormControl<TH> for HTMLFieldSetElement<TH> {
    fn form_owner(&self) -> Option<DomRoot<HTMLFormElement<TH>>> {
        self.form_owner.get()
    }

    fn set_form_owner(&self, form: Option<&HTMLFormElement<TH>>) {
        self.form_owner.set(form);
    }

    fn to_element<'a>(&'a self) -> &'a Element<TH> {
        self.upcast::<Element<TH>>()
    }
}

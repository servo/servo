/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::default::Default;

use dom_struct::dom_struct;
use html5ever::{local_name, LocalName, Prefix};
use js::rust::HandleObject;
use style_dom::ElementState;

use crate::dom::attr::Attr;
use crate::dom::bindings::codegen::Bindings::HTMLFieldSetElementBinding::HTMLFieldSetElementMethods;
use crate::dom::bindings::inheritance::{Castable, ElementTypeId, HTMLElementTypeId, NodeTypeId};
use crate::dom::bindings::root::{DomRoot, MutNullableDom};
use crate::dom::bindings::str::DOMString;
use crate::dom::customelementregistry::CallbackReaction;
use crate::dom::document::Document;
use crate::dom::element::{AttributeMutation, Element};
use crate::dom::htmlcollection::HTMLCollection;
use crate::dom::htmlelement::HTMLElement;
use crate::dom::htmlformelement::{FormControl, HTMLFormElement};
use crate::dom::htmllegendelement::HTMLLegendElement;
use crate::dom::node::{Node, NodeTraits, ShadowIncluding};
use crate::dom::validation::Validatable;
use crate::dom::validitystate::ValidityState;
use crate::dom::virtualmethods::VirtualMethods;
use crate::script_runtime::CanGc;
use crate::script_thread::ScriptThread;

#[dom_struct]
pub(crate) struct HTMLFieldSetElement {
    htmlelement: HTMLElement,
    form_owner: MutNullableDom<HTMLFormElement>,
    validity_state: MutNullableDom<ValidityState>,
}

impl HTMLFieldSetElement {
    fn new_inherited(
        local_name: LocalName,
        prefix: Option<Prefix>,
        document: &Document,
    ) -> HTMLFieldSetElement {
        HTMLFieldSetElement {
            htmlelement: HTMLElement::new_inherited_with_state(
                ElementState::ENABLED | ElementState::VALID,
                local_name,
                prefix,
                document,
            ),
            form_owner: Default::default(),
            validity_state: Default::default(),
        }
    }

    #[cfg_attr(crown, allow(crown::unrooted_must_root))]
    pub(crate) fn new(
        local_name: LocalName,
        prefix: Option<Prefix>,
        document: &Document,
        proto: Option<HandleObject>,
        can_gc: CanGc,
    ) -> DomRoot<HTMLFieldSetElement> {
        Node::reflect_node_with_proto(
            Box::new(HTMLFieldSetElement::new_inherited(
                local_name, prefix, document,
            )),
            document,
            proto,
            can_gc,
        )
    }

    pub(crate) fn update_validity(&self) {
        let has_invalid_child = self
            .upcast::<Node>()
            .traverse_preorder(ShadowIncluding::No)
            .flat_map(DomRoot::downcast::<Element>)
            .any(|element| element.is_invalid(false));

        self.upcast::<Element>()
            .set_state(ElementState::VALID, !has_invalid_child);
        self.upcast::<Element>()
            .set_state(ElementState::INVALID, has_invalid_child);
    }
}

impl HTMLFieldSetElementMethods<crate::DomTypeHolder> for HTMLFieldSetElement {
    // https://html.spec.whatwg.org/multipage/#dom-fieldset-elements
    fn Elements(&self) -> DomRoot<HTMLCollection> {
        HTMLCollection::new_with_filter_fn(&self.owner_window(), self.upcast(), |element, _| {
            element
                .downcast::<HTMLElement>()
                .is_some_and(HTMLElement::is_listed_element)
        })
    }

    // https://html.spec.whatwg.org/multipage/#dom-fieldset-disabled
    make_bool_getter!(Disabled, "disabled");

    // https://html.spec.whatwg.org/multipage/#dom-fieldset-disabled
    make_bool_setter!(SetDisabled, "disabled");

    // https://html.spec.whatwg.org/multipage/#dom-fe-name
    make_atomic_setter!(SetName, "name");

    // https://html.spec.whatwg.org/multipage/#dom-fe-name
    make_getter!(Name, "name");

    // https://html.spec.whatwg.org/multipage/#dom-fae-form
    fn GetForm(&self) -> Option<DomRoot<HTMLFormElement>> {
        self.form_owner()
    }

    // https://html.spec.whatwg.org/multipage/#dom-cva-willvalidate
    fn WillValidate(&self) -> bool {
        self.is_instance_validatable()
    }

    // https://html.spec.whatwg.org/multipage/#dom-cva-validity
    fn Validity(&self) -> DomRoot<ValidityState> {
        self.validity_state()
    }

    // https://html.spec.whatwg.org/multipage/#dom-cva-checkvalidity
    fn CheckValidity(&self, can_gc: CanGc) -> bool {
        self.check_validity(can_gc)
    }

    // https://html.spec.whatwg.org/multipage/#dom-cva-reportvalidity
    fn ReportValidity(&self, can_gc: CanGc) -> bool {
        self.report_validity(can_gc)
    }

    // https://html.spec.whatwg.org/multipage/#dom-cva-validationmessage
    fn ValidationMessage(&self) -> DOMString {
        self.validation_message()
    }

    // https://html.spec.whatwg.org/multipage/#dom-cva-setcustomvalidity
    fn SetCustomValidity(&self, error: DOMString) {
        self.validity_state().set_custom_error_message(error);
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-fieldset-type>
    fn Type(&self) -> DOMString {
        DOMString::from_string(String::from("fieldset"))
    }
}

impl VirtualMethods for HTMLFieldSetElement {
    fn super_type(&self) -> Option<&dyn VirtualMethods> {
        Some(self.upcast::<HTMLElement>() as &dyn VirtualMethods)
    }

    fn attribute_mutated(&self, attr: &Attr, mutation: AttributeMutation) {
        self.super_type().unwrap().attribute_mutated(attr, mutation);
        match *attr.local_name() {
            local_name!("disabled") => {
                let disabled_state = match mutation {
                    AttributeMutation::Set(None) => true,
                    AttributeMutation::Set(Some(_)) => {
                        // Fieldset was already disabled before.
                        return;
                    },
                    AttributeMutation::Removed => false,
                };
                let node = self.upcast::<Node>();
                let element = self.upcast::<Element>();
                element.set_disabled_state(disabled_state);
                element.set_enabled_state(!disabled_state);
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
                    child
                        .traverse_preorder(ShadowIncluding::No)
                        .filter(|descendant| match descendant.type_id() {
                            NodeTypeId::Element(ElementTypeId::HTMLElement(
                                HTMLElementTypeId::HTMLButtonElement |
                                HTMLElementTypeId::HTMLInputElement |
                                HTMLElementTypeId::HTMLSelectElement |
                                HTMLElementTypeId::HTMLTextAreaElement,
                            )) => true,
                            NodeTypeId::Element(ElementTypeId::HTMLElement(
                                HTMLElementTypeId::HTMLElement,
                            )) => descendant
                                .downcast::<HTMLElement>()
                                .unwrap()
                                .is_form_associated_custom_element(),
                            _ => false,
                        })
                });
                if disabled_state {
                    for field in fields {
                        let element = field.downcast::<Element>().unwrap();
                        if element.enabled_state() {
                            element.set_disabled_state(true);
                            element.set_enabled_state(false);
                            if element
                                .downcast::<HTMLElement>()
                                .is_some_and(|h| h.is_form_associated_custom_element())
                            {
                                ScriptThread::enqueue_callback_reaction(
                                    element,
                                    CallbackReaction::FormDisabled(true),
                                    None,
                                );
                            }
                        }
                        element.update_sequentially_focusable_status(CanGc::note());
                    }
                } else {
                    for field in fields {
                        let element = field.downcast::<Element>().unwrap();
                        if element.disabled_state() {
                            element.check_disabled_attribute();
                            element.check_ancestors_disabled_state_for_form_control();
                            // Fire callback only if this has actually enabled the custom element
                            if element.enabled_state() &&
                                element
                                    .downcast::<HTMLElement>()
                                    .is_some_and(|h| h.is_form_associated_custom_element())
                            {
                                ScriptThread::enqueue_callback_reaction(
                                    element,
                                    CallbackReaction::FormDisabled(false),
                                    None,
                                );
                            }
                        }
                        element.update_sequentially_focusable_status(CanGc::note());
                    }
                }
                element.update_sequentially_focusable_status(CanGc::note());
            },
            local_name!("form") => {
                self.form_attribute_mutated(mutation);
            },
            _ => {},
        }
    }
}

impl FormControl for HTMLFieldSetElement {
    fn form_owner(&self) -> Option<DomRoot<HTMLFormElement>> {
        self.form_owner.get()
    }

    fn set_form_owner(&self, form: Option<&HTMLFormElement>) {
        self.form_owner.set(form);
    }

    fn to_element(&self) -> &Element {
        self.upcast::<Element>()
    }
}

impl Validatable for HTMLFieldSetElement {
    fn as_element(&self) -> &Element {
        self.upcast()
    }

    fn validity_state(&self) -> DomRoot<ValidityState> {
        self.validity_state
            .or_init(|| ValidityState::new(&self.owner_window(), self.upcast()))
    }

    fn is_instance_validatable(&self) -> bool {
        // fieldset is not a submittable element (https://html.spec.whatwg.org/multipage/#category-submit)
        false
    }
}

/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use html5ever::{local_name, LocalName, Prefix};
use js::rust::HandleObject;

use crate::dom::attr::Attr;
use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::HTMLOutputElementBinding::HTMLOutputElementMethods;
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::root::{DomRoot, MutNullableDom};
use crate::dom::bindings::str::DOMString;
use crate::dom::document::Document;
use crate::dom::element::{AttributeMutation, Element};
use crate::dom::htmlelement::HTMLElement;
use crate::dom::htmlformelement::{FormControl, HTMLFormElement};
use crate::dom::node::{Node, NodeTraits};
use crate::dom::nodelist::NodeList;
use crate::dom::validation::Validatable;
use crate::dom::validitystate::ValidityState;
use crate::dom::virtualmethods::VirtualMethods;
use crate::script_runtime::CanGc;

#[dom_struct]
pub(crate) struct HTMLOutputElement {
    htmlelement: HTMLElement,
    form_owner: MutNullableDom<HTMLFormElement>,
    labels_node_list: MutNullableDom<NodeList>,
    default_value_override: DomRefCell<Option<DOMString>>,
    validity_state: MutNullableDom<ValidityState>,
}

impl HTMLOutputElement {
    fn new_inherited(
        local_name: LocalName,
        prefix: Option<Prefix>,
        document: &Document,
    ) -> HTMLOutputElement {
        HTMLOutputElement {
            htmlelement: HTMLElement::new_inherited(local_name, prefix, document),
            form_owner: Default::default(),
            labels_node_list: Default::default(),
            default_value_override: DomRefCell::new(None),
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
    ) -> DomRoot<HTMLOutputElement> {
        Node::reflect_node_with_proto(
            Box::new(HTMLOutputElement::new_inherited(
                local_name, prefix, document,
            )),
            document,
            proto,
            can_gc,
        )
    }

    pub(crate) fn reset(&self, can_gc: CanGc) {
        Node::string_replace_all(self.DefaultValue(), self.upcast::<Node>(), can_gc);
        *self.default_value_override.borrow_mut() = None;
    }
}

impl HTMLOutputElementMethods<crate::DomTypeHolder> for HTMLOutputElement {
    // https://html.spec.whatwg.org/multipage/#dom-fae-form
    fn GetForm(&self) -> Option<DomRoot<HTMLFormElement>> {
        self.form_owner()
    }

    // https://html.spec.whatwg.org/multipage/#dom-lfe-labels
    make_labels_getter!(Labels, labels_node_list);

    // https://html.spec.whatwg.org/multipage/#dom-output-defaultvaleu
    fn DefaultValue(&self) -> DOMString {
        let dvo = self.default_value_override.borrow();
        if let Some(ref dv) = *dvo {
            dv.clone()
        } else {
            self.upcast::<Node>().descendant_text_content()
        }
    }

    // https://html.spec.whatwg.org/multipage/#dom-output-defaultvalue
    fn SetDefaultValue(&self, value: DOMString, can_gc: CanGc) {
        if self.default_value_override.borrow().is_none() {
            // Step 1 ("and return")
            Node::string_replace_all(value.clone(), self.upcast::<Node>(), can_gc);
        } else {
            // Step 2, if not returned from step 1
            *self.default_value_override.borrow_mut() = Some(value);
        }
    }

    // https://html.spec.whatwg.org/multipage/#dom-output-value
    fn Value(&self) -> DOMString {
        self.upcast::<Node>().descendant_text_content()
    }

    // https://html.spec.whatwg.org/multipage/#dom-output-value
    fn SetValue(&self, value: DOMString, can_gc: CanGc) {
        *self.default_value_override.borrow_mut() = Some(self.DefaultValue());
        Node::string_replace_all(value, self.upcast::<Node>(), can_gc);
    }

    // https://html.spec.whatwg.org/multipage/#dom-output-type
    fn Type(&self) -> DOMString {
        DOMString::from("output")
    }

    // https://html.spec.whatwg.org/multipage/#dom-fe-name
    make_atomic_setter!(SetName, "name");

    // https://html.spec.whatwg.org/multipage/#dom-fe-name
    make_getter!(Name, "name");

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
}

impl VirtualMethods for HTMLOutputElement {
    fn super_type(&self) -> Option<&dyn VirtualMethods> {
        Some(self.upcast::<HTMLElement>() as &dyn VirtualMethods)
    }

    fn attribute_mutated(&self, attr: &Attr, mutation: AttributeMutation) {
        self.super_type().unwrap().attribute_mutated(attr, mutation);
        if attr.local_name() == &local_name!("form") {
            self.form_attribute_mutated(mutation);
        }
    }
}

impl FormControl for HTMLOutputElement {
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

impl Validatable for HTMLOutputElement {
    fn as_element(&self) -> &Element {
        self.upcast()
    }

    fn validity_state(&self) -> DomRoot<ValidityState> {
        self.validity_state
            .or_init(|| ValidityState::new(&self.owner_window(), self.upcast(), CanGc::note()))
    }

    fn is_instance_validatable(&self) -> bool {
        // output is not a submittable element (https://html.spec.whatwg.org/multipage/#category-submit)
        false
    }
}

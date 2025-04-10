/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::Cell;
use std::default::Default;

use dom_struct::dom_struct;
use html5ever::{LocalName, Prefix, local_name, namespace_url};
use js::rust::HandleObject;
use stylo_dom::ElementState;

use crate::dom::activation::Activatable;
use crate::dom::attr::Attr;
use crate::dom::bindings::codegen::Bindings::HTMLButtonElementBinding::HTMLButtonElementMethods;
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::root::{DomRoot, MutNullableDom};
use crate::dom::bindings::str::DOMString;
use crate::dom::document::Document;
use crate::dom::element::{AttributeMutation, Element};
use crate::dom::event::Event;
use crate::dom::eventtarget::EventTarget;
use crate::dom::htmlelement::HTMLElement;
use crate::dom::htmlfieldsetelement::HTMLFieldSetElement;
use crate::dom::htmlformelement::{
    FormControl, FormDatum, FormDatumValue, FormSubmitterElement, HTMLFormElement, ResetFrom,
    SubmittedFrom,
};
use crate::dom::node::{BindContext, Node, NodeTraits, UnbindContext};
use crate::dom::nodelist::NodeList;
use crate::dom::validation::{Validatable, is_barred_by_datalist_ancestor};
use crate::dom::validitystate::{ValidationFlags, ValidityState};
use crate::dom::virtualmethods::VirtualMethods;
use crate::script_runtime::CanGc;

#[derive(Clone, Copy, JSTraceable, MallocSizeOf, PartialEq)]
enum ButtonType {
    Submit,
    Reset,
    Button,
}

#[dom_struct]
pub(crate) struct HTMLButtonElement {
    htmlelement: HTMLElement,
    button_type: Cell<ButtonType>,
    form_owner: MutNullableDom<HTMLFormElement>,
    labels_node_list: MutNullableDom<NodeList>,
    validity_state: MutNullableDom<ValidityState>,
}

impl HTMLButtonElement {
    fn new_inherited(
        local_name: LocalName,
        prefix: Option<Prefix>,
        document: &Document,
    ) -> HTMLButtonElement {
        HTMLButtonElement {
            htmlelement: HTMLElement::new_inherited_with_state(
                ElementState::ENABLED,
                local_name,
                prefix,
                document,
            ),
            button_type: Cell::new(ButtonType::Submit),
            form_owner: Default::default(),
            labels_node_list: Default::default(),
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
    ) -> DomRoot<HTMLButtonElement> {
        Node::reflect_node_with_proto(
            Box::new(HTMLButtonElement::new_inherited(
                local_name, prefix, document,
            )),
            document,
            proto,
            can_gc,
        )
    }

    #[inline]
    pub(crate) fn is_submit_button(&self) -> bool {
        self.button_type.get() == ButtonType::Submit
    }
}

impl HTMLButtonElementMethods<crate::DomTypeHolder> for HTMLButtonElement {
    // https://html.spec.whatwg.org/multipage/#dom-fe-disabled
    make_bool_getter!(Disabled, "disabled");

    // https://html.spec.whatwg.org/multipage/#dom-fe-disabled
    make_bool_setter!(SetDisabled, "disabled");

    // https://html.spec.whatwg.org/multipage/#dom-fae-form
    fn GetForm(&self) -> Option<DomRoot<HTMLFormElement>> {
        self.form_owner()
    }

    // <https://html.spec.whatwg.org/multipage/#dom-button-type>
    make_enumerated_getter!(
        Type,
        "type",
        "submit" | "reset" | "button",
        missing => "submit",
        invalid => "submit"
    );

    // https://html.spec.whatwg.org/multipage/#dom-button-type
    make_setter!(SetType, "type");

    // https://html.spec.whatwg.org/multipage/#dom-fs-formaction
    make_form_action_getter!(FormAction, "formaction");

    // https://html.spec.whatwg.org/multipage/#dom-fs-formaction
    make_setter!(SetFormAction, "formaction");

    // https://html.spec.whatwg.org/multipage/#dom-fs-formenctype
    make_enumerated_getter!(
        FormEnctype,
        "formenctype",
        "application/x-www-form-urlencoded" | "multipart/form-data" | "text/plain",
        invalid => "application/x-www-form-urlencoded"
    );

    // https://html.spec.whatwg.org/multipage/#dom-fs-formenctype
    make_setter!(SetFormEnctype, "formenctype");

    // https://html.spec.whatwg.org/multipage/#dom-fs-formmethod
    make_enumerated_getter!(
        FormMethod,
        "formmethod",
        "get" | "post" | "dialog",
        missing => "get",
        invalid => "get"
    );

    // https://html.spec.whatwg.org/multipage/#dom-fs-formmethod
    make_setter!(SetFormMethod, "formmethod");

    // https://html.spec.whatwg.org/multipage/#dom-fs-formtarget
    make_getter!(FormTarget, "formtarget");

    // https://html.spec.whatwg.org/multipage/#dom-fs-formtarget
    make_setter!(SetFormTarget, "formtarget");

    // https://html.spec.whatwg.org/multipage/#attr-fs-formnovalidate
    make_bool_getter!(FormNoValidate, "formnovalidate");

    // https://html.spec.whatwg.org/multipage/#attr-fs-formnovalidate
    make_bool_setter!(SetFormNoValidate, "formnovalidate");

    // https://html.spec.whatwg.org/multipage/#dom-fe-name
    make_getter!(Name, "name");

    // https://html.spec.whatwg.org/multipage/#dom-fe-name
    make_atomic_setter!(SetName, "name");

    // https://html.spec.whatwg.org/multipage/#dom-button-value
    make_getter!(Value, "value");

    // https://html.spec.whatwg.org/multipage/#dom-button-value
    make_setter!(SetValue, "value");

    // https://html.spec.whatwg.org/multipage/#dom-lfe-labels
    make_labels_getter!(Labels, labels_node_list);

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

impl HTMLButtonElement {
    /// <https://html.spec.whatwg.org/multipage/#constructing-the-form-data-set>
    /// Steps range from 3.1 to 3.7 (specific to HTMLButtonElement)
    pub(crate) fn form_datum(&self, submitter: Option<FormSubmitterElement>) -> Option<FormDatum> {
        // Step 3.1: disabled state check is in get_unclean_dataset

        // Step 3.1: only run steps if this is the submitter
        if let Some(FormSubmitterElement::Button(submitter)) = submitter {
            if submitter != self {
                return None;
            }
        } else {
            return None;
        }
        // Step 3.2
        let ty = self.Type();
        // Step 3.4
        let name = self.Name();

        if name.is_empty() {
            // Step 3.1: Must have a name
            return None;
        }

        // Step 3.9
        Some(FormDatum {
            ty,
            name,
            value: FormDatumValue::String(self.Value()),
        })
    }
}

impl VirtualMethods for HTMLButtonElement {
    fn super_type(&self) -> Option<&dyn VirtualMethods> {
        Some(self.upcast::<HTMLElement>() as &dyn VirtualMethods)
    }

    fn attribute_mutated(&self, attr: &Attr, mutation: AttributeMutation, can_gc: CanGc) {
        self.super_type()
            .unwrap()
            .attribute_mutated(attr, mutation, can_gc);
        match *attr.local_name() {
            local_name!("disabled") => {
                let el = self.upcast::<Element>();
                match mutation {
                    AttributeMutation::Set(Some(_)) => {},
                    AttributeMutation::Set(None) => {
                        el.set_disabled_state(true);
                        el.set_enabled_state(false);
                    },
                    AttributeMutation::Removed => {
                        el.set_disabled_state(false);
                        el.set_enabled_state(true);
                        el.check_ancestors_disabled_state_for_form_control();
                    },
                }
                el.update_sequentially_focusable_status(CanGc::note());
                self.validity_state()
                    .perform_validation_and_update(ValidationFlags::all(), can_gc);
            },
            local_name!("type") => match mutation {
                AttributeMutation::Set(_) => {
                    let value = match &**attr.value() {
                        "reset" => ButtonType::Reset,
                        "button" => ButtonType::Button,
                        _ => ButtonType::Submit,
                    };
                    self.button_type.set(value);
                    self.validity_state()
                        .perform_validation_and_update(ValidationFlags::all(), can_gc);
                },
                AttributeMutation::Removed => {
                    self.button_type.set(ButtonType::Submit);
                },
            },
            local_name!("form") => {
                self.form_attribute_mutated(mutation, can_gc);
                self.validity_state()
                    .perform_validation_and_update(ValidationFlags::empty(), can_gc);
            },
            _ => {},
        }
    }

    fn bind_to_tree(&self, context: &BindContext, can_gc: CanGc) {
        if let Some(s) = self.super_type() {
            s.bind_to_tree(context, can_gc);
        }

        self.upcast::<Element>()
            .check_ancestors_disabled_state_for_form_control();
    }

    fn unbind_from_tree(&self, context: &UnbindContext, can_gc: CanGc) {
        self.super_type().unwrap().unbind_from_tree(context, can_gc);

        let node = self.upcast::<Node>();
        let el = self.upcast::<Element>();
        if node
            .ancestors()
            .any(|ancestor| ancestor.is::<HTMLFieldSetElement>())
        {
            el.check_ancestors_disabled_state_for_form_control();
        } else {
            el.check_disabled_attribute();
        }
    }
}

impl FormControl for HTMLButtonElement {
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

impl Validatable for HTMLButtonElement {
    fn as_element(&self) -> &Element {
        self.upcast()
    }

    fn validity_state(&self) -> DomRoot<ValidityState> {
        self.validity_state
            .or_init(|| ValidityState::new(&self.owner_window(), self.upcast(), CanGc::note()))
    }

    fn is_instance_validatable(&self) -> bool {
        // https://html.spec.whatwg.org/multipage/#the-button-element%3Abarred-from-constraint-validation
        // https://html.spec.whatwg.org/multipage/#enabling-and-disabling-form-controls%3A-the-disabled-attribute%3Abarred-from-constraint-validation
        // https://html.spec.whatwg.org/multipage/#the-datalist-element%3Abarred-from-constraint-validation
        self.button_type.get() == ButtonType::Submit &&
            !self.upcast::<Element>().disabled_state() &&
            !is_barred_by_datalist_ancestor(self.upcast())
    }
}

impl Activatable for HTMLButtonElement {
    fn as_element(&self) -> &Element {
        self.upcast()
    }

    fn is_instance_activatable(&self) -> bool {
        //https://html.spec.whatwg.org/multipage/#the-button-element
        !self.upcast::<Element>().disabled_state()
    }

    // https://html.spec.whatwg.org/multipage/#run-post-click-activation-steps
    fn activation_behavior(&self, _event: &Event, _target: &EventTarget, can_gc: CanGc) {
        let ty = self.button_type.get();
        match ty {
            //https://html.spec.whatwg.org/multipage/#attr-button-type-submit-state
            ButtonType::Submit => {
                // TODO: is document owner fully active?
                if let Some(owner) = self.form_owner() {
                    owner.submit(
                        SubmittedFrom::NotFromForm,
                        FormSubmitterElement::Button(self),
                        can_gc,
                    );
                }
            },
            ButtonType::Reset => {
                // TODO: is document owner fully active?
                if let Some(owner) = self.form_owner() {
                    owner.reset(ResetFrom::NotFromForm, can_gc);
                }
            },
            _ => (),
        }
    }
}

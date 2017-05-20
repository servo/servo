/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::activation::{Activatable, ActivationSource, synthetic_click_activation};
use dom::attr::Attr;
use dom::bindings::codegen::Bindings::HTMLButtonElementBinding;
use dom::bindings::codegen::Bindings::HTMLButtonElementBinding::HTMLButtonElementMethods;
use dom::bindings::inheritance::Castable;
use dom::bindings::js::{MutNullableJS, Root};
use dom::bindings::str::DOMString;
use dom::document::Document;
use dom::element::{AttributeMutation, Element};
use dom::event::Event;
use dom::eventtarget::EventTarget;
use dom::htmlelement::HTMLElement;
use dom::htmlfieldsetelement::HTMLFieldSetElement;
use dom::htmlformelement::{FormControl, FormDatum, FormDatumValue};
use dom::htmlformelement::{FormSubmitter, ResetFrom, SubmittedFrom};
use dom::htmlformelement::HTMLFormElement;
use dom::node::{Node, UnbindContext, document_from_node, window_from_node};
use dom::nodelist::NodeList;
use dom::validation::Validatable;
use dom::validitystate::{ValidityState, ValidationFlags};
use dom::virtualmethods::VirtualMethods;
use dom_struct::dom_struct;
use html5ever::{LocalName, Prefix};
use std::cell::Cell;
use std::default::Default;
use style::element_state::*;

#[derive(JSTraceable, PartialEq, Copy, Clone)]
#[derive(HeapSizeOf)]
enum ButtonType {
    Submit,
    Reset,
    Button,
    Menu
}

#[dom_struct]
pub struct HTMLButtonElement {
    htmlelement: HTMLElement,
    button_type: Cell<ButtonType>,
    form_owner: MutNullableJS<HTMLFormElement>,
}

impl HTMLButtonElement {
    fn new_inherited(local_name: LocalName,
                     prefix: Option<Prefix>,
                     document: &Document) -> HTMLButtonElement {
        HTMLButtonElement {
            htmlelement:
                HTMLElement::new_inherited_with_state(IN_ENABLED_STATE,
                                                      local_name, prefix, document),
            button_type: Cell::new(ButtonType::Submit),
            form_owner: Default::default(),
        }
    }

    #[allow(unrooted_must_root)]
    pub fn new(local_name: LocalName,
               prefix: Option<Prefix>,
               document: &Document) -> Root<HTMLButtonElement> {
        Node::reflect_node(box HTMLButtonElement::new_inherited(local_name, prefix, document),
                           document,
                           HTMLButtonElementBinding::Wrap)
    }
}

impl HTMLButtonElementMethods for HTMLButtonElement {
    // https://html.spec.whatwg.org/multipage/#dom-cva-validity
    fn Validity(&self) -> Root<ValidityState> {
        let window = window_from_node(self);
        ValidityState::new(&window, self.upcast())
    }

    // https://html.spec.whatwg.org/multipage/#dom-fe-disabled
    make_bool_getter!(Disabled, "disabled");

    // https://html.spec.whatwg.org/multipage/#dom-fe-disabled
    make_bool_setter!(SetDisabled, "disabled");

    // https://html.spec.whatwg.org/multipage/#dom-fae-form
    fn GetForm(&self) -> Option<Root<HTMLFormElement>> {
        self.form_owner()
    }

    // https://html.spec.whatwg.org/multipage/#dom-button-type
    make_enumerated_getter!(Type, "type", "submit", "reset" | "button" | "menu");

    // https://html.spec.whatwg.org/multipage/#dom-button-type
    make_setter!(SetType, "type");

    // https://html.spec.whatwg.org/multipage/#dom-fs-formaction
    make_url_or_base_getter!(FormAction, "formaction");

    // https://html.spec.whatwg.org/multipage/#dom-fs-formaction
    make_setter!(SetFormAction, "formaction");

    // https://html.spec.whatwg.org/multipage/#dom-fs-formenctype
    make_enumerated_getter!(FormEnctype,
                            "formenctype",
                            "application/x-www-form-urlencoded",
                            "text/plain" | "multipart/form-data");

    // https://html.spec.whatwg.org/multipage/#dom-fs-formenctype
    make_setter!(SetFormEnctype, "formenctype");

    // https://html.spec.whatwg.org/multipage/#dom-fs-formmethod
    make_enumerated_getter!(FormMethod, "formmethod", "get", "post" | "dialog");

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
    make_setter!(SetName, "name");

    // https://html.spec.whatwg.org/multipage/#dom-button-value
    make_getter!(Value, "value");

    // https://html.spec.whatwg.org/multipage/#dom-button-value
    make_setter!(SetValue, "value");

    // https://html.spec.whatwg.org/multipage/#dom-lfe-labels
    fn Labels(&self) -> Root<NodeList> {
        self.upcast::<HTMLElement>().labels()
    }
}

impl HTMLButtonElement {
    /// https://html.spec.whatwg.org/multipage/#constructing-the-form-data-set
    /// Steps range from 3.1 to 3.7 (specific to HTMLButtonElement)
    pub fn form_datum(&self, submitter: Option<FormSubmitter>) -> Option<FormDatum> {
        // Step 3.1: disabled state check is in get_unclean_dataset

        // Step 3.1: only run steps if this is the submitter
        if let Some(FormSubmitter::ButtonElement(submitter)) = submitter {
            if submitter != self {
                return None
            }
        } else {
            return None
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
            ty: ty,
            name: name,
            value: FormDatumValue::String(self.Value())
        })
    }
}

impl VirtualMethods for HTMLButtonElement {
    fn super_type(&self) -> Option<&VirtualMethods> {
        Some(self.upcast::<HTMLElement>() as &VirtualMethods)
    }

    fn attribute_mutated(&self, attr: &Attr, mutation: AttributeMutation) {
        self.super_type().unwrap().attribute_mutated(attr, mutation);
        match attr.local_name() {
            &local_name!("disabled") => {
                let el = self.upcast::<Element>();
                match mutation {
                    AttributeMutation::Set(Some(_)) => {}
                    AttributeMutation::Set(None) => {
                        el.set_disabled_state(true);
                        el.set_enabled_state(false);
                    },
                    AttributeMutation::Removed => {
                        el.set_disabled_state(false);
                        el.set_enabled_state(true);
                        el.check_ancestors_disabled_state_for_form_control();
                    }
                }
            },
            &local_name!("type") => {
                match mutation {
                    AttributeMutation::Set(_) => {
                        let value = match &**attr.value() {
                            "reset" => ButtonType::Reset,
                            "button" => ButtonType::Button,
                            "menu" => ButtonType::Menu,
                            _ => ButtonType::Submit,
                        };
                        self.button_type.set(value);
                    }
                    AttributeMutation::Removed => {
                        self.button_type.set(ButtonType::Submit);
                    }
                }
            },
            &local_name!("form") => {
                self.form_attribute_mutated(mutation);
            }
            _ => {},
        }
    }

    fn bind_to_tree(&self, tree_in_doc: bool) {
        if let Some(ref s) = self.super_type() {
            s.bind_to_tree(tree_in_doc);
        }

        self.upcast::<Element>().check_ancestors_disabled_state_for_form_control();
    }

    fn unbind_from_tree(&self, context: &UnbindContext) {
        self.super_type().unwrap().unbind_from_tree(context);

        let node = self.upcast::<Node>();
        let el = self.upcast::<Element>();
        if node.ancestors().any(|ancestor| ancestor.is::<HTMLFieldSetElement>()) {
            el.check_ancestors_disabled_state_for_form_control();
        } else {
            el.check_disabled_attribute();
        }
    }
}

impl FormControl for HTMLButtonElement {
    fn form_owner(&self) -> Option<Root<HTMLFormElement>> {
        self.form_owner.get()
    }

    fn set_form_owner(&self, form: Option<&HTMLFormElement>) {
        self.form_owner.set(form);
    }

    fn to_element<'a>(&'a self) -> &'a Element {
        self.upcast::<Element>()
    }
}

impl Validatable for HTMLButtonElement {
    fn is_instance_validatable(&self) -> bool {
        true
    }
    fn validate(&self, validate_flags: ValidationFlags) -> bool {
        if validate_flags.is_empty() {}
        // Need more flag check for different validation types later
        true
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

    // https://html.spec.whatwg.org/multipage/#run-pre-click-activation-steps
    // https://html.spec.whatwg.org/multipage/#the-button-element:activation-behavior
    fn pre_click_activation(&self) {
    }

    // https://html.spec.whatwg.org/multipage/#run-canceled-activation-steps
    fn canceled_activation(&self) {
    }

    // https://html.spec.whatwg.org/multipage/#run-post-click-activation-steps
    fn activation_behavior(&self, _event: &Event, _target: &EventTarget) {
        let ty = self.button_type.get();
        match ty {
            //https://html.spec.whatwg.org/multipage/#attr-button-type-submit-state
            ButtonType::Submit => {
                // TODO: is document owner fully active?
                if let Some(owner) = self.form_owner() {
                    owner.submit(SubmittedFrom::NotFromForm,
                                 FormSubmitter::ButtonElement(self.clone()));
                }
            }
            ButtonType::Reset => {
                // TODO: is document owner fully active?
                if let Some(owner) = self.form_owner() {
                    owner.reset(ResetFrom::NotFromForm);
                }
            }
            _ => (),
        }
    }

    // https://html.spec.whatwg.org/multipage/#implicit-submission
    #[allow(unsafe_code)]
    fn implicit_submission(&self, ctrl_key: bool, shift_key: bool, alt_key: bool, meta_key: bool) {
        let doc = document_from_node(self);
        let node = doc.upcast::<Node>();
        let owner = self.form_owner();
        if owner.is_none() || self.upcast::<Element>().click_in_progress() {
            return;
        }
        node.query_selector_iter(DOMString::from("button[type=submit]")).unwrap()
            .filter_map(Root::downcast::<HTMLButtonElement>)
            .find(|r| r.form_owner() == owner)
            .map(|s| synthetic_click_activation(s.as_element(),
                                                ctrl_key,
                                                shift_key,
                                                alt_key,
                                                meta_key,
                                                ActivationSource::NotFromClick));
    }
}

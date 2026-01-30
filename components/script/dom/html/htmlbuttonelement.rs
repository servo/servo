/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::Cell;
use std::default::Default;

use dom_struct::dom_struct;
use html5ever::{LocalName, Prefix, local_name, ns};
use js::rust::HandleObject;
use script_bindings::codegen::GenericBindings::AttrBinding::AttrMethods;
use script_bindings::codegen::GenericBindings::DocumentBinding::DocumentMethods;
use script_bindings::codegen::GenericBindings::DocumentFragmentBinding::DocumentFragmentMethods;
use script_bindings::codegen::GenericBindings::NodeBinding::NodeMethods;
use servo_config::pref;
use stylo_dom::ElementState;

use crate::dom::activation::Activatable;
use crate::dom::attr::Attr;
use crate::dom::bindings::codegen::Bindings::HTMLButtonElementBinding::HTMLButtonElementMethods;
use crate::dom::bindings::codegen::Bindings::NodeBinding::GetRootNodeOptions;
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::root::{DomRoot, MutNullableDom};
use crate::dom::bindings::str::DOMString;
use crate::dom::commandevent::CommandEvent;
use crate::dom::document::Document;
use crate::dom::documentfragment::DocumentFragment;
use crate::dom::element::{AttributeMutation, Element};
use crate::dom::event::{Event, EventBubbles, EventCancelable};
use crate::dom::eventtarget::EventTarget;
use crate::dom::html::htmlelement::HTMLElement;
use crate::dom::html::htmlfieldsetelement::HTMLFieldSetElement;
use crate::dom::html::htmlformelement::{
    FormControl, FormDatum, FormDatumValue, FormSubmitterElement, HTMLFormElement, ResetFrom,
    SubmittedFrom,
};
use crate::dom::node::{BindContext, Node, NodeTraits, UnbindContext};
use crate::dom::nodelist::NodeList;
use crate::dom::validation::{Validatable, is_barred_by_datalist_ancestor};
use crate::dom::validitystate::{ValidationFlags, ValidityState};
use crate::dom::virtualmethods::{VirtualMethods, vtable_for};
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
    /// <https://html.spec.whatwg.org/multipage/#dom-button-command>
    fn Command(&self) -> DOMString {
        // Step 1. Let command be this's command attribute.
        match self.command_state() {
            // Step 2. If command is in the Custom state, then return command's value.
            CommandState::Custom => self
                .upcast::<Element>()
                .get_string_attribute(&local_name!("command")),
            // Step 3. If command is in the Unknown state, then return the empty string.
            CommandState::Unknown => DOMString::default(),
            // Step 4. Return the keyword corresponding to the value of command.
            CommandState::Close => DOMString::from("close"),
            CommandState::ShowModal => DOMString::from("show-modal"),
        }
    }

    // https://html.spec.whatwg.org/multipage/#dom-button-command
    make_setter!(SetCommand, "command");

    // https://html.spec.whatwg.org/multipage/#dom-fe-disabled
    make_bool_getter!(Disabled, "disabled");

    // https://html.spec.whatwg.org/multipage/#dom-fe-disabled
    make_bool_setter!(SetDisabled, "disabled");

    /// <https://html.spec.whatwg.org/multipage/#dom-fae-form>
    fn GetForm(&self) -> Option<DomRoot<HTMLFormElement>> {
        self.form_owner()
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-button-type>
    fn Type(&self) -> DOMString {
        match self.button_type.get() {
            ButtonType::Submit => DOMString::from("submit"),
            ButtonType::Button => DOMString::from("button"),
            ButtonType::Reset => DOMString::from("reset"),
        }
    }

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

    /// <https://html.spec.whatwg.org/multipage/#dom-cva-willvalidate>
    fn WillValidate(&self) -> bool {
        self.is_instance_validatable()
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-cva-validity>
    fn Validity(&self, can_gc: CanGc) -> DomRoot<ValidityState> {
        self.validity_state(can_gc)
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-cva-checkvalidity>
    fn CheckValidity(&self, can_gc: CanGc) -> bool {
        self.check_validity(can_gc)
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-cva-reportvalidity>
    fn ReportValidity(&self, can_gc: CanGc) -> bool {
        self.report_validity(can_gc)
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-cva-validationmessage>
    fn ValidationMessage(&self) -> DOMString {
        self.validation_message()
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-cva-setcustomvalidity>
    fn SetCustomValidity(&self, error: DOMString, can_gc: CanGc) {
        self.validity_state(can_gc).set_custom_error_message(error);
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

    fn set_type(&self, value: DOMString, can_gc: CanGc) {
        let value = match value.to_ascii_lowercase().as_str() {
            "reset" => ButtonType::Reset,
            "button" => ButtonType::Button,
            "submit" => ButtonType::Submit,
            _ => {
                if pref!(dom_command_invokers_enabled) {
                    let element = self.upcast::<Element>();
                    if element.has_attribute(&local_name!("command")) ||
                        element.has_attribute(&local_name!("commandfor"))
                    {
                        ButtonType::Button
                    } else {
                        ButtonType::Submit
                    }
                } else {
                    ButtonType::Submit
                }
            },
        };
        self.button_type.set(value);
        self.validity_state(can_gc)
            .perform_validation_and_update(ValidationFlags::all(), can_gc);
    }

    fn command_for_element(&self) -> Option<DomRoot<Element>> {
        let command_for_value = self
            .upcast::<Element>()
            .get_attribute(&ns!(), &local_name!("commandfor"))?
            .Value();

        let root_node = self
            .upcast::<Node>()
            .GetRootNode(&GetRootNodeOptions::empty());

        if let Some(document) = root_node.downcast::<Document>() {
            return document.GetElementById(command_for_value);
        } else if let Some(document_fragment) = root_node.downcast::<DocumentFragment>() {
            return document_fragment.GetElementById(command_for_value);
        }
        unreachable!("Button element must be in a document or document fragment");
    }

    fn command_state(&self) -> CommandState {
        let command = self
            .upcast::<Element>()
            .get_string_attribute(&local_name!("command"));
        if command.starts_with_str("--") {
            return CommandState::Custom;
        }
        let value = command.to_ascii_lowercase();
        if value == "close" {
            return CommandState::Close;
        }
        if value == "show-modal" {
            return CommandState::ShowModal;
        }

        CommandState::Unknown
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
                    AttributeMutation::Set(Some(_), _) => {},
                    AttributeMutation::Set(None, _) => {
                        el.set_disabled_state(true);
                        el.set_enabled_state(false);
                    },
                    AttributeMutation::Removed => {
                        el.set_disabled_state(false);
                        el.set_enabled_state(true);
                        el.check_ancestors_disabled_state_for_form_control();
                    },
                }
                el.update_sequentially_focusable_status(can_gc);
                self.validity_state(can_gc)
                    .perform_validation_and_update(ValidationFlags::all(), can_gc);
            },
            local_name!("type") => self.set_type(attr.Value(), can_gc),
            local_name!("command") => self.set_type(
                self.upcast::<Element>()
                    .get_string_attribute(&local_name!("type")),
                can_gc,
            ),
            local_name!("commandfor") => self.set_type(
                self.upcast::<Element>()
                    .get_string_attribute(&local_name!("type")),
                can_gc,
            ),
            local_name!("form") => {
                self.form_attribute_mutated(mutation, can_gc);
                self.validity_state(can_gc)
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

    fn validity_state(&self, can_gc: CanGc) -> DomRoot<ValidityState> {
        self.validity_state
            .or_init(|| ValidityState::new(&self.owner_window(), self.upcast(), can_gc))
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
        // https://html.spec.whatwg.org/multipage/#the-button-element
        !self.upcast::<Element>().disabled_state()
    }

    /// <https://html.spec.whatwg.org/multipage/#the-button-element:activation-behaviour>
    fn activation_behavior(&self, _event: &Event, target: &EventTarget, can_gc: CanGc) {
        // Step 2. If element's node document is not fully active, then return.
        if !target
            .downcast::<Node>()
            .is_none_or(|node| node.owner_document().is_fully_active())
        {
            return;
        }

        let ty = self.button_type.get();
        // Step 3. If element has a form owner:
        if let Some(owner) = self.form_owner() {
            // Step 3.1 If element is a submit button, then submit element's form owner from element
            // ..., and return.
            if ty == ButtonType::Submit {
                owner.submit(
                    SubmittedFrom::NotFromForm,
                    FormSubmitterElement::Button(self),
                    can_gc,
                );
                return;
            }
            // Step 3.2 If element's type attribute is in the Reset Button state, then reset
            // element's form owner and return.
            if ty == ButtonType::Reset {
                owner.reset(ResetFrom::NotFromForm, can_gc);
                return;
            }
            // Step 3.3 If element's type attribute is in the Auto state, then return.
            if ty == ButtonType::Button &&
                self.upcast::<Element>()
                    .get_string_attribute(&local_name!("type"))
                    .to_ascii_lowercase() ==
                    "button"
            {
                return;
            }
        }
        // Step 4. Let target be the result of running element's get the commandfor-associated
        // element.
        // Step 5. If target is not null:
        if let Some(target) = self.command_for_element() {
            // Steps 5.1 Let command be element's command attribute.
            let command = self.command_state();
            // Step 5.2 If command is in the unknown state, then return.
            if command == CommandState::Unknown {
                return;
            }
            // TODO Step 5.3 Let isPopover be true if target's popover attribute is not in the No
            // Popover state; otherwise false
            // Step 5.4 If isPopover is false and command is not in the Custom state:
            if command != CommandState::Custom {
                // TODO Step 5.4.1 Assert: target's namespace is the HTML namespace
                // Step 5.4.2 If this standard does not define is valid command steps given command
                // is false, then return.
                // Step 5.4.3 Otherwise, if the result of running target's corresponding is valid
                // command steps given command is false, then return.
                if !vtable_for(target.upcast::<Node>()).is_valid_command_steps(command) {
                    return;
                }
            }
            // Step 5.5 Let continue be the result of firing an event named command at target, using
            // CommandEvent, with its command attribute initialized to command, its source attribute
            // initialized to element, and its cancelable attribute initialized to true.
            let event = CommandEvent::new(
                &self.owner_window(),
                atom!("command"),
                EventBubbles::DoesNotBubble,
                EventCancelable::Cancelable,
                None,
                self.upcast::<Element>()
                    .get_string_attribute(&local_name!("command")),
                can_gc,
            );
            let event = event.upcast::<Event>();
            // Step 5.6 If continue is false, then return.
            if !event.fire(self.upcast::<EventTarget>(), can_gc) {
                return;
            }
            // Step 5.7 If target is not connected, then return.
            let target_node = target.upcast::<Node>();
            if !target_node.is_connected() {
                return;
            }
            // Step 5.8 If command is in the Custom state, then return.
            if command == CommandState::Custom {
                return;
            }
            // TODO Steps 5.9, 5.10, 511
            // Step 5.12 Otherwise, if this standard defines command steps for target's local name,
            // then run the corresponding command steps given target, element, and command.
            let _ = vtable_for(target_node).command_steps(
                DomRoot::from_ref(self),
                self.command_state(),
                can_gc,
            );
        }
        // TODO Step 6 Otherwise, run the popover target attribute activation behavior given element
        // and event's target.
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub(crate) enum CommandState {
    Unknown,
    Custom,
    ShowModal,
    Close,
}

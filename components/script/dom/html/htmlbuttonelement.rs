/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::Cell;
use std::default::Default;

use dom_struct::dom_struct;
use html5ever::{LocalName, Prefix, local_name};
use js::context::JSContext;
use js::rust::HandleObject;
use script_bindings::codegen::GenericBindings::AttrBinding::AttrMethods;
use script_bindings::codegen::GenericBindings::DocumentBinding::DocumentMethods;
use script_bindings::codegen::GenericBindings::DocumentFragmentBinding::DocumentFragmentMethods;
use script_bindings::codegen::GenericBindings::HTMLElementBinding::HTMLElementMethods;
use script_bindings::codegen::GenericBindings::NodeBinding::NodeMethods;
use script_bindings::script_runtime::temp_cx;
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
            CommandState::TogglePopover => DOMString::from("toggle-popover"),
            CommandState::HidePopover => DOMString::from("hide-popover"),
            CommandState::ShowPopover => DOMString::from("show-popover"),
        }
    }

    // https://html.spec.whatwg.org/multipage/#dom-button-command
    make_setter!(SetCommand, "command");

    // https://html.spec.whatwg.org/multipage/#dom-popovertargetaction
    make_enumerated_getter!(
        PopoverTargetAction,
        "popovertargetaction",
        "toggle" | "show" | "hide",
        missing => "toggle",
        invalid => "toggle"
    );

    // https://html.spec.whatwg.org/multipage/#dom-popovertargetaction
    make_setter!(SetPopoverTargetAction, "popovertargetaction");

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
    fn CheckValidity(&self, cx: &mut JSContext) -> bool {
        self.check_validity(cx)
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-cva-reportvalidity>
    fn ReportValidity(&self, cx: &mut JSContext) -> bool {
        self.report_validity(cx)
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
                let element = self.upcast::<Element>();
                if element.has_attribute(&local_name!("command")) ||
                    element.has_attribute(&local_name!("commandfor"))
                {
                    ButtonType::Button
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
            .get_attribute(&local_name!("commandfor"))?
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

    /// <https://html.spec.whatwg.org/multipage/#popover-target-element>
    pub(crate) fn popover_target_element(&self) -> Option<DomRoot<Element>> {
        // 1. If node is not a button, then return null.
        // 2. If node is disabled, then return null.
        if self.Disabled() {
            return None;
        }

        // 3. If node has a form owner and node is a submit button, then return null.
        if self.form_owner().is_some() && self.button_type.get() == ButtonType::Submit {
            return None;
        }

        // 4. Let popoverElement be the result of running node's get the popovertarget-associated element.
        let popover_target_value = self
            .upcast::<Element>()
            .get_attribute(&local_name!("popovertarget"))?
            .Value();

        let root_node = self
            .upcast::<Node>()
            .GetRootNode(&GetRootNodeOptions::empty());

        let popover_element = if let Some(document) = root_node.downcast::<Document>() {
            document.GetElementById(popover_target_value)
        } else if let Some(document_fragment) = root_node.downcast::<DocumentFragment>() {
            document_fragment.GetElementById(popover_target_value)
        } else {
            unreachable!("Button element must be in a document or document fragment")
        };

        // 5. If popoverElement is null, then return null.
        // 6. If popoverElement's popover attribute is in the No Popover state, then return null.
        // 7. Return popoverElement.
        if let Some(popover_element) = popover_element {
            if let Some(popover_element) = popover_element.downcast::<HTMLElement>() {
                if popover_element.GetPopover().is_some() {
                    return Some(DomRoot::from_ref(popover_element.upcast()));
                }
            }
        }

        None
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
        if value == "toggle-popover" {
            return CommandState::TogglePopover;
        }
        if value == "show-popover" {
            return CommandState::ShowPopover;
        }
        if value == "hide-popover" {
            return CommandState::HidePopover;
        }

        CommandState::Unknown
    }

    /// <https://html.spec.whatwg.org/multipage/#determine-if-command-is-valid>
    fn determine_if_command_is_valid_for_target(
        command: CommandState,
        target: DomRoot<Element>,
    ) -> bool {
        // Step 1. If command is in the Unknown state, then return false.
        if command == CommandState::Unknown {
            return false;
        }
        // Step 2. If command is in the Custom state, then return true.
        if command == CommandState::Custom {
            return true;
        }
        // Step 3. If target is not an HTML element, then return false.
        if !target.is_html_element() {
            return false;
        }
        // Step 4. If command is in any of the following states:
        // - Toggle Popover
        // - Show Popover
        // - Hide Popover
        // then return true.
        if matches!(
            command,
            CommandState::TogglePopover | CommandState::ShowPopover | CommandState::HidePopover
        ) {
            return true;
        }
        // Step 5. If this standard does not define is valid command steps for target's local name, then return false.
        // Step 6. Otherwise, return the result of running target's corresponding is valid command steps given command.
        vtable_for(target.upcast::<Node>()).is_valid_command_steps(command)
    }

    /// <https://html.spec.whatwg.org/multipage/#the-button-element:concept-fe-optional-value>
    pub(crate) fn optional_value(&self) -> Option<DOMString> {
        // The element's optional value is the value of the element's value attribute,
        // if there is one; otherwise null.
        self.upcast::<Element>()
            .get_attribute(&local_name!("value"))
            .map(|attribute| attribute.Value())
    }

    /// <https://html.spec.whatwg.org/multipage/#popover-target-attribute-activation-behavior>
    pub(crate) fn popover_target_attribute_activation_behavior(
        &self,
        cx: &mut JSContext,
        _event_target: &EventTarget,
    ) {
        // 1. Let popover be node's popover target element.
        let popover = self.popover_target_element();

        // 2. If popover is null, then return.
        if popover.is_none() {
            return;
        }
        let popover = popover.unwrap();

        // TODO: 3. If eventTarget is a shadow-including inclusive descendant of popover and popover
        // is a shadow-including descendant of node, then return.

        // 4. If node's popovertargetaction attribute is in the show state and popover's popover
        // visibility state is showing, then return.
        if self.PopoverTargetAction() == DOMString::from("show") && popover.popover_open_state() {
            return;
        }

        // 5. If node's popovertargetaction attribute is in the hide state and popover's popover
        // visibility state is hidden, then return.
        if self.PopoverTargetAction() == DOMString::from("hide") && !popover.popover_open_state() {
            return;
        }

        let popover_element = popover.downcast::<HTMLElement>().unwrap();
        // 6. If popover's popover visibility state is showing, then
        if popover.popover_open_state() {
            // run the hide popover algorithm given popover, true, true, false, and node.
            popover_element
                .hide_popover(
                    cx,
                    true,
                    true,
                    false,
                    Some(DomRoot::from_ref(self.upcast::<HTMLElement>())),
                )
                .expect("This shouldn't fail in this codepath");
        }
        // 7. Otherwise, if popover's popover visibility state is hidden and the result of running
        // check popover validity given popover, false, false, and null is true,
        else if popover_element
            .check_popover_validity(false, false, None)
            .expect("This shouldn't fail in this codepath")
        {
            // then run show popover given popover, false, and node.
            popover_element
                .show_popover(
                    cx,
                    false,
                    Some(DomRoot::from_ref(self.upcast::<HTMLElement>())),
                )
                .expect("This shouldn't fail in this codepath");
        }
    }
}

impl VirtualMethods for HTMLButtonElement {
    fn super_type(&self) -> Option<&dyn VirtualMethods> {
        Some(self.upcast::<HTMLElement>() as &dyn VirtualMethods)
    }

    fn attribute_mutated(
        &self,
        cx: &mut js::context::JSContext,
        attr: &Attr,
        mutation: AttributeMutation,
    ) {
        self.super_type()
            .unwrap()
            .attribute_mutated(cx, attr, mutation);
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
                self.validity_state(CanGc::from_cx(cx))
                    .perform_validation_and_update(ValidationFlags::all(), CanGc::from_cx(cx));
            },
            local_name!("type") => self.set_type(attr.Value(), CanGc::from_cx(cx)),
            local_name!("command") => self.set_type(
                self.upcast::<Element>()
                    .get_string_attribute(&local_name!("type")),
                CanGc::from_cx(cx),
            ),
            local_name!("commandfor") => self.set_type(
                self.upcast::<Element>()
                    .get_string_attribute(&local_name!("type")),
                CanGc::from_cx(cx),
            ),
            local_name!("form") => {
                self.form_attribute_mutated(mutation, CanGc::from_cx(cx));
                self.validity_state(CanGc::from_cx(cx))
                    .perform_validation_and_update(ValidationFlags::empty(), CanGc::from_cx(cx));
            },
            _ => {},
        }
    }

    fn bind_to_tree(&self, cx: &mut JSContext, context: &BindContext) {
        if let Some(s) = self.super_type() {
            s.bind_to_tree(cx, context);
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
    #[expect(unsafe_code)]
    fn activation_behavior(&self, _event: &Event, target: &EventTarget, can_gc: CanGc) {
        let mut cx = unsafe { temp_cx() };
        let cx = &mut cx;

        // Step 2. If element's node document is not fully active, then return.
        if !target
            .downcast::<Node>()
            .is_none_or(|node| node.owner_document().is_fully_active())
        {
            return;
        }

        let button_type = self.button_type.get();
        // Step 3. If element has a form owner:
        if let Some(owner) = self.form_owner() {
            // Step 3.1 If element is a submit button, then submit element's form owner from element
            // ..., and return.
            if button_type == ButtonType::Submit {
                owner.submit(
                    SubmittedFrom::NotFromForm,
                    FormSubmitterElement::Button(self),
                    can_gc,
                );
                return;
            }
            // Step 3.2 If element's type attribute is in the Reset Button state, then reset
            // element's form owner and return.
            if button_type == ButtonType::Reset {
                owner.reset(ResetFrom::NotFromForm, can_gc);
                return;
            }
            // Step 3.3 If element's type attribute is in the Auto state, then return.
            if button_type == ButtonType::Button &&
                self.upcast::<Element>()
                    .get_string_attribute(&local_name!("type"))
                    .to_ascii_lowercase() !=
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
            // Step 5.2 If the result of determining if a command is valid for a target given command and target is false, then return.
            if !Self::determine_if_command_is_valid_for_target(command, target.clone()) {
                return;
            }
            // Step 5.3 Let continue be the result of firing an event named command at target, using
            // CommandEvent, with its command attribute initialized to command, its source attribute
            // initialized to element, and its cancelable attribute initialized to true.
            // source attribute
            // Step 5.4 If continue is false, then return.
            let event = CommandEvent::new(
                &self.owner_window(),
                atom!("command"),
                EventBubbles::DoesNotBubble,
                EventCancelable::Cancelable,
                Some(DomRoot::from_ref(self.upcast())),
                self.upcast::<Element>()
                    .get_string_attribute(&local_name!("command")),
                can_gc,
            );
            let event = event.upcast::<Event>();
            if !event.fire(target.upcast::<EventTarget>(), can_gc) {
                return;
            }
            // Step 5.5 If target is not connected, then return.
            let target_node = target.upcast::<Node>();
            if !target_node.is_connected() {
                return;
            }
            // Step 5.6 If command is in the Custom state, then return.
            if command == CommandState::Custom {
                return;
            }

            let element_target = target.downcast::<HTMLElement>().unwrap();
            // Step 5.7. If command is in the Hide Popover state:
            if command == CommandState::HidePopover {
                // Step 5.7.1. If the result of running check popover validity given target, true,
                // false, and null is true, then run the hide popover algorithm given target, true,
                // true, false, and element.
                if element_target
                    .check_popover_validity(true, false, None)
                    .expect("This should never fail in this codepath")
                {
                    element_target
                        .hide_popover(
                            cx,
                            true,
                            true,
                            false,
                            Some(DomRoot::from_ref(self.upcast::<HTMLElement>())),
                        )
                        .expect("This should never fail in this codepath");
                }
            }
            // Step 5.8. Otherwise, if command is in the Toggle Popover state:
            else if command == CommandState::TogglePopover {
                // Step 5.8.1. If the result of running check popover validity given target, false,
                // false, and null is true, then run the show popover algorithm given target, false,
                // and element.
                if element_target
                    .check_popover_validity(false, false, None)
                    .expect("This should never fail in this codepath")
                {
                    element_target
                        .show_popover(
                            cx,
                            false,
                            Some(DomRoot::from_ref(self.upcast::<HTMLElement>())),
                        )
                        .expect("This should never fail in this codepath");
                }
                // Step 5.8.2. Otherwise, if the result of running check popover validity given
                // target, true, false, and null is true, then run the hide popover algorithm given
                // target, true, true, false, and element.
                else if element_target
                    .check_popover_validity(true, false, None)
                    .expect("This should never fail in this codepath")
                {
                    element_target
                        .hide_popover(
                            cx,
                            true,
                            true,
                            false,
                            Some(DomRoot::from_ref(self.upcast::<HTMLElement>())),
                        )
                        .expect("This should never fail in this codepath");
                }
            // Step 5.9. Otherwise, if command is in the Show Popover state:
            } else if command == CommandState::ShowPopover {
                // Step 5.9.1. If the result of running check popover validity given target, false,
                // false, and null is true, then run the show popover algorithm given target, false,
                // and element.
                if element_target
                    .check_popover_validity(false, false, None)
                    .expect("This should never fail in this codepath")
                {
                    element_target
                        .show_popover(
                            cx,
                            false,
                            Some(DomRoot::from_ref(self.upcast::<HTMLElement>())),
                        )
                        .expect("This should never fail in this codepath");
                }
            } else {
                // Step 5.10 Otherwise, if this standard defines command steps for target's local name,
                // then run the corresponding command steps given target, element, and command.
                let _ =
                    vtable_for(target_node).command_steps(DomRoot::from_ref(self), command, can_gc);
            }
        } else {
            // Step 6 Otherwise, run the popover target attribute activation behavior given element
            // and event's target.
            self.popover_target_attribute_activation_behavior(cx, target)
        }
    }
}

/// <https://html.spec.whatwg.org/multipage/#attr-button-command>
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub(crate) enum CommandState {
    Unknown,
    Custom,
    TogglePopover,
    ShowPopover,
    HidePopover,
    ShowModal,
    Close,
}

/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::activation::Activatable;
use dom::attr::{Attr, AttrValue};
use dom::bindings::cell::DOMRefCell;
use dom::bindings::codegen::Bindings::AttrBinding::AttrMethods;
use dom::bindings::codegen::Bindings::EventBinding::EventMethods;
use dom::bindings::codegen::Bindings::HTMLInputElementBinding;
use dom::bindings::codegen::Bindings::HTMLInputElementBinding::HTMLInputElementMethods;
use dom::bindings::codegen::Bindings::KeyboardEventBinding::KeyboardEventMethods;
use dom::bindings::codegen::InheritTypes::KeyboardEventCast;
use dom::bindings::codegen::InheritTypes::{ElementCast, HTMLElementCast, HTMLInputElementCast, NodeCast};
use dom::bindings::codegen::InheritTypes::{HTMLInputElementDerived, HTMLFieldSetElementDerived, EventTargetCast};
use dom::bindings::global::GlobalRef;
use dom::bindings::js::{JS, LayoutJS, Root, RootedReference};
use dom::document::Document;
use dom::element::{Element, ElementTypeId, RawLayoutElementHelpers};
use dom::event::{Event, EventBubbles, EventCancelable};
use dom::eventtarget::{EventTarget, EventTargetTypeId};
use dom::htmlelement::{HTMLElement, HTMLElementTypeId};
use dom::htmlformelement::{FormSubmitter, FormControl, HTMLFormElement};
use dom::htmlformelement::{SubmittedFrom, ResetFrom};
use dom::keyboardevent::KeyboardEvent;
use dom::node::{Node, NodeDamage, NodeTypeId};
use dom::node::{document_from_node, window_from_node};
use dom::virtualmethods::VirtualMethods;
use msg::constellation_msg::ConstellationChan;
use textinput::KeyReaction::{TriggerDefaultAction, DispatchInput, Nothing};
use textinput::Lines::Single;
use textinput::TextInput;

use string_cache::Atom;
use util::str::DOMString;

use std::borrow::ToOwned;
use std::cell::Cell;

const DEFAULT_SUBMIT_VALUE: &'static str = "Submit";
const DEFAULT_RESET_VALUE: &'static str = "Reset";

#[derive(JSTraceable, PartialEq, Copy, Clone)]
#[allow(dead_code)]
#[derive(HeapSizeOf)]
enum InputType {
    InputSubmit,
    InputReset,
    InputButton,
    InputText,
    InputFile,
    InputImage,
    InputCheckbox,
    InputRadio,
    InputPassword
}

#[dom_struct]
pub struct HTMLInputElement {
    htmlelement: HTMLElement,
    input_type: Cell<InputType>,
    checked: Cell<bool>,
    checked_changed: Cell<bool>,
    placeholder: DOMRefCell<DOMString>,
    indeterminate: Cell<bool>,
    value_changed: Cell<bool>,
    size: Cell<u32>,
    #[ignore_heap_size_of = "#7193"]
    textinput: DOMRefCell<TextInput<ConstellationChan>>,
    activation_state: DOMRefCell<InputActivationState>,
}

impl PartialEq for HTMLInputElement {
    fn eq(&self, other: &HTMLInputElement) -> bool {
        self as *const HTMLInputElement == &*other
    }
}

#[derive(JSTraceable)]
#[must_root]
#[derive(HeapSizeOf)]
struct InputActivationState {
    indeterminate: bool,
    checked: bool,
    checked_changed: bool,
    checked_radio: Option<JS<HTMLInputElement>>,
    // In case mutability changed
    was_mutable: bool,
    // In case the type changed
    old_type: InputType,
}

impl InputActivationState {
    fn new() -> InputActivationState {
        InputActivationState {
            indeterminate: false,
            checked: false,
            checked_changed: false,
            checked_radio: None,
            was_mutable: false,
            old_type: InputType::InputText
        }
    }
}

impl HTMLInputElementDerived for EventTarget {
    fn is_htmlinputelement(&self) -> bool {
        *self.type_id() ==
            EventTargetTypeId::Node(
                NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLInputElement)))
    }
}

static DEFAULT_INPUT_SIZE: u32 = 20;

impl HTMLInputElement {
    fn new_inherited(localName: DOMString, prefix: Option<DOMString>, document: &Document) -> HTMLInputElement {
        let chan = document.window().r().constellation_chan();
        HTMLInputElement {
            htmlelement: HTMLElement::new_inherited(HTMLElementTypeId::HTMLInputElement, localName, prefix, document),
            input_type: Cell::new(InputType::InputText),
            checked: Cell::new(false),
            placeholder: DOMRefCell::new("".to_owned()),
            indeterminate: Cell::new(false),
            checked_changed: Cell::new(false),
            value_changed: Cell::new(false),
            size: Cell::new(DEFAULT_INPUT_SIZE),
            textinput: DOMRefCell::new(TextInput::new(Single, "".to_owned(), chan)),
            activation_state: DOMRefCell::new(InputActivationState::new())
        }
    }

    #[allow(unrooted_must_root)]
    pub fn new(localName: DOMString,
               prefix: Option<DOMString>,
               document: &Document) -> Root<HTMLInputElement> {
        let element = HTMLInputElement::new_inherited(localName, prefix, document);
        Node::reflect_node(box element, document, HTMLInputElementBinding::Wrap)
    }
}

pub trait LayoutHTMLInputElementHelpers {
    #[allow(unsafe_code)]
    unsafe fn get_value_for_layout(self) -> String;
    #[allow(unsafe_code)]
    unsafe fn get_size_for_layout(self) -> u32;
}

pub trait RawLayoutHTMLInputElementHelpers {
    #[allow(unsafe_code)]
    unsafe fn get_checked_state_for_layout(&self) -> bool;
    #[allow(unsafe_code)]
    unsafe fn get_indeterminate_state_for_layout(&self) -> bool;
    #[allow(unsafe_code)]
    unsafe fn get_size_for_layout(&self) -> u32;
}

impl LayoutHTMLInputElementHelpers for LayoutJS<HTMLInputElement> {
    #[allow(unsafe_code)]
    unsafe fn get_value_for_layout(self) -> String {
        #[allow(unsafe_code)]
        unsafe fn get_raw_textinput_value(input: LayoutJS<HTMLInputElement>) -> String {
            let textinput = (*input.unsafe_get()).textinput.borrow_for_layout().get_content();
            if !textinput.is_empty() {
                textinput
            } else {
                (*input.unsafe_get()).placeholder.borrow_for_layout().to_owned()
            }
        }

        #[allow(unsafe_code)]
        unsafe fn get_raw_attr_value(input: LayoutJS<HTMLInputElement>) -> Option<String> {
            let elem = ElementCast::from_layout_js(&input);
            (*elem.unsafe_get()).get_attr_val_for_layout(&ns!(""), &atom!("value"))
                                .map(|s| s.to_owned())
        }

        match (*self.unsafe_get()).input_type.get() {
            InputType::InputCheckbox | InputType::InputRadio => "".to_owned(),
            InputType::InputFile | InputType::InputImage => "".to_owned(),
            InputType::InputButton => get_raw_attr_value(self).unwrap_or_else(|| "".to_owned()),
            InputType::InputSubmit => get_raw_attr_value(self).unwrap_or_else(|| DEFAULT_SUBMIT_VALUE.to_owned()),
            InputType::InputReset => get_raw_attr_value(self).unwrap_or_else(|| DEFAULT_RESET_VALUE.to_owned()),
            InputType::InputPassword => {
                let raw = get_raw_textinput_value(self);
                raw.chars().map(|_| '●').collect()
            }
            _ => get_raw_textinput_value(self),
        }
    }

    #[allow(unrooted_must_root)]
    #[allow(unsafe_code)]
    unsafe fn get_size_for_layout(self) -> u32 {
        (*self.unsafe_get()).get_size_for_layout()
    }
}

impl RawLayoutHTMLInputElementHelpers for HTMLInputElement {
    #[allow(unrooted_must_root)]
    #[allow(unsafe_code)]
    unsafe fn get_checked_state_for_layout(&self) -> bool {
        self.checked.get()
    }

    #[allow(unrooted_must_root)]
    #[allow(unsafe_code)]
    unsafe fn get_indeterminate_state_for_layout(&self) -> bool {
        self.indeterminate.get()
    }

    #[allow(unrooted_must_root)]
    #[allow(unsafe_code)]
    unsafe fn get_size_for_layout(&self) -> u32 {
        self.size.get()
    }
}

impl HTMLInputElementMethods for HTMLInputElement {
    // https://www.whatwg.org/html/#dom-fe-disabled
    make_bool_getter!(Disabled);

    // https://www.whatwg.org/html/#dom-fe-disabled
    make_bool_setter!(SetDisabled, "disabled");

    // https://html.spec.whatwg.org/multipage/#dom-input-defaultchecked
    make_bool_getter!(DefaultChecked, "checked");

    // https://html.spec.whatwg.org/multipage/#dom-input-defaultchecked
    make_bool_setter!(SetDefaultChecked, "checked");

    // https://html.spec.whatwg.org/multipage/#dom-input-checked
    fn Checked(&self) -> bool {
        self.checked.get()
    }

    // https://html.spec.whatwg.org/multipage/#dom-input-checked
    fn SetChecked(&self, checked: bool) {
        self.update_checked_state(checked, true);
    }

    // https://html.spec.whatwg.org/multipage/#dom-input-readonly
    make_bool_getter!(ReadOnly);

    // https://html.spec.whatwg.org/multipage/#dom-input-readonly
    make_bool_setter!(SetReadOnly, "readonly");

    // https://html.spec.whatwg.org/multipage/#dom-input-size
    make_uint_getter!(Size, "size", DEFAULT_INPUT_SIZE);
    make_limited_uint_setter!(SetSize, "size", DEFAULT_INPUT_SIZE);

    // https://html.spec.whatwg.org/multipage/#dom-input-type
    make_enumerated_getter!(Type, "text", ("hidden") | ("search") | ("tel") |
                                  ("url") | ("email") | ("password") |
                                  ("datetime") | ("date") | ("month") |
                                  ("week") | ("time") | ("datetime-local") |
                                  ("number") | ("range") | ("color") |
                                  ("checkbox") | ("radio") | ("file") |
                                  ("submit") | ("image") | ("reset") | ("button"));

    // https://html.spec.whatwg.org/multipage/#dom-input-type
    make_setter!(SetType, "type");

    // https://html.spec.whatwg.org/multipage/#dom-input-value
    fn Value(&self) -> DOMString {
        self.textinput.borrow().get_content()
    }

    // https://html.spec.whatwg.org/multipage/#dom-input-value
    fn SetValue(&self, value: DOMString) {
        self.textinput.borrow_mut().set_content(value);
        self.value_changed.set(true);
        self.force_relayout();
    }

    // https://html.spec.whatwg.org/multipage/#dom-input-defaultvalue
    make_getter!(DefaultValue, "value");

    // https://html.spec.whatwg.org/multipage/#dom-input-defaultvalue
    make_setter!(SetDefaultValue, "value");

    // https://html.spec.whatwg.org/multipage/#attr-fe-name
    make_getter!(Name);

    // https://html.spec.whatwg.org/multipage/#attr-fe-name
    make_setter!(SetName, "name");

    // https://html.spec.whatwg.org/multipage/#attr-input-placeholder
    make_getter!(Placeholder);

    // https://html.spec.whatwg.org/multipage/#attr-input-placeholder
    make_setter!(SetPlaceholder, "placeholder");

    // https://html.spec.whatwg.org/multipage/#dom-input-formaction
    make_url_or_base_getter!(FormAction);

    // https://html.spec.whatwg.org/multipage/#dom-input-formaction
    make_setter!(SetFormAction, "formaction");

    // https://html.spec.whatwg.org/multipage/#dom-input-formenctype
    make_enumerated_getter!(
        FormEnctype, "application/x-www-form-urlencoded", ("text/plain") | ("multipart/form-data"));

    // https://html.spec.whatwg.org/multipage/#dom-input-formenctype
    make_setter!(SetFormEnctype, "formenctype");

    // https://html.spec.whatwg.org/multipage/#dom-input-formmethod
    make_enumerated_getter!(FormMethod, "get", ("post") | ("dialog"));

    // https://html.spec.whatwg.org/multipage/#dom-input-formmethod
    make_setter!(SetFormMethod, "formmethod");

    // https://html.spec.whatwg.org/multipage/#dom-input-formtarget
    make_getter!(FormTarget);

    // https://html.spec.whatwg.org/multipage/#dom-input-formtarget
    make_setter!(SetFormTarget, "formtarget");

    // https://html.spec.whatwg.org/multipage/#dom-input-indeterminate
    fn Indeterminate(&self) -> bool {
        self.indeterminate.get()
    }

    // https://html.spec.whatwg.org/multipage/#dom-input-indeterminate
    fn SetIndeterminate(&self, val: bool) {
        self.indeterminate.set(val)
    }
}


#[allow(unsafe_code)]
fn broadcast_radio_checked(broadcaster: &HTMLInputElement, group: Option<&str>) {
    //TODO: if not in document, use root ancestor instead of document
    let owner = broadcaster.form_owner();
    let doc = document_from_node(broadcaster);
    let doc_node = NodeCast::from_ref(doc.r());

    // This function is a workaround for lifetime constraint difficulties.
    fn do_broadcast(doc_node: &Node, broadcaster: &HTMLInputElement,
                        owner: Option<&HTMLFormElement>, group: Option<&str>) {
        // There is no DOM tree manipulation here, so this is safe
        let iter = unsafe {
            doc_node.query_selector_iter("input[type=radio]".to_owned()).unwrap()
                .filter_map(HTMLInputElementCast::to_root)
                .filter(|r| in_same_group(r.r(), owner, group) && broadcaster != r.r())
        };
        for ref r in iter {
            if r.r().Checked() {
                r.r().SetChecked(false);
            }
        }
    }

    do_broadcast(doc_node, broadcaster, owner.r(), group)
}

fn in_same_group<'a,'b>(other: &'a HTMLInputElement,
                        owner: Option<&'b HTMLFormElement>,
                        group: Option<&str>) -> bool {
    let other_owner = other.form_owner();
    let other_owner = other_owner.r();
    other.input_type.get() == InputType::InputRadio &&
    // TODO Both a and b are in the same home subtree.
    other_owner == owner &&
    // TODO should be a unicode compatibility caseless match
    match (other.get_radio_group_name(), group) {
        (Some(ref s1), Some(s2)) => &**s1 == s2,
        (None, None) => true,
        _ => false
    }
}

impl HTMLInputElement {
    pub fn force_relayout(&self) {
        let doc = document_from_node(self);
        let node = NodeCast::from_ref(self);
        doc.r().content_changed(node, NodeDamage::OtherNodeDamage)
    }

    pub fn radio_group_updated(&self, group: Option<&str>) {
        if self.Checked() {
            broadcast_radio_checked(self, group);
        }
    }

    pub fn get_radio_group_name(&self) -> Option<String> {
        //TODO: determine form owner
        let elem = ElementCast::from_ref(self);
        elem.get_attribute(&ns!(""), &atom!("name"))
            .map(|name| name.r().Value())
    }

    pub fn update_checked_state(&self, checked: bool, dirty: bool) {
        self.checked.set(checked);

        if dirty {
            self.checked_changed.set(true);
        }

        if self.input_type.get() == InputType::InputRadio && checked {
            broadcast_radio_checked(self,
                                    self.get_radio_group_name()
                                        .as_ref()
                                        .map(|group| &**group));
        }

        self.force_relayout();
        //TODO: dispatch change event
    }

    pub fn get_size(&self) -> u32 {
        self.size.get()
    }

    pub fn get_indeterminate_state(&self) -> bool {
        self.indeterminate.get()
    }

    // https://html.spec.whatwg.org/multipage/#concept-fe-mutable
    pub fn mutable(&self) -> bool {
        // https://html.spec.whatwg.org/multipage/#the-input-element:concept-fe-mutable
        // https://html.spec.whatwg.org/multipage/#the-readonly-attribute:concept-fe-mutable
        let node = NodeCast::from_ref(self);
        !(node.get_disabled_state() || self.ReadOnly())
    }

    // https://html.spec.whatwg.org/multipage/#the-input-element:concept-form-reset-control
    pub fn reset(&self) {
        match self.input_type.get() {
            InputType::InputRadio | InputType::InputCheckbox => {
                self.update_checked_state(self.DefaultChecked(), false);
                self.checked_changed.set(false);
            },
            InputType::InputImage => (),
            _ => ()
        }

        self.SetValue(self.DefaultValue());
        self.value_changed.set(false);
        self.force_relayout();
    }
}

impl VirtualMethods for HTMLInputElement {
    fn super_type<'b>(&'b self) -> Option<&'b VirtualMethods> {
        let htmlelement: &HTMLElement = HTMLElementCast::from_ref(self);
        Some(htmlelement as &VirtualMethods)
    }

    fn after_set_attr(&self, attr: &Attr) {
        if let Some(ref s) = self.super_type() {
            s.after_set_attr(attr);
        }

        match attr.local_name() {
            &atom!("disabled") => {
                let node = NodeCast::from_ref(self);
                node.set_disabled_state(true);
                node.set_enabled_state(false);
            }
            &atom!("checked") => {
                // https://html.spec.whatwg.org/multipage/#the-input-element:concept-input-checked-dirty
                if !self.checked_changed.get() {
                    self.update_checked_state(true, false);
                }
            }
            &atom!("size") => {
                match *attr.value() {
                    AttrValue::UInt(_, value) => self.size.set(value),
                    _ => panic!("Expected an AttrValue::UInt"),
                }
            }
            &atom!("type") => {
                let value = attr.value();
                self.input_type.set(match &**value {
                    "button" => InputType::InputButton,
                    "submit" => InputType::InputSubmit,
                    "reset" => InputType::InputReset,
                    "file" => InputType::InputFile,
                    "radio" => InputType::InputRadio,
                    "checkbox" => InputType::InputCheckbox,
                    "password" => InputType::InputPassword,
                    _ => InputType::InputText,
                });
                if self.input_type.get() == InputType::InputRadio {
                    self.radio_group_updated(self.get_radio_group_name()
                                                 .as_ref()
                                                 .map(|group| &**group));
                }
            }
            &atom!("value") => {
                if !self.value_changed.get() {
                    self.textinput.borrow_mut().set_content((**attr.value()).to_owned());
                }
            }
            &atom!("name") => {
                if self.input_type.get() == InputType::InputRadio {
                    let value = attr.value();
                    self.radio_group_updated(Some(&value));
                }
            }
            _ if attr.local_name() == &Atom::from_slice("placeholder") => {
                let value = attr.value();
                let stripped = value.chars()
                    .filter(|&c| c != '\n' && c != '\r')
                    .collect::<String>();
                *self.placeholder.borrow_mut() = stripped;
            }
            _ => ()
        }
    }

    fn before_remove_attr(&self, attr: &Attr) {
        if let Some(ref s) = self.super_type() {
            s.before_remove_attr(attr);
        }

        match attr.local_name() {
            &atom!("disabled") => {
                let node = NodeCast::from_ref(self);
                node.set_disabled_state(false);
                node.set_enabled_state(true);
                node.check_ancestors_disabled_state_for_form_control();
            }
            &atom!("checked") => {
                // https://html.spec.whatwg.org/multipage/#the-input-element:concept-input-checked-dirty
                if !self.checked_changed.get() {
                    self.update_checked_state(false, false);
                }
            }
            &atom!("size") => {
                self.size.set(DEFAULT_INPUT_SIZE);
            }
            &atom!("type") => {
                if self.input_type.get() == InputType::InputRadio {
                    broadcast_radio_checked(self,
                                            self.get_radio_group_name()
                                                .as_ref()
                                                .map(|group| &**group));
                }
                self.input_type.set(InputType::InputText);
            }
            &atom!("value") => {
                if !self.value_changed.get() {
                    self.textinput.borrow_mut().set_content("".to_owned());
                }
            }
            &atom!("name") => {
                if self.input_type.get() == InputType::InputRadio {
                    self.radio_group_updated(None);
                }
            }
            _ if attr.local_name() == &Atom::from_slice("placeholder") => {
                self.placeholder.borrow_mut().clear();
            }
            _ => ()
        }
    }

    fn parse_plain_attribute(&self, name: &Atom, value: DOMString) -> AttrValue {
        match name {
            &atom!("size") => AttrValue::from_limited_u32(value, DEFAULT_INPUT_SIZE),
            _ => self.super_type().unwrap().parse_plain_attribute(name, value),
        }
    }

    fn bind_to_tree(&self, tree_in_doc: bool) {
        if let Some(ref s) = self.super_type() {
            s.bind_to_tree(tree_in_doc);
        }

        let node = NodeCast::from_ref(self);
        node.check_ancestors_disabled_state_for_form_control();
    }

    fn unbind_from_tree(&self, tree_in_doc: bool) {
        if let Some(ref s) = self.super_type() {
            s.unbind_from_tree(tree_in_doc);
        }

        let node = NodeCast::from_ref(self);
        if node.ancestors().any(|ancestor| ancestor.r().is_htmlfieldsetelement()) {
            node.check_ancestors_disabled_state_for_form_control();
        } else {
            node.check_disabled_attribute();
        }
    }

    fn handle_event(&self, event: &Event) {
        if let Some(s) = self.super_type() {
            s.handle_event(event);
        }

        if &*event.Type() == "click" && !event.DefaultPrevented() {
            match self.input_type.get() {
                InputType::InputRadio => self.update_checked_state(true, true),
                _ => {}
            }

            // TODO: Dispatch events for non activatable inputs
            // https://html.spec.whatwg.org/multipage/#common-input-element-events

            //TODO: set the editing position for text inputs

            let doc = document_from_node(self);
            doc.r().request_focus(ElementCast::from_ref(self));
        } else if &*event.Type() == "keydown" && !event.DefaultPrevented() &&
            (self.input_type.get() == InputType::InputText ||
             self.input_type.get() == InputType::InputPassword) {
                let keyevent: Option<&KeyboardEvent> = KeyboardEventCast::to_ref(event);
                keyevent.map(|keyevent| {
                    // This can't be inlined, as holding on to textinput.borrow_mut()
                    // during self.implicit_submission will cause a panic.
                    let action = self.textinput.borrow_mut().handle_keydown(keyevent);
                    match action {
                        TriggerDefaultAction => {
                            self.implicit_submission(keyevent.CtrlKey(),
                                                     keyevent.ShiftKey(),
                                                     keyevent.AltKey(),
                                                     keyevent.MetaKey());
                        },
                        DispatchInput => {
                            self.value_changed.set(true);
                            self.force_relayout();
                            event.PreventDefault();
                        }
                        Nothing => (),
                    }
                });
        }
    }
}

impl<'a> FormControl<'a> for &'a HTMLInputElement {
    fn to_element(self) -> &'a Element {
        ElementCast::from_ref(self)
    }
}

impl Activatable for HTMLInputElement {
    fn as_element<'b>(&'b self) -> &'b Element {
        ElementCast::from_ref(self)
    }

    fn is_instance_activatable(&self) -> bool {
        match self.input_type.get() {
            // https://html.spec.whatwg.org/multipage/#submit-button-state-%28type=submit%29:activation-behaviour-2
            // https://html.spec.whatwg.org/multipage/#reset-button-state-%28type=reset%29:activation-behaviour-2
            // https://html.spec.whatwg.org/multipage/#checkbox-state-%28type=checkbox%29:activation-behaviour-2
            // https://html.spec.whatwg.org/multipage/#radio-button-state-%28type=radio%29:activation-behaviour-2
            InputType::InputSubmit | InputType::InputReset
            | InputType::InputCheckbox | InputType::InputRadio => self.mutable(),
            _ => false
        }
    }

    // https://html.spec.whatwg.org/multipage/#run-pre-click-activation-steps
    #[allow(unsafe_code)]
    fn pre_click_activation(&self) {
        let mut cache = self.activation_state.borrow_mut();
        let ty = self.input_type.get();
        cache.old_type = ty;
        cache.was_mutable = self.mutable();
        if cache.was_mutable {
            match ty {
                // https://html.spec.whatwg.org/multipage/#submit-button-state-(type=submit):activation-behavior
                // InputType::InputSubmit => (), // No behavior defined
                // https://html.spec.whatwg.org/multipage/#reset-button-state-(type=reset):activation-behavior
                // InputType::InputSubmit => (), // No behavior defined
                InputType::InputCheckbox => {
                    /*
                    https://html.spec.whatwg.org/multipage/#checkbox-state-(type=checkbox):pre-click-activation-steps
                    cache current values of `checked` and `indeterminate`
                    we may need to restore them later
                    */
                    cache.indeterminate = self.Indeterminate();
                    cache.checked = self.Checked();
                    cache.checked_changed = self.checked_changed.get();
                    self.SetIndeterminate(false);
                    self.SetChecked(!cache.checked);
                },
                // https://html.spec.whatwg.org/multipage/#radio-button-state-(type=radio):pre-click-activation-steps
                InputType::InputRadio => {
                    //TODO: if not in document, use root ancestor instead of document
                    let owner = self.form_owner();
                    let doc = document_from_node(self);
                    let doc_node = NodeCast::from_ref(doc.r());
                    let group = self.get_radio_group_name();;

                    // Safe since we only manipulate the DOM tree after finding an element
                    let checked_member = unsafe {
                        doc_node.query_selector_iter("input[type=radio]".to_owned()).unwrap()
                                .filter_map(HTMLInputElementCast::to_root)
                                .find(|r| {
                                    in_same_group(r.r(), owner.r(), group.as_ref().map(|gr| &**gr)) &&
                                    r.r().Checked()
                                })
                    };
                    cache.checked_radio = checked_member.r().map(JS::from_ref);
                    cache.checked_changed = self.checked_changed.get();
                    self.SetChecked(true);
                }
                _ => ()
            }
        }
    }

    // https://html.spec.whatwg.org/multipage/#run-canceled-activation-steps
    fn canceled_activation(&self) {
        let cache = self.activation_state.borrow();
        let ty = self.input_type.get();
        if cache.old_type != ty  {
            // Type changed, abandon ship
            // https://www.w3.org/Bugs/Public/show_bug.cgi?id=27414
            return;
        }
        match ty {
            // https://html.spec.whatwg.org/multipage/#submit-button-state-(type=submit):activation-behavior
            // InputType::InputSubmit => (), // No behavior defined
            // https://html.spec.whatwg.org/multipage/#reset-button-state-(type=reset):activation-behavior
            // InputType::InputReset => (), // No behavior defined
            // https://html.spec.whatwg.org/multipage/#checkbox-state-(type=checkbox):canceled-activation-steps
            InputType::InputCheckbox => {
                // We want to restore state only if the element had been changed in the first place
                if cache.was_mutable {
                    self.SetIndeterminate(cache.indeterminate);
                    self.SetChecked(cache.checked);
                    self.checked_changed.set(cache.checked_changed);
                }
            },
            // https://html.spec.whatwg.org/multipage/#radio-button-state-(type=radio):canceled-activation-steps
            InputType::InputRadio => {
                // We want to restore state only if the element had been changed in the first place
                if cache.was_mutable {
                    let old_checked: Option<Root<HTMLInputElement>> = cache.checked_radio.map(|t| t.root());
                    let name = self.get_radio_group_name();
                    match old_checked {
                        Some(ref o) => {
                            // Avoiding iterating through the whole tree here, instead
                            // we can check if the conditions for radio group siblings apply
                            if name == o.r().get_radio_group_name() && // TODO should be compatibility caseless
                               self.form_owner() == o.r().form_owner() &&
                               // TODO Both a and b are in the same home subtree
                               o.r().input_type.get() == InputType::InputRadio {
                                    o.r().SetChecked(true);
                            } else {
                                self.SetChecked(false);
                            }
                        },
                        None => self.SetChecked(false)
                    };
                    self.checked_changed.set(cache.checked_changed);
                }
            }
            _ => ()
        }
    }

    // https://html.spec.whatwg.org/multipage/#run-post-click-activation-steps
    fn activation_behavior(&self, _event: &Event, _target: &EventTarget) {
        let ty = self.input_type.get();
        if self.activation_state.borrow().old_type != ty {
            // Type changed, abandon ship
            // https://www.w3.org/Bugs/Public/show_bug.cgi?id=27414
            return;
        }
        match ty {
            InputType::InputSubmit => {
                // https://html.spec.whatwg.org/multipage/#submit-button-state-(type=submit):activation-behavior
                // FIXME (Manishearth): support document owners (needs ability to get parent browsing context)
                if self.mutable() /* and document owner is fully active */ {
                    self.form_owner().map(|o| {
                        o.r().submit(SubmittedFrom::NotFromFormSubmitMethod,
                                     FormSubmitter::InputElement(self.clone()))
                    });
                }
            },
            InputType::InputReset => {
                // https://html.spec.whatwg.org/multipage/#reset-button-state-(type=reset):activation-behavior
                // FIXME (Manishearth): support document owners (needs ability to get parent browsing context)
                if self.mutable() /* and document owner is fully active */ {
                    self.form_owner().map(|o| {
                        o.r().reset(ResetFrom::NotFromFormResetMethod)
                    });
                }
            },
            InputType::InputCheckbox | InputType::InputRadio => {
                // https://html.spec.whatwg.org/multipage/#checkbox-state-(type=checkbox):activation-behavior
                // https://html.spec.whatwg.org/multipage/#radio-button-state-(type=radio):activation-behavior
                if self.mutable() {
                    let win = window_from_node(self);
                    let event = Event::new(GlobalRef::Window(win.r()),
                                           "input".to_owned(),
                                           EventBubbles::Bubbles,
                                           EventCancelable::NotCancelable);
                    let target = EventTargetCast::from_ref(self);
                    event.r().fire(target);

                    let event = Event::new(GlobalRef::Window(win.r()),
                                           "change".to_owned(),
                                           EventBubbles::Bubbles,
                                           EventCancelable::NotCancelable);
                    let target = EventTargetCast::from_ref(self);
                    event.r().fire(target);
                }
            },
            _ => ()
        }
    }

    // https://html.spec.whatwg.org/multipage/#implicit-submission
    #[allow(unsafe_code)]
    fn implicit_submission(&self, ctrlKey: bool, shiftKey: bool, altKey: bool, metaKey: bool) {
        let doc = document_from_node(self);
        let node = NodeCast::from_ref(doc.r());
        let owner = self.form_owner();
        let form = match owner {
            None => return,
            Some(ref f) => f
        };

        let elem = ElementCast::from_ref(self);
        if elem.click_in_progress() {
            return;
        }
        // This is safe because we are stopping after finding the first element
        // and only then performing actions which may modify the DOM tree
        let submit_button;
        unsafe {
            submit_button = node.query_selector_iter("input[type=submit]".to_owned()).unwrap()
                .filter_map(HTMLInputElementCast::to_root)
                .find(|r| r.r().form_owner() == owner);
        }
        match submit_button {
            Some(ref button) => {
                if button.r().is_instance_activatable() {
                    button.r().synthetic_click_activation(ctrlKey, shiftKey, altKey, metaKey)
                }
            }
            None => {
                unsafe {
                    // Safe because we don't perform any DOM modification
                    // until we're done with the iterator.
                    let inputs = node.query_selector_iter("input".to_owned()).unwrap()
                        .filter_map(HTMLInputElementCast::to_root)
                        .filter(|input| {
                            input.r().form_owner() == owner && match &*input.r().Type() {
                                "text" | "search" | "url" | "tel" |
                                "email" | "password" | "datetime" |
                                "date" | "month" | "week" | "time" |
                                "datetime-local" | "number"
                                  => true,
                                _ => false
                            }
                        });

                    if inputs.skip(1).next().is_some() {
                        // lazily test for > 1 submission-blocking inputs
                        return;
                    }
                }

                form.r().submit(SubmittedFrom::NotFromFormSubmitMethod,
                                FormSubmitter::FormElement(form.r()));
            }
        }
    }
}

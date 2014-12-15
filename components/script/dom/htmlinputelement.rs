/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::activation::Activatable;
use dom::attr::{Attr, AttrValue, UIntAttrValue};
use dom::attr::AttrHelpers;
use dom::bindings::cell::DOMRefCell;
use dom::bindings::codegen::Bindings::AttrBinding::AttrMethods;
use dom::bindings::codegen::Bindings::DocumentBinding::DocumentMethods;
use dom::bindings::codegen::Bindings::EventBinding::EventMethods;
use dom::bindings::codegen::Bindings::EventTargetBinding::EventTargetMethods;
use dom::bindings::codegen::Bindings::HTMLInputElementBinding;
use dom::bindings::codegen::Bindings::HTMLInputElementBinding::HTMLInputElementMethods;
use dom::bindings::codegen::InheritTypes::{ElementCast, HTMLElementCast, HTMLFormElementCast, HTMLInputElementCast, NodeCast};
use dom::bindings::codegen::InheritTypes::{HTMLInputElementDerived, HTMLFieldSetElementDerived, EventTargetCast};
use dom::bindings::codegen::InheritTypes::KeyboardEventCast;
use dom::bindings::global::Window;
use dom::bindings::js::{Comparable, JS, JSRef, Root, Temporary, OptionalRootable};
use dom::bindings::js::{ResultRootable, RootedReference, MutNullableJS};
use dom::bindings::utils::{Reflectable, Reflector};
use dom::document::{Document, DocumentHelpers};
use dom::element::{AttributeHandlers, Element, HTMLInputElementTypeId};
use dom::element::{RawLayoutElementHelpers, ActivationElementHelpers};
use dom::event::{Event, Bubbles, NotCancelable, EventHelpers};
use dom::eventtarget::{EventTarget, NodeTargetTypeId};
use dom::htmlelement::HTMLElement;
use dom::keyboardevent::KeyboardEvent;
use dom::htmlformelement::{InputElement, FormControl, HTMLFormElement, HTMLFormElementHelpers};
use dom::htmlformelement::{NotFromFormSubmitMethod};
use dom::node::{DisabledStateHelpers, Node, NodeHelpers, ElementNodeTypeId, OtherNodeDamage};
use dom::node::{document_from_node, window_from_node};
use dom::virtualmethods::VirtualMethods;
use textinput::{Single, TextInput, TriggerDefaultAction, DispatchInput, Nothing};

use servo_util::str::DOMString;
use string_cache::Atom;

use std::ascii::OwnedAsciiExt;
use std::cell::Cell;
use std::default::Default;

const DEFAULT_SUBMIT_VALUE: &'static str = "Submit";
const DEFAULT_RESET_VALUE: &'static str = "Reset";

#[jstraceable]
#[deriving(PartialEq)]
#[allow(dead_code)]
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
    indeterminate: Cell<bool>,
    size: Cell<u32>,
    textinput: DOMRefCell<TextInput>,
    activation_state: DOMRefCell<InputActivationState>,
}

#[jstraceable]
#[must_root]
struct InputActivationState {
    indeterminate: bool,
    checked: bool,
    checked_radio: MutNullableJS<HTMLInputElement>,
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
            checked_radio: Default::default(),
            was_mutable: false,
            old_type: InputText
        }
    }
}

impl HTMLInputElementDerived for EventTarget {
    fn is_htmlinputelement(&self) -> bool {
        *self.type_id() == NodeTargetTypeId(ElementNodeTypeId(HTMLInputElementTypeId))
    }
}

static DEFAULT_INPUT_SIZE: u32 = 20;

impl HTMLInputElement {
    fn new_inherited(localName: DOMString, prefix: Option<DOMString>, document: JSRef<Document>) -> HTMLInputElement {
        HTMLInputElement {
            htmlelement: HTMLElement::new_inherited(HTMLInputElementTypeId, localName, prefix, document),
            input_type: Cell::new(InputText),
            checked: Cell::new(false),
            indeterminate: Cell::new(false),
            size: Cell::new(DEFAULT_INPUT_SIZE),
            textinput: DOMRefCell::new(TextInput::new(Single, "".to_string())),
            activation_state: DOMRefCell::new(InputActivationState::new())
        }
    }

    #[allow(unrooted_must_root)]
    pub fn new(localName: DOMString, prefix: Option<DOMString>, document: JSRef<Document>) -> Temporary<HTMLInputElement> {
        let element = HTMLInputElement::new_inherited(localName, prefix, document);
        Node::reflect_node(box element, document, HTMLInputElementBinding::Wrap)
    }
}

pub trait LayoutHTMLInputElementHelpers {
    unsafe fn get_value_for_layout(self) -> String;
    unsafe fn get_size_for_layout(self) -> u32;
}

pub trait RawLayoutHTMLInputElementHelpers {
    unsafe fn get_checked_state_for_layout(&self) -> bool;
    unsafe fn get_size_for_layout(&self) -> u32;
}

impl LayoutHTMLInputElementHelpers for JS<HTMLInputElement> {
    #[allow(unrooted_must_root)]
    unsafe fn get_value_for_layout(self) -> String {
        unsafe fn get_raw_textinput_value(input: JS<HTMLInputElement>) -> String {
            (*input.unsafe_get()).textinput.borrow_for_layout().get_content()
        }

        unsafe fn get_raw_attr_value(input: JS<HTMLInputElement>) -> Option<String> {
            let elem: JS<Element> = input.transmute_copy();
            (*elem.unsafe_get()).get_attr_val_for_layout(&ns!(""), &atom!("value"))
                                .map(|s| s.to_string())
        }

        match (*self.unsafe_get()).input_type.get() {
            InputCheckbox | InputRadio => "".to_string(),
            InputFile | InputImage => "".to_string(),
            InputButton => get_raw_attr_value(self).unwrap_or_else(|| "".to_string()),
            InputSubmit => get_raw_attr_value(self).unwrap_or_else(|| DEFAULT_SUBMIT_VALUE.to_string()),
            InputReset => get_raw_attr_value(self).unwrap_or_else(|| DEFAULT_RESET_VALUE.to_string()),
            InputPassword => {
                let raw = get_raw_textinput_value(self);
                String::from_char(raw.char_len(), '●')
            }
            _ => get_raw_textinput_value(self),
        }
    }

    #[allow(unrooted_must_root)]
    unsafe fn get_size_for_layout(self) -> u32 {
        (*self.unsafe_get()).get_size_for_layout()
    }
}

impl RawLayoutHTMLInputElementHelpers for HTMLInputElement {
    #[allow(unrooted_must_root)]
    unsafe fn get_checked_state_for_layout(&self) -> bool {
        self.checked.get()
    }

    #[allow(unrooted_must_root)]
    unsafe fn get_size_for_layout(&self) -> u32 {
        self.size.get()
    }
}

impl<'a> HTMLInputElementMethods for JSRef<'a, HTMLInputElement> {
    // http://www.whatwg.org/html/#dom-fe-disabled
    make_bool_getter!(Disabled)

    // http://www.whatwg.org/html/#dom-fe-disabled
    make_bool_setter!(SetDisabled, "disabled")

    // https://html.spec.whatwg.org/multipage/forms.html#dom-input-checked
    fn Checked(self) -> bool {
        self.checked.get()
    }

    // https://html.spec.whatwg.org/multipage/forms.html#dom-input-checked
    fn SetChecked(self, checked: bool) {
        self.update_checked_state(checked);
    }

    // https://html.spec.whatwg.org/multipage/forms.html#dom-input-readonly
    make_bool_getter!(ReadOnly)

    // https://html.spec.whatwg.org/multipage/forms.html#dom-input-readonly
    make_bool_setter!(SetReadOnly, "readonly")

    // https://html.spec.whatwg.org/multipage/forms.html#dom-input-size
    make_uint_getter!(Size)

    // https://html.spec.whatwg.org/multipage/forms.html#dom-input-size
    make_uint_setter!(SetSize, "size")

    // https://html.spec.whatwg.org/multipage/forms.html#dom-input-type
    make_enumerated_getter!(Type, "text", "hidden" | "search" | "tel" |
                                  "url" | "email" | "password" |
                                  "datetime" | "date" | "month" |
                                  "week" | "time" | "datetime-local" |
                                  "number" | "range" | "color" |
                                  "checkbox" | "radio" | "file" |
                                  "submit" | "image" | "reset" | "button")

    // https://html.spec.whatwg.org/multipage/forms.html#dom-input-type
    make_setter!(SetType, "type")

    // https://html.spec.whatwg.org/multipage/forms.html#dom-input-value
    fn Value(self) -> DOMString {
        self.textinput.borrow().get_content()
    }

    // https://html.spec.whatwg.org/multipage/forms.html#dom-input-value
    make_setter!(SetValue, "value")

    // https://html.spec.whatwg.org/multipage/forms.html#attr-fe-name
    make_getter!(Name)

    // https://html.spec.whatwg.org/multipage/forms.html#attr-fe-name
    make_setter!(SetName, "name")

    // https://html.spec.whatwg.org/multipage/forms.html#dom-input-formaction
    make_url_or_base_getter!(FormAction)

    // https://html.spec.whatwg.org/multipage/forms.html#dom-input-formaction
    make_setter!(SetFormAction, "formaction")

    // https://html.spec.whatwg.org/multipage/forms.html#dom-input-formenctype
    make_enumerated_getter!(FormEnctype, "application/x-www-form-urlencoded", "text/plain" | "multipart/form-data")

    // https://html.spec.whatwg.org/multipage/forms.html#dom-input-formenctype
    make_setter!(SetFormEnctype, "formenctype")

    // https://html.spec.whatwg.org/multipage/forms.html#dom-input-formmethod
    make_enumerated_getter!(FormMethod, "get", "post" | "dialog")

    // https://html.spec.whatwg.org/multipage/forms.html#dom-input-formmethod
    make_setter!(SetFormMethod, "formmethod")

    // https://html.spec.whatwg.org/multipage/forms.html#dom-input-formtarget
    make_getter!(FormTarget)

    // https://html.spec.whatwg.org/multipage/forms.html#dom-input-formtarget
    make_setter!(SetFormTarget, "formtarget")

    // https://html.spec.whatwg.org/multipage/forms.html#dom-input-indeterminate
    fn Indeterminate(self) -> bool {
        self.indeterminate.get()
    }

    // https://html.spec.whatwg.org/multipage/forms.html#dom-input-indeterminate
    fn SetIndeterminate(self, val: bool) {
        // FIXME #4079 this should change the appearance
        self.indeterminate.set(val)
    }
}

pub trait HTMLInputElementHelpers {
    fn force_relayout(self);
    fn radio_group_updated(self, group: Option<&str>);
    fn get_radio_group_name(self) -> Option<String>;
    fn update_checked_state(self, checked: bool);
    fn get_size(&self) -> u32;
}

fn broadcast_radio_checked(broadcaster: JSRef<HTMLInputElement>, group: Option<&str>) {
    //TODO: if not in document, use root ancestor instead of document
    let owner = broadcaster.form_owner().root();
    let doc = document_from_node(broadcaster).root();
    let doc_node: JSRef<Node> = NodeCast::from_ref(*doc);

    // There is no DOM tree manipulation here, so this is safe
    let mut iter = unsafe {
        doc_node.query_selector_iter("input[type=radio]".to_string()).unwrap()
                .filter_map(|t| HTMLInputElementCast::to_ref(t))
                .filter(|&r| in_same_group(r, owner.root_ref(), group) && broadcaster != r)
    };
    for r in iter {
        if r.Checked() {
            r.SetChecked(false);
        }
    }
}

fn in_same_group<'a,'b>(other: JSRef<'a, HTMLInputElement>,
                        owner: Option<JSRef<'b, HTMLFormElement>>,
                        group: Option<&str>) -> bool {
    let other_owner = other.form_owner().root();
    let other_owner = other_owner.root_ref();
    other.input_type.get() == InputRadio &&
    // TODO Both a and b are in the same home subtree.
    other_owner.equals(owner) &&
    // TODO should be a unicode compatibility caseless match
    match (other.get_radio_group_name(), group) {
        (Some(ref s1), Some(s2)) => s1.as_slice() == s2,
        (None, None) => true,
        _ => false
    }
}

impl<'a> HTMLInputElementHelpers for JSRef<'a, HTMLInputElement> {
    fn force_relayout(self) {
        let doc = document_from_node(self).root();
        let node: JSRef<Node> = NodeCast::from_ref(self);
        doc.content_changed(node, OtherNodeDamage)
    }

    fn radio_group_updated(self, group: Option<&str>) {
        if self.Checked() {
            broadcast_radio_checked(self, group);
        }
    }

    fn get_radio_group_name(self) -> Option<String> {
        //TODO: determine form owner
        let elem: JSRef<Element> = ElementCast::from_ref(self);
        elem.get_attribute(ns!(""), &atom!("name"))
            .root()
            .map(|name| name.Value())
    }

    fn update_checked_state(self, checked: bool) {
        self.checked.set(checked);
        if self.input_type.get() == InputRadio && checked {
            broadcast_radio_checked(self,
                                    self.get_radio_group_name()
                                        .as_ref()
                                        .map(|group| group.as_slice()));
        }
        //TODO: dispatch change event
        self.force_relayout();
    }

    fn get_size(&self) -> u32 {
        self.size.get()
    }
}

impl<'a> VirtualMethods for JSRef<'a, HTMLInputElement> {
    fn super_type<'a>(&'a self) -> Option<&'a VirtualMethods> {
        let htmlelement: &JSRef<HTMLElement> = HTMLElementCast::from_borrowed_ref(self);
        Some(htmlelement as &VirtualMethods)
    }

    fn after_set_attr(&self, attr: JSRef<Attr>) {
        match self.super_type() {
            Some(ref s) => s.after_set_attr(attr),
            _ => ()
        }

        match attr.local_name() {
            &atom!("disabled") => {
                let node: JSRef<Node> = NodeCast::from_ref(*self);
                node.set_disabled_state(true);
                node.set_enabled_state(false);
            }
            &atom!("checked") => {
                self.update_checked_state(true);
            }
            &atom!("size") => {
                match *attr.value() {
                    UIntAttrValue(_, value) => self.size.set(value),
                    _ => panic!("Expected a UIntAttrValue"),
                }
                self.force_relayout();
            }
            &atom!("type") => {
                let value = attr.value();
                self.input_type.set(match value.as_slice() {
                    "button" => InputButton,
                    "submit" => InputSubmit,
                    "reset" => InputReset,
                    "file" => InputFile,
                    "radio" => InputRadio,
                    "checkbox" => InputCheckbox,
                    "password" => InputPassword,
                    _ => InputText,
                });
                if self.input_type.get() == InputRadio {
                    self.radio_group_updated(self.get_radio_group_name()
                                                 .as_ref()
                                                 .map(|group| group.as_slice()));
                }
                self.force_relayout();
            }
            &atom!("value") => {
                self.textinput.borrow_mut().set_content(attr.value().as_slice().to_string());
                self.force_relayout();
            }
            &atom!("name") => {
                if self.input_type.get() == InputRadio {
                    let value = attr.value();
                    self.radio_group_updated(Some(value.as_slice()));
                }
            }
            _ => ()
        }
    }

    fn before_remove_attr(&self, attr: JSRef<Attr>) {
        match self.super_type() {
            Some(ref s) => s.before_remove_attr(attr),
            _ => ()
        }

        match attr.local_name() {
            &atom!("disabled") => {
                let node: JSRef<Node> = NodeCast::from_ref(*self);
                node.set_disabled_state(false);
                node.set_enabled_state(true);
                node.check_ancestors_disabled_state_for_form_control();
            }
            &atom!("checked") => {
                self.update_checked_state(false);
            }
            &atom!("size") => {
                self.size.set(DEFAULT_INPUT_SIZE);
                self.force_relayout();
            }
            &atom!("type") => {
                if self.input_type.get() == InputRadio {
                    broadcast_radio_checked(*self,
                                            self.get_radio_group_name()
                                                .as_ref()
                                                .map(|group| group.as_slice()));
                }
                self.input_type.set(InputText);
                self.force_relayout();
            }
            &atom!("value") => {
                self.textinput.borrow_mut().set_content("".to_string());
                self.force_relayout();
            }
            &atom!("name") => {
                if self.input_type.get() == InputRadio {
                    self.radio_group_updated(None);
                }
            }
            _ => ()
        }
    }

    fn parse_plain_attribute(&self, name: &Atom, value: DOMString) -> AttrValue {
        match name {
            &atom!("size") => AttrValue::from_u32(value, DEFAULT_INPUT_SIZE),
            _ => self.super_type().unwrap().parse_plain_attribute(name, value),
        }
    }

    fn bind_to_tree(&self, tree_in_doc: bool) {
        match self.super_type() {
            Some(ref s) => s.bind_to_tree(tree_in_doc),
            _ => (),
        }

        let node: JSRef<Node> = NodeCast::from_ref(*self);
        node.check_ancestors_disabled_state_for_form_control();
    }

    fn unbind_from_tree(&self, tree_in_doc: bool) {
        match self.super_type() {
            Some(ref s) => s.unbind_from_tree(tree_in_doc),
            _ => (),
        }

        let node: JSRef<Node> = NodeCast::from_ref(*self);
        if node.ancestors().any(|ancestor| ancestor.is_htmlfieldsetelement()) {
            node.check_ancestors_disabled_state_for_form_control();
        } else {
            node.check_disabled_attribute();
        }
    }

    fn handle_event(&self, event: JSRef<Event>) {
        match self.super_type() {
            Some(s) => {
                s.handle_event(event);
            }
            _ => (),
        }

        if "click" == event.Type().as_slice() && !event.DefaultPrevented() {
            match self.input_type.get() {
                InputRadio => self.SetChecked(true),
                _ => {}
            }

            // TODO: Dispatch events for non activatable inputs
            // https://html.spec.whatwg.org/multipage/forms.html#common-input-element-events

            //TODO: set the editing position for text inputs

            let doc = document_from_node(*self).root();
            doc.request_focus(ElementCast::from_ref(*self));
        } else if "keydown" == event.Type().as_slice() && !event.DefaultPrevented() &&
            (self.input_type.get() == InputText || self.input_type.get() == InputPassword) {
                let keyevent: Option<JSRef<KeyboardEvent>> = KeyboardEventCast::to_ref(event);
                keyevent.map(|keyevent| {
                    match self.textinput.borrow_mut().handle_keydown(keyevent) {
                        TriggerDefaultAction => (),
                        DispatchInput => {
                            self.force_relayout();
                            event.PreventDefault();
                        }
                        Nothing => (),
                    }
                });
        }
    }
}

impl Reflectable for HTMLInputElement {
    fn reflector<'a>(&'a self) -> &'a Reflector {
        self.htmlelement.reflector()
    }
}

impl<'a> FormControl<'a> for JSRef<'a, HTMLInputElement> {
    // FIXME: This is wrong (https://github.com/servo/servo/issues/3553)
    //        but we need html5ever to do it correctly
    fn form_owner(self) -> Option<Temporary<HTMLFormElement>> {
        // https://html.spec.whatwg.org/multipage/forms.html#reset-the-form-owner
        let elem: JSRef<Element> = ElementCast::from_ref(self);
        let owner = elem.get_string_attribute(&atom!("form"));
        if !owner.is_empty() {
            let doc = document_from_node(self).root();
            let owner = doc.GetElementById(owner).root();
            match owner {
                Some(o) => {
                    let maybe_form: Option<JSRef<HTMLFormElement>> = HTMLFormElementCast::to_ref(*o);
                    if maybe_form.is_some() {
                        return maybe_form.map(Temporary::from_rooted);
                    }
                },
                _ => ()
            }
        }
        let node: JSRef<Node> = NodeCast::from_ref(self);
        node.ancestors().filter_map(|a| HTMLFormElementCast::to_ref(a)).next()
            .map(Temporary::from_rooted)
    }

    fn to_element(self) -> JSRef<'a, Element> {
        ElementCast::from_ref(self)
    }

    // https://html.spec.whatwg.org/multipage/forms.html#concept-fe-mutable
    fn mutable(self) -> bool {
        // https://html.spec.whatwg.org/multipage/forms.html#the-input-element:concept-fe-mutable
        // https://html.spec.whatwg.org/multipage/forms.html#the-readonly-attribute:concept-fe-mutable
        !(self.Disabled() || self.ReadOnly())
    }
}


impl<'a> Activatable for JSRef<'a, HTMLInputElement> {
    fn as_element(&self) -> Temporary<Element> {
        Temporary::from_rooted(ElementCast::from_ref(*self))
    }

    // https://html.spec.whatwg.org/multipage/interaction.html#run-pre-click-activation-steps
    fn pre_click_activation(&self) {
        let mut cache = self.activation_state.borrow_mut();
        let ty = self.input_type.get();
        cache.old_type = ty;
        cache.was_mutable = self.mutable();
        if cache.was_mutable {
            match ty {
                // https://html.spec.whatwg.org/multipage/forms.html#submit-button-state-(type=submit):activation-behavior
                // InputSubmit => (), // No behavior defined
                InputCheckbox => {
                    // https://html.spec.whatwg.org/multipage/forms.html#checkbox-state-(type=checkbox):pre-click-activation-steps
                    // cache current values of `checked` and `indeterminate`
                    // we may need to restore them later
                    cache.indeterminate = self.Indeterminate();
                    cache.checked = self.Checked();
                    self.SetIndeterminate(false);
                    self.SetChecked(!cache.checked);
                },
                // https://html.spec.whatwg.org/multipage/forms.html#radio-button-state-(type=radio):pre-click-activation-steps
                InputRadio => {
                    //TODO: if not in document, use root ancestor instead of document
                    let owner = self.form_owner().root();
                    let doc = document_from_node(*self).root();
                    let doc_node: JSRef<Node> = NodeCast::from_ref(*doc);
                    let group = self.get_radio_group_name();;

                    // Safe since we only manipulate the DOM tree after finding an element
                    let checked_member = unsafe {
                        doc_node.query_selector_iter("input[type=radio]".to_string()).unwrap()
                                .filter_map(|t| HTMLInputElementCast::to_ref(t))
                                .filter(|&r| in_same_group(r, owner.root_ref(),
                                                           group.as_ref().map(|gr| gr.as_slice())))
                                .find(|r| r.Checked())
                    };
                    cache.checked_radio.assign(checked_member);
                    self.SetChecked(true);
                }
                _ => ()
            }
        }
    }

    // https://html.spec.whatwg.org/multipage/interaction.html#run-canceled-activation-steps
    fn canceled_activation(&self) {
        let cache = self.activation_state.borrow();
        let ty = self.input_type.get();
        if cache.old_type != ty  {
            // Type changed, abandon ship
            // https://www.w3.org/Bugs/Public/show_bug.cgi?id=27414
            return;
        }
        match ty {
            // https://html.spec.whatwg.org/multipage/forms.html#submit-button-state-(type=submit):activation-behavior
            // InputSubmit => (), // No behavior defined
            // https://html.spec.whatwg.org/multipage/forms.html#checkbox-state-(type=checkbox):canceled-activation-steps
            InputCheckbox => {
                // We want to restore state only if the element had been changed in the first place
                if cache.was_mutable {
                    self.SetIndeterminate(cache.indeterminate);
                    self.SetChecked(cache.checked);
                }
            },
            // https://html.spec.whatwg.org/multipage/forms.html#radio-button-state-(type=radio):canceled-activation-steps
            InputRadio => {
                // We want to restore state only if the element had been changed in the first place
                if cache.was_mutable {
                    let old_checked: Option<Root<HTMLInputElement>> = cache.checked_radio.get().root();
                    let name = self.get_radio_group_name();
                    match old_checked {
                        Some(o) => {
                            // Avoiding iterating through the whole tree here, instead
                            // we can check if the conditions for radio group siblings apply
                            if name == o.get_radio_group_name() && // TODO should be compatibility caseless
                               self.form_owner() == o.form_owner() &&
                               // TODO Both a and b are in the same home subtree
                               o.input_type.get() == InputRadio {
                                    o.SetChecked(true);
                            } else {
                                self.SetChecked(false);
                            }
                        },
                        None => self.SetChecked(false)
                    };
                }
            }
            _ => ()
        }
    }

    // https://html.spec.whatwg.org/multipage/interaction.html#run-post-click-activation-steps
    fn activation_behavior(&self) {
        let ty = self.input_type.get();
        if self.activation_state.borrow().old_type != ty {
            // Type changed, abandon ship
            // https://www.w3.org/Bugs/Public/show_bug.cgi?id=27414
            return;
        }
        match ty {
            InputSubmit => {
                // https://html.spec.whatwg.org/multipage/forms.html#submit-button-state-(type=submit):activation-behavior
                // FIXME (Manishearth): support document owners (needs ability to get parent browsing context)
                if self.mutable() /* and document owner is fully active */ {
                    self.form_owner().map(|o| {
                        o.root().submit(NotFromFormSubmitMethod, InputElement(self.clone()))
                    });
                }
            },
            InputCheckbox | InputRadio => {
                // https://html.spec.whatwg.org/multipage/forms.html#checkbox-state-(type=checkbox):activation-behavior
                // https://html.spec.whatwg.org/multipage/forms.html#radio-button-state-(type=radio):activation-behavior
                if self.mutable() {
                    let win = window_from_node(*self).root();
                    let event = Event::new(Window(*win),
                                           "input".to_string(),
                                           Bubbles, NotCancelable).root();
                    event.set_trusted(true);
                    let target: JSRef<EventTarget> = EventTargetCast::from_ref(*self);
                    target.DispatchEvent(*event).ok();

                    let event = Event::new(Window(*win),
                                           "change".to_string(),
                                           Bubbles, NotCancelable).root();
                    event.set_trusted(true);
                    let target: JSRef<EventTarget> = EventTargetCast::from_ref(*self);
                    target.DispatchEvent(*event).ok();
                }
            },
            _ => ()
        }
    }

    // https://html.spec.whatwg.org/multipage/forms.html#implicit-submission
    fn implicit_submission(&self, ctrlKey: bool, shiftKey: bool, altKey: bool, metaKey: bool) {
        let doc = document_from_node(*self).root();
        let node: JSRef<Node> = NodeCast::from_ref(*doc);
        let owner = self.form_owner();
        if owner.is_none() || ElementCast::from_ref(*self).click_in_progress() {
            return;
        }
        // This is safe because we are stopping after finding the first element
        // and only then performing actions which may modify the DOM tree
        unsafe {
            node.query_selector_iter("input[type=submit]".to_string()).unwrap()
                .filter_map(|t| HTMLInputElementCast::to_ref(t))
                .find(|r| r.form_owner() == owner)
                .map(|s| s.synthetic_click_activation(ctrlKey, shiftKey, altKey, metaKey));
        }
    }
}

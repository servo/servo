/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::attr::{Attr, AttrValue, UIntAttrValue};
use dom::attr::AttrHelpers;
use dom::bindings::cell::DOMRefCell;
use dom::bindings::codegen::Bindings::AttrBinding::AttrMethods;
use dom::bindings::codegen::Bindings::DocumentBinding::DocumentMethods;
use dom::bindings::codegen::Bindings::EventBinding::EventMethods;
use dom::bindings::codegen::Bindings::HTMLInputElementBinding;
use dom::bindings::codegen::Bindings::HTMLInputElementBinding::HTMLInputElementMethods;
use dom::bindings::codegen::Bindings::NodeListBinding::NodeListMethods;
use dom::bindings::codegen::InheritTypes::{ElementCast, HTMLElementCast, HTMLFormElementCast, HTMLInputElementCast, NodeCast};
use dom::bindings::codegen::InheritTypes::{HTMLInputElementDerived, HTMLFieldSetElementDerived};
use dom::bindings::codegen::InheritTypes::KeyboardEventCast;
use dom::bindings::js::{JS, JSRef, Temporary, OptionalRootable, ResultRootable};
use dom::bindings::utils::{Reflectable, Reflector};
use dom::document::{Document, DocumentHelpers};
use dom::element::{AttributeHandlers, Element, HTMLInputElementTypeId};
use dom::element::RawLayoutElementHelpers;
use dom::event::Event;
use dom::eventtarget::{EventTarget, NodeTargetTypeId};
use dom::htmlelement::HTMLElement;
use dom::keyboardevent::KeyboardEvent;
use dom::htmlformelement::{InputElement, FormOwner, HTMLFormElement, HTMLFormElementHelpers, NotFromFormSubmitMethod};
use dom::node::{DisabledStateHelpers, Node, NodeHelpers, ElementNodeTypeId, document_from_node, window_from_node};
use dom::virtualmethods::VirtualMethods;
use textinput::{Single, TextInput, TriggerDefaultAction, DispatchInput, Nothing};

use servo_util::str::DOMString;
use string_cache::Atom;

use std::ascii::OwnedAsciiExt;
use std::cell::Cell;

const DEFAULT_SUBMIT_VALUE: &'static str = "Submit";
const DEFAULT_RESET_VALUE: &'static str = "Reset";

#[jstraceable]
#[deriving(PartialEq)]
#[allow(dead_code)]
enum InputType {
    InputButton(Option<&'static str>),
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
    size: Cell<u32>,
    textinput: DOMRefCell<TextInput>,
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
            size: Cell::new(DEFAULT_INPUT_SIZE),
            textinput: DOMRefCell::new(TextInput::new(Single, "".to_string())),
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
            InputButton(ref default) => get_raw_attr_value(self)
                                          .or_else(|| default.map(|v| v.to_string()))
                                          .unwrap_or_else(|| "".to_string()),
            InputPassword => {
                let raw = get_raw_textinput_value(self);
                String::from_char(raw.len(), '●')
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
    make_bool_setter!(SetChecked, "checked")

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
}

pub trait HTMLInputElementHelpers {
    fn force_relayout(self);
    fn radio_group_updated(self, group: Option<&str>);
    fn get_radio_group(self) -> Option<String>;
    fn update_checked_state(self, checked: bool);
    fn get_size(&self) -> u32;
}

fn broadcast_radio_checked(broadcaster: JSRef<HTMLInputElement>, group: Option<&str>) {
    //TODO: if not in document, use root ancestor instead of document
    let doc = document_from_node(broadcaster).root();
    let radios = doc.QuerySelectorAll("input[type=\"radio\"]".to_string()).unwrap().root();
    let mut i = 0;
    while i < radios.Length() {
        let node = radios.Item(i).unwrap().root();
        let radio: JSRef<HTMLInputElement> = HTMLInputElementCast::to_ref(*node).unwrap();
        if radio != broadcaster {
            //TODO: determine form owner
            let other_group = radio.get_radio_group();
            //TODO: ensure compatibility caseless match (https://html.spec.whatwg.org/multipage/infrastructure.html#compatibility-caseless)
            let group_matches = other_group.as_ref().map(|group| group.as_slice()) == group.as_ref().map(|&group| &*group);
            if group_matches && radio.Checked() {
                radio.SetChecked(false);
            }
        }
        i += 1;
    }
}

impl<'a> HTMLInputElementHelpers for JSRef<'a, HTMLInputElement> {
    fn force_relayout(self) {
        let doc = document_from_node(self).root();
        let node: JSRef<Node> = NodeCast::from_ref(self);
        doc.content_changed(node)
    }

    fn radio_group_updated(self, group: Option<&str>) {
        if self.Checked() {
            broadcast_radio_checked(self, group);
        }
    }

    // https://html.spec.whatwg.org/multipage/forms.html#radio-button-group
    fn get_radio_group(self) -> Option<String> {
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
                                    self.get_radio_group()
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
                    "button" => InputButton(None),
                    "submit" => InputButton(Some(DEFAULT_SUBMIT_VALUE)),
                    "reset" => InputButton(Some(DEFAULT_RESET_VALUE)),
                    "file" => InputFile,
                    "radio" => InputRadio,
                    "checkbox" => InputCheckbox,
                    "password" => InputPassword,
                    _ => InputText,
                });
                if self.input_type.get() == InputRadio {
                    self.radio_group_updated(self.get_radio_group()
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
                                            self.get_radio_group()
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
                InputCheckbox => self.SetChecked(!self.checked.get()),
                InputRadio => self.SetChecked(true),
                InputButton(Some(DEFAULT_SUBMIT_VALUE)) => {
                    self.form_owner().map(|o| {
                        o.root().submit(NotFromFormSubmitMethod, InputElement(self.clone()))
                    });
                }
                _ => {}
            }

            //TODO: set the editing position for text inputs

            let doc = document_from_node(*self).root();
            doc.request_focus(ElementCast::from_ref(*self));
        } else if "keydown" == event.Type().as_slice() && !event.DefaultPrevented() &&
            (self.input_type.get() == InputText || self.input_type.get() == InputPassword) {
                let keyevent: Option<JSRef<KeyboardEvent>> = KeyboardEventCast::to_ref(event);
                keyevent.map(|event| {
                    match self.textinput.borrow_mut().handle_keydown(event) {
                        TriggerDefaultAction => (),
                        DispatchInput => {
                            self.force_relayout();
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

impl<'a> FormOwner<'a> for JSRef<'a, HTMLInputElement> {
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
}

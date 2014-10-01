/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::AttrBinding::AttrMethods;
use dom::bindings::codegen::Bindings::DocumentBinding::DocumentMethods;
use dom::bindings::codegen::Bindings::EventBinding::EventMethods;
use dom::bindings::codegen::Bindings::HTMLInputElementBinding;
use dom::bindings::codegen::Bindings::HTMLInputElementBinding::HTMLInputElementMethods;
use dom::bindings::codegen::Bindings::NodeListBinding::NodeListMethods;
use dom::bindings::codegen::InheritTypes::{ElementCast, HTMLElementCast, HTMLInputElementCast, NodeCast};
use dom::bindings::codegen::InheritTypes::{HTMLInputElementDerived, HTMLFieldSetElementDerived};
use dom::bindings::js::{JS, JSRef, Temporary, OptionalRootable, ResultRootable};
use dom::bindings::utils::{Reflectable, Reflector};
use dom::attr::{AttrHelpers};
use dom::document::{Document, DocumentHelpers};
use dom::element::{AttributeHandlers, Element, HTMLInputElementTypeId};
use dom::event::Event;
use dom::eventtarget::{EventTarget, NodeTargetTypeId};
use dom::htmlelement::HTMLElement;
use dom::node::{DisabledStateHelpers, Node, NodeHelpers, ElementNodeTypeId, document_from_node};
use dom::virtualmethods::VirtualMethods;

use servo_util::str::{DOMString, parse_unsigned_integer};
use string_cache::Atom;

use std::cell::{Cell, RefCell};
use std::mem;

static DEFAULT_SUBMIT_VALUE: &'static str = "Submit";
static DEFAULT_RESET_VALUE: &'static str = "Reset";

#[jstraceable]
#[deriving(PartialEq)]
enum InputType {
    InputButton(Option<&'static str>),
    InputText,
    InputFile,
    InputImage,
    InputCheckbox,
    InputRadio,
    InputPassword
}

#[jstraceable]
#[must_root]
pub struct HTMLInputElement {
    pub htmlelement: HTMLElement,
    input_type: Cell<InputType>,
    checked: Cell<bool>,
    uncommitted_value: RefCell<Option<String>>,
    value: RefCell<Option<String>>,
    size: Cell<u32>,
}

impl HTMLInputElementDerived for EventTarget {
    fn is_htmlinputelement(&self) -> bool {
        self.type_id == NodeTargetTypeId(ElementNodeTypeId(HTMLInputElementTypeId))
    }
}

static DEFAULT_INPUT_SIZE: u32 = 20;

impl HTMLInputElement {
    fn new_inherited(localName: DOMString, document: JSRef<Document>) -> HTMLInputElement {
        HTMLInputElement {
            htmlelement: HTMLElement::new_inherited(HTMLInputElementTypeId, localName, document),
            input_type: Cell::new(InputText),
            checked: Cell::new(false),
            uncommitted_value: RefCell::new(None),
            value: RefCell::new(None),
            size: Cell::new(DEFAULT_INPUT_SIZE),
        }
    }

    #[allow(unrooted_must_root)]
    pub fn new(localName: DOMString, document: JSRef<Document>) -> Temporary<HTMLInputElement> {
        let element = HTMLInputElement::new_inherited(localName, document);
        Node::reflect_node(box element, document, HTMLInputElementBinding::Wrap)
    }
}

pub trait LayoutHTMLInputElementHelpers {
    unsafe fn get_value_for_layout(&self) -> String;
    unsafe fn get_size_for_layout(&self) -> u32;
}

impl LayoutHTMLInputElementHelpers for JS<HTMLInputElement> {
    #[allow(unrooted_must_root)]
    unsafe fn get_value_for_layout(&self) -> String {
        unsafe fn get_raw_value(input: &JS<HTMLInputElement>) -> Option<String> {
            mem::transmute::<&RefCell<Option<String>>, &Option<String>>(&(*input.unsafe_get()).value).clone()
        }

        match (*self.unsafe_get()).input_type.get() {
            InputCheckbox | InputRadio => "".to_string(),
            InputFile | InputImage => "".to_string(),
            InputButton(ref default) => get_raw_value(self)
                                          .or_else(|| default.map(|v| v.to_string()))
                                          .unwrap_or_else(|| "".to_string()),
            InputPassword => {
                let raw = get_raw_value(self).unwrap_or_else(|| "".to_string());
                String::from_char(raw.len(), '*')
            }
            _ => get_raw_value(self).unwrap_or_else(|| "".to_string()),
        }
    }

    #[allow(unrooted_must_root)]
    unsafe fn get_size_for_layout(&self) -> u32 {
        (*self.unsafe_get()).size.get()
    }
}

impl<'a> HTMLInputElementMethods for JSRef<'a, HTMLInputElement> {
    // http://www.whatwg.org/html/#dom-fe-disabled
    make_bool_getter!(Disabled)

    // http://www.whatwg.org/html/#dom-fe-disabled
    fn SetDisabled(self, disabled: bool) {
        let elem: JSRef<Element> = ElementCast::from_ref(self);
        elem.set_bool_attribute("disabled", disabled)
    }

    // https://html.spec.whatwg.org/multipage/forms.html#dom-input-checked
    make_bool_getter!(Checked)

    // https://html.spec.whatwg.org/multipage/forms.html#dom-input-checked
    fn SetChecked(self, checked: bool) {
        let elem: JSRef<Element> = ElementCast::from_ref(self);
        elem.set_bool_attribute("checked", checked)
    }

    // https://html.spec.whatwg.org/multipage/forms.html#dom-input-size
    make_uint_getter!(Size)

    // https://html.spec.whatwg.org/multipage/forms.html#dom-input-size
    fn SetSize(self, size: u32) {
        let elem: JSRef<Element> = ElementCast::from_ref(self);
        elem.set_uint_attribute("size", size)
    }

    // https://html.spec.whatwg.org/multipage/forms.html#dom-input-value
    make_getter!(Value)

    // https://html.spec.whatwg.org/multipage/forms.html#dom-input-value
    fn SetValue(self, value: DOMString) {
        let elem: JSRef<Element> = ElementCast::from_ref(self);
        elem.set_string_attribute("value", value)
    }

    // https://html.spec.whatwg.org/multipage/forms.html#attr-fe-name
    make_getter!(Name)

    // https://html.spec.whatwg.org/multipage/forms.html#attr-fe-name
    fn SetName(self, name: DOMString) {
        let elem: JSRef<Element> = ElementCast::from_ref(self);
        elem.set_string_attribute("name", name)
    }
}

trait HTMLInputElementHelpers {
    fn force_relayout(self);
    fn radio_group_updated(self, group: Option<&str>);
    fn get_radio_group(self) -> Option<String>;
    fn update_checked_state(self, checked: bool);
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
        doc.content_changed()
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
        elem.get_attribute(ns!(""), "name")
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
}

impl<'a> VirtualMethods for JSRef<'a, HTMLInputElement> {
    fn super_type<'a>(&'a self) -> Option<&'a VirtualMethods> {
        let htmlelement: &JSRef<HTMLElement> = HTMLElementCast::from_borrowed_ref(self);
        Some(htmlelement as &VirtualMethods)
    }

    fn after_set_attr(&self, name: &Atom, value: DOMString) {
        match self.super_type() {
            Some(ref s) => s.after_set_attr(name, value.clone()),
            _ => (),
        }

        let node: JSRef<Node> = NodeCast::from_ref(*self);
        match name.as_slice() {
            "disabled" => {
                node.set_disabled_state(true);
                node.set_enabled_state(false);
            }
            "checked" => {
                self.update_checked_state(true);
            }
            "size" => {
                let parsed = parse_unsigned_integer(value.as_slice().chars());
                self.size.set(parsed.unwrap_or(DEFAULT_INPUT_SIZE));
                self.force_relayout();
            }
            "type" => {
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
            "value" => {
                *self.value.borrow_mut() = Some(value);
                self.force_relayout();
            }
            "name" => {
                if self.input_type.get() == InputRadio {
                    self.radio_group_updated(Some(value.as_slice()));
                }
            }
            _ => ()
        }
    }

    fn before_remove_attr(&self, name: &Atom, value: DOMString) {
        match self.super_type() {
            Some(ref s) => s.before_remove_attr(name, value),
            _ => (),
        }

        let node: JSRef<Node> = NodeCast::from_ref(*self);
        match name.as_slice() {
            "disabled" => {
                node.set_disabled_state(false);
                node.set_enabled_state(true);
                node.check_ancestors_disabled_state_for_form_control();
            }
            "checked" => {
                self.update_checked_state(false);
            }
            "size" => {
                self.size.set(DEFAULT_INPUT_SIZE);
                self.force_relayout();
            }
            "type" => {
                if self.input_type.get() == InputRadio {
                    broadcast_radio_checked(*self,
                                            self.get_radio_group()
                                                .as_ref()
                                                .map(|group| group.as_slice()));
                }
                self.input_type.set(InputText);
                self.force_relayout();
            }
            "value" => {
                *self.value.borrow_mut() = None;
                self.force_relayout();
            }
            "name" => {
                if self.input_type.get() == InputRadio {
                    self.radio_group_updated(None);
                }
            }
            _ => ()
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
                _ => {}
            }
        }
    }
}

impl Reflectable for HTMLInputElement {
    fn reflector<'a>(&'a self) -> &'a Reflector {
        self.htmlelement.reflector()
    }
}

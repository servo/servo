/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::attr::Attr;
use dom::bindings::cell::DOMRefCell;
use dom::bindings::codegen::Bindings::EventBinding::EventMethods;
use dom::bindings::codegen::Bindings::HTMLTextAreaElementBinding;
use dom::bindings::codegen::Bindings::HTMLTextAreaElementBinding::HTMLTextAreaElementMethods;
use dom::bindings::codegen::Bindings::NodeBinding::NodeMethods;
use dom::bindings::inheritance::Castable;
use dom::bindings::js::{LayoutJS, Root};
use dom::bindings::str::DOMString;
use dom::document::Document;
use dom::element::{AttributeMutation, Element};
use dom::element::RawLayoutElementHelpers;
use dom::event::{Event, EventBubbles, EventCancelable};
use dom::globalscope::GlobalScope;
use dom::htmlelement::HTMLElement;
use dom::htmlfieldsetelement::HTMLFieldSetElement;
use dom::htmlformelement::{FormControl, HTMLFormElement};
use dom::keyboardevent::KeyboardEvent;
use dom::node::{ChildrenMutation, Node, NodeDamage, UnbindContext};
use dom::node::{document_from_node, window_from_node};
use dom::nodelist::NodeList;
use dom::validation::Validatable;
use dom::virtualmethods::VirtualMethods;
use ipc_channel::ipc::IpcSender;
use script_traits::ScriptMsg as ConstellationMsg;
use std::cell::Cell;
use std::ops::Range;
use string_cache::Atom;
use style::attr::AttrValue;
use style::element_state::*;
use textinput::{KeyReaction, Lines, SelectionDirection, TextInput};

#[dom_struct]
pub struct HTMLTextAreaElement {
    htmlelement: HTMLElement,
    #[ignore_heap_size_of = "#7193"]
    textinput: DOMRefCell<TextInput<IpcSender<ConstellationMsg>>>,
    // https://html.spec.whatwg.org/multipage/#concept-textarea-dirty
    value_changed: Cell<bool>,
}

pub trait LayoutHTMLTextAreaElementHelpers {
    #[allow(unsafe_code)]
    unsafe fn get_value_for_layout(self) -> String;
    #[allow(unsafe_code)]
    unsafe fn selection_for_layout(self) -> Option<Range<usize>>;
    #[allow(unsafe_code)]
    fn get_cols(self) -> u32;
    #[allow(unsafe_code)]
    fn get_rows(self) -> u32;
}

impl LayoutHTMLTextAreaElementHelpers for LayoutJS<HTMLTextAreaElement> {
    #[allow(unrooted_must_root)]
    #[allow(unsafe_code)]
    unsafe fn get_value_for_layout(self) -> String {
        String::from((*self.unsafe_get()).textinput.borrow_for_layout().get_content())
    }

    #[allow(unrooted_must_root)]
    #[allow(unsafe_code)]
    unsafe fn selection_for_layout(self) -> Option<Range<usize>> {
        if !(*self.unsafe_get()).upcast::<Element>().focus_state() {
            return None;
        }
        let textinput = (*self.unsafe_get()).textinput.borrow_for_layout();
        Some(textinput.get_absolute_selection_range())
    }

    #[allow(unsafe_code)]
    fn get_cols(self) -> u32 {
        unsafe {
            (*self.upcast::<Element>().unsafe_get())
                .get_attr_for_layout(&ns!(), &atom!("cols"))
                .map_or(DEFAULT_COLS, AttrValue::as_uint)
        }
    }

    #[allow(unsafe_code)]
    fn get_rows(self) -> u32 {
        unsafe {
            (*self.upcast::<Element>().unsafe_get())
                .get_attr_for_layout(&ns!(), &atom!("rows"))
                .map_or(DEFAULT_ROWS, AttrValue::as_uint)
        }
    }
}

// https://html.spec.whatwg.org/multipage/#attr-textarea-cols-value
static DEFAULT_COLS: u32 = 20;

// https://html.spec.whatwg.org/multipage/#attr-textarea-rows-value
static DEFAULT_ROWS: u32 = 2;

impl HTMLTextAreaElement {
    fn new_inherited(local_name: Atom,
                     prefix: Option<DOMString>,
                     document: &Document) -> HTMLTextAreaElement {
        let chan = document.window().upcast::<GlobalScope>().constellation_chan().clone();
        HTMLTextAreaElement {
            htmlelement:
                HTMLElement::new_inherited_with_state(IN_ENABLED_STATE | IN_READ_WRITE_STATE,
                                                      local_name, prefix, document),
            textinput: DOMRefCell::new(TextInput::new(
                    Lines::Multiple, DOMString::new(), chan, None, None, SelectionDirection::None)),
            value_changed: Cell::new(false),
        }
    }

    #[allow(unrooted_must_root)]
    pub fn new(local_name: Atom,
               prefix: Option<DOMString>,
               document: &Document) -> Root<HTMLTextAreaElement> {
        Node::reflect_node(box HTMLTextAreaElement::new_inherited(local_name, prefix, document),
                           document,
                           HTMLTextAreaElementBinding::Wrap)
    }
}

impl HTMLTextAreaElementMethods for HTMLTextAreaElement {
    // TODO A few of these attributes have default values and additional
    // constraints

    // https://html.spec.whatwg.org/multipage/#dom-textarea-cols
    make_uint_getter!(Cols, "cols", DEFAULT_COLS);

    // https://html.spec.whatwg.org/multipage/#dom-textarea-cols
    make_limited_uint_setter!(SetCols, "cols", DEFAULT_COLS);

    // https://html.spec.whatwg.org/multipage/#dom-fe-disabled
    make_bool_getter!(Disabled, "disabled");

    // https://html.spec.whatwg.org/multipage/#dom-fe-disabled
    make_bool_setter!(SetDisabled, "disabled");

    // https://html.spec.whatwg.org/multipage/#dom-fae-form
    fn GetForm(&self) -> Option<Root<HTMLFormElement>> {
        self.form_owner()
    }

    // https://html.spec.whatwg.org/multipage/#attr-fe-name
    make_getter!(Name, "name");

    // https://html.spec.whatwg.org/multipage/#attr-fe-name
    make_setter!(SetName, "name");

    // https://html.spec.whatwg.org/multipage/#dom-textarea-placeholder
    make_getter!(Placeholder, "placeholder");

    // https://html.spec.whatwg.org/multipage/#dom-textarea-placeholder
    make_setter!(SetPlaceholder, "placeholder");

    // https://html.spec.whatwg.org/multipage/#attr-textarea-readonly
    make_bool_getter!(ReadOnly, "readonly");

    // https://html.spec.whatwg.org/multipage/#attr-textarea-readonly
    make_bool_setter!(SetReadOnly, "readonly");

    // https://html.spec.whatwg.org/multipage/#dom-textarea-required
    make_bool_getter!(Required, "required");

    // https://html.spec.whatwg.org/multipage/#dom-textarea-required
    make_bool_setter!(SetRequired, "required");

    // https://html.spec.whatwg.org/multipage/#dom-textarea-rows
    make_uint_getter!(Rows, "rows", DEFAULT_ROWS);

    // https://html.spec.whatwg.org/multipage/#dom-textarea-rows
    make_limited_uint_setter!(SetRows, "rows", DEFAULT_ROWS);

    // https://html.spec.whatwg.org/multipage/#dom-textarea-wrap
    make_getter!(Wrap, "wrap");

    // https://html.spec.whatwg.org/multipage/#dom-textarea-wrap
    make_setter!(SetWrap, "wrap");

    // https://html.spec.whatwg.org/multipage/#dom-textarea-type
    fn Type(&self) -> DOMString {
        DOMString::from("textarea")
    }

    // https://html.spec.whatwg.org/multipage/#dom-textarea-defaultvalue
    fn DefaultValue(&self) -> DOMString {
        self.upcast::<Node>().GetTextContent().unwrap()
    }

    // https://html.spec.whatwg.org/multipage/#dom-textarea-defaultvalue
    fn SetDefaultValue(&self, value: DOMString) {
        self.upcast::<Node>().SetTextContent(Some(value));

        // if the element's dirty value flag is false, then the element's
        // raw value must be set to the value of the element's textContent IDL attribute
        if !self.value_changed.get() {
            self.reset();
        }
    }

    // https://html.spec.whatwg.org/multipage/#dom-textarea-value
    fn Value(&self) -> DOMString {
        self.textinput.borrow().get_content()
    }

    // https://html.spec.whatwg.org/multipage/#dom-textarea-value
    fn SetValue(&self, value: DOMString) {
        // TODO move the cursor to the end of the field
        self.textinput.borrow_mut().set_content(value);
        self.value_changed.set(true);

        self.upcast::<Node>().dirty(NodeDamage::OtherNodeDamage);
    }

    // https://html.spec.whatwg.org/multipage/#dom-lfe-labels
    fn Labels(&self) -> Root<NodeList> {
        self.upcast::<HTMLElement>().labels()
    }

    // https://html.spec.whatwg.org/multipage/#dom-textarea/input-selectiondirection
    fn SetSelectionDirection(&self, direction: DOMString) {
        self.textinput.borrow_mut().selection_direction = SelectionDirection::from(direction);
    }

    // https://html.spec.whatwg.org/multipage/#dom-textarea/input-selectiondirection
    fn SelectionDirection(&self) -> DOMString {
        DOMString::from(self.textinput.borrow().selection_direction)
    }

    // https://html.spec.whatwg.org/multipage/#dom-textarea/input-selectionend
    fn SetSelectionEnd(&self, end: u32) {
        let selection_start = self.SelectionStart();
        self.textinput.borrow_mut().set_selection_range(selection_start, end);
        self.upcast::<Node>().dirty(NodeDamage::OtherNodeDamage);
    }

    // https://html.spec.whatwg.org/multipage/#dom-textarea/input-selectionend
    fn SelectionEnd(&self) -> u32 {
        self.textinput.borrow().get_absolute_insertion_point() as u32
    }

    // https://html.spec.whatwg.org/multipage/#dom-textarea/input-selectionstart
    fn SetSelectionStart(&self, start: u32) {
        let selection_end = self.SelectionEnd();
        self.textinput.borrow_mut().set_selection_range(start, selection_end);
        self.upcast::<Node>().dirty(NodeDamage::OtherNodeDamage);
    }

    // https://html.spec.whatwg.org/multipage/#dom-textarea/input-selectionstart
    fn SelectionStart(&self) -> u32 {
        self.textinput.borrow().get_selection_start()
    }

    // https://html.spec.whatwg.org/multipage/#dom-textarea/input-setselectionrange
    fn SetSelectionRange(&self, start: u32, end: u32, direction: Option<DOMString>) {
        let direction = direction.map_or(SelectionDirection::None, |d| SelectionDirection::from(d));
        self.textinput.borrow_mut().selection_direction = direction;
        self.textinput.borrow_mut().set_selection_range(start, end);
        let window = window_from_node(self);
        let _ = window.user_interaction_task_source().queue_event(
            &self.upcast(),
            atom!("select"),
            EventBubbles::Bubbles,
            EventCancelable::NotCancelable,
            window.r());
        self.upcast::<Node>().dirty(NodeDamage::OtherNodeDamage);
    }
}


impl HTMLTextAreaElement {
    // https://html.spec.whatwg.org/multipage/#concept-fe-mutable
    pub fn mutable(&self) -> bool {
        // https://html.spec.whatwg.org/multipage/#the-textarea-element:concept-fe-mutable
        !(self.Disabled() || self.ReadOnly())
    }
    pub fn reset(&self) {
        // https://html.spec.whatwg.org/multipage/#the-textarea-element:concept-form-reset-control
        self.SetValue(self.DefaultValue());
        self.value_changed.set(false);
    }
}


impl VirtualMethods for HTMLTextAreaElement {
    fn super_type(&self) -> Option<&VirtualMethods> {
        Some(self.upcast::<HTMLElement>() as &VirtualMethods)
    }

    fn attribute_mutated(&self, attr: &Attr, mutation: AttributeMutation) {
        self.super_type().unwrap().attribute_mutated(attr, mutation);
        match *attr.local_name() {
            atom!("disabled") => {
                let el = self.upcast::<Element>();
                match mutation {
                    AttributeMutation::Set(_) => {
                        el.set_disabled_state(true);
                        el.set_enabled_state(false);

                        el.set_read_write_state(false);
                    },
                    AttributeMutation::Removed => {
                        el.set_disabled_state(false);
                        el.set_enabled_state(true);
                        el.check_ancestors_disabled_state_for_form_control();

                        if !el.disabled_state() && !el.read_write_state() {
                            el.set_read_write_state(true);
                        }
                    }
                }
            },
            atom!("readonly") => {
                let el = self.upcast::<Element>();
                match mutation {
                    AttributeMutation::Set(_) => {
                        el.set_read_write_state(false);
                    },
                    AttributeMutation::Removed => {
                        el.set_read_write_state(!el.disabled_state());
                    }
                }
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

    fn parse_plain_attribute(&self, name: &Atom, value: DOMString) -> AttrValue {
        match *name {
            atom!("cols") => AttrValue::from_limited_u32(value.into(), DEFAULT_COLS),
            atom!("rows") => AttrValue::from_limited_u32(value.into(), DEFAULT_ROWS),
            _ => self.super_type().unwrap().parse_plain_attribute(name, value),
        }
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

    fn children_changed(&self, mutation: &ChildrenMutation) {
        if let Some(ref s) = self.super_type() {
            s.children_changed(mutation);
        }
        if !self.value_changed.get() {
            self.reset();
        }
    }

    // copied and modified from htmlinputelement.rs
    fn handle_event(&self, event: &Event) {
        if let Some(s) = self.super_type() {
            s.handle_event(event);
        }

        if event.type_() == atom!("click") && !event.DefaultPrevented() {
            //TODO: set the editing position for text inputs

            document_from_node(self).request_focus(self.upcast());
        } else if event.type_() == atom!("keydown") && !event.DefaultPrevented() {
            if let Some(kevent) = event.downcast::<KeyboardEvent>() {
                match self.textinput.borrow_mut().handle_keydown(kevent) {
                    KeyReaction::TriggerDefaultAction => (),
                    KeyReaction::DispatchInput => {
                        self.value_changed.set(true);

                        if event.IsTrusted() {
                            let window = window_from_node(self);
                            let _ = window.user_interaction_task_source().queue_event(
                                &self.upcast(),
                                atom!("input"),
                                EventBubbles::Bubbles,
                                EventCancelable::NotCancelable,
                                window.r());
                        }

                        self.upcast::<Node>().dirty(NodeDamage::OtherNodeDamage);
                        event.PreventDefault();
                    }
                    KeyReaction::RedrawSelection => {
                        self.upcast::<Node>().dirty(NodeDamage::OtherNodeDamage);
                        event.PreventDefault();
                    }
                    KeyReaction::Nothing => (),
                }
            }
        }
    }
}

impl FormControl for HTMLTextAreaElement {}

impl Validatable for HTMLTextAreaElement {}

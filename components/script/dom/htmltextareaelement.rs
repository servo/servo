/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::attr::Attr;
use dom::bindings::cell::DomRefCell;
use dom::bindings::codegen::Bindings::EventBinding::EventMethods;
use dom::bindings::codegen::Bindings::HTMLFormElementBinding::SelectionMode;
use dom::bindings::codegen::Bindings::HTMLTextAreaElementBinding;
use dom::bindings::codegen::Bindings::HTMLTextAreaElementBinding::HTMLTextAreaElementMethods;
use dom::bindings::codegen::Bindings::NodeBinding::NodeMethods;
use dom::bindings::error::ErrorResult;
use dom::bindings::inheritance::Castable;
use dom::bindings::root::{DomRoot, LayoutDom, MutNullableDom};
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
use dom::node::{CloneChildrenFlag, ChildrenMutation, Node, NodeDamage, UnbindContext};
use dom::node::{document_from_node, window_from_node};
use dom::nodelist::NodeList;
use dom::textcontrol::{TextControlElement, TextControlSelection};
use dom::validation::Validatable;
use dom::virtualmethods::VirtualMethods;
use dom_struct::dom_struct;
use html5ever::{LocalName, Prefix};
use script_traits::ScriptToConstellationChan;
use std::cell::Cell;
use std::default::Default;
use std::ops::Range;
use style::attr::AttrValue;
use style::element_state::ElementState;
use textinput::{Direction, KeyReaction, Lines, SelectionDirection, TextInput};

#[dom_struct]
pub struct HTMLTextAreaElement {
    htmlelement: HTMLElement,
    #[ignore_malloc_size_of = "#7193"]
    textinput: DomRefCell<TextInput<ScriptToConstellationChan>>,
    placeholder: DomRefCell<DOMString>,
    // https://html.spec.whatwg.org/multipage/#concept-textarea-dirty
    value_dirty: Cell<bool>,
    form_owner: MutNullableDom<HTMLFormElement>,
}

pub trait LayoutHTMLTextAreaElementHelpers {
    #[allow(unsafe_code)]
    unsafe fn value_for_layout(self) -> String;
    #[allow(unsafe_code)]
    unsafe fn selection_for_layout(self) -> Option<Range<usize>>;
    #[allow(unsafe_code)]
    fn get_cols(self) -> u32;
    #[allow(unsafe_code)]
    fn get_rows(self) -> u32;
}

impl LayoutHTMLTextAreaElementHelpers for LayoutDom<HTMLTextAreaElement> {
    #[allow(unrooted_must_root)]
    #[allow(unsafe_code)]
    unsafe fn value_for_layout(self) -> String {
        let text = (*self.unsafe_get())
            .textinput
            .borrow_for_layout()
            .get_content();
        if text.is_empty() {
            (*self.unsafe_get())
                .placeholder
                .borrow_for_layout()
                .replace("\r\n", "\n")
                .replace("\r", "\n")
        } else {
            text.into()
        }
    }

    #[allow(unrooted_must_root)]
    #[allow(unsafe_code)]
    unsafe fn selection_for_layout(self) -> Option<Range<usize>> {
        if !(*self.unsafe_get()).upcast::<Element>().focus_state() {
            return None;
        }
        let textinput = (*self.unsafe_get()).textinput.borrow_for_layout();
        Some(textinput.sorted_selection_offsets_range())
    }

    #[allow(unsafe_code)]
    fn get_cols(self) -> u32 {
        unsafe {
            (*self.upcast::<Element>().unsafe_get())
                .get_attr_for_layout(&ns!(), &local_name!("cols"))
                .map_or(DEFAULT_COLS, AttrValue::as_uint)
        }
    }

    #[allow(unsafe_code)]
    fn get_rows(self) -> u32 {
        unsafe {
            (*self.upcast::<Element>().unsafe_get())
                .get_attr_for_layout(&ns!(), &local_name!("rows"))
                .map_or(DEFAULT_ROWS, AttrValue::as_uint)
        }
    }
}

// https://html.spec.whatwg.org/multipage/#attr-textarea-cols-value
const DEFAULT_COLS: u32 = 20;

// https://html.spec.whatwg.org/multipage/#attr-textarea-rows-value
const DEFAULT_ROWS: u32 = 2;

const DEFAULT_MAX_LENGTH: i32 = -1;
const DEFAULT_MIN_LENGTH: i32 = -1;

impl HTMLTextAreaElement {
    fn new_inherited(
        local_name: LocalName,
        prefix: Option<Prefix>,
        document: &Document,
    ) -> HTMLTextAreaElement {
        let chan = document
            .window()
            .upcast::<GlobalScope>()
            .script_to_constellation_chan()
            .clone();
        HTMLTextAreaElement {
            htmlelement: HTMLElement::new_inherited_with_state(
                ElementState::IN_ENABLED_STATE | ElementState::IN_READ_WRITE_STATE,
                local_name,
                prefix,
                document,
            ),
            placeholder: DomRefCell::new(DOMString::new()),
            textinput: DomRefCell::new(TextInput::new(
                Lines::Multiple,
                DOMString::new(),
                chan,
                None,
                None,
                SelectionDirection::None,
            )),
            value_dirty: Cell::new(false),
            form_owner: Default::default(),
        }
    }

    #[allow(unrooted_must_root)]
    pub fn new(
        local_name: LocalName,
        prefix: Option<Prefix>,
        document: &Document,
    ) -> DomRoot<HTMLTextAreaElement> {
        Node::reflect_node(
            Box::new(HTMLTextAreaElement::new_inherited(
                local_name, prefix, document,
            )),
            document,
            HTMLTextAreaElementBinding::Wrap,
        )
    }

    fn update_placeholder_shown_state(&self) {
        let has_placeholder = !self.placeholder.borrow().is_empty();
        let has_value = !self.textinput.borrow().is_empty();
        let el = self.upcast::<Element>();
        el.set_placeholder_shown_state(has_placeholder && !has_value);
    }
}

impl TextControlElement for HTMLTextAreaElement {
    fn selection_api_applies(&self) -> bool {
        true
    }

    fn has_selectable_text(&self) -> bool {
        true
    }

    fn set_dirty_value_flag(&self, value: bool) {
        self.value_dirty.set(value)
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
    fn GetForm(&self) -> Option<DomRoot<HTMLFormElement>> {
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

    // https://html.spec.whatwg.org/multipage/#attr-textarea-maxlength
    make_int_getter!(MaxLength, "maxlength", DEFAULT_MAX_LENGTH);

    // https://html.spec.whatwg.org/multipage/#attr-textarea-maxlength
    make_limited_int_setter!(SetMaxLength, "maxlength", DEFAULT_MAX_LENGTH);

    // https://html.spec.whatwg.org/multipage/#attr-textarea-minlength
    make_int_getter!(MinLength, "minlength", DEFAULT_MIN_LENGTH);

    // https://html.spec.whatwg.org/multipage/#attr-textarea-minlength
    make_limited_int_setter!(SetMinLength, "minlength", DEFAULT_MIN_LENGTH);

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
        if !self.value_dirty.get() {
            self.reset();
        }
    }

    // https://html.spec.whatwg.org/multipage/#dom-textarea-value
    fn Value(&self) -> DOMString {
        self.textinput.borrow().get_content()
    }

    // https://html.spec.whatwg.org/multipage/#dom-textarea-value
    fn SetValue(&self, value: DOMString) {
        let mut textinput = self.textinput.borrow_mut();

        // Step 1
        let old_value = textinput.get_content();

        // Step 2
        textinput.set_content(value);

        // Step 3
        self.value_dirty.set(true);

        if old_value != textinput.get_content() {
            // Step 4
            textinput.clear_selection_to_limit(Direction::Forward);
        }

        self.upcast::<Node>().dirty(NodeDamage::OtherNodeDamage);
    }

    // https://html.spec.whatwg.org/multipage/#dom-textarea-textlength
    fn TextLength(&self) -> u32 {
        self.textinput.borrow().utf16_len() as u32
    }

    // https://html.spec.whatwg.org/multipage/#dom-lfe-labels
    fn Labels(&self) -> DomRoot<NodeList> {
        self.upcast::<HTMLElement>().labels()
    }

    // https://html.spec.whatwg.org/multipage/#dom-textarea/input-select
    fn Select(&self) {
        self.selection().dom_select();
    }

    // https://html.spec.whatwg.org/multipage/#dom-textarea/input-selectionstart
    fn GetSelectionStart(&self) -> Option<u32> {
        self.selection().dom_start()
    }

    // https://html.spec.whatwg.org/multipage/#dom-textarea/input-selectionstart
    fn SetSelectionStart(&self, start: Option<u32>) -> ErrorResult {
        self.selection().set_dom_start(start)
    }

    // https://html.spec.whatwg.org/multipage/#dom-textarea/input-selectionend
    fn GetSelectionEnd(&self) -> Option<u32> {
        self.selection().dom_end()
    }

    // https://html.spec.whatwg.org/multipage/#dom-textarea/input-selectionend
    fn SetSelectionEnd(&self, end: Option<u32>) -> ErrorResult {
        self.selection().set_dom_end(end)
    }

    // https://html.spec.whatwg.org/multipage/#dom-textarea/input-selectiondirection
    fn GetSelectionDirection(&self) -> Option<DOMString> {
        self.selection().dom_direction()
    }

    // https://html.spec.whatwg.org/multipage/#dom-textarea/input-selectiondirection
    fn SetSelectionDirection(&self, direction: Option<DOMString>) -> ErrorResult {
        self.selection().set_dom_direction(direction)
    }

    // https://html.spec.whatwg.org/multipage/#dom-textarea/input-setselectionrange
    fn SetSelectionRange(&self, start: u32, end: u32, direction: Option<DOMString>) -> ErrorResult {
        self.selection().set_dom_range(start, end, direction)
    }

    // https://html.spec.whatwg.org/multipage/#dom-textarea/input-setrangetext
    fn SetRangeText(&self, replacement: DOMString) -> ErrorResult {
        self.selection()
            .set_dom_range_text(replacement, None, None, Default::default())
    }

    // https://html.spec.whatwg.org/multipage/#dom-textarea/input-setrangetext
    fn SetRangeText_(
        &self,
        replacement: DOMString,
        start: u32,
        end: u32,
        selection_mode: SelectionMode,
    ) -> ErrorResult {
        self.selection()
            .set_dom_range_text(replacement, Some(start), Some(end), selection_mode)
    }
}

impl HTMLTextAreaElement {
    pub fn reset(&self) {
        // https://html.spec.whatwg.org/multipage/#the-textarea-element:concept-form-reset-control
        let mut textinput = self.textinput.borrow_mut();
        textinput.set_content(self.DefaultValue());
        self.value_dirty.set(false);
    }

    #[allow(unrooted_must_root)]
    fn selection(&self) -> TextControlSelection<Self> {
        TextControlSelection::new(&self, &self.textinput)
    }
}

impl VirtualMethods for HTMLTextAreaElement {
    fn super_type(&self) -> Option<&VirtualMethods> {
        Some(self.upcast::<HTMLElement>() as &VirtualMethods)
    }

    fn attribute_mutated(&self, attr: &Attr, mutation: AttributeMutation) {
        self.super_type().unwrap().attribute_mutated(attr, mutation);
        match *attr.local_name() {
            local_name!("disabled") => {
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
                    },
                }
            },
            local_name!("maxlength") => match *attr.value() {
                AttrValue::Int(_, value) => {
                    let mut textinput = self.textinput.borrow_mut();

                    if value < 0 {
                        textinput.set_max_length(None);
                    } else {
                        textinput.set_max_length(Some(value as usize))
                    }
                },
                _ => panic!("Expected an AttrValue::Int"),
            },
            local_name!("minlength") => match *attr.value() {
                AttrValue::Int(_, value) => {
                    let mut textinput = self.textinput.borrow_mut();

                    if value < 0 {
                        textinput.set_min_length(None);
                    } else {
                        textinput.set_min_length(Some(value as usize))
                    }
                },
                _ => panic!("Expected an AttrValue::Int"),
            },
            local_name!("placeholder") => {
                {
                    let mut placeholder = self.placeholder.borrow_mut();
                    placeholder.clear();
                    if let AttributeMutation::Set(_) = mutation {
                        placeholder.push_str(&attr.value());
                    }
                }
                self.update_placeholder_shown_state();
            },
            local_name!("readonly") => {
                let el = self.upcast::<Element>();
                match mutation {
                    AttributeMutation::Set(_) => {
                        el.set_read_write_state(false);
                    },
                    AttributeMutation::Removed => {
                        el.set_read_write_state(!el.disabled_state());
                    },
                }
            },
            local_name!("form") => {
                self.form_attribute_mutated(mutation);
            },
            _ => {},
        }
    }

    fn bind_to_tree(&self, tree_in_doc: bool) {
        if let Some(ref s) = self.super_type() {
            s.bind_to_tree(tree_in_doc);
        }

        self.upcast::<Element>()
            .check_ancestors_disabled_state_for_form_control();
    }

    fn parse_plain_attribute(&self, name: &LocalName, value: DOMString) -> AttrValue {
        match *name {
            local_name!("cols") => AttrValue::from_limited_u32(value.into(), DEFAULT_COLS),
            local_name!("rows") => AttrValue::from_limited_u32(value.into(), DEFAULT_ROWS),
            local_name!("maxlength") => {
                AttrValue::from_limited_i32(value.into(), DEFAULT_MAX_LENGTH)
            },
            local_name!("minlength") => {
                AttrValue::from_limited_i32(value.into(), DEFAULT_MIN_LENGTH)
            },
            _ => self
                .super_type()
                .unwrap()
                .parse_plain_attribute(name, value),
        }
    }

    fn unbind_from_tree(&self, context: &UnbindContext) {
        self.super_type().unwrap().unbind_from_tree(context);

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

    // The cloning steps for textarea elements must propagate the raw value
    // and dirty value flag from the node being cloned to the copy.
    fn cloning_steps(
        &self,
        copy: &Node,
        maybe_doc: Option<&Document>,
        clone_children: CloneChildrenFlag,
    ) {
        if let Some(ref s) = self.super_type() {
            s.cloning_steps(copy, maybe_doc, clone_children);
        }
        let el = copy.downcast::<HTMLTextAreaElement>().unwrap();
        el.value_dirty.set(self.value_dirty.get());
        let mut textinput = el.textinput.borrow_mut();
        textinput.set_content(self.textinput.borrow().get_content());
    }

    fn children_changed(&self, mutation: &ChildrenMutation) {
        if let Some(ref s) = self.super_type() {
            s.children_changed(mutation);
        }
        if !self.value_dirty.get() {
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
                // This can't be inlined, as holding on to textinput.borrow_mut()
                // during self.implicit_submission will cause a panic.
                let action = self.textinput.borrow_mut().handle_keydown(kevent);
                match action {
                    KeyReaction::TriggerDefaultAction => (),
                    KeyReaction::DispatchInput => {
                        self.value_dirty.set(true);
                        self.update_placeholder_shown_state();
                        self.upcast::<Node>().dirty(NodeDamage::OtherNodeDamage);
                        event.mark_as_handled();
                    },
                    KeyReaction::RedrawSelection => {
                        self.upcast::<Node>().dirty(NodeDamage::OtherNodeDamage);
                        event.mark_as_handled();
                    },
                    KeyReaction::Nothing => (),
                }
            }
        } else if event.type_() == atom!("keypress") && !event.DefaultPrevented() {
            if event.IsTrusted() {
                let window = window_from_node(self);
                let _ = window.user_interaction_task_source().queue_event(
                    &self.upcast(),
                    atom!("input"),
                    EventBubbles::Bubbles,
                    EventCancelable::NotCancelable,
                    &window,
                );
            }
        }
    }

    fn pop(&self) {
        self.super_type().unwrap().pop();

        // https://html.spec.whatwg.org/multipage/#the-textarea-element:stack-of-open-elements
        self.reset();
    }
}

impl FormControl for HTMLTextAreaElement {
    fn form_owner(&self) -> Option<DomRoot<HTMLFormElement>> {
        self.form_owner.get()
    }

    fn set_form_owner(&self, form: Option<&HTMLFormElement>) {
        self.form_owner.set(form);
    }

    fn to_element<'a>(&'a self) -> &'a Element {
        self.upcast::<Element>()
    }
}

impl Validatable for HTMLTextAreaElement {}

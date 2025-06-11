/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::Cell;
use std::default::Default;
use std::ops::Range;

use dom_struct::dom_struct;
use html5ever::{LocalName, Prefix, local_name, ns};
use js::rust::HandleObject;
use style::attr::AttrValue;
use stylo_dom::ElementState;
use utf16string::Utf16String;

use crate::clipboard_provider::EmbedderClipboardProvider;
use crate::dom::attr::Attr;
use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::EventBinding::EventMethods;
use crate::dom::bindings::codegen::Bindings::HTMLFormElementBinding::SelectionMode;
use crate::dom::bindings::codegen::Bindings::HTMLTextAreaElementBinding::HTMLTextAreaElementMethods;
use crate::dom::bindings::codegen::Bindings::NodeBinding::NodeMethods;
use crate::dom::bindings::error::ErrorResult;
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::root::{DomRoot, LayoutDom, MutNullableDom};
use crate::dom::bindings::str::DOMString;
use crate::dom::clipboardevent::ClipboardEvent;
use crate::dom::compositionevent::CompositionEvent;
use crate::dom::document::Document;
use crate::dom::element::{AttributeMutation, Element, LayoutElementHelpers};
use crate::dom::event::{Event, EventBubbles, EventCancelable};
use crate::dom::htmlelement::HTMLElement;
use crate::dom::htmlfieldsetelement::HTMLFieldSetElement;
use crate::dom::htmlformelement::{FormControl, HTMLFormElement};
use crate::dom::htmlinputelement::HTMLInputElement;
use crate::dom::keyboardevent::KeyboardEvent;
use crate::dom::node::{
    BindContext, ChildrenMutation, CloneChildrenFlag, Node, NodeDamage, NodeTraits, UnbindContext,
};
use crate::dom::nodelist::NodeList;
use crate::dom::textcontrol::{TextControlElement, TextControlSelection};
use crate::dom::validation::{Validatable, is_barred_by_datalist_ancestor};
use crate::dom::validitystate::{ValidationFlags, ValidityState};
use crate::dom::virtualmethods::VirtualMethods;
use crate::script_runtime::CanGc;
use crate::textinput::{
    Direction, KeyReaction, Lines, SelectionDirection, TextInput, UTF16CodeUnits,
    handle_text_clipboard_action,
};

#[dom_struct]
pub(crate) struct HTMLTextAreaElement {
    htmlelement: HTMLElement,
    #[ignore_malloc_size_of = "TextInput contains an IPCSender which cannot be measured"]
    #[no_trace]
    textinput: DomRefCell<TextInput<EmbedderClipboardProvider>>,
    placeholder: DomRefCell<DOMString>,
    // https://html.spec.whatwg.org/multipage/#concept-textarea-dirty
    value_dirty: Cell<bool>,
    form_owner: MutNullableDom<HTMLFormElement>,
    labels_node_list: MutNullableDom<NodeList>,
    validity_state: MutNullableDom<ValidityState>,
}

pub(crate) trait LayoutHTMLTextAreaElementHelpers {
    fn value_for_layout(self) -> String;
    fn selection_for_layout(self) -> Option<Range<usize>>;
    fn get_cols(self) -> u32;
    fn get_rows(self) -> u32;
}

#[allow(unsafe_code)]
impl<'dom> LayoutDom<'dom, HTMLTextAreaElement> {
    fn textinput_content(self) -> DOMString {
        unsafe {
            self.unsafe_get()
                .textinput
                .borrow_for_layout()
                .get_content()
                .into()
        }
    }

    fn textinput_sorted_selection_offsets_range(self) -> Range<usize> {
        unsafe {
            self.unsafe_get()
                .textinput
                .borrow_for_layout()
                .sorted_selection_offsets_range()
        }
    }

    fn placeholder(self) -> &'dom str {
        unsafe { self.unsafe_get().placeholder.borrow_for_layout() }
    }
}

impl LayoutHTMLTextAreaElementHelpers for LayoutDom<'_, HTMLTextAreaElement> {
    fn value_for_layout(self) -> String {
        let text = self.textinput_content();
        if text.is_empty() {
            // FIXME(nox): Would be cool to not allocate a new string if the
            // placeholder is single line, but that's an unimportant detail.
            self.placeholder().replace("\r\n", "\n").replace('\r', "\n")
        } else {
            text.into()
        }
    }

    fn selection_for_layout(self) -> Option<Range<usize>> {
        if !self.upcast::<Element>().focus_state() {
            return None;
        }
        Some(self.textinput_sorted_selection_offsets_range())
    }

    fn get_cols(self) -> u32 {
        self.upcast::<Element>()
            .get_attr_for_layout(&ns!(), &local_name!("cols"))
            .map_or(DEFAULT_COLS, AttrValue::as_uint)
    }

    fn get_rows(self) -> u32 {
        self.upcast::<Element>()
            .get_attr_for_layout(&ns!(), &local_name!("rows"))
            .map_or(DEFAULT_ROWS, AttrValue::as_uint)
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
        let constellation_sender = document
            .window()
            .as_global_scope()
            .script_to_constellation_chan()
            .clone();
        HTMLTextAreaElement {
            htmlelement: HTMLElement::new_inherited_with_state(
                ElementState::ENABLED | ElementState::READWRITE,
                local_name,
                prefix,
                document,
            ),
            placeholder: DomRefCell::new(DOMString::new()),
            textinput: DomRefCell::new(TextInput::new(
                Lines::Multiple,
                Default::default(),
                EmbedderClipboardProvider {
                    constellation_sender,
                    webview_id: document.webview_id(),
                },
                None,
                None,
                SelectionDirection::None,
            )),
            value_dirty: Cell::new(false),
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
    ) -> DomRoot<HTMLTextAreaElement> {
        Node::reflect_node_with_proto(
            Box::new(HTMLTextAreaElement::new_inherited(
                local_name, prefix, document,
            )),
            document,
            proto,
            can_gc,
        )
    }

    pub(crate) fn auto_directionality(&self) -> String {
        let value: String = self.Value().to_string();
        HTMLInputElement::directionality_from_value(&value)
    }

    fn update_placeholder_shown_state(&self) {
        let has_placeholder = !self.placeholder.borrow().is_empty();
        let has_value = !self.textinput.borrow().is_empty();
        let el = self.upcast::<Element>();
        el.set_placeholder_shown_state(has_placeholder && !has_value);
    }

    // https://html.spec.whatwg.org/multipage/#concept-fe-mutable
    fn is_mutable(&self) -> bool {
        // https://html.spec.whatwg.org/multipage/#the-textarea-element%3Aconcept-fe-mutable
        // https://html.spec.whatwg.org/multipage/#the-readonly-attribute:concept-fe-mutable
        !(self.upcast::<Element>().disabled_state() || self.ReadOnly())
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

impl HTMLTextAreaElementMethods<crate::DomTypeHolder> for HTMLTextAreaElement {
    // TODO A few of these attributes have default values and additional
    // constraints

    // https://html.spec.whatwg.org/multipage/#dom-textarea-cols
    make_uint_getter!(Cols, "cols", DEFAULT_COLS);

    // https://html.spec.whatwg.org/multipage/#dom-textarea-cols
    make_limited_uint_setter!(SetCols, "cols", DEFAULT_COLS);

    // https://html.spec.whatwg.org/multipage/#dom-input-dirName
    make_getter!(DirName, "dirname");

    // https://html.spec.whatwg.org/multipage/#dom-input-dirName
    make_setter!(SetDirName, "dirname");

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
    make_atomic_setter!(SetName, "name");

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
    fn SetDefaultValue(&self, value: DOMString, can_gc: CanGc) {
        self.upcast::<Node>().SetTextContent(Some(value), can_gc);

        // if the element's dirty value flag is false, then the element's
        // raw value must be set to the value of the element's textContent IDL attribute
        if !self.value_dirty.get() {
            self.reset();
        }
    }

    // https://html.spec.whatwg.org/multipage/#dom-textarea-value
    fn Value(&self) -> DOMString {
        self.textinput.borrow().get_content().to_string().into()
    }

    // https://html.spec.whatwg.org/multipage/#dom-textarea-value
    fn SetValue(&self, value: DOMString) {
        {
            let mut textinput = self.textinput.borrow_mut();

            // Step 1
            let old_value = textinput.get_content();

            // Step 2
            textinput.set_content(Utf16String::from(value));

            // Step 3
            self.value_dirty.set(true);

            if old_value != textinput.get_content() {
                // Step 4
                textinput.clear_selection_to_limit(Direction::Forward);
            }
        }

        self.validity_state()
            .perform_validation_and_update(ValidationFlags::all(), CanGc::note());
        self.upcast::<Node>().dirty(NodeDamage::OtherNodeDamage);
    }

    // https://html.spec.whatwg.org/multipage/#dom-textarea-textlength
    fn TextLength(&self) -> u32 {
        let UTF16CodeUnits(num_units) = self.textinput.borrow().utf16_len();
        num_units as u32
    }

    // https://html.spec.whatwg.org/multipage/#dom-lfe-labels
    make_labels_getter!(Labels, labels_node_list);

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
        self.selection().set_dom_range_text(
            Utf16String::from(replacement),
            None,
            None,
            Default::default(),
        )
    }

    // https://html.spec.whatwg.org/multipage/#dom-textarea/input-setrangetext
    fn SetRangeText_(
        &self,
        replacement: DOMString,
        start: u32,
        end: u32,
        selection_mode: SelectionMode,
    ) -> ErrorResult {
        self.selection().set_dom_range_text(
            Utf16String::from(replacement),
            Some(start),
            Some(end),
            selection_mode,
        )
    }

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

impl HTMLTextAreaElement {
    pub(crate) fn reset(&self) {
        // https://html.spec.whatwg.org/multipage/#the-textarea-element:concept-form-reset-control
        let mut textinput = self.textinput.borrow_mut();
        textinput.set_content(self.DefaultValue().str().into());
        self.value_dirty.set(false);
    }

    #[cfg_attr(crown, allow(crown::unrooted_must_root))]
    fn selection(&self) -> TextControlSelection<Self> {
        TextControlSelection::new(self, &self.textinput)
    }
}

impl VirtualMethods for HTMLTextAreaElement {
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
                el.update_sequentially_focusable_status(CanGc::note());
            },
            local_name!("maxlength") => match *attr.value() {
                AttrValue::Int(_, value) => {
                    let mut textinput = self.textinput.borrow_mut();

                    if value < 0 {
                        textinput.set_max_length(None);
                    } else {
                        textinput.set_max_length(Some(UTF16CodeUnits(value as usize)))
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
                        textinput.set_min_length(Some(UTF16CodeUnits(value as usize)))
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
                self.form_attribute_mutated(mutation, can_gc);
            },
            _ => {},
        }

        self.validity_state()
            .perform_validation_and_update(ValidationFlags::all(), can_gc);
    }

    fn bind_to_tree(&self, context: &BindContext, can_gc: CanGc) {
        if let Some(s) = self.super_type() {
            s.bind_to_tree(context, can_gc);
        }

        self.upcast::<Element>()
            .check_ancestors_disabled_state_for_form_control();

        self.validity_state()
            .perform_validation_and_update(ValidationFlags::all(), can_gc);
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

        self.validity_state()
            .perform_validation_and_update(ValidationFlags::all(), can_gc);
    }

    // The cloning steps for textarea elements must propagate the raw value
    // and dirty value flag from the node being cloned to the copy.
    fn cloning_steps(
        &self,
        copy: &Node,
        maybe_doc: Option<&Document>,
        clone_children: CloneChildrenFlag,
        can_gc: CanGc,
    ) {
        if let Some(s) = self.super_type() {
            s.cloning_steps(copy, maybe_doc, clone_children, can_gc);
        }
        let el = copy.downcast::<HTMLTextAreaElement>().unwrap();
        el.value_dirty.set(self.value_dirty.get());
        {
            let mut textinput = el.textinput.borrow_mut();
            textinput.set_content(self.textinput.borrow().get_content());
        }
        el.validity_state()
            .perform_validation_and_update(ValidationFlags::all(), can_gc);
    }

    fn children_changed(&self, mutation: &ChildrenMutation) {
        if let Some(s) = self.super_type() {
            s.children_changed(mutation);
        }
        if !self.value_dirty.get() {
            self.reset();
        }
    }

    // copied and modified from htmlinputelement.rs
    fn handle_event(&self, event: &Event, can_gc: CanGc) {
        if let Some(s) = self.super_type() {
            s.handle_event(event, can_gc);
        }

        if event.type_() == atom!("click") && !event.DefaultPrevented() {
            //TODO: set the editing position for text inputs
        } else if event.type_() == atom!("keydown") && !event.DefaultPrevented() {
            if let Some(kevent) = event.downcast::<KeyboardEvent>() {
                // This can't be inlined, as holding on to textinput.borrow_mut()
                // during self.implicit_submission will cause a panic.
                let action = self.textinput.borrow_mut().handle_keydown(kevent);
                match action {
                    KeyReaction::TriggerDefaultAction => (),
                    KeyReaction::DispatchInput => {
                        if event.IsTrusted() {
                            self.owner_global()
                                .task_manager()
                                .user_interaction_task_source()
                                .queue_event(
                                    self.upcast(),
                                    atom!("input"),
                                    EventBubbles::Bubbles,
                                    EventCancelable::NotCancelable,
                                );
                        }
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
            // keypress should be deprecated and replaced by beforeinput.
            // keypress was supposed to fire "blur" and "focus" events
            // but already done in `document.rs`
        } else if event.type_() == atom!("compositionstart") ||
            event.type_() == atom!("compositionupdate") ||
            event.type_() == atom!("compositionend")
        {
            if let Some(compositionevent) = event.downcast::<CompositionEvent>() {
                if event.type_() == atom!("compositionend") {
                    let _ = self
                        .textinput
                        .borrow_mut()
                        .handle_compositionend(compositionevent);
                    self.upcast::<Node>().dirty(NodeDamage::OtherNodeDamage);
                } else if event.type_() == atom!("compositionupdate") {
                    let _ = self
                        .textinput
                        .borrow_mut()
                        .handle_compositionupdate(compositionevent);
                    self.upcast::<Node>().dirty(NodeDamage::OtherNodeDamage);
                }
                event.mark_as_handled();
            }
        } else if let Some(clipboard_event) = event.downcast::<ClipboardEvent>() {
            if !event.DefaultPrevented() {
                handle_text_clipboard_action(self, &self.textinput, clipboard_event, CanGc::note());
            }
        }

        self.validity_state()
            .perform_validation_and_update(ValidationFlags::all(), can_gc);
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

    fn to_element(&self) -> &Element {
        self.upcast::<Element>()
    }
}

impl Validatable for HTMLTextAreaElement {
    fn as_element(&self) -> &Element {
        self.upcast()
    }

    fn validity_state(&self) -> DomRoot<ValidityState> {
        self.validity_state
            .or_init(|| ValidityState::new(&self.owner_window(), self.upcast(), CanGc::note()))
    }

    fn is_instance_validatable(&self) -> bool {
        // https://html.spec.whatwg.org/multipage/#enabling-and-disabling-form-controls%3A-the-disabled-attribute%3Abarred-from-constraint-validation
        // https://html.spec.whatwg.org/multipage/#the-textarea-element%3Abarred-from-constraint-validation
        // https://html.spec.whatwg.org/multipage/#the-datalist-element%3Abarred-from-constraint-validation
        !self.upcast::<Element>().disabled_state() &&
            !self.ReadOnly() &&
            !is_barred_by_datalist_ancestor(self.upcast())
    }

    fn perform_validation(
        &self,
        validate_flags: ValidationFlags,
        _can_gc: CanGc,
    ) -> ValidationFlags {
        let mut failed_flags = ValidationFlags::empty();

        let textinput = self.textinput.borrow();
        let UTF16CodeUnits(value_len) = textinput.utf16_len();
        let last_edit_by_user = !textinput.was_last_change_by_set_content();
        let value_dirty = self.value_dirty.get();

        // https://html.spec.whatwg.org/multipage/#suffering-from-being-missing
        // https://html.spec.whatwg.org/multipage/#the-textarea-element%3Asuffering-from-being-missing
        if validate_flags.contains(ValidationFlags::VALUE_MISSING) &&
            self.Required() &&
            self.is_mutable() &&
            value_len == 0
        {
            failed_flags.insert(ValidationFlags::VALUE_MISSING);
        }

        if value_dirty && last_edit_by_user && value_len > 0 {
            // https://html.spec.whatwg.org/multipage/#suffering-from-being-too-long
            // https://html.spec.whatwg.org/multipage/#limiting-user-input-length%3A-the-maxlength-attribute%3Asuffering-from-being-too-long
            if validate_flags.contains(ValidationFlags::TOO_LONG) {
                let max_length = self.MaxLength();
                if max_length != DEFAULT_MAX_LENGTH && value_len > (max_length as usize) {
                    failed_flags.insert(ValidationFlags::TOO_LONG);
                }
            }

            // https://html.spec.whatwg.org/multipage/#suffering-from-being-too-short
            // https://html.spec.whatwg.org/multipage/#setting-minimum-input-length-requirements%3A-the-minlength-attribute%3Asuffering-from-being-too-short
            if validate_flags.contains(ValidationFlags::TOO_SHORT) {
                let min_length = self.MinLength();
                if min_length != DEFAULT_MIN_LENGTH && value_len < (min_length as usize) {
                    failed_flags.insert(ValidationFlags::TOO_SHORT);
                }
            }
        }

        failed_flags
    }
}

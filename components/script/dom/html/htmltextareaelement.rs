/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::{Cell, Ref, RefCell};
use std::default::Default;

use dom_struct::dom_struct;
use embedder_traits::{EmbedderControlRequest, InputMethodRequest, InputMethodType};
use fonts::{ByteIndex, TextByteRange};
use html5ever::{LocalName, Prefix, local_name, ns};
use js::context::JSContext;
use js::rust::HandleObject;
use layout_api::{ScriptSelection, SharedSelection};
use servo_base::text::Utf16CodeUnitLength;
use style::attr::AttrValue;
use stylo_dom::ElementState;

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
use crate::dom::clipboardevent::{ClipboardEvent, ClipboardEventType};
use crate::dom::compositionevent::CompositionEvent;
use crate::dom::document::Document;
use crate::dom::document_embedder_controls::ControlElement;
use crate::dom::element::{AttributeMutation, Element};
use crate::dom::event::Event;
use crate::dom::html::htmlelement::HTMLElement;
use crate::dom::html::htmlfieldsetelement::HTMLFieldSetElement;
use crate::dom::html::htmlformelement::{FormControl, HTMLFormElement};
use crate::dom::html::input_element::HTMLInputElement;
use crate::dom::htmlinputelement::text_input_widget::TextInputWidget;
use crate::dom::keyboardevent::KeyboardEvent;
use crate::dom::node::{
    BindContext, ChildrenMutation, CloneChildrenFlag, Node, NodeDamage, NodeTraits, UnbindContext,
};
use crate::dom::nodelist::NodeList;
use crate::dom::textcontrol::{TextControlElement, TextControlSelection};
use crate::dom::types::{FocusEvent, MouseEvent};
use crate::dom::validation::{Validatable, is_barred_by_datalist_ancestor};
use crate::dom::validitystate::{ValidationFlags, ValidityState};
use crate::dom::virtualmethods::VirtualMethods;
use crate::script_runtime::CanGc;
use crate::textinput::{ClipboardEventFlags, IsComposing, KeyReaction, Lines, TextInput};

#[dom_struct]
pub(crate) struct HTMLTextAreaElement {
    htmlelement: HTMLElement,
    #[no_trace]
    textinput: DomRefCell<TextInput<EmbedderClipboardProvider>>,
    placeholder: RefCell<DOMString>,
    // https://html.spec.whatwg.org/multipage/#concept-textarea-dirty
    value_dirty: Cell<bool>,
    form_owner: MutNullableDom<HTMLFormElement>,
    labels_node_list: MutNullableDom<NodeList>,
    validity_state: MutNullableDom<ValidityState>,
    /// A [`TextInputWidget`] that manages the shadow DOM for this `<textarea>`.
    text_input_widget: DomRefCell<TextInputWidget>,
    /// A [`SharedSelection`] that is shared with layout. This can be updated dyanmnically
    /// and layout should reflect the new value after a display list update.
    #[no_trace]
    #[conditional_malloc_size_of]
    shared_selection: SharedSelection,
}

impl LayoutDom<'_, HTMLTextAreaElement> {
    pub(crate) fn selection_for_layout(self) -> SharedSelection {
        self.unsafe_get().shared_selection.clone()
    }

    pub(crate) fn get_cols(self) -> u32 {
        self.upcast::<Element>()
            .get_attr_for_layout(&ns!(), &local_name!("cols"))
            .map_or(DEFAULT_COLS, AttrValue::as_uint)
    }

    pub(crate) fn get_rows(self) -> u32 {
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
        let embedder_sender = document
            .window()
            .as_global_scope()
            .script_to_embedder_chan()
            .clone();
        HTMLTextAreaElement {
            htmlelement: HTMLElement::new_inherited_with_state(
                ElementState::ENABLED | ElementState::READWRITE,
                local_name,
                prefix,
                document,
            ),
            placeholder: Default::default(),
            textinput: DomRefCell::new(TextInput::new(
                Lines::Multiple,
                DOMString::new(),
                EmbedderClipboardProvider {
                    embedder_sender,
                    webview_id: document.webview_id(),
                },
            )),
            value_dirty: Cell::new(false),
            form_owner: Default::default(),
            labels_node_list: Default::default(),
            validity_state: Default::default(),
            text_input_widget: Default::default(),
            shared_selection: Default::default(),
        }
    }

    pub(crate) fn new(
        cx: &mut js::context::JSContext,
        local_name: LocalName,
        prefix: Option<Prefix>,
        document: &Document,
        proto: Option<HandleObject>,
    ) -> DomRoot<HTMLTextAreaElement> {
        Node::reflect_node_with_proto(
            cx,
            Box::new(HTMLTextAreaElement::new_inherited(
                local_name, prefix, document,
            )),
            document,
            proto,
        )
    }

    pub(crate) fn auto_directionality(&self) -> String {
        let value: String = self.Value().to_string();
        HTMLInputElement::directionality_from_value(&value)
    }

    // https://html.spec.whatwg.org/multipage/#concept-fe-mutable
    pub(crate) fn is_mutable(&self) -> bool {
        // https://html.spec.whatwg.org/multipage/#the-textarea-element%3Aconcept-fe-mutable
        // https://html.spec.whatwg.org/multipage/#the-readonly-attribute:concept-fe-mutable
        !(self.upcast::<Element>().disabled_state() || self.ReadOnly())
    }

    fn handle_focus_event(&self, event: &FocusEvent) {
        let event_type = event.upcast::<Event>().type_();
        if *event_type == *"blur" {
            self.owner_document()
                .embedder_controls()
                .hide_embedder_control(self.upcast());
        } else if *event_type == *"focus" {
            self.owner_document()
                .embedder_controls()
                .show_embedder_control(
                    ControlElement::Ime(DomRoot::from_ref(self.upcast())),
                    EmbedderControlRequest::InputMethod(InputMethodRequest {
                        input_method_type: InputMethodType::Text,
                        text: self.Value().to_string(),
                        insertion_point: self.GetSelectionEnd(),
                        multiline: false,
                        // We follow chromium's heuristic to show the virtual keyboard only if user had interacted before.
                        allow_virtual_keyboard: self.owner_window().has_sticky_activation(),
                    }),
                    None,
                );
        } else {
            unreachable!("Got unexpected FocusEvent {event_type:?}");
        }

        // Focus changes can activate or deactivate a selection.
        self.maybe_update_shared_selection();
    }

    #[expect(unsafe_code)]
    fn handle_text_content_changed(&self, _can_gc: CanGc) {
        // TODO https://github.com/servo/servo/issues/43255
        let mut cx = unsafe { script_bindings::script_runtime::temp_cx() };
        let cx = &mut cx;

        self.validity_state(CanGc::from_cx(cx))
            .perform_validation_and_update(ValidationFlags::all(), CanGc::from_cx(cx));

        let placeholder_shown =
            self.textinput.borrow().is_empty() && !self.placeholder.borrow().is_empty();
        self.upcast::<Element>()
            .set_placeholder_shown_state(placeholder_shown);

        self.text_input_widget.borrow().update_shadow_tree(cx, self);
        self.text_input_widget
            .borrow()
            .update_placeholder_contents(cx, self);
        self.maybe_update_shared_selection();
    }

    fn handle_mouse_event(&self, mouse_event: &MouseEvent) {
        if mouse_event.upcast::<Event>().DefaultPrevented() {
            return;
        }

        // Only respond to mouse events if we are displayed as text input or a password. If the
        // placeholder is displayed, also don't do any interactive mouse event handling.
        if self.textinput.borrow().is_empty() {
            return;
        }
        let node = self.upcast();
        if self
            .textinput
            .borrow_mut()
            .handle_mouse_event(node, mouse_event)
        {
            self.maybe_update_shared_selection();
        }
    }
}

impl TextControlElement for HTMLTextAreaElement {
    fn selection_api_applies(&self) -> bool {
        true
    }

    fn has_selectable_text(&self) -> bool {
        !self.textinput.borrow().get_content().is_empty()
    }

    fn has_uncollapsed_selection(&self) -> bool {
        self.textinput.borrow().has_uncollapsed_selection()
    }

    fn set_dirty_value_flag(&self, value: bool) {
        self.value_dirty.set(value)
    }

    fn select_all(&self) {
        self.textinput.borrow_mut().select_all();
        self.maybe_update_shared_selection();
    }

    fn maybe_update_shared_selection(&self) {
        let offsets = self.textinput.borrow().sorted_selection_offsets_range();
        let (start, end) = (offsets.start.0, offsets.end.0);
        let range = TextByteRange::new(ByteIndex(start), ByteIndex(end));
        let enabled = self.upcast::<Element>().focus_state();

        let mut shared_selection = self.shared_selection.borrow_mut();
        if range == shared_selection.range && enabled == shared_selection.enabled {
            return;
        }
        *shared_selection = ScriptSelection {
            range,
            character_range: self
                .textinput
                .borrow()
                .sorted_selection_character_offsets_range(),
            enabled,
        };
        self.owner_window().layout().set_needs_new_display_list();
    }

    fn placeholder_text<'a>(&'a self) -> Ref<'a, DOMString> {
        self.placeholder.borrow()
    }

    fn value_text(&self) -> DOMString {
        self.Value()
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

    /// <https://html.spec.whatwg.org/multipage/#dom-fae-form>
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

    /// <https://html.spec.whatwg.org/multipage/#dom-textarea-type>
    fn Type(&self) -> DOMString {
        DOMString::from("textarea")
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-textarea-defaultvalue>
    fn DefaultValue(&self) -> DOMString {
        self.upcast::<Node>().GetTextContent().unwrap()
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-textarea-defaultvalue>
    fn SetDefaultValue(&self, cx: &mut JSContext, value: DOMString) {
        self.upcast::<Node>()
            .set_text_content_for_element(cx, Some(value));

        // if the element's dirty value flag is false, then the element's
        // raw value must be set to the value of the element's textContent IDL attribute
        if !self.value_dirty.get() {
            self.reset(CanGc::from_cx(cx));
        }
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-textarea-value>
    fn Value(&self) -> DOMString {
        self.textinput.borrow().get_content()
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-textarea-value>
    fn SetValue(&self, value: DOMString, can_gc: CanGc) {
        // Step 1: Let oldAPIValue be this element's API value.
        let old_api_value = self.Value();

        // Step 2:  Set this element's raw value to the new value.
        self.textinput.borrow_mut().set_content(value);

        // Step 3: Set this element's dirty value flag to true.
        self.value_dirty.set(true);

        // Step 4: If the new API value is different from oldAPIValue, then move
        // the text entry cursor position to the end of the text control,
        // unselecting any selected text and resetting the selection direction to
        // "none".
        if old_api_value != self.Value() {
            self.textinput.borrow_mut().clear_selection_to_end();
            self.handle_text_content_changed(can_gc);
        }
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-textarea-textlength>
    fn TextLength(&self) -> u32 {
        self.textinput.borrow().len_utf16().0 as u32
    }

    // https://html.spec.whatwg.org/multipage/#dom-lfe-labels
    make_labels_getter!(Labels, labels_node_list);

    /// <https://html.spec.whatwg.org/multipage/#dom-textarea/input-select>
    fn Select(&self) {
        self.selection().dom_select();
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-textarea/input-selectionstart>
    fn GetSelectionStart(&self) -> Option<u32> {
        self.selection().dom_start().map(|start| start.0 as u32)
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-textarea/input-selectionstart>
    fn SetSelectionStart(&self, start: Option<u32>) -> ErrorResult {
        self.selection()
            .set_dom_start(start.map(Utf16CodeUnitLength::from))
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-textarea/input-selectionend>
    fn GetSelectionEnd(&self) -> Option<u32> {
        self.selection().dom_end().map(|end| end.0 as u32)
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-textarea/input-selectionend>
    fn SetSelectionEnd(&self, end: Option<u32>) -> ErrorResult {
        self.selection()
            .set_dom_end(end.map(Utf16CodeUnitLength::from))
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-textarea/input-selectiondirection>
    fn GetSelectionDirection(&self) -> Option<DOMString> {
        self.selection().dom_direction()
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-textarea/input-selectiondirection>
    fn SetSelectionDirection(&self, direction: Option<DOMString>) -> ErrorResult {
        self.selection().set_dom_direction(direction)
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-textarea/input-setselectionrange>
    fn SetSelectionRange(&self, start: u32, end: u32, direction: Option<DOMString>) -> ErrorResult {
        self.selection().set_dom_range(
            Utf16CodeUnitLength::from(start),
            Utf16CodeUnitLength::from(end),
            direction,
        )
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-textarea/input-setrangetext>
    fn SetRangeText(&self, replacement: DOMString) -> ErrorResult {
        self.selection()
            .set_dom_range_text(replacement, None, None, Default::default())
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-textarea/input-setrangetext>
    fn SetRangeText_(
        &self,
        replacement: DOMString,
        start: u32,
        end: u32,
        selection_mode: SelectionMode,
    ) -> ErrorResult {
        self.selection().set_dom_range_text(
            replacement,
            Some(Utf16CodeUnitLength::from(start)),
            Some(Utf16CodeUnitLength::from(end)),
            selection_mode,
        )
    }

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

impl HTMLTextAreaElement {
    /// <https://w3c.github.io/webdriver/#ref-for-dfn-clear-algorithm-4>
    /// Used by WebDriver to clear the textarea element.
    pub(crate) fn clear(&self) {
        self.value_dirty.set(false);
        self.textinput.borrow_mut().set_content(DOMString::from(""));
    }

    pub(crate) fn reset(&self, can_gc: CanGc) {
        // https://html.spec.whatwg.org/multipage/#the-textarea-element:concept-form-reset-control
        self.value_dirty.set(false);
        self.textinput.borrow_mut().set_content(self.DefaultValue());
        self.handle_text_content_changed(can_gc);
    }

    #[cfg_attr(crown, expect(crown::unrooted_must_root))]
    fn selection(&self) -> TextControlSelection<'_, Self> {
        TextControlSelection::new(self, &self.textinput)
    }

    fn handle_key_reaction(&self, action: KeyReaction, event: &Event, can_gc: CanGc) {
        match action {
            KeyReaction::TriggerDefaultAction => (),
            KeyReaction::DispatchInput(text, is_composing, input_type) => {
                if event.IsTrusted() {
                    self.textinput.borrow().queue_input_event(
                        self.upcast(),
                        text,
                        is_composing,
                        input_type,
                    );
                }
                self.value_dirty.set(true);
                self.handle_text_content_changed(can_gc);
                event.mark_as_handled();
            },
            KeyReaction::RedrawSelection => {
                self.maybe_update_shared_selection();
                event.mark_as_handled();
            },
            KeyReaction::Nothing => (),
        }
    }
}

impl VirtualMethods for HTMLTextAreaElement {
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
                    AttributeMutation::Set(..) => {
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
                        textinput.set_max_length(Some(Utf16CodeUnitLength(value as usize)))
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
                        textinput.set_min_length(Some(Utf16CodeUnitLength(value as usize)))
                    }
                },
                _ => panic!("Expected an AttrValue::Int"),
            },
            local_name!("placeholder") => {
                {
                    let mut placeholder = self.placeholder.borrow_mut();
                    match mutation {
                        AttributeMutation::Set(..) => {
                            let value = attr.value();
                            let value_str: &str = value.as_ref();
                            *placeholder =
                                value_str.replace("\r\n", "\n").replace('\r', "\n").into();
                        },
                        AttributeMutation::Removed => placeholder.clear(),
                    }
                }
                self.handle_text_content_changed(CanGc::from_cx(cx));
            },
            local_name!("readonly") => {
                let el = self.upcast::<Element>();
                match mutation {
                    AttributeMutation::Set(..) => {
                        el.set_read_write_state(false);
                    },
                    AttributeMutation::Removed => {
                        el.set_read_write_state(!el.disabled_state());
                    },
                }
            },
            local_name!("form") => {
                self.form_attribute_mutated(mutation, CanGc::from_cx(cx));
            },
            _ => {},
        }

        self.validity_state(CanGc::from_cx(cx))
            .perform_validation_and_update(ValidationFlags::all(), CanGc::from_cx(cx));
    }

    fn bind_to_tree(&self, cx: &mut JSContext, context: &BindContext) {
        if let Some(s) = self.super_type() {
            s.bind_to_tree(cx, context);
        }

        self.upcast::<Element>()
            .check_ancestors_disabled_state_for_form_control();

        self.handle_text_content_changed(CanGc::from_cx(cx));
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

        self.validity_state(can_gc)
            .perform_validation_and_update(ValidationFlags::all(), can_gc);
    }

    // The cloning steps for textarea elements must propagate the raw value
    // and dirty value flag from the node being cloned to the copy.
    fn cloning_steps(
        &self,
        cx: &mut JSContext,
        copy: &Node,
        maybe_doc: Option<&Document>,
        clone_children: CloneChildrenFlag,
    ) {
        if let Some(s) = self.super_type() {
            s.cloning_steps(cx, copy, maybe_doc, clone_children);
        }
        let el = copy.downcast::<HTMLTextAreaElement>().unwrap();
        el.value_dirty.set(self.value_dirty.get());
        {
            let mut textinput = el.textinput.borrow_mut();
            textinput.set_content(self.textinput.borrow().get_content());
        }
        el.validity_state(CanGc::from_cx(cx))
            .perform_validation_and_update(ValidationFlags::all(), CanGc::from_cx(cx));
    }

    fn children_changed(&self, cx: &mut JSContext, mutation: &ChildrenMutation) {
        if let Some(s) = self.super_type() {
            s.children_changed(cx, mutation);
        }
        if !self.value_dirty.get() {
            self.reset(CanGc::from_cx(cx));
        }
    }

    // copied and modified from htmlinputelement.rs
    fn handle_event(&self, event: &Event, can_gc: CanGc) {
        if let Some(mouse_event) = event.downcast::<MouseEvent>() {
            self.handle_mouse_event(mouse_event);
            event.mark_as_handled();
        } else if event.type_() == atom!("keydown") && !event.DefaultPrevented() {
            if let Some(keyboard_event) = event.downcast::<KeyboardEvent>() {
                // This can't be inlined, as holding on to textinput.borrow_mut()
                // during self.implicit_submission will cause a panic.
                let action = self.textinput.borrow_mut().handle_keydown(keyboard_event);
                self.handle_key_reaction(action, event, can_gc);
            }
        } else if event.type_() == atom!("compositionstart") ||
            event.type_() == atom!("compositionupdate") ||
            event.type_() == atom!("compositionend")
        {
            if let Some(compositionevent) = event.downcast::<CompositionEvent>() {
                if event.type_() == atom!("compositionend") {
                    let action = self
                        .textinput
                        .borrow_mut()
                        .handle_compositionend(compositionevent);
                    self.handle_key_reaction(action, event, can_gc);
                    self.upcast::<Node>().dirty(NodeDamage::Other);
                } else if event.type_() == atom!("compositionupdate") {
                    let action = self
                        .textinput
                        .borrow_mut()
                        .handle_compositionupdate(compositionevent);
                    self.handle_key_reaction(action, event, can_gc);
                    self.upcast::<Node>().dirty(NodeDamage::Other);
                }
                self.maybe_update_shared_selection();
                event.mark_as_handled();
            }
        } else if let Some(clipboard_event) = event.downcast::<ClipboardEvent>() {
            let reaction = self
                .textinput
                .borrow_mut()
                .handle_clipboard_event(clipboard_event);

            let flags = reaction.flags;
            if flags.contains(ClipboardEventFlags::FireClipboardChangedEvent) {
                self.owner_document().event_handler().fire_clipboard_event(
                    None,
                    ClipboardEventType::Change,
                    can_gc,
                );
            }
            if flags.contains(ClipboardEventFlags::QueueInputEvent) {
                self.textinput.borrow().queue_input_event(
                    self.upcast(),
                    reaction.text,
                    IsComposing::NotComposing,
                    reaction.input_type,
                );
            }
            if !flags.is_empty() {
                event.mark_as_handled();
                self.handle_text_content_changed(can_gc);
            }
        } else if let Some(event) = event.downcast::<FocusEvent>() {
            self.handle_focus_event(event);
        }

        self.validity_state(can_gc)
            .perform_validation_and_update(ValidationFlags::all(), can_gc);

        if let Some(super_type) = self.super_type() {
            super_type.handle_event(event, can_gc);
        }
    }

    fn pop(&self) {
        self.super_type().unwrap().pop();

        // https://html.spec.whatwg.org/multipage/#the-textarea-element:stack-of-open-elements
        self.reset(CanGc::deprecated_note());
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

    fn validity_state(&self, can_gc: CanGc) -> DomRoot<ValidityState> {
        self.validity_state
            .or_init(|| ValidityState::new(&self.owner_window(), self.upcast(), can_gc))
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
        let Utf16CodeUnitLength(value_len) = textinput.len_utf16();
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

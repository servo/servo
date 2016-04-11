/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use caseless::compatibility_caseless_match_str;
use dom::activation::{Activatable, ActivationSource, synthetic_click_activation};
use dom::attr::{Attr, AttrValue};
use dom::bindings::cell::DOMRefCell;
use dom::bindings::codegen::Bindings::AttrBinding::AttrMethods;
use dom::bindings::codegen::Bindings::EventBinding::EventMethods;
use dom::bindings::codegen::Bindings::HTMLInputElementBinding;
use dom::bindings::codegen::Bindings::HTMLInputElementBinding::HTMLInputElementMethods;
use dom::bindings::codegen::Bindings::KeyboardEventBinding::KeyboardEventMethods;
use dom::bindings::error::{Error, ErrorResult};
use dom::bindings::global::GlobalRef;
use dom::bindings::inheritance::Castable;
use dom::bindings::js::{JS, LayoutJS, Root, RootedReference};
use dom::bindings::refcounted::Trusted;
use dom::document::Document;
use dom::element::{AttributeMutation, Element, RawLayoutElementHelpers, LayoutElementHelpers};
use dom::event::{Event, EventBubbles, EventCancelable};
use dom::eventtarget::EventTarget;
use dom::htmlelement::HTMLElement;
use dom::htmlfieldsetelement::HTMLFieldSetElement;
use dom::htmlformelement::{FormControl, FormDatum, FormSubmitter, HTMLFormElement};
use dom::htmlformelement::{ResetFrom, SubmittedFrom};
use dom::keyboardevent::KeyboardEvent;
use dom::node::{Node, NodeDamage, UnbindContext};
use dom::node::{document_from_node, window_from_node};
use dom::nodelist::NodeList;
use dom::validation::Validatable;
use dom::virtualmethods::VirtualMethods;
use msg::constellation_msg::ConstellationChan;
use range::Range;
use script_runtime::CommonScriptMsg;
use script_runtime::ScriptThreadEventCategory::InputEvent;
use script_thread::Runnable;
use script_traits::ScriptMsg as ConstellationMsg;
use std::borrow::ToOwned;
use std::cell::Cell;
use string_cache::Atom;
use style::element_state::*;
use textinput::KeyReaction::{DispatchInput, Nothing, RedrawSelection, TriggerDefaultAction};
use textinput::Lines::Single;
use textinput::TextInput;
use util::str::{DOMString, search_index};

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

#[derive(Debug, PartialEq)]
enum ValueMode {
    Value,
    Default,
    DefaultOn,
    Filename,
}

#[derive(JSTraceable, PartialEq, Copy, Clone)]
#[derive(HeapSizeOf)]
enum SelectionDirection {
    Forward,
    Backward,
    None
}

#[dom_struct]
pub struct HTMLInputElement {
    htmlelement: HTMLElement,
    input_type: Cell<InputType>,
    checked_changed: Cell<bool>,
    placeholder: DOMRefCell<DOMString>,
    value_changed: Cell<bool>,
    size: Cell<u32>,
    maxlength: Cell<i32>,
    #[ignore_heap_size_of = "#7193"]
    textinput: DOMRefCell<TextInput<ConstellationChan<ConstellationMsg>>>,
    activation_state: DOMRefCell<InputActivationState>,
    // https://html.spec.whatwg.org/multipage/#concept-input-value-dirty-flag
    value_dirty: Cell<bool>,

    selection_direction: Cell<SelectionDirection>,

    // TODO: selected files for file input
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

static DEFAULT_INPUT_SIZE: u32 = 20;
static DEFAULT_MAX_LENGTH: i32 = -1;

impl HTMLInputElement {
    fn new_inherited(localName: Atom, prefix: Option<DOMString>, document: &Document) -> HTMLInputElement {
        let chan = document.window().constellation_chan();
        HTMLInputElement {
            htmlelement:
                HTMLElement::new_inherited_with_state(IN_ENABLED_STATE,
                                                      localName, prefix, document),
            input_type: Cell::new(InputType::InputText),
            placeholder: DOMRefCell::new(DOMString::new()),
            checked_changed: Cell::new(false),
            value_changed: Cell::new(false),
            maxlength: Cell::new(DEFAULT_MAX_LENGTH),
            size: Cell::new(DEFAULT_INPUT_SIZE),
            textinput: DOMRefCell::new(TextInput::new(Single, DOMString::new(), chan, None)),
            activation_state: DOMRefCell::new(InputActivationState::new()),
            value_dirty: Cell::new(false),
            selection_direction: Cell::new(SelectionDirection::None)
        }
    }

    #[allow(unrooted_must_root)]
    pub fn new(localName: Atom,
               prefix: Option<DOMString>,
               document: &Document) -> Root<HTMLInputElement> {
        let element = HTMLInputElement::new_inherited(localName, prefix, document);
        Node::reflect_node(box element, document, HTMLInputElementBinding::Wrap)
    }

    pub fn type_(&self) -> Atom {
        self.upcast::<Element>()
            .get_attribute(&ns!(), &atom!("type"))
            .map_or_else(|| atom!(""), |a| a.value().as_atom().to_owned())
    }

    // https://html.spec.whatwg.org/multipage/#input-type-attr-summary
    fn get_value_mode(&self) -> ValueMode {
        match self.input_type.get() {
            InputType::InputSubmit |
            InputType::InputReset |
            InputType::InputButton |
            InputType::InputImage => ValueMode::Default,
            InputType::InputCheckbox |
            InputType::InputRadio => ValueMode::DefaultOn,
            InputType::InputPassword |
            InputType::InputText => ValueMode::Value,
            InputType::InputFile => ValueMode::Filename,
        }
    }

    // this method exists so that the functions SetSelectionStart() and SetSelectionEnd()
    // don't needlessly allocate strings
    fn set_selection_range(&self, start: u32, end: u32, direction: &SelectionDirection) {
        let mut text_input = self.textinput.borrow_mut();

        let mut start = start as usize;
        let mut end = end as usize;

        let text_end = text_input.get_content().len();
        if start > text_end {
            start = text_end;
        }
        if end > text_end {
            end = text_end;
        }

        if start >= end {
            start = end;
        }

        text_input.selection_begin = Some(text_input.get_text_point_for_absolute_point(start));
        text_input.edit_point = text_input.get_text_point_for_absolute_point(end);
        self.selection_direction.set(*direction);
        self.upcast::<Node>().dirty(NodeDamage::OtherNodeDamage);
    }

}

pub trait LayoutHTMLInputElementHelpers {
    #[allow(unsafe_code)]
    unsafe fn get_value_for_layout(self) -> String;
    #[allow(unsafe_code)]
    unsafe fn get_size_for_layout(self) -> u32;
    #[allow(unsafe_code)]
    unsafe fn get_selection_for_layout(self) -> Option<Range<isize>>;
    #[allow(unsafe_code)]
    unsafe fn get_checked_state_for_layout(self) -> bool;
    #[allow(unsafe_code)]
    unsafe fn get_indeterminate_state_for_layout(self) -> bool;
}

#[allow(unsafe_code)]
unsafe fn get_raw_textinput_value(input: LayoutJS<HTMLInputElement>) -> DOMString {
    (*input.unsafe_get()).textinput.borrow_for_layout().get_content()
}

impl LayoutHTMLInputElementHelpers for LayoutJS<HTMLInputElement> {
    #[allow(unsafe_code)]
    unsafe fn get_value_for_layout(self) -> String {
        #[allow(unsafe_code)]
        unsafe fn get_raw_attr_value(input: LayoutJS<HTMLInputElement>, default: &str) -> String {
            let elem = input.upcast::<Element>();
            let value = (*elem.unsafe_get())
                .get_attr_val_for_layout(&ns!(), &atom!("value"))
                .unwrap_or(default);
            String::from(value)
        }

        match (*self.unsafe_get()).input_type.get() {
            InputType::InputCheckbox | InputType::InputRadio => String::new(),
            InputType::InputFile | InputType::InputImage => String::new(),
            InputType::InputButton => get_raw_attr_value(self, ""),
            InputType::InputSubmit => get_raw_attr_value(self, DEFAULT_SUBMIT_VALUE),
            InputType::InputReset => get_raw_attr_value(self, DEFAULT_RESET_VALUE),
            InputType::InputPassword => {
                let text = get_raw_textinput_value(self);
                if !text.is_empty() {
                    // The implementation of get_selection_for_layout expects a 1:1 mapping of chars.
                    text.chars().map(|_| '●').collect()
                } else {
                    String::from((*self.unsafe_get()).placeholder.borrow_for_layout().clone())
                }
            },
            _ => {
                let text = get_raw_textinput_value(self);
                if !text.is_empty() {
                    // The implementation of get_selection_for_layout expects a 1:1 mapping of chars.
                    String::from(text)
                } else {
                    String::from((*self.unsafe_get()).placeholder.borrow_for_layout().clone())
                }
            },
        }
    }

    #[allow(unrooted_must_root)]
    #[allow(unsafe_code)]
    unsafe fn get_size_for_layout(self) -> u32 {
        (*self.unsafe_get()).size.get()
    }

    #[allow(unrooted_must_root)]
    #[allow(unsafe_code)]
    unsafe fn get_selection_for_layout(self) -> Option<Range<isize>> {
        if !(*self.unsafe_get()).upcast::<Element>().focus_state() {
            return None;
        }

        // Use the raw textinput to get the index as long as we use a 1:1 char mapping
        // in get_value_for_layout.
        let raw = match (*self.unsafe_get()).input_type.get() {
            InputType::InputText |
            InputType::InputPassword => get_raw_textinput_value(self),
            _ => return None
        };
        let textinput = (*self.unsafe_get()).textinput.borrow_for_layout();
        let selection = textinput.get_absolute_selection_range();
        let begin_byte = selection.begin();
        let begin = search_index(begin_byte, raw.char_indices());
        let length = search_index(selection.length(), raw[begin_byte..].char_indices());
        Some(Range::new(begin, length))
    }

    #[allow(unrooted_must_root)]
    #[allow(unsafe_code)]
    unsafe fn get_checked_state_for_layout(self) -> bool {
        self.upcast::<Element>().get_state_for_layout().contains(IN_CHECKED_STATE)
    }

    #[allow(unrooted_must_root)]
    #[allow(unsafe_code)]
    unsafe fn get_indeterminate_state_for_layout(self) -> bool {
        self.upcast::<Element>().get_state_for_layout().contains(IN_INDETERMINATE_STATE)
    }
}

impl HTMLInputElementMethods for HTMLInputElement {
    // https://html.spec.whatwg.org/multipage/#dom-fe-disabled
    make_bool_getter!(Disabled, "disabled");

    // https://html.spec.whatwg.org/multipage/#dom-fe-disabled
    make_bool_setter!(SetDisabled, "disabled");

    // https://html.spec.whatwg.org/multipage/#dom-fae-form
    fn GetForm(&self) -> Option<Root<HTMLFormElement>> {
        self.form_owner()
    }

    // https://html.spec.whatwg.org/multipage/#dom-input-defaultchecked
    make_bool_getter!(DefaultChecked, "checked");

    // https://html.spec.whatwg.org/multipage/#dom-input-defaultchecked
    make_bool_setter!(SetDefaultChecked, "checked");

    // https://html.spec.whatwg.org/multipage/#dom-input-checked
    fn Checked(&self) -> bool {
        self.upcast::<Element>().state().contains(IN_CHECKED_STATE)
    }

    // https://html.spec.whatwg.org/multipage/#dom-input-checked
    fn SetChecked(&self, checked: bool) {
        self.update_checked_state(checked, true);
    }

    // https://html.spec.whatwg.org/multipage/#dom-input-readonly
    make_bool_getter!(ReadOnly, "readonly");

    // https://html.spec.whatwg.org/multipage/#dom-input-readonly
    make_bool_setter!(SetReadOnly, "readonly");

    // https://html.spec.whatwg.org/multipage/#dom-input-size
    make_uint_getter!(Size, "size", DEFAULT_INPUT_SIZE);

    // https://html.spec.whatwg.org/multipage/#dom-input-size
    make_limited_uint_setter!(SetSize, "size", DEFAULT_INPUT_SIZE);

    // https://html.spec.whatwg.org/multipage/#dom-input-type
    make_enumerated_getter!(Type,
                            "type",
                            "text",
                            ("hidden") | ("search") | ("tel") |
                            ("url") | ("email") | ("password") |
                            ("datetime") | ("date") | ("month") |
                            ("week") | ("time") | ("datetime-local") |
                            ("number") | ("range") | ("color") |
                            ("checkbox") | ("radio") | ("file") |
                            ("submit") | ("image") | ("reset") | ("button"));

    // https://html.spec.whatwg.org/multipage/#dom-input-type
    make_atomic_setter!(SetType, "type");

    // https://html.spec.whatwg.org/multipage/#dom-input-value
    fn Value(&self) -> DOMString {
        match self.get_value_mode() {
            ValueMode::Value => self.textinput.borrow().get_content(),
            ValueMode::Default => {
                self.upcast::<Element>()
                    .get_attribute(&ns!(), &atom!("value"))
                    .map_or(DOMString::from(""),
                            |a| DOMString::from(a.summarize().value))
            }
            ValueMode::DefaultOn => {
                self.upcast::<Element>()
                    .get_attribute(&ns!(), &atom!("value"))
                    .map_or(DOMString::from("on"),
                            |a| DOMString::from(a.summarize().value))
            }
            ValueMode::Filename => {
                // TODO: return C:\fakepath\<first of selected files> when a file is selected
                DOMString::from("")
            }
        }
    }

    // https://html.spec.whatwg.org/multipage/#dom-input-value
    fn SetValue(&self, value: DOMString) -> ErrorResult {
        match self.get_value_mode() {
            ValueMode::Value => {
                self.textinput.borrow_mut().set_content(value);
                self.value_dirty.set(true);
            }
            ValueMode::Default |
            ValueMode::DefaultOn => {
                self.upcast::<Element>().set_string_attribute(&atom!("value"), value);
            }
            ValueMode::Filename => {
                if value.is_empty() {
                    // TODO: empty list of selected files
                } else {
                    return Err(Error::InvalidState);
                }
            }
        }

        self.value_changed.set(true);
        self.upcast::<Node>().dirty(NodeDamage::OtherNodeDamage);
        Ok(())
    }

    // https://html.spec.whatwg.org/multipage/#dom-input-defaultvalue
    make_getter!(DefaultValue, "value");

    // https://html.spec.whatwg.org/multipage/#dom-input-defaultvalue
    make_setter!(SetDefaultValue, "value");

    // https://html.spec.whatwg.org/multipage/#attr-fe-name
    make_getter!(Name, "name");

    // https://html.spec.whatwg.org/multipage/#attr-fe-name
    make_atomic_setter!(SetName, "name");

    // https://html.spec.whatwg.org/multipage/#attr-input-placeholder
    make_getter!(Placeholder, "placeholder");

    // https://html.spec.whatwg.org/multipage/#attr-input-placeholder
    make_setter!(SetPlaceholder, "placeholder");

    // https://html.spec.whatwg.org/multipage/#dom-input-formaction
    make_url_or_base_getter!(FormAction, "formaction");

    // https://html.spec.whatwg.org/multipage/#dom-input-formaction
    make_setter!(SetFormAction, "formaction");

    // https://html.spec.whatwg.org/multipage/#dom-input-formenctype
    make_enumerated_getter!(FormEnctype,
                            "formenctype",
                            "application/x-www-form-urlencoded",
                            ("text/plain") | ("multipart/form-data"));

    // https://html.spec.whatwg.org/multipage/#dom-input-formenctype
    make_setter!(SetFormEnctype, "formenctype");

    // https://html.spec.whatwg.org/multipage/#dom-input-formmethod
    make_enumerated_getter!(FormMethod, "formmethod", "get", ("post") | ("dialog"));

    // https://html.spec.whatwg.org/multipage/#dom-input-formmethod
    make_setter!(SetFormMethod, "formmethod");

    // https://html.spec.whatwg.org/multipage/#dom-input-formtarget
    make_getter!(FormTarget, "formtarget");

    // https://html.spec.whatwg.org/multipage/#dom-input-formtarget
    make_setter!(SetFormTarget, "formtarget");

    // https://html.spec.whatwg.org/multipage/#attr-fs-formnovalidate
    make_bool_getter!(FormNoValidate, "formnovalidate");

    // https://html.spec.whatwg.org/multipage/#attr-fs-formnovalidate
    make_bool_setter!(SetFormNoValidate, "formnovalidate");

    // https://html.spec.whatwg.org/multipage/#dom-input-maxlength
    make_int_getter!(MaxLength, "maxlength", DEFAULT_MAX_LENGTH);

    // https://html.spec.whatwg.org/multipage/#dom-input-maxlength
    make_limited_int_setter!(SetMaxLength, "maxlength", DEFAULT_MAX_LENGTH);

    // https://html.spec.whatwg.org/multipage/#dom-input-indeterminate
    fn Indeterminate(&self) -> bool {
        self.upcast::<Element>().state().contains(IN_INDETERMINATE_STATE)
    }

    // https://html.spec.whatwg.org/multipage/#dom-input-indeterminate
    fn SetIndeterminate(&self, val: bool) {
        self.upcast::<Element>().set_state(IN_INDETERMINATE_STATE, val)
    }

    // https://html.spec.whatwg.org/multipage/#dom-lfe-labels
    fn Labels(&self) -> Root<NodeList> {
        if self.type_() == atom!("hidden") {
            let window = window_from_node(self);
            NodeList::empty(&window)
        } else {
            self.upcast::<HTMLElement>().labels()
        }
    }

    // https://html.spec.whatwg.org/multipage/#dom-input-selectionstart
    fn SelectionStart(&self) -> u32 {
        let text_input = self.textinput.borrow();
        let selection_start = match text_input.selection_begin {
            Some(selection_begin_point) => {
                text_input.get_absolute_point_for_text_point(&selection_begin_point)
            },
            None => text_input.get_absolute_insertion_point()
        };

        selection_start as u32
    }

    // https://html.spec.whatwg.org/multipage/#dom-textarea/input-selectionstart
    fn SetSelectionStart(&self, start: u32) {
        self.set_selection_range(start, self.SelectionEnd(), &self.selection_direction.get());
    }

    // https://html.spec.whatwg.org/multipage/#dom-textarea/input-selectionend
    fn SelectionEnd(&self) -> u32 {
        let text_input = self.textinput.borrow();
        text_input.get_absolute_insertion_point() as u32
    }

    // https://html.spec.whatwg.org/multipage/#dom-textarea/input-selectionend
    fn SetSelectionEnd(&self, end: u32) {
        self.set_selection_range(self.SelectionStart(), end, &self.selection_direction.get());
    }

    // https://html.spec.whatwg.org/multipage/#dom-textarea/input-selectiondirection
    fn SelectionDirection(&self) -> DOMString {
        match self.selection_direction.get() {
            SelectionDirection::Forward => DOMString::from("forward"),
            SelectionDirection::Backward => DOMString::from("backward"),
            SelectionDirection::None => DOMString::from("none"),
        }
    }

    // https://html.spec.whatwg.org/multipage/#dom-textarea/input-selectiondirection
    fn SetSelectionDirection(&self, direction: DOMString) {
        self.SetSelectionRange(self.SelectionStart(), self.SelectionEnd(), Some(direction));
    }

    // https://html.spec.whatwg.org/multipage/#dom-textarea/input-setselectionrange
    fn SetSelectionRange(&self, start: u32, end: u32, direction: Option<DOMString>) {
        let selection_direction = match direction {
            Some(selection_direction) => {
                match &*selection_direction {
                    "forward" => SelectionDirection::Forward,
                    "backward" => SelectionDirection::Backward,
                    _ => SelectionDirection::None,
                }
            },
            None => SelectionDirection::None,
        };
        self.set_selection_range(start, end, &selection_direction);
    }
}


#[allow(unsafe_code)]
fn broadcast_radio_checked(broadcaster: &HTMLInputElement, group: Option<&Atom>) {
    match group {
        None | Some(&atom!("")) => {
            // Radio input elements with a missing or empty name are alone in their
            // own group.
            return;
        },
        _ => {},
    }

    //TODO: if not in document, use root ancestor instead of document
    let owner = broadcaster.form_owner();
    let doc = document_from_node(broadcaster);

    // This function is a workaround for lifetime constraint difficulties.
    fn do_broadcast(doc_node: &Node, broadcaster: &HTMLInputElement,
                        owner: Option<&HTMLFormElement>, group: Option<&Atom>) {
        let iter = doc_node.query_selector_iter(DOMString::from("input[type=radio]")).unwrap()
                .filter_map(Root::downcast::<HTMLInputElement>)
                .filter(|r| in_same_group(r.r(), owner, group) && broadcaster != r.r());
        for ref r in iter {
            if r.Checked() {
                r.SetChecked(false);
            }
        }
    }

    do_broadcast(doc.upcast(), broadcaster, owner.r(), group)
}

// https://html.spec.whatwg.org/multipage/#radio-button-group
fn in_same_group(other: &HTMLInputElement, owner: Option<&HTMLFormElement>,
                 group: Option<&Atom>) -> bool {
    other.input_type.get() == InputType::InputRadio &&
    // TODO Both a and b are in the same home subtree.
    other.form_owner().r() == owner &&
    match (other.get_radio_group_name(), group) {
        (Some(ref s1), Some(s2)) => compatibility_caseless_match_str(s1, s2) && s2 != &atom!(""),
        _ => false
    }
}

impl HTMLInputElement {
    fn radio_group_updated(&self, group: Option<&Atom>) {
        if self.Checked() {
            broadcast_radio_checked(self, group);
        }
    }

    /// https://html.spec.whatwg.org/multipage/#constructing-the-form-data-set
    /// Steps range from 3.1 to 3.7 which related to the HTMLInputElement
    pub fn get_form_datum(&self, submitter: Option<FormSubmitter>) -> Option<FormDatum> {
        // Step 3.2
        let ty = self.type_();
        // Step 3.4
        let name = self.Name();
        let is_submitter = match submitter {
            Some(FormSubmitter::InputElement(s)) => {
                self == s
            },
            _ => false
        };

        match ty {
            // Step 3.1: it's a button but it is not submitter.
            atom!("submit") | atom!("button") | atom!("reset") if !is_submitter => return None,
            // Step 3.1: it's the "Checkbox" or "Radio Button" and whose checkedness is false.
            atom!("radio") | atom!("checkbox") => if !self.Checked() || name.is_empty() {
                return None;
            },
            atom!("image") | atom!("file") => return None, // Unimplemented
            // Step 3.1: it's not the "Image Button" and doesn't have a name attribute.
            _ => if name.is_empty() {
                return None;
            }

        }

        // Step 3.6
        Some(FormDatum {
            ty: DOMString::from(&*ty), // FIXME(ajeffrey): Convert directly from Atoms to DOMStrings
            name: name,
            value: self.Value()
        })
    }

    // https://html.spec.whatwg.org/multipage/#radio-button-group
    fn get_radio_group_name(&self) -> Option<Atom> {
        //TODO: determine form owner
        self.upcast::<Element>()
            .get_attribute(&ns!(), &atom!("name"))
            .map(|name| name.value().as_atom().clone())
    }

    fn update_checked_state(&self, checked: bool, dirty: bool) {
        self.upcast::<Element>().set_state(IN_CHECKED_STATE, checked);

        if dirty {
            self.checked_changed.set(true);
        }

        if self.input_type.get() == InputType::InputRadio && checked {
            broadcast_radio_checked(self,
                                    self.get_radio_group_name().as_ref());
        }

        self.upcast::<Node>().dirty(NodeDamage::OtherNodeDamage);
        //TODO: dispatch change event
    }

    pub fn get_indeterminate_state(&self) -> bool {
        self.Indeterminate()
    }

    // https://html.spec.whatwg.org/multipage/#concept-fe-mutable
    fn mutable(&self) -> bool {
        // https://html.spec.whatwg.org/multipage/#the-input-element:concept-fe-mutable
        // https://html.spec.whatwg.org/multipage/#the-readonly-attribute:concept-fe-mutable
        !(self.upcast::<Element>().disabled_state() || self.ReadOnly())
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

        self.SetValue(self.DefaultValue())
            .expect("Failed to reset input value to default.");
        self.value_dirty.set(false);
        self.value_changed.set(false);
        self.upcast::<Node>().dirty(NodeDamage::OtherNodeDamage);
    }
}

impl VirtualMethods for HTMLInputElement {
    fn super_type(&self) -> Option<&VirtualMethods> {
        Some(self.upcast::<HTMLElement>() as &VirtualMethods)
    }

    fn attribute_mutated(&self, attr: &Attr, mutation: AttributeMutation) {
        self.super_type().unwrap().attribute_mutated(attr, mutation);

        match attr.local_name() {
            &atom!("disabled") => {
                let disabled_state = match mutation {
                    AttributeMutation::Set(None) => true,
                    AttributeMutation::Set(Some(_)) => {
                       // Input was already disabled before.
                       return;
                    },
                    AttributeMutation::Removed => false,
                };
                let el = self.upcast::<Element>();
                el.set_disabled_state(disabled_state);
                el.set_enabled_state(!disabled_state);
                el.check_ancestors_disabled_state_for_form_control();
            },
            &atom!("checked") if !self.checked_changed.get() => {
                let checked_state = match mutation {
                    AttributeMutation::Set(None) => true,
                    AttributeMutation::Set(Some(_)) => {
                       // Input was already checked before.
                       return;
                    },
                    AttributeMutation::Removed => false,
                };
                self.update_checked_state(checked_state, false);
            },
            &atom!("size") => {
                let size = mutation.new_value(attr).map(|value| {
                    value.as_uint()
                });
                self.size.set(size.unwrap_or(DEFAULT_INPUT_SIZE));
            }
            &atom!("type") => {
                match mutation {
                    AttributeMutation::Set(_) => {
                        let new_type = match attr.value().as_atom() {
                            &atom!("button") => InputType::InputButton,
                            &atom!("submit") => InputType::InputSubmit,
                            &atom!("reset") => InputType::InputReset,
                            &atom!("file") => InputType::InputFile,
                            &atom!("radio") => InputType::InputRadio,
                            &atom!("checkbox") => InputType::InputCheckbox,
                            &atom!("password") => InputType::InputPassword,
                            _ => InputType::InputText,
                        };

                        // https://html.spec.whatwg.org/multipage/#input-type-change
                        let (old_value_mode, old_idl_value) = (self.get_value_mode(), self.Value());
                        self.input_type.set(new_type);
                        let new_value_mode = self.get_value_mode();

                        match (&old_value_mode, old_idl_value.is_empty(), new_value_mode) {

                            // Step 1
                            (&ValueMode::Value, false, ValueMode::Default) |
                            (&ValueMode::Value, false, ValueMode::DefaultOn) => {
                                self.SetValue(old_idl_value)
                                    .expect("Failed to set input value on type change to a default ValueMode.");
                            }

                            // Step 2
                            (_, _, ValueMode::Value) if old_value_mode != ValueMode::Value => {
                                self.SetValue(self.upcast::<Element>()
                                                  .get_attribute(&ns!(), &atom!("value"))
                                                  .map_or(DOMString::from(""),
                                                          |a| DOMString::from(a.summarize().value)))
                                    .expect("Failed to set input value on type change to ValueMode::Value.");
                                self.value_dirty.set(false);
                            }

                            // Step 3
                            (_, _, ValueMode::Filename) if old_value_mode != ValueMode::Filename => {
                                self.SetValue(DOMString::from(""))
                                    .expect("Failed to set input value on type change to ValueMode::Filename.");
                            }
                            _ => {}
                        }

                        // Step 5
                        if new_type == InputType::InputRadio {
                            self.radio_group_updated(
                                self.get_radio_group_name().as_ref());
                        }

                        // TODO: Step 6 - value sanitization
                    },
                    AttributeMutation::Removed => {
                        if self.input_type.get() == InputType::InputRadio {
                            broadcast_radio_checked(
                                self,
                                self.get_radio_group_name().as_ref());
                        }
                        self.input_type.set(InputType::InputText);
                    }
                }
            },
            &atom!("value") if !self.value_changed.get() => {
                let value = mutation.new_value(attr).map(|value| (**value).to_owned());
                self.textinput.borrow_mut().set_content(
                    value.map_or(DOMString::new(), DOMString::from));
            },
            &atom!("name") if self.input_type.get() == InputType::InputRadio => {
                self.radio_group_updated(
                    mutation.new_value(attr).as_ref().map(|name| name.as_atom()));
            },
            &atom!("maxlength") => {
                match *attr.value() {
                    AttrValue::Int(_, value) => {
                        if value < 0 {
                            self.textinput.borrow_mut().max_length = None
                        } else {
                            self.textinput.borrow_mut().max_length = Some(value as usize)
                        }
                    },
                    _ => panic!("Expected an AttrValue::Int"),
                }
            }
            &atom!("placeholder") => {
                // FIXME(ajeffrey): Should we do in-place mutation of the placeholder?
                let mut placeholder = self.placeholder.borrow_mut();
                placeholder.clear();
                if let AttributeMutation::Set(_) = mutation {
                    placeholder.extend(
                        attr.value().chars().filter(|&c| c != '\n' && c != '\r'));
                }
            },
            _ => {},
        }
    }

    fn parse_plain_attribute(&self, name: &Atom, value: DOMString) -> AttrValue {
        match name {
            &atom!("name") => AttrValue::from_atomic(value),
            &atom!("size") => AttrValue::from_limited_u32(value, DEFAULT_INPUT_SIZE),
            &atom!("type") => AttrValue::from_atomic(value),
            &atom!("maxlength") => AttrValue::from_limited_i32(value, DEFAULT_MAX_LENGTH),
            _ => self.super_type().unwrap().parse_plain_attribute(name, value),
        }
    }

    fn bind_to_tree(&self, tree_in_doc: bool) {
        if let Some(ref s) = self.super_type() {
            s.bind_to_tree(tree_in_doc);
        }

        self.upcast::<Element>().check_ancestors_disabled_state_for_form_control();
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

    fn handle_event(&self, event: &Event) {
        if let Some(s) = self.super_type() {
            s.handle_event(event);
        }

        if event.type_() == atom!("click") && !event.DefaultPrevented() {
            // TODO: Dispatch events for non activatable inputs
            // https://html.spec.whatwg.org/multipage/#common-input-element-events

            //TODO: set the editing position for text inputs

            document_from_node(self).request_focus(self.upcast());
        } else if event.type_() == atom!("keydown") && !event.DefaultPrevented() &&
            (self.input_type.get() == InputType::InputText ||
             self.input_type.get() == InputType::InputPassword) {
                if let Some(keyevent) = event.downcast::<KeyboardEvent>() {
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

                            if event.IsTrusted() {
                                ChangeEventRunnable::send(self.upcast::<Node>());
                            }

                            self.upcast::<Node>().dirty(NodeDamage::OtherNodeDamage);
                            event.PreventDefault();
                        }
                        RedrawSelection => {
                            self.upcast::<Node>().dirty(NodeDamage::OtherNodeDamage);
                            event.PreventDefault();
                        }
                        Nothing => (),
                    }
                }
        }
    }
}

impl FormControl for HTMLInputElement {}

impl Validatable for HTMLInputElement {}

impl Activatable for HTMLInputElement {
    fn as_element(&self) -> &Element {
        self.upcast()
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
                    let doc_node = doc.upcast::<Node>();
                    let group = self.get_radio_group_name();;

                    // Safe since we only manipulate the DOM tree after finding an element
                    let checked_member = doc_node.query_selector_iter(DOMString::from("input[type=radio]"))
                            .unwrap()
                            .filter_map(Root::downcast::<HTMLInputElement>)
                            .find(|r| {
                                in_same_group(r.r(), owner.r(), group.as_ref()) &&
                                r.Checked()
                            });
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
                    let name = self.get_radio_group_name();
                    match cache.checked_radio.r() {
                        Some(o) => {
                            // Avoiding iterating through the whole tree here, instead
                            // we can check if the conditions for radio group siblings apply
                            if name == o.get_radio_group_name() && // TODO should be compatibility caseless
                               self.form_owner() == o.form_owner() &&
                               // TODO Both a and b are in the same home subtree
                               o.input_type.get() == InputType::InputRadio {
                                    o.SetChecked(true);
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
                        o.submit(SubmittedFrom::NotFromFormSubmitMethod,
                                 FormSubmitter::InputElement(self.clone()))
                    });
                }
            },
            InputType::InputReset => {
                // https://html.spec.whatwg.org/multipage/#reset-button-state-(type=reset):activation-behavior
                // FIXME (Manishearth): support document owners (needs ability to get parent browsing context)
                if self.mutable() /* and document owner is fully active */ {
                    self.form_owner().map(|o| {
                        o.reset(ResetFrom::NotFromFormResetMethod)
                    });
                }
            },
            InputType::InputCheckbox | InputType::InputRadio => {
                // https://html.spec.whatwg.org/multipage/#checkbox-state-(type=checkbox):activation-behavior
                // https://html.spec.whatwg.org/multipage/#radio-button-state-(type=radio):activation-behavior
                if self.mutable() {
                    let target = self.upcast::<EventTarget>();
                    target.fire_event("input",
                                      EventBubbles::Bubbles,
                                      EventCancelable::NotCancelable);
                    target.fire_event("change",
                                      EventBubbles::Bubbles,
                                      EventCancelable::NotCancelable);
                }
            },
            _ => ()
        }
    }

    // https://html.spec.whatwg.org/multipage/#implicit-submission
    #[allow(unsafe_code)]
    fn implicit_submission(&self, ctrlKey: bool, shiftKey: bool, altKey: bool, metaKey: bool) {
        let doc = document_from_node(self);
        let node = doc.upcast::<Node>();
        let owner = self.form_owner();
        let form = match owner {
            None => return,
            Some(ref f) => f
        };

        if self.upcast::<Element>().click_in_progress() {
            return;
        }
        let submit_button;
        submit_button = node.query_selector_iter(DOMString::from("input[type=submit]")).unwrap()
            .filter_map(Root::downcast::<HTMLInputElement>)
            .find(|r| r.form_owner() == owner);
        match submit_button {
            Some(ref button) => {
                if button.is_instance_activatable() {
                    synthetic_click_activation(button.as_element(),
                                               ctrlKey,
                                               shiftKey,
                                               altKey,
                                               metaKey,
                                               ActivationSource::NotFromClick)
                }
            }
            None => {
                let inputs = node.query_selector_iter(DOMString::from("input")).unwrap()
                    .filter_map(Root::downcast::<HTMLInputElement>)
                    .filter(|input| {
                        input.form_owner() == owner && match input.type_() {
                            atom!("text") | atom!("search") | atom!("url") | atom!("tel") |
                            atom!("email") | atom!("password") | atom!("datetime") |
                            atom!("date") | atom!("month") | atom!("week") | atom!("time") |
                            atom!("datetime-local") | atom!("number")
                              => true,
                            _ => false
                        }
                    });

                if inputs.skip(1).next().is_some() {
                    // lazily test for > 1 submission-blocking inputs
                    return;
                }
                form.submit(SubmittedFrom::NotFromFormSubmitMethod,
                            FormSubmitter::FormElement(form.r()));
            }
        }
    }
}

pub struct ChangeEventRunnable {
    element: Trusted<Node>,
}

impl ChangeEventRunnable {
    pub fn send(node: &Node) {
        let window = window_from_node(node);
        let window = window.r();
        let chan = window.user_interaction_task_source();
        let handler = Trusted::new(node, chan.clone());
        let dispatcher = ChangeEventRunnable {
            element: handler,
        };
        let _ = chan.send(CommonScriptMsg::RunnableMsg(InputEvent, box dispatcher));
    }
}

impl Runnable for ChangeEventRunnable {
    fn handler(self: Box<ChangeEventRunnable>) {
        let target = self.element.root();
        let window = window_from_node(target.r());
        let window = window.r();
        let event = Event::new(GlobalRef::Window(window),
                               atom!("input"),
                               EventBubbles::Bubbles,
                               EventCancelable::NotCancelable);
        target.upcast::<EventTarget>().dispatch_event(&event);
    }
}

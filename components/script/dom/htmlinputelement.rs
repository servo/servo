/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use caseless::compatibility_caseless_match_str;
use dom::activation::{Activatable, ActivationSource, synthetic_click_activation};
use dom::attr::Attr;
use dom::bindings::cell::DOMRefCell;
use dom::bindings::codegen::Bindings::EventBinding::EventMethods;
use dom::bindings::codegen::Bindings::FileListBinding::FileListMethods;
use dom::bindings::codegen::Bindings::HTMLInputElementBinding;
use dom::bindings::codegen::Bindings::HTMLInputElementBinding::HTMLInputElementMethods;
use dom::bindings::codegen::Bindings::KeyboardEventBinding::KeyboardEventMethods;
use dom::bindings::codegen::Bindings::MouseEventBinding::MouseEventMethods;
use dom::bindings::codegen::Bindings::WindowBinding::WindowMethods;
use dom::bindings::error::{Error, ErrorResult};
use dom::bindings::inheritance::Castable;
use dom::bindings::js::{JS, LayoutJS, MutNullableJS, Root, RootedReference};
use dom::bindings::str::DOMString;
use dom::document::Document;
use dom::element::{AttributeMutation, Element, LayoutElementHelpers, RawLayoutElementHelpers};
use dom::event::{Event, EventBubbles, EventCancelable};
use dom::eventtarget::EventTarget;
use dom::file::File;
use dom::filelist::FileList;
use dom::globalscope::GlobalScope;
use dom::htmlelement::HTMLElement;
use dom::htmlfieldsetelement::HTMLFieldSetElement;
use dom::htmlformelement::{FormControl, FormDatum, FormDatumValue, FormSubmitter, HTMLFormElement};
use dom::htmlformelement::{ResetFrom, SubmittedFrom};
use dom::keyboardevent::KeyboardEvent;
use dom::mouseevent::MouseEvent;
use dom::node::{Node, NodeDamage, UnbindContext};
use dom::node::{document_from_node, window_from_node};
use dom::nodelist::NodeList;
use dom::validation::Validatable;
use dom::validitystate::ValidationFlags;
use dom::virtualmethods::VirtualMethods;
use dom_struct::dom_struct;
use html5ever::{LocalName, Prefix};
use ipc_channel::ipc::{self, IpcSender};
use mime_guess;
use net_traits::{CoreResourceMsg, IpcSend};
use net_traits::blob_url_store::get_blob_origin;
use net_traits::filemanager_thread::{FileManagerThreadMsg, FilterPattern};
use script_layout_interface::rpc::TextIndexResponse;
use script_traits::ScriptMsg as ConstellationMsg;
use servo_atoms::Atom;
use std::borrow::ToOwned;
use std::cell::Cell;
use std::ops::Range;
use style::attr::AttrValue;
use style::element_state::*;
use style::str::split_commas;
use textinput::{SelectionDirection, TextInput};
use textinput::KeyReaction::{DispatchInput, Nothing, RedrawSelection, TriggerDefaultAction};
use textinput::Lines::Single;

const DEFAULT_SUBMIT_VALUE: &'static str = "Submit";
const DEFAULT_RESET_VALUE: &'static str = "Reset";
const PASSWORD_REPLACEMENT_CHAR: char = '●';

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

#[dom_struct]
pub struct HTMLInputElement {
    htmlelement: HTMLElement,
    input_type: Cell<InputType>,
    checked_changed: Cell<bool>,
    placeholder: DOMRefCell<DOMString>,
    value_changed: Cell<bool>,
    size: Cell<u32>,
    maxlength: Cell<i32>,
    minlength: Cell<i32>,
    #[ignore_heap_size_of = "#7193"]
    textinput: DOMRefCell<TextInput<IpcSender<ConstellationMsg>>>,
    activation_state: DOMRefCell<InputActivationState>,
    // https://html.spec.whatwg.org/multipage/#concept-input-value-dirty-flag
    value_dirty: Cell<bool>,

    filelist: MutNullableJS<FileList>,
    form_owner: MutNullableJS<HTMLFormElement>,
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
static DEFAULT_MIN_LENGTH: i32 = -1;

impl HTMLInputElement {
    fn new_inherited(local_name: LocalName, prefix: Option<Prefix>, document: &Document) -> HTMLInputElement {
        let chan = document.window().upcast::<GlobalScope>().constellation_chan().clone();
        HTMLInputElement {
            htmlelement:
                HTMLElement::new_inherited_with_state(IN_ENABLED_STATE | IN_READ_WRITE_STATE,
                                                      local_name, prefix, document),
            input_type: Cell::new(InputType::InputText),
            placeholder: DOMRefCell::new(DOMString::new()),
            checked_changed: Cell::new(false),
            value_changed: Cell::new(false),
            maxlength: Cell::new(DEFAULT_MAX_LENGTH),
            minlength: Cell::new(DEFAULT_MIN_LENGTH),
            size: Cell::new(DEFAULT_INPUT_SIZE),
            textinput: DOMRefCell::new(TextInput::new(Single,
                                                      DOMString::new(),
                                                      chan,
                                                      None,
                                                      None,
                                                      SelectionDirection::None)),
            activation_state: DOMRefCell::new(InputActivationState::new()),
            value_dirty: Cell::new(false),
            filelist: MutNullableJS::new(None),
            form_owner: Default::default(),
        }
    }

    #[allow(unrooted_must_root)]
    pub fn new(local_name: LocalName,
               prefix: Option<Prefix>,
               document: &Document) -> Root<HTMLInputElement> {
        Node::reflect_node(box HTMLInputElement::new_inherited(local_name, prefix, document),
                           document,
                           HTMLInputElementBinding::Wrap)
    }

    pub fn type_(&self) -> Atom {
        self.upcast::<Element>()
            .get_attribute(&ns!(), &local_name!("type"))
            .map_or_else(|| atom!(""), |a| a.value().as_atom().to_owned())
    }

    // https://html.spec.whatwg.org/multipage/#input-type-attr-summary
    fn value_mode(&self) -> ValueMode {
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
}

pub trait LayoutHTMLInputElementHelpers {
    #[allow(unsafe_code)]
    unsafe fn value_for_layout(self) -> String;
    #[allow(unsafe_code)]
    unsafe fn size_for_layout(self) -> u32;
    #[allow(unsafe_code)]
    unsafe fn selection_for_layout(self) -> Option<Range<usize>>;
    #[allow(unsafe_code)]
    unsafe fn checked_state_for_layout(self) -> bool;
    #[allow(unsafe_code)]
    unsafe fn indeterminate_state_for_layout(self) -> bool;
}

#[allow(unsafe_code)]
unsafe fn get_raw_textinput_value(input: LayoutJS<HTMLInputElement>) -> DOMString {
    (*input.unsafe_get()).textinput.borrow_for_layout().get_content()
}

impl LayoutHTMLInputElementHelpers for LayoutJS<HTMLInputElement> {
    #[allow(unsafe_code)]
    unsafe fn value_for_layout(self) -> String {
        #[allow(unsafe_code)]
        unsafe fn get_raw_attr_value(input: LayoutJS<HTMLInputElement>, default: &str) -> String {
            let elem = input.upcast::<Element>();
            let value = (*elem.unsafe_get())
                .get_attr_val_for_layout(&ns!(), &local_name!("value"))
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
                    text.chars().map(|_| PASSWORD_REPLACEMENT_CHAR).collect()
                } else {
                    String::from((*self.unsafe_get()).placeholder.borrow_for_layout().clone())
                }
            },
            _ => {
                let text = get_raw_textinput_value(self);
                if !text.is_empty() {
                    String::from(text)
                } else {
                    String::from((*self.unsafe_get()).placeholder.borrow_for_layout().clone())
                }
            },
        }
    }

    #[allow(unrooted_must_root)]
    #[allow(unsafe_code)]
    unsafe fn size_for_layout(self) -> u32 {
        (*self.unsafe_get()).size.get()
    }

    #[allow(unrooted_must_root)]
    #[allow(unsafe_code)]
    unsafe fn selection_for_layout(self) -> Option<Range<usize>> {
        if !(*self.unsafe_get()).upcast::<Element>().focus_state() {
            return None;
        }

        let textinput = (*self.unsafe_get()).textinput.borrow_for_layout();

        match (*self.unsafe_get()).input_type.get() {
            InputType::InputPassword => {
                let text = get_raw_textinput_value(self);
                let sel = textinput.get_absolute_selection_range();

                // Translate indices from the raw value to indices in the replacement value.
                let char_start = text[.. sel.start].chars().count();
                let char_end = char_start + text[sel].chars().count();

                let bytes_per_char = PASSWORD_REPLACEMENT_CHAR.len_utf8();
                Some(char_start * bytes_per_char .. char_end * bytes_per_char)
            }
            InputType::InputText => Some(textinput.get_absolute_selection_range()),
            _ => None
        }
    }

    #[allow(unrooted_must_root)]
    #[allow(unsafe_code)]
    unsafe fn checked_state_for_layout(self) -> bool {
        self.upcast::<Element>().get_state_for_layout().contains(IN_CHECKED_STATE)
    }

    #[allow(unrooted_must_root)]
    #[allow(unsafe_code)]
    unsafe fn indeterminate_state_for_layout(self) -> bool {
        self.upcast::<Element>().get_state_for_layout().contains(IN_INDETERMINATE_STATE)
    }
}

impl HTMLInputElementMethods for HTMLInputElement {
    // https://html.spec.whatwg.org/multipage/#dom-input-accept
    make_getter!(Accept, "accept");

    // https://html.spec.whatwg.org/multipage/#dom-input-accept
    make_setter!(SetAccept, "accept");

    // https://html.spec.whatwg.org/multipage/#dom-input-alt
    make_getter!(Alt, "alt");

    // https://html.spec.whatwg.org/multipage/#dom-input-alt
    make_setter!(SetAlt, "alt");

    // https://html.spec.whatwg.org/multipage/#dom-input-dirName
    make_getter!(DirName, "dirname");

    // https://html.spec.whatwg.org/multipage/#dom-input-dirName
    make_setter!(SetDirName, "dirname");

    // https://html.spec.whatwg.org/multipage/#dom-fe-disabled
    make_bool_getter!(Disabled, "disabled");

    // https://html.spec.whatwg.org/multipage/#dom-fe-disabled
    make_bool_setter!(SetDisabled, "disabled");

    // https://html.spec.whatwg.org/multipage/#dom-fae-form
    fn GetForm(&self) -> Option<Root<HTMLFormElement>> {
        self.form_owner()
    }

    // https://html.spec.whatwg.org/multipage/#dom-input-files
    fn GetFiles(&self) -> Option<Root<FileList>> {
        match self.filelist.get() {
            Some(ref fl) => Some(fl.clone()),
            None => None,
        }
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
                            "hidden" | "search" | "tel" |
                            "url" | "email" | "password" |
                            "datetime" | "date" | "month" |
                            "week" | "time" | "datetime-local" |
                            "number" | "range" | "color" |
                            "checkbox" | "radio" | "file" |
                            "submit" | "image" | "reset" | "button");

    // https://html.spec.whatwg.org/multipage/#dom-input-type
    make_atomic_setter!(SetType, "type");

    // https://html.spec.whatwg.org/multipage/#dom-input-value
    fn Value(&self) -> DOMString {
        match self.value_mode() {
            ValueMode::Value => self.textinput.borrow().get_content(),
            ValueMode::Default => {
                self.upcast::<Element>()
                    .get_attribute(&ns!(), &local_name!("value"))
                    .map_or(DOMString::from(""),
                            |a| DOMString::from(a.summarize().value))
            }
            ValueMode::DefaultOn => {
                self.upcast::<Element>()
                    .get_attribute(&ns!(), &local_name!("value"))
                    .map_or(DOMString::from("on"),
                            |a| DOMString::from(a.summarize().value))
            }
            ValueMode::Filename => {
                let mut path = DOMString::from("");
                match self.filelist.get() {
                    Some(ref fl) => match fl.Item(0) {
                        Some(ref f) => {
                            path.push_str("C:\\fakepath\\");
                            path.push_str(f.name());
                            path
                        }
                        None => path,
                    },
                    None => path,
                }
            }
        }
    }

    // https://html.spec.whatwg.org/multipage/#dom-input-value
    fn SetValue(&self, value: DOMString) -> ErrorResult {
        match self.value_mode() {
            ValueMode::Value => {
                self.textinput.borrow_mut().set_content(value);
                self.value_dirty.set(true);
            }
            ValueMode::Default |
            ValueMode::DefaultOn => {
                self.upcast::<Element>().set_string_attribute(&local_name!("value"), value);
            }
            ValueMode::Filename => {
                if value.is_empty() {
                    let window = window_from_node(self);
                    let fl = FileList::new(&window, vec![]);
                    self.filelist.set(Some(&fl));
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

    // https://html.spec.whatwg.org/multipage/#dom-input-placeholder
    make_getter!(Placeholder, "placeholder");

    // https://html.spec.whatwg.org/multipage/#dom-input-placeholder
    make_setter!(SetPlaceholder, "placeholder");

    // https://html.spec.whatwg.org/multipage/#dom-input-formaction
    make_url_or_base_getter!(FormAction, "formaction");

    // https://html.spec.whatwg.org/multipage/#dom-input-formaction
    make_setter!(SetFormAction, "formaction");

    // https://html.spec.whatwg.org/multipage/#dom-input-formenctype
    make_enumerated_getter!(FormEnctype,
                            "formenctype",
                            "application/x-www-form-urlencoded",
                            "text/plain" | "multipart/form-data");

    // https://html.spec.whatwg.org/multipage/#dom-input-formenctype
    make_setter!(SetFormEnctype, "formenctype");

    // https://html.spec.whatwg.org/multipage/#dom-input-formmethod
    make_enumerated_getter!(FormMethod, "formmethod", "get", "post" | "dialog");

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

    // https://html.spec.whatwg.org/multipage/#dom-input-max
    make_getter!(Max, "max");

    // https://html.spec.whatwg.org/multipage/#dom-input-max
    make_setter!(SetMax, "max");

    // https://html.spec.whatwg.org/multipage/#dom-input-maxlength
    make_int_getter!(MaxLength, "maxlength", DEFAULT_MAX_LENGTH);

    // https://html.spec.whatwg.org/multipage/#dom-input-maxlength
    make_limited_int_setter!(SetMaxLength, "maxlength", DEFAULT_MAX_LENGTH);

    // https://html.spec.whatwg.org/multipage/#dom-input-minlength
    make_int_getter!(MinLength, "minlength", DEFAULT_MIN_LENGTH);

    // https://html.spec.whatwg.org/multipage/#dom-input-minlength
    make_limited_int_setter!(SetMinLength, "minlength", DEFAULT_MIN_LENGTH);

    // https://html.spec.whatwg.org/multipage/#dom-input-min
    make_getter!(Min, "min");

    // https://html.spec.whatwg.org/multipage/#dom-input-min
    make_setter!(SetMin, "min");

    // https://html.spec.whatwg.org/multipage/#dom-input-multiple
    make_bool_getter!(Multiple, "multiple");

    // https://html.spec.whatwg.org/multipage/#dom-input-multiple
    make_bool_setter!(SetMultiple, "multiple");

    // https://html.spec.whatwg.org/multipage/#dom-input-pattern
    make_getter!(Pattern, "pattern");

    // https://html.spec.whatwg.org/multipage/#dom-input-pattern
    make_setter!(SetPattern, "pattern");

    // https://html.spec.whatwg.org/multipage/#dom-input-required
    make_bool_getter!(Required, "required");

    // https://html.spec.whatwg.org/multipage/#dom-input-required
    make_bool_setter!(SetRequired, "required");

    // https://html.spec.whatwg.org/multipage/#dom-input-src
    make_url_getter!(Src, "src");

    // https://html.spec.whatwg.org/multipage/#dom-input-src
    make_url_setter!(SetSrc, "src");

    // https://html.spec.whatwg.org/multipage/#dom-input-step
    make_getter!(Step, "step");

    // https://html.spec.whatwg.org/multipage/#dom-input-step
    make_setter!(SetStep, "step");

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
        self.textinput.borrow().get_selection_start()
    }

    // https://html.spec.whatwg.org/multipage/#dom-textarea/input-selectionstart
    fn SetSelectionStart(&self, start: u32) {
        let selection_end = self.SelectionEnd();
        self.textinput.borrow_mut().set_selection_range(start, selection_end);
        self.upcast::<Node>().dirty(NodeDamage::OtherNodeDamage);
    }

    // https://html.spec.whatwg.org/multipage/#dom-textarea/input-selectionend
    fn SelectionEnd(&self) -> u32 {
        self.textinput.borrow().get_absolute_insertion_point() as u32
    }

    // https://html.spec.whatwg.org/multipage/#dom-textarea/input-selectionend
    fn SetSelectionEnd(&self, end: u32) {
        let selection_start = self.SelectionStart();
        self.textinput.borrow_mut().set_selection_range(selection_start, end);
        self.upcast::<Node>().dirty(NodeDamage::OtherNodeDamage);
    }

    // https://html.spec.whatwg.org/multipage/#dom-textarea/input-selectiondirection
    fn SelectionDirection(&self) -> DOMString {
        DOMString::from(self.textinput.borrow().selection_direction)
    }

    // https://html.spec.whatwg.org/multipage/#dom-textarea/input-selectiondirection
    fn SetSelectionDirection(&self, direction: DOMString) {
        self.textinput.borrow_mut().selection_direction = SelectionDirection::from(direction);
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
            &window);
        self.upcast::<Node>().dirty(NodeDamage::OtherNodeDamage);
    }

    // Select the files based on filepaths passed in,
    // enabled by dom.htmlinputelement.select_files.enabled,
    // used for test purpose.
    // check-tidy: no specs after this line
    fn SelectFiles(&self, paths: Vec<DOMString>) {
        if self.input_type.get() == InputType::InputFile {
            self.select_files(Some(paths));
        }
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
                .filter(|r| in_same_group(&r, owner, group) && broadcaster != &**r);
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
    match (other.radio_group_name(), group) {
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
    /// Steps range from 3.1 to 3.7 (specific to HTMLInputElement)
    pub fn form_datums(&self, submitter: Option<FormSubmitter>) -> Vec<FormDatum> {
        // 3.1: disabled state check is in get_unclean_dataset

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
            atom!("submit") | atom!("button") | atom!("reset") if !is_submitter => return vec![],
            // Step 3.1: it's the "Checkbox" or "Radio Button" and whose checkedness is false.
            atom!("radio") | atom!("checkbox") => if !self.Checked() || name.is_empty() {
                return vec![];
            },
            atom!("file") => {
                let mut datums = vec![];

                // Step 3.2-3.7
                let name = self.Name();
                let type_ = self.Type();

                match self.GetFiles() {
                    Some(fl) => {
                        for f in fl.iter_files() {
                            datums.push(FormDatum {
                                ty: type_.clone(),
                                name: name.clone(),
                                value: FormDatumValue::File(Root::from_ref(&f)),
                            });
                        }
                    }
                    None => {
                        datums.push(FormDatum {
                            // XXX(izgzhen): Spec says 'application/octet-stream' as the type,
                            // but this is _type_ of element rather than content right?
                            ty: type_.clone(),
                            name: name.clone(),
                            value: FormDatumValue::String(DOMString::from("")),
                        })
                    }
                }

                return datums;
            }
            atom!("image") => return vec![], // Unimplemented
            // Step 3.1: it's not the "Image Button" and doesn't have a name attribute.
            _ => if name.is_empty() {
                return vec![];
            }

        }

        // Step 3.9
        vec![FormDatum {
            ty: DOMString::from(&*ty), // FIXME(ajeffrey): Convert directly from Atoms to DOMStrings
            name: name,
            value: FormDatumValue::String(self.Value())
        }]
    }

    // https://html.spec.whatwg.org/multipage/#radio-button-group
    fn radio_group_name(&self) -> Option<Atom> {
        //TODO: determine form owner
        self.upcast::<Element>()
            .get_attribute(&ns!(), &local_name!("name"))
            .map(|name| name.value().as_atom().clone())
    }

    fn update_checked_state(&self, checked: bool, dirty: bool) {
        self.upcast::<Element>().set_state(IN_CHECKED_STATE, checked);

        if dirty {
            self.checked_changed.set(true);
        }

        if self.input_type.get() == InputType::InputRadio && checked {
            broadcast_radio_checked(self,
                                    self.radio_group_name().as_ref());
        }

        self.upcast::<Node>().dirty(NodeDamage::OtherNodeDamage);
        //TODO: dispatch change event
    }

    // https://html.spec.whatwg.org/multipage/#concept-fe-mutable
    fn is_mutable(&self) -> bool {
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

    fn update_placeholder_shown_state(&self) {
        match self.input_type.get() {
            InputType::InputText | InputType::InputPassword => {},
            _ => return,
        }
        let has_placeholder = !self.placeholder.borrow().is_empty();
        let has_value = !self.textinput.borrow().is_empty();
        let el = self.upcast::<Element>();
        el.set_placeholder_shown_state(has_placeholder && !has_value);
    }

    // https://html.spec.whatwg.org/multipage/#file-upload-state-(type=file)
    // Select files by invoking UI or by passed in argument
    fn select_files(&self, opt_test_paths: Option<Vec<DOMString>>) {
        let window = window_from_node(self);
        let origin = get_blob_origin(&window.get_url());
        let resource_threads = window.upcast::<GlobalScope>().resource_threads();

        let mut files: Vec<Root<File>> = vec![];
        let mut error = None;

        let filter = filter_from_accept(&self.Accept());
        let target = self.upcast::<EventTarget>();

        if self.Multiple() {
            let opt_test_paths = opt_test_paths.map(|paths| paths.iter().map(|p| p.to_string()).collect());

            let (chan, recv) = ipc::channel().expect("Error initializing channel");
            let msg = FileManagerThreadMsg::SelectFiles(filter, chan, origin, opt_test_paths);
            let _ = resource_threads.send(CoreResourceMsg::ToFileManager(msg)).unwrap();

            match recv.recv().expect("IpcSender side error") {
                Ok(selected_files) => {
                    for selected in selected_files {
                        files.push(File::new_from_selected(&window, selected));
                    }
                },
                Err(err) => error = Some(err),
            };
        } else {
            let opt_test_path = match opt_test_paths {
                Some(paths) => {
                    if paths.len() == 0 {
                        return;
                    } else {
                        Some(paths[0].to_string()) // neglect other paths
                    }
                }
                None => None,
            };

            let (chan, recv) = ipc::channel().expect("Error initializing channel");
            let msg = FileManagerThreadMsg::SelectFile(filter, chan, origin, opt_test_path);
            let _ = resource_threads.send(CoreResourceMsg::ToFileManager(msg)).unwrap();

            match recv.recv().expect("IpcSender side error") {
                Ok(selected) => {
                    files.push(File::new_from_selected(&window, selected));
                },
                Err(err) => error = Some(err),
            };
        }

        if let Some(err) = error {
            debug!("Input file select error: {:?}", err);
        } else {
            let filelist = FileList::new(&window, files);
            self.filelist.set(Some(&filelist));

            target.fire_bubbling_event(atom!("input"));
            target.fire_bubbling_event(atom!("change"));
        }
    }
}

impl VirtualMethods for HTMLInputElement {
    fn super_type(&self) -> Option<&VirtualMethods> {
        Some(self.upcast::<HTMLElement>() as &VirtualMethods)
    }

    fn attribute_mutated(&self, attr: &Attr, mutation: AttributeMutation) {
        self.super_type().unwrap().attribute_mutated(attr, mutation);

        match attr.local_name() {
            &local_name!("disabled") => {
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

                if self.input_type.get() == InputType::InputText {
                    let read_write = !(self.ReadOnly() || el.disabled_state());
                    el.set_read_write_state(read_write);
                }
            },
            &local_name!("checked") if !self.checked_changed.get() => {
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
            &local_name!("size") => {
                let size = mutation.new_value(attr).map(|value| {
                    value.as_uint()
                });
                self.size.set(size.unwrap_or(DEFAULT_INPUT_SIZE));
            }
            &local_name!("type") => {
                let el = self.upcast::<Element>();
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
                        let (old_value_mode, old_idl_value) = (self.value_mode(), self.Value());
                        self.input_type.set(new_type);

                        if new_type == InputType::InputText {
                            let read_write = !(self.ReadOnly() || el.disabled_state());
                            el.set_read_write_state(read_write);
                        } else {
                            el.set_read_write_state(false);
                        }

                        if new_type == InputType::InputFile {
                            let window = window_from_node(self);
                            let filelist = FileList::new(&window, vec![]);
                            self.filelist.set(Some(&filelist));
                        }

                        let new_value_mode = self.value_mode();

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
                                                  .get_attribute(&ns!(), &local_name!("value"))
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
                                self.radio_group_name().as_ref());
                        }

                        // TODO: Step 6 - value sanitization
                    },
                    AttributeMutation::Removed => {
                        if self.input_type.get() == InputType::InputRadio {
                            broadcast_radio_checked(
                                self,
                                self.radio_group_name().as_ref());
                        }
                        self.input_type.set(InputType::InputText);
                        let el = self.upcast::<Element>();

                        let read_write = !(self.ReadOnly() || el.disabled_state());
                        el.set_read_write_state(read_write);
                    }
                }

                self.update_placeholder_shown_state();
            },
            &local_name!("value") if !self.value_changed.get() => {
                let value = mutation.new_value(attr).map(|value| (**value).to_owned());
                self.textinput.borrow_mut().set_content(
                    value.map_or(DOMString::new(), DOMString::from));
                self.update_placeholder_shown_state();
            },
            &local_name!("name") if self.input_type.get() == InputType::InputRadio => {
                self.radio_group_updated(
                    mutation.new_value(attr).as_ref().map(|name| name.as_atom()));
            },
            &local_name!("maxlength") => {
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
            },
            &local_name!("minlength") => {
                match *attr.value() {
                    AttrValue::Int(_, value) => {
                        if value < 0 {
                            self.textinput.borrow_mut().min_length = None
                        } else {
                            self.textinput.borrow_mut().min_length = Some(value as usize)
                        }
                    },
                    _ => panic!("Expected an AttrValue::Int"),
                }
            },
            &local_name!("placeholder") => {
                {
                    let mut placeholder = self.placeholder.borrow_mut();
                    placeholder.clear();
                    if let AttributeMutation::Set(_) = mutation {
                        placeholder.extend(
                            attr.value().chars().filter(|&c| c != '\n' && c != '\r'));
                    }
                }
                self.update_placeholder_shown_state();
            },
            &local_name!("readonly") if self.input_type.get() == InputType::InputText => {
                let el = self.upcast::<Element>();
                match mutation {
                    AttributeMutation::Set(_) => {
                        el.set_read_write_state(false);
                    },
                    AttributeMutation::Removed => {
                        el.set_read_write_state(!el.disabled_state());
                    }
                }
            },
            &local_name!("form") => {
                self.form_attribute_mutated(mutation);
            },
            _ => {},
        }
    }

    fn parse_plain_attribute(&self, name: &LocalName, value: DOMString) -> AttrValue {
        match name {
            &local_name!("accept") => AttrValue::from_comma_separated_tokenlist(value.into()),
            &local_name!("name") => AttrValue::from_atomic(value.into()),
            &local_name!("size") => AttrValue::from_limited_u32(value.into(), DEFAULT_INPUT_SIZE),
            &local_name!("type") => AttrValue::from_atomic(value.into()),
            &local_name!("maxlength") => AttrValue::from_limited_i32(value.into(), DEFAULT_MAX_LENGTH),
            &local_name!("minlength") => AttrValue::from_limited_i32(value.into(), DEFAULT_MIN_LENGTH),
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
            if (self.input_type.get() == InputType::InputText ||
                self.input_type.get() == InputType::InputPassword) &&
                // Check if we display a placeholder. Layout doesn't know about this.
                !self.textinput.borrow().is_empty() {
                    if let Some(mouse_event) = event.downcast::<MouseEvent>() {
                        // dispatch_key_event (document.rs) triggers a click event when releasing
                        // the space key. There's no nice way to catch this so let's use this for
                        // now.
                        if !(mouse_event.ScreenX() == 0 && mouse_event.ScreenY() == 0 &&
                            mouse_event.GetRelatedTarget().is_none()) {
                                let window = window_from_node(self);
                                let translated_x = mouse_event.ClientX() + window.PageXOffset();
                                let translated_y = mouse_event.ClientY() + window.PageYOffset();
                                let TextIndexResponse(index) = window.text_index_query(
                                    self.upcast::<Node>().to_trusted_node_address(),
                                    translated_x,
                                    translated_y
                                );
                                if let Some(i) = index {
                                    self.textinput.borrow_mut().set_edit_point_index(i as usize);
                                    // trigger redraw
                                    self.upcast::<Node>().dirty(NodeDamage::OtherNodeDamage);
                                    event.PreventDefault();
                                }
                            }
                    }
                }
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
                            self.update_placeholder_shown_state();
                            self.upcast::<Node>().dirty(NodeDamage::OtherNodeDamage);
                            event.mark_as_handled();
                        }
                        RedrawSelection => {
                            self.upcast::<Node>().dirty(NodeDamage::OtherNodeDamage);
                            event.mark_as_handled();
                        }
                        Nothing => (),
                    }
                }
        } else if event.type_() == atom!("keypress") && !event.DefaultPrevented() &&
            (self.input_type.get() == InputType::InputText ||
             self.input_type.get() == InputType::InputPassword) {
                if event.IsTrusted() {
                    let window = window_from_node(self);
                    let _ = window.user_interaction_task_source()
                                  .queue_event(&self.upcast(),
                                               atom!("input"),
                                               EventBubbles::Bubbles,
                                               EventCancelable::NotCancelable,
                                               &window);
                }
            }
    }
}

impl FormControl for HTMLInputElement {
    fn form_owner(&self) -> Option<Root<HTMLFormElement>> {
        self.form_owner.get()
    }

    fn set_form_owner(&self, form: Option<&HTMLFormElement>) {
        self.form_owner.set(form);
    }

    fn to_element<'a>(&'a self) -> &'a Element {
        self.upcast::<Element>()
    }
}

impl Validatable for HTMLInputElement {
    fn is_instance_validatable(&self) -> bool {
        // https://html.spec.whatwg.org/multipage/#candidate-for-constraint-validation
        true
    }
    fn validate(&self, _validate_flags: ValidationFlags) -> bool {
        // call stub methods defined in validityState.rs file here according to the flags set in validate_flags
        true
    }
}

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
            InputType::InputSubmit | InputType::InputReset | InputType::InputFile
            | InputType::InputCheckbox | InputType::InputRadio => self.is_mutable(),
            _ => false
        }
    }

    // https://html.spec.whatwg.org/multipage/#run-pre-click-activation-steps
    #[allow(unsafe_code)]
    fn pre_click_activation(&self) {
        let mut cache = self.activation_state.borrow_mut();
        let ty = self.input_type.get();
        cache.old_type = ty;
        cache.was_mutable = self.is_mutable();
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
                    let group = self.radio_group_name();;

                    // Safe since we only manipulate the DOM tree after finding an element
                    let checked_member = doc_node.query_selector_iter(DOMString::from("input[type=radio]"))
                            .unwrap()
                            .filter_map(Root::downcast::<HTMLInputElement>)
                            .find(|r| {
                                in_same_group(&*r, owner.r(), group.as_ref()) &&
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
                    match cache.checked_radio.r() {
                        Some(o) => {
                            // Avoiding iterating through the whole tree here, instead
                            // we can check if the conditions for radio group siblings apply
                            if in_same_group(&o, self.form_owner().r(), self.radio_group_name().as_ref()) {
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
        if self.activation_state.borrow().old_type != ty || !self.is_mutable() {
            // Type changed or input is immutable, abandon ship
            // https://www.w3.org/Bugs/Public/show_bug.cgi?id=27414
            return;
        }
        match ty {
            InputType::InputSubmit => {
                // https://html.spec.whatwg.org/multipage/#submit-button-state-(type=submit):activation-behavior
                // FIXME (Manishearth): support document owners (needs ability to get parent browsing context)
                // Check if document owner is fully active
                self.form_owner().map(|o| {
                    o.submit(SubmittedFrom::NotFromForm,
                             FormSubmitter::InputElement(self.clone()))
                });
            },
            InputType::InputReset => {
                // https://html.spec.whatwg.org/multipage/#reset-button-state-(type=reset):activation-behavior
                // FIXME (Manishearth): support document owners (needs ability to get parent browsing context)
                // Check if document owner is fully active
                self.form_owner().map(|o| {
                    o.reset(ResetFrom::NotFromForm)
                });
            },
            InputType::InputCheckbox | InputType::InputRadio => {
                // https://html.spec.whatwg.org/multipage/#checkbox-state-(type=checkbox):activation-behavior
                // https://html.spec.whatwg.org/multipage/#radio-button-state-(type=radio):activation-behavior
                // Check if document owner is fully active
                let target = self.upcast::<EventTarget>();
                target.fire_bubbling_event(atom!("input"));
                target.fire_bubbling_event(atom!("change"));
            },
            InputType::InputFile => self.select_files(None),
            _ => ()
        }
    }

    // https://html.spec.whatwg.org/multipage/#implicit-submission
    #[allow(unsafe_code)]
    fn implicit_submission(&self, ctrl_key: bool, shift_key: bool, alt_key: bool, meta_key: bool) {
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
                                               ctrl_key,
                                               shift_key,
                                               alt_key,
                                               meta_key,
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
                form.submit(SubmittedFrom::NotFromForm,
                            FormSubmitter::FormElement(&form));
            }
        }
    }
}

// https://html.spec.whatwg.org/multipage/#attr-input-accept
fn filter_from_accept(s: &DOMString) -> Vec<FilterPattern> {
    let mut filter = vec![];
    for p in split_commas(s) {
        if let Some('.') = p.chars().nth(0) {
            filter.push(FilterPattern(p[1..].to_string()));
        } else {
            if let Some(exts) = mime_guess::get_mime_extensions_str(p) {
                for ext in exts {
                    filter.push(FilterPattern(ext.to_string()));
                }
            }
        }
    }

    filter
}

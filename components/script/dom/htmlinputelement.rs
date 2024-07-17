/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::borrow::Cow;
use std::cell::Cell;
use std::ops::Range;
use std::ptr::NonNull;
use std::{f64, ptr};

use chrono::naive::{NaiveDate, NaiveDateTime};
use chrono::{DateTime, Datelike, Weekday};
use dom_struct::dom_struct;
use embedder_traits::{FilterPattern, InputMethodType};
use encoding_rs::Encoding;
use html5ever::{local_name, namespace_url, ns, LocalName, Prefix};
use js::jsapi::{
    ClippedTime, DateGetMsecSinceEpoch, Handle, JSObject, JS_ClearPendingException, NewDateObject,
    NewUCRegExpObject, ObjectIsDate, RegExpFlag_Unicode, RegExpFlags,
};
use js::jsval::UndefinedValue;
use js::rust::jsapi_wrapped::{ExecuteRegExpNoStatics, ObjectIsRegExp};
use js::rust::{HandleObject, MutableHandleObject};
use net_traits::blob_url_store::get_blob_origin;
use net_traits::filemanager_thread::FileManagerThreadMsg;
use net_traits::{CoreResourceMsg, IpcSend};
use profile_traits::ipc;
use script_traits::ScriptToConstellationChan;
use servo_atoms::Atom;
use style::attr::AttrValue;
use style::str::{split_commas, str_join};
use style_traits::dom::ElementState;
use unicode_bidi::{bidi_class, BidiClass};
use url::Url;

use crate::dom::activation::Activatable;
use crate::dom::attr::Attr;
use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::ElementBinding::ElementMethods;
use crate::dom::bindings::codegen::Bindings::EventBinding::EventMethods;
use crate::dom::bindings::codegen::Bindings::FileListBinding::FileListMethods;
use crate::dom::bindings::codegen::Bindings::HTMLFormElementBinding::SelectionMode;
use crate::dom::bindings::codegen::Bindings::HTMLInputElementBinding::HTMLInputElementMethods;
use crate::dom::bindings::codegen::Bindings::NodeBinding::{GetRootNodeOptions, NodeMethods};
use crate::dom::bindings::error::{Error, ErrorResult};
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::reflector::DomObject;
use crate::dom::bindings::root::{DomRoot, LayoutDom, MutNullableDom};
use crate::dom::bindings::str::{DOMString, USVString};
use crate::dom::compositionevent::CompositionEvent;
use crate::dom::document::Document;
use crate::dom::element::{AttributeMutation, Element, LayoutElementHelpers};
use crate::dom::event::{Event, EventBubbles, EventCancelable};
use crate::dom::eventtarget::EventTarget;
use crate::dom::file::File;
use crate::dom::filelist::FileList;
use crate::dom::globalscope::GlobalScope;
use crate::dom::htmldatalistelement::HTMLDataListElement;
use crate::dom::htmlelement::HTMLElement;
use crate::dom::htmlfieldsetelement::HTMLFieldSetElement;
use crate::dom::htmlformelement::{
    FormControl, FormDatum, FormDatumValue, FormSubmitterElement, HTMLFormElement, ResetFrom,
    SubmittedFrom,
};
use crate::dom::keyboardevent::KeyboardEvent;
use crate::dom::mouseevent::MouseEvent;
use crate::dom::node::{
    document_from_node, window_from_node, BindContext, CloneChildrenFlag, Node, NodeDamage,
    ShadowIncluding, UnbindContext,
};
use crate::dom::nodelist::NodeList;
use crate::dom::textcontrol::{TextControlElement, TextControlSelection};
use crate::dom::validation::{is_barred_by_datalist_ancestor, Validatable};
use crate::dom::validitystate::{ValidationFlags, ValidityState};
use crate::dom::virtualmethods::VirtualMethods;
use crate::realms::enter_realm;
use crate::script_runtime::JSContext as SafeJSContext;
use crate::textinput::KeyReaction::{
    DispatchInput, Nothing, RedrawSelection, TriggerDefaultAction,
};
use crate::textinput::Lines::Single;
use crate::textinput::{Direction, SelectionDirection, TextInput, UTF16CodeUnits, UTF8Bytes};

const DEFAULT_SUBMIT_VALUE: &str = "Submit";
const DEFAULT_RESET_VALUE: &str = "Reset";
const PASSWORD_REPLACEMENT_CHAR: char = '●';

#[derive(Clone, Copy, Default, JSTraceable, PartialEq)]
#[allow(dead_code)]
#[derive(MallocSizeOf)]
pub enum InputType {
    Button,
    Checkbox,
    Color,
    Date,
    DatetimeLocal,
    Email,
    File,
    Hidden,
    Image,
    Month,
    Number,
    Password,
    Radio,
    Range,
    Reset,
    Search,
    Submit,
    Tel,
    #[default]
    Text,
    Time,
    Url,
    Week,
}

impl InputType {
    // Note that Password is not included here since it is handled
    // slightly differently, with placeholder characters shown rather
    // than the underlying value.
    fn is_textual(&self) -> bool {
        matches!(
            *self,
            InputType::Color |
                InputType::Date |
                InputType::DatetimeLocal |
                InputType::Email |
                InputType::Hidden |
                InputType::Month |
                InputType::Number |
                InputType::Range |
                InputType::Search |
                InputType::Tel |
                InputType::Text |
                InputType::Time |
                InputType::Url |
                InputType::Week
        )
    }

    fn is_textual_or_password(&self) -> bool {
        self.is_textual() || *self == InputType::Password
    }

    // https://html.spec.whatwg.org/multipage/#has-a-periodic-domain
    fn has_periodic_domain(&self) -> bool {
        *self == InputType::Time
    }

    fn to_str(&self) -> &str {
        match *self {
            InputType::Button => "button",
            InputType::Checkbox => "checkbox",
            InputType::Color => "color",
            InputType::Date => "date",
            InputType::DatetimeLocal => "datetime-local",
            InputType::Email => "email",
            InputType::File => "file",
            InputType::Hidden => "hidden",
            InputType::Image => "image",
            InputType::Month => "month",
            InputType::Number => "number",
            InputType::Password => "password",
            InputType::Radio => "radio",
            InputType::Range => "range",
            InputType::Reset => "reset",
            InputType::Search => "search",
            InputType::Submit => "submit",
            InputType::Tel => "tel",
            InputType::Text => "text",
            InputType::Time => "time",
            InputType::Url => "url",
            InputType::Week => "week",
        }
    }

    pub fn as_ime_type(&self) -> Option<InputMethodType> {
        match *self {
            InputType::Color => Some(InputMethodType::Color),
            InputType::Date => Some(InputMethodType::Date),
            InputType::DatetimeLocal => Some(InputMethodType::DatetimeLocal),
            InputType::Email => Some(InputMethodType::Email),
            InputType::Month => Some(InputMethodType::Month),
            InputType::Number => Some(InputMethodType::Number),
            InputType::Password => Some(InputMethodType::Password),
            InputType::Search => Some(InputMethodType::Search),
            InputType::Tel => Some(InputMethodType::Tel),
            InputType::Text => Some(InputMethodType::Text),
            InputType::Time => Some(InputMethodType::Time),
            InputType::Url => Some(InputMethodType::Url),
            InputType::Week => Some(InputMethodType::Week),
            _ => None,
        }
    }
}

impl<'a> From<&'a Atom> for InputType {
    fn from(value: &Atom) -> InputType {
        match value.to_ascii_lowercase() {
            atom!("button") => InputType::Button,
            atom!("checkbox") => InputType::Checkbox,
            atom!("color") => InputType::Color,
            atom!("date") => InputType::Date,
            atom!("datetime-local") => InputType::DatetimeLocal,
            atom!("email") => InputType::Email,
            atom!("file") => InputType::File,
            atom!("hidden") => InputType::Hidden,
            atom!("image") => InputType::Image,
            atom!("month") => InputType::Month,
            atom!("number") => InputType::Number,
            atom!("password") => InputType::Password,
            atom!("radio") => InputType::Radio,
            atom!("range") => InputType::Range,
            atom!("reset") => InputType::Reset,
            atom!("search") => InputType::Search,
            atom!("submit") => InputType::Submit,
            atom!("tel") => InputType::Tel,
            atom!("text") => InputType::Text,
            atom!("time") => InputType::Time,
            atom!("url") => InputType::Url,
            atom!("week") => InputType::Week,
            _ => Self::default(),
        }
    }
}

#[derive(Debug, PartialEq)]
enum ValueMode {
    Value,
    Default,
    DefaultOn,
    Filename,
}

#[derive(Debug, PartialEq)]
enum StepDirection {
    Up,
    Down,
}

#[dom_struct]
pub struct HTMLInputElement {
    htmlelement: HTMLElement,
    input_type: Cell<InputType>,
    checked_changed: Cell<bool>,
    placeholder: DomRefCell<DOMString>,
    size: Cell<u32>,
    maxlength: Cell<i32>,
    minlength: Cell<i32>,
    #[ignore_malloc_size_of = "#7193"]
    #[no_trace]
    textinput: DomRefCell<TextInput<ScriptToConstellationChan>>,
    // https://html.spec.whatwg.org/multipage/#concept-input-value-dirty-flag
    value_dirty: Cell<bool>,
    // not specified explicitly, but implied by the fact that sanitization can't
    // happen until after all of step/min/max/value content attributes have
    // been added
    sanitization_flag: Cell<bool>,

    filelist: MutNullableDom<FileList>,
    form_owner: MutNullableDom<HTMLFormElement>,
    labels_node_list: MutNullableDom<NodeList>,
    validity_state: MutNullableDom<ValidityState>,
}

#[derive(JSTraceable)]
pub struct InputActivationState {
    indeterminate: bool,
    checked: bool,
    checked_radio: Option<DomRoot<HTMLInputElement>>,
    // In case the type changed
    old_type: InputType,
    // was_mutable is implied: pre-activation would return None if it wasn't
}

static DEFAULT_INPUT_SIZE: u32 = 20;
static DEFAULT_MAX_LENGTH: i32 = -1;
static DEFAULT_MIN_LENGTH: i32 = -1;

#[allow(non_snake_case)]
impl HTMLInputElement {
    fn new_inherited(
        local_name: LocalName,
        prefix: Option<Prefix>,
        document: &Document,
    ) -> HTMLInputElement {
        let chan = document
            .window()
            .upcast::<GlobalScope>()
            .script_to_constellation_chan()
            .clone();
        HTMLInputElement {
            htmlelement: HTMLElement::new_inherited_with_state(
                ElementState::ENABLED | ElementState::READWRITE,
                local_name,
                prefix,
                document,
            ),
            input_type: Cell::new(Default::default()),
            placeholder: DomRefCell::new(DOMString::new()),
            checked_changed: Cell::new(false),
            maxlength: Cell::new(DEFAULT_MAX_LENGTH),
            minlength: Cell::new(DEFAULT_MIN_LENGTH),
            size: Cell::new(DEFAULT_INPUT_SIZE),
            textinput: DomRefCell::new(TextInput::new(
                Single,
                DOMString::new(),
                chan,
                None,
                None,
                SelectionDirection::None,
            )),
            value_dirty: Cell::new(false),
            sanitization_flag: Cell::new(true),
            filelist: MutNullableDom::new(None),
            form_owner: Default::default(),
            labels_node_list: MutNullableDom::new(None),
            validity_state: Default::default(),
        }
    }

    #[allow(crown::unrooted_must_root)]
    pub fn new(
        local_name: LocalName,
        prefix: Option<Prefix>,
        document: &Document,
        proto: Option<HandleObject>,
    ) -> DomRoot<HTMLInputElement> {
        Node::reflect_node_with_proto(
            Box::new(HTMLInputElement::new_inherited(
                local_name, prefix, document,
            )),
            document,
            proto,
        )
    }

    pub fn auto_directionality(&self) -> Option<String> {
        match self.input_type() {
            InputType::Text | InputType::Search | InputType::Url | InputType::Email => {
                let value: String = self.Value().to_string();
                Some(HTMLInputElement::directionality_from_value(&value))
            },
            _ => None,
        }
    }

    pub fn directionality_from_value(value: &str) -> String {
        if HTMLInputElement::is_first_strong_character_rtl(value) {
            "rtl".to_owned()
        } else {
            "ltr".to_owned()
        }
    }

    fn is_first_strong_character_rtl(value: &str) -> bool {
        for ch in value.chars() {
            return match bidi_class(ch) {
                BidiClass::L => false,
                BidiClass::AL => true,
                BidiClass::R => true,
                _ => continue,
            };
        }
        false
    }

    // https://html.spec.whatwg.org/multipage/#dom-input-value
    // https://html.spec.whatwg.org/multipage/#concept-input-apply
    fn value_mode(&self) -> ValueMode {
        match self.input_type() {
            InputType::Submit |
            InputType::Reset |
            InputType::Button |
            InputType::Image |
            InputType::Hidden => ValueMode::Default,

            InputType::Checkbox | InputType::Radio => ValueMode::DefaultOn,

            InputType::Color |
            InputType::Date |
            InputType::DatetimeLocal |
            InputType::Email |
            InputType::Month |
            InputType::Number |
            InputType::Password |
            InputType::Range |
            InputType::Search |
            InputType::Tel |
            InputType::Text |
            InputType::Time |
            InputType::Url |
            InputType::Week => ValueMode::Value,

            InputType::File => ValueMode::Filename,
        }
    }

    #[inline]
    pub fn input_type(&self) -> InputType {
        self.input_type.get()
    }

    #[inline]
    pub fn is_submit_button(&self) -> bool {
        let input_type = self.input_type.get();
        input_type == InputType::Submit || input_type == InputType::Image
    }

    pub fn disable_sanitization(&self) {
        self.sanitization_flag.set(false);
    }

    pub fn enable_sanitization(&self) {
        self.sanitization_flag.set(true);
        let mut textinput = self.textinput.borrow_mut();
        let mut value = textinput.single_line_content().clone();
        self.sanitize_value(&mut value);
        textinput.set_content(value);
    }

    fn does_readonly_apply(&self) -> bool {
        matches!(
            self.input_type(),
            InputType::Text |
                InputType::Search |
                InputType::Url |
                InputType::Tel |
                InputType::Email |
                InputType::Password |
                InputType::Date |
                InputType::Month |
                InputType::Week |
                InputType::Time |
                InputType::DatetimeLocal |
                InputType::Number
        )
    }

    fn does_minmaxlength_apply(&self) -> bool {
        matches!(
            self.input_type(),
            InputType::Text |
                InputType::Search |
                InputType::Url |
                InputType::Tel |
                InputType::Email |
                InputType::Password
        )
    }

    fn does_pattern_apply(&self) -> bool {
        matches!(
            self.input_type(),
            InputType::Text |
                InputType::Search |
                InputType::Url |
                InputType::Tel |
                InputType::Email |
                InputType::Password
        )
    }

    fn does_multiple_apply(&self) -> bool {
        self.input_type() == InputType::Email
    }

    // valueAsNumber, step, min, and max all share the same set of
    // input types they apply to
    fn does_value_as_number_apply(&self) -> bool {
        matches!(
            self.input_type(),
            InputType::Date |
                InputType::Month |
                InputType::Week |
                InputType::Time |
                InputType::DatetimeLocal |
                InputType::Number |
                InputType::Range
        )
    }

    fn does_value_as_date_apply(&self) -> bool {
        matches!(
            self.input_type(),
            InputType::Date | InputType::Month | InputType::Week | InputType::Time
        )
    }

    // https://html.spec.whatwg.org/multipage#concept-input-step
    fn allowed_value_step(&self) -> Option<f64> {
        if let Some(attr) = self
            .upcast::<Element>()
            .get_attribute(&ns!(), &local_name!("step"))
        {
            if let Some(step) =
                DOMString::from(attr.summarize().value).parse_floating_point_number()
            {
                if step > 0.0 {
                    return Some(step * self.step_scale_factor());
                }
            }
        }
        self.default_step()
            .map(|step| step * self.step_scale_factor())
    }

    // https://html.spec.whatwg.org/multipage#concept-input-min
    fn minimum(&self) -> Option<f64> {
        if let Some(attr) = self
            .upcast::<Element>()
            .get_attribute(&ns!(), &local_name!("min"))
        {
            if let Some(min) =
                self.convert_string_to_number(&DOMString::from(attr.summarize().value))
            {
                return Some(min);
            }
        }
        self.default_minimum()
    }

    // https://html.spec.whatwg.org/multipage#concept-input-max
    fn maximum(&self) -> Option<f64> {
        if let Some(attr) = self
            .upcast::<Element>()
            .get_attribute(&ns!(), &local_name!("max"))
        {
            if let Some(max) =
                self.convert_string_to_number(&DOMString::from(attr.summarize().value))
            {
                return Some(max);
            }
        }
        self.default_maximum()
    }

    // when allowed_value_step and minumum both exist, this is the smallest
    // value >= minimum that lies on an integer step
    fn stepped_minimum(&self) -> Option<f64> {
        match (self.minimum(), self.allowed_value_step()) {
            (Some(min), Some(allowed_step)) => {
                let step_base = self.step_base();
                // how many steps is min from step_base?
                let nsteps = (min - step_base) / allowed_step;
                // count that many integer steps, rounded +, from step_base
                Some(step_base + (allowed_step * nsteps.ceil()))
            },
            (_, _) => None,
        }
    }

    // when allowed_value_step and maximum both exist, this is the smallest
    // value <= maximum that lies on an integer step
    fn stepped_maximum(&self) -> Option<f64> {
        match (self.maximum(), self.allowed_value_step()) {
            (Some(max), Some(allowed_step)) => {
                let step_base = self.step_base();
                // how many steps is max from step_base?
                let nsteps = (max - step_base) / allowed_step;
                // count that many integer steps, rounded -, from step_base
                Some(step_base + (allowed_step * nsteps.floor()))
            },
            (_, _) => None,
        }
    }

    // https://html.spec.whatwg.org/multipage#concept-input-min-default
    fn default_minimum(&self) -> Option<f64> {
        match self.input_type() {
            InputType::Range => Some(0.0),
            _ => None,
        }
    }

    // https://html.spec.whatwg.org/multipage#concept-input-max-default
    fn default_maximum(&self) -> Option<f64> {
        match self.input_type() {
            InputType::Range => Some(100.0),
            _ => None,
        }
    }

    // https://html.spec.whatwg.org/multipage#concept-input-value-default-range
    fn default_range_value(&self) -> f64 {
        let min = self.minimum().unwrap_or(0.0);
        let max = self.maximum().unwrap_or(100.0);
        if max < min {
            min
        } else {
            min + (max - min) * 0.5
        }
    }

    // https://html.spec.whatwg.org/multipage#concept-input-step-default
    fn default_step(&self) -> Option<f64> {
        match self.input_type() {
            InputType::Date => Some(1.0),
            InputType::Month => Some(1.0),
            InputType::Week => Some(1.0),
            InputType::Time => Some(60.0),
            InputType::DatetimeLocal => Some(60.0),
            InputType::Number => Some(1.0),
            InputType::Range => Some(1.0),
            _ => None,
        }
    }

    // https://html.spec.whatwg.org/multipage#concept-input-step-scale
    fn step_scale_factor(&self) -> f64 {
        match self.input_type() {
            InputType::Date => 86400000.0,
            InputType::Month => 1.0,
            InputType::Week => 604800000.0,
            InputType::Time => 1000.0,
            InputType::DatetimeLocal => 1000.0,
            InputType::Number => 1.0,
            InputType::Range => 1.0,
            _ => unreachable!(),
        }
    }

    // https://html.spec.whatwg.org/multipage#concept-input-min-zero
    fn step_base(&self) -> f64 {
        if let Some(attr) = self
            .upcast::<Element>()
            .get_attribute(&ns!(), &local_name!("min"))
        {
            let minstr = &DOMString::from(attr.summarize().value);
            if let Some(min) = self.convert_string_to_number(minstr) {
                return min;
            }
        }
        if let Some(attr) = self
            .upcast::<Element>()
            .get_attribute(&ns!(), &local_name!("value"))
        {
            if let Some(value) =
                self.convert_string_to_number(&DOMString::from(attr.summarize().value))
            {
                return value;
            }
        }
        self.default_step_base().unwrap_or(0.0)
    }

    // https://html.spec.whatwg.org/multipage#concept-input-step-default-base
    fn default_step_base(&self) -> Option<f64> {
        match self.input_type() {
            InputType::Week => Some(-259200000.0),
            _ => None,
        }
    }

    // https://html.spec.whatwg.org/multipage/#dom-input-stepdown
    // https://html.spec.whatwg.org/multipage/#dom-input-stepup
    fn step_up_or_down(&self, n: i32, dir: StepDirection) -> ErrorResult {
        // Step 1
        if !self.does_value_as_number_apply() {
            return Err(Error::InvalidState);
        }
        let step_base = self.step_base();
        // Step 2
        let allowed_value_step = match self.allowed_value_step() {
            Some(avs) => avs,
            None => return Err(Error::InvalidState),
        };
        let minimum = self.minimum();
        let maximum = self.maximum();
        if let (Some(min), Some(max)) = (minimum, maximum) {
            // Step 3
            if min > max {
                return Ok(());
            }
            // Step 4
            if let Some(smin) = self.stepped_minimum() {
                if smin > max {
                    return Ok(());
                }
            }
        }
        // Step 5
        let mut value: f64 = self.convert_string_to_number(&self.Value()).unwrap_or(0.0);

        // Step 6
        let valueBeforeStepping = value;

        // Step 7
        if (value - step_base) % allowed_value_step != 0.0 {
            value = match dir {
                StepDirection::Down =>
                //step down a fractional step to be on a step multiple
                {
                    let intervals_from_base = ((value - step_base) / allowed_value_step).floor();
                    intervals_from_base * allowed_value_step + step_base
                },
                StepDirection::Up =>
                // step up a fractional step to be on a step multiple
                {
                    let intervals_from_base = ((value - step_base) / allowed_value_step).ceil();
                    intervals_from_base * allowed_value_step + step_base
                },
            };
        } else {
            value += match dir {
                StepDirection::Down => -f64::from(n) * allowed_value_step,
                StepDirection::Up => f64::from(n) * allowed_value_step,
            };
        }

        // Step 8
        if let Some(min) = minimum {
            if value < min {
                value = self.stepped_minimum().unwrap_or(value);
            }
        }

        // Step 9
        if let Some(max) = maximum {
            if value > max {
                value = self.stepped_maximum().unwrap_or(value);
            }
        }

        // Step 10
        match dir {
            StepDirection::Down => {
                if value > valueBeforeStepping {
                    return Ok(());
                }
            },
            StepDirection::Up => {
                if value < valueBeforeStepping {
                    return Ok(());
                }
            },
        }

        // Step 11
        self.SetValueAsNumber(value)
    }

    // https://html.spec.whatwg.org/multipage/#concept-input-list
    fn suggestions_source_element(&self) -> Option<DomRoot<HTMLDataListElement>> {
        let list_string = self
            .upcast::<Element>()
            .get_string_attribute(&local_name!("list"));
        if list_string.is_empty() {
            return None;
        }
        let ancestor = self
            .upcast::<Node>()
            .GetRootNode(&GetRootNodeOptions::empty());
        let first_with_id = &ancestor
            .traverse_preorder(ShadowIncluding::No)
            .find(|node| {
                node.downcast::<Element>()
                    .map_or(false, |e| e.Id() == list_string)
            });
        first_with_id
            .as_ref()
            .and_then(|el| el.downcast::<HTMLDataListElement>())
            .map(DomRoot::from_ref)
    }

    // https://html.spec.whatwg.org/multipage/#suffering-from-being-missing
    fn suffers_from_being_missing(&self, value: &DOMString) -> bool {
        match self.input_type() {
            // https://html.spec.whatwg.org/multipage/#checkbox-state-(type%3Dcheckbox)%3Asuffering-from-being-missing
            InputType::Checkbox => self.Required() && !self.Checked(),
            // https://html.spec.whatwg.org/multipage/#radio-button-state-(type%3Dradio)%3Asuffering-from-being-missing
            InputType::Radio => {
                let mut is_required = self.Required();
                let mut is_checked = self.Checked();
                for other in radio_group_iter(self, self.radio_group_name().as_ref()) {
                    is_required = is_required || other.Required();
                    is_checked = is_checked || other.Checked();
                }
                is_required && !is_checked
            },
            // https://html.spec.whatwg.org/multipage/#file-upload-state-(type%3Dfile)%3Asuffering-from-being-missing
            InputType::File => {
                self.Required() &&
                    self.filelist
                        .get()
                        .map_or(true, |files| files.Length() == 0)
            },
            // https://html.spec.whatwg.org/multipage/#the-required-attribute%3Asuffering-from-being-missing
            _ => {
                self.Required() &&
                    self.value_mode() == ValueMode::Value &&
                    self.is_mutable() &&
                    value.is_empty()
            },
        }
    }

    // https://html.spec.whatwg.org/multipage/#suffering-from-a-type-mismatch
    fn suffers_from_type_mismatch(&self, value: &DOMString) -> bool {
        if value.is_empty() {
            return false;
        }

        match self.input_type() {
            // https://html.spec.whatwg.org/multipage/#url-state-(type%3Durl)%3Asuffering-from-a-type-mismatch
            InputType::Url => Url::parse(value).is_err(),
            // https://html.spec.whatwg.org/multipage/#e-mail-state-(type%3Demail)%3Asuffering-from-a-type-mismatch
            // https://html.spec.whatwg.org/multipage/#e-mail-state-(type%3Demail)%3Asuffering-from-a-type-mismatch-2
            InputType::Email => {
                if self.Multiple() {
                    !split_commas(value).all(|s| {
                        DOMString::from_string(s.to_string()).is_valid_email_address_string()
                    })
                } else {
                    !value.is_valid_email_address_string()
                }
            },
            // Other input types don't suffer from type mismatch
            _ => false,
        }
    }

    // https://html.spec.whatwg.org/multipage/#suffering-from-a-pattern-mismatch
    fn suffers_from_pattern_mismatch(&self, value: &DOMString) -> bool {
        // https://html.spec.whatwg.org/multipage/#the-pattern-attribute%3Asuffering-from-a-pattern-mismatch
        // https://html.spec.whatwg.org/multipage/#the-pattern-attribute%3Asuffering-from-a-pattern-mismatch-2
        let pattern_str = self.Pattern();
        if value.is_empty() || pattern_str.is_empty() || !self.does_pattern_apply() {
            return false;
        }

        // Rust's regex is not compatible, we need to use mozjs RegExp.
        let cx = GlobalScope::get_cx();
        let _ac = enter_realm(self);
        rooted!(in(*cx) let mut pattern = ptr::null_mut::<JSObject>());

        if compile_pattern(cx, &pattern_str, pattern.handle_mut()) {
            if self.Multiple() && self.does_multiple_apply() {
                !split_commas(value)
                    .all(|s| matches_js_regex(cx, pattern.handle(), s).unwrap_or(true))
            } else {
                !matches_js_regex(cx, pattern.handle(), value).unwrap_or(true)
            }
        } else {
            // Element doesn't suffer from pattern mismatch if pattern is invalid.
            false
        }
    }

    // https://html.spec.whatwg.org/multipage/#suffering-from-bad-input
    fn suffers_from_bad_input(&self, value: &DOMString) -> bool {
        if value.is_empty() {
            return false;
        }

        match self.input_type() {
            // https://html.spec.whatwg.org/multipage/#e-mail-state-(type%3Demail)%3Asuffering-from-bad-input
            // https://html.spec.whatwg.org/multipage/#e-mail-state-(type%3Demail)%3Asuffering-from-bad-input-2
            InputType::Email => {
                // TODO: Check for input that cannot be converted to punycode.
                // Currently we don't support conversion of email values to punycode
                // so always return false.
                false
            },
            // https://html.spec.whatwg.org/multipage/#date-state-(type%3Ddate)%3Asuffering-from-bad-input
            InputType::Date => !value.is_valid_date_string(),
            // https://html.spec.whatwg.org/multipage/#month-state-(type%3Dmonth)%3Asuffering-from-bad-input
            InputType::Month => !value.is_valid_month_string(),
            // https://html.spec.whatwg.org/multipage/#week-state-(type%3Dweek)%3Asuffering-from-bad-input
            InputType::Week => !value.is_valid_week_string(),
            // https://html.spec.whatwg.org/multipage/#time-state-(type%3Dtime)%3Asuffering-from-bad-input
            InputType::Time => !value.is_valid_time_string(),
            // https://html.spec.whatwg.org/multipage/#local-date-and-time-state-(type%3Ddatetime-local)%3Asuffering-from-bad-input
            InputType::DatetimeLocal => value.parse_local_date_and_time_string().is_none(),
            // https://html.spec.whatwg.org/multipage/#number-state-(type%3Dnumber)%3Asuffering-from-bad-input
            // https://html.spec.whatwg.org/multipage/#range-state-(type%3Drange)%3Asuffering-from-bad-input
            InputType::Number | InputType::Range => !value.is_valid_floating_point_number_string(),
            // https://html.spec.whatwg.org/multipage/#color-state-(type%3Dcolor)%3Asuffering-from-bad-input
            InputType::Color => !value.is_valid_simple_color_string(),
            // Other input types don't suffer from bad input
            _ => false,
        }
    }

    // https://html.spec.whatwg.org/multipage/#suffering-from-being-too-long
    // https://html.spec.whatwg.org/multipage/#suffering-from-being-too-short
    fn suffers_from_length_issues(&self, value: &DOMString) -> ValidationFlags {
        // https://html.spec.whatwg.org/multipage/#limiting-user-input-length%3A-the-maxlength-attribute%3Asuffering-from-being-too-long
        // https://html.spec.whatwg.org/multipage/#setting-minimum-input-length-requirements%3A-the-minlength-attribute%3Asuffering-from-being-too-short
        let value_dirty = self.value_dirty.get();
        let textinput = self.textinput.borrow();
        let edit_by_user = !textinput.was_last_change_by_set_content();

        if value.is_empty() || !value_dirty || !edit_by_user || !self.does_minmaxlength_apply() {
            return ValidationFlags::empty();
        }

        let mut failed_flags = ValidationFlags::empty();
        let UTF16CodeUnits(value_len) = textinput.utf16_len();
        let min_length = self.MinLength();
        let max_length = self.MaxLength();

        if min_length != DEFAULT_MIN_LENGTH && value_len < (min_length as usize) {
            failed_flags.insert(ValidationFlags::TOO_SHORT);
        }

        if max_length != DEFAULT_MAX_LENGTH && value_len > (max_length as usize) {
            failed_flags.insert(ValidationFlags::TOO_LONG);
        }

        failed_flags
    }

    // https://html.spec.whatwg.org/multipage/#suffering-from-an-underflow
    // https://html.spec.whatwg.org/multipage/#suffering-from-an-overflow
    // https://html.spec.whatwg.org/multipage/#suffering-from-a-step-mismatch
    fn suffers_from_range_issues(&self, value: &DOMString) -> ValidationFlags {
        if value.is_empty() || !self.does_value_as_number_apply() {
            return ValidationFlags::empty();
        }

        let Some(value_as_number) = self.convert_string_to_number(value) else {
            return ValidationFlags::empty();
        };

        let mut failed_flags = ValidationFlags::empty();
        let min_value = self.minimum();
        let max_value = self.maximum();

        // https://html.spec.whatwg.org/multipage/#has-a-reversed-range
        let has_reversed_range = match (min_value, max_value) {
            (Some(min), Some(max)) => self.input_type().has_periodic_domain() && min > max,
            _ => false,
        };

        if has_reversed_range {
            // https://html.spec.whatwg.org/multipage/#the-min-and-max-attributes:has-a-reversed-range-3
            if value_as_number > max_value.unwrap() && value_as_number < min_value.unwrap() {
                failed_flags.insert(ValidationFlags::RANGE_UNDERFLOW);
                failed_flags.insert(ValidationFlags::RANGE_OVERFLOW);
            }
        } else {
            // https://html.spec.whatwg.org/multipage/#the-min-and-max-attributes%3Asuffering-from-an-underflow-2
            if let Some(min_value) = min_value {
                if value_as_number < min_value {
                    failed_flags.insert(ValidationFlags::RANGE_UNDERFLOW);
                }
            }
            // https://html.spec.whatwg.org/multipage/#the-min-and-max-attributes%3Asuffering-from-an-overflow-2
            if let Some(max_value) = max_value {
                if value_as_number > max_value {
                    failed_flags.insert(ValidationFlags::RANGE_OVERFLOW);
                }
            }
        }

        // https://html.spec.whatwg.org/multipage/#the-step-attribute%3Asuffering-from-a-step-mismatch
        if let Some(step) = self.allowed_value_step() {
            // TODO: Spec has some issues here, see https://github.com/whatwg/html/issues/5207.
            // Chrome and Firefox parse values as decimals to get exact results,
            // we probably should too.
            let diff = (self.step_base() - value_as_number) % step / value_as_number;
            if diff.abs() > 1e-12 {
                failed_flags.insert(ValidationFlags::STEP_MISMATCH);
            }
        }

        failed_flags
    }
}

pub trait LayoutHTMLInputElementHelpers<'dom> {
    fn value_for_layout(self) -> Cow<'dom, str>;
    fn size_for_layout(self) -> u32;
    fn selection_for_layout(self) -> Option<Range<usize>>;
}

#[allow(unsafe_code)]
impl<'dom> LayoutDom<'dom, HTMLInputElement> {
    fn get_raw_textinput_value(self) -> DOMString {
        unsafe {
            self.unsafe_get()
                .textinput
                .borrow_for_layout()
                .get_content()
        }
    }

    fn placeholder(self) -> &'dom str {
        unsafe { self.unsafe_get().placeholder.borrow_for_layout() }
    }

    fn input_type(self) -> InputType {
        self.unsafe_get().input_type.get()
    }

    fn textinput_sorted_selection_offsets_range(self) -> Range<UTF8Bytes> {
        unsafe {
            self.unsafe_get()
                .textinput
                .borrow_for_layout()
                .sorted_selection_offsets_range()
        }
    }
}

impl<'dom> LayoutHTMLInputElementHelpers<'dom> for LayoutDom<'dom, HTMLInputElement> {
    fn value_for_layout(self) -> Cow<'dom, str> {
        fn get_raw_attr_value<'dom>(
            input: LayoutDom<'dom, HTMLInputElement>,
            default: &'static str,
        ) -> Cow<'dom, str> {
            input
                .upcast::<Element>()
                .get_attr_val_for_layout(&ns!(), &local_name!("value"))
                .unwrap_or(default)
                .into()
        }

        match self.input_type() {
            InputType::Checkbox | InputType::Radio => "".into(),
            InputType::File | InputType::Image => "".into(),
            InputType::Button => get_raw_attr_value(self, ""),
            InputType::Submit => get_raw_attr_value(self, DEFAULT_SUBMIT_VALUE),
            InputType::Reset => get_raw_attr_value(self, DEFAULT_RESET_VALUE),
            InputType::Password => {
                let text = self.get_raw_textinput_value();
                if !text.is_empty() {
                    text.chars()
                        .map(|_| PASSWORD_REPLACEMENT_CHAR)
                        .collect::<String>()
                        .into()
                } else {
                    self.placeholder().into()
                }
            },
            _ => {
                let text = self.get_raw_textinput_value();
                if !text.is_empty() {
                    text.into()
                } else {
                    self.placeholder().into()
                }
            },
        }
    }

    fn size_for_layout(self) -> u32 {
        self.unsafe_get().size.get()
    }

    fn selection_for_layout(self) -> Option<Range<usize>> {
        if !self.upcast::<Element>().focus_state() {
            return None;
        }

        let sorted_selection_offsets_range = self.textinput_sorted_selection_offsets_range();

        match self.input_type() {
            InputType::Password => {
                let text = self.get_raw_textinput_value();
                let sel = UTF8Bytes::unwrap_range(sorted_selection_offsets_range);

                // Translate indices from the raw value to indices in the replacement value.
                let char_start = text[..sel.start].chars().count();
                let char_end = char_start + text[sel].chars().count();

                let bytes_per_char = PASSWORD_REPLACEMENT_CHAR.len_utf8();
                Some(char_start * bytes_per_char..char_end * bytes_per_char)
            },
            input_type if input_type.is_textual() => {
                Some(UTF8Bytes::unwrap_range(sorted_selection_offsets_range))
            },
            _ => None,
        }
    }
}

impl TextControlElement for HTMLInputElement {
    // https://html.spec.whatwg.org/multipage/#concept-input-apply
    fn selection_api_applies(&self) -> bool {
        matches!(
            self.input_type(),
            InputType::Text |
                InputType::Search |
                InputType::Url |
                InputType::Tel |
                InputType::Password
        )
    }

    // https://html.spec.whatwg.org/multipage/#concept-input-apply
    //
    // Defines input types to which the select() IDL method applies. These are a superset of the
    // types for which selection_api_applies() returns true.
    //
    // Types omitted which could theoretically be included if they were
    // rendered as a text control: file
    fn has_selectable_text(&self) -> bool {
        match self.input_type() {
            InputType::Text |
            InputType::Search |
            InputType::Url |
            InputType::Tel |
            InputType::Password |
            InputType::Email |
            InputType::Date |
            InputType::Month |
            InputType::Week |
            InputType::Time |
            InputType::DatetimeLocal |
            InputType::Number |
            InputType::Color => true,

            InputType::Button |
            InputType::Checkbox |
            InputType::File |
            InputType::Hidden |
            InputType::Image |
            InputType::Radio |
            InputType::Range |
            InputType::Reset |
            InputType::Submit => false,
        }
    }

    fn set_dirty_value_flag(&self, value: bool) {
        self.value_dirty.set(value)
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
    fn GetForm(&self) -> Option<DomRoot<HTMLFormElement>> {
        self.form_owner()
    }

    // https://html.spec.whatwg.org/multipage/#dom-input-files
    fn GetFiles(&self) -> Option<DomRoot<FileList>> {
        self.filelist.get().as_ref().cloned()
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-input-files>
    fn SetFiles(&self, files: Option<&FileList>) {
        if self.input_type() == InputType::File && files.is_some() {
            self.filelist.set(files);
        }
    }

    // https://html.spec.whatwg.org/multipage/#dom-input-defaultchecked
    make_bool_getter!(DefaultChecked, "checked");

    // https://html.spec.whatwg.org/multipage/#dom-input-defaultchecked
    make_bool_setter!(SetDefaultChecked, "checked");

    // https://html.spec.whatwg.org/multipage/#dom-input-checked
    fn Checked(&self) -> bool {
        self.upcast::<Element>()
            .state()
            .contains(ElementState::CHECKED)
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
    fn Type(&self) -> DOMString {
        DOMString::from(self.input_type().to_str())
    }

    // https://html.spec.whatwg.org/multipage/#dom-input-type
    make_atomic_setter!(SetType, "type");

    // https://html.spec.whatwg.org/multipage/#dom-input-value
    fn Value(&self) -> DOMString {
        match self.value_mode() {
            ValueMode::Value => self.textinput.borrow().get_content(),
            ValueMode::Default => self
                .upcast::<Element>()
                .get_attribute(&ns!(), &local_name!("value"))
                .map_or(DOMString::from(""), |a| {
                    DOMString::from(a.summarize().value)
                }),
            ValueMode::DefaultOn => self
                .upcast::<Element>()
                .get_attribute(&ns!(), &local_name!("value"))
                .map_or(DOMString::from("on"), |a| {
                    DOMString::from(a.summarize().value)
                }),
            ValueMode::Filename => {
                let mut path = DOMString::from("");
                match self.filelist.get() {
                    Some(ref fl) => match fl.Item(0) {
                        Some(ref f) => {
                            path.push_str("C:\\fakepath\\");
                            path.push_str(f.name());
                            path
                        },
                        None => path,
                    },
                    None => path,
                }
            },
        }
    }

    // https://html.spec.whatwg.org/multipage/#dom-input-value
    fn SetValue(&self, mut value: DOMString) -> ErrorResult {
        match self.value_mode() {
            ValueMode::Value => {
                // Step 3.
                self.value_dirty.set(true);

                // Step 4.
                self.sanitize_value(&mut value);

                let mut textinput = self.textinput.borrow_mut();

                // Step 5.
                if *textinput.single_line_content() != value {
                    // Steps 1-2
                    textinput.set_content(value);

                    // Step 5.
                    textinput.clear_selection_to_limit(Direction::Forward);
                }
            },
            ValueMode::Default | ValueMode::DefaultOn => {
                self.upcast::<Element>()
                    .set_string_attribute(&local_name!("value"), value);
            },
            ValueMode::Filename => {
                if value.is_empty() {
                    let window = window_from_node(self);
                    let fl = FileList::new(&window, vec![]);
                    self.filelist.set(Some(&fl));
                } else {
                    return Err(Error::InvalidState);
                }
            },
        }

        self.validity_state()
            .perform_validation_and_update(ValidationFlags::all());
        self.upcast::<Node>().dirty(NodeDamage::OtherNodeDamage);
        Ok(())
    }

    // https://html.spec.whatwg.org/multipage/#dom-input-defaultvalue
    make_getter!(DefaultValue, "value");

    // https://html.spec.whatwg.org/multipage/#dom-input-defaultvalue
    make_setter!(SetDefaultValue, "value");

    // https://html.spec.whatwg.org/multipage/#dom-input-min
    make_getter!(Min, "min");

    // https://html.spec.whatwg.org/multipage/#dom-input-min
    make_setter!(SetMin, "min");

    // https://html.spec.whatwg.org/multipage/#dom-input-list
    fn GetList(&self) -> Option<DomRoot<HTMLDataListElement>> {
        self.suggestions_source_element()
    }

    // https://html.spec.whatwg.org/multipage/#dom-input-valueasdate
    #[allow(unsafe_code)]
    fn GetValueAsDate(&self, cx: SafeJSContext) -> Option<NonNull<JSObject>> {
        self.convert_string_to_naive_datetime(self.Value())
            .map(|dt| unsafe {
                let time = ClippedTime {
                    t: dt.and_utc().timestamp_millis() as f64,
                };
                NonNull::new_unchecked(NewDateObject(*cx, time))
            })
    }

    // https://html.spec.whatwg.org/multipage/#dom-input-valueasdate
    #[allow(unsafe_code, non_snake_case)]
    fn SetValueAsDate(&self, cx: SafeJSContext, value: *mut JSObject) -> ErrorResult {
        rooted!(in(*cx) let value = value);
        if !self.does_value_as_date_apply() {
            return Err(Error::InvalidState);
        }
        if value.is_null() {
            return self.SetValue(DOMString::from(""));
        }
        let mut msecs: f64 = 0.0;
        // We need to go through unsafe code to interrogate jsapi about a Date.
        // To minimize the amount of unsafe code to maintain, this just gets the milliseconds,
        // which we then reinflate into a NaiveDate for use in safe code.
        unsafe {
            let mut isDate = false;
            if !ObjectIsDate(*cx, Handle::from(value.handle()), &mut isDate) {
                return Err(Error::JSFailed);
            }
            if !isDate {
                return Err(Error::Type("Value was not a date".to_string()));
            }
            if !DateGetMsecSinceEpoch(*cx, Handle::from(value.handle()), &mut msecs) {
                return Err(Error::JSFailed);
            }
            if !msecs.is_finite() {
                return self.SetValue(DOMString::from(""));
            }
        }
        // now we make a Rust date out of it so we can use safe code for the
        // actual conversion logic
        match milliseconds_to_datetime(msecs) {
            Ok(dt) => match self.convert_naive_datetime_to_string(dt) {
                Ok(converted) => self.SetValue(converted),
                _ => self.SetValue(DOMString::from("")),
            },
            _ => self.SetValue(DOMString::from("")),
        }
    }

    // https://html.spec.whatwg.org/multipage/#dom-input-valueasnumber
    fn ValueAsNumber(&self) -> f64 {
        self.convert_string_to_number(&self.Value())
            .unwrap_or(std::f64::NAN)
    }

    // https://html.spec.whatwg.org/multipage/#dom-input-valueasnumber
    fn SetValueAsNumber(&self, value: f64) -> ErrorResult {
        if value.is_infinite() {
            Err(Error::Type("value is not finite".to_string()))
        } else if !self.does_value_as_number_apply() {
            Err(Error::InvalidState)
        } else if value.is_nan() {
            self.SetValue(DOMString::from(""))
        } else if let Ok(converted) = self.convert_number_to_string(value) {
            self.SetValue(converted)
        } else {
            // The most literal spec-compliant implementation would
            // use bignum chrono types so overflow is impossible,
            // but just setting an overflow to the empty string matches
            // Firefox's behavior.
            // (for example, try input.valueAsNumber=1e30 on a type="date" input)
            self.SetValue(DOMString::from(""))
        }
    }

    // https://html.spec.whatwg.org/multipage/#attr-fe-name
    make_getter!(Name, "name");

    // https://html.spec.whatwg.org/multipage/#attr-fe-name
    make_atomic_setter!(SetName, "name");

    // https://html.spec.whatwg.org/multipage/#dom-input-placeholder
    make_getter!(Placeholder, "placeholder");

    // https://html.spec.whatwg.org/multipage/#dom-input-placeholder
    make_setter!(SetPlaceholder, "placeholder");

    // https://html.spec.whatwg.org/multipage/#dom-input-formaction
    make_form_action_getter!(FormAction, "formaction");

    // https://html.spec.whatwg.org/multipage/#dom-input-formaction
    make_setter!(SetFormAction, "formaction");

    // https://html.spec.whatwg.org/multipage/#dom-input-formenctype
    make_enumerated_getter!(
        FormEnctype,
        "formenctype",
        "application/x-www-form-urlencoded",
        "text/plain" | "multipart/form-data"
    );

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
        self.upcast::<Element>()
            .state()
            .contains(ElementState::INDETERMINATE)
    }

    // https://html.spec.whatwg.org/multipage/#dom-input-indeterminate
    fn SetIndeterminate(&self, val: bool) {
        self.upcast::<Element>()
            .set_state(ElementState::INDETERMINATE, val)
    }

    // https://html.spec.whatwg.org/multipage/#dom-lfe-labels
    // Different from make_labels_getter because this one
    // conditionally returns null.
    fn GetLabels(&self) -> Option<DomRoot<NodeList>> {
        if self.input_type() == InputType::Hidden {
            None
        } else {
            Some(self.labels_node_list.or_init(|| {
                NodeList::new_labels_list(
                    self.upcast::<Node>().owner_doc().window(),
                    self.upcast::<HTMLElement>(),
                )
            }))
        }
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

    // Select the files based on filepaths passed in,
    // enabled by dom.htmlinputelement.select_files.enabled,
    // used for test purpose.
    // check-tidy: no specs after this line
    fn SelectFiles(&self, paths: Vec<DOMString>) {
        if self.input_type() == InputType::File {
            self.select_files(Some(paths));
        }
    }

    // https://html.spec.whatwg.org/multipage/#dom-input-stepup
    fn StepUp(&self, n: i32) -> ErrorResult {
        self.step_up_or_down(n, StepDirection::Up)
    }

    // https://html.spec.whatwg.org/multipage/#dom-input-stepdown
    fn StepDown(&self, n: i32) -> ErrorResult {
        self.step_up_or_down(n, StepDirection::Down)
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
    fn CheckValidity(&self) -> bool {
        self.check_validity()
    }

    // https://html.spec.whatwg.org/multipage/#dom-cva-reportvalidity
    fn ReportValidity(&self) -> bool {
        self.report_validity()
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

fn radio_group_iter<'a>(
    elem: &'a HTMLInputElement,
    group: Option<&'a Atom>,
) -> impl Iterator<Item = DomRoot<HTMLInputElement>> + 'a {
    let owner = elem.form_owner();
    let root = elem
        .upcast::<Node>()
        .GetRootNode(&GetRootNodeOptions::empty());

    // If group is None, in_same_group always fails, but we need to always return elem.
    root.traverse_preorder(ShadowIncluding::No)
        .filter_map(DomRoot::downcast::<HTMLInputElement>)
        .filter(move |r| &**r == elem || in_same_group(r, owner.as_deref(), group, None))
}

fn broadcast_radio_checked(broadcaster: &HTMLInputElement, group: Option<&Atom>) {
    for r in radio_group_iter(broadcaster, group) {
        if broadcaster != &*r && r.Checked() {
            r.SetChecked(false);
        }
    }
}

// https://html.spec.whatwg.org/multipage/#radio-button-group
fn in_same_group(
    other: &HTMLInputElement,
    owner: Option<&HTMLFormElement>,
    group: Option<&Atom>,
    tree_root: Option<&Node>,
) -> bool {
    if group.is_none() {
        // Radio input elements with a missing or empty name are alone in their own group.
        return false;
    }

    if other.input_type() != InputType::Radio ||
        other.form_owner().as_deref() != owner ||
        other.radio_group_name().as_ref() != group
    {
        return false;
    }

    match tree_root {
        Some(tree_root) => {
            let other_root = other
                .upcast::<Node>()
                .GetRootNode(&GetRootNodeOptions::empty());
            tree_root == &*other_root
        },
        None => {
            // Skip check if the tree root isn't provided.
            true
        },
    }
}

impl HTMLInputElement {
    fn radio_group_updated(&self, group: Option<&Atom>) {
        if self.Checked() {
            broadcast_radio_checked(self, group);
        }
    }

    /// <https://html.spec.whatwg.org/multipage/#constructing-the-form-data-set>
    /// Steps range from 5.1 to 5.10 (specific to HTMLInputElement)
    pub fn form_datums(
        &self,
        submitter: Option<FormSubmitterElement>,
        encoding: Option<&'static Encoding>,
    ) -> Vec<FormDatum> {
        // 3.1: disabled state check is in get_unclean_dataset

        // Step 5.2
        let ty = self.Type();

        // Step 5.4
        let name = self.Name();
        let is_submitter = match submitter {
            Some(FormSubmitterElement::Input(s)) => self == s,
            _ => false,
        };

        match self.input_type() {
            // Step 5.1: it's a button but it is not submitter.
            InputType::Submit | InputType::Button | InputType::Reset if !is_submitter => {
                return vec![];
            },

            // Step 5.1: it's the "Checkbox" or "Radio Button" and whose checkedness is false.
            InputType::Radio | InputType::Checkbox => {
                if !self.Checked() || name.is_empty() {
                    return vec![];
                }
            },

            InputType::File => {
                let mut datums = vec![];

                // Step 5.2-5.7
                let name = self.Name();

                match self.GetFiles() {
                    Some(fl) => {
                        for f in fl.iter_files() {
                            datums.push(FormDatum {
                                ty: ty.clone(),
                                name: name.clone(),
                                value: FormDatumValue::File(DomRoot::from_ref(f)),
                            });
                        }
                    },
                    None => {
                        datums.push(FormDatum {
                            // XXX(izgzhen): Spec says 'application/octet-stream' as the type,
                            // but this is _type_ of element rather than content right?
                            ty: ty.clone(),
                            name: name.clone(),
                            value: FormDatumValue::String(DOMString::from("")),
                        })
                    },
                }

                return datums;
            },

            InputType::Image => return vec![], // Unimplemented

            // Step 5.10: it's a hidden field named _charset_
            InputType::Hidden => {
                if name.to_ascii_lowercase() == "_charset_" {
                    return vec![FormDatum {
                        ty: ty.clone(),
                        name,
                        value: FormDatumValue::String(match encoding {
                            None => DOMString::from("UTF-8"),
                            Some(enc) => DOMString::from(enc.name()),
                        }),
                    }];
                }
            },

            // Step 5.1: it's not the "Image Button" and doesn't have a name attribute.
            _ => {
                if name.is_empty() {
                    return vec![];
                }
            },
        }

        // Step 5.12
        vec![FormDatum {
            ty: ty.clone(),
            name,
            value: FormDatumValue::String(self.Value()),
        }]
    }

    // https://html.spec.whatwg.org/multipage/#radio-button-group
    fn radio_group_name(&self) -> Option<Atom> {
        self.upcast::<Element>().get_name().and_then(|name| {
            if name == atom!("") {
                None
            } else {
                Some(name)
            }
        })
    }

    fn update_checked_state(&self, checked: bool, dirty: bool) {
        self.upcast::<Element>()
            .set_state(ElementState::CHECKED, checked);

        if dirty {
            self.checked_changed.set(true);
        }

        if self.input_type() == InputType::Radio && checked {
            broadcast_radio_checked(self, self.radio_group_name().as_ref());
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
        match self.input_type() {
            InputType::Radio | InputType::Checkbox => {
                self.update_checked_state(self.DefaultChecked(), false);
                self.checked_changed.set(false);
            },
            InputType::Image => (),
            _ => (),
        }
        self.textinput.borrow_mut().set_content(self.DefaultValue());
        self.value_dirty.set(false);
        self.upcast::<Node>().dirty(NodeDamage::OtherNodeDamage);
    }

    fn update_placeholder_shown_state(&self) {
        if !self.input_type().is_textual_or_password() {
            return;
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

        let mut files: Vec<DomRoot<File>> = vec![];
        let mut error = None;

        let filter = filter_from_accept(&self.Accept());
        let target = self.upcast::<EventTarget>();

        if self.Multiple() {
            let opt_test_paths =
                opt_test_paths.map(|paths| paths.iter().map(|p| p.to_string()).collect());

            let (chan, recv) = ipc::channel(self.global().time_profiler_chan().clone())
                .expect("Error initializing channel");
            let msg = FileManagerThreadMsg::SelectFiles(filter, chan, origin, opt_test_paths);
            resource_threads
                .send(CoreResourceMsg::ToFileManager(msg))
                .unwrap();

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
                    if paths.is_empty() {
                        return;
                    } else {
                        Some(paths[0].to_string()) // neglect other paths
                    }
                },
                None => None,
            };

            let (chan, recv) = ipc::channel(self.global().time_profiler_chan().clone())
                .expect("Error initializing channel");
            let msg = FileManagerThreadMsg::SelectFile(filter, chan, origin, opt_test_path);
            resource_threads
                .send(CoreResourceMsg::ToFileManager(msg))
                .unwrap();

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

    // https://html.spec.whatwg.org/multipage/#value-sanitization-algorithm
    fn sanitize_value(&self, value: &mut DOMString) {
        // if sanitization_flag is false, we are setting content attributes
        // on an element we haven't really finished creating; we will
        // enable the flag and really sanitize before this element becomes
        // observable.
        if !self.sanitization_flag.get() {
            return;
        }
        match self.input_type() {
            InputType::Text | InputType::Search | InputType::Tel | InputType::Password => {
                value.strip_newlines();
            },
            InputType::Url => {
                value.strip_newlines();
                value.strip_leading_and_trailing_ascii_whitespace();
            },
            InputType::Date => {
                if !value.is_valid_date_string() {
                    value.clear();
                }
            },
            InputType::Month => {
                if !value.is_valid_month_string() {
                    value.clear();
                }
            },
            InputType::Week => {
                if !value.is_valid_week_string() {
                    value.clear();
                }
            },
            InputType::Color => {
                if value.is_valid_simple_color_string() {
                    value.make_ascii_lowercase();
                } else {
                    *value = "#000000".into();
                }
            },
            InputType::Time => {
                if !value.is_valid_time_string() {
                    value.clear();
                }
            },
            InputType::DatetimeLocal => {
                if value
                    .convert_valid_normalized_local_date_and_time_string()
                    .is_none()
                {
                    value.clear();
                }
            },
            InputType::Number => {
                if !value.is_valid_floating_point_number_string() {
                    value.clear();
                }
                // Spec says that user agent "may" round the value
                // when it's suffering a step mismatch, but WPT tests
                // want it unrounded, and this matches other browser
                // behavior (typing an unrounded number into an
                // integer field box and pressing enter generally keeps
                // the number intact but makes the input box :invalid)
            },
            // https://html.spec.whatwg.org/multipage/#range-state-(type=range):value-sanitization-algorithm
            InputType::Range => {
                if !value.is_valid_floating_point_number_string() {
                    *value = DOMString::from(self.default_range_value().to_string());
                }
                if let Ok(fval) = &value.parse::<f64>() {
                    let mut fval = *fval;
                    // comparing max first, because if they contradict
                    // the spec wants min to be the one that applies
                    if let Some(max) = self.maximum() {
                        if fval > max {
                            fval = max;
                        }
                    }
                    if let Some(min) = self.minimum() {
                        if fval < min {
                            fval = min;
                        }
                    }
                    // https://html.spec.whatwg.org/multipage/#range-state-(type=range):suffering-from-a-step-mismatch
                    // Spec does not describe this in a way that lends itself to
                    // reproducible handling of floating-point rounding;
                    // Servo may fail a WPT test because .1 * 6 == 6.000000000000001
                    if let Some(allowed_value_step) = self.allowed_value_step() {
                        let step_base = self.step_base();
                        let steps_from_base = (fval - step_base) / allowed_value_step;
                        if steps_from_base.fract() != 0.0 {
                            // not an integer number of steps, there's a mismatch
                            // round the number of steps...
                            let int_steps = round_halves_positive(steps_from_base);
                            // and snap the value to that rounded value...
                            fval = int_steps * allowed_value_step + step_base;

                            // but if after snapping we're now outside min..max
                            // we have to adjust! (adjusting to min last because
                            // that "wins" over max in the spec)
                            if let Some(stepped_maximum) = self.stepped_maximum() {
                                if fval > stepped_maximum {
                                    fval = stepped_maximum;
                                }
                            }
                            if let Some(stepped_minimum) = self.stepped_minimum() {
                                if fval < stepped_minimum {
                                    fval = stepped_minimum;
                                }
                            }
                        }
                    }
                    *value = DOMString::from(fval.to_string());
                };
            },
            InputType::Email => {
                if !self.Multiple() {
                    value.strip_newlines();
                    value.strip_leading_and_trailing_ascii_whitespace();
                } else {
                    let sanitized = str_join(
                        split_commas(value).map(|token| {
                            let mut token = DOMString::from_string(token.to_string());
                            token.strip_newlines();
                            token.strip_leading_and_trailing_ascii_whitespace();
                            token
                        }),
                        ",",
                    );
                    value.clear();
                    value.push_str(sanitized.as_str());
                }
            },
            // The following inputs don't have a value sanitization algorithm.
            // See https://html.spec.whatwg.org/multipage/#value-sanitization-algorithm
            InputType::Button |
            InputType::Checkbox |
            InputType::File |
            InputType::Hidden |
            InputType::Image |
            InputType::Radio |
            InputType::Reset |
            InputType::Submit => (),
        }
    }

    #[allow(crown::unrooted_must_root)]
    fn selection(&self) -> TextControlSelection<Self> {
        TextControlSelection::new(self, &self.textinput)
    }

    // https://html.spec.whatwg.org/multipage/#implicit-submission
    #[allow(unsafe_code)]
    fn implicit_submission(&self) {
        let doc = document_from_node(self);
        let node = doc.upcast::<Node>();
        let owner = self.form_owner();
        let form = match owner {
            None => return,
            Some(ref f) => f,
        };

        if self.upcast::<Element>().click_in_progress() {
            return;
        }
        let submit_button = node
            .query_selector_iter(DOMString::from("input[type=submit]"))
            .unwrap()
            .filter_map(DomRoot::downcast::<HTMLInputElement>)
            .find(|r| r.form_owner() == owner);
        match submit_button {
            Some(ref button) => {
                if button.is_instance_activatable() {
                    // spec does not actually say to set the not trusted flag,
                    // but we can get here from synthetic keydown events
                    button
                        .upcast::<Node>()
                        .fire_synthetic_mouse_event_not_trusted(DOMString::from("click"));
                }
            },
            None => {
                let mut inputs = node
                    .query_selector_iter(DOMString::from("input"))
                    .unwrap()
                    .filter_map(DomRoot::downcast::<HTMLInputElement>)
                    .filter(|input| {
                        input.form_owner() == owner &&
                            matches!(
                                input.input_type(),
                                InputType::Text |
                                    InputType::Search |
                                    InputType::Url |
                                    InputType::Tel |
                                    InputType::Email |
                                    InputType::Password |
                                    InputType::Date |
                                    InputType::Month |
                                    InputType::Week |
                                    InputType::Time |
                                    InputType::DatetimeLocal |
                                    InputType::Number
                            )
                    });

                if inputs.nth(1).is_some() {
                    // lazily test for > 1 submission-blocking inputs
                    return;
                }
                form.submit(SubmittedFrom::NotFromForm, FormSubmitterElement::Form(form));
            },
        }
    }

    // https://html.spec.whatwg.org/multipage/#concept-input-value-string-number
    fn convert_string_to_number(&self, value: &DOMString) -> Option<f64> {
        match self.input_type() {
            InputType::Date => value
                .parse_date_string()
                .and_then(|(year, month, day)| NaiveDate::from_ymd_opt(year, month, day))
                .and_then(|date| date.and_hms_opt(0, 0, 0))
                .map(|time| time.and_utc().timestamp_millis() as f64),
            InputType::Month => match value.parse_month_string() {
                // This one returns number of months, not milliseconds
                // (specification requires this, presumably because number of
                // milliseconds is not consistent across months)
                // the - 1.0 is because january is 1, not 0
                Some((year, month)) => Some(((year - 1970) * 12) as f64 + (month as f64 - 1.0)),
                _ => None,
            },
            InputType::Week => value
                .parse_week_string()
                .and_then(|(year, weeknum)| NaiveDate::from_isoywd_opt(year, weeknum, Weekday::Mon))
                .and_then(|date| date.and_hms_opt(0, 0, 0))
                .map(|time| time.and_utc().timestamp_millis() as f64),
            InputType::Time => match value.parse_time_string() {
                Some((hours, minutes, seconds)) => {
                    Some((seconds + 60.0 * minutes as f64 + 3600.0 * hours as f64) * 1000.0)
                },
                _ => None,
            },
            InputType::DatetimeLocal => {
                // Is this supposed to know the locale's daylight-savings-time rules?
                value.parse_local_date_and_time_string().and_then(|date| {
                    let seconds = date.seconds as u32;
                    let milliseconds = ((date.seconds - seconds as f64) * 1000.) as u32;
                    Some(
                        NaiveDate::from_ymd_opt(date.year, date.month, date.day)?
                            .and_hms_milli_opt(date.hour, date.minute, seconds, milliseconds)?
                            .and_utc()
                            .timestamp_millis() as f64,
                    )
                })
            },
            InputType::Number | InputType::Range => value.parse_floating_point_number(),
            // min/max/valueAsNumber/stepDown/stepUp do not apply to
            // the remaining types
            _ => None,
        }
    }

    // https://html.spec.whatwg.org/multipage/#concept-input-value-string-number
    fn convert_number_to_string(&self, value: f64) -> Result<DOMString, ()> {
        match self.input_type() {
            InputType::Date => {
                let datetime = milliseconds_to_datetime(value)?;
                Ok(DOMString::from(datetime.format("%Y-%m-%d").to_string()))
            },
            InputType::Month => {
                // interpret value as months(not millis) in epoch, return monthstring
                let year_from_1970 = (value / 12.0).floor();
                let month = (value - year_from_1970 * 12.0).floor() as u32 + 1; // january is 1, not 0
                let year = (year_from_1970 + 1970.0) as u64;
                Ok(DOMString::from(format!("{:04}-{:02}", year, month)))
            },
            InputType::Week => {
                let datetime = milliseconds_to_datetime(value)?;
                let year = datetime.iso_week().year(); // not necessarily the same as datetime.year()
                let week = datetime.iso_week().week();
                Ok(DOMString::from(format!("{:04}-W{:02}", year, week)))
            },
            InputType::Time => {
                let datetime = milliseconds_to_datetime(value)?;
                Ok(DOMString::from(datetime.format("%H:%M:%S%.3f").to_string()))
            },
            InputType::DatetimeLocal => {
                let datetime = milliseconds_to_datetime(value)?;
                Ok(DOMString::from(
                    datetime.format("%Y-%m-%dT%H:%M:%S%.3f").to_string(),
                ))
            },
            InputType::Number | InputType::Range => {
                let mut value = DOMString::from(value.to_string());

                value.set_best_representation_of_the_floating_point_number();

                Ok(value)
            },
            // this won't be called from other input types
            _ => unreachable!(),
        }
    }

    // https://html.spec.whatwg.org/multipage/#concept-input-value-string-date
    // This does the safe Rust part of conversion; the unsafe JS Date part
    // is in GetValueAsDate
    fn convert_string_to_naive_datetime(&self, value: DOMString) -> Option<NaiveDateTime> {
        match self.input_type() {
            InputType::Date => value
                .parse_date_string()
                .and_then(|(y, m, d)| NaiveDate::from_ymd_opt(y, m, d))
                .and_then(|date| date.and_hms_opt(0, 0, 0)),
            InputType::Time => value.parse_time_string().and_then(|(h, m, s)| {
                let whole_seconds = s.floor();
                let nanos = ((s - whole_seconds) * 1e9).floor() as u32;
                NaiveDate::from_ymd_opt(1970, 1, 1)
                    .and_then(|date| date.and_hms_nano_opt(h, m, whole_seconds as u32, nanos))
            }),
            InputType::Week => value
                .parse_week_string()
                .and_then(|(iso_year, week)| {
                    NaiveDate::from_isoywd_opt(iso_year, week, Weekday::Mon)
                })
                .and_then(|date| date.and_hms_opt(0, 0, 0)),
            InputType::Month => value
                .parse_month_string()
                .and_then(|(y, m)| NaiveDate::from_ymd_opt(y, m, 1))
                .and_then(|date| date.and_hms_opt(0, 0, 0)),
            // does not apply to other types
            _ => None,
        }
    }

    // https://html.spec.whatwg.org/multipage/#concept-input-value-date-string
    // This does the safe Rust part of conversion; the unsafe JS Date part
    // is in SetValueAsDate
    fn convert_naive_datetime_to_string(&self, value: NaiveDateTime) -> Result<DOMString, ()> {
        match self.input_type() {
            InputType::Date => Ok(DOMString::from(value.format("%Y-%m-%d").to_string())),
            InputType::Month => Ok(DOMString::from(value.format("%Y-%m").to_string())),
            InputType::Week => {
                let year = value.iso_week().year(); // not necessarily the same as value.year()
                let week = value.iso_week().week();
                Ok(DOMString::from(format!("{:04}-W{:02}", year, week)))
            },
            InputType::Time => Ok(DOMString::from(value.format("%H:%M:%S%.3f").to_string())),
            // this won't be called from other input types
            _ => unreachable!(),
        }
    }
}

impl VirtualMethods for HTMLInputElement {
    fn super_type(&self) -> Option<&dyn VirtualMethods> {
        Some(self.upcast::<HTMLElement>() as &dyn VirtualMethods)
    }

    fn attribute_mutated(&self, attr: &Attr, mutation: AttributeMutation) {
        self.super_type().unwrap().attribute_mutated(attr, mutation);
        match *attr.local_name() {
            local_name!("disabled") => {
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

                if self.input_type().is_textual() {
                    let read_write = !(self.ReadOnly() || el.disabled_state());
                    el.set_read_write_state(read_write);
                }

                el.update_sequentially_focusable_status();
            },
            local_name!("checked") if !self.checked_changed.get() => {
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
            local_name!("size") => {
                let size = mutation.new_value(attr).map(|value| value.as_uint());
                self.size.set(size.unwrap_or(DEFAULT_INPUT_SIZE));
            },
            local_name!("type") => {
                let el = self.upcast::<Element>();
                match mutation {
                    AttributeMutation::Set(_) => {
                        let new_type = InputType::from(attr.value().as_atom());

                        // https://html.spec.whatwg.org/multipage/#input-type-change
                        let (old_value_mode, old_idl_value) = (self.value_mode(), self.Value());
                        let previously_selectable = self.selection_api_applies();

                        self.input_type.set(new_type);

                        if new_type.is_textual() {
                            let read_write = !(self.ReadOnly() || el.disabled_state());
                            el.set_read_write_state(read_write);
                        } else {
                            el.set_read_write_state(false);
                        }

                        if new_type == InputType::File {
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
                            },

                            // Step 2
                            (_, _, ValueMode::Value) if old_value_mode != ValueMode::Value => {
                                self.SetValue(
                                    self.upcast::<Element>()
                                        .get_attribute(&ns!(), &local_name!("value"))
                                        .map_or(DOMString::from(""), |a| {
                                            DOMString::from(a.summarize().value)
                                        }),
                                )
                                .expect(
                                    "Failed to set input value on type change to ValueMode::Value.",
                                );
                                self.value_dirty.set(false);
                            },

                            // Step 3
                            (_, _, ValueMode::Filename)
                                if old_value_mode != ValueMode::Filename =>
                            {
                                self.SetValue(DOMString::from(""))
                                    .expect("Failed to set input value on type change to ValueMode::Filename.");
                            },
                            _ => {},
                        }

                        // Step 5
                        if new_type == InputType::Radio {
                            self.radio_group_updated(self.radio_group_name().as_ref());
                        }

                        // Step 6
                        let mut textinput = self.textinput.borrow_mut();
                        let mut value = textinput.single_line_content().clone();
                        self.sanitize_value(&mut value);
                        textinput.set_content(value);

                        // Steps 7-9
                        if !previously_selectable && self.selection_api_applies() {
                            textinput.clear_selection_to_limit(Direction::Backward);
                        }
                    },
                    AttributeMutation::Removed => {
                        if self.input_type() == InputType::Radio {
                            broadcast_radio_checked(self, self.radio_group_name().as_ref());
                        }
                        self.input_type.set(InputType::default());
                        let el = self.upcast::<Element>();

                        let read_write = !(self.ReadOnly() || el.disabled_state());
                        el.set_read_write_state(read_write);
                    },
                }

                self.update_placeholder_shown_state();
            },
            local_name!("value") if !self.value_dirty.get() => {
                let value = mutation.new_value(attr).map(|value| (**value).to_owned());
                let mut value = value.map_or(DOMString::new(), DOMString::from);

                self.sanitize_value(&mut value);
                self.textinput.borrow_mut().set_content(value);
                self.update_placeholder_shown_state();
            },
            local_name!("name") if self.input_type() == InputType::Radio => {
                self.radio_group_updated(
                    mutation.new_value(attr).as_ref().map(|name| name.as_atom()),
                );
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
                        placeholder
                            .extend(attr.value().chars().filter(|&c| c != '\n' && c != '\r'));
                    }
                }
                self.update_placeholder_shown_state();
            },
            local_name!("readonly") => {
                if self.input_type().is_textual() {
                    let el = self.upcast::<Element>();
                    match mutation {
                        AttributeMutation::Set(_) => {
                            el.set_read_write_state(false);
                        },
                        AttributeMutation::Removed => {
                            el.set_read_write_state(!el.disabled_state());
                        },
                    }
                }
            },
            local_name!("form") => {
                self.form_attribute_mutated(mutation);
            },
            _ => {},
        }

        self.validity_state()
            .perform_validation_and_update(ValidationFlags::all());
    }

    fn parse_plain_attribute(&self, name: &LocalName, value: DOMString) -> AttrValue {
        match *name {
            local_name!("accept") => AttrValue::from_comma_separated_tokenlist(value.into()),
            local_name!("size") => AttrValue::from_limited_u32(value.into(), DEFAULT_INPUT_SIZE),
            local_name!("type") => AttrValue::from_atomic(value.into()),
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

    fn bind_to_tree(&self, context: &BindContext) {
        if let Some(s) = self.super_type() {
            s.bind_to_tree(context);
        }
        self.upcast::<Element>()
            .check_ancestors_disabled_state_for_form_control();

        self.validity_state()
            .perform_validation_and_update(ValidationFlags::all());
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

        self.validity_state()
            .perform_validation_and_update(ValidationFlags::all());
    }

    // This represents behavior for which the UIEvents spec and the
    // DOM/HTML specs are out of sync.
    // Compare:
    // https://w3c.github.io/uievents/#default-action
    // https://dom.spec.whatwg.org/#action-versus-occurance
    fn handle_event(&self, event: &Event) {
        if let Some(s) = self.super_type() {
            s.handle_event(event);
        }

        if event.type_() == atom!("click") && !event.DefaultPrevented() {
            // WHATWG-specified activation behaviors are handled elsewhere;
            // this is for all the other things a UI click might do

            //TODO: set the editing position for text inputs

            if self.input_type().is_textual_or_password() &&
                // Check if we display a placeholder. Layout doesn't know about this.
                !self.textinput.borrow().is_empty()
            {
                if let Some(mouse_event) = event.downcast::<MouseEvent>() {
                    // dispatch_key_event (document.rs) triggers a click event when releasing
                    // the space key. There's no nice way to catch this so let's use this for
                    // now.
                    if let Some(point_in_target) = mouse_event.point_in_target() {
                        let window = window_from_node(self);
                        let index = window.text_index_query(self.upcast::<Node>(), point_in_target);
                        if let Some(i) = index {
                            self.textinput.borrow_mut().set_edit_point_index(i);
                            // trigger redraw
                            self.upcast::<Node>().dirty(NodeDamage::OtherNodeDamage);
                            event.PreventDefault();
                        }
                    }
                }
            }
        } else if event.type_() == atom!("keydown") &&
            !event.DefaultPrevented() &&
            self.input_type().is_textual_or_password()
        {
            if let Some(keyevent) = event.downcast::<KeyboardEvent>() {
                // This can't be inlined, as holding on to textinput.borrow_mut()
                // during self.implicit_submission will cause a panic.
                let action = self.textinput.borrow_mut().handle_keydown(keyevent);
                match action {
                    TriggerDefaultAction => {
                        self.implicit_submission();
                    },
                    DispatchInput => {
                        self.value_dirty.set(true);
                        self.update_placeholder_shown_state();
                        self.upcast::<Node>().dirty(NodeDamage::OtherNodeDamage);
                        event.mark_as_handled();
                    },
                    RedrawSelection => {
                        self.upcast::<Node>().dirty(NodeDamage::OtherNodeDamage);
                        event.mark_as_handled();
                    },
                    Nothing => (),
                }
            }
        } else if event.type_() == atom!("keypress") &&
            !event.DefaultPrevented() &&
            self.input_type().is_textual_or_password()
        {
            if event.IsTrusted() {
                let window = window_from_node(self);
                window
                    .task_manager()
                    .user_interaction_task_source()
                    .queue_event(
                        self.upcast(),
                        atom!("input"),
                        EventBubbles::Bubbles,
                        EventCancelable::NotCancelable,
                        &window,
                    );
            }
        } else if (event.type_() == atom!("compositionstart") ||
            event.type_() == atom!("compositionupdate") ||
            event.type_() == atom!("compositionend")) &&
            self.input_type().is_textual_or_password()
        {
            // TODO: Update DOM on start and continue
            // and generally do proper CompositionEvent handling.
            if let Some(compositionevent) = event.downcast::<CompositionEvent>() {
                if event.type_() == atom!("compositionend") {
                    let _ = self
                        .textinput
                        .borrow_mut()
                        .handle_compositionend(compositionevent);
                    self.upcast::<Node>().dirty(NodeDamage::OtherNodeDamage);
                }
                event.mark_as_handled();
            }
        }

        self.validity_state()
            .perform_validation_and_update(ValidationFlags::all());
    }

    // https://html.spec.whatwg.org/multipage/#the-input-element%3Aconcept-node-clone-ext
    fn cloning_steps(
        &self,
        copy: &Node,
        maybe_doc: Option<&Document>,
        clone_children: CloneChildrenFlag,
    ) {
        if let Some(s) = self.super_type() {
            s.cloning_steps(copy, maybe_doc, clone_children);
        }
        let elem = copy.downcast::<HTMLInputElement>().unwrap();
        elem.value_dirty.set(self.value_dirty.get());
        elem.checked_changed.set(self.checked_changed.get());
        elem.upcast::<Element>()
            .set_state(ElementState::CHECKED, self.Checked());
        elem.textinput
            .borrow_mut()
            .set_content(self.textinput.borrow().get_content());
        elem.validity_state()
            .perform_validation_and_update(ValidationFlags::all());
    }
}

impl FormControl for HTMLInputElement {
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

impl Validatable for HTMLInputElement {
    fn as_element(&self) -> &Element {
        self.upcast()
    }

    fn validity_state(&self) -> DomRoot<ValidityState> {
        self.validity_state
            .or_init(|| ValidityState::new(&window_from_node(self), self.upcast()))
    }

    fn is_instance_validatable(&self) -> bool {
        // https://html.spec.whatwg.org/multipage/#hidden-state-(type%3Dhidden)%3Abarred-from-constraint-validation
        // https://html.spec.whatwg.org/multipage/#button-state-(type%3Dbutton)%3Abarred-from-constraint-validation
        // https://html.spec.whatwg.org/multipage/#reset-button-state-(type%3Dreset)%3Abarred-from-constraint-validation
        // https://html.spec.whatwg.org/multipage/#enabling-and-disabling-form-controls%3A-the-disabled-attribute%3Abarred-from-constraint-validation
        // https://html.spec.whatwg.org/multipage/#the-readonly-attribute%3Abarred-from-constraint-validation
        // https://html.spec.whatwg.org/multipage/#the-datalist-element%3Abarred-from-constraint-validation
        match self.input_type() {
            InputType::Hidden | InputType::Button | InputType::Reset => false,
            _ => {
                !(self.upcast::<Element>().disabled_state() ||
                    (self.ReadOnly() && self.does_readonly_apply()) ||
                    is_barred_by_datalist_ancestor(self.upcast()))
            },
        }
    }

    fn perform_validation(&self, validate_flags: ValidationFlags) -> ValidationFlags {
        let mut failed_flags = ValidationFlags::empty();
        let value = self.Value();

        if validate_flags.contains(ValidationFlags::VALUE_MISSING) &&
            self.suffers_from_being_missing(&value)
        {
            failed_flags.insert(ValidationFlags::VALUE_MISSING);
        }

        if validate_flags.contains(ValidationFlags::TYPE_MISMATCH) &&
            self.suffers_from_type_mismatch(&value)
        {
            failed_flags.insert(ValidationFlags::TYPE_MISMATCH);
        }

        if validate_flags.contains(ValidationFlags::PATTERN_MISMATCH) &&
            self.suffers_from_pattern_mismatch(&value)
        {
            failed_flags.insert(ValidationFlags::PATTERN_MISMATCH);
        }

        if validate_flags.contains(ValidationFlags::BAD_INPUT) &&
            self.suffers_from_bad_input(&value)
        {
            failed_flags.insert(ValidationFlags::BAD_INPUT);
        }

        if validate_flags.intersects(ValidationFlags::TOO_LONG | ValidationFlags::TOO_SHORT) {
            failed_flags |= self.suffers_from_length_issues(&value);
        }

        if validate_flags.intersects(
            ValidationFlags::RANGE_UNDERFLOW |
                ValidationFlags::RANGE_OVERFLOW |
                ValidationFlags::STEP_MISMATCH,
        ) {
            failed_flags |= self.suffers_from_range_issues(&value);
        }

        failed_flags & validate_flags
    }
}

impl Activatable for HTMLInputElement {
    fn as_element(&self) -> &Element {
        self.upcast()
    }

    fn is_instance_activatable(&self) -> bool {
        match self.input_type() {
            // https://html.spec.whatwg.org/multipage/#submit-button-state-%28type=submit%29:activation-behaviour-2
            // https://html.spec.whatwg.org/multipage/#reset-button-state-%28type=reset%29:activation-behaviour-2
            // https://html.spec.whatwg.org/multipage/#checkbox-state-%28type=checkbox%29:activation-behaviour-2
            // https://html.spec.whatwg.org/multipage/#radio-button-state-%28type=radio%29:activation-behaviour-2
            InputType::Submit | InputType::Reset | InputType::File => self.is_mutable(),
            InputType::Checkbox | InputType::Radio => true,
            _ => false,
        }
    }

    // https://dom.spec.whatwg.org/#eventtarget-legacy-pre-activation-behavior
    fn legacy_pre_activation_behavior(&self) -> Option<InputActivationState> {
        let ty = self.input_type();
        match ty {
            InputType::Checkbox => {
                let was_checked = self.Checked();
                let was_indeterminate = self.Indeterminate();
                self.SetIndeterminate(false);
                self.SetChecked(!was_checked);
                return Some(InputActivationState {
                    checked: was_checked,
                    indeterminate: was_indeterminate,
                    checked_radio: None,
                    old_type: InputType::Checkbox,
                });
            },
            InputType::Radio => {
                let checked_member =
                    radio_group_iter(self, self.radio_group_name().as_ref()).find(|r| r.Checked());
                let was_checked = self.Checked();
                self.SetChecked(true);
                return Some(InputActivationState {
                    checked: was_checked,
                    indeterminate: false,
                    checked_radio: checked_member.as_deref().map(DomRoot::from_ref),
                    old_type: InputType::Radio,
                });
            },
            _ => (),
        }
        None
    }

    // https://dom.spec.whatwg.org/#eventtarget-legacy-canceled-activation-behavior
    fn legacy_canceled_activation_behavior(&self, cache: Option<InputActivationState>) {
        // Step 1
        let ty = self.input_type();
        let cache = match cache {
            Some(cache) => {
                if cache.old_type != ty {
                    // Type changed, abandon ship
                    // https://www.w3.org/Bugs/Public/show_bug.cgi?id=27414
                    return;
                }
                cache
            },
            None => {
                return;
            },
        };

        match ty {
            // Step 2
            InputType::Checkbox => {
                self.SetIndeterminate(cache.indeterminate);
                self.SetChecked(cache.checked);
            },
            // Step 3
            InputType::Radio => {
                if let Some(ref o) = cache.checked_radio {
                    let tree_root = self
                        .upcast::<Node>()
                        .GetRootNode(&GetRootNodeOptions::empty());
                    // Avoiding iterating through the whole tree here, instead
                    // we can check if the conditions for radio group siblings apply
                    if in_same_group(
                        o,
                        self.form_owner().as_deref(),
                        self.radio_group_name().as_ref(),
                        Some(&*tree_root),
                    ) {
                        o.SetChecked(true);
                    } else {
                        self.SetChecked(false);
                    }
                } else {
                    self.SetChecked(false);
                }
            },
            _ => (),
        }
    }

    // https://html.spec.whatwg.org/multipage/#run-post-click-activation-steps
    fn activation_behavior(&self, _event: &Event, _target: &EventTarget) {
        let ty = self.input_type();
        match ty {
            InputType::Submit => {
                // https://html.spec.whatwg.org/multipage/#submit-button-state-(type=submit):activation-behavior
                // FIXME (Manishearth): support document owners (needs ability to get parent browsing context)
                // Check if document owner is fully active
                if let Some(o) = self.form_owner() {
                    o.submit(
                        SubmittedFrom::NotFromForm,
                        FormSubmitterElement::Input(self),
                    )
                }
            },
            InputType::Reset => {
                // https://html.spec.whatwg.org/multipage/#reset-button-state-(type=reset):activation-behavior
                // FIXME (Manishearth): support document owners (needs ability to get parent browsing context)
                // Check if document owner is fully active
                if let Some(o) = self.form_owner() {
                    o.reset(ResetFrom::NotFromForm)
                }
            },
            InputType::Checkbox | InputType::Radio => {
                // https://html.spec.whatwg.org/multipage/#checkbox-state-(type=checkbox):activation-behavior
                // https://html.spec.whatwg.org/multipage/#radio-button-state-(type=radio):activation-behavior
                // Check if document owner is fully active
                if !self.upcast::<Node>().is_connected() {
                    return;
                }
                let target = self.upcast::<EventTarget>();
                target.fire_bubbling_event(atom!("input"));
                target.fire_bubbling_event(atom!("change"));
            },
            InputType::File => self.select_files(None),
            _ => (),
        }
    }
}

// https://html.spec.whatwg.org/multipage/#attr-input-accept
fn filter_from_accept(s: &DOMString) -> Vec<FilterPattern> {
    let mut filter = vec![];
    for p in split_commas(s) {
        if let Some('.') = p.chars().next() {
            filter.push(FilterPattern(p[1..].to_string()));
        } else if let Some(exts) = mime_guess::get_mime_extensions_str(p) {
            for ext in exts {
                filter.push(FilterPattern(ext.to_string()));
            }
        }
    }

    filter
}

fn round_halves_positive(n: f64) -> f64 {
    // WHATWG specs about input steps say to round to the nearest step,
    // rounding halves always to positive infinity.
    // This differs from Rust's .round() in the case of -X.5.
    if n.fract() == -0.5 {
        n.ceil()
    } else {
        n.round()
    }
}

fn milliseconds_to_datetime(value: f64) -> Result<NaiveDateTime, ()> {
    let seconds = (value / 1000.0).floor();
    let milliseconds = value - (seconds * 1000.0);
    let nanoseconds = milliseconds * 1e6;
    match DateTime::from_timestamp(seconds as i64, nanoseconds as u32) {
        Some(datetime) => Ok(datetime.naive_utc()),
        None => Err(()),
    }
}

// This is used to compile JS-compatible regex provided in pattern attribute
// that matches only the entirety of string.
// https://html.spec.whatwg.org/multipage/#compiled-pattern-regular-expression
fn compile_pattern(cx: SafeJSContext, pattern_str: &str, out_regex: MutableHandleObject) -> bool {
    // First check if pattern compiles...
    if new_js_regex(cx, pattern_str, out_regex) {
        // ...and if it does make pattern that matches only the entirety of string
        let pattern_str = format!("^(?:{})$", pattern_str);
        new_js_regex(cx, &pattern_str, out_regex)
    } else {
        false
    }
}

#[allow(unsafe_code)]
fn new_js_regex(cx: SafeJSContext, pattern: &str, mut out_regex: MutableHandleObject) -> bool {
    let pattern: Vec<u16> = pattern.encode_utf16().collect();
    unsafe {
        out_regex.set(NewUCRegExpObject(
            *cx,
            pattern.as_ptr(),
            pattern.len(),
            RegExpFlags {
                flags_: RegExpFlag_Unicode,
            },
        ));
        if out_regex.is_null() {
            JS_ClearPendingException(*cx);
            return false;
        }
    }
    true
}

#[allow(unsafe_code)]
fn matches_js_regex(cx: SafeJSContext, regex_obj: HandleObject, value: &str) -> Result<bool, ()> {
    let mut value: Vec<u16> = value.encode_utf16().collect();

    unsafe {
        let mut is_regex = false;
        assert!(ObjectIsRegExp(*cx, regex_obj, &mut is_regex));
        assert!(is_regex);

        rooted!(in(*cx) let mut rval = UndefinedValue());
        let mut index = 0;

        let ok = ExecuteRegExpNoStatics(
            *cx,
            regex_obj,
            value.as_mut_ptr(),
            value.len(),
            &mut index,
            true,
            &mut rval.handle_mut(),
        );

        if ok {
            Ok(!rval.is_null())
        } else {
            JS_ClearPendingException(*cx);
            Err(())
        }
    }
}

/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::Cell;

use dom_struct::dom_struct;
use html5ever::local_name;

use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::ElementInternalsBinding::{
    ElementInternalsMethods, ValidityStateFlags,
};
use crate::dom::bindings::codegen::UnionTypes::FileOrUSVStringOrFormData;
use crate::dom::bindings::error::{Error, ErrorResult, Fallible};
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::reflector::{reflect_dom_object, Reflector};
use crate::dom::bindings::root::{Dom, DomRoot, MutNullableDom};
use crate::dom::bindings::str::{DOMString, USVString};
use crate::dom::element::Element;
use crate::dom::file::File;
use crate::dom::htmlelement::HTMLElement;
use crate::dom::htmlformelement::{FormDatum, FormDatumValue, HTMLFormElement};
use crate::dom::node::{window_from_node, Node};
use crate::dom::nodelist::NodeList;
use crate::dom::validation::{is_barred_by_datalist_ancestor, Validatable};
use crate::dom::validitystate::{ValidationFlags, ValidityState};

#[derive(Clone, JSTraceable, MallocSizeOf)]
enum SubmissionValue {
    File(DomRoot<File>),
    FormData(Vec<FormDatum>),
    USVString(USVString),
    None,
}

#[dom_struct]
pub struct ElementInternals {
    reflector_: Reflector,
    // if attached is false, we're using this to hold form-related state
    // on an element for which attachInternals() wasn't called yet; this is
    // necessary because it might have a form owner.
    attached: Cell<bool>,
    target_element: Dom<HTMLElement>,
    validity_state: MutNullableDom<ValidityState>,
    validation_message: DomRefCell<DOMString>,
    custom_validity_error_message: DomRefCell<DOMString>,
    validation_anchor: MutNullableDom<HTMLElement>,
    submission_value: DomRefCell<SubmissionValue>,
    state: DomRefCell<SubmissionValue>,
    form_owner: MutNullableDom<HTMLFormElement>,
    labels_node_list: MutNullableDom<NodeList>,
}

impl ElementInternals {
    fn new_inherited(element: &HTMLElement) -> ElementInternals {
        ElementInternals {
            reflector_: Reflector::new(),
            attached: Cell::new(false),
            target_element: Dom::from_ref(element),
            validity_state: Default::default(),
            validation_message: DomRefCell::new(DOMString::new()),
            custom_validity_error_message: DomRefCell::new(DOMString::new()),
            validation_anchor: MutNullableDom::new(None),
            submission_value: DomRefCell::new(SubmissionValue::None),
            state: DomRefCell::new(SubmissionValue::None),
            form_owner: MutNullableDom::new(None),
            labels_node_list: MutNullableDom::new(None),
        }
    }

    pub fn new(element: &HTMLElement) -> DomRoot<ElementInternals> {
        let global = window_from_node(element);
        reflect_dom_object(Box::new(ElementInternals::new_inherited(element)), &*global)
    }

    fn is_target_form_associated(&self) -> bool {
        self.target_element.is_form_associated_custom_element()
    }

    fn set_validation_message(&self, message: DOMString) {
        *self.validation_message.borrow_mut() = message;
    }

    fn set_custom_validity_error_message(&self, message: DOMString) {
        *self.custom_validity_error_message.borrow_mut() = message;
    }

    fn set_submission_value(&self, value: SubmissionValue) {
        *self.submission_value.borrow_mut() = value;
    }

    fn set_state(&self, value: SubmissionValue) {
        *self.state.borrow_mut() = value;
    }

    pub fn set_form_owner(&self, form: Option<&HTMLFormElement>) {
        self.form_owner.set(form);
    }

    pub fn form_owner(&self) -> Option<DomRoot<HTMLFormElement>> {
        self.form_owner.get()
    }

    pub fn set_attached(&self) {
        self.attached.set(true);
    }

    pub fn attached(&self) -> bool {
        self.attached.get()
    }

    pub fn perform_entry_construction(&self, entry_list: &mut Vec<FormDatum>) {
        if self
            .target_element
            .upcast::<Element>()
            .has_attribute(&local_name!("disabled"))
        {
            warn!("We are in perform_entry_construction on an element with disabled attribute!");
        }
        if self.target_element.upcast::<Element>().disabled_state() {
            warn!("We are in perform_entry_construction on an element with disabled bit!");
        }
        if !self.target_element.upcast::<Element>().enabled_state() {
            warn!("We are in perform_entry_construction on an element without enabled bit!");
        }

        if let SubmissionValue::FormData(datums) = &*self.submission_value.borrow() {
            entry_list.extend(datums.iter().map(|d| d.clone()));
            return;
        }
        let name = self
            .target_element
            .upcast::<Element>()
            .get_string_attribute(&local_name!("name"));
        if name == "" {
            return;
        }
        match &*self.submission_value.borrow() {
            SubmissionValue::FormData(_) => unreachable!(),
            SubmissionValue::None => {},
            SubmissionValue::USVString(s) => {
                entry_list.push(FormDatum {
                    ty: DOMString::from("string"),
                    name: name,
                    value: FormDatumValue::String(DOMString::from(s.to_string())),
                });
            },
            SubmissionValue::File(f) => {
                entry_list.push(FormDatum {
                    ty: DOMString::from("file"),
                    name: name,
                    value: FormDatumValue::File(DomRoot::from_ref(&*f)),
                });
            },
        }
    }
}

impl ElementInternalsMethods for ElementInternals {
    /// <https://html.spec.whatwg.org/multipage#dom-elementinternals-setformvalue>
    fn SetFormValue(
        &self,
        value: Option<FileOrUSVStringOrFormData>,
        maybe_state: Option<Option<FileOrUSVStringOrFormData>>,
    ) -> ErrorResult {
        // Steps 1-2: If element is not a form-associated custom element, then throw a "NotSupportedError" DOMException
        if !self.is_target_form_associated() {
            return Err(Error::NotSupported);
        }

        // Step 3: Set target element's submission value
        self.set_submission_value(submission_value_from(&value));

        match maybe_state {
            // Step 4: If the state argument of the function is omitted, set element's state to its submission value
            None => self.set_state(submission_value_from(&value)),
            // Steps 5-6: Otherwise, set element's state to state
            Some(state) => self.set_state(submission_value_from(&state)),
        }
        Ok(())
    }

    /// <https://html.spec.whatwg.org/multipage#dom-elementinternals-setvalidity>
    fn SetValidity(
        &self,
        flags: &ValidityStateFlags,
        message: Option<DOMString>,
        anchor: Option<&HTMLElement>,
    ) -> ErrorResult {
        // Steps 1-2: Check form-associated custom element
        if !self.is_target_form_associated() {
            return Err(Error::NotSupported);
        }

        // Step 3: Check bits and message
        let bits = validity_bits(flags);
        if !bits.is_empty() && !message.iter().any(|m| m.len() > 0) {
            return Err(Error::Type(
                "Setting an element to invalid requires a message string as the second argument."
                    .to_string(),
            ));
        }

        // Step 4: For each entry flag → value of flags, set element's validity flag with the name flag to value
        self.validity_state().update_invalid_flags(bits);

        // Step 5: Set element's validation message to the empty string
        if bits.is_empty() {
            self.set_validation_message(DOMString::new());
        } else {
            self.set_validation_message(message.unwrap_or_else(|| DOMString::new()));
        }

        // Step 6: set element's custom validity error message to element's validation message
        if bits.contains(ValidationFlags::CUSTOM_ERROR) {
            self.set_custom_validity_error_message(self.validation_message.borrow().clone());
        } else {
            self.set_custom_validity_error_message(DOMString::new());
        }

        // Step 7: Set element's validation anchor to null
        match anchor {
            None => self.validation_anchor.set(None),
            Some(a) => {
                if a == &*self.target_element ||
                    !self
                        .target_element
                        .upcast::<Node>()
                        .is_shadow_including_inclusive_ancestor_of(a.upcast::<Node>())
                {
                    return Err(Error::NotFound);
                }
                self.validation_anchor.set(Some(a));
            },
        }
        Ok(())
    }

    /// <https://html.spec.whatwg.org/multipage#dom-elementinternals-validationmessage>
    fn GetValidationMessage(&self) -> Fallible<DOMString> {
        // This check isn't in the spec but it's in WPT tests and it maintains
        // consistency with other methods that do specify it
        if !self.is_target_form_associated() {
            return Err(Error::NotSupported);
        }
        Ok(self.validation_message.borrow().clone())
    }

    /// <https://html.spec.whatwg.org/multipage#dom-elementinternals-validity>
    fn GetValidity(&self) -> Fallible<DomRoot<ValidityState>> {
        if !self.is_target_form_associated() {
            return Err(Error::NotSupported);
        }
        Ok(self.validity_state())
    }

    /// <https://html.spec.whatwg.org/multipage#dom-elementinternals-labels>
    fn GetLabels(&self) -> Fallible<DomRoot<NodeList>> {
        if !self.is_target_form_associated() {
            return Err(Error::NotSupported);
        }
        Ok(self.labels_node_list.or_init(|| {
            NodeList::new_labels_list(
                self.target_element.upcast::<Node>().owner_doc().window(),
                &*self.target_element,
            )
        }))
    }

    /// <https://html.spec.whatwg.org/multipage#dom-elementinternals-willvalidate>
    fn GetWillValidate(&self) -> Fallible<bool> {
        if !self.is_target_form_associated() {
            return Err(Error::NotSupported);
        }
        Ok(self.is_instance_validatable())
    }

    /// <https://html.spec.whatwg.org/multipage#dom-elementinternals-form>
    fn GetForm(&self) -> Fallible<Option<DomRoot<HTMLFormElement>>> {
        if !self.is_target_form_associated() {
            return Err(Error::NotSupported);
        }
        Ok(self.form_owner.get())
    }

    /// <https://html.spec.whatwg.org/multipage#dom-elementinternals-checkvalidity>
    fn CheckValidity(&self) -> Fallible<bool> {
        if !self.is_target_form_associated() {
            return Err(Error::NotSupported);
        }
        Ok(self.check_validity())
    }

    /// <https://html.spec.whatwg.org/multipage#dom-elementinternals-reportvalidity>
    fn ReportValidity(&self) -> Fallible<bool> {
        if !self.is_target_form_associated() {
            return Err(Error::NotSupported);
        }
        Ok(self.report_validity())
    }
}

fn submission_value_from(value: &Option<FileOrUSVStringOrFormData>) -> SubmissionValue {
    match value {
        None => SubmissionValue::None,
        Some(FileOrUSVStringOrFormData::File(f)) => SubmissionValue::File(DomRoot::from_ref(f)),
        Some(FileOrUSVStringOrFormData::USVString(s)) => SubmissionValue::USVString(s.clone()),
        Some(FileOrUSVStringOrFormData::FormData(fd)) => SubmissionValue::FormData(fd.datums()),
    }
}

fn validity_bits(flags: &ValidityStateFlags) -> ValidationFlags {
    let mut bits = ValidationFlags::empty();
    if flags.valueMissing {
        bits |= ValidationFlags::VALUE_MISSING;
    }
    if flags.typeMismatch {
        bits |= ValidationFlags::TYPE_MISMATCH;
    }
    if flags.patternMismatch {
        bits |= ValidationFlags::PATTERN_MISMATCH;
    }
    if flags.tooLong {
        bits |= ValidationFlags::TOO_LONG;
    }
    if flags.tooShort {
        bits |= ValidationFlags::TOO_SHORT;
    }
    if flags.rangeUnderflow {
        bits |= ValidationFlags::RANGE_UNDERFLOW;
    }
    if flags.rangeOverflow {
        bits |= ValidationFlags::RANGE_OVERFLOW;
    }
    if flags.stepMismatch {
        bits |= ValidationFlags::STEP_MISMATCH;
    }
    if flags.badInput {
        bits |= ValidationFlags::BAD_INPUT;
    }
    if flags.customError {
        bits |= ValidationFlags::CUSTOM_ERROR;
    }
    bits
}

// Form-associated custom elements also need the Validatable trait.
impl Validatable for ElementInternals {
    fn as_element(&self) -> &Element {
        debug_assert!(self.is_target_form_associated());
        self.target_element.upcast::<Element>()
    }

    fn validity_state(&self) -> DomRoot<ValidityState> {
        self.validity_state.or_init(|| {
            ValidityState::new(
                &window_from_node(self.target_element.upcast::<Node>()),
                self.target_element.upcast(),
            )
        })
    }

    /// <https://html.spec.whatwg.org/multipage/#candidate-for-constraint-validation>
    fn is_instance_validatable(&self) -> bool {
        if !self.target_element.is_submittable_element() {
            return false;
        }

        // The form-associated custom element is barred from constraint validation,
        // if the readonly attribute is specified, the element is disabled,
        // or the element has a datalist element ancestor.
        !self.as_element().read_write_state() &&
            !self.as_element().disabled_state() &&
            !is_barred_by_datalist_ancestor(self.target_element.upcast::<Node>())
    }
}

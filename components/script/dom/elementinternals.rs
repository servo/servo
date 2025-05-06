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
use crate::dom::bindings::reflector::{Reflector, reflect_dom_object};
use crate::dom::bindings::root::{Dom, DomRoot, MutNullableDom};
use crate::dom::bindings::str::{DOMString, USVString};
use crate::dom::element::Element;
use crate::dom::file::File;
use crate::dom::htmlelement::HTMLElement;
use crate::dom::htmlformelement::{FormDatum, FormDatumValue, HTMLFormElement};
use crate::dom::node::{Node, NodeTraits};
use crate::dom::nodelist::NodeList;
use crate::dom::shadowroot::ShadowRoot;
use crate::dom::validation::{Validatable, is_barred_by_datalist_ancestor};
use crate::dom::validitystate::{ValidationFlags, ValidityState};
use crate::script_runtime::CanGc;

#[derive(Clone, JSTraceable, MallocSizeOf)]
enum SubmissionValue {
    File(DomRoot<File>),
    FormData(Vec<FormDatum>),
    USVString(USVString),
    None,
}

impl From<Option<&FileOrUSVStringOrFormData>> for SubmissionValue {
    fn from(value: Option<&FileOrUSVStringOrFormData>) -> Self {
        match value {
            None => SubmissionValue::None,
            Some(FileOrUSVStringOrFormData::File(file)) => {
                SubmissionValue::File(DomRoot::from_ref(file))
            },
            Some(FileOrUSVStringOrFormData::USVString(usv_string)) => {
                SubmissionValue::USVString(usv_string.clone())
            },
            Some(FileOrUSVStringOrFormData::FormData(form_data)) => {
                SubmissionValue::FormData(form_data.datums())
            },
        }
    }
}

#[dom_struct]
pub(crate) struct ElementInternals {
    reflector_: Reflector,
    /// If `attached` is false, we're using this to hold form-related state
    /// on an element for which `attachInternals()` wasn't called yet; this is
    /// necessary because it might have a form owner.
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
    fn new_inherited(target_element: &HTMLElement) -> ElementInternals {
        ElementInternals {
            reflector_: Reflector::new(),
            attached: Cell::new(false),
            target_element: Dom::from_ref(target_element),
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

    pub(crate) fn new(element: &HTMLElement, can_gc: CanGc) -> DomRoot<ElementInternals> {
        let global = element.owner_window();
        reflect_dom_object(
            Box::new(ElementInternals::new_inherited(element)),
            &*global,
            can_gc,
        )
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

    pub(crate) fn set_form_owner(&self, form: Option<&HTMLFormElement>) {
        self.form_owner.set(form);
    }

    pub(crate) fn form_owner(&self) -> Option<DomRoot<HTMLFormElement>> {
        self.form_owner.get()
    }

    pub(crate) fn set_attached(&self) {
        self.attached.set(true);
    }

    pub(crate) fn attached(&self) -> bool {
        self.attached.get()
    }

    pub(crate) fn perform_entry_construction(&self, entry_list: &mut Vec<FormDatum>) {
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
            entry_list.extend(datums.iter().cloned());
            return;
        }
        let name = self
            .target_element
            .upcast::<Element>()
            .get_string_attribute(&local_name!("name"));
        if name.is_empty() {
            return;
        }
        match &*self.submission_value.borrow() {
            SubmissionValue::FormData(_) => unreachable!(
                "The FormData submission value has been handled before name empty checking"
            ),
            SubmissionValue::None => {},
            SubmissionValue::USVString(string) => {
                entry_list.push(FormDatum {
                    ty: DOMString::from("string"),
                    name,
                    value: FormDatumValue::String(DOMString::from(string.to_string())),
                });
            },
            SubmissionValue::File(file) => {
                entry_list.push(FormDatum {
                    ty: DOMString::from("file"),
                    name,
                    value: FormDatumValue::File(DomRoot::from_ref(file)),
                });
            },
        }
    }

    pub(crate) fn is_invalid(&self) -> bool {
        self.is_target_form_associated() &&
            self.is_instance_validatable() &&
            !self.satisfies_constraints()
    }
}

impl ElementInternalsMethods<crate::DomTypeHolder> for ElementInternals {
    /// <https://html.spec.whatwg.org/multipage/#dom-elementinternals-shadowroot>
    fn GetShadowRoot(&self) -> Option<DomRoot<ShadowRoot>> {
        // Step 1. Let target be this's target element.
        // Step 2. If target is not a shadow host, then return null.
        // Step 3. Let shadow be target's shadow root.
        let shadow = self.target_element.upcast::<Element>().shadow_root()?;

        // Step 4. If shadow's available to element internals is false, then return null.
        if !shadow.is_available_to_element_internals() {
            return None;
        }

        // Step 5. Return shadow.
        Some(shadow)
    }

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
        self.set_submission_value(value.as_ref().into());

        match maybe_state {
            // Step 4: If the state argument of the function is omitted, set element's state to its submission value
            None => self.set_state(value.as_ref().into()),
            // Steps 5-6: Otherwise, set element's state to state
            Some(state) => self.set_state(state.as_ref().into()),
        }
        Ok(())
    }

    /// <https://html.spec.whatwg.org/multipage#dom-elementinternals-setvalidity>
    fn SetValidity(
        &self,
        flags: &ValidityStateFlags,
        message: Option<DOMString>,
        anchor: Option<&HTMLElement>,
        can_gc: CanGc,
    ) -> ErrorResult {
        // Steps 1-2: Check form-associated custom element
        if !self.is_target_form_associated() {
            return Err(Error::NotSupported);
        }

        // Step 3: If flags contains one or more true values and message is not given or is the empty
        // string, then throw a TypeError.
        let bits: ValidationFlags = flags.into();
        if !bits.is_empty() && !message.as_ref().map_or_else(|| false, |m| !m.is_empty()) {
            return Err(Error::Type(
                "Setting an element to invalid requires a message string as the second argument."
                    .to_string(),
            ));
        }

        // Step 4: For each entry `flag` → `value` of `flags`, set element's validity flag with the name
        // `flag` to `value`.
        self.validity_state().update_invalid_flags(bits);
        self.validity_state().update_pseudo_classes(can_gc);

        // Step 5: Set element's validation message to the empty string if message is not given
        // or all of element's validity flags are false, or to message otherwise.
        if bits.is_empty() {
            self.set_validation_message(DOMString::new());
        } else {
            self.set_validation_message(message.unwrap_or_default());
        }

        // Step 6: If element's customError validity flag is true, then set element's custom validity error
        // message to element's validation message. Otherwise, set element's custom validity error
        // message to the empty string.
        if bits.contains(ValidationFlags::CUSTOM_ERROR) {
            self.set_custom_validity_error_message(self.validation_message.borrow().clone());
        } else {
            self.set_custom_validity_error_message(DOMString::new());
        }

        // Step 7: Set element's validation anchor to null if anchor is not given.
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
    fn GetLabels(&self, can_gc: CanGc) -> Fallible<DomRoot<NodeList>> {
        if !self.is_target_form_associated() {
            return Err(Error::NotSupported);
        }
        Ok(self.labels_node_list.or_init(|| {
            NodeList::new_labels_list(
                self.target_element.upcast::<Node>().owner_doc().window(),
                &self.target_element,
                can_gc,
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
    fn CheckValidity(&self, can_gc: CanGc) -> Fallible<bool> {
        if !self.is_target_form_associated() {
            return Err(Error::NotSupported);
        }
        Ok(self.check_validity(can_gc))
    }

    /// <https://html.spec.whatwg.org/multipage#dom-elementinternals-reportvalidity>
    fn ReportValidity(&self, can_gc: CanGc) -> Fallible<bool> {
        if !self.is_target_form_associated() {
            return Err(Error::NotSupported);
        }
        Ok(self.report_validity(can_gc))
    }
}

// Form-associated custom elements also need the Validatable trait.
impl Validatable for ElementInternals {
    fn as_element(&self) -> &Element {
        debug_assert!(self.is_target_form_associated());
        self.target_element.upcast::<Element>()
    }

    fn validity_state(&self) -> DomRoot<ValidityState> {
        debug_assert!(self.is_target_form_associated());
        self.validity_state.or_init(|| {
            ValidityState::new(
                &self.target_element.owner_window(),
                self.target_element.upcast(),
                CanGc::note(),
            )
        })
    }

    /// <https://html.spec.whatwg.org/multipage#candidate-for-constraint-validation>
    fn is_instance_validatable(&self) -> bool {
        debug_assert!(self.is_target_form_associated());
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

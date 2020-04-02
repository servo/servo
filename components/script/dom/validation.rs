/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
use crate::dom::bindings::codegen::Bindings::EventBinding::EventBinding::EventMethods;
use crate::dom::bindings::codegen::Bindings::HTMLElementBinding::HTMLElementMethods;
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::DOMString;
use crate::dom::element::Element;
use crate::dom::eventtarget::EventTarget;
use crate::dom::htmldatalistelement::HTMLDataListElement;
use crate::dom::htmlelement::HTMLElement;
use crate::dom::node::Node;
use crate::dom::validitystate::{ValidationFlags, ValidityState};

/// Trait for elements with constraint validation support
pub trait Validatable {
    fn as_element(&self) -> &Element;

    // https://html.spec.whatwg.org/multipage/#dom-cva-validity
    fn validity_state(&self) -> DomRoot<ValidityState>;

    // https://html.spec.whatwg.org/multipage/#candidate-for-constraint-validation
    fn is_instance_validatable(&self) -> bool;

    // Check if element satisfies its constraints, excluding custom errors
    fn perform_validation(&self, _validate_flags: ValidationFlags) -> ValidationFlags {
        ValidationFlags::empty()
    }

    // https://html.spec.whatwg.org/multipage/#concept-fv-valid
    fn validate(&self, validate_flags: ValidationFlags) -> ValidationFlags {
        let mut failed_flags = self.perform_validation(validate_flags);

        // https://html.spec.whatwg.org/multipage/#suffering-from-a-custom-error
        if validate_flags.contains(ValidationFlags::CUSTOM_ERROR) {
            if !self.validity_state().custom_error_message().is_empty() {
                failed_flags.insert(ValidationFlags::CUSTOM_ERROR);
            }
        }

        failed_flags
    }

    // https://html.spec.whatwg.org/multipage/#check-validity-steps
    fn check_validity(&self) -> bool {
        if self.is_instance_validatable() && !self.validate(ValidationFlags::all()).is_empty() {
            self.as_element()
                .upcast::<EventTarget>()
                .fire_cancelable_event(atom!("invalid"));
            false
        } else {
            true
        }
    }

    // https://html.spec.whatwg.org/multipage/#report-validity-steps
    fn report_validity(&self) -> bool {
        // Step 1.
        if !self.is_instance_validatable() {
            return true;
        }

        let flags = self.validate(ValidationFlags::all());
        if flags.is_empty() {
            return true;
        }

        // Step 1.1.
        let event = self
            .as_element()
            .upcast::<EventTarget>()
            .fire_cancelable_event(atom!("invalid"));

        // Step 1.2.
        if !event.DefaultPrevented() {
            println!(
                "Validation error: {}",
                validation_message_for_flags(&self.validity_state(), flags)
            );
            if let Some(html_elem) = self.as_element().downcast::<HTMLElement>() {
                html_elem.Focus();
            }
        }

        // Step 1.3.
        false
    }

    // https://html.spec.whatwg.org/multipage/#dom-cva-validationmessage
    fn validation_message(&self) -> DOMString {
        if self.is_instance_validatable() {
            let flags = self.validate(ValidationFlags::all());
            validation_message_for_flags(&self.validity_state(), flags)
        } else {
            DOMString::new()
        }
    }
}

// https://html.spec.whatwg.org/multipage/#the-datalist-element%3Abarred-from-constraint-validation
pub fn is_barred_by_datalist_ancestor(elem: &Node) -> bool {
    elem.upcast::<Node>()
        .ancestors()
        .any(|node| node.is::<HTMLDataListElement>())
}

// Get message for given validation flags or custom error message
fn validation_message_for_flags(state: &ValidityState, failed_flags: ValidationFlags) -> DOMString {
    if failed_flags.contains(ValidationFlags::CUSTOM_ERROR) {
        state.custom_error_message().clone()
    } else {
        DOMString::from(failed_flags.to_string())
    }
}

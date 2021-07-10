/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::dom::bindings::cell::{DomRefCell, Ref};
use crate::dom::bindings::codegen::Bindings::ValidityStateBinding::ValidityStateMethods;
use crate::dom::bindings::reflector::{reflect_dom_object, Reflector};
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::bindings::str::DOMString;
use crate::dom::element::Element;
use crate::dom::window::Window;
use dom_struct::dom_struct;
use itertools::Itertools;
use std::fmt;

// https://html.spec.whatwg.org/multipage/#validity-states
bitflags! {
    pub struct ValidationFlags: u32 {
        const VALUE_MISSING    = 0b0000000001;
        const TYPE_MISMATCH    = 0b0000000010;
        const PATTERN_MISMATCH = 0b0000000100;
        const TOO_LONG         = 0b0000001000;
        const TOO_SHORT        = 0b0000010000;
        const RANGE_UNDERFLOW  = 0b0000100000;
        const RANGE_OVERFLOW   = 0b0001000000;
        const STEP_MISMATCH    = 0b0010000000;
        const BAD_INPUT        = 0b0100000000;
        const CUSTOM_ERROR     = 0b1000000000;
    }
}

impl fmt::Display for ValidationFlags {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        let flag_to_message = [
            (ValidationFlags::VALUE_MISSING, "Value missing"),
            (ValidationFlags::TYPE_MISMATCH, "Type mismatch"),
            (ValidationFlags::PATTERN_MISMATCH, "Pattern mismatch"),
            (ValidationFlags::TOO_LONG, "Too long"),
            (ValidationFlags::TOO_SHORT, "Too short"),
            (ValidationFlags::RANGE_UNDERFLOW, "Range underflow"),
            (ValidationFlags::RANGE_OVERFLOW, "Range overflow"),
            (ValidationFlags::STEP_MISMATCH, "Step mismatch"),
            (ValidationFlags::BAD_INPUT, "Bad input"),
            (ValidationFlags::CUSTOM_ERROR, "Custom error"),
        ];

        flag_to_message
            .iter()
            .filter_map(|&(flag, flag_str)| {
                if self.contains(flag) {
                    Some(flag_str)
                } else {
                    None
                }
            })
            .format(", ")
            .fmt(formatter)
    }
}

// https://html.spec.whatwg.org/multipage/#validitystate
#[dom_struct]
pub struct ValidityState {
    reflector_: Reflector,
    element: Dom<Element>,
    custom_error_message: DomRefCell<DOMString>,
}

impl ValidityState {
    fn new_inherited(element: &Element) -> ValidityState {
        ValidityState {
            reflector_: Reflector::new(),
            element: Dom::from_ref(element),
            custom_error_message: DomRefCell::new(DOMString::new()),
        }
    }

    pub fn new(window: &Window, element: &Element) -> DomRoot<ValidityState> {
        reflect_dom_object(Box::new(ValidityState::new_inherited(element)), window)
    }

    // https://html.spec.whatwg.org/multipage/#custom-validity-error-message
    pub fn custom_error_message(&self) -> Ref<DOMString> {
        self.custom_error_message.borrow()
    }

    // https://html.spec.whatwg.org/multipage/#custom-validity-error-message
    pub fn set_custom_error_message(&self, error: DOMString) {
        *self.custom_error_message.borrow_mut() = error;
    }
}

impl ValidityStateMethods for ValidityState {
    // https://html.spec.whatwg.org/multipage/#dom-validitystate-valuemissing
    fn ValueMissing(&self) -> bool {
        self.element.as_maybe_validatable().map_or(false, |e| {
            !e.validate(ValidationFlags::VALUE_MISSING).is_empty()
        })
    }

    // https://html.spec.whatwg.org/multipage/#dom-validitystate-typemismatch
    fn TypeMismatch(&self) -> bool {
        self.element.as_maybe_validatable().map_or(false, |e| {
            !e.validate(ValidationFlags::TYPE_MISMATCH).is_empty()
        })
    }

    // https://html.spec.whatwg.org/multipage/#dom-validitystate-patternmismatch
    fn PatternMismatch(&self) -> bool {
        self.element.as_maybe_validatable().map_or(false, |e| {
            !e.validate(ValidationFlags::PATTERN_MISMATCH).is_empty()
        })
    }

    // https://html.spec.whatwg.org/multipage/#dom-validitystate-toolong
    fn TooLong(&self) -> bool {
        self.element
            .as_maybe_validatable()
            .map_or(false, |e| !e.validate(ValidationFlags::TOO_LONG).is_empty())
    }

    // https://html.spec.whatwg.org/multipage/#dom-validitystate-tooshort
    fn TooShort(&self) -> bool {
        self.element.as_maybe_validatable().map_or(false, |e| {
            !e.validate(ValidationFlags::TOO_SHORT).is_empty()
        })
    }

    // https://html.spec.whatwg.org/multipage/#dom-validitystate-rangeunderflow
    fn RangeUnderflow(&self) -> bool {
        self.element.as_maybe_validatable().map_or(false, |e| {
            !e.validate(ValidationFlags::RANGE_UNDERFLOW).is_empty()
        })
    }

    // https://html.spec.whatwg.org/multipage/#dom-validitystate-rangeoverflow
    fn RangeOverflow(&self) -> bool {
        self.element.as_maybe_validatable().map_or(false, |e| {
            !e.validate(ValidationFlags::RANGE_OVERFLOW).is_empty()
        })
    }

    // https://html.spec.whatwg.org/multipage/#dom-validitystate-stepmismatch
    fn StepMismatch(&self) -> bool {
        self.element.as_maybe_validatable().map_or(false, |e| {
            !e.validate(ValidationFlags::STEP_MISMATCH).is_empty()
        })
    }

    // https://html.spec.whatwg.org/multipage/#dom-validitystate-badinput
    fn BadInput(&self) -> bool {
        self.element.as_maybe_validatable().map_or(false, |e| {
            !e.validate(ValidationFlags::BAD_INPUT).is_empty()
        })
    }

    // https://html.spec.whatwg.org/multipage/#dom-validitystate-customerror
    fn CustomError(&self) -> bool {
        self.element.as_maybe_validatable().map_or(false, |e| {
            !e.validate(ValidationFlags::CUSTOM_ERROR).is_empty()
        })
    }

    // https://html.spec.whatwg.org/multipage/#dom-validitystate-valid
    fn Valid(&self) -> bool {
        self.element
            .as_maybe_validatable()
            .map_or(true, |e| e.validate(ValidationFlags::all()).is_empty())
    }
}

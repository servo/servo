/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::ValidityStateBinding;
use dom::bindings::codegen::Bindings::ValidityStateBinding::ValidityStateMethods;
use dom::bindings::js::{JS, Root};
use dom::bindings::reflector::{Reflector, reflect_dom_object};
use dom::element::Element;
use dom::window::Window;

// https://html.spec.whatwg.org/multipage/#validity-states
#[derive(JSTraceable, HeapSizeOf)]
#[allow(dead_code)]
pub enum ValidityStatus {
    ValueMissing,
    TypeMismatch,
    PatternMismatch,
    TooLong,
    TooShort,
    RangeUnderflow,
    RangeOverflow,
    StepMismatch,
    BadInput,
    CustomError,
    Valid
}

bitflags!{
    pub flags ValidationFlags: u32 {
        const VALUE_MISSING    = 0b0000000001,
        const TYPE_MISMATCH    = 0b0000000010,
        const PATTERN_MISMATCH = 0b0000000100,
        const TOO_LONG         = 0b0000001000,
        const TOO_SHORT        = 0b0000010000,
        const RANGE_UNDERFLOW  = 0b0000100000,
        const RANGE_OVERFLOW   = 0b0001000000,
        const STEP_MISMATCH    = 0b0010000000,
        const BAD_INPUT        = 0b0100000000,
        const CUSTOM_ERROR     = 0b1000000000,
    }
}

// https://html.spec.whatwg.org/multipage/#validitystate
#[dom_struct]
pub struct ValidityState {
    reflector_: Reflector,
    element: JS<Element>,
    state: ValidityStatus
}


impl ValidityState {
    fn new_inherited(element: &Element) -> ValidityState {
        ValidityState {
            reflector_: Reflector::new(),
            element: JS::from_ref(element),
            state: ValidityStatus::Valid
        }
    }

    pub fn new(window: &Window, element: &Element) -> Root<ValidityState> {
        reflect_dom_object(box ValidityState::new_inherited(element),
                           window,
                           ValidityStateBinding::Wrap)
    }
}

impl ValidityStateMethods for ValidityState {
    // https://html.spec.whatwg.org/multipage/#dom-validitystate-valuemissing
    fn ValueMissing(&self) -> bool {
        false
    }

    // https://html.spec.whatwg.org/multipage/#dom-validitystate-typemismatch
    fn TypeMismatch(&self) -> bool {
        false
    }

    // https://html.spec.whatwg.org/multipage/#dom-validitystate-patternmismatch
    fn PatternMismatch(&self) -> bool {
        false
    }

    // https://html.spec.whatwg.org/multipage/#dom-validitystate-toolong
    fn TooLong(&self) -> bool {
        false
    }

    // https://html.spec.whatwg.org/multipage/#dom-validitystate-tooshort
    fn TooShort(&self) -> bool {
        false
    }

    // https://html.spec.whatwg.org/multipage/#dom-validitystate-rangeunderflow
    fn RangeUnderflow(&self) -> bool {
        false
    }

    // https://html.spec.whatwg.org/multipage/#dom-validitystate-rangeoverflow
    fn RangeOverflow(&self) -> bool {
        false
    }

    // https://html.spec.whatwg.org/multipage/#dom-validitystate-stepmismatch
    fn StepMismatch(&self) -> bool {
        false
    }

    // https://html.spec.whatwg.org/multipage/#dom-validitystate-badinput
    fn BadInput(&self) -> bool {
        false
    }

    // https://html.spec.whatwg.org/multipage/#dom-validitystate-customerror
    fn CustomError(&self) -> bool {
        false
    }

    // https://html.spec.whatwg.org/multipage/#dom-validitystate-valid
    fn Valid(&self) -> bool {
        false
    }
}

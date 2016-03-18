/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::ValidityStateBinding;
use dom::bindings::codegen::Bindings::ValidityStateBinding::ValidityStateMethods;
use dom::bindings::global::GlobalRef;
use dom::bindings::js::Root;
use dom::bindings::reflector::{Reflector, reflect_dom_object};
use dom::window::Window;

// https://html.spec.whatwg.org/multipage/#validitystate
#[dom_struct]
pub struct ValidityState {
    reflector_: Reflector,
    state: u8,
}

impl ValidityState {
    fn new_inherited() -> ValidityState {
        ValidityState {
            reflector_: Reflector::new(),
            state: 0,
        }
    }

    pub fn new(window: &Window) -> Root<ValidityState> {
        reflect_dom_object(box ValidityState::new_inherited(),
                           GlobalRef::Window(window),
                           ValidityStateBinding::Wrap)
    }
}

impl ValidityStateMethods for ValidityState {

    fn ValueMissing(&self) -> bool {
        false
    }

    fn TypeMismatch(&self) -> bool {
        false
    }

    fn PatternMismatch(&self) -> bool {
        false
    }

    fn TooLong(&self) -> bool {
        false
    }

    fn TooShort(&self) -> bool {
        false
    }

    fn RangeUnderflow(&self) -> bool {
        false
    }

    fn RangeOverflow(&self) -> bool {
        false
    }

    fn StepMismatch(&self) -> bool {
        false
    }

    fn BadInput(&self) -> bool {
        false
    }

    fn CustomError(&self) -> bool {
        false
    }

    fn Valid(&self) -> bool {
        false
    }
}

/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::ValidityStateBinding;
use dom::bindings::jsmanaged::JSManaged;
use dom::bindings::utils::{Reflectable, Reflector, reflect_dom_object2};
use dom::window::Window;

pub struct ValidityState {
    reflector_: Reflector,
    window: JSManaged<Window>,
    state: u8
}

impl ValidityState {
    pub fn new_inherited(window: JSManaged<Window>) -> ValidityState {
        ValidityState {
            reflector_: Reflector::new(),
            window: window,
            state: 0
        }
    }

    pub fn new(window: JSManaged<Window>) -> JSManaged<ValidityState> {
        reflect_dom_object2(~ValidityState::new_inherited(window), window.value(),
                            ValidityStateBinding::Wrap)
    }
}

impl ValidityState {
    pub fn ValueMissing(&self) -> bool {
        false
    }

    pub fn TypeMismatch(&self) -> bool {
        false
    }

    pub fn PatternMismatch(&self) -> bool {
        false
    }

    pub fn TooLong(&self) -> bool {
        false
    }

    pub fn RangeUnderflow(&self) -> bool {
        false
    }

    pub fn RangeOverflow(&self) -> bool {
        false
    }

    pub fn StepMismatch(&self) -> bool {
        false
    }

    pub fn CustomError(&self) -> bool {
        false
    }

    pub fn Valid(&self) -> bool {
        true
    }
}

impl Reflectable for ValidityState {
    fn reflector<'a>(&'a self) -> &'a Reflector {
        &self.reflector_
    }

    fn mut_reflector<'a>(&'a mut self) -> &'a mut Reflector {
        &mut self.reflector_
    }
}

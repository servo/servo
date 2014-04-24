/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::BindingDeclarations::ValidityStateBinding;
use dom::bindings::js::{JS, JSRef, Temporary};
use dom::bindings::utils::{Reflectable, Reflector, reflect_dom_object};
use dom::window::Window;

#[deriving(Encodable)]
pub struct ValidityState {
    pub reflector_: Reflector,
    pub window: JS<Window>,
    pub state: u8,
}

impl ValidityState {
    pub fn new_inherited(window: &JSRef<Window>) -> ValidityState {
        ValidityState {
            reflector_: Reflector::new(),
            window: window.unrooted(),
            state: 0,
        }
    }

    pub fn new(window: &JSRef<Window>) -> Temporary<ValidityState> {
        reflect_dom_object(~ValidityState::new_inherited(window),
                           window,
                           ValidityStateBinding::Wrap)
    }
}

pub trait ValidityStateMethods {
    fn ValueMissing(&self) -> bool;
    fn TypeMismatch(&self) -> bool;
    fn PatternMismatch(&self) -> bool;
    fn TooLong(&self) -> bool;
    fn RangeUnderflow(&self) -> bool;
    fn RangeOverflow(&self) -> bool;
    fn StepMismatch(&self) -> bool;
    fn CustomError(&self) -> bool;
    fn Valid(&self) -> bool;
}

impl<'a> ValidityStateMethods for JSRef<'a, ValidityState> {
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

    fn RangeUnderflow(&self) -> bool {
        false
    }

    fn RangeOverflow(&self) -> bool {
        false
    }

    fn StepMismatch(&self) -> bool {
        false
    }

    fn CustomError(&self) -> bool {
        false
    }

    fn Valid(&self) -> bool {
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

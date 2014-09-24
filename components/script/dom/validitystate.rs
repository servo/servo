/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::ValidityStateBinding;
use dom::bindings::global::Window;
use dom::bindings::js::{JSRef, Temporary};
use dom::bindings::utils::{Reflectable, Reflector, reflect_dom_object};
use dom::window::Window;

#[jstraceable]
#[must_root]
pub struct ValidityState {
    reflector_: Reflector,
    state: u8,
}

impl ValidityState {
    pub fn new_inherited() -> ValidityState {
        ValidityState {
            reflector_: Reflector::new(),
            state: 0,
        }
    }

    pub fn new(window: JSRef<Window>) -> Temporary<ValidityState> {
        reflect_dom_object(box ValidityState::new_inherited(),
                           &Window(window),
                           ValidityStateBinding::Wrap)
    }
}

impl Reflectable for ValidityState {
    fn reflector<'a>(&'a self) -> &'a Reflector {
        &self.reflector_
    }
}

/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::DOMPointReadOnlyBinding;
use dom::bindings::codegen::Bindings::DOMPointReadOnlyBinding::DOMPointReadOnlyMethods;
use dom::bindings::global::GlobalRef;
use dom::bindings::js::Root;
use dom::bindings::utils::{Reflector, reflect_dom_object};
use dom::window::Window;

#[dom_struct]
pub struct DOMPointReadOnly {
    reflector_: Reflector,
    x: f64,
    y: f64,
    z: f64,
    w: f64,
}

impl DOMPointReadOnly {
    fn new_inherited(x: f64, y: f64, z: f64, w: f64) -> DOMPointReadOnly {
        DOMPointReadOnly {
            x: x,
            y: y,
            z: z,
            w: w,
            reflector_: Reflector::new(),
        }
    }

    pub fn new(window: &Window,
               x: f64, y: f64, z: f64, w: f64) -> Root<DOMPointReadOnly> {
        reflect_dom_object(box DOMPointReadOnly::new_inherited(x, y, z, w),
                           GlobalRef::Window(window), DOMPointReadOnlyBinding::Wrap)
    }
}

impl<'a> DOMPointReadOnlyMethods for &'a DOMPointReadOnly {
    fn X(self) -> f64 {
        self.x
    }

    fn Y(self) -> f64 {
        self.y
    }

    fn Z(self) -> f64 {
        self.z
    }

    fn W(self) -> f64 {
        self.w
    }
}

#[dom_struct]
pub struct DOMPoint {
    reflector_: Reflector,
    x: f64,
    y: f64,
    z: f64,
    w: f64,
}

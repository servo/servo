/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::DOMPointBinding::{DOMPointInit, DOMPointMethods, Wrap};
use dom::bindings::error::Fallible;
use dom::bindings::global::GlobalRef;
use dom::bindings::js::Root;
use dom::bindings::utils::{Reflector, reflect_dom_object};
use std::cell::Cell;

#[dom_struct]
pub struct DOMPoint {
    reflector_: Reflector,
    x: Cell<f64>,
    y: Cell<f64>,
    z: Cell<f64>,
    w: Cell<f64>,
}

impl DOMPoint {
    fn new_inherited(x: f64, y: f64, z: f64, w: f64) -> DOMPoint {
        DOMPoint {
            x: Cell::new(x),
            y: Cell::new(y),
            z: Cell::new(z),
            w: Cell::new(w),
            reflector_: Reflector::new(),
        }
    }

    pub fn new(global: GlobalRef, x: f64, y: f64, z: f64, w: f64) -> Root<DOMPoint> {
        reflect_dom_object(box DOMPoint::new_inherited(x, y, z, w), global, Wrap)
    }

    pub fn Constructor(global: GlobalRef, init: &DOMPointInit) -> Fallible<Root<DOMPoint>> {
        Ok(DOMPoint::new(global, init.x, init.y, init.z, init.w))
    }

    pub fn Constructor_(global: GlobalRef,
                       x: f64, y: f64, z: f64, w: f64) -> Fallible<Root<DOMPoint>> {
        Ok(DOMPoint::new(global, x, y, z, w))
    }
}

impl<'a> DOMPointMethods for &'a DOMPoint {
    fn X(self) -> f64 {
        self.x.get()
    }
    fn SetX(self, value: f64) -> () {
        self.x.set(value);
    }

    fn Y(self) -> f64 {
        self.y.get()
    }
    fn SetY(self, value: f64) -> () {
        self.y.set(value);
    }

    fn Z(self) -> f64 {
        self.z.get()
    }
    fn SetZ(self, value: f64) -> () {
        self.z.set(value);
    }

    fn W(self) -> f64 {
        self.w.get()
    }
    fn SetW(self, value: f64) -> () {
        self.w.set(value);
    }
}


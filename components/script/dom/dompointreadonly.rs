/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::DOMPointReadOnlyBinding::{DOMPointReadOnlyMethods, Wrap};
use dom::bindings::error::Fallible;
use dom::bindings::reflector::{Reflector, reflect_dom_object};
use dom::bindings::root::DomRoot;
use dom::globalscope::GlobalScope;
use dom_struct::dom_struct;
use std::cell::Cell;
use typeholder::TypeHolderTrait;

// http://dev.w3.org/fxtf/geometry/Overview.html#dompointreadonly
#[dom_struct]
pub struct DOMPointReadOnly<TH: TypeHolderTrait> {
    reflector_: Reflector<TH>,
    x: Cell<f64>,
    y: Cell<f64>,
    z: Cell<f64>,
    w: Cell<f64>,
}

impl<TH: TypeHolderTrait> DOMPointReadOnly<TH> {
    pub fn new_inherited(x: f64, y: f64, z: f64, w: f64) -> DOMPointReadOnly<TH> {
        DOMPointReadOnly {
            x: Cell::new(x),
            y: Cell::new(y),
            z: Cell::new(z),
            w: Cell::new(w),
            reflector_: Reflector::new(),
        }
    }

    pub fn new(global: &GlobalScope<TH>, x: f64, y: f64, z: f64, w: f64) -> DomRoot<DOMPointReadOnly<TH>> {
        reflect_dom_object(Box::new(DOMPointReadOnly::new_inherited(x, y, z, w)),
                           global,
                           Wrap)
    }

    pub fn Constructor(global: &GlobalScope<TH>,
                       x: f64,
                       y: f64,
                       z: f64,
                       w: f64)
                       -> Fallible<DomRoot<DOMPointReadOnly<TH>>> {
        Ok(DOMPointReadOnly::new(global, x, y, z, w))
    }
}

impl<TH: TypeHolderTrait> DOMPointReadOnlyMethods for DOMPointReadOnly<TH> {
    // https://dev.w3.org/fxtf/geometry/Overview.html#dom-dompointreadonly-x
    fn X(&self) -> f64 {
        self.x.get()
    }

    // https://dev.w3.org/fxtf/geometry/Overview.html#dom-dompointreadonly-y
    fn Y(&self) -> f64 {
        self.y.get()
    }

    // https://dev.w3.org/fxtf/geometry/Overview.html#dom-dompointreadonly-z
    fn Z(&self) -> f64 {
        self.z.get()
    }

    // https://dev.w3.org/fxtf/geometry/Overview.html#dom-dompointreadonly-w
    fn W(&self) -> f64 {
        self.w.get()
    }
}

pub trait DOMPointWriteMethods {
    fn SetX(&self, value: f64);
    fn SetY(&self, value: f64);
    fn SetZ(&self, value: f64);
    fn SetW(&self, value: f64);
}

impl<TH: TypeHolderTrait> DOMPointWriteMethods for DOMPointReadOnly<TH> {
    fn SetX(&self, value: f64) {
        self.x.set(value);
    }

    fn SetY(&self, value: f64) {
        self.y.set(value);
    }

    fn SetZ(&self, value: f64) {
        self.z.set(value);
    }

    fn SetW(&self, value: f64) {
        self.w.set(value);
    }
}

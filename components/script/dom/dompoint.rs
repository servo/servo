/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use js::rust::HandleObject;

use crate::dom::bindings::codegen::Bindings::DOMPointBinding::{DOMPointInit, DOMPointMethods};
use crate::dom::bindings::codegen::Bindings::DOMPointReadOnlyBinding::DOMPointReadOnlyMethods;
use crate::dom::bindings::error::Fallible;
use crate::dom::bindings::reflector::reflect_dom_object_with_proto;
use crate::dom::bindings::root::DomRoot;
use crate::dom::dompointreadonly::{DOMPointReadOnly, DOMPointWriteMethods};
use crate::dom::globalscope::GlobalScope;

// http://dev.w3.org/fxtf/geometry/Overview.html#dompoint
#[dom_struct]
pub struct DOMPoint {
    point: DOMPointReadOnly,
}

#[allow(non_snake_case)]
impl DOMPoint {
    fn new_inherited(x: f64, y: f64, z: f64, w: f64) -> DOMPoint {
        DOMPoint {
            point: DOMPointReadOnly::new_inherited(x, y, z, w),
        }
    }

    pub fn new(global: &GlobalScope, x: f64, y: f64, z: f64, w: f64) -> DomRoot<DOMPoint> {
        Self::new_with_proto(global, None, x, y, z, w)
    }

    fn new_with_proto(
        global: &GlobalScope,
        proto: Option<HandleObject>,
        x: f64,
        y: f64,
        z: f64,
        w: f64,
    ) -> DomRoot<DOMPoint> {
        reflect_dom_object_with_proto(Box::new(DOMPoint::new_inherited(x, y, z, w)), global, proto)
    }

    pub fn Constructor(
        global: &GlobalScope,
        proto: Option<HandleObject>,
        x: f64,
        y: f64,
        z: f64,
        w: f64,
    ) -> Fallible<DomRoot<DOMPoint>> {
        Ok(DOMPoint::new_with_proto(global, proto, x, y, z, w))
    }

    // https://drafts.fxtf.org/geometry/#dom-dompoint-frompoint
    pub fn FromPoint(global: &GlobalScope, init: &DOMPointInit) -> DomRoot<Self> {
        Self::new_from_init(global, init)
    }

    pub fn new_from_init(global: &GlobalScope, p: &DOMPointInit) -> DomRoot<DOMPoint> {
        DOMPoint::new(global, p.x, p.y, p.z, p.w)
    }
}

impl DOMPointMethods for DOMPoint {
    // https://dev.w3.org/fxtf/geometry/Overview.html#dom-dompointreadonly-x
    fn X(&self) -> f64 {
        self.point.X()
    }

    // https://dev.w3.org/fxtf/geometry/Overview.html#dom-dompointreadonly-x
    fn SetX(&self, value: f64) {
        self.point.SetX(value);
    }

    // https://dev.w3.org/fxtf/geometry/Overview.html#dom-dompointreadonly-y
    fn Y(&self) -> f64 {
        self.point.Y()
    }

    // https://dev.w3.org/fxtf/geometry/Overview.html#dom-dompointreadonly-y
    fn SetY(&self, value: f64) {
        self.point.SetY(value);
    }

    // https://dev.w3.org/fxtf/geometry/Overview.html#dom-dompointreadonly-z
    fn Z(&self) -> f64 {
        self.point.Z()
    }

    // https://dev.w3.org/fxtf/geometry/Overview.html#dom-dompointreadonly-z
    fn SetZ(&self, value: f64) {
        self.point.SetZ(value);
    }

    // https://dev.w3.org/fxtf/geometry/Overview.html#dom-dompointreadonly-w
    fn W(&self) -> f64 {
        self.point.W()
    }

    // https://dev.w3.org/fxtf/geometry/Overview.html#dom-dompointreadonly-w
    fn SetW(&self, value: f64) {
        self.point.SetW(value);
    }
}

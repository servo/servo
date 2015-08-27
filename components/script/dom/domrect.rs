/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::DOMRectBinding::{DOMRectMethods, Wrap};
use dom::bindings::codegen::Bindings::DOMRectReadOnlyBinding::DOMRectReadOnlyMethods;
use dom::bindings::error::Fallible;
use dom::bindings::global::GlobalRef;
use dom::bindings::js::Root;
use dom::bindings::utils::reflect_dom_object;
use dom::domrectreadonly::{DOMRectReadOnly, DOMRectWriteMethods};

#[dom_struct]
pub struct DOMRect {
    rect: DOMRectReadOnly,
}

impl DOMRect {
    fn new_inherited(x: f64, y: f64, width: f64, height: f64) -> DOMRect {
        DOMRect {
            rect: DOMRectReadOnly::new_inherited(x, y, width, height),
        }
    }

    pub fn new(global: GlobalRef, x: f64, y: f64, width: f64, height: f64) -> Root<DOMRect> {
        reflect_dom_object(box DOMRect::new_inherited(x, y, width, height), global, Wrap)
    }

    pub fn Constructor(global: GlobalRef,
                        x: f64, y: f64, width: f64, height: f64) -> Fallible<Root<DOMRect>> {
        Ok(DOMRect::new(global, x, y, width, height))
    }
}

impl DOMRectMethods for DOMRect {
    // https://dev.w3.org/fxtf/geometry/Overview.html#dom-domrect-x
    fn X(&self) -> f64 {
        self.rect.X()
    }

    // https://dev.w3.org/fxtf/geometry/Overview.html#dom-domrect-x
    fn SetX(&self, value: f64) {
        self.rect.SetX(value);
    }

    // https://dev.w3.org/fxtf/geometry/Overview.html#dom-domrect-y
    fn Y(&self) -> f64 {
        self.rect.Y()
    }

    // https://dev.w3.org/fxtf/geometry/Overview.html#dom-domrect-y
    fn SetY(&self, value: f64) {
        self.rect.SetY(value);
    }

    // https://dev.w3.org/fxtf/geometry/Overview.html#dom-domrect-width
    fn Width(&self) -> f64 {
        self.rect.Width()
    }

    // https://dev.w3.org/fxtf/geometry/Overview.html#dom-domrect-width
    fn SetWidth(&self, value: f64) {
        self.rect.SetWidth(value);
    }

    // https://dev.w3.org/fxtf/geometry/Overview.html#dom-domrect-height
    fn Height(&self) -> f64 {
        self.rect.Height()
    }

    // https://dev.w3.org/fxtf/geometry/Overview.html#dom-domrect-height
    fn SetHeight(&self, value: f64) {
        self.rect.SetHeight(value);
    }
}

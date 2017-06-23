/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::PaintSizeBinding;
use dom::bindings::codegen::Bindings::PaintSizeBinding::PaintSizeMethods;
use dom::bindings::js::Root;
use dom::bindings::num::Finite;
use dom::bindings::reflector::Reflector;
use dom::bindings::reflector::reflect_dom_object;
use dom::paintworkletglobalscope::PaintWorkletGlobalScope;
use dom_struct::dom_struct;
use euclid::TypedSize2D;
use style_traits::CSSPixel;

#[dom_struct]
pub struct PaintSize {
    reflector: Reflector,
    width: Finite<f64>,
    height: Finite<f64>,
}

impl PaintSize {
    fn new_inherited(size: TypedSize2D<f32, CSSPixel>) -> PaintSize {
        PaintSize {
            reflector: Reflector::new(),
            width: Finite::wrap(size.width as f64),
            height: Finite::wrap(size.height as f64),
        }
    }

    pub fn new(global: &PaintWorkletGlobalScope, size: TypedSize2D<f32, CSSPixel>) -> Root<PaintSize> {
        reflect_dom_object(box PaintSize::new_inherited(size), global, PaintSizeBinding::Wrap)
    }
}

impl PaintSizeMethods for PaintSize {
    /// https://drafts.css-houdini.org/css-paint-api/#paintsize
    fn Width(&self) -> Finite<f64> {
        self.width
    }

    /// https://drafts.css-houdini.org/css-paint-api/#paintsize
    fn Height(&self) -> Finite<f64> {
        self.height
    }
}

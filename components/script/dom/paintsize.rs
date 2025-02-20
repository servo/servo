/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use euclid::Size2D;
use style_traits::CSSPixel;

use crate::dom::bindings::codegen::Bindings::PaintSizeBinding::PaintSizeMethods;
use crate::dom::bindings::num::Finite;
use crate::dom::bindings::reflector::{reflect_dom_object, Reflector};
use crate::dom::bindings::root::DomRoot;
use crate::dom::paintworkletglobalscope::PaintWorkletGlobalScope;
use crate::script_runtime::CanGc;

#[dom_struct]
pub(crate) struct PaintSize {
    reflector: Reflector,
    width: Finite<f64>,
    height: Finite<f64>,
}

impl PaintSize {
    fn new_inherited(size: Size2D<f32, CSSPixel>) -> PaintSize {
        PaintSize {
            reflector: Reflector::new(),
            width: Finite::wrap(size.width as f64),
            height: Finite::wrap(size.height as f64),
        }
    }

    pub(crate) fn new(
        global: &PaintWorkletGlobalScope,
        size: Size2D<f32, CSSPixel>,
        can_gc: CanGc,
    ) -> DomRoot<PaintSize> {
        reflect_dom_object(Box::new(PaintSize::new_inherited(size)), global, can_gc)
    }
}

impl PaintSizeMethods<crate::DomTypeHolder> for PaintSize {
    /// <https://drafts.css-houdini.org/css-paint-api/#paintsize>
    fn Width(&self) -> Finite<f64> {
        self.width
    }

    /// <https://drafts.css-houdini.org/css-paint-api/#paintsize>
    fn Height(&self) -> Finite<f64> {
        self.height
    }
}

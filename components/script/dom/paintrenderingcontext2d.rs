/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::PaintRenderingContext2DBinding;
use dom::bindings::js::Root;
use dom::bindings::reflector::Reflector;
use dom::bindings::reflector::reflect_dom_object;
use dom::paintworkletglobalscope::PaintWorkletGlobalScope;
use dom_struct::dom_struct;

#[dom_struct]
pub struct PaintRenderingContext2D {
    reflector: Reflector,
}

impl PaintRenderingContext2D {
    fn new_inherited() -> PaintRenderingContext2D {
        PaintRenderingContext2D {
            reflector: Reflector::new(),
        }
    }

    pub fn new(global: &PaintWorkletGlobalScope) -> Root<PaintRenderingContext2D> {
        reflect_dom_object(box PaintRenderingContext2D::new_inherited(),
                           global,
                           PaintRenderingContext2DBinding::Wrap)
    }
}

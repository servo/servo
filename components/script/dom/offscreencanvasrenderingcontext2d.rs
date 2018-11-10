/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use dom::bindings::codegen::Bindings::OffscreenCanvasRenderingContext2DBinding;
use dom::bindings::codegen::Bindings::OffscreenCanvasBinding::{Wrap as OffscreenCanvasRenderingContext2DWrap};
use dom::offscreencanvas::OffscreenCanvas;
use dom::bindings::root::DomRoot;
use dom::globalscope::GlobalScope;
use dom::bindings::reflector::{DomObject, Reflector, reflect_dom_object};

#[dom_struct]
pub struct OffscreenCanvasRenderingContext2D{
canvas: OffscreenCanvas,
}

impl OffscreenCanvasRenderingContext2D {
pub fn new_inherited(canvas: OffscreenCanvas) -> OffscreenCanvasRenderingContext2D {
OffscreenCanvasRenderingContext2D {
canvas: canvas,
}
}

pub fn new(global: &GlobalScope,canvas: OffscreenCanvas) -> DomRoot<OffscreenCanvasRenderingContext2D> {
reflect_dom_object(Box::new(OffscreenCanvasRenderingContext2D::new_inherited()), global, OffscreenCanvasRenderingContext2DWrap)
}
}


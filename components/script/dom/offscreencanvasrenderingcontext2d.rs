/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::OffscreenCanvasRenderingContext2DBinding;
use dom::bindings::codegen::Bindings::OffscreenCanvasRenderingContext2DBinding::OffscreenCanvasRenderingContext2DMethods;
use dom::bindings::reflector::{Reflector, reflect_dom_object};
use dom::bindings::root::{Dom,DomRoot};
use dom::globalscope::GlobalScope;  
use dom::offscreencanvas::{OffscreenCanvas};
use dom_struct::dom_struct;
use euclid::Size2D;

#[dom_struct]
pub struct OffscreenCanvasRenderingContext2D{
    reflector_: Reflector,
    canvas: Option<Dom<OffscreenCanvas>>,
}

impl OffscreenCanvasRenderingContext2D {
    pub fn new_inherited(
        global: &GlobalScope,
        canvas: Option<&OffscreenCanvas>,
        size: Size2D<u64>,
    ) -> OffscreenCanvasRenderingContext2D {
        OffscreenCanvasRenderingContext2D {
            reflector_: Reflector::new(),
            canvas: canvas.map(Dom::from_ref),
        }
    }

    pub fn new(
        global: &GlobalScope,
        canvas: &OffscreenCanvas,
        size: Size2D<u64>,
    ) -> DomRoot<OffscreenCanvasRenderingContext2D> {
        //let window = window_from_node(canvas);
        let boxed = Box::new(OffscreenCanvasRenderingContext2D::new_inherited(
            global,
            Some(canvas),
            size,
        ));
        reflect_dom_object(boxed, global, OffscreenCanvasRenderingContext2DBinding::Wrap)
    }
}

impl OffscreenCanvasRenderingContext2DMethods for OffscreenCanvasRenderingContext2D {

    fn Canvas(&self) -> DomRoot<OffscreenCanvas> {
        DomRoot::from_ref(self.canvas.as_ref().expect("No canvas."))
    }
}

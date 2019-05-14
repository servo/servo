/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::OffscreenCanvasRenderingContext2DBinding;
use crate::dom::bindings::codegen::Bindings::OffscreenCanvasRenderingContext2DBinding::OffscreenCanvasRenderingContext2DMethods;
use crate::dom::bindings::reflector::{reflect_dom_object, Reflector};
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::canvasrenderingcontext2d::CanvasState;
use crate::dom::globalscope::GlobalScope;
use crate::dom::offscreencanvas::OffscreenCanvas;
use dom_struct::dom_struct;
use euclid::Size2D;

#[dom_struct]
pub struct OffscreenCanvasRenderingContext2D {
    reflector_: Reflector,
    canvas: Option<Dom<OffscreenCanvas>>,
    canvas_state: DomRefCell<CanvasState>,
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
            canvas_state: DomRefCell::new(CanvasState::new(global, size)),
        }
    }

    pub fn new(
        global: &GlobalScope,
        canvas: &OffscreenCanvas,
        size: Size2D<u64>,
    ) -> DomRoot<OffscreenCanvasRenderingContext2D> {
        let boxed = Box::new(OffscreenCanvasRenderingContext2D::new_inherited(
            global,
            Some(canvas),
            size,
        ));
        reflect_dom_object(
            boxed,
            global,
            OffscreenCanvasRenderingContext2DBinding::Wrap,
        )
    }
}

impl OffscreenCanvasRenderingContext2DMethods for OffscreenCanvasRenderingContext2D {
    // https://html.spec.whatwg.org/multipage/offscreencontext2d-canvas
    fn Canvas(&self) -> DomRoot<OffscreenCanvas> {
        DomRoot::from_ref(self.canvas.as_ref().expect("No canvas."))
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-fillrect
    fn FillRect(&self, x: f64, y: f64, width: f64, height: f64) {
        self.canvas_state.borrow().FillRect(x, y, width, height);
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-clearrect
    fn ClearRect(&self, x: f64, y: f64, width: f64, height: f64) {
        self.canvas_state.borrow().ClearRect(x, y, width, height);
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-strokerect
    fn StrokeRect(&self, x: f64, y: f64, width: f64, height: f64) {
        self.canvas_state.borrow().StrokeRect(x, y, width, height);
    }
}

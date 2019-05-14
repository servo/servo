/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::CanvasRenderingContext2DBinding::CanvasImageSource;
use crate::dom::bindings::codegen::Bindings::OffscreenCanvasRenderingContext2DBinding;
use crate::dom::bindings::codegen::Bindings::OffscreenCanvasRenderingContext2DBinding::OffscreenCanvasRenderingContext2DMethods;
use crate::dom::bindings::codegen::UnionTypes::StringOrCanvasGradientOrCanvasPattern;
use crate::dom::bindings::error::Fallible;
use crate::dom::bindings::num::Finite;
use crate::dom::bindings::reflector::{reflect_dom_object, Reflector};
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::bindings::str::DOMString;
use crate::dom::canvasgradient::CanvasGradient;
use crate::dom::canvaspattern::CanvasPattern;
use crate::dom::canvasrenderingcontext2d::CanvasState;
use crate::dom::globalscope::GlobalScope;
use crate::dom::offscreencanvas::OffscreenCanvas;
use crate::dom::node::window_from_node;
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
        let window = window_from_node(canvas);
        let image_cache = window.image_cache();
        let base_url = window.get_url();
        OffscreenCanvasRenderingContext2D {
            reflector_: Reflector::new(),
            canvas: canvas.map(Dom::from_ref),
            canvas_state: DomRefCell::new(CanvasState::new(
                global,
                canvas,
                image_cache,
                base_url,
                Size2D::new(size.width as u64, size.height as u64),
            )),
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

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-shadowoffsetx
    fn ShadowOffsetX(&self) -> f64 {
        self.canvas_state.borrow().ShadowOffsetX()
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-shadowoffsetx
    fn SetShadowOffsetX(&self, value: f64) {
        self.canvas_state.borrow().SetShadowOffsetX(value)
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-shadowoffsety
    fn ShadowOffsetY(&self) -> f64 {
        self.canvas_state.borrow().ShadowOffsetY()
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-shadowoffsety
    fn SetShadowOffsetY(&self, value: f64) {
        self.canvas_state.borrow().SetShadowOffsetY(value)
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-shadowblur
    fn ShadowBlur(&self) -> f64 {
        self.canvas_state.borrow().ShadowBlur()
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-shadowblur
    fn SetShadowBlur(&self, value: f64) {
        self.canvas_state.borrow().SetShadowBlur(value)
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-shadowcolor
    fn ShadowColor(&self) -> DOMString {
        self.canvas_state.borrow().ShadowColor()
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-shadowcolor
    fn SetShadowColor(&self, value: DOMString) {
        self.canvas_state.borrow().SetShadowColor(value)
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-strokestyle
    fn StrokeStyle(&self) -> StringOrCanvasGradientOrCanvasPattern {
        self.canvas_state.borrow().StrokeStyle()
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-strokestyle
    fn SetStrokeStyle(&self, value: StringOrCanvasGradientOrCanvasPattern) {
        self.canvas_state.borrow().SetStrokeStyle(value)
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-strokestyle
    fn FillStyle(&self) -> StringOrCanvasGradientOrCanvasPattern {
        self.canvas_state.borrow().FillStyle()
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-strokestyle
    fn SetFillStyle(&self, value: StringOrCanvasGradientOrCanvasPattern) {
        self.canvas_state.borrow().SetFillStyle(value)
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-createlineargradient
    fn CreateLinearGradient(
        &self,
        x0: Finite<f64>,
        y0: Finite<f64>,
        x1: Finite<f64>,
        y1: Finite<f64>,
    ) -> DomRoot<CanvasGradient> {
        self.canvas_state
            .borrow()
            .CreateLinearGradient(x0, y0, x1, y1)
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-createradialgradient
    fn CreateRadialGradient(
        &self,
        x0: Finite<f64>,
        y0: Finite<f64>,
        r0: Finite<f64>,
        x1: Finite<f64>,
        y1: Finite<f64>,
        r1: Finite<f64>,
    ) -> Fallible<DomRoot<CanvasGradient>> {
        self.canvas_state
            .borrow()
            .CreateRadialGradient(x0, y0, r0, x1, y1, r1)
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-createpattern
    fn CreatePattern(
        &self,
        image: CanvasImageSource,
        mut repetition: DOMString,
    ) -> Fallible<DomRoot<CanvasPattern>> {
        self.canvas_state.borrow().CreatePattern(image, repetition)
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-save
    fn Save(&self) {
        self.canvas_state.borrow().Save()
    }

    #[allow(unrooted_must_root)]
    // https://html.spec.whatwg.org/multipage/#dom-context-2d-restore
    fn Restore(&self) {
        self.canvas_state.borrow().Restore()
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-globalalpha
    fn GlobalAlpha(&self) -> f64 {
        self.canvas_state.borrow().GlobalAlpha()
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-globalalpha
    fn SetGlobalAlpha(&self, alpha: f64) {
        self.canvas_state.borrow().SetGlobalAlpha(alpha)
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-globalcompositeoperation
    fn GlobalCompositeOperation(&self) -> DOMString {
        self.canvas_state.borrow().GlobalCompositeOperation()
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-globalcompositeoperation
    fn SetGlobalCompositeOperation(&self, op_str: DOMString) {
        self.canvas_state.borrow().SetGlobalCompositeOperation(op_str)
    }

        // https://html.spec.whatwg.org/multipage/#dom-context-2d-imagesmoothingenabled
    fn ImageSmoothingEnabled(&self) -> bool {
        self.canvas_state
            .borrow()
            .ImageSmoothingEnabled()
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-imagesmoothingenabled
    fn SetImageSmoothingEnabled(&self, value: bool) {
        self.canvas_state
            .borrow()
            .SetImageSmoothingEnabled(value)
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-filltext
    fn FillText(&self, text: DOMString, x: f64, y: f64, max_width: Option<f64>) {
        self.canvas_state.borrow().FillText(text,x,y,max_width)
        self.mark_as_dirty();
    }
}

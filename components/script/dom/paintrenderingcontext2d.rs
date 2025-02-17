/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::Cell;

use canvas_traits::canvas::CanvasImageData;
use dom_struct::dom_struct;
use euclid::{Scale, Size2D};
use ipc_channel::ipc::IpcSender;
use servo_url::ServoUrl;
use style_traits::CSSPixel;
use webrender_api::units::DevicePixel;

use crate::dom::bindings::codegen::Bindings::CanvasRenderingContext2DBinding::{
    CanvasFillRule, CanvasImageSource, CanvasLineCap, CanvasLineJoin,
    CanvasRenderingContext2DMethods,
};
use crate::dom::bindings::codegen::Bindings::PaintRenderingContext2DBinding::PaintRenderingContext2DMethods;
use crate::dom::bindings::codegen::UnionTypes::StringOrCanvasGradientOrCanvasPattern;
use crate::dom::bindings::error::{ErrorResult, Fallible};
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::num::Finite;
use crate::dom::bindings::reflector::reflect_dom_object;
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::DOMString;
use crate::dom::canvasgradient::CanvasGradient;
use crate::dom::canvaspattern::CanvasPattern;
use crate::dom::canvasrenderingcontext2d::CanvasRenderingContext2D;
use crate::dom::dommatrix::DOMMatrix;
use crate::dom::paintworkletglobalscope::PaintWorkletGlobalScope;
use crate::script_runtime::CanGc;

#[dom_struct]
pub(crate) struct PaintRenderingContext2D {
    context: CanvasRenderingContext2D,
    #[no_trace]
    device_pixel_ratio: Cell<Scale<f32, CSSPixel, DevicePixel>>,
}

impl PaintRenderingContext2D {
    fn new_inherited(global: &PaintWorkletGlobalScope) -> PaintRenderingContext2D {
        let size = Size2D::zero();
        PaintRenderingContext2D {
            context: CanvasRenderingContext2D::new_inherited(global.upcast(), None, size),
            device_pixel_ratio: Cell::new(Scale::new(1.0)),
        }
    }

    pub(crate) fn new(
        global: &PaintWorkletGlobalScope,
        can_gc: CanGc,
    ) -> DomRoot<PaintRenderingContext2D> {
        reflect_dom_object(
            Box::new(PaintRenderingContext2D::new_inherited(global)),
            global,
            can_gc,
        )
    }

    pub(crate) fn send_data(&self, sender: IpcSender<CanvasImageData>) {
        self.context.send_data(sender);
    }

    pub(crate) fn take_missing_image_urls(&self) -> Vec<ServoUrl> {
        self.context.take_missing_image_urls()
    }

    pub(crate) fn set_bitmap_dimensions(
        &self,
        size: Size2D<f32, CSSPixel>,
        device_pixel_ratio: Scale<f32, CSSPixel, DevicePixel>,
    ) {
        let size = size * device_pixel_ratio;
        self.device_pixel_ratio.set(device_pixel_ratio);
        self.context
            .set_canvas_bitmap_dimensions(size.to_untyped().to_u64());
        self.scale_by_device_pixel_ratio();
    }

    fn scale_by_device_pixel_ratio(&self) {
        let device_pixel_ratio = self.device_pixel_ratio.get().get() as f64;
        if device_pixel_ratio != 1.0 {
            self.Scale(device_pixel_ratio, device_pixel_ratio);
        }
    }
}

impl PaintRenderingContext2DMethods<crate::DomTypeHolder> for PaintRenderingContext2D {
    // https://html.spec.whatwg.org/multipage/#dom-context-2d-save
    fn Save(&self) {
        self.context.Save()
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-restore
    fn Restore(&self) {
        self.context.Restore()
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-context-2d-reset>
    fn Reset(&self) {
        self.context.Reset()
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-scale
    fn Scale(&self, x: f64, y: f64) {
        self.context.Scale(x, y)
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-rotate
    fn Rotate(&self, angle: f64) {
        self.context.Rotate(angle)
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-translate
    fn Translate(&self, x: f64, y: f64) {
        self.context.Translate(x, y)
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-transform
    fn Transform(&self, a: f64, b: f64, c: f64, d: f64, e: f64, f: f64) {
        self.context.Transform(a, b, c, d, e, f)
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-gettransform
    fn GetTransform(&self, can_gc: CanGc) -> DomRoot<DOMMatrix> {
        self.context.GetTransform(can_gc)
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-settransform
    fn SetTransform(&self, a: f64, b: f64, c: f64, d: f64, e: f64, f: f64) {
        self.context.SetTransform(a, b, c, d, e, f);
        self.scale_by_device_pixel_ratio();
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-resettransform
    fn ResetTransform(&self) {
        self.context.ResetTransform();
        self.scale_by_device_pixel_ratio();
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-globalalpha
    fn GlobalAlpha(&self) -> f64 {
        self.context.GlobalAlpha()
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-globalalpha
    fn SetGlobalAlpha(&self, alpha: f64) {
        self.context.SetGlobalAlpha(alpha)
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-globalcompositeoperation
    fn GlobalCompositeOperation(&self) -> DOMString {
        self.context.GlobalCompositeOperation()
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-globalcompositeoperation
    fn SetGlobalCompositeOperation(&self, op_str: DOMString) {
        self.context.SetGlobalCompositeOperation(op_str)
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-fillrect
    fn FillRect(&self, x: f64, y: f64, width: f64, height: f64) {
        self.context.FillRect(x, y, width, height)
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-clearrect
    fn ClearRect(&self, x: f64, y: f64, width: f64, height: f64) {
        self.context.ClearRect(x, y, width, height)
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-strokerect
    fn StrokeRect(&self, x: f64, y: f64, width: f64, height: f64) {
        self.context.StrokeRect(x, y, width, height)
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-beginpath
    fn BeginPath(&self) {
        self.context.BeginPath()
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-closepath
    fn ClosePath(&self) {
        self.context.ClosePath()
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-fill
    fn Fill(&self, fill_rule: CanvasFillRule) {
        self.context.Fill(fill_rule)
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-stroke
    fn Stroke(&self) {
        self.context.Stroke()
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-clip
    fn Clip(&self, fill_rule: CanvasFillRule) {
        self.context.Clip(fill_rule)
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-ispointinpath
    fn IsPointInPath(&self, x: f64, y: f64, fill_rule: CanvasFillRule) -> bool {
        self.context.IsPointInPath(x, y, fill_rule)
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-drawimage
    fn DrawImage(&self, image: CanvasImageSource, dx: f64, dy: f64) -> ErrorResult {
        self.context.DrawImage(image, dx, dy)
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-drawimage
    fn DrawImage_(
        &self,
        image: CanvasImageSource,
        dx: f64,
        dy: f64,
        dw: f64,
        dh: f64,
    ) -> ErrorResult {
        self.context.DrawImage_(image, dx, dy, dw, dh)
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-drawimage
    fn DrawImage__(
        &self,
        image: CanvasImageSource,
        sx: f64,
        sy: f64,
        sw: f64,
        sh: f64,
        dx: f64,
        dy: f64,
        dw: f64,
        dh: f64,
    ) -> ErrorResult {
        self.context
            .DrawImage__(image, sx, sy, sw, sh, dx, dy, dw, dh)
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-moveto
    fn MoveTo(&self, x: f64, y: f64) {
        self.context.MoveTo(x, y)
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-lineto
    fn LineTo(&self, x: f64, y: f64) {
        self.context.LineTo(x, y)
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-rect
    fn Rect(&self, x: f64, y: f64, width: f64, height: f64) {
        self.context.Rect(x, y, width, height)
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-quadraticcurveto
    fn QuadraticCurveTo(&self, cpx: f64, cpy: f64, x: f64, y: f64) {
        self.context.QuadraticCurveTo(cpx, cpy, x, y)
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-beziercurveto
    fn BezierCurveTo(&self, cp1x: f64, cp1y: f64, cp2x: f64, cp2y: f64, x: f64, y: f64) {
        self.context.BezierCurveTo(cp1x, cp1y, cp2x, cp2y, x, y)
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-arc
    fn Arc(&self, x: f64, y: f64, r: f64, start: f64, end: f64, ccw: bool) -> ErrorResult {
        self.context.Arc(x, y, r, start, end, ccw)
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-arcto
    fn ArcTo(&self, cp1x: f64, cp1y: f64, cp2x: f64, cp2y: f64, r: f64) -> ErrorResult {
        self.context.ArcTo(cp1x, cp1y, cp2x, cp2y, r)
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-ellipse
    fn Ellipse(
        &self,
        x: f64,
        y: f64,
        rx: f64,
        ry: f64,
        rotation: f64,
        start: f64,
        end: f64,
        ccw: bool,
    ) -> ErrorResult {
        self.context
            .Ellipse(x, y, rx, ry, rotation, start, end, ccw)
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-imagesmoothingenabled
    fn ImageSmoothingEnabled(&self) -> bool {
        self.context.ImageSmoothingEnabled()
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-imagesmoothingenabled
    fn SetImageSmoothingEnabled(&self, value: bool) {
        self.context.SetImageSmoothingEnabled(value)
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-strokestyle
    fn StrokeStyle(&self) -> StringOrCanvasGradientOrCanvasPattern {
        self.context.StrokeStyle()
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-strokestyle
    fn SetStrokeStyle(&self, value: StringOrCanvasGradientOrCanvasPattern, can_gc: CanGc) {
        self.context.SetStrokeStyle(value, can_gc)
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-strokestyle
    fn FillStyle(&self) -> StringOrCanvasGradientOrCanvasPattern {
        self.context.FillStyle()
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-strokestyle
    fn SetFillStyle(&self, value: StringOrCanvasGradientOrCanvasPattern, can_gc: CanGc) {
        self.context.SetFillStyle(value, can_gc)
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-createlineargradient
    fn CreateLinearGradient(
        &self,
        x0: Finite<f64>,
        y0: Finite<f64>,
        x1: Finite<f64>,
        y1: Finite<f64>,
    ) -> DomRoot<CanvasGradient> {
        self.context.CreateLinearGradient(x0, y0, x1, y1)
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
        self.context.CreateRadialGradient(x0, y0, r0, x1, y1, r1)
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-createpattern
    fn CreatePattern(
        &self,
        image: CanvasImageSource,
        repetition: DOMString,
    ) -> Fallible<Option<DomRoot<CanvasPattern>>> {
        self.context.CreatePattern(image, repetition)
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-linewidth
    fn LineWidth(&self) -> f64 {
        self.context.LineWidth()
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-linewidth
    fn SetLineWidth(&self, width: f64) {
        self.context.SetLineWidth(width)
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-linecap
    fn LineCap(&self) -> CanvasLineCap {
        self.context.LineCap()
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-linecap
    fn SetLineCap(&self, cap: CanvasLineCap) {
        self.context.SetLineCap(cap)
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-linejoin
    fn LineJoin(&self) -> CanvasLineJoin {
        self.context.LineJoin()
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-linejoin
    fn SetLineJoin(&self, join: CanvasLineJoin) {
        self.context.SetLineJoin(join)
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-miterlimit
    fn MiterLimit(&self) -> f64 {
        self.context.MiterLimit()
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-miterlimit
    fn SetMiterLimit(&self, limit: f64) {
        self.context.SetMiterLimit(limit)
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-shadowoffsetx
    fn ShadowOffsetX(&self) -> f64 {
        self.context.ShadowOffsetX()
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-shadowoffsetx
    fn SetShadowOffsetX(&self, value: f64) {
        self.context.SetShadowOffsetX(value)
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-shadowoffsety
    fn ShadowOffsetY(&self) -> f64 {
        self.context.ShadowOffsetY()
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-shadowoffsety
    fn SetShadowOffsetY(&self, value: f64) {
        self.context.SetShadowOffsetY(value)
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-shadowblur
    fn ShadowBlur(&self) -> f64 {
        self.context.ShadowBlur()
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-shadowblur
    fn SetShadowBlur(&self, value: f64) {
        self.context.SetShadowBlur(value)
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-shadowcolor
    fn ShadowColor(&self) -> DOMString {
        self.context.ShadowColor()
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-shadowcolor
    fn SetShadowColor(&self, value: DOMString, can_gc: CanGc) {
        self.context.SetShadowColor(value, can_gc)
    }
}

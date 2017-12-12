/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use canvas_traits::canvas::CanvasImageData;
use canvas_traits::canvas::CanvasMsg;
use canvas_traits::canvas::FromLayoutMsg;
use dom::bindings::codegen::Bindings::CanvasRenderingContext2DBinding::CanvasFillRule;
use dom::bindings::codegen::Bindings::CanvasRenderingContext2DBinding::CanvasLineCap;
use dom::bindings::codegen::Bindings::CanvasRenderingContext2DBinding::CanvasLineJoin;
use dom::bindings::codegen::Bindings::CanvasRenderingContext2DBinding::CanvasRenderingContext2DMethods;
use dom::bindings::codegen::Bindings::PaintRenderingContext2DBinding;
use dom::bindings::codegen::Bindings::PaintRenderingContext2DBinding::PaintRenderingContext2DMethods;
use dom::bindings::codegen::UnionTypes::HTMLImageElementOrHTMLCanvasElementOrCanvasRenderingContext2DOrCSSStyleValue;
use dom::bindings::codegen::UnionTypes::StringOrCanvasGradientOrCanvasPattern;
use dom::bindings::error::ErrorResult;
use dom::bindings::error::Fallible;
use dom::bindings::inheritance::Castable;
use dom::bindings::num::Finite;
use dom::bindings::reflector::reflect_dom_object;
use dom::bindings::root::DomRoot;
use dom::bindings::str::DOMString;
use dom::canvasgradient::CanvasGradient;
use dom::canvaspattern::CanvasPattern;
use dom::canvasrenderingcontext2d::CanvasRenderingContext2D;
use dom::paintworkletglobalscope::PaintWorkletGlobalScope;
use dom::workletglobalscope::WorkletGlobalScope;
use dom_struct::dom_struct;
use euclid::Size2D;
use euclid::TypedScale;
use euclid::TypedSize2D;
use ipc_channel::ipc::IpcSender;
use servo_url::ServoUrl;
use std::cell::Cell;
use style_traits::CSSPixel;
use style_traits::DevicePixel;

#[dom_struct]
pub struct PaintRenderingContext2D {
    context: CanvasRenderingContext2D,
    device_pixel_ratio: Cell<TypedScale<f32, CSSPixel, DevicePixel>>,
}

impl PaintRenderingContext2D {
    fn new_inherited(global: &PaintWorkletGlobalScope) -> PaintRenderingContext2D {
        let size = Size2D::zero();
        let image_cache = global.image_cache();
        let base_url = global.upcast::<WorkletGlobalScope>().base_url();
        PaintRenderingContext2D {
            context: CanvasRenderingContext2D::new_inherited(global.upcast(), None, image_cache, base_url, size),
            device_pixel_ratio: Cell::new(TypedScale::new(1.0)),
        }
    }

    pub fn new(global: &PaintWorkletGlobalScope) -> DomRoot<PaintRenderingContext2D> {
        reflect_dom_object(Box::new(PaintRenderingContext2D::new_inherited(global)),
                           global,
                           PaintRenderingContext2DBinding::Wrap)
    }

    pub fn send_data(&self, sender: IpcSender<CanvasImageData>) {
        let msg = CanvasMsg::FromLayout(FromLayoutMsg::SendData(sender));
        let _ = self.context.get_ipc_renderer().send(msg);
    }

    pub fn take_missing_image_urls(&self) -> Vec<ServoUrl> {
        self.context.take_missing_image_urls()
    }

    pub fn set_bitmap_dimensions(&self,
                                 size: TypedSize2D<f32, CSSPixel>,
                                 device_pixel_ratio: TypedScale<f32, CSSPixel, DevicePixel>)
    {
        let size = size * device_pixel_ratio;
        self.device_pixel_ratio.set(device_pixel_ratio);
        self.context.set_bitmap_dimensions(size.to_untyped().to_i32());
        self.scale_by_device_pixel_ratio();
    }

    fn scale_by_device_pixel_ratio(&self) {
        let device_pixel_ratio = self.device_pixel_ratio.get().get() as f64;
        if device_pixel_ratio != 1.0 {
            self.Scale(device_pixel_ratio, device_pixel_ratio);
        }
    }
}

impl PaintRenderingContext2DMethods for PaintRenderingContext2D {
    // https://html.spec.whatwg.org/multipage/#dom-context-2d-save
    fn Save(&self) {
        self.context.Save()
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-restore
    fn Restore(&self) {
        self.context.Restore()
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
    fn DrawImage(&self,
                 image: HTMLImageElementOrHTMLCanvasElementOrCanvasRenderingContext2DOrCSSStyleValue,
                 dx: f64,
                 dy: f64)
                 -> ErrorResult {
        self.context.DrawImage(image, dx, dy)
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-drawimage
    fn DrawImage_(&self,
                  image: HTMLImageElementOrHTMLCanvasElementOrCanvasRenderingContext2DOrCSSStyleValue,
                  dx: f64,
                  dy: f64,
                  dw: f64,
                  dh: f64)
                  -> ErrorResult {
        self.context.DrawImage_(image, dx, dy, dw, dh)
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-drawimage
    fn DrawImage__(&self,
                   image: HTMLImageElementOrHTMLCanvasElementOrCanvasRenderingContext2DOrCSSStyleValue,
                   sx: f64,
                   sy: f64,
                   sw: f64,
                   sh: f64,
                   dx: f64,
                   dy: f64,
                   dw: f64,
                   dh: f64)
                   -> ErrorResult {
        self.context.DrawImage__(image, sx, sy, sw, sh, dx, dy, dw, dh)
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
    fn Ellipse(&self, x: f64, y: f64, rx: f64, ry: f64, rotation: f64, start: f64, end: f64, ccw: bool) -> ErrorResult {
        self.context.Ellipse(x, y, rx, ry, rotation, start, end, ccw)
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
    fn SetStrokeStyle(&self, value: StringOrCanvasGradientOrCanvasPattern) {
        self.context.SetStrokeStyle(value)
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-strokestyle
    fn FillStyle(&self) -> StringOrCanvasGradientOrCanvasPattern {
        self.context.FillStyle()
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-strokestyle
    fn SetFillStyle(&self, value: StringOrCanvasGradientOrCanvasPattern) {
        self.context.SetFillStyle(value)
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-createlineargradient
    fn CreateLinearGradient(&self,
                            x0: Finite<f64>,
                            y0: Finite<f64>,
                            x1: Finite<f64>,
                            y1: Finite<f64>)
                            -> DomRoot<CanvasGradient> {
        self.context.CreateLinearGradient(x0, y0, x1, y1)
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-createradialgradient
    fn CreateRadialGradient(&self,
                            x0: Finite<f64>,
                            y0: Finite<f64>,
                            r0: Finite<f64>,
                            x1: Finite<f64>,
                            y1: Finite<f64>,
                            r1: Finite<f64>)
                            -> Fallible<DomRoot<CanvasGradient>> {
        self.context.CreateRadialGradient(x0, y0, r0, x1, y1, r1)
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-createpattern
    fn CreatePattern(&self,
                     image: HTMLImageElementOrHTMLCanvasElementOrCanvasRenderingContext2DOrCSSStyleValue,
                     repetition: DOMString)
                     -> Fallible<DomRoot<CanvasPattern>> {
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
    fn SetShadowColor(&self, value: DOMString) {
        self.context.SetShadowColor(value)
    }

}

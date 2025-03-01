/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::dom::bindings::codegen::GenericBindings::CanvasRenderingContext2DBinding::CanvasRenderingContext2D_Binding::CanvasRenderingContext2DMethods;
use crate::canvas_context::CanvasContext as _;
use crate::dom::bindings::codegen::UnionTypes::HTMLCanvasElementOrOffscreenCanvas;
use canvas_traits::canvas::Canvas2dMsg;
use dom_struct::dom_struct;
use euclid::default::Size2D;
use ipc_channel::ipc::IpcSharedMemory;

use crate::dom::bindings::codegen::Bindings::CanvasRenderingContext2DBinding::{
    CanvasDirection, CanvasFillRule, CanvasImageSource, CanvasLineCap, CanvasLineJoin,
    CanvasTextAlign, CanvasTextBaseline,
};
use crate::dom::bindings::codegen::Bindings::OffscreenCanvasRenderingContext2DBinding::OffscreenCanvasRenderingContext2DMethods;
use crate::dom::bindings::codegen::UnionTypes::StringOrCanvasGradientOrCanvasPattern;
use crate::dom::bindings::error::{ErrorResult, Fallible};
use crate::dom::bindings::num::Finite;
use crate::dom::bindings::reflector::reflect_dom_object;
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::DOMString;
use crate::dom::canvasgradient::CanvasGradient;
use crate::dom::canvaspattern::CanvasPattern;
use crate::dom::dommatrix::DOMMatrix;
use crate::dom::globalscope::GlobalScope;
use crate::dom::imagedata::ImageData;
use crate::dom::offscreencanvas::OffscreenCanvas;
use crate::dom::textmetrics::TextMetrics;
use crate::script_runtime::CanGc;

use super::canvasrenderingcontext2d::CanvasRenderingContext2D;

#[dom_struct]
pub(crate) struct OffscreenCanvasRenderingContext2D {
    context: CanvasRenderingContext2D,
}

impl OffscreenCanvasRenderingContext2D {
    fn new_inherited(
        global: &GlobalScope,
        canvas: &OffscreenCanvas,
    ) -> OffscreenCanvasRenderingContext2D {
        let size = canvas.get_size().cast();
        OffscreenCanvasRenderingContext2D {
            context: CanvasRenderingContext2D::new_inherited(
                global,
                HTMLCanvasElementOrOffscreenCanvas::OffscreenCanvas(DomRoot::from_ref(canvas)),
                size,
            ),
        }
    }

    pub(crate) fn new(
        global: &GlobalScope,
        canvas: &OffscreenCanvas,
        can_gc: CanGc,
    ) -> DomRoot<OffscreenCanvasRenderingContext2D> {
        let boxed = Box::new(OffscreenCanvasRenderingContext2D::new_inherited(
            global, canvas,
        ));
        reflect_dom_object(boxed, global, can_gc)
    }

    pub(crate) fn set_canvas_bitmap_dimensions(&self, size: Size2D<u64>) {
        self.context.set_bitmap_dimensions(size.cast());
    }

    pub(crate) fn send_canvas_2d_msg(&self, msg: Canvas2dMsg) {
        self.context.send_canvas_2d_msg(msg)
    }

    pub(crate) fn origin_is_clean(&self) -> bool {
        self.context.origin_is_clean()
    }

    pub(crate) fn get_image_data_as_shared_memory(&self) -> Option<IpcSharedMemory> {
        self.context.get_image_data_as_shared_memory()
    }
}

impl OffscreenCanvasRenderingContext2DMethods<crate::DomTypeHolder>
    for OffscreenCanvasRenderingContext2D
{
    // https://html.spec.whatwg.org/multipage/offscreencontext2d-canvas
    fn Canvas(&self) -> DomRoot<OffscreenCanvas> {
        match self.context.canvas() {
            HTMLCanvasElementOrOffscreenCanvas::OffscreenCanvas(canvas) => canvas,
            _ => panic!("Should not be called from onscreen canvas"),
        }
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-fillrect
    fn FillRect(&self, x: f64, y: f64, width: f64, height: f64) {
        self.context.FillRect(x, y, width, height);
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-clearrect
    fn ClearRect(&self, x: f64, y: f64, width: f64, height: f64) {
        self.context.ClearRect(x, y, width, height);
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-strokerect
    fn StrokeRect(&self, x: f64, y: f64, width: f64, height: f64) {
        self.context.StrokeRect(x, y, width, height);
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

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-save
    fn Save(&self) {
        self.context.Save()
    }

    #[cfg_attr(crown, allow(crown::unrooted_must_root))]
    // https://html.spec.whatwg.org/multipage/#dom-context-2d-restore
    fn Restore(&self) {
        self.context.Restore()
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-context-2d-reset>
    fn Reset(&self) {
        self.context.Reset()
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

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-imagesmoothingenabled
    fn ImageSmoothingEnabled(&self) -> bool {
        self.context.ImageSmoothingEnabled()
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-imagesmoothingenabled
    fn SetImageSmoothingEnabled(&self, value: bool) {
        self.context.SetImageSmoothingEnabled(value)
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-filltext
    fn FillText(&self, text: DOMString, x: f64, y: f64, max_width: Option<f64>, can_gc: CanGc) {
        self.context.FillText(text, x, y, max_width, can_gc)
    }

    // https://html.spec.whatwg.org/multipage/#textmetrics
    fn MeasureText(&self, text: DOMString, can_gc: CanGc) -> DomRoot<TextMetrics> {
        self.context.MeasureText(text, can_gc)
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-font
    fn Font(&self) -> DOMString {
        self.context.Font()
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-font
    fn SetFont(&self, value: DOMString, can_gc: CanGc) {
        self.context.SetFont(value, can_gc)
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-textalign
    fn TextAlign(&self) -> CanvasTextAlign {
        self.context.TextAlign()
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-textalign
    fn SetTextAlign(&self, value: CanvasTextAlign) {
        self.context.SetTextAlign(value)
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-textbaseline
    fn TextBaseline(&self) -> CanvasTextBaseline {
        self.context.TextBaseline()
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-textbaseline
    fn SetTextBaseline(&self, value: CanvasTextBaseline) {
        self.context.SetTextBaseline(value)
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-direction
    fn Direction(&self) -> CanvasDirection {
        self.context.Direction()
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-direction
    fn SetDirection(&self, value: CanvasDirection) {
        self.context.SetDirection(value)
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

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-createimagedata
    fn CreateImageData(&self, sw: i32, sh: i32, can_gc: CanGc) -> Fallible<DomRoot<ImageData>> {
        self.context.CreateImageData(sw, sh, can_gc)
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-createimagedata
    fn CreateImageData_(
        &self,
        imagedata: &ImageData,
        can_gc: CanGc,
    ) -> Fallible<DomRoot<ImageData>> {
        self.context.CreateImageData_(imagedata, can_gc)
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-getimagedata
    fn GetImageData(
        &self,
        sx: i32,
        sy: i32,
        sw: i32,
        sh: i32,
        can_gc: CanGc,
    ) -> Fallible<DomRoot<ImageData>> {
        self.context.GetImageData(sx, sy, sw, sh, can_gc)
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-putimagedata
    fn PutImageData(&self, imagedata: &ImageData, dx: i32, dy: i32) {
        self.context.PutImageData(imagedata, dx, dy)
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-putimagedata
    #[allow(unsafe_code)]
    fn PutImageData_(
        &self,
        imagedata: &ImageData,
        dx: i32,
        dy: i32,
        dirty_x: i32,
        dirty_y: i32,
        dirty_width: i32,
        dirty_height: i32,
    ) {
        self.context.PutImageData_(
            imagedata,
            dx,
            dy,
            dirty_x,
            dirty_y,
            dirty_width,
            dirty_height,
        )
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

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-beginpath
    fn BeginPath(&self) {
        self.context.BeginPath()
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
        self.context.SetTransform(a, b, c, d, e, f)
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-resettransform
    fn ResetTransform(&self) {
        self.context.ResetTransform()
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-closepath
    fn ClosePath(&self) {
        self.context.ClosePath()
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
}

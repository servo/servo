/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use canvas_traits::canvas::{Canvas2dMsg, CanvasId, CanvasMsg};
use dom_struct::dom_struct;
use euclid::default::Size2D;
use ipc_channel::ipc::IpcSender;

use crate::canvas_state::CanvasState;
use crate::dom::bindings::codegen::Bindings::CanvasRenderingContext2DBinding::{
    CanvasDirection, CanvasFillRule, CanvasImageSource, CanvasLineCap, CanvasLineJoin,
    CanvasTextAlign, CanvasTextBaseline,
};
use crate::dom::bindings::codegen::Bindings::OffscreenCanvasRenderingContext2DBinding::OffscreenCanvasRenderingContext2DMethods;
use crate::dom::bindings::codegen::UnionTypes::StringOrCanvasGradientOrCanvasPattern;
use crate::dom::bindings::error::{ErrorResult, Fallible};
use crate::dom::bindings::num::Finite;
use crate::dom::bindings::reflector::{reflect_dom_object, DomGlobal, Reflector};
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::bindings::str::DOMString;
use crate::dom::canvasgradient::CanvasGradient;
use crate::dom::canvaspattern::CanvasPattern;
use crate::dom::dommatrix::DOMMatrix;
use crate::dom::globalscope::GlobalScope;
use crate::dom::htmlcanvaselement::HTMLCanvasElement;
use crate::dom::imagedata::ImageData;
use crate::dom::offscreencanvas::OffscreenCanvas;
use crate::dom::textmetrics::TextMetrics;
use crate::script_runtime::CanGc;

#[dom_struct]
pub(crate) struct OffscreenCanvasRenderingContext2D {
    reflector_: Reflector,
    canvas: Dom<OffscreenCanvas>,
    canvas_state: CanvasState,
    htmlcanvas: Option<Dom<HTMLCanvasElement>>,
}

impl OffscreenCanvasRenderingContext2D {
    fn new_inherited(
        global: &GlobalScope,
        canvas: &OffscreenCanvas,
        htmlcanvas: Option<&HTMLCanvasElement>,
    ) -> OffscreenCanvasRenderingContext2D {
        OffscreenCanvasRenderingContext2D {
            reflector_: Reflector::new(),
            canvas: Dom::from_ref(canvas),
            htmlcanvas: htmlcanvas.map(Dom::from_ref),
            canvas_state: CanvasState::new(global, canvas.get_size()),
        }
    }

    pub(crate) fn new(
        global: &GlobalScope,
        canvas: &OffscreenCanvas,
        htmlcanvas: Option<&HTMLCanvasElement>,
    ) -> DomRoot<OffscreenCanvasRenderingContext2D> {
        let boxed = Box::new(OffscreenCanvasRenderingContext2D::new_inherited(
            global, canvas, htmlcanvas,
        ));
        reflect_dom_object(boxed, global, CanGc::note())
    }

    pub(crate) fn set_canvas_bitmap_dimensions(&self, size: Size2D<u64>) {
        self.canvas_state.set_bitmap_dimensions(size);
    }

    pub(crate) fn send_canvas_2d_msg(&self, msg: Canvas2dMsg) {
        self.canvas_state.send_canvas_2d_msg(msg)
    }

    pub(crate) fn origin_is_clean(&self) -> bool {
        self.canvas_state.origin_is_clean()
    }

    pub(crate) fn get_canvas_id(&self) -> CanvasId {
        self.canvas_state.get_canvas_id()
    }

    pub(crate) fn get_ipc_renderer(&self) -> IpcSender<CanvasMsg> {
        self.canvas_state.get_ipc_renderer().clone()
    }
}

impl OffscreenCanvasRenderingContext2DMethods<crate::DomTypeHolder>
    for OffscreenCanvasRenderingContext2D
{
    // https://html.spec.whatwg.org/multipage/offscreencontext2d-canvas
    fn Canvas(&self) -> DomRoot<OffscreenCanvas> {
        DomRoot::from_ref(&self.canvas)
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-fillrect
    fn FillRect(&self, x: f64, y: f64, width: f64, height: f64) {
        self.canvas_state.fill_rect(x, y, width, height);
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-clearrect
    fn ClearRect(&self, x: f64, y: f64, width: f64, height: f64) {
        self.canvas_state.clear_rect(x, y, width, height);
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-strokerect
    fn StrokeRect(&self, x: f64, y: f64, width: f64, height: f64) {
        self.canvas_state.stroke_rect(x, y, width, height);
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-shadowoffsetx
    fn ShadowOffsetX(&self) -> f64 {
        self.canvas_state.shadow_offset_x()
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-shadowoffsetx
    fn SetShadowOffsetX(&self, value: f64) {
        self.canvas_state.set_shadow_offset_x(value)
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-shadowoffsety
    fn ShadowOffsetY(&self) -> f64 {
        self.canvas_state.shadow_offset_y()
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-shadowoffsety
    fn SetShadowOffsetY(&self, value: f64) {
        self.canvas_state.set_shadow_offset_y(value)
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-shadowblur
    fn ShadowBlur(&self) -> f64 {
        self.canvas_state.shadow_blur()
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-shadowblur
    fn SetShadowBlur(&self, value: f64) {
        self.canvas_state.set_shadow_blur(value)
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-shadowcolor
    fn ShadowColor(&self) -> DOMString {
        self.canvas_state.shadow_color()
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-shadowcolor
    fn SetShadowColor(&self, value: DOMString, can_gc: CanGc) {
        self.canvas_state
            .set_shadow_color(self.htmlcanvas.as_deref(), value, can_gc)
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-strokestyle
    fn StrokeStyle(&self) -> StringOrCanvasGradientOrCanvasPattern {
        self.canvas_state.stroke_style()
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-strokestyle
    fn SetStrokeStyle(&self, value: StringOrCanvasGradientOrCanvasPattern, can_gc: CanGc) {
        self.canvas_state
            .set_stroke_style(self.htmlcanvas.as_deref(), value, can_gc)
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-strokestyle
    fn FillStyle(&self) -> StringOrCanvasGradientOrCanvasPattern {
        self.canvas_state.fill_style()
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-strokestyle
    fn SetFillStyle(&self, value: StringOrCanvasGradientOrCanvasPattern, can_gc: CanGc) {
        self.canvas_state
            .set_fill_style(self.htmlcanvas.as_deref(), value, can_gc)
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
            .create_linear_gradient(&self.global(), x0, y0, x1, y1)
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
            .create_radial_gradient(&self.global(), x0, y0, r0, x1, y1, r1)
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-createpattern
    fn CreatePattern(
        &self,
        image: CanvasImageSource,
        repetition: DOMString,
    ) -> Fallible<Option<DomRoot<CanvasPattern>>> {
        self.canvas_state
            .create_pattern(&self.global(), image, repetition)
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-save
    fn Save(&self) {
        self.canvas_state.save()
    }

    #[cfg_attr(crown, allow(crown::unrooted_must_root))]
    // https://html.spec.whatwg.org/multipage/#dom-context-2d-restore
    fn Restore(&self) {
        self.canvas_state.restore()
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-context-2d-reset>
    fn Reset(&self) {
        self.canvas_state.reset()
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-globalalpha
    fn GlobalAlpha(&self) -> f64 {
        self.canvas_state.global_alpha()
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-globalalpha
    fn SetGlobalAlpha(&self, alpha: f64) {
        self.canvas_state.set_global_alpha(alpha)
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-globalcompositeoperation
    fn GlobalCompositeOperation(&self) -> DOMString {
        self.canvas_state.global_composite_operation()
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-globalcompositeoperation
    fn SetGlobalCompositeOperation(&self, op_str: DOMString) {
        self.canvas_state.set_global_composite_operation(op_str)
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-imagesmoothingenabled
    fn ImageSmoothingEnabled(&self) -> bool {
        self.canvas_state.image_smoothing_enabled()
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-imagesmoothingenabled
    fn SetImageSmoothingEnabled(&self, value: bool) {
        self.canvas_state.set_image_smoothing_enabled(value)
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-filltext
    fn FillText(&self, text: DOMString, x: f64, y: f64, max_width: Option<f64>, can_gc: CanGc) {
        self.canvas_state
            .fill_text(self.htmlcanvas.as_deref(), text, x, y, max_width, can_gc)
    }

    // https://html.spec.whatwg.org/multipage/#textmetrics
    fn MeasureText(&self, text: DOMString, can_gc: CanGc) -> DomRoot<TextMetrics> {
        self.canvas_state
            .measure_text(&self.global(), self.htmlcanvas.as_deref(), text, can_gc)
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-font
    fn Font(&self) -> DOMString {
        self.canvas_state.font()
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-font
    fn SetFont(&self, value: DOMString, can_gc: CanGc) {
        self.canvas_state
            .set_font(self.htmlcanvas.as_deref(), value, can_gc)
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-textalign
    fn TextAlign(&self) -> CanvasTextAlign {
        self.canvas_state.text_align()
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-textalign
    fn SetTextAlign(&self, value: CanvasTextAlign) {
        self.canvas_state.set_text_align(value)
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-textbaseline
    fn TextBaseline(&self) -> CanvasTextBaseline {
        self.canvas_state.text_baseline()
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-textbaseline
    fn SetTextBaseline(&self, value: CanvasTextBaseline) {
        self.canvas_state.set_text_baseline(value)
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-direction
    fn Direction(&self) -> CanvasDirection {
        self.canvas_state.direction()
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-direction
    fn SetDirection(&self, value: CanvasDirection) {
        self.canvas_state.set_direction(value)
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-linewidth
    fn LineWidth(&self) -> f64 {
        self.canvas_state.line_width()
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-linewidth
    fn SetLineWidth(&self, width: f64) {
        self.canvas_state.set_line_width(width)
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-linecap
    fn LineCap(&self) -> CanvasLineCap {
        self.canvas_state.line_cap()
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-linecap
    fn SetLineCap(&self, cap: CanvasLineCap) {
        self.canvas_state.set_line_cap(cap)
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-linejoin
    fn LineJoin(&self) -> CanvasLineJoin {
        self.canvas_state.line_join()
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-linejoin
    fn SetLineJoin(&self, join: CanvasLineJoin) {
        self.canvas_state.set_line_join(join)
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-miterlimit
    fn MiterLimit(&self) -> f64 {
        self.canvas_state.miter_limit()
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-miterlimit
    fn SetMiterLimit(&self, limit: f64) {
        self.canvas_state.set_miter_limit(limit)
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-createimagedata
    fn CreateImageData(&self, sw: i32, sh: i32, can_gc: CanGc) -> Fallible<DomRoot<ImageData>> {
        self.canvas_state
            .create_image_data(&self.global(), sw, sh, can_gc)
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-createimagedata
    fn CreateImageData_(
        &self,
        imagedata: &ImageData,
        can_gc: CanGc,
    ) -> Fallible<DomRoot<ImageData>> {
        self.canvas_state
            .create_image_data_(&self.global(), imagedata, can_gc)
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
        self.canvas_state.get_image_data(
            self.canvas.get_size(),
            &self.global(),
            sx,
            sy,
            sw,
            sh,
            can_gc,
        )
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-putimagedata
    fn PutImageData(&self, imagedata: &ImageData, dx: i32, dy: i32) {
        self.canvas_state
            .put_image_data(self.canvas.get_size(), imagedata, dx, dy)
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
        self.canvas_state.put_image_data_(
            self.canvas.get_size(),
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
        self.canvas_state
            .draw_image(self.htmlcanvas.as_deref(), image, dx, dy)
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
        self.canvas_state
            .draw_image_(self.htmlcanvas.as_deref(), image, dx, dy, dw, dh)
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
        self.canvas_state.draw_image__(
            self.htmlcanvas.as_deref(),
            image,
            sx,
            sy,
            sw,
            sh,
            dx,
            dy,
            dw,
            dh,
        )
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-beginpath
    fn BeginPath(&self) {
        self.canvas_state.begin_path()
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-fill
    fn Fill(&self, fill_rule: CanvasFillRule) {
        self.canvas_state.fill(fill_rule)
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-stroke
    fn Stroke(&self) {
        self.canvas_state.stroke()
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-clip
    fn Clip(&self, fill_rule: CanvasFillRule) {
        self.canvas_state.clip(fill_rule)
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-ispointinpath
    fn IsPointInPath(&self, x: f64, y: f64, fill_rule: CanvasFillRule) -> bool {
        self.canvas_state
            .is_point_in_path(&self.global(), x, y, fill_rule)
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-scale
    fn Scale(&self, x: f64, y: f64) {
        self.canvas_state.scale(x, y)
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-rotate
    fn Rotate(&self, angle: f64) {
        self.canvas_state.rotate(angle)
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-translate
    fn Translate(&self, x: f64, y: f64) {
        self.canvas_state.translate(x, y)
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-transform
    fn Transform(&self, a: f64, b: f64, c: f64, d: f64, e: f64, f: f64) {
        self.canvas_state.transform(a, b, c, d, e, f)
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-gettransform
    fn GetTransform(&self, can_gc: CanGc) -> DomRoot<DOMMatrix> {
        self.canvas_state.get_transform(&self.global(), can_gc)
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-settransform
    fn SetTransform(&self, a: f64, b: f64, c: f64, d: f64, e: f64, f: f64) {
        self.canvas_state.set_transform(a, b, c, d, e, f)
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-resettransform
    fn ResetTransform(&self) {
        self.canvas_state.reset_transform()
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-closepath
    fn ClosePath(&self) {
        self.canvas_state.close_path()
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-moveto
    fn MoveTo(&self, x: f64, y: f64) {
        self.canvas_state.move_to(x, y)
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-lineto
    fn LineTo(&self, x: f64, y: f64) {
        self.canvas_state.line_to(x, y)
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-rect
    fn Rect(&self, x: f64, y: f64, width: f64, height: f64) {
        self.canvas_state.rect(x, y, width, height)
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-quadraticcurveto
    fn QuadraticCurveTo(&self, cpx: f64, cpy: f64, x: f64, y: f64) {
        self.canvas_state.quadratic_curve_to(cpx, cpy, x, y)
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-beziercurveto
    fn BezierCurveTo(&self, cp1x: f64, cp1y: f64, cp2x: f64, cp2y: f64, x: f64, y: f64) {
        self.canvas_state
            .bezier_curve_to(cp1x, cp1y, cp2x, cp2y, x, y)
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-arc
    fn Arc(&self, x: f64, y: f64, r: f64, start: f64, end: f64, ccw: bool) -> ErrorResult {
        self.canvas_state.arc(x, y, r, start, end, ccw)
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-arcto
    fn ArcTo(&self, cp1x: f64, cp1y: f64, cp2x: f64, cp2y: f64, r: f64) -> ErrorResult {
        self.canvas_state.arc_to(cp1x, cp1y, cp2x, cp2y, r)
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
        self.canvas_state
            .ellipse(x, y, rx, ry, rotation, start, end, ccw)
    }
}

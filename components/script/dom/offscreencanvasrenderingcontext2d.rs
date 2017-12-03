/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use canvas_traits::canvas::{Canvas2dMsg, CanvasMsg};
use canvas_traits::canvas::{CompositionOrBlending, FillOrStrokeStyle, FillRule};
use canvas_traits::canvas::{LineCapStyle, LineJoinStyle, LinearGradientStyle};
use canvas_traits::canvas::{RadialGradientStyle, RepetitionStyle, byte_swap_and_premultiply};
use cssparser::{Parser, ParserInput, RGBA};
use cssparser::Color as CSSColor;
use dom::bindings::cell::DomRefCell;
use dom::bindings::codegen::Bindings::CanvasRenderingContext2DBinding::CanvasFillRule;
use dom::bindings::codegen::Bindings::CanvasRenderingContext2DBinding::CanvasImageSource;
use dom::bindings::codegen::Bindings::CanvasRenderingContext2DBinding::CanvasLineCap;
use dom::bindings::codegen::Bindings::CanvasRenderingContext2DBinding::CanvasLineJoin;
use dom::bindings::codegen::Bindings::CanvasRenderingContext2DBinding::CanvasRenderingContext2DMethods;
use dom::bindings::codegen::Bindings::ImageDataBinding::ImageDataMethods;
use dom::bindings::codegen::Bindings::OffscreenCanvasRenderingContext2DBinding;
use dom::bindings::codegen::Bindings::OffscreenCanvasRenderingContext2DBinding::
OffscreenCanvasRenderingContext2DMethods;
use dom::bindings::codegen::UnionTypes::StringOrCanvasGradientOrCanvasPattern;
use dom::bindings::error::{Error, ErrorResult, Fallible};
use dom::bindings::num::Finite;
use dom::bindings::reflector::{DomObject, Reflector, reflect_dom_object};
use dom::bindings::root::{Dom, DomRoot, LayoutDom};
use dom::bindings::str::DOMString;
use dom::canvasgradient::{CanvasGradient, CanvasGradientStyle};
use dom::canvaspattern::CanvasPattern;
use dom::canvasrenderingcontext2d::{LayoutCanvasRenderingContext2DHelpers,CanvasRenderingContext2D};
use dom::globalscope::GlobalScope;
use dom::imagedata::ImageData;
use dom::node::{document_from_node, window_from_node};
use dom::offscreencanvas::OffscreenCanvas;
use dom_struct::dom_struct;
use euclid::{Transform2D, Point2D, Rect, Size2D};
use ipc_channel::ipc::{self, IpcSender};
use net_traits::image::base::PixelFormat;
use net_traits::image_cache::CanRequestImages;
use net_traits::image_cache::ImageCache;
use net_traits::image_cache::ImageOrMetadataAvailable;
use net_traits::image_cache::ImageResponse;
use net_traits::image_cache::ImageState;
use net_traits::image_cache::UsePlaceholder;
use script_traits::ScriptMsg;
use servo_url::ServoUrl;
use std::{cmp, fmt, mem};
use std::cell::Cell;
use std::str::FromStr;
use std::sync::Arc;

const DEFAULT_WIDTH: u32 = 300;
const DEFAULT_HEIGHT: u32 = 150;

#[must_root]
#[derive(Clone, JSTraceable, MallocSizeOf)]
#[allow(dead_code)]
enum CanvasFillOrStrokeStyle {
    Color(RGBA),
    Gradient(Dom<CanvasGradient>),
    Pattern(Dom<CanvasPattern>),
}

#[dom_struct]
pub struct OffscreenCanvasRenderingContext2D {
    reflector_: Reflector,
    base: Dom<CanvasRenderingContext2D>,
}

#[must_root]
#[derive(Clone, JSTraceable, MallocSizeOf)]
struct CanvasContextState {
    global_alpha: f64,
    global_composition: CompositionOrBlending,
    image_smoothing_enabled: bool,
    fill_style: CanvasFillOrStrokeStyle,
    stroke_style: CanvasFillOrStrokeStyle,
    line_width: f64,
    line_cap: LineCapStyle,
    line_join: LineJoinStyle,
    miter_limit: f64,
    transform: Transform2D<f32>,
    shadow_offset_x: f64,
    shadow_offset_y: f64,
    shadow_blur: f64,
    shadow_color: RGBA,
}

fn is_rect_valid(rect: Rect<f64>) -> bool {
    rect.size.width > 0.0 && rect.size.height > 0.0
}

impl CanvasContextState {
    fn new() -> CanvasContextState {
        let black = RGBA::new(0, 0, 0, 255);
        CanvasContextState {
            global_alpha: 1.0,
            global_composition: CompositionOrBlending::default(),
            image_smoothing_enabled: true,
            fill_style: CanvasFillOrStrokeStyle::Color(black),
            stroke_style: CanvasFillOrStrokeStyle::Color(black),
            line_width: 1.0,
            line_cap: LineCapStyle::Butt,
            line_join: LineJoinStyle::Miter,
            miter_limit: 10.0,
            transform: Transform2D::identity(),
            shadow_offset_x: 0.0,
            shadow_offset_y: 0.0,
            shadow_blur: 0.0,
            shadow_color: RGBA::transparent(),
        }
    }
}


impl OffscreenCanvasRenderingContext2D {
    pub fn new_inherited(global: &GlobalScope,
                         canvas: Option<&OffscreenCanvas>,
                         image_cache: Arc<ImageCache>,
                         base_url: ServoUrl,
                         size: Size2D<i32>)
                         -> OffscreenCanvasRenderingContext2D {
        let base = CanvasRenderingContext2D::new(global, &canvas, size);
        Some(OffscreenCanvasRenderingContext2D {
            reflector_: Reflector::new(),
            base: Dom::from_ref(&*base),
        })
    }

    pub fn new(global: &GlobalScope,
               canvas: &OffscreenCanvas,
               size: Size2D<i32>)
               -> DomRoot<OffscreenCanvasRenderingContext2D> {
                   let window = window_from_node(canvas);
                   let image_cache = window.image_cache();
                   let base_url = window.get_url();
                   let boxed = Box::new(OffscreenCanvasRenderingContext2D::new_inherited(
                       global, Some(canvas), image_cache, base_url, size
                   ));
                   reflect_dom_object(boxed, global, OffscreenCanvasRenderingContext2DBinding::Wrap)
    }

    fn draw_image(&self,
                  image: CanvasImageSource,
                  sx: f64,
                  sy: f64,
                  sw: Option<f64>,
                  sh: Option<f64>,
                  dx: f64,
                  dy: f64,
                  dw: Option<f64>,
                  dh: Option<f64>)
                  -> ErrorResult {
        self.base.draw_image(image,sx,sy,sw,sh,dx,dy,dw,dh)
    }

    fn set_origin_unclean(&self) {
        self.base.set_origin_unclean();
    }

    #[inline]
    fn request_image_from_cache(&self, url: ServoUrl) -> ImageResponse {
        self.base.request_image_from_cache(url)
    }
    pub fn origin_is_clean(&self) -> bool {
        self.base.origin_is_clean()
    }

    fn fetch_and_draw_image_data(&self,
                                 url: ServoUrl,
                                 sx: f64,
                                 sy: f64,
                                 sw: Option<f64>,
                                 sh: Option<f64>,
                                 dx: f64,
                                 dy: f64,
                                 dw: Option<f64>,
                                 dh: Option<f64>)
                                 -> ErrorResult {
        self.base.fetch_and_draw_image_data(url,sx,sy,sw,sh,dx,dy,dw,dh)
    }

    fn fetch_image_data(&self, url: ServoUrl) -> Option<(Vec<u8>, Size2D<i32>)> {
        self.base.fetch_image_data(url)
    }
    fn draw_html_canvas_element(&self,
                                canvas: &OffscreenCanvas,
                                sx: f64,
                                sy: f64,
                                sw: Option<f64>,
                                sh: Option<f64>,
                                dx: f64,
                                dy: f64,
                                dw: Option<f64>,
                                dh: Option<f64>)
                                -> ErrorResult {
    //self.base.draw_html_canvas_element(canvas,sx,sy,sw,sh,dx,dy,dw,dh)
    unimplemented!()
    }
}

pub trait LayoutOffscreenCanvasRenderingContext2DHelpers {
    #[allow(unsafe_code)]
    unsafe fn get_ipc_renderer(&self) -> IpcSender<CanvasMsg>;
}

pub fn parse_color(string: &str) -> Result<RGBA, ()> {
    let mut input = ParserInput::new(string);
    let mut parser = Parser::new(&mut input);
    match CSSColor::parse(&mut parser) {
        Ok(CSSColor::RGBA(rgba)) => {
            if parser.is_exhausted() {
                Ok(rgba)
            } else {
                Err(())
            }
        },
        _ => Err(()),
    }
}

impl LayoutOffscreenCanvasRenderingContext2DHelpers for LayoutDom<OffscreenCanvasRenderingContext2D> {
    #[allow(unsafe_code)]
    unsafe fn get_ipc_renderer(&self) -> IpcSender<CanvasMsg> {
        self.base.get_ipc_renderer()
    }
}


// https://html.spec.whatwg.org/multipage/#serialisation-of-a-colour
fn serialize<W>(color: &RGBA, dest: &mut W) -> fmt::Result
    where W: fmt::Write
{
    let red = color.red;
    let green = color.green;
    let blue = color.blue;

    if color.alpha == 255 {
        write!(dest,
               "#{:x}{:x}{:x}{:x}{:x}{:x}",
               red >> 4,
               red & 0xF,
               green >> 4,
               green & 0xF,
               blue >> 4,
               blue & 0xF)
    } else {
        write!(dest, "rgba({}, {}, {}, {})", red, green, blue, color.alpha_f32())
    }
}

impl OffscreenCanvasRenderingContext2DMethods for OffscreenCanvasRenderingContext2D {
    // https://html.spec.whatwg.org/multipage/#dom-context-2d-canvas
    fn Canvas(&self) -> DomRoot<OffscreenCanvas> {
        // This method is not called from a paint worklet rendering context,
        // so it's OK to panic if self.canvas is None.
        unimplemented!()
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-globalcompositeoperation
    fn GlobalCompositeOperation(&self) -> DOMString {
        self.base.GlobalCompositeOperation()
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-globalcompositeoperation
    fn SetGlobalCompositeOperation(&self, op_str: DOMString) {
        self.base.SetGlobalCompositeOperation(op_str)
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-drawimage
    fn DrawImage(&self,
                 image: CanvasImageSource,
                 dx: f64,
                 dy: f64)
                 -> ErrorResult {
        self.base.DrawImage(image, dx, dy)
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-drawimage
    fn Commit(&self) {
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-drawimage
    fn DrawImage_(&self,
                  image: CanvasImageSource,
                  dx: f64,
                  dy: f64,
                  dw: f64,
                  dh: f64)
                  -> ErrorResult {
        self.base.DrawImage_(image, dx, dy, dw, dh)
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-drawimage
    fn DrawImage__(&self,
                   image: CanvasImageSource,
                   sx: f64,
                   sy: f64,
                   sw: f64,
                   sh: f64,
                   dx: f64,
                   dy: f64,
                   dw: f64,
                   dh: f64)
                   -> ErrorResult {
        self.base.DrawImage__(image, sx, sy, sw, sh, dx, dy, dw, dh)
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-beginpath
    fn BeginPath(&self) {
        self.base.BeginPath()
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-fill
    fn Fill(&self, cx: CanvasFillRule) {
        self.base.Fill(cx)
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-stroke
    fn Stroke(&self) {
        self.base.Stroke()
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-clip
    fn Clip(&self, cx: CanvasFillRule) {
        // TODO: Process fill rule
        self.base.Clip(cx)
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-strokestyle
    fn StrokeStyle(&self) -> StringOrCanvasGradientOrCanvasPattern {
        self.base.StrokeStyle()
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-strokestyle
    fn SetStrokeStyle(&self, value: StringOrCanvasGradientOrCanvasPattern) {
        self.SetStrokeStyle(value)
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-strokestyle
    fn FillStyle(&self) -> StringOrCanvasGradientOrCanvasPattern {
        self.base.FillStyle()
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-createlineargradient
    fn CreateLinearGradient(&self,
                            x0: Finite<f64>,
                            y0: Finite<f64>,
                            x1: Finite<f64>,
                            y1: Finite<f64>)
                            -> DomRoot<CanvasGradient> {
        self.base.CreateLinearGradient(x0, y0, x1, y1)
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
        self.base.CreateRadialGradient(x0, y0, r0, x1, y1, r1)
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-createpattern
   fn CreatePattern(&self,
                     image: CanvasImageSource,
                     mut repetition: DOMString)
                     -> Fallible<DomRoot<CanvasPattern>> {
        self.base.CreatePattern(image, repetition)
    }
    // https://html.spec.whatwg.org/multipage/#dom-context-2d-createimagedata
    fn CreateImageData(&self, sw: Finite<f64>, sh: Finite<f64>) -> Fallible<DomRoot<ImageData>> {
        self.base.CreateImageData(sw, sh)
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-createimagedata
    fn CreateImageData_(&self, imagedata: &ImageData) -> Fallible<DomRoot<ImageData>> {
        self.base.CreateImageData_(imagedata)
    }


    // https://html.spec.whatwg.org/multipage/#dom-context-2d-getimagedata
    fn GetImageData(&self,
                    sx: Finite<f64>,
                    sy: Finite<f64>,
                    sw: Finite<f64>,
                    sh: Finite<f64>)
                    -> Fallible<DomRoot<ImageData>> {
        self.base.GetImageData(sx,sy,sw,sh)
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-putimagedata
    fn PutImageData(&self, imagedata: &ImageData, dx: Finite<f64>, dy: Finite<f64>) {
        self.base.PutImageData(imagedata, dx, dy)
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-putimagedata
    fn PutImageData_(&self,
                     imagedata: &ImageData,
                     dx: Finite<f64>,
                     dy: Finite<f64>,
                     dirty_x: Finite<f64>,
                     dirty_y: Finite<f64>,
                     dirty_width: Finite<f64>,
                     dirty_height: Finite<f64>) {
        self.base.PutImageData_(imagedata, dx, dy, dirty_x, dirty_y, dirty_width, dirty_height)
    }
    // https://html.spec.whatwg.org/multipage/#dom-context-2d-imagesmoothingenabled
    fn ImageSmoothingEnabled(&self) -> bool {
        self.base.ImageSmoothingEnabled()
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-imagesmoothingenabled
    fn SetImageSmoothingEnabled(&self, value: bool) {
        self.base.SetImageSmoothingEnabled(value)
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-closepath
    fn ClosePath(&self) {
        self.base.ClosePath()
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-moveto
    fn MoveTo(&self, x: f64, y: f64) {
        self.base.MoveTo(x, y)
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-lineto
    fn LineTo(&self, x: f64, y: f64) {
        self.base.LineTo(x, y)
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-quadraticcurveto
    fn QuadraticCurveTo(&self, cpx: f64, cpy: f64, x: f64, y: f64) {
        self.base.QuadraticCurveTo(cpx, cpy, x, y)
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-beziercurveto
    fn BezierCurveTo(&self, cp1x: f64, cp1y: f64, cp2x: f64, cp2y: f64, x: f64, y: f64) {
        self.base.BezierCurveTo(cp1x, cp1y, cp2x, cp2y, x, y)
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-arc
    fn Arc(&self, x: f64, y: f64, r: f64, start: f64, end: f64, ccw: bool) -> ErrorResult {
        self.base.Arc(x, y, r, start, end, ccw)
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-arcto
    fn ArcTo(&self, cp1x: f64, cp1y: f64, cp2x: f64, cp2y: f64, r: f64) -> ErrorResult {
        self.base.ArcTo(cp1x, cp1y, cp2x, cp2y, r)
    }
    // https://html.spec.whatwg.org/multipage/#dom-context-2d-rect
    fn Rect(&self, x: f64, y: f64, width: f64, height: f64) {
        self.base.Rect(x, y, width, height)
    }
    // https://html.spec.whatwg.org/multipage/#dom-context-2d-ellipse
    fn Ellipse(&self, x: f64, y: f64,
         rx: f64, ry: f64, rotation: f64, start: f64,
          end: f64, ccw: bool) -> ErrorResult {
              self.base.Ellipse(x, y, rx, ry, rotation, start, end, ccw)
    }
    // https://html.spec.whatwg.org/multipage/#dom-context-2d-linewidth
    fn LineWidth(&self) -> f64 {
        self.base.LineWidth()
    }

        // https://html.spec.whatwg.org/multipage/#dom-context-2d-linewidth
        fn SetLineWidth(&self, width: f64) {
            self.base.SetLineWidth(width)
        }

        // https://html.spec.whatwg.org/multipage/#dom-context-2d-linecap
        fn LineCap(&self) -> CanvasLineCap {
            self.base.LineCap()
        }

        // https://html.spec.whatwg.org/multipage/#dom-context-2d-linecap
        fn SetLineCap(&self, cap: CanvasLineCap) {
            self.base.SetLineCap(cap)
        }

        // https://html.spec.whatwg.org/multipage/#dom-context-2d-linejoin
        fn LineJoin(&self) -> CanvasLineJoin {
            self.base.LineJoin()
        }

        // https://html.spec.whatwg.org/multipage/#dom-context-2d-linejoin
        fn SetLineJoin(&self, join: CanvasLineJoin) {
            self.base.SetLineJoin(join)
        }

        // https://html.spec.whatwg.org/multipage/#dom-context-2d-miterlimit
        fn MiterLimit(&self) -> f64 {
            self.base.MiterLimit()
        }

        // https://html.spec.whatwg.org/multipage/#dom-context-2d-miterlimit
        fn SetMiterLimit(&self, limit: f64) {
            self.base.SetMiterLimit(limit)
        }

        // https://html.spec.whatwg.org/multipage/#dom-context-2d-fillrect
        fn FillRect(&self, x: f64, y: f64, width: f64, height: f64) {
            self.base.FillRect(x, y, width, height)
        }

        // https://html.spec.whatwg.org/multipage/#dom-context-2d-clearrect
        fn ClearRect(&self, x: f64, y: f64, width: f64, height: f64) {
            self.base.ClearRect(x, y, width, height)
        }

        // https://html.spec.whatwg.org/multipage/#dom-context-2d-strokerect
        fn StrokeRect(&self, x: f64, y: f64, width: f64, height: f64) {
            self.base.StrokeRect(x, y, width, height)
        }

        // https://html.spec.whatwg.org/multipage/#dom-context-2d-shadowoffsetx
        fn ShadowOffsetX(&self) -> f64 {
            self.base.ShadowOffsetX()
        }

        // https://html.spec.whatwg.org/multipage/#dom-context-2d-shadowoffsetx
        fn SetShadowOffsetX(&self, value: f64) {
            self.base.SetShadowOffsetX(value)
        }

        // https://html.spec.whatwg.org/multipage/#dom-context-2d-shadowoffsety
        fn ShadowOffsetY(&self) -> f64 {
            self.base.ShadowOffsetY()
        }

        // https://html.spec.whatwg.org/multipage/#dom-context-2d-shadowoffsety
        fn SetShadowOffsetY(&self, value: f64) {
            self.base.SetShadowOffsetY(value)
        }

        // https://html.spec.whatwg.org/multipage/#dom-context-2d-shadowblur
        fn ShadowBlur(&self) -> f64 {
            self.base.ShadowBlur()
        }

        // https://html.spec.whatwg.org/multipage/#dom-context-2d-shadowblur
        fn SetShadowBlur(&self, value: f64) {
            self.base.SetShadowBlur(value)
        }

        // https://html.spec.whatwg.org/multipage/#dom-context-2d-shadowcolor
        fn ShadowColor(&self) -> DOMString {
            self.base.ShadowColor()
        }

        // https://html.spec.whatwg.org/multipage/#dom-context-2d-shadowcolor
        fn SetShadowColor(&self, value: DOMString) {
            self.base.SetShadowColor(value)
        }

        // https://html.spec.whatwg.org/multipage/#dom-context-2d-save
        fn Save(&self) {
            self.base.Save()
        }

        #[allow(unrooted_must_root)]
        // https://html.spec.whatwg.org/multipage/#dom-context-2d-restore
        fn Restore(&self) {
            self.base.Restore()
        }

        // https://html.spec.whatwg.org/multipage/#dom-context-2d-scale
        fn Scale(&self, x: f64, y: f64) {
            self.base.Scale(x, y)
        }

        // https://html.spec.whatwg.org/multipage/#dom-context-2d-rotate
        fn Rotate(&self, angle: f64) {
            self.base.Rotate(angle)
        }

        // https://html.spec.whatwg.org/multipage/#dom-context-2d-translate
        fn Translate(&self, x: f64, y: f64) {
            self.base.Translate(x, y)
        }

        // https://html.spec.whatwg.org/multipage/#dom-context-2d-transform
        fn Transform(&self, a: f64, b: f64, c: f64, d: f64, e: f64, f: f64) {
            self.base.Transform(a, b, c, d, e, f)
        }

        // https://html.spec.whatwg.org/multipage/#dom-context-2d-settransform
        fn SetTransform(&self, a: f64, b: f64, c: f64, d: f64, e: f64, f: f64) {
            self.base.SetTransform(a, b, c, d, e, f)
        }

        // https://html.spec.whatwg.org/multipage/#dom-context-2d-resettransform
        fn ResetTransform(&self) {
            self.base.ResetTransform()
        }

        // https://html.spec.whatwg.org/multipage/#dom-context-2d-globalalpha
        fn GlobalAlpha(&self) -> f64 {
            self.base.GlobalAlpha()
        }

        // https://html.spec.whatwg.org/multipage/#dom-context-2d-globalalpha
        fn SetGlobalAlpha(&self, alpha: f64) {
            self.base.SetGlobalAlpha(alpha)
        }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-strokestyle
    fn SetFillStyle(&self, value: StringOrCanvasGradientOrCanvasPattern) {
        self.base.SetFillStyle(value)
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-ispointinpath
    fn IsPointInPath(&self, x: f64, y: f64, fill_rule: CanvasFillRule) -> bool {
        self.base.IsPointInPath(x, y, fill_rule)
    }
}

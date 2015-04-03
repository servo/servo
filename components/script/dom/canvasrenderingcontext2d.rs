/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::CanvasRenderingContext2DBinding;
use dom::bindings::codegen::Bindings::CanvasRenderingContext2DBinding::CanvasRenderingContext2DMethods;
use dom::bindings::codegen::Bindings::CanvasRenderingContext2DBinding::CanvasWindingRule;
use dom::bindings::codegen::Bindings::ImageDataBinding::ImageDataMethods;
use dom::bindings::codegen::UnionTypes::HTMLImageElementOrHTMLCanvasElementOrCanvasRenderingContext2D;
use dom::bindings::codegen::UnionTypes::StringOrCanvasGradientOrCanvasPattern;
use dom::bindings::error::Error::{IndexSize, NotSupported, Type, InvalidState};
use dom::bindings::error::Fallible;
use dom::bindings::global::{GlobalRef, GlobalField};
use dom::bindings::js::{JS, JSRef, LayoutJS, Temporary};
use dom::bindings::num::Finite;
use dom::bindings::utils::{Reflector, reflect_dom_object};
use dom::canvasgradient::{CanvasGradient, CanvasGradientStyle, ToFillOrStrokeStyle};
use dom::htmlcanvaselement::{HTMLCanvasElement, HTMLCanvasElementHelpers};
use dom::htmlimageelement::{HTMLImageElement, HTMLImageElementHelpers};
use dom::imagedata::{ImageData, ImageDataHelpers};
use dom::node::{window_from_node};

use cssparser::Color as CSSColor;
use cssparser::{Parser, RGBA, ToCss};
use geom::matrix2d::Matrix2D;
use geom::point::Point2D;
use geom::rect::Rect;
use geom::size::Size2D;

use canvas::canvas_msg::{CanvasMsg, Canvas2dMsg, CanvasCommonMsg};
use canvas::canvas_paint_task::{CanvasPaintTask, FillOrStrokeStyle};
use canvas::canvas_paint_task::{LinearGradientStyle, RadialGradientStyle};
use canvas::canvas_paint_task::{LineCapStyle, LineJoinStyle};

use net_traits::image::base::Image;
use net_traits::image_cache_task::{ImageResponseMsg, Msg};
use png::PixelsByColorType;

use std::borrow::ToOwned;
use std::cell::Cell;
use std::num::{Float, ToPrimitive};
use std::sync::{Arc};
use std::sync::mpsc::{channel, Sender};

use util::str::DOMString;
use url::Url;
use util::vec::byte_swap;

// https://html.spec.whatwg.org/multipage/#canvasrenderingcontext2d
#[dom_struct]
pub struct CanvasRenderingContext2D {
    reflector_: Reflector,
    global: GlobalField,
    renderer: Sender<CanvasMsg>,
    canvas: JS<HTMLCanvasElement>,
    global_alpha: Cell<f64>,
    image_smoothing_enabled: Cell<bool>,
    stroke_color: Cell<RGBA>,
    line_width: Cell<f64>,
    line_cap: Cell<LineCapStyle>,
    line_join: Cell<LineJoinStyle>,
    miter_limit: Cell<f64>,
    fill_color: Cell<RGBA>,
    transform: Cell<Matrix2D<f32>>,
}

impl CanvasRenderingContext2D {
    fn new_inherited(global: GlobalRef, canvas: JSRef<HTMLCanvasElement>, size: Size2D<i32>)
                     -> CanvasRenderingContext2D {
        let black = RGBA {
            red: 0.0,
            green: 0.0,
            blue: 0.0,
            alpha: 1.0,
        };
        CanvasRenderingContext2D {
            reflector_: Reflector::new(),
            global: GlobalField::from_rooted(&global),
            renderer: CanvasPaintTask::start(size),
            canvas: JS::from_rooted(canvas),
            global_alpha: Cell::new(1.0),
            image_smoothing_enabled: Cell::new(true),
            stroke_color: Cell::new(black),
            line_width: Cell::new(1.0),
            line_cap: Cell::new(LineCapStyle::Butt),
            line_join: Cell::new(LineJoinStyle::Miter),
            miter_limit: Cell::new(10.0),
            fill_color: Cell::new(black),
            transform: Cell::new(Matrix2D::identity()),
        }
    }

    pub fn new(global: GlobalRef, canvas: JSRef<HTMLCanvasElement>, size: Size2D<i32>)
               -> Temporary<CanvasRenderingContext2D> {
        reflect_dom_object(box CanvasRenderingContext2D::new_inherited(global, canvas, size),
                           global, CanvasRenderingContext2DBinding::Wrap)
    }

    pub fn recreate(&self, size: Size2D<i32>) {
        self.renderer.send(CanvasMsg::Common(CanvasCommonMsg::Recreate(size))).unwrap();
    }

    fn update_transform(&self) {
        self.renderer.send(CanvasMsg::Canvas2d(Canvas2dMsg::SetTransform(self.transform.get()))).unwrap()
    }

    // It is used by DrawImage to calculate the size of the source and destination rectangles based
    // on the drawImage call arguments
    // source rectangle = area of the original image to be copied
    // destination rectangle = area of the destination canvas where the source image is going to be drawn
    fn adjust_source_dest_rects(&self,
                  image_size: Size2D<f64>,
                  sx: f64, sy: f64, sw: f64, sh: f64,
                  dx: f64, dy: f64, dw: f64, dh: f64) -> (Rect<f64>, Rect<f64>) {
        let image_rect = Rect(Point2D(0f64, 0f64),
                              Size2D(image_size.width as f64, image_size.height as f64));

        // The source rectangle is the rectangle whose corners are the four points (sx, sy),
        // (sx+sw, sy), (sx+sw, sy+sh), (sx, sy+sh).
        let source_rect = Rect(Point2D(sx, sy),
                               Size2D(sw, sh));

        // When the source rectangle is outside the source image,
        // the source rectangle must be clipped to the source image
        let source_rect_clipped = source_rect.intersection(&image_rect).unwrap_or(Rect::zero());

        // Width and height ratios between the non clipped and clipped source rectangles
        let width_ratio: f64 = source_rect_clipped.size.width / source_rect.size.width;
        let height_ratio: f64 = source_rect_clipped.size.height / source_rect.size.height;

        // When the source rectangle is outside the source image,
        // the destination rectangle must be clipped in the same proportion.
        let dest_rect_width_scaled: f64 = dw * width_ratio;
        let dest_rect_height_scaled: f64 = dh * height_ratio;

        // The destination rectangle is the rectangle whose corners are the four points (dx, dy),
        // (dx+dw, dy), (dx+dw, dy+dh), (dx, dy+dh).
        let dest_rect = Rect(Point2D(dx, dy),
                             Size2D(dest_rect_width_scaled, dest_rect_height_scaled));

        let source_rect = Rect(Point2D(source_rect_clipped.origin.x,
                                     source_rect_clipped.origin.y),
                             Size2D(source_rect_clipped.size.width,
                                    source_rect_clipped.size.height));

        return (source_rect, dest_rect)
    }

    //
    // drawImage coordinates explained
    //
    //  Source Image      Destination Canvas
    // +-------------+     +-------------+
    // |             |     |             |
    // |(sx,sy)      |     |(dx,dy)      |
    // |   +----+    |     |   +----+    |
    // |   |    |    |     |   |    |    |
    // |   |    |sh  |---->|   |    |dh  |
    // |   |    |    |     |   |    |    |
    // |   +----+    |     |   +----+    |
    // |     sw      |     |     dw      |
    // |             |     |             |
    // +-------------+     +-------------+
    //
    //
    // The rectangle (sx, sy, sw, sh) from the source image
    // is copied on the rectangle (dx, dy, dh, dw) of the destination canvas
    //
    // https://html.spec.whatwg.org/multipage/#dom-context-2d-drawimage
    fn draw_html_canvas_element(&self,
                  canvas: JSRef<HTMLCanvasElement>,
                  sx: f64, sy: f64, sw: f64, sh: f64,
                  dx: f64, dy: f64, dw: f64, dh: f64) -> Fallible<()> {
        // 1. Check the usability of the image argument
        if !canvas.is_valid() {
            return Err(InvalidState)
        }

        let canvas_size = canvas.get_size();
        let image_size = Size2D(canvas_size.width as f64, canvas_size.height as f64);
        // 2. Establish the source and destination rectangles
        let (source_rect, dest_rect) = self.adjust_source_dest_rects(image_size, sx, sy, sw, sh, dx, dy, dw, dh);

        if !is_rect_valid(source_rect) || !is_rect_valid(dest_rect) {
            return Err(IndexSize)
        }

        let smoothing_enabled = self.image_smoothing_enabled.get();

        // If the source and target canvas are the same
        let msg = if self.canvas.root().r() == canvas {
            CanvasMsg::Canvas2d(Canvas2dMsg::DrawImageSelf(image_size, dest_rect, source_rect, smoothing_enabled))
        } else { // Source and target canvases are different
            let context = canvas.get_2d_context().root();
            let renderer = context.r().get_renderer();
            let (sender, receiver) = channel::<Vec<u8>>();
            // Reads pixels from source image
            renderer.send(CanvasMsg::Canvas2d(Canvas2dMsg::GetImageData(source_rect, image_size, sender))).unwrap();
            let imagedata = receiver.recv().unwrap();
            // Writes pixels to destination canvas
            CanvasMsg::Canvas2d(Canvas2dMsg::DrawImage(imagedata, source_rect.size, dest_rect, source_rect, smoothing_enabled))
        };

        self.renderer.send(msg).unwrap();
        Ok(())
    }

    fn draw_image_data(&self,
                       image_data: Vec<u8>,
                       image_size: Size2D<f64>,
                       sx: f64, sy: f64, sw: f64, sh: f64,
                       dx: f64, dy: f64, dw: f64, dh: f64) -> Fallible<()> {
        // Establish the source and destination rectangles
        let (source_rect, dest_rect) = self.adjust_source_dest_rects(image_size, sx, sy, sw, sh, dx, dy, dw, dh);

        if !is_rect_valid(source_rect) || !is_rect_valid(dest_rect) {
            return Err(IndexSize)
        }

        let smoothing_enabled = self.image_smoothing_enabled.get();
        self.renderer.send(CanvasMsg::Canvas2d(Canvas2dMsg::DrawImage(
                           image_data, image_size, dest_rect,
                           source_rect, smoothing_enabled))).unwrap();
        Ok(())
    }

    fn fetch_image_data(&self,
                        image_element: &JSRef<HTMLImageElement>)
                        -> Option<(Vec<u8>, Size2D<f64>)> {
        let url = match image_element.get_url() {
            Some(url) => url,
            None => return None,
        };

        let img = match self.request_image_from_cache(url) {
            Some(img) => img,
            None => return None,
        };

        let image_size = Size2D(img.width as f64, img.height as f64);
        let mut image_data = match img.pixels {
            PixelsByColorType::RGBA8(ref pixels) => pixels.to_vec(),
            PixelsByColorType::K8(_) => panic!("K8 color type not supported"),
            PixelsByColorType::RGB8(_) => panic!("RGB8 color type not supported"),
            PixelsByColorType::KA8(_) => panic!("KA8 color type not supported"),
        };
        // Pixels come from cache in BGRA order and drawImage expects RGBA so we
        // have to swap the color values
        {
            let mut pixel_colors = image_data.as_mut_slice();
            byte_swap(pixel_colors);
        }
        return Some((image_data, image_size));
    }

    fn request_image_from_cache(&self, url: Url) -> Option<Arc<Image>> {
        let canvas = self.canvas.root();
        let window = window_from_node(canvas.r()).root();
        let window = window.r();
        let image_cache_task = window.image_cache_task().clone();
        image_cache_task.send(Msg::Prefetch(url.clone()));
        image_cache_task.send(Msg::Decode(url.clone()));
        let (response_chan, response_port) = channel();
        image_cache_task.send(Msg::WaitForImage(url, response_chan));
        match response_port.recv().unwrap() {
           ImageResponseMsg::ImageReady(image) => Some(image),
           ImageResponseMsg::ImageFailed => None,
           _ => panic!("Image Cache: Unknown Result")
         }
    }

    fn create_drawable_rect(&self, x: f64, y: f64, w: f64, h: f64) -> Option<Rect<f32>> {
        if !([x, y, w, h].iter().all(|val| val.is_finite())) {
            return None;
        }

        if w == 0.0 && h == 0.0 {
            return None;
        }

        Some(Rect(Point2D(x as f32, y as f32), Size2D(w as f32, h as f32)))
    }
}

pub trait CanvasRenderingContext2DHelpers {
    fn get_renderer(&self) -> Sender<CanvasMsg>;
}

impl CanvasRenderingContext2DHelpers for CanvasRenderingContext2D {
    fn get_renderer(&self) -> Sender<CanvasMsg> {
        self.renderer.clone()
    }
}

pub trait LayoutCanvasRenderingContext2DHelpers {
    #[allow(unsafe_code)]
    unsafe fn get_renderer(&self) -> Sender<CanvasMsg>;
}

impl LayoutCanvasRenderingContext2DHelpers for LayoutJS<CanvasRenderingContext2D> {
    #[allow(unsafe_code)]
    unsafe fn get_renderer(&self) -> Sender<CanvasMsg> {
        (*self.unsafe_get()).renderer.clone()
    }
}

// We add a guard to each of methods by the spec:
// http://www.w3.org/html/wg/drafts/2dcontext/html5_canvas_CR/
//
// > Except where otherwise specified, for the 2D context interface,
// > any method call with a numeric argument whose value is infinite or a NaN value must be ignored.
//
//  Restricted values are guarded in glue code. Therefore we need not add a guard.
//
// FIXME: this behavior should might be generated by some annotattions to idl.
impl<'a> CanvasRenderingContext2DMethods for JSRef<'a, CanvasRenderingContext2D> {
    // https://html.spec.whatwg.org/multipage/#dom-context-2d-canvas
    fn Canvas(self) -> Temporary<HTMLCanvasElement> {
        Temporary::new(self.canvas)
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-scale
    fn Scale(self, x: f64, y: f64) {
        if !(x.is_finite() && y.is_finite()) {
            return;
        }

        self.transform.set(self.transform.get().scale(x as f32, y as f32));
        self.update_transform()
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-translate
    fn Translate(self, x: f64, y: f64) {
        if !(x.is_finite() && y.is_finite()) {
            return;
        }

        self.transform.set(self.transform.get().translate(x as f32, y as f32));
        self.update_transform()
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-transform
    fn Transform(self, a: f64, b: f64, c: f64, d: f64, e: f64, f: f64) {
        if !(a.is_finite() && b.is_finite() && c.is_finite() &&
             d.is_finite() && e.is_finite() && f.is_finite()) {
            return;
        }

        self.transform.set(self.transform.get().mul(&Matrix2D::new(a as f32,
                                                                   b as f32,
                                                                   c as f32,
                                                                   d as f32,
                                                                   e as f32,
                                                                   f as f32)));
        self.update_transform()
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-settransform
    fn SetTransform(self, a: f64, b: f64, c: f64, d: f64, e: f64, f: f64) {
        if !(a.is_finite() && b.is_finite() && c.is_finite() &&
             d.is_finite() && e.is_finite() && f.is_finite()) {
            return;
        }

        self.transform.set(Matrix2D::new(a as f32,
                                         b as f32,
                                         c as f32,
                                         d as f32,
                                         e as f32,
                                         f as f32));
        self.update_transform()
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-globalalpha
    fn GlobalAlpha(self) -> f64 {
        self.global_alpha.get()
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-globalalpha
    fn SetGlobalAlpha(self, alpha: f64) {
        if !alpha.is_finite() || alpha > 1.0 || alpha < 0.0 {
            return;
        }

        self.global_alpha.set(alpha);
        self.renderer.send(CanvasMsg::Canvas2d(Canvas2dMsg::SetGlobalAlpha(alpha as f32))).unwrap()
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-fillrect
    fn FillRect(self, x: f64, y: f64, width: f64, height: f64) {
        if let Some(rect) = self.create_drawable_rect(x, y, width, height) {
            self.renderer.send(CanvasMsg::Canvas2d(Canvas2dMsg::FillRect(rect))).unwrap();
        }
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-clearrect
    fn ClearRect(self, x: f64, y: f64, width: f64, height: f64) {
        if let Some(rect) = self.create_drawable_rect(x, y, width, height) {
            self.renderer.send(CanvasMsg::Canvas2d(Canvas2dMsg::ClearRect(rect))).unwrap();
        }
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-strokerect
    fn StrokeRect(self, x: f64, y: f64, width: f64, height: f64) {
        if let Some(rect) = self.create_drawable_rect(x, y, width, height) {
            self.renderer.send(CanvasMsg::Canvas2d(Canvas2dMsg::StrokeRect(rect))).unwrap();
        }
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-beginpath
    fn BeginPath(self) {
        self.renderer.send(CanvasMsg::Canvas2d(Canvas2dMsg::BeginPath)).unwrap();
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-closepath
    fn ClosePath(self) {
        self.renderer.send(CanvasMsg::Canvas2d(Canvas2dMsg::ClosePath)).unwrap();
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-fill
    fn Fill(self, _: CanvasWindingRule) {
        self.renderer.send(CanvasMsg::Canvas2d(Canvas2dMsg::Fill)).unwrap();
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-stroke
    fn Stroke(self) {
        self.renderer.send(CanvasMsg::Canvas2d(Canvas2dMsg::Stroke)).unwrap();
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-drawimage
    fn DrawImage(self, image: HTMLImageElementOrHTMLCanvasElementOrCanvasRenderingContext2D,
                 dx: f64, dy: f64) -> Fallible<()> {
        if !(dx.is_finite() && dy.is_finite()) {
            return Ok(());
        }

        // From rules described in the spec:
        // If the sx, sy, sw, and sh arguments are omitted, they must default to 0, 0,
        // the image's intrinsic width in image pixels,
        // and the image's intrinsic height in image pixels, respectively
        let sx: f64 = 0f64;
        let sy: f64 = 0f64;

        match image {
            HTMLImageElementOrHTMLCanvasElementOrCanvasRenderingContext2D::eHTMLCanvasElement(image) => {
                let canvas = image.root();
                let canvas_size = canvas.r().get_size();
                let dw: f64 = canvas_size.width as f64;
                let dh: f64 = canvas_size.height as f64;
                let sw: f64 = dw;
                let sh: f64 = dh;
                return self.draw_html_canvas_element(canvas.r(),
                                                     sx, sy, sw, sh,
                                                     dx, dy, dw, dh)
            }
            HTMLImageElementOrHTMLCanvasElementOrCanvasRenderingContext2D::eCanvasRenderingContext2D(image) => {
                let image = image.root();
                let context = image.r();
                let canvas = context.Canvas().root();
                let canvas_size = canvas.r().get_size();
                let dw: f64 = canvas_size.width as f64;
                let dh: f64 = canvas_size.height as f64;
                let sw: f64 = dw;
                let sh: f64 = dh;
                return self.draw_html_canvas_element(canvas.r(),
                                                     sx, sy, sw, sh,
                                                     dx, dy, dw, dh)
            }
            HTMLImageElementOrHTMLCanvasElementOrCanvasRenderingContext2D::eHTMLImageElement(image) => {
                let image = image.root();
                let image_element = image.r();
                // https://html.spec.whatwg.org/multipage/#img-error
                // If the image argument is an HTMLImageElement object that is in the broken state,
                // then throw an InvalidStateError exception
                let (image_data, image_size) = match self.fetch_image_data(&image_element) {
                    Some((data, size)) => (data, size),
                    None => return Err(InvalidState),
                };
                let dw: f64 = image_size.width as f64;
                let dh: f64 = image_size.height as f64;
                let sw: f64 = dw;
                let sh: f64 = dh;
                return self.draw_image_data(image_data,
                                            image_size,
                                            sx, sy, sw, sh,
                                            dx, dy, dw, dh)
            }

        }
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-drawimage
    fn DrawImage_(self, image: HTMLImageElementOrHTMLCanvasElementOrCanvasRenderingContext2D,
                  dx: f64, dy: f64, dw: f64, dh: f64) -> Fallible<()> {
        if !(dx.is_finite() && dy.is_finite() &&
             dw.is_finite() && dh.is_finite()) {
            return Ok(());
        }

        // From rules described in the spec:
        // If the sx, sy, sw, and sh arguments are omitted, they must default to 0, 0,
        // the image's intrinsic width in image pixels,
        // and the image's intrinsic height in image pixels, respectively
        let sx: f64 = 0f64;
        let sy: f64 = 0f64;

        match image {
            HTMLImageElementOrHTMLCanvasElementOrCanvasRenderingContext2D::eHTMLCanvasElement(image) => {
                let canvas = image.root();
                let canvas_size = canvas.r().get_size();
                let sw: f64 = canvas_size.width as f64;
                let sh: f64 = canvas_size.height as f64;
                return self.draw_html_canvas_element(canvas.r(),
                                                     sx, sy, sw, sh,
                                                     dx, dy, dw, dh)
            }
            HTMLImageElementOrHTMLCanvasElementOrCanvasRenderingContext2D::eCanvasRenderingContext2D(image) => {
                let image = image.root();
                let context = image.r();
                let canvas = context.Canvas().root();
                let canvas_size = canvas.r().get_size();
                let sw: f64 = canvas_size.width as f64;
                let sh: f64 = canvas_size.height as f64;
                return self.draw_html_canvas_element(canvas.r(),
                                                     sx, sy, sw, sh,
                                                     dx, dy, dw, dh)
            }
            HTMLImageElementOrHTMLCanvasElementOrCanvasRenderingContext2D::eHTMLImageElement(image) => {
                let image = image.root();
                let image_element = image.r();
                // https://html.spec.whatwg.org/multipage/#img-error
                // If the image argument is an HTMLImageElement object that is in the broken state,
                // then throw an InvalidStateError exception
                let (image_data, image_size) = match self.fetch_image_data(&image_element) {
                    Some((data, size)) => (data, size),
                    None => return Err(InvalidState),
                };
                let sw: f64 = image_size.width as f64;
                let sh: f64 = image_size.height as f64;
                return self.draw_image_data(image_data,
                                            image_size,
                                            sx, sy, sw, sh,
                                            dx, dy, dw, dh)
            }
        }
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-drawimage
    fn DrawImage__(self, image: HTMLImageElementOrHTMLCanvasElementOrCanvasRenderingContext2D,
                         sx: f64, sy: f64, sw: f64, sh: f64,
                         dx: f64, dy: f64, dw: f64, dh: f64) -> Fallible<()> {
        if !(sx.is_finite() && sy.is_finite() && sw.is_finite() && sh.is_finite() &&
             dx.is_finite() && dy.is_finite() && dw.is_finite() && dh.is_finite()) {
            return Ok(());
        }

        match image {
            HTMLImageElementOrHTMLCanvasElementOrCanvasRenderingContext2D::eHTMLCanvasElement(image) => {
                let canvas = image.root();
                return self.draw_html_canvas_element(canvas.r(),
                                                     sx, sy, sw, sh,
                                                     dx, dy, dw, dh)
            }
            HTMLImageElementOrHTMLCanvasElementOrCanvasRenderingContext2D::eCanvasRenderingContext2D(image) => {
                let image = image.root();
                let context = image.r();
                let canvas = context.Canvas().root();
                return self.draw_html_canvas_element(canvas.r(),
                                                     sx, sy, sw, sh,
                                                     dx, dy, dw, dh)
            }
            HTMLImageElementOrHTMLCanvasElementOrCanvasRenderingContext2D::eHTMLImageElement(image) => {
                let image = image.root();
                let image_element = image.r();
                // https://html.spec.whatwg.org/multipage/#img-error
                // If the image argument is an HTMLImageElement object that is in the broken state,
                // then throw an InvalidStateError exception
                let (image_data, image_size) = match self.fetch_image_data(&image_element) {
                    Some((data, size)) => (data, size),
                    None => return Err(InvalidState),
                };
                return self.draw_image_data(image_data,
                                            image_size,
                                            sx, sy, sw, sh,
                                            dx, dy, dw, dh)
            }
        }
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-moveto
    fn MoveTo(self, x: f64, y: f64) {
        if !(x.is_finite() && y.is_finite()) {
            return;
        }

        self.renderer.send(CanvasMsg::Canvas2d(Canvas2dMsg::MoveTo(Point2D(x as f32, y as f32)))).unwrap();
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-lineto
    fn LineTo(self, x: f64, y: f64) {
        if !(x.is_finite() && y.is_finite()) {
            return;
        }

        self.renderer.send(CanvasMsg::Canvas2d(Canvas2dMsg::LineTo(Point2D(x as f32, y as f32)))).unwrap();
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-quadraticcurveto
    fn QuadraticCurveTo(self, cpx: f64, cpy: f64, x: f64, y: f64) {
        if !(cpx.is_finite() && cpy.is_finite() &&
             x.is_finite() && y.is_finite()) {
            return;
        }

        self.renderer.send(CanvasMsg::Canvas2d(Canvas2dMsg::QuadraticCurveTo(Point2D(cpx as f32, cpy as f32),
                                                            Point2D(x as f32, y as f32)))).unwrap();
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-beziercurveto
    fn BezierCurveTo(self, cp1x: f64, cp1y: f64, cp2x: f64, cp2y: f64, x: f64, y: f64) {
        if !(cp1x.is_finite() && cp1y.is_finite() && cp2x.is_finite() && cp2y.is_finite() &&
             x.is_finite() && y.is_finite()) {
            return;
        }

        self.renderer.send(CanvasMsg::Canvas2d(Canvas2dMsg::BezierCurveTo(Point2D(cp1x as f32, cp1y as f32),
                                                            Point2D(cp2x as f32, cp2y as f32),
                                                            Point2D(x as f32, y as f32)))).unwrap();
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-arc
    fn Arc(self, x: Finite<f64>, y: Finite<f64>, r: Finite<f64>,
           start: Finite<f64>, end: Finite<f64>, ccw: bool) -> Fallible<()> {
        let x = *x;
        let y = *y;
        let r = *r;
        let start = *start;
        let end = *end;

        if r < 0.0 {
            return Err(IndexSize);
        }

        self.renderer.send(CanvasMsg::Canvas2d(Canvas2dMsg::Arc(Point2D(x as f32, y as f32), r as f32,
                                                                start as f32, end as f32, ccw))).unwrap();
        Ok(())
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-arcto
    fn ArcTo(self, cp1x: f64, cp1y: f64, cp2x: f64, cp2y: f64, r: f64) -> Fallible<()> {
        if !([cp1x, cp1y, cp2x, cp2y, r].iter().all(|x| x.is_finite())) {
            return Ok(());
        }
        if r < 0.0 {
            return Err(IndexSize);
        }
        self.renderer.send(CanvasMsg::Canvas2d(Canvas2dMsg::ArcTo(Point2D(cp1x as f32, cp1y as f32),
                                                                  Point2D(cp2x as f32, cp2y as f32),
                                                                  r as f32))).unwrap();
        Ok(())
    }

    // https://html.spec.whatwg.org/#dom-context-2d-imagesmoothingenabled
    fn ImageSmoothingEnabled(self) -> bool {
        self.image_smoothing_enabled.get()
    }

    // https://html.spec.whatwg.org/#dom-context-2d-imagesmoothingenabled
    fn SetImageSmoothingEnabled(self, value: bool) -> () {
        self.image_smoothing_enabled.set(value);
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-strokestyle
    fn StrokeStyle(self) -> StringOrCanvasGradientOrCanvasPattern {
        // FIXME(pcwalton, #4761): This is not spec-compliant. See:
        //
        // https://html.spec.whatwg.org/multipage/#serialisation-of-a-colour
        let mut result = String::new();
        self.stroke_color.get().to_css(&mut result).unwrap();
        StringOrCanvasGradientOrCanvasPattern::eString(result)
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-strokestyle
    fn SetStrokeStyle(self, value: StringOrCanvasGradientOrCanvasPattern) {
        match value {
            StringOrCanvasGradientOrCanvasPattern::eString(string) => {
                match parse_color(&string) {
                    Ok(rgba) => {
                        self.stroke_color.set(rgba);
                        self.renderer
                            .send(CanvasMsg::Canvas2d(Canvas2dMsg::SetStrokeStyle(FillOrStrokeStyle::Color(rgba))))
                            .unwrap();
                    }
                    _ => {}
                }
            }
            _ => {
                // TODO(pcwalton)
            }
        }
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-strokestyle
    fn FillStyle(self) -> StringOrCanvasGradientOrCanvasPattern {
        // FIXME(pcwalton, #4761): This is not spec-compliant. See:
        //
        // https://html.spec.whatwg.org/multipage/#serialisation-of-a-colour
        let mut result = String::new();
        self.stroke_color.get().to_css(&mut result).unwrap();
        StringOrCanvasGradientOrCanvasPattern::eString(result)
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-strokestyle
    fn SetFillStyle(self, value: StringOrCanvasGradientOrCanvasPattern) {
        match value {
            StringOrCanvasGradientOrCanvasPattern::eString(string) => {
                match parse_color(&string) {
                    Ok(rgba) => {
                        self.fill_color.set(rgba);
                        self.renderer
                            .send(CanvasMsg::Canvas2d(Canvas2dMsg::SetFillStyle(FillOrStrokeStyle::Color(rgba))))
                            .unwrap()
                    }
                    _ => {}
                }
            }
            StringOrCanvasGradientOrCanvasPattern::eCanvasGradient(gradient) => {
                self.renderer.send(CanvasMsg::Canvas2d(Canvas2dMsg::SetFillStyle(gradient.root().r().to_fill_or_stroke_style()))).unwrap();
            }
            _ => {}
        }
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-createimagedata
    fn CreateImageData(self, sw: f64, sh: f64) -> Fallible<Temporary<ImageData>> {
        if !(sw.is_finite() && sh.is_finite()) {
            return Err(NotSupported);
        }

        if sw == 0.0 || sh == 0.0 {
            return Err(IndexSize)
        }

        Ok(ImageData::new(self.global.root().r(), sw.abs().to_u32().unwrap(), sh.abs().to_u32().unwrap(), None))
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-createimagedata
    fn CreateImageData_(self, imagedata: JSRef<ImageData>) -> Fallible<Temporary<ImageData>> {
        Ok(ImageData::new(self.global.root().r(), imagedata.Width(), imagedata.Height(), None))
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-getimagedata
    fn GetImageData(self, sx: Finite<f64>, sy: Finite<f64>, sw: Finite<f64>, sh: Finite<f64>) -> Fallible<Temporary<ImageData>> {
        let sx = *sx;
        let sy = *sy;
        let sw = *sw;
        let sh = *sh;

        if sw == 0.0 || sh == 0.0 {
            return Err(IndexSize)
        }

        let (sender, receiver) = channel::<Vec<u8>>();
        let dest_rect = Rect(Point2D(sx as f64, sy as f64), Size2D(sw as f64, sh as f64));
        let canvas_size = self.canvas.root().r().get_size();
        let canvas_size = Size2D(canvas_size.width as f64, canvas_size.height as f64);
        self.renderer.send(CanvasMsg::Canvas2d(Canvas2dMsg::GetImageData(dest_rect, canvas_size, sender))).unwrap();
        let data = receiver.recv().unwrap();
        Ok(ImageData::new(self.global.root().r(), sw.abs().to_u32().unwrap(), sh.abs().to_u32().unwrap(), Some(data)))
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-putimagedata
    fn PutImageData(self, imagedata: JSRef<ImageData>, dx: Finite<f64>, dy: Finite<f64>) {
        let dx = *dx;
        let dy = *dy;

        // XXX:
        // By the spec: http://www.w3.org/html/wg/drafts/2dcontext/html5_canvas_CR/#dom-context-2d-putimagedata
        // "If any of the arguments to the method are infinite or NaN, the method must throw a NotSupportedError exception"
        // But this arguments are stricted value, so if they are not finite values,
        // they will be TypeError by WebIDL spec before call this methods.

        let data = imagedata.get_data_array(&self.global.root().r());
        let image_data_size = imagedata.get_size();
        let image_data_size = Size2D(image_data_size.width as f64, image_data_size.height as f64);
        let image_data_rect = Rect(Point2D(dx, dy), image_data_size);
        let dirty_rect = None;
        self.renderer.send(CanvasMsg::Canvas2d(Canvas2dMsg::PutImageData(data, image_data_rect, dirty_rect))).unwrap()
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-putimagedata
    fn PutImageData_(self, imagedata: JSRef<ImageData>, dx: Finite<f64>, dy: Finite<f64>,
                     dirtyX: Finite<f64>, dirtyY: Finite<f64>, dirtyWidth: Finite<f64>, dirtyHeight: Finite<f64>) {
        let dx = *dx;
        let dy = *dy;
        let dirtyX = *dirtyX;
        let dirtyY = *dirtyY;
        let dirtyWidth = *dirtyWidth;
        let dirtyHeight = *dirtyHeight;

        // XXX:
        // By the spec: http://www.w3.org/html/wg/drafts/2dcontext/html5_canvas_CR/#dom-context-2d-putimagedata
        // "If any of the arguments to the method are infinite or NaN, the method must throw a NotSupportedError exception"
        // But this arguments are stricted value, so if they are not finite values,
        // they will be TypeError by WebIDL spec before call this methods.

        let data = imagedata.get_data_array(&self.global.root().r());
        let image_data_rect = Rect(Point2D(dx, dy),
                                   Size2D(imagedata.Width() as f64,
                                          imagedata.Height() as f64));
        let dirty_rect = Some(Rect(Point2D(dirtyX, dirtyY),
                                   Size2D(dirtyWidth, dirtyHeight)));
        self.renderer.send(CanvasMsg::Canvas2d(Canvas2dMsg::PutImageData(data, image_data_rect, dirty_rect))).unwrap()
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-createlineargradient
    fn CreateLinearGradient(self, x0: Finite<f64>, y0: Finite<f64>,
                                  x1: Finite<f64>, y1: Finite<f64>) -> Fallible<Temporary<CanvasGradient>> {
        let x0 = *x0;
        let y0 = *y0;
        let x1 = *x1;
        let y1 = *y1;

        if [x0, y0, x1, y1].iter().any(|x| x.is_nan() || x.is_infinite()) {
            return Err(Type("One of the arguments of createLinearGradient() is not a finite floating-point value.".to_owned()));
        }
        Ok(CanvasGradient::new(self.global.root().r(),
                               CanvasGradientStyle::Linear(LinearGradientStyle::new(x0, y0, x1, y1, Vec::new()))))
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-createradialgradient
    fn CreateRadialGradient(self, x0: Finite<f64>, y0: Finite<f64>, r0: Finite<f64>,
                                  x1: Finite<f64>, y1: Finite<f64>, r1: Finite<f64>) -> Fallible<Temporary<CanvasGradient>> {
        let x0 = *x0;
        let y0 = *y0;
        let r0 = *r0;
        let x1 = *x1;
        let y1 = *y1;
        let r1 = *r1;

        if [x0, y0, r0, x1, y1, r1].iter().any(|x| x.is_nan() || x.is_infinite()) {
            return Err(Type("One of the arguments of createRadialGradient() is not a finite floating-point value.".to_owned()));
        }
        Ok(CanvasGradient::new(self.global.root().r(),
                               CanvasGradientStyle::Radial(RadialGradientStyle::new(x0, y0, r0, x1, y1, r1, Vec::new()))))
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-linewidth
    fn LineWidth(self) -> f64 {
        self.line_width.get()
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-linewidth
    fn SetLineWidth(self, width: f64) {
        if !width.is_finite() || width <= 0.0 {
            return;
        }

        self.line_width.set(width);
        self.renderer.send(CanvasMsg::Canvas2d(Canvas2dMsg::SetLineWidth(width as f32))).unwrap()
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-linecap
    fn LineCap(self) -> DOMString {
        match self.line_cap.get() {
            LineCapStyle::Butt => "butt".to_owned(),
            LineCapStyle::Round => "round".to_owned(),
            LineCapStyle::Square => "square".to_owned(),
        }
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-linecap
    fn SetLineCap(self, cap_str: DOMString) {
        if let Some(cap) = LineCapStyle::from_str(&cap_str) {
            self.line_cap.set(cap);
            self.renderer.send(CanvasMsg::Canvas2d(Canvas2dMsg::SetLineCap(cap))).unwrap()
        }
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-linejoin
    fn LineJoin(self) -> DOMString {
        match self.line_join.get() {
            LineJoinStyle::Round => "round".to_owned(),
            LineJoinStyle::Bevel => "bevel".to_owned(),
            LineJoinStyle::Miter => "miter".to_owned(),
        }
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-linejoin
    fn SetLineJoin(self, join_str: DOMString) {
        if let Some(join) = LineJoinStyle::from_str(&join_str) {
            self.line_join.set(join);
            self.renderer.send(CanvasMsg::Canvas2d(Canvas2dMsg::SetLineJoin(join))).unwrap()
        }
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-miterlimit
    fn MiterLimit(self) -> f64 {
        self.miter_limit.get()
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-miterlimit
    fn SetMiterLimit(self, limit: f64) {
        if !limit.is_finite() || limit <= 0.0 {
            return;
        }

        self.miter_limit.set(limit);
        self.renderer.send(CanvasMsg::Canvas2d(Canvas2dMsg::SetMiterLimit(limit as f32))).unwrap()
    }
}

#[unsafe_destructor]
impl Drop for CanvasRenderingContext2D {
    fn drop(&mut self) {
        self.renderer.send(CanvasMsg::Common(CanvasCommonMsg::Close)).unwrap();
    }
}

pub fn parse_color(string: &str) -> Result<RGBA,()> {
    match CSSColor::parse(&mut Parser::new(&string)) {
        Ok(CSSColor::RGBA(rgba)) => Ok(rgba),
        _ => Err(()),
    }
}

// Used by drawImage to determine if a source or destination rectangle is valid
// Origin coordinates and size cannot be negative. Size has to be greater than zero
fn is_rect_valid(rect: Rect<f64>) -> bool {
    rect.size.width > 0.0 && rect.size.height > 0.0
}

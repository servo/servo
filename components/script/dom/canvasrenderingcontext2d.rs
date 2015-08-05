/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::CanvasRenderingContext2DBinding;
use dom::bindings::codegen::Bindings::CanvasRenderingContext2DBinding::CanvasRenderingContext2DMethods;
use dom::bindings::codegen::Bindings::CanvasRenderingContext2DBinding::CanvasWindingRule;
use dom::bindings::codegen::Bindings::ImageDataBinding::ImageDataMethods;
use dom::bindings::codegen::InheritTypes::NodeCast;
use dom::bindings::codegen::UnionTypes::HTMLImageElementOrHTMLCanvasElementOrCanvasRenderingContext2D;
use dom::bindings::codegen::UnionTypes::StringOrCanvasGradientOrCanvasPattern;
use dom::bindings::error::Error::{IndexSize, InvalidState, Syntax};
use dom::bindings::error::Fallible;
use dom::bindings::global::{GlobalRef, GlobalField};
use dom::bindings::js::{JS, LayoutJS, Root};
use dom::bindings::num::Finite;
use dom::bindings::utils::{Reflector, reflect_dom_object};
use dom::canvasgradient::{CanvasGradient, CanvasGradientStyle, ToFillOrStrokeStyle};
use dom::canvaspattern::CanvasPattern;
use dom::htmlcanvaselement::{HTMLCanvasElement, HTMLCanvasElementHelpers};
use dom::htmlimageelement::{HTMLImageElement, HTMLImageElementHelpers};
use dom::imagedata::{ImageData, ImageDataHelpers};
use dom::node::{window_from_node, NodeHelpers, NodeDamage};

use cssparser::Color as CSSColor;
use cssparser::{Parser, RGBA};
use euclid::matrix2d::Matrix2D;
use euclid::point::Point2D;
use euclid::rect::Rect;
use euclid::size::Size2D;

use canvas_traits::{CanvasMsg, Canvas2dMsg, CanvasCommonMsg};
use canvas_traits::{FillOrStrokeStyle, LinearGradientStyle, RadialGradientStyle, RepetitionStyle};
use canvas_traits::{LineCapStyle, LineJoinStyle, CompositionOrBlending};
use canvas::canvas_paint_task::RectToi32;

use msg::constellation_msg::Msg as ConstellationMsg;
use net_traits::image_cache_task::{ImageCacheChan, ImageResponse};
use net_traits::image::base::PixelFormat;

use ipc_channel::ipc::{self, IpcSender};
use num::{Float, ToPrimitive};
use std::borrow::ToOwned;
use std::cell::RefCell;
use std::fmt;
use std::cmp;
use std::sync::mpsc::channel;

use util::str::DOMString;
use url::Url;
use util::vec::byte_swap;

#[must_root]
#[derive(JSTraceable, Clone, HeapSizeOf)]
pub enum CanvasFillOrStrokeStyle {
    Color(RGBA),
    Gradient(JS<CanvasGradient>),
    // Pattern(JS<CanvasPattern>),  // https://github.com/servo/servo/pull/6157
}

// https://html.spec.whatwg.org/multipage/#canvasrenderingcontext2d
#[dom_struct]
#[derive(HeapSizeOf)]
pub struct CanvasRenderingContext2D {
    reflector_: Reflector,
    global: GlobalField,
    renderer_id: usize,
    #[ignore_heap_size_of = "Defined in ipc-channel"]
    ipc_renderer: IpcSender<CanvasMsg>,
    canvas: JS<HTMLCanvasElement>,
    state: RefCell<CanvasContextState>,
    saved_states: RefCell<Vec<CanvasContextState>>,
}

#[must_root]
#[derive(JSTraceable, Clone, HeapSizeOf)]
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
    transform: Matrix2D<f32>,
    shadow_offset_x: f64,
    shadow_offset_y: f64,
    shadow_blur: f64,
    shadow_color: RGBA,
}

impl CanvasContextState {
    fn new() -> CanvasContextState {
        let black = RGBA {
            red: 0.0,
            green: 0.0,
            blue: 0.0,
            alpha: 1.0,
        };
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
            transform: Matrix2D::identity(),
            shadow_offset_x: 0.0,
            shadow_offset_y: 0.0,
            shadow_blur: 0.0,
            shadow_color: RGBA { red: 0.0, green: 0.0, blue: 0.0, alpha: 0.0 }, // transparent black
        }
    }
}

impl CanvasRenderingContext2D {
    fn new_inherited(global: GlobalRef, canvas: &HTMLCanvasElement, size: Size2D<i32>)
                     -> CanvasRenderingContext2D {
        let (sender, receiver) = ipc::channel().unwrap();
        let constellation_chan = global.constellation_chan();
        constellation_chan.0.send(ConstellationMsg::CreateCanvasPaintTask(size, sender)).unwrap();
        let (ipc_renderer, renderer_id) = receiver.recv().unwrap();
        CanvasRenderingContext2D {
            reflector_: Reflector::new(),
            global: GlobalField::from_rooted(&global),
            renderer_id: renderer_id,
            ipc_renderer: ipc_renderer,
            canvas: JS::from_ref(canvas),
            state: RefCell::new(CanvasContextState::new()),
            saved_states: RefCell::new(Vec::new()),
        }
    }

    pub fn new(global: GlobalRef, canvas: &HTMLCanvasElement, size: Size2D<i32>)
               -> Root<CanvasRenderingContext2D> {
        reflect_dom_object(box CanvasRenderingContext2D::new_inherited(global, canvas, size),
                           global, CanvasRenderingContext2DBinding::Wrap)
    }

    pub fn recreate(&self, size: Size2D<i32>) {
        self.ipc_renderer
            .send(CanvasMsg::Common(CanvasCommonMsg::Recreate(size)))
            .unwrap();
    }

    fn mark_as_dirty(&self) {
        let canvas = self.canvas.root();
        let node = NodeCast::from_ref(canvas.r());
        node.dirty(NodeDamage::OtherNodeDamage);
    }

    fn update_transform(&self) {
        self.ipc_renderer
            .send(CanvasMsg::Canvas2d(Canvas2dMsg::SetTransform(self.state.borrow().transform)))
            .unwrap()
    }

    // It is used by DrawImage to calculate the size of the source and destination rectangles based
    // on the drawImage call arguments
    // source rectangle = area of the original image to be copied
    // destination rectangle = area of the destination canvas where the source image is going to be drawn
    fn adjust_source_dest_rects(&self,
                  image_size: Size2D<f64>,
                  sx: f64, sy: f64, sw: f64, sh: f64,
                  dx: f64, dy: f64, dw: f64, dh: f64) -> (Rect<f64>, Rect<f64>) {
        let image_rect = Rect::new(Point2D::new(0f64, 0f64),
                                   Size2D::new(image_size.width as f64, image_size.height as f64));

        // The source rectangle is the rectangle whose corners are the four points (sx, sy),
        // (sx+sw, sy), (sx+sw, sy+sh), (sx, sy+sh).
        let source_rect = Rect::new(Point2D::new(sx.min(sx+sw), sy.min(sy+sh)),
                                    Size2D::new(sw.abs(), sh.abs()));

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
        let dest_rect = Rect::new(Point2D::new(dx.min(dx+dest_rect_width_scaled), dy.min(dy+dest_rect_height_scaled)),
                                  Size2D::new(dest_rect_width_scaled.abs(), dest_rect_height_scaled.abs()));

        let source_rect = Rect::new(Point2D::new(source_rect_clipped.origin.x,
                                                 source_rect_clipped.origin.y),
                                    Size2D::new(source_rect_clipped.size.width,
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
    fn draw_image(&self, image: HTMLImageElementOrHTMLCanvasElementOrCanvasRenderingContext2D,
                  sx: f64, sy: f64, sw: Option<f64>, sh: Option<f64>,
                  dx: f64, dy: f64, dw: Option<f64>, dh: Option<f64>) -> Fallible<()> {
        match image {
            HTMLImageElementOrHTMLCanvasElementOrCanvasRenderingContext2D::eHTMLCanvasElement(canvas) =>
                self.draw_html_canvas_element(canvas.r(),
                                              sx, sy, sw, sh,
                                              dx, dy, dw, dh),
            HTMLImageElementOrHTMLCanvasElementOrCanvasRenderingContext2D::eCanvasRenderingContext2D(image) => {
                let context = image.r();
                let canvas = context.Canvas();
                self.draw_html_canvas_element(canvas.r(),
                                              sx, sy, sw, sh,
                                              dx, dy, dw, dh)
            }
            HTMLImageElementOrHTMLCanvasElementOrCanvasRenderingContext2D::eHTMLImageElement(image) => {
                let image_element = image.r();
                // https://html.spec.whatwg.org/multipage/#img-error
                // If the image argument is an HTMLImageElement object that is in the broken state,
                // then throw an InvalidStateError exception
                let (image_data, image_size) = match self.fetch_image_data(&image_element) {
                    Some((mut data, size)) => {
                        // Pixels come from cache in BGRA order and drawImage expects RGBA so we
                        // have to swap the color values
                        byte_swap(&mut data);
                        (data, size)
                    },
                    None => return Err(InvalidState),
                };
                let dw = dw.unwrap_or(image_size.width);
                let dh = dh.unwrap_or(image_size.height);
                let sw = sw.unwrap_or(image_size.width);
                let sh = sh.unwrap_or(image_size.height);
                self.draw_image_data(image_data,
                                     image_size,
                                     sx, sy, sw, sh,
                                     dx, dy, dw, dh)
            }
        }
    }

    fn draw_html_canvas_element(&self,
                  canvas: &HTMLCanvasElement,
                  sx: f64, sy: f64, sw: Option<f64>, sh: Option<f64>,
                  dx: f64, dy: f64, dw: Option<f64>, dh: Option<f64>) -> Fallible<()> {
        // 1. Check the usability of the image argument
        if !canvas.is_valid() {
            return Err(InvalidState)
        }

        let canvas_size = canvas.get_size();
        let dw = dw.unwrap_or(canvas_size.width as f64);
        let dh = dh.unwrap_or(canvas_size.height as f64);
        let sw = sw.unwrap_or(canvas_size.width as f64);
        let sh = sh.unwrap_or(canvas_size.height as f64);

        let image_size = Size2D::new(canvas_size.width as f64, canvas_size.height as f64);
        // 2. Establish the source and destination rectangles
        let (source_rect, dest_rect) = self.adjust_source_dest_rects(image_size, sx, sy, sw, sh, dx, dy, dw, dh);

        if !is_rect_valid(source_rect) || !is_rect_valid(dest_rect) {
            return Err(IndexSize)
        }

        let smoothing_enabled = self.state.borrow().image_smoothing_enabled;

        // If the source and target canvas are the same
        let msg = if self.canvas.root().r() == canvas {
            CanvasMsg::Canvas2d(Canvas2dMsg::DrawImageSelf(image_size, dest_rect, source_rect, smoothing_enabled))
        } else { // Source and target canvases are different
            let context = match canvas.get_or_init_2d_context() {
                Some(context) => context,
                None => return Err(InvalidState),
            };

            let renderer = context.r().get_ipc_renderer();
            let (sender, receiver) = ipc::channel::<Vec<u8>>().unwrap();
            // Reads pixels from source image
            renderer.send(CanvasMsg::Canvas2d(Canvas2dMsg::GetImageData(source_rect.to_i32(),
                                                                        image_size,
                                                                        sender))).unwrap();
            let imagedata = receiver.recv().unwrap();
            // Writes pixels to destination canvas
            CanvasMsg::Canvas2d(
                Canvas2dMsg::DrawImage(imagedata, source_rect.size, dest_rect, source_rect, smoothing_enabled))
        };

        self.ipc_renderer.send(msg).unwrap();
        self.mark_as_dirty();
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

        let smoothing_enabled = self.state.borrow().image_smoothing_enabled;
        self.ipc_renderer
            .send(CanvasMsg::Canvas2d(Canvas2dMsg::DrawImage(image_data,
                                                             image_size,
                                                             dest_rect,
                                                             source_rect,
                                                             smoothing_enabled)))
            .unwrap();
        self.mark_as_dirty();
        Ok(())
    }

    fn fetch_image_data(&self,
                        image_element: &HTMLImageElement)
                        -> Option<(Vec<u8>, Size2D<f64>)> {
        let url = match image_element.get_url() {
            Some(url) => url,
            None => return None,
        };

        let img = match self.request_image_from_cache(url) {
            ImageResponse::Loaded(img) => img,
            ImageResponse::PlaceholderLoaded(_) | ImageResponse::None => {
                return None
            }
        };

        let image_size = Size2D::new(img.width as f64, img.height as f64);
        let image_data = match img.format {
            PixelFormat::RGBA8 => img.bytes.to_vec(),
            PixelFormat::K8 => panic!("K8 color type not supported"),
            PixelFormat::RGB8 => panic!("RGB8 color type not supported"),
            PixelFormat::KA8 => panic!("KA8 color type not supported"),
        };

        return Some((image_data, image_size));
    }

    fn fetch_canvas_data(&self,
                         canvas_element: &HTMLCanvasElement,
                         source_rect: Rect<f64>)
                         -> Option<(Vec<u8>, Size2D<f64>)> {
        let context = match canvas_element.get_or_init_2d_context() {
            Some(context) => context,
            None => return None,
        };

        let canvas_size = canvas_element.get_size();
        let image_size = Size2D::new(canvas_size.width as f64, canvas_size.height as f64);

        let renderer = context.r().get_ipc_renderer();
        let (sender, receiver) = ipc::channel::<Vec<u8>>().unwrap();
        // Reads pixels from source canvas
        renderer.send(CanvasMsg::Canvas2d(Canvas2dMsg::GetImageData(source_rect.to_i32(),
                                                                    image_size, sender))).unwrap();

        return Some((receiver.recv().unwrap(), image_size));
    }

    fn request_image_from_cache(&self, url: Url) -> ImageResponse {
        let canvas = self.canvas.root();
        let window = window_from_node(canvas.r());
        let window = window.r();
        let image_cache = window.image_cache_task();
        let (response_chan, response_port) = ipc::channel().unwrap();
        image_cache.request_image(url, ImageCacheChan(response_chan), None);
        let result = response_port.recv().unwrap();
        result.image_response
    }

    fn create_drawable_rect(&self, x: f64, y: f64, w: f64, h: f64) -> Option<Rect<f32>> {
        if !([x, y, w, h].iter().all(|val| val.is_finite())) {
            return None;
        }

        if w == 0.0 && h == 0.0 {
            return None;
        }

        Some(Rect::new(Point2D::new(x as f32, y as f32), Size2D::new(w as f32, h as f32)))
    }
}

pub trait CanvasRenderingContext2DHelpers {
    fn get_renderer_id(self) -> usize;
    fn get_ipc_renderer(self) -> IpcSender<CanvasMsg>;
}

impl<'a> CanvasRenderingContext2DHelpers for &'a CanvasRenderingContext2D {
    fn get_renderer_id(self) -> usize {
        self.renderer_id
    }
    fn get_ipc_renderer(self) -> IpcSender<CanvasMsg> {
        self.ipc_renderer.clone()
    }
}

pub trait LayoutCanvasRenderingContext2DHelpers {
    #[allow(unsafe_code)]
    unsafe fn get_renderer_id(&self) -> usize;
    #[allow(unsafe_code)]
    unsafe fn get_ipc_renderer(&self) -> IpcSender<CanvasMsg>;
}

impl LayoutCanvasRenderingContext2DHelpers for LayoutJS<CanvasRenderingContext2D> {
    #[allow(unsafe_code)]
    unsafe fn get_renderer_id(&self) -> usize {
        (*self.unsafe_get()).renderer_id
    }
    #[allow(unsafe_code)]
    unsafe fn get_ipc_renderer(&self) -> IpcSender<CanvasMsg> {
        (*self.unsafe_get()).ipc_renderer.clone()
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
impl<'a> CanvasRenderingContext2DMethods for &'a CanvasRenderingContext2D {
    // https://html.spec.whatwg.org/multipage/#dom-context-2d-canvas
    fn Canvas(self) -> Root<HTMLCanvasElement> {
        self.canvas.root()
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-save
    fn Save(self) {
        self.saved_states.borrow_mut().push(self.state.borrow().clone());
        self.ipc_renderer.send(CanvasMsg::Canvas2d(Canvas2dMsg::SaveContext)).unwrap();
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-restore
    fn Restore(self) {
        let mut saved_states = self.saved_states.borrow_mut();
        if let Some(state) = saved_states.pop() {
            self.state.borrow_mut().clone_from(&state);
            self.ipc_renderer.send(CanvasMsg::Canvas2d(Canvas2dMsg::RestoreContext)).unwrap();
        }
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-scale
    fn Scale(self, x: f64, y: f64) {
        if !(x.is_finite() && y.is_finite()) {
            return;
        }

        let transform = self.state.borrow().transform;
        self.state.borrow_mut().transform = transform.scale(x as f32, y as f32);
        self.update_transform()
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-rotate
    fn Rotate(self, angle: f64) {
        if angle == 0.0 || !angle.is_finite() {
            return;
        }

        let (sin, cos) = (angle.sin(), angle.cos());
        let transform = self.state.borrow().transform;
        self.state.borrow_mut().transform = transform.mul(&Matrix2D::new(cos as f32, sin as f32,
                                                                         -sin as f32, cos as f32,
                                                                         0.0, 0.0));
        self.update_transform()
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-translate
    fn Translate(self, x: f64, y: f64) {
        if !(x.is_finite() && y.is_finite()) {
            return;
        }

        let transform = self.state.borrow().transform;
        self.state.borrow_mut().transform = transform.translate(x as f32, y as f32);
        self.update_transform()
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-transform
    fn Transform(self, a: f64, b: f64, c: f64, d: f64, e: f64, f: f64) {
        if !(a.is_finite() && b.is_finite() && c.is_finite() &&
             d.is_finite() && e.is_finite() && f.is_finite()) {
            return;
        }

        let transform = self.state.borrow().transform;
        self.state.borrow_mut().transform = transform.mul(&Matrix2D::new(a as f32,
                                                                         b as f32,
                                                                         c as f32,
                                                                         d as f32,
                                                                         e as f32,
                                                                         f as f32));
        self.update_transform()
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-settransform
    fn SetTransform(self, a: f64, b: f64, c: f64, d: f64, e: f64, f: f64) {
        if !(a.is_finite() && b.is_finite() && c.is_finite() &&
             d.is_finite() && e.is_finite() && f.is_finite()) {
            return;
        }

        self.state.borrow_mut().transform = Matrix2D::new(a as f32,
                                                          b as f32,
                                                          c as f32,
                                                          d as f32,
                                                          e as f32,
                                                          f as f32);
        self.update_transform()
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-resettransform
    fn ResetTransform(self) {
        self.state.borrow_mut().transform = Matrix2D::identity();
        self.update_transform()
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-globalalpha
    fn GlobalAlpha(self) -> f64 {
        let state = self.state.borrow();
        state.global_alpha
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-globalalpha
    fn SetGlobalAlpha(self, alpha: f64) {
        if !alpha.is_finite() || alpha > 1.0 || alpha < 0.0 {
            return;
        }

        self.state.borrow_mut().global_alpha = alpha;
        self.ipc_renderer
            .send(CanvasMsg::Canvas2d(Canvas2dMsg::SetGlobalAlpha(alpha as f32)))
            .unwrap()
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-globalcompositeoperation
    fn GlobalCompositeOperation(self) -> DOMString {
        let state = self.state.borrow();
        match state.global_composition {
            CompositionOrBlending::Composition(op) => op.to_str().to_owned(),
            CompositionOrBlending::Blending(op) => op.to_str().to_owned(),
        }
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-globalcompositeoperation
    fn SetGlobalCompositeOperation(self, op_str: DOMString) {
        if let Some(op) = CompositionOrBlending::from_str(&op_str) {
            self.state.borrow_mut().global_composition = op;
            self.ipc_renderer
                .send(CanvasMsg::Canvas2d(Canvas2dMsg::SetGlobalComposition(op)))
                .unwrap()
        }
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-fillrect
    fn FillRect(self, x: f64, y: f64, width: f64, height: f64) {
        if let Some(rect) = self.create_drawable_rect(x, y, width, height) {
            self.ipc_renderer.send(CanvasMsg::Canvas2d(Canvas2dMsg::FillRect(rect))).unwrap();
            self.mark_as_dirty();
        }
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-clearrect
    fn ClearRect(self, x: f64, y: f64, width: f64, height: f64) {
        if let Some(rect) = self.create_drawable_rect(x, y, width, height) {
            self.ipc_renderer
                .send(CanvasMsg::Canvas2d(Canvas2dMsg::ClearRect(rect)))
                .unwrap();
            self.mark_as_dirty();
        }
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-strokerect
    fn StrokeRect(self, x: f64, y: f64, width: f64, height: f64) {
        if let Some(rect) = self.create_drawable_rect(x, y, width, height) {
            self.ipc_renderer
                .send(CanvasMsg::Canvas2d(Canvas2dMsg::StrokeRect(rect)))
                .unwrap();
            self.mark_as_dirty();
        }
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-beginpath
    fn BeginPath(self) {
        self.ipc_renderer.send(CanvasMsg::Canvas2d(Canvas2dMsg::BeginPath)).unwrap();
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-closepath
    fn ClosePath(self) {
        self.ipc_renderer.send(CanvasMsg::Canvas2d(Canvas2dMsg::ClosePath)).unwrap();
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-fill
    fn Fill(self, _: CanvasWindingRule) {
        // TODO: Process winding rule
        self.ipc_renderer.send(CanvasMsg::Canvas2d(Canvas2dMsg::Fill)).unwrap();
        self.mark_as_dirty();
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-stroke
    fn Stroke(self) {
        self.ipc_renderer.send(CanvasMsg::Canvas2d(Canvas2dMsg::Stroke)).unwrap();
        self.mark_as_dirty();
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-clip
    fn Clip(self, _: CanvasWindingRule) {
        // TODO: Process winding rule
        self.ipc_renderer.send(CanvasMsg::Canvas2d(Canvas2dMsg::Clip)).unwrap();
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-drawimage
    fn DrawImage(self, image: HTMLImageElementOrHTMLCanvasElementOrCanvasRenderingContext2D,
                 dx: f64, dy: f64) -> Fallible<()> {
        if !(dx.is_finite() && dy.is_finite()) {
            return Ok(());
        }

        self.draw_image(image, 0f64, 0f64, None, None, dx, dy, None, None)
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-drawimage
    fn DrawImage_(self, image: HTMLImageElementOrHTMLCanvasElementOrCanvasRenderingContext2D,
                  dx: f64, dy: f64, dw: f64, dh: f64) -> Fallible<()> {
        if !(dx.is_finite() && dy.is_finite() &&
             dw.is_finite() && dh.is_finite()) {
            return Ok(());
        }

        self.draw_image(image, 0f64, 0f64, None, None, dx, dy, Some(dw), Some(dh))
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-drawimage
    fn DrawImage__(self, image: HTMLImageElementOrHTMLCanvasElementOrCanvasRenderingContext2D,
                   sx: f64, sy: f64, sw: f64, sh: f64,
                   dx: f64, dy: f64, dw: f64, dh: f64) -> Fallible<()> {
        if !(sx.is_finite() && sy.is_finite() && sw.is_finite() && sh.is_finite() &&
             dx.is_finite() && dy.is_finite() && dw.is_finite() && dh.is_finite()) {
            return Ok(());
        }

        self.draw_image(image, sx, sy, Some(sw), Some(sh), dx, dy, Some(dw), Some(dh))
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-moveto
    fn MoveTo(self, x: f64, y: f64) {
        if !(x.is_finite() && y.is_finite()) {
            return;
        }

        let msg = CanvasMsg::Canvas2d(
            Canvas2dMsg::MoveTo(
                Point2D::new(x as f32, y as f32)));
        self.ipc_renderer.send(msg).unwrap();
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-lineto
    fn LineTo(self, x: f64, y: f64) {
        if !(x.is_finite() && y.is_finite()) {
            return;
        }

        let msg = CanvasMsg::Canvas2d(
            Canvas2dMsg::LineTo(
                Point2D::new(x as f32, y as f32)));
        self.ipc_renderer.send(msg).unwrap();
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-rect
    fn Rect(self, x: f64, y: f64, width: f64, height: f64) {
        if [x, y, width, height].iter().all(|val| val.is_finite()) {
            let rect = Rect::new(Point2D::new(x as f32, y as f32),
                                 Size2D::new(width as f32, height as f32));
            let msg = CanvasMsg::Canvas2d(Canvas2dMsg::Rect(rect));
            self.ipc_renderer.send(msg).unwrap();
        }
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-quadraticcurveto
    fn QuadraticCurveTo(self, cpx: f64, cpy: f64, x: f64, y: f64) {
        if !(cpx.is_finite() && cpy.is_finite() &&
             x.is_finite() && y.is_finite()) {
            return;
        }

        let msg = CanvasMsg::Canvas2d(
            Canvas2dMsg::QuadraticCurveTo(
                Point2D::new(cpx as f32, cpy as f32),
                Point2D::new(x as f32, y as f32)));
        self.ipc_renderer.send(msg).unwrap();
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-beziercurveto
    fn BezierCurveTo(self, cp1x: f64, cp1y: f64, cp2x: f64, cp2y: f64, x: f64, y: f64) {
        if !(cp1x.is_finite() && cp1y.is_finite() && cp2x.is_finite() && cp2y.is_finite() &&
             x.is_finite() && y.is_finite()) {
            return;
        }

        let msg = CanvasMsg::Canvas2d(
            Canvas2dMsg::BezierCurveTo(
                Point2D::new(cp1x as f32, cp1y as f32),
                Point2D::new(cp2x as f32, cp2y as f32),
                Point2D::new(x as f32, y as f32)));
        self.ipc_renderer.send(msg).unwrap();
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-arc
    fn Arc(self, x: f64, y: f64, r: f64,
           start: f64, end: f64, ccw: bool) -> Fallible<()> {
        if !([x, y, r, start, end].iter().all(|x| x.is_finite())) {
            return Ok(());
        }

        if r < 0.0 {
            return Err(IndexSize);
        }

        let msg = CanvasMsg::Canvas2d(
            Canvas2dMsg::Arc(
                Point2D::new(x as f32, y as f32), r as f32,
                start as f32, end as f32, ccw));

        self.ipc_renderer.send(msg).unwrap();
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

        let msg = CanvasMsg::Canvas2d(
            Canvas2dMsg::ArcTo(
                Point2D::new(cp1x as f32, cp1y as f32),
                Point2D::new(cp2x as f32, cp2y as f32),
                r as f32));
        self.ipc_renderer.send(msg).unwrap();
        Ok(())
    }

    // https://html.spec.whatwg.org/#dom-context-2d-imagesmoothingenabled
    fn ImageSmoothingEnabled(self) -> bool {
        let state = self.state.borrow();
        state.image_smoothing_enabled
    }

    // https://html.spec.whatwg.org/#dom-context-2d-imagesmoothingenabled
    fn SetImageSmoothingEnabled(self, value: bool) -> () {
        self.state.borrow_mut().image_smoothing_enabled = value;
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-strokestyle
    fn StrokeStyle(self) -> StringOrCanvasGradientOrCanvasPattern {
        match self.state.borrow().stroke_style {
            CanvasFillOrStrokeStyle::Color(ref rgba) => {
                let mut result = String::new();
                serialize(rgba, &mut result).unwrap();
                StringOrCanvasGradientOrCanvasPattern::eString(result)
            },
            CanvasFillOrStrokeStyle::Gradient(gradient) => {
                StringOrCanvasGradientOrCanvasPattern::eCanvasGradient(gradient.root())
            },
        }
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-strokestyle
    fn SetStrokeStyle(self, value: StringOrCanvasGradientOrCanvasPattern) {
        match value {
            StringOrCanvasGradientOrCanvasPattern::eString(string) => {
                match parse_color(&string) {
                    Ok(rgba) => {
                        self.state.borrow_mut().stroke_style = CanvasFillOrStrokeStyle::Color(rgba);
                        self.ipc_renderer
                            .send(CanvasMsg::Canvas2d(Canvas2dMsg::SetStrokeStyle(
                                        FillOrStrokeStyle::Color(rgba))))
                            .unwrap();
                    }
                    _ => {}
                }
            },
            StringOrCanvasGradientOrCanvasPattern::eCanvasGradient(gradient) => {
                self.state.borrow_mut().stroke_style = CanvasFillOrStrokeStyle::Gradient(
                                                           JS::from_ref(gradient.r()));
                let msg = CanvasMsg::Canvas2d(
                    Canvas2dMsg::SetStrokeStyle(gradient.r().to_fill_or_stroke_style()));
                self.ipc_renderer.send(msg).unwrap();
            },
            _ => {}
        }
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-strokestyle
    fn FillStyle(self) -> StringOrCanvasGradientOrCanvasPattern {
        match self.state.borrow().fill_style {
            CanvasFillOrStrokeStyle::Color(ref rgba) => {
                let mut result = String::new();
                serialize(rgba, &mut result).unwrap();
                StringOrCanvasGradientOrCanvasPattern::eString(result)
            },
            CanvasFillOrStrokeStyle::Gradient(gradient) => {
                StringOrCanvasGradientOrCanvasPattern::eCanvasGradient(gradient.root())
            },
        }
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-strokestyle
    fn SetFillStyle(self, value: StringOrCanvasGradientOrCanvasPattern) {
        match value {
            StringOrCanvasGradientOrCanvasPattern::eString(string) => {
                if let Ok(rgba) = parse_color(&string) {
                    self.state.borrow_mut().fill_style = CanvasFillOrStrokeStyle::Color(rgba);
                    self.ipc_renderer
                        .send(CanvasMsg::Canvas2d(Canvas2dMsg::SetFillStyle(
                                    FillOrStrokeStyle::Color(rgba))))
                        .unwrap()
                }
            }
            StringOrCanvasGradientOrCanvasPattern::eCanvasGradient(gradient) => {
                self.state.borrow_mut().fill_style = CanvasFillOrStrokeStyle::Gradient(
                                                        JS::from_rooted(&gradient));
                let msg = CanvasMsg::Canvas2d(
                    Canvas2dMsg::SetFillStyle(gradient.r().to_fill_or_stroke_style()));
                self.ipc_renderer.send(msg).unwrap();
            }
            StringOrCanvasGradientOrCanvasPattern::eCanvasPattern(pattern) => {
                self.ipc_renderer.send(CanvasMsg::Canvas2d(Canvas2dMsg::SetFillStyle(
                                                       pattern.r().to_fill_or_stroke_style()))).unwrap();
            }
        }
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-createimagedata
    fn CreateImageData(self, sw: Finite<f64>, sh: Finite<f64>) -> Fallible<Root<ImageData>> {
        if *sw == 0.0 || *sh == 0.0 {
            return Err(IndexSize)
        }

        let sw = cmp::max(1, sw.abs().to_u32().unwrap());
        let sh = cmp::max(1, sh.abs().to_u32().unwrap());
        Ok(ImageData::new(self.global.root().r(), sw, sh, None))
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-createimagedata
    fn CreateImageData_(self, imagedata: &ImageData) -> Fallible<Root<ImageData>> {
        Ok(ImageData::new(self.global.root().r(), imagedata.Width(), imagedata.Height(), None))
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-getimagedata
    fn GetImageData(self,
                    sx: Finite<f64>,
                    sy: Finite<f64>,
                    sw: Finite<f64>,
                    sh: Finite<f64>) -> Fallible<Root<ImageData>> {
        let mut sx = *sx;
        let mut sy = *sy;
        let mut sw = *sw;
        let mut sh = *sh;

        if sw == 0.0 || sh == 0.0 {
            return Err(IndexSize)
        }

        if sw < 0.0 {
            sw = -sw;
            sx -= sw;
        }
        if sh < 0.0 {
            sh = -sh;
            sy -= sh;
        }

        let sh = cmp::max(1, sh.to_u32().unwrap());
        let sw = cmp::max(1, sw.to_u32().unwrap());

        let (sender, receiver) = ipc::channel::<Vec<u8>>().unwrap();
        let dest_rect = Rect::new(Point2D::new(sx.to_i32().unwrap(), sy.to_i32().unwrap()),
                                  Size2D::new(sw as i32, sh as i32));
        let canvas_size = self.canvas.root().r().get_size();
        let canvas_size = Size2D::new(canvas_size.width as f64, canvas_size.height as f64);
        self.ipc_renderer
            .send(CanvasMsg::Canvas2d(Canvas2dMsg::GetImageData(dest_rect, canvas_size, sender)))
            .unwrap();
        let mut data = receiver.recv().unwrap();

        // Un-premultiply alpha
        // TODO: may want a precomputed un-premultiply table to make this fast.
        // https://github.com/servo/servo/issues/6969
        for chunk in data.chunks_mut(4) {
             let alpha = chunk[3] as f32 / 255.;
             chunk[0] = (chunk[0] as f32 / alpha) as u8;
             chunk[1] = (chunk[1] as f32 / alpha) as u8;
             chunk[2] = (chunk[2] as f32 / alpha) as u8;
        }

        Ok(ImageData::new(self.global.root().r(), sw, sh, Some(data)))
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-putimagedata
    fn PutImageData(self, imagedata: &ImageData, dx: Finite<f64>, dy: Finite<f64>) {
        self.PutImageData_(imagedata, dx, dy, Finite::wrap(0f64), Finite::wrap(0f64),
                           Finite::wrap(imagedata.Width() as f64), Finite::wrap(imagedata.Height() as f64))
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-putimagedata
    fn PutImageData_(self, imagedata: &ImageData, dx: Finite<f64>, dy: Finite<f64>,
                     dirtyX: Finite<f64>, dirtyY: Finite<f64>, dirtyWidth: Finite<f64>, dirtyHeight: Finite<f64>) {
        let data = imagedata.get_data_array(&self.global.root().r());
        let offset = Point2D::new(*dx, *dy);
        let image_data_size = Size2D::new(imagedata.Width() as f64,
                                          imagedata.Height() as f64);

        let dirty_rect = Rect::new(Point2D::new(*dirtyX, *dirtyY),
                                   Size2D::new(*dirtyWidth, *dirtyHeight));
        let msg = CanvasMsg::Canvas2d(Canvas2dMsg::PutImageData(data, offset, image_data_size, dirty_rect));
        self.ipc_renderer.send(msg).unwrap();
        self.mark_as_dirty();
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-createlineargradient
    fn CreateLinearGradient(self, x0: Finite<f64>, y0: Finite<f64>,
                                  x1: Finite<f64>, y1: Finite<f64>) -> Root<CanvasGradient> {
        CanvasGradient::new(self.global.root().r(),
                            CanvasGradientStyle::Linear(LinearGradientStyle::new(*x0, *y0, *x1, *y1, Vec::new())))
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-createradialgradient
    fn CreateRadialGradient(self, x0: Finite<f64>, y0: Finite<f64>, r0: Finite<f64>,
                            x1: Finite<f64>, y1: Finite<f64>, r1: Finite<f64>)
                            -> Fallible<Root<CanvasGradient>> {
        if *r0 < 0. || *r1 < 0. {
            return Err(IndexSize);
        }

        Ok(CanvasGradient::new(self.global.root().r(),
                               CanvasGradientStyle::Radial(
                                   RadialGradientStyle::new(*x0, *y0, *r0, *x1, *y1, *r1, Vec::new()))))
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-createpattern
    fn CreatePattern(self, image: HTMLImageElementOrHTMLCanvasElementOrCanvasRenderingContext2D,
                    repetition: DOMString) -> Fallible<Root<CanvasPattern>> {
        let (image_data, image_size) = match image {
            HTMLImageElementOrHTMLCanvasElementOrCanvasRenderingContext2D::eHTMLImageElement(image) => {
                let image_element = image.r();
                // https://html.spec.whatwg.org/multipage/#img-error
                // If the image argument is an HTMLImageElement object that is in the broken state,
                // then throw an InvalidStateError exception
                match self.fetch_image_data(&image_element) {
                    Some((data, size)) => (data, size),
                    None => return Err(InvalidState),
                }
            },
            HTMLImageElementOrHTMLCanvasElementOrCanvasRenderingContext2D::eHTMLCanvasElement(canvas) => {
                let canvas_element = canvas.r();

                let canvas_size = canvas_element.get_size();
                let source_rect = Rect::new(Point2D::zero(),
                                            Size2D::new(canvas_size.width as f64, canvas_size.height as f64));

                match self.fetch_canvas_data(&canvas_element, source_rect) {
                    Some((data, size)) => (data, size),
                    None => return Err(InvalidState),
                }
            },
            HTMLImageElementOrHTMLCanvasElementOrCanvasRenderingContext2D::eCanvasRenderingContext2D(context) => {
                let canvas = context.r().Canvas();
                let canvas_element = canvas.r();

                let canvas_size = canvas_element.get_size();
                let source_rect = Rect::new(Point2D::zero(),
                                            Size2D::new(canvas_size.width as f64, canvas_size.height as f64));

                match self.fetch_canvas_data(&canvas_element, source_rect) {
                    Some((data, size)) => (data, size),
                    None => return Err(InvalidState),
                }
            },
        };

        if let Some(rep) = RepetitionStyle::from_str(&repetition) {
            return Ok(CanvasPattern::new(self.global.root().r(),
                                         image_data,
                                         Size2D::new(image_size.width as i32, image_size.height as i32),
                                         rep));
        }
        return Err(Syntax);
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-linewidth
    fn LineWidth(self) -> f64 {
        let state = self.state.borrow();
        state.line_width
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-linewidth
    fn SetLineWidth(self, width: f64) {
        if !width.is_finite() || width <= 0.0 {
            return;
        }

        self.state.borrow_mut().line_width = width;
        self.ipc_renderer
            .send(CanvasMsg::Canvas2d(Canvas2dMsg::SetLineWidth(width as f32)))
            .unwrap()
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-linecap
    fn LineCap(self) -> DOMString {
        let state = self.state.borrow();
        match state.line_cap {
            LineCapStyle::Butt => "butt".to_owned(),
            LineCapStyle::Round => "round".to_owned(),
            LineCapStyle::Square => "square".to_owned(),
        }
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-linecap
    fn SetLineCap(self, cap_str: DOMString) {
        if let Some(cap) = LineCapStyle::from_str(&cap_str) {
            self.state.borrow_mut().line_cap = cap;
            self.ipc_renderer.send(CanvasMsg::Canvas2d(Canvas2dMsg::SetLineCap(cap))).unwrap()
        }
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-linejoin
    fn LineJoin(self) -> DOMString {
        let state = self.state.borrow();
        match state.line_join {
            LineJoinStyle::Round => "round".to_owned(),
            LineJoinStyle::Bevel => "bevel".to_owned(),
            LineJoinStyle::Miter => "miter".to_owned(),
        }
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-linejoin
    fn SetLineJoin(self, join_str: DOMString) {
        if let Some(join) = LineJoinStyle::from_str(&join_str) {
            self.state.borrow_mut().line_join = join;
            self.ipc_renderer.send(CanvasMsg::Canvas2d(Canvas2dMsg::SetLineJoin(join))).unwrap()
        }
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-miterlimit
    fn MiterLimit(self) -> f64 {
        let state = self.state.borrow();
        state.miter_limit
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-miterlimit
    fn SetMiterLimit(self, limit: f64) {
        if !limit.is_finite() || limit <= 0.0 {
            return;
        }

        self.state.borrow_mut().miter_limit = limit;
        self.ipc_renderer
            .send(CanvasMsg::Canvas2d(Canvas2dMsg::SetMiterLimit(limit as f32)))
            .unwrap()
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-shadowoffsetx
    fn ShadowOffsetX(self) -> f64 {
        self.state.borrow().shadow_offset_x
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-shadowoffsetx
    fn SetShadowOffsetX(self, value: f64) {
        if !value.is_finite() || value == self.state.borrow().shadow_offset_x {
            return;
        }
        self.state.borrow_mut().shadow_offset_x = value;
        self.ipc_renderer.send(CanvasMsg::Canvas2d(Canvas2dMsg::SetShadowOffsetX(value))).unwrap()
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-shadowoffsety
    fn ShadowOffsetY(self) -> f64 {
        self.state.borrow().shadow_offset_y
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-shadowoffsety
    fn SetShadowOffsetY(self, value: f64) {
        if !value.is_finite() || value == self.state.borrow().shadow_offset_y {
            return;
        }
        self.state.borrow_mut().shadow_offset_y = value;
        self.ipc_renderer.send(CanvasMsg::Canvas2d(Canvas2dMsg::SetShadowOffsetY(value))).unwrap()
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-shadowblur
    fn ShadowBlur(self) -> f64 {
        self.state.borrow().shadow_blur
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-shadowblur
    fn SetShadowBlur(self, value: f64) {
        if !value.is_finite() || value < 0f64 || value == self.state.borrow().shadow_blur {
            return;
        }
        self.state.borrow_mut().shadow_blur = value;
        self.ipc_renderer.send(CanvasMsg::Canvas2d(Canvas2dMsg::SetShadowBlur(value))).unwrap()
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-shadowcolor
    fn ShadowColor(self) -> DOMString {
        let mut result = String::new();
        serialize(&self.state.borrow().shadow_color, &mut result).unwrap();
        result
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-shadowcolor
    fn SetShadowColor(self, value: DOMString) {
        if let Ok(color) = parse_color(&value) {
            self.state.borrow_mut().shadow_color = color;
            self.ipc_renderer
                .send(CanvasMsg::Canvas2d(Canvas2dMsg::SetShadowColor(color)))
                .unwrap()
        }
    }
}

impl Drop for CanvasRenderingContext2D {
    fn drop(&mut self) {
        self.ipc_renderer.send(CanvasMsg::Common(CanvasCommonMsg::Close)).unwrap();
    }
}

pub fn parse_color(string: &str) -> Result<RGBA,()> {
    let mut parser = Parser::new(&string);
    match CSSColor::parse(&mut parser) {
        Ok(CSSColor::RGBA(rgba)) => {
            if parser.is_exhausted() { Ok(rgba) }
            else { Err(()) }
        },
        _ => Err(()),
    }
}

// Used by drawImage to determine if a source or destination rectangle is valid
// Origin coordinates and size cannot be negative. Size has to be greater than zero
fn is_rect_valid(rect: Rect<f64>) -> bool {
    rect.size.width > 0.0 && rect.size.height > 0.0
}

// https://html.spec.whatwg.org/multipage/#serialisation-of-a-colour
fn serialize<W>(color: &RGBA, dest: &mut W) -> fmt::Result where W: fmt::Write {
    let red = (color.red * 255.).round() as u8;
    let green = (color.green * 255.).round() as u8;
    let blue = (color.blue * 255.).round() as u8;

    if color.alpha == 1f32 {
        write!(dest, "#{:x}{:x}{:x}{:x}{:x}{:x}",
               red >> 4, red & 0xF,
               green >> 4, green & 0xF,
               blue >> 4, blue & 0xF)
    } else {
        write!(dest, "rgba({}, {}, {}, {})",
               red, green, blue, color.alpha)
    }
}

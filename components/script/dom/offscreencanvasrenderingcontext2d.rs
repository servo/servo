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
use dom::bindings::codegen::Bindings::ImageDataBinding::ImageDataMethods;
use dom::bindings::codegen::Bindings::OffscreenCanvasRenderingContext2DBinding;
use dom::bindings::codegen::Bindings::OffscreenCanvasRenderingContext2DBinding::OffscreenCanvasRenderingContext2DMethods;
use dom::bindings::codegen::UnionTypes::StringOrCanvasGradientOrCanvasPattern;
use dom::bindings::error::{Error, ErrorResult, Fallible};
use dom::bindings::num::Finite;
use dom::bindings::reflector::{DomObject, Reflector, reflect_dom_object};
use dom::bindings::root::{Dom, DomRoot, LayoutDom};
use dom::bindings::str::DOMString;
use dom::canvasgradient::{CanvasGradient, CanvasGradientStyle};
use dom::canvaspattern::CanvasPattern;
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
    #[ignore_malloc_size_of = "Defined in ipc-channel"]
    ipc_renderer: IpcSender<CanvasMsg>,
    /// For rendering contexts created by an HTML canvas element, this is Some,
    /// for ones created by a paint worklet, this is None.
    canvas: Option<Dom<OffscreenCanvas>>,
    #[ignore_malloc_size_of = "Arc"]
    image_cache: Arc<ImageCache>,
    /// Any missing image URLs.
    missing_image_urls: DomRefCell<Vec<ServoUrl>>,
    /// The base URL for resolving CSS image URL values.
    /// Needed because of https://github.com/servo/servo/issues/17625
    base_url: ServoUrl,
    state: DomRefCell<CanvasContextState>,
    saved_states: DomRefCell<Vec<CanvasContextState>>,
    origin_clean: Cell<bool>,
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
        debug!("Creating new canvas rendering context.");
        let (sender, receiver) = ipc::channel().unwrap();
        let script_to_constellation_chan = global.script_to_constellation_chan();
        debug!("Asking constellation to create new canvas thread.");
        script_to_constellation_chan.send(ScriptMsg::CreateCanvasPaintThread(size, sender)).unwrap();
        let ipc_renderer = receiver.recv().unwrap();
        debug!("Done.");
        OffscreenCanvasRenderingContext2D {
            reflector_: Reflector::new(),
            ipc_renderer: ipc_renderer,
            canvas: canvas.map(Dom::from_ref),
            image_cache: image_cache,
            missing_image_urls: DomRefCell::new(Vec::new()),
            base_url: base_url,
            state: DomRefCell::new(CanvasContextState::new()),
            saved_states: DomRefCell::new(Vec::new()),
            origin_clean: Cell::new(true),
        }
    }

/*    pub fn new(global: &GlobalScope,
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
    } */
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
        //let result = match image {
        //    CanvasImageSource::HTMLImageElement(ref image) => {
        //        // https://html.spec.whatwg.org/multipage/#img-error
        //        // If the image argument is an HTMLImageElement object that is in the broken state,
        //        // then throw an InvalidStateError exception
        //        let url = image.get_url().ok_or(Error::InvalidState)?;
        //        self.fetch_and_draw_image_data(url,
        //                                       sx, sy, sw, sh,
        //                                       dx, dy, dw, dh)
        //    }
        //    CanvasImageSource::HTMLImageElement(ref image) => {
        //        // https://html.spec.whatwg.org/multipage/#img-error
        //        // If the image argument is an HTMLImageElement object that is in the broken state,
        //        // then throw an InvalidStateError exception
        //        let url = image.get_url().ok_or(Error::InvalidState)?;
        //        self.fetch_and_draw_image_data(url,
        //                                       sx, sy, sw, sh,
        //                                       dx, dy, dw, dh)
        //    }
        //    CanvasImageSource::HTMLImageElement(ref image) => {
        //        // https://html.spec.whatwg.org/multipage/#img-error
        //        // If the image argument is an HTMLImageElement object that is in the broken state,
        //        // then throw an InvalidStateError exception
        //        let url = image.get_url().ok_or(Error::InvalidState)?;
        //        self.fetch_and_draw_image_data(url,
        //                                       sx, sy, sw, sh,
        //                                       dx, dy, dw, dh)
        //    }
        //    CanvasImageSource::CSSStyleValue(ref value) => {
        //        let url = value.get_url(self.base_url.clone()).ok_or(Error::InvalidState)?;
        //        self.fetch_and_draw_image_data(url,
        //                                       sx, sy, sw, sh,
        //                                       dx, dy, dw, dh)
        //    }
        //};
        //
        //if result.is_ok() && !self.is_origin_clean(image) {
        //    self.set_origin_unclean()
        //}
        unimplemented!()
    }

    fn set_origin_unclean(&self) {
        self.origin_clean.set(false)
    }
    #[inline]
    fn request_image_from_cache(&self, url: ServoUrl) -> ImageResponse {
        let response = self.image_cache
            .find_image_or_metadata(url.clone(),
                                    UsePlaceholder::No,
                                    CanRequestImages::No);
        match response {
            Ok(ImageOrMetadataAvailable::ImageAvailable(image, url)) =>
                ImageResponse::Loaded(image, url),
            Err(ImageState::Pending(_)) =>
                ImageResponse::None,
            _ => {
                // Rather annoyingly, we get the same response back from
                // A load which really failed and from a load which hasn't started yet.
                self.missing_image_urls.borrow_mut().push(url);
                ImageResponse::None
            },
        }
    }
    pub fn origin_is_clean(&self) -> bool {
        self.origin_clean.get()
    }

    // https://html.spec.whatwg.org/multipage/#the-image-argument-is-not-origin-clean
    fn is_origin_clean(&self,
                       image: CanvasImageSource)
                           -> bool {
        true
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
    /*    debug!("Fetching image {}.", url);
        // https://html.spec.whatwg.org/multipage/#img-error
        // If the image argument is an HTMLImageElement object that is in the broken state,
        // then throw an InvalidStateError exception
        let (image_data, image_size) = match self.fetch_image_data(url) {
            Some((mut data, size)) => {
                // Pixels come from cache in BGRA order and drawImage expects RGBA so we
                // have to swap the color values
                byte_swap_and_premultiply(&mut data);
                let size = Size2D::new(size.width as f64, size.height as f64);
                (data, size)
            },
            None => return Err(Error::InvalidState),
        };
        let dw = dw.unwrap_or(image_size.width);
        let dh = dh.unwrap_or(image_size.height);
        let sw = sw.unwrap_or(image_size.width);
        let sh = sh.unwrap_or(image_size.height);
        self.draw_image_data(image_data,
                             image_size,
                             sx, sy, sw, sh,
                             dx, dy, dw, dh)
    */    Ok(())
    }

    fn fetch_image_data(&self, url: ServoUrl) -> Option<(Vec<u8>, Size2D<i32>)> {
        let img = match self.request_image_from_cache(url) {
            ImageResponse::Loaded(img, _) => img,
            ImageResponse::PlaceholderLoaded(_, _) |
            ImageResponse::None |
            ImageResponse::MetadataLoaded(_) => {
                return None;
            }
        };

        let image_size = Size2D::new(img.width as i32, img.height as i32);
        let image_data = match img.format {
            PixelFormat::BGRA8 => img.bytes.to_vec(),
            PixelFormat::K8 => panic!("K8 color type not supported"),
            PixelFormat::RGB8 => panic!("RGB8 color type not supported"),
            PixelFormat::KA8 => panic!("KA8 color type not supported"),
        };

        Some((image_data, image_size))
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
        // 1. Check the usability of the image argument
    /*    if !canvas.is_valid() {
            return Err(Error::InvalidState);
        }

        let canvas_size = canvas.get_size();
        let dw = dw.unwrap_or(canvas_size.width as f64);
        let dh = dh.unwrap_or(canvas_size.height as f64);
        let sw = sw.unwrap_or(canvas_size.width as f64);
        let sh = sh.unwrap_or(canvas_size.height as f64);

        let image_size = Size2D::new(canvas_size.width as f64, canvas_size.height as f64);
        // 2. Establish the source and destination rectangles
        let (source_rect, dest_rect) = self.adjust_source_dest_rects(image_size,
                                                                     sx,
                                                                     sy,
                                                                     sw,
                                                                     sh,
                                                                     dx,
                                                                     dy,
                                                                     dw,
                                                                     dh);

        if !is_rect_valid(source_rect) || !is_rect_valid(dest_rect) {
            return Ok(());
        }

        let smoothing_enabled = self.state.borrow().image_smoothing_enabled;

        if self.canvas.as_ref().map_or(false, |c| &**c == canvas) {
            let msg = CanvasMsg::Canvas2d(Canvas2dMsg::DrawImageSelf(
                image_size, dest_rect, source_rect, smoothing_enabled));
            self.ipc_renderer.send(msg).unwrap();
        } else {
            let context = match canvas.get_or_init_2d_context() {
                Some(context) => context,
                None => return Err(Error::InvalidState),
            };

            let (sender, receiver) = ipc::channel().unwrap();
            let msg = CanvasMsg::Canvas2d(Canvas2dMsg::DrawImageInOther(
                self.ipc_renderer.clone(),
                image_size,
                dest_rect,
                source_rect,
                smoothing_enabled,
                sender));

            let renderer = context.get_ipc_renderer();
            renderer.send(msg).unwrap();
            receiver.recv().unwrap();
        };

    //    self.mark_as_dirty();
    */    Ok(())
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
        (*self.unsafe_get()).ipc_renderer.clone()
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
        DomRoot::from_ref(self.canvas.as_ref().expect("No canvas."))
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-globalcompositeoperation
    fn GlobalCompositeOperation(&self) -> DOMString {
        let state = self.state.borrow();
        match state.global_composition {
            CompositionOrBlending::Composition(op) => DOMString::from(op.to_str()),
            CompositionOrBlending::Blending(op) => DOMString::from(op.to_str()),
        }
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-globalcompositeoperation
    fn SetGlobalCompositeOperation(&self, op_str: DOMString) {
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-drawimage
    fn DrawImage(&self,
                 image: CanvasImageSource,
                 dx: f64,
                 dy: f64)
                 -> ErrorResult {
        if !(dx.is_finite() && dy.is_finite()) {
            return Ok(());
        }

        self.draw_image(image, 0f64, 0f64, None, None, dx, dy, None, None)
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-drawimage
    fn DrawImage_(&self,
                  image: CanvasImageSource,
                  dx: f64,
                  dy: f64,
                  dw: f64,
                  dh: f64)
                  -> ErrorResult {
        if !(dx.is_finite() && dy.is_finite() && dw.is_finite() && dh.is_finite()) {
            return Ok(());
        }

        self.draw_image(image, 0f64, 0f64, None, None, dx, dy, Some(dw), Some(dh))
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
        if !(sx.is_finite() && sy.is_finite() && sw.is_finite() && sh.is_finite() &&
             dx.is_finite() && dy.is_finite() && dw.is_finite() && dh.is_finite()) {
            return Ok(());
        }

        self.draw_image(image,
                        sx,
                        sy,
                        Some(sw),
                        Some(sh),
                        dx,
                        dy,
                        Some(dw),
                        Some(dh))
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-beginpath
    fn BeginPath(&self) {
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-fill
    fn Fill(&self, _: CanvasFillRule) {
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-stroke
    fn Stroke(&self) {
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-clip
    fn Clip(&self, _: CanvasFillRule) {
        // TODO: Process fill rule
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-strokestyle
    fn StrokeStyle(&self) -> StringOrCanvasGradientOrCanvasPattern {
        match self.state.borrow().stroke_style {
            CanvasFillOrStrokeStyle::Color(ref rgba) => {
                let mut result = String::new();
                serialize(rgba, &mut result).unwrap();
                StringOrCanvasGradientOrCanvasPattern::String(DOMString::from(result))
            },
            CanvasFillOrStrokeStyle::Gradient(ref gradient) => {
                StringOrCanvasGradientOrCanvasPattern::CanvasGradient(DomRoot::from_ref(&*gradient))
            },
            CanvasFillOrStrokeStyle::Pattern(ref pattern) => {
                StringOrCanvasGradientOrCanvasPattern::CanvasPattern(DomRoot::from_ref(&*pattern))
            }
        }

    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-strokestyle
    fn SetStrokeStyle(&self, value: StringOrCanvasGradientOrCanvasPattern) {
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-strokestyle
    fn FillStyle(&self) -> StringOrCanvasGradientOrCanvasPattern {
        match self.state.borrow().fill_style {
            CanvasFillOrStrokeStyle::Color(ref rgba) => {
                let mut result = String::new();
                serialize(rgba, &mut result).unwrap();
                StringOrCanvasGradientOrCanvasPattern::String(DOMString::from(result))
            },
            CanvasFillOrStrokeStyle::Gradient(ref gradient) => {
                StringOrCanvasGradientOrCanvasPattern::CanvasGradient(DomRoot::from_ref(&*gradient))
            },
            CanvasFillOrStrokeStyle::Pattern(ref pattern) => {
                StringOrCanvasGradientOrCanvasPattern::CanvasPattern(DomRoot::from_ref(&*pattern))
            }
        }
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-createlineargradient
    fn CreateLinearGradient(&self,
                            x0: Finite<f64>,
                            y0: Finite<f64>,
                            x1: Finite<f64>,
                            y1: Finite<f64>)
                            -> DomRoot<CanvasGradient> {
        CanvasGradient::new(&self.global(),
                            CanvasGradientStyle::Linear(LinearGradientStyle::new(*x0,
                                                                                 *y0,
                                                                                 *x1,
                                                                                 *y1,
                                                                                 Vec::new())))
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
        if *r0 < 0. || *r1 < 0. {
            return Err(Error::IndexSize);
        }

        Ok(CanvasGradient::new(&self.global(),
                               CanvasGradientStyle::Radial(RadialGradientStyle::new(*x0,
                                                                                    *y0,
                                                                                    *r0,
                                                                                    *x1,
                                                                                    *y1,
                                                                                    *r1,
                                                                                    Vec::new()))))
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-createpattern
   fn CreatePattern(&self,
                     image: CanvasImageSource,
                     mut repetition: DOMString)
                     -> Fallible<DomRoot<CanvasPattern>> {
        //let (image_data, image_size) = match image {
        //    CanvasImageSource::HTMLImageElement(ref image) => {
        //        // https://html.spec.whatwg.org/multipage/#img-error
        //        // If the image argument is an HTMLImageElement object that is in the broken state,
        //        // then throw an InvalidStateError exception
        //        image.get_url()
        //            .and_then(|url| self.fetch_image_data(url))
        //            .ok_or(Error::InvalidState)?
        //    },
        //    CanvasImageSource::HTMLImageElement(ref image) => {
        //        // https://html.spec.whatwg.org/multipage/#img-error
        //        // If the image argument is an HTMLImageElement object that is in the broken state,
        //        // then throw an InvalidStateError exception
        //        image.get_url()
        //            .and_then(|url| self.fetch_image_data(url))
        //            .ok_or(Error::InvalidState)?
        //    },
        //    CanvasImageSource::HTMLImageElement(ref image) => {
        //        // https://html.spec.whatwg.org/multipage/#img-error
        //        // If the image argument is an HTMLImageElement object that is in the broken state,
        //        // then throw an InvalidStateError exception
        //        image.get_url()
        //            .and_then(|url| self.fetch_image_data(url))
        //            .ok_or(Error::InvalidState)?
        //    }
        //    CanvasImageSource::CSSStyleValue(ref value) => {
        //        value.get_url(self.base_url.clone())
        //            .and_then(|url| self.fetch_image_data(url))
        //            .ok_or(Error::InvalidState)?
        //    }
        //};
        //
        //if repetition.is_empty() {
        //    repetition.push_str("repeat");
        //}
        //
        //if let Ok(rep) = RepetitionStyle::from_str(&repetition) {
        //    Ok(CanvasPattern::new(&self.global(),
        //                          image_data,
        //                          image_size,
        //                          rep,
        //                          self.is_origin_clean(image)))
        //} else {
        //    Err(Error::Syntax)
        //}
        unimplemented!()
    }
    // https://html.spec.whatwg.org/multipage/#dom-context-2d-createimagedata
    fn CreateImageData(&self, sw: Finite<f64>, sh: Finite<f64>) -> Fallible<DomRoot<ImageData>> {
        if *sw == 0.0 || *sh == 0.0 {
            return Err(Error::IndexSize);
        }

        ImageData::new(&self.global(), DEFAULT_WIDTH, DEFAULT_HEIGHT, None)
    }


    // https://html.spec.whatwg.org/multipage/#dom-context-2d-createimagedata
    fn CreateImageData_(&self, imagedata: &ImageData) -> Fallible<DomRoot<ImageData>> {
        ImageData::new(&self.global(),
                       imagedata.Width(),
                       imagedata.Height(),
                       None)
    }


    // https://html.spec.whatwg.org/multipage/#dom-context-2d-getimagedata
    fn GetImageData(&self,
                    sx: Finite<f64>,
                    sy: Finite<f64>,
                    sw: Finite<f64>,
                    sh: Finite<f64>)
                    -> Fallible<DomRoot<ImageData>> {
        if !self.origin_is_clean() {
            return Err(Error::Security)
        }
        let mut sx = *sx;
        let mut sy = *sy;
        let mut sw = *sw;
        let mut sh = *sh;
        //let sh = cmp::max(1, sh.to_u32().unwrap());
        //let sw = cmp::max(1, sw.to_u32().unwrap());

        let (sender, receiver) = ipc::channel::<Vec<u8>>().unwrap();
        let mut data = receiver.recv().unwrap();
        ImageData::new(&self.global(), DEFAULT_WIDTH, DEFAULT_HEIGHT, Some(data))
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-putimagedata
    fn PutImageData(&self, imagedata: &ImageData, dx: Finite<f64>, dy: Finite<f64>) {
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
    }
    // https://html.spec.whatwg.org/multipage/#dom-context-2d-imagesmoothingenabled
    fn ImageSmoothingEnabled(&self) -> bool {
        let state = self.state.borrow();
        state.image_smoothing_enabled
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-imagesmoothingenabled
    fn SetImageSmoothingEnabled(&self, value: bool) {
    }
    // https://html.spec.whatwg.org/multipage/#dom-context-2d-closepath
    fn ClosePath(&self) {
    }
    // https://html.spec.whatwg.org/multipage/#dom-context-2d-moveto
    fn MoveTo(&self, x: f64, y: f64) {
    }
    // https://html.spec.whatwg.org/multipage/#dom-context-2d-lineto
    fn LineTo(&self, x: f64, y: f64) {
    }
    fn QuadraticCurveTo(&self, cpx: f64, cpy: f64, x: f64, y: f64) {
    }
    // https://html.spec.whatwg.org/multipage/#dom-context-2d-beziercurveto
    fn BezierCurveTo(&self, cp1x: f64, cp1y: f64, cp2x: f64, cp2y: f64, x: f64, y: f64) {
    }
    
    // https://html.spec.whatwg.org/multipage/#dom-context-2d-arc
    fn Arc(&self, x: f64, y: f64, r: f64, start: f64, end: f64, ccw: bool) -> ErrorResult {
            if !([x, y, r, start, end].iter().all(|x| x.is_finite())) {
                return Ok(());
            }

            if r < 0.0 {
                return Err(Error::IndexSize);
            }

            let msg = CanvasMsg::Canvas2d(Canvas2dMsg::Arc(Point2D::new(x as f32, y as f32),
                                                           r as f32,
                                                           start as f32,
                                                           end as f32,
                                                           ccw));

            self.ipc_renderer.send(msg).unwrap();
            Ok(())
    }

        // https://html.spec.whatwg.org/multipage/#dom-context-2d-arcto
        fn ArcTo(&self, cp1x: f64, cp1y: f64, cp2x: f64, cp2y: f64, r: f64) -> ErrorResult {
            if !([cp1x, cp1y, cp2x, cp2y, r].iter().all(|x| x.is_finite())) {
                return Ok(());
            }
            if r < 0.0 {
                return Err(Error::IndexSize);
            }
            let msg = CanvasMsg::Canvas2d(Canvas2dMsg::ArcTo(Point2D::new(cp1x as f32, cp1y as f32),
                                                             Point2D::new(cp2x as f32, cp2y as f32),
                                                             r as f32));
            self.ipc_renderer.send(msg).unwrap();
            Ok(())
        }
        // https://html.spec.whatwg.org/multipage/#dom-context-2d-rect
        fn Rect(&self, x: f64, y: f64, width: f64, height: f64) {
        }
        // https://html.spec.whatwg.org/multipage/#dom-context-2d-ellipse
        fn Ellipse(&self, x: f64, y: f64,
             rx: f64, ry: f64, rotation: f64, start: f64,
              end: f64, ccw: bool) -> ErrorResult {
            if !([x, y, rx, ry, rotation, start, end].iter().all(|x| x.is_finite())) {
                return Ok(());
            }
            if rx < 0.0 || ry < 0.0 {
                return Err(Error::IndexSize);
            }

            let msg = CanvasMsg::Canvas2d(Canvas2dMsg::Ellipse(Point2D::new(x as f32, y as f32),
                                                           rx as f32,
                                                           ry as f32,
                                                           rotation as f32,
                                                           start as f32,
                                                           end as f32,
                                                           ccw));
            self.ipc_renderer.send(msg).unwrap();
            Ok(())
        }
        // https://html.spec.whatwg.org/multipage/#dom-context-2d-linewidth
        fn LineWidth(&self) -> f64 {
            let state = self.state.borrow();
            state.line_width
        }

        // https://html.spec.whatwg.org/multipage/#dom-context-2d-linewidth
        fn SetLineWidth(&self, width: f64) {
        }

        // https://html.spec.whatwg.org/multipage/#dom-context-2d-linecap
        fn LineCap(&self) -> CanvasLineCap {
            match self.state.borrow().line_cap {
                LineCapStyle::Butt => CanvasLineCap::Butt,
                LineCapStyle::Round => CanvasLineCap::Round,
                LineCapStyle::Square => CanvasLineCap::Square,
            }
        }

        // https://html.spec.whatwg.org/multipage/#dom-context-2d-linecap
        fn SetLineCap(&self, cap: CanvasLineCap) {
        }

        // https://html.spec.whatwg.org/multipage/#dom-context-2d-linejoin
        fn LineJoin(&self) -> CanvasLineJoin {
            match self.state.borrow().line_join {
                LineJoinStyle::Round => CanvasLineJoin::Round,
                LineJoinStyle::Bevel => CanvasLineJoin::Bevel,
                LineJoinStyle::Miter => CanvasLineJoin::Miter,
            }
        }

        // https://html.spec.whatwg.org/multipage/#dom-context-2d-linejoin
        fn SetLineJoin(&self, join: CanvasLineJoin) {
        }

        // https://html.spec.whatwg.org/multipage/#dom-context-2d-miterlimit
        fn MiterLimit(&self) -> f64 {
            let state = self.state.borrow();
            state.miter_limit
        }

        // https://html.spec.whatwg.org/multipage/#dom-context-2d-miterlimit
        fn SetMiterLimit(&self, limit: f64) {
        }
        // https://html.spec.whatwg.org/multipage/#dom-context-2d-fillrect
        fn FillRect(&self, x: f64, y: f64, width: f64, height: f64) {
        }

        // https://html.spec.whatwg.org/multipage/#dom-context-2d-clearrect
        fn ClearRect(&self, x: f64, y: f64, width: f64, height: f64) {
        }

        // https://html.spec.whatwg.org/multipage/#dom-context-2d-strokerect
        fn StrokeRect(&self, x: f64, y: f64, width: f64, height: f64) {
        }

        // https://html.spec.whatwg.org/multipage/#dom-context-2d-shadowoffsetx
        fn ShadowOffsetX(&self) -> f64 {
            self.state.borrow().shadow_offset_x
        }

        // https://html.spec.whatwg.org/multipage/#dom-context-2d-shadowoffsetx
        fn SetShadowOffsetX(&self, value: f64) {
        }

        // https://html.spec.whatwg.org/multipage/#dom-context-2d-shadowoffsety
        fn ShadowOffsetY(&self) -> f64 {
            self.state.borrow().shadow_offset_y
        }

        // https://html.spec.whatwg.org/multipage/#dom-context-2d-shadowoffsety
        fn SetShadowOffsetY(&self, value: f64) {
        }

        // https://html.spec.whatwg.org/multipage/#dom-context-2d-shadowblur
        fn ShadowBlur(&self) -> f64 {
            self.state.borrow().shadow_blur
        }

        // https://html.spec.whatwg.org/multipage/#dom-context-2d-shadowblur
        fn SetShadowBlur(&self, value: f64) {
        }

        // https://html.spec.whatwg.org/multipage/#dom-context-2d-shadowcolor
        fn ShadowColor(&self) -> DOMString {
            let mut result = String::new();
            serialize(&self.state.borrow().shadow_color, &mut result).unwrap();
            DOMString::from(result)
        }

        // https://html.spec.whatwg.org/multipage/#dom-context-2d-shadowcolor
        fn SetShadowColor(&self, value: DOMString) {
        }
        fn Save(&self) {
        }

        #[allow(unrooted_must_root)]
        // https://html.spec.whatwg.org/multipage/#dom-context-2d-restore
        fn Restore(&self) {
        }

        // https://html.spec.whatwg.org/multipage/#dom-context-2d-scale
        fn Scale(&self, x: f64, y: f64) {
        }

        // https://html.spec.whatwg.org/multipage/#dom-context-2d-rotate
        fn Rotate(&self, angle: f64) {
        }

        // https://html.spec.whatwg.org/multipage/#dom-context-2d-translate
        fn Translate(&self, x: f64, y: f64) {
        }

        // https://html.spec.whatwg.org/multipage/#dom-context-2d-transform
        fn Transform(&self, a: f64, b: f64, c: f64, d: f64, e: f64, f: f64) {
        }

        // https://html.spec.whatwg.org/multipage/#dom-context-2d-settransform
        fn SetTransform(&self, a: f64, b: f64, c: f64, d: f64, e: f64, f: f64) {
        }

        // https://html.spec.whatwg.org/multipage/#dom-context-2d-resettransform
        fn ResetTransform(&self) {
        }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-strokestyle
    fn SetFillStyle(&self, value: StringOrCanvasGradientOrCanvasPattern) {
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-ispointinpath
    fn IsPointInPath(&self, x: f64, y: f64, fill_rule: CanvasFillRule) -> bool {
        let fill_rule = match fill_rule {
            CanvasFillRule::Nonzero => FillRule::Nonzero,
            CanvasFillRule::Evenodd => FillRule::Evenodd,
        };
        let (sender, receiver) = ipc::channel::<bool>().unwrap();
        self.ipc_renderer
            .send(CanvasMsg::Canvas2d(Canvas2dMsg::IsPointInPath(x, y, fill_rule, sender)))
            .unwrap();
        receiver.recv().unwrap()
    }
}

/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::CanvasRenderingContext2DBinding;
use crate::dom::bindings::codegen::Bindings::CanvasRenderingContext2DBinding::CanvasFillRule;
use crate::dom::bindings::codegen::Bindings::CanvasRenderingContext2DBinding::CanvasImageSource;
use crate::dom::bindings::codegen::Bindings::CanvasRenderingContext2DBinding::CanvasLineCap;
use crate::dom::bindings::codegen::Bindings::CanvasRenderingContext2DBinding::CanvasLineJoin;
use crate::dom::bindings::codegen::Bindings::CanvasRenderingContext2DBinding::CanvasRenderingContext2DMethods;
use crate::dom::bindings::codegen::Bindings::ImageDataBinding::ImageDataMethods;
use crate::dom::bindings::codegen::UnionTypes::StringOrCanvasGradientOrCanvasPattern;
use crate::dom::bindings::error::{Error, ErrorResult, Fallible};
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::num::Finite;
use crate::dom::bindings::reflector::{reflect_dom_object, DomObject, Reflector};
use crate::dom::bindings::root::{Dom, DomRoot, LayoutDom};
use crate::dom::bindings::str::DOMString;
use crate::dom::canvasgradient::{CanvasGradient, CanvasGradientStyle, ToFillOrStrokeStyle};
use crate::dom::canvaspattern::CanvasPattern;
use crate::dom::element::Element;
use crate::dom::globalscope::GlobalScope;
use crate::dom::htmlcanvaselement::{CanvasContext, HTMLCanvasElement};
use crate::dom::imagedata::ImageData;
use crate::dom::node::{window_from_node, Node, NodeDamage};
use crate::unpremultiplytable::UNPREMULTIPLY_TABLE;
use canvas_traits::canvas::{Canvas2dMsg, CanvasId, CanvasMsg};
use canvas_traits::canvas::{CompositionOrBlending, FillOrStrokeStyle, FillRule};
use canvas_traits::canvas::{LineCapStyle, LineJoinStyle, LinearGradientStyle};
use canvas_traits::canvas::{RadialGradientStyle, RepetitionStyle};
use cssparser::Color as CSSColor;
use cssparser::{Parser, ParserInput, RGBA};
use dom_struct::dom_struct;
use euclid::{vec2, Point2D, Rect, Size2D, Transform2D};
use ipc_channel::ipc::{self, IpcSender};
use net_traits::image_cache::CanRequestImages;
use net_traits::image_cache::ImageCache;
use net_traits::image_cache::ImageOrMetadataAvailable;
use net_traits::image_cache::ImageResponse;
use net_traits::image_cache::ImageState;
use net_traits::image_cache::UsePlaceholder;
use pixels::PixelFormat;
use profile_traits::ipc as profiled_ipc;
use script_traits::ScriptMsg;
use servo_url::ServoUrl;
use std::cell::Cell;
use std::str::FromStr;
use std::sync::Arc;
use std::{fmt, mem};

#[must_root]
#[derive(Clone, JSTraceable, MallocSizeOf)]
#[allow(dead_code)]
enum CanvasFillOrStrokeStyle {
    Color(RGBA),
    Gradient(Dom<CanvasGradient>),
    Pattern(Dom<CanvasPattern>),
}

// https://html.spec.whatwg.org/multipage/#canvasrenderingcontext2d
#[dom_struct]
pub struct CanvasRenderingContext2D {
    reflector_: Reflector,
    /// For rendering contexts created by an HTML canvas element, this is Some,
    /// for ones created by a paint worklet, this is None.
    canvas: Option<Dom<HTMLCanvasElement>>,
    #[ignore_malloc_size_of = "Arc"]
    image_cache: Arc<ImageCache>,
    /// The base URL for resolving CSS image URL values.
    /// Needed because of https://github.com/servo/servo/issues/17625
    base_url: ServoUrl,
    canvas_state: DomRefCell<CanvasState>,
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

#[must_root]
#[derive(JSTraceable, MallocSizeOf)]
pub struct CanvasState {
    #[ignore_malloc_size_of = "Defined in ipc-channel"]
    ipc_renderer: IpcSender<CanvasMsg>,
    canvas_id: CanvasId,
    state: DomRefCell<CanvasContextState>,
    origin_clean: Cell<bool>,
    canvas: Option<&HTMLCanvasElement>,
    #[ignore_malloc_size_of = "Arc"]
    image_cache: Arc<dyn ImageCache>,
    base_url: ServoUrl,
    global: &GlobalScope,
    /// Any missing image URLs.
    missing_image_urls: DomRefCell<Vec<ServoUrl>>,
    saved_states: DomRefCell<Vec<CanvasContextState>>,
}

impl CanvasState {
    pub fn new(
        global: &GlobalScope,
        canvas: Option<&HTMLCanvasElement>,
        image_cache: Arc<dyn ImageCache>,
        base_url: ServoUrl,
        size: Size2D<u64>,
    ) -> CanvasState {
        debug!("Creating new canvas rendering context.");
        let (sender, receiver) =
            profiled_ipc::channel(global.time_profiler_chan().clone()).unwrap();
        let script_to_constellation_chan = global.script_to_constellation_chan();
        debug!("Asking constellation to create new canvas thread.");
        script_to_constellation_chan
            .send(ScriptMsg::CreateCanvasPaintThread(size, sender))
            .unwrap();
        let (ipc_renderer, canvas_id) = receiver.recv().unwrap();
        debug!("Done.");
        CanvasState {
            ipc_renderer: ipc_renderer,
            canvas_id: canvas_id,
            state: DomRefCell::new(CanvasContextState::new()),
            origin_clean: Cell::new(true),
            canvas: canvas,
            image_cache: image_cache,
            base_url: base_url,
            global: global,
            missing_image_urls: DomRefCell::new(Vec::new()),
            saved_states: DomRefCell::new(Vec::new()),
        }
    }

    pub fn get_canvas_id(&self) -> CanvasId {
        self.canvas_id.clone()
    }

    pub fn send_canvas_2d_msg(&self, msg: Canvas2dMsg) {
        self.ipc_renderer
            .send(CanvasMsg::Canvas2d(msg, self.get_canvas_id()))
            .unwrap()
    }

    fn create_drawable_rect(&self, x: f64, y: f64, w: f64, h: f64) -> Option<Rect<f32>> {
        if !([x, y, w, h].iter().all(|val| val.is_finite())) {
            return None;
        }

        if w == 0.0 && h == 0.0 {
            return None;
        }

        Some(Rect::new(
            Point2D::new(x as f32, y as f32),
            Size2D::new(w as f32, h as f32),
        ))
    }

    fn origin_is_clean(&self) -> bool {
        self.origin_clean.get()
    }

    fn set_origin_unclean(&self) {
        self.origin_clean.set(false)
    }

    // https://html.spec.whatwg.org/multipage/#the-image-argument-is-not-origin-clean
    fn is_origin_clean(&self, image: CanvasImageSource) -> bool {
        match image {
            CanvasImageSource::HTMLCanvasElement(canvas) => canvas.origin_is_clean(),
            CanvasImageSource::HTMLImageElement(image) => {
                image.same_origin(GlobalScope::entry().origin())
            },
            CanvasImageSource::CSSStyleValue(_) => true,
        }
    }

    fn fetch_image_data(&self, url: ServoUrl) -> Option<(Vec<u8>, Size2D<u32>)> {
        let img = match self.request_image_from_cache(url) {
            ImageResponse::Loaded(img, _) => img,
            ImageResponse::PlaceholderLoaded(_, _) |
            ImageResponse::None |
            ImageResponse::MetadataLoaded(_) => {
                return None;
            },
        };

        let image_size = Size2D::new(img.width, img.height);
        let image_data = match img.format {
            PixelFormat::BGRA8 => img.bytes.to_vec(),
            pixel_format => unimplemented!("unsupported pixel format ({:?})", pixel_format),
        };

        Some((image_data, image_size))
    }

    #[inline]
    fn request_image_from_cache(&self, url: ServoUrl) -> ImageResponse {
        let response = self.image_cache.find_image_or_metadata(
            url.clone(),
            UsePlaceholder::No,
            CanRequestImages::No,
        );
        match response {
            Ok(ImageOrMetadataAvailable::ImageAvailable(image, url)) => {
                ImageResponse::Loaded(image, url)
            },
            Err(ImageState::Pending(_)) => ImageResponse::None,
            _ => {
                // Rather annoyingly, we get the same response back from
                // A load which really failed and from a load which hasn't started yet.
                self.missing_image_urls.borrow_mut().push(url);
                ImageResponse::None
            },
        }
    }

     fn parse_color(&self, string: &str) -> Result<RGBA, ()> {
        let mut input = ParserInput::new(string);
        let mut parser = Parser::new(&mut input);
        let color = CSSColor::parse(&mut parser);
        if parser.is_exhausted() {
            match color {
                Ok(CSSColor::RGBA(rgba)) => Ok(rgba),
                Ok(CSSColor::CurrentColor) => {
                    // TODO: https://github.com/whatwg/html/issues/1099
                    // Reconsider how to calculate currentColor in a display:none canvas

                    // TODO: will need to check that the context bitmap mode is fixed
                    // once we implement CanvasProxy
                    let canvas = match self.canvas {
                        // https://drafts.css-houdini.org/css-paint-api/#2d-rendering-context
                        // Whenever "currentColor" is used as a color in the PaintRenderingContext2D API,
                        // it is treated as opaque black.
                        None => return Ok(RGBA::new(0, 0, 0, 255)),
                        Some(ref canvas) => &**canvas,
                    };

                    let canvas_element = canvas.upcast::<Element>();

                    match canvas_element.style() {
                        Some(ref s) if canvas_element.has_css_layout_box() => {
                            Ok(s.get_color().color)
                        },
                        _ => Ok(RGBA::new(0, 0, 0, 255)),
                    }
                },
                _ => Err(()),
            }
        } else {
            Err(())
        }
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-fillrect
    pub fn FillRect(&self, x: f64, y: f64, width: f64, height: f64) {
        if let Some(rect) = self.create_drawable_rect(x, y, width, height) {
            self.send_canvas_2d_msg(Canvas2dMsg::FillRect(rect));
        }
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-clearrect
    pub fn ClearRect(&self, x: f64, y: f64, width: f64, height: f64) {
        if let Some(rect) = self.create_drawable_rect(x, y, width, height) {
            self.send_canvas_2d_msg(Canvas2dMsg::ClearRect(rect));
        }
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-strokerect
    pub fn StrokeRect(&self, x: f64, y: f64, width: f64, height: f64) {
        if let Some(rect) = self.create_drawable_rect(x, y, width, height) {
            self.send_canvas_2d_msg(Canvas2dMsg::StrokeRect(rect));
        }
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-shadowoffsetx
    pub fn ShadowOffsetX(&self) -> f64 {
        self.state.borrow().shadow_offset_x
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-shadowoffsetx
    pub fn SetShadowOffsetX(&self, value: f64) {
        if !value.is_finite() || value == self.state.borrow().shadow_offset_x {
            return;
        }
        self.state.borrow_mut().shadow_offset_x = value;
        self.send_canvas_2d_msg(Canvas2dMsg::SetShadowOffsetX(value))
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-shadowoffsety
    pub fn ShadowOffsetY(&self) -> f64 {
        self.state.borrow().shadow_offset_y
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-shadowoffsety
    pub fn SetShadowOffsetY(&self, value: f64) {
        if !value.is_finite() || value == self.state.borrow().shadow_offset_y {
            return;
        }
        self.state.borrow_mut().shadow_offset_y = value;
        self.send_canvas_2d_msg(Canvas2dMsg::SetShadowOffsetY(value))
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-shadowblur
    pub fn ShadowBlur(&self) -> f64 {
        self.state.borrow().shadow_blur
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-shadowblur
    pub fn SetShadowBlur(&self, value: f64) {
        if !value.is_finite() || value < 0f64 || value == self.state.borrow().shadow_blur {
            return;
        }
        self.state.borrow_mut().shadow_blur = value;
        self.send_canvas_2d_msg(Canvas2dMsg::SetShadowBlur(value))
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-shadowcolor
    pub fn ShadowColor(&self) -> DOMString {
        let mut result = String::new();
        serialize(&self.state.borrow().shadow_color, &mut result).unwrap();
        DOMString::from(result)
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-shadowcolor
    pub fn SetShadowColor(&self, value: DOMString) {
        if let Ok(color) = parse_color(&value) {
            self.state.borrow_mut().shadow_color = color;
            self.send_canvas_2d_msg(Canvas2dMsg::SetShadowColor(color))
        }
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-strokestyle
    pub fn StrokeStyle(&self) -> StringOrCanvasGradientOrCanvasPattern {
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
            },
        }
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-strokestyle
    pub fn SetStrokeStyle(&self, value: StringOrCanvasGradientOrCanvasPattern) {
        match value {
            StringOrCanvasGradientOrCanvasPattern::String(string) => {
                if let Ok(rgba) = self.parse_color(&string) {
                    self.state.borrow_mut().stroke_style = CanvasFillOrStrokeStyle::Color(rgba);
                    self.send_canvas_2d_msg(Canvas2dMsg::SetStrokeStyle(FillOrStrokeStyle::Color(
                        rgba,
                    )));
                }
            },
            StringOrCanvasGradientOrCanvasPattern::CanvasGradient(gradient) => {
                self.state.borrow_mut().stroke_style =
                    CanvasFillOrStrokeStyle::Gradient(Dom::from_ref(&*gradient));
                self.send_canvas_2d_msg(Canvas2dMsg::SetStrokeStyle(
                    gradient.to_fill_or_stroke_style(),
                ));
            },
            StringOrCanvasGradientOrCanvasPattern::CanvasPattern(pattern) => {
                self.state.borrow_mut().stroke_style =
                    CanvasFillOrStrokeStyle::Pattern(Dom::from_ref(&*pattern));
                self.send_canvas_2d_msg(Canvas2dMsg::SetStrokeStyle(
                    pattern.to_fill_or_stroke_style(),
                ));
                if !pattern.origin_is_clean() {
                    self.set_origin_unclean();
                }
            },
        }
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-strokestyle
    pub fn FillStyle(&self) -> StringOrCanvasGradientOrCanvasPattern {
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
            },
        }
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-strokestyle
    pub fn SetFillStyle(&self, value: StringOrCanvasGradientOrCanvasPattern) {
        match value {
            StringOrCanvasGradientOrCanvasPattern::String(string) => {
                if let Ok(rgba) = self.parse_color(&string) {
                    self.state.borrow_mut().fill_style = CanvasFillOrStrokeStyle::Color(rgba);
                    self.send_canvas_2d_msg(Canvas2dMsg::SetFillStyle(FillOrStrokeStyle::Color(
                        rgba,
                    )))
                }
            },
            StringOrCanvasGradientOrCanvasPattern::CanvasGradient(gradient) => {
                self.state.borrow_mut().fill_style =
                    CanvasFillOrStrokeStyle::Gradient(Dom::from_ref(&*gradient));
                self.send_canvas_2d_msg(Canvas2dMsg::SetFillStyle(
                    gradient.to_fill_or_stroke_style(),
                ));
            },
            StringOrCanvasGradientOrCanvasPattern::CanvasPattern(pattern) => {
                self.state.borrow_mut().fill_style =
                    CanvasFillOrStrokeStyle::Pattern(Dom::from_ref(&*pattern));
                self.send_canvas_2d_msg(Canvas2dMsg::SetFillStyle(
                    pattern.to_fill_or_stroke_style(),
                ));
                if !pattern.origin_is_clean() {
                    self.set_origin_unclean();
                }
            },
        }
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-createlineargradient
    pub fn CreateLinearGradient(
        &self,
        x0: Finite<f64>,
        y0: Finite<f64>,
        x1: Finite<f64>,
        y1: Finite<f64>,
    ) -> DomRoot<CanvasGradient> {
        CanvasGradient::new(
            self.global,
            CanvasGradientStyle::Linear(LinearGradientStyle::new(*x0, *y0, *x1, *y1, Vec::new())),
        )
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-createradialgradient
    pub fn CreateRadialGradient(
        &self,
        x0: Finite<f64>,
        y0: Finite<f64>,
        r0: Finite<f64>,
        x1: Finite<f64>,
        y1: Finite<f64>,
        r1: Finite<f64>,
    ) -> Fallible<DomRoot<CanvasGradient>> {
        if *r0 < 0. || *r1 < 0. {
            return Err(Error::IndexSize);
        }

        Ok(CanvasGradient::new(
            self.global,
            CanvasGradientStyle::Radial(RadialGradientStyle::new(
                *x0,
                *y0,
                *r0,
                *x1,
                *y1,
                *r1,
                Vec::new(),
            )),
        ))
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-createpattern
    pub fn CreatePattern(
        &self,
        image: CanvasImageSource,
        mut repetition: DOMString,
    ) -> Fallible<DomRoot<CanvasPattern>> {
        let (image_data, image_size) = match image {
            CanvasImageSource::HTMLImageElement(ref image) => {
                // https://html.spec.whatwg.org/multipage/#img-error
                // If the image argument is an HTMLImageElement object that is in the broken state,
                // then throw an InvalidStateError exception
                image
                    .get_url()
                    .and_then(|url| self.fetch_image_data(url))
                    .ok_or(Error::InvalidState)?
            },
            CanvasImageSource::HTMLCanvasElement(ref canvas) => {
                let (data, size) = canvas.fetch_all_data().ok_or(Error::InvalidState)?;
                let data = data
                    .map(|data| data.to_vec())
                    .unwrap_or_else(|| vec![0; size.area() as usize * 4]);
                (data, size)
            },
            CanvasImageSource::CSSStyleValue(ref value) => value
                .get_url(self.base_url.clone())
                .and_then(|url| self.fetch_image_data(url))
                .ok_or(Error::InvalidState)?,
        };

        if repetition.is_empty() {
            repetition.push_str("repeat");
        }

        if let Ok(rep) = RepetitionStyle::from_str(&repetition) {
            Ok(CanvasPattern::new(
                self.global,
                image_data,
                image_size,
                rep,
                self.is_origin_clean(image),
            ))
        } else {
            Err(Error::Syntax)
        }
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-save
    pub fn Save(&self) {
        self.saved_states
            .borrow_mut()
            .push(self.state.borrow().clone());
        self.send_canvas_2d_msg(Canvas2dMsg::SaveContext);
    }

    #[allow(unrooted_must_root)]
    // https://html.spec.whatwg.org/multipage/#dom-context-2d-restore
    pub fn Restore(&self) {
        let mut saved_states = self.saved_states.borrow_mut();
        if let Some(state) = saved_states.pop() {
            self.state
                .borrow_mut()
                .clone_from(&state);
            self.send_canvas_2d_msg(Canvas2dMsg::RestoreContext);
        }
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-globalalpha
    pub fn GlobalAlpha(&self) -> f64 {
        self.state.borrow().global_alpha
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-globalalpha
    pub fn SetGlobalAlpha(&self, alpha: f64) {
        if !alpha.is_finite() || alpha > 1.0 || alpha < 0.0 {
            return;
        }

        self.state.borrow_mut().global_alpha = alpha;
        self.send_canvas_2d_msg(Canvas2dMsg::SetGlobalAlpha(alpha as f32))
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-globalcompositeoperation
    pub fn GlobalCompositeOperation(&self) -> DOMString {
        match self.state.borrow().global_composition {
            CompositionOrBlending::Composition(op) => DOMString::from(op.to_str()),
            CompositionOrBlending::Blending(op) => DOMString::from(op.to_str()),
        }
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-globalcompositeoperation
    pub fn SetGlobalCompositeOperation(&self, op_str: DOMString) {
        if let Ok(op) = CompositionOrBlending::from_str(&op_str) {
            self.state
                .borrow_mut()
                .global_composition = op;
            self.send_canvas_2d_msg(Canvas2dMsg::SetGlobalComposition(op))
        }
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-imagesmoothingenabled
    pub fn ImageSmoothingEnabled(&self) -> bool {
        self.state
            .borrow()
            .image_smoothing_enabled
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-imagesmoothingenabled
    pub fn SetImageSmoothingEnabled(&self, value: bool) {
        self.state
            .borrow_mut()
            .image_smoothing_enabled = value;
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-filltext
    pub fn FillText(&self, text: DOMString, x: f64, y: f64, max_width: Option<f64>) {
        let parsed_text: String = text.into();
        self.send_canvas_2d_msg(Canvas2dMsg::FillText(parsed_text, x, y, max_width));
    }
}

impl CanvasRenderingContext2D {
    pub fn new_inherited(
        global: &GlobalScope,
        canvas: Option<&HTMLCanvasElement>,
        image_cache: Arc<dyn ImageCache>,
        base_url: ServoUrl,
        size: Size2D<u32>,
    ) -> CanvasRenderingContext2D {
        CanvasRenderingContext2D {
            reflector_: Reflector::new(),
            canvas: canvas.map(Dom::from_ref),
            image_cache: image_cache,
            base_url: base_url,
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
        canvas: &HTMLCanvasElement,
        size: Size2D<u32>,
    ) -> DomRoot<CanvasRenderingContext2D> {
        let window = window_from_node(canvas);
        let image_cache = window.image_cache();
        let base_url = window.get_url();
        let boxed = Box::new(CanvasRenderingContext2D::new_inherited(
            global,
            Some(canvas),
            image_cache,
            base_url,
            size,
        ));
        reflect_dom_object(boxed, global, CanvasRenderingContext2DBinding::Wrap)
    }

    // https://html.spec.whatwg.org/multipage/#concept-canvas-set-bitmap-dimensions
    pub fn set_bitmap_dimensions(&self, size: Size2D<u32>) {
        self.reset_to_initial_state();
        self.canvas_state
            .borrow()
            .ipc_renderer
            .send(CanvasMsg::Recreate(
                size,
                self.canvas_state.borrow().get_canvas_id(),
            ))
            .unwrap();
    }

    // https://html.spec.whatwg.org/multipage/#reset-the-rendering-context-to-its-default-state
    fn reset_to_initial_state(&self) {
        self.canvas_state.borrow().saved_states.borrow_mut().clear();
        *self.canvas_state.borrow().state.borrow_mut() = CanvasContextState::new();
    }

    fn mark_as_dirty(&self) {
        if let Some(ref canvas) = self.canvas {
            canvas.upcast::<Node>().dirty(NodeDamage::OtherNodeDamage);
        }
    }

    fn update_transform(&self) {
        self.send_canvas_2d_msg(Canvas2dMsg::SetTransform(
            self.canvas_state.borrow().state.borrow().transform,
        ))
    }

    // It is used by DrawImage to calculate the size of the source and destination rectangles based
    // on the drawImage call arguments
    // source rectangle = area of the original image to be copied
    // destination rectangle = area of the destination canvas where the source image is going to be drawn
    fn adjust_source_dest_rects(
        &self,
        image_size: Size2D<f64>,
        sx: f64,
        sy: f64,
        sw: f64,
        sh: f64,
        dx: f64,
        dy: f64,
        dw: f64,
        dh: f64,
    ) -> (Rect<f64>, Rect<f64>) {
        let image_rect = Rect::new(
            Point2D::new(0f64, 0f64),
            Size2D::new(image_size.width as f64, image_size.height as f64),
        );

        // The source rectangle is the rectangle whose corners are the four points (sx, sy),
        // (sx+sw, sy), (sx+sw, sy+sh), (sx, sy+sh).
        let source_rect = Rect::new(
            Point2D::new(sx.min(sx + sw), sy.min(sy + sh)),
            Size2D::new(sw.abs(), sh.abs()),
        );

        // When the source rectangle is outside the source image,
        // the source rectangle must be clipped to the source image
        let source_rect_clipped = source_rect
            .intersection(&image_rect)
            .unwrap_or(Rect::zero());

        // Width and height ratios between the non clipped and clipped source rectangles
        let width_ratio: f64 = source_rect_clipped.size.width / source_rect.size.width;
        let height_ratio: f64 = source_rect_clipped.size.height / source_rect.size.height;

        // When the source rectangle is outside the source image,
        // the destination rectangle must be clipped in the same proportion.
        let dest_rect_width_scaled: f64 = dw * width_ratio;
        let dest_rect_height_scaled: f64 = dh * height_ratio;

        // The destination rectangle is the rectangle whose corners are the four points (dx, dy),
        // (dx+dw, dy), (dx+dw, dy+dh), (dx, dy+dh).
        let dest_rect = Rect::new(
            Point2D::new(
                dx.min(dx + dest_rect_width_scaled),
                dy.min(dy + dest_rect_height_scaled),
            ),
            Size2D::new(dest_rect_width_scaled.abs(), dest_rect_height_scaled.abs()),
        );

        let source_rect = Rect::new(
            Point2D::new(source_rect_clipped.origin.x, source_rect_clipped.origin.y),
            Size2D::new(
                source_rect_clipped.size.width,
                source_rect_clipped.size.height,
            ),
        );

        (source_rect, dest_rect)
    }

    // https://html.spec.whatwg.org/multipage/#the-image-argument-is-not-origin-clean
    fn is_origin_clean(&self, image: CanvasImageSource) -> bool {
        self.canvas_state.borrow().is_origin_clean(image)
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
    fn draw_image(
        &self,
        image: CanvasImageSource,
        sx: f64,
        sy: f64,
        sw: Option<f64>,
        sh: Option<f64>,
        dx: f64,
        dy: f64,
        dw: Option<f64>,
        dh: Option<f64>,
    ) -> ErrorResult {
        let result = match image {
            CanvasImageSource::HTMLCanvasElement(ref canvas) => {
                self.draw_html_canvas_element(&canvas, sx, sy, sw, sh, dx, dy, dw, dh)
            },
            CanvasImageSource::HTMLImageElement(ref image) => {
                // https://html.spec.whatwg.org/multipage/#img-error
                // If the image argument is an HTMLImageElement object that is in the broken state,
                // then throw an InvalidStateError exception
                let url = image.get_url().ok_or(Error::InvalidState)?;
                self.fetch_and_draw_image_data(url, sx, sy, sw, sh, dx, dy, dw, dh)
            },
            CanvasImageSource::CSSStyleValue(ref value) => {
                let url = value
                    .get_url(self.base_url.clone())
                    .ok_or(Error::InvalidState)?;
                self.fetch_and_draw_image_data(url, sx, sy, sw, sh, dx, dy, dw, dh)
            },
        };

        if result.is_ok() && !self.is_origin_clean(image) {
            self.set_origin_unclean()
        }
        result
    }

    fn draw_html_canvas_element(
        &self,
        canvas: &HTMLCanvasElement,
        sx: f64,
        sy: f64,
        sw: Option<f64>,
        sh: Option<f64>,
        dx: f64,
        dy: f64,
        dw: Option<f64>,
        dh: Option<f64>,
    ) -> ErrorResult {
        // 1. Check the usability of the image argument
        if !canvas.is_valid() {
            return Err(Error::InvalidState);
        }

        let canvas_size = canvas.get_size();
        let dw = dw.unwrap_or(canvas_size.width as f64);
        let dh = dh.unwrap_or(canvas_size.height as f64);
        let sw = sw.unwrap_or(canvas_size.width as f64);
        let sh = sh.unwrap_or(canvas_size.height as f64);

        let image_size = Size2D::new(canvas_size.width as f64, canvas_size.height as f64);
        // 2. Establish the source and destination rectangles
        let (source_rect, dest_rect) =
            self.adjust_source_dest_rects(image_size, sx, sy, sw, sh, dx, dy, dw, dh);

        if !is_rect_valid(source_rect) || !is_rect_valid(dest_rect) {
            return Ok(());
        }

        let smoothing_enabled = self
            .canvas_state
            .borrow()
            .state
            .borrow()
            .image_smoothing_enabled;

        if let Some(context) = canvas.context() {
            match *context {
                CanvasContext::Context2d(ref context) => {
                    context.send_canvas_2d_msg(Canvas2dMsg::DrawImageInOther(
                        self.canvas_state.borrow().get_canvas_id(),
                        image_size,
                        dest_rect,
                        source_rect,
                        smoothing_enabled,
                    ));
                },
                _ => return Err(Error::InvalidState),
            }
        } else {
            self.send_canvas_2d_msg(Canvas2dMsg::DrawImage(
                None,
                image_size,
                dest_rect,
                source_rect,
                smoothing_enabled,
            ));
        }

        self.mark_as_dirty();
        Ok(())
    }

    fn fetch_and_draw_image_data(
        &self,
        url: ServoUrl,
        sx: f64,
        sy: f64,
        sw: Option<f64>,
        sh: Option<f64>,
        dx: f64,
        dy: f64,
        dw: Option<f64>,
        dh: Option<f64>,
    ) -> ErrorResult {
        debug!("Fetching image {}.", url);
        let (mut image_data, image_size) = self.canvas_state.borrow().fetch_image_data(url).ok_or(Error::InvalidState)?;
        pixels::rgba8_premultiply_inplace(&mut image_data);
        let image_size = image_size.to_f64();

        let dw = dw.unwrap_or(image_size.width);
        let dh = dh.unwrap_or(image_size.height);
        let sw = sw.unwrap_or(image_size.width);
        let sh = sh.unwrap_or(image_size.height);

        // Establish the source and destination rectangles
        let (source_rect, dest_rect) =
            self.adjust_source_dest_rects(image_size, sx, sy, sw, sh, dx, dy, dw, dh);

        if !is_rect_valid(source_rect) || !is_rect_valid(dest_rect) {
            return Ok(());
        }

        let smoothing_enabled = self
            .canvas_state
            .borrow()
            .state
            .borrow()
            .image_smoothing_enabled;
        self.send_canvas_2d_msg(Canvas2dMsg::DrawImage(
            Some(image_data.into()),
            image_size,
            dest_rect,
            source_rect,
            smoothing_enabled,
        ));
        self.mark_as_dirty();
        Ok(())
    }

    fn fetch_image_data(&self, url: ServoUrl) -> Option<(Vec<u8>, Size2D<u32>)> {
        self.canvas_state.borrow().fetch_image_data(url)
    }

    pub fn take_missing_image_urls(&self) -> Vec<ServoUrl> {
        mem::replace(&mut self.canvas_state.borrow().missing_image_urls.borrow_mut(), vec![])
    }

    pub fn get_canvas_id(&self) -> CanvasId {
        self.canvas_state.borrow().get_canvas_id()
    }

    pub fn send_canvas_2d_msg(&self, msg: Canvas2dMsg) {
        self.canvas_state.borrow().send_canvas_2d_msg(msg)
    }

    pub fn get_ipc_renderer(&self) -> IpcSender<CanvasMsg> {
        self.canvas_state.borrow().ipc_renderer.clone()
    }

    pub fn origin_is_clean(&self) -> bool {
        self.canvas_state.borrow().origin_is_clean()
    }

    fn set_origin_unclean(&self) {
        self.canvas_state.borrow().set_origin_unclean()
    }

    pub fn get_rect(&self, rect: Rect<u32>) -> Vec<u8> {
        assert!(self.origin_is_clean());

        // FIXME(nox): This is probably wrong when this is a context for an
        // offscreen canvas.
        let canvas_size = self
            .canvas
            .as_ref()
            .map_or(Size2D::zero(), |c| c.get_size());
        assert!(Rect::from_size(canvas_size).contains_rect(&rect));

        let (sender, receiver) = ipc::bytes_channel().unwrap();
        self.send_canvas_2d_msg(Canvas2dMsg::GetImageData(rect, canvas_size, sender));
        let mut pixels = receiver.recv().unwrap().to_vec();

        for chunk in pixels.chunks_mut(4) {
            let b = chunk[0];
            chunk[0] = UNPREMULTIPLY_TABLE[256 * (chunk[3] as usize) + chunk[2] as usize];
            chunk[1] = UNPREMULTIPLY_TABLE[256 * (chunk[3] as usize) + chunk[1] as usize];
            chunk[2] = UNPREMULTIPLY_TABLE[256 * (chunk[3] as usize) + b as usize];
        }

        pixels
    }
}

pub trait LayoutCanvasRenderingContext2DHelpers {
    #[allow(unsafe_code)]
    unsafe fn get_ipc_renderer(&self) -> IpcSender<CanvasMsg>;
    #[allow(unsafe_code)]
    unsafe fn get_canvas_id(&self) -> CanvasId;
}

impl LayoutCanvasRenderingContext2DHelpers for LayoutDom<CanvasRenderingContext2D> {
    #[allow(unsafe_code)]
    unsafe fn get_ipc_renderer(&self) -> IpcSender<CanvasMsg> {
        (*self.unsafe_get())
            .canvas_state
            .borrow_for_layout()
            .ipc_renderer
            .clone()
    }

    #[allow(unsafe_code)]
    unsafe fn get_canvas_id(&self) -> CanvasId {
        (*self.unsafe_get())
            .canvas_state
            .borrow_for_layout()
            .get_canvas_id()
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
impl CanvasRenderingContext2DMethods for CanvasRenderingContext2D {
    // https://html.spec.whatwg.org/multipage/#dom-context-2d-canvas
    fn Canvas(&self) -> DomRoot<HTMLCanvasElement> {
        // This method is not called from a paint worklet rendering context,
        // so it's OK to panic if self.canvas is None.
        DomRoot::from_ref(self.canvas.as_ref().expect("No canvas."))
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

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-scale
    fn Scale(&self, x: f64, y: f64) {
        if !(x.is_finite() && y.is_finite()) {
            return;
        }

        let transform = self.canvas_state.borrow().state.borrow().transform;
        self.canvas_state.borrow().state.borrow_mut().transform =
            transform.pre_scale(x as f32, y as f32);
        self.update_transform()
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-rotate
    fn Rotate(&self, angle: f64) {
        if angle == 0.0 || !angle.is_finite() {
            return;
        }

        let (sin, cos) = (angle.sin(), angle.cos());
        let transform = self.canvas_state.borrow().state.borrow().transform;
        self.canvas_state.borrow().state.borrow_mut().transform = transform.pre_mul(
            &Transform2D::row_major(cos as f32, sin as f32, -sin as f32, cos as f32, 0.0, 0.0),
        );
        self.update_transform()
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-translate
    fn Translate(&self, x: f64, y: f64) {
        if !(x.is_finite() && y.is_finite()) {
            return;
        }

        let transform = self.canvas_state.borrow().state.borrow().transform;
        self.canvas_state.borrow().state.borrow_mut().transform =
            transform.pre_translate(vec2(x as f32, y as f32));
        self.update_transform()
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-transform
    fn Transform(&self, a: f64, b: f64, c: f64, d: f64, e: f64, f: f64) {
        if !(a.is_finite() &&
            b.is_finite() &&
            c.is_finite() &&
            d.is_finite() &&
            e.is_finite() &&
            f.is_finite())
        {
            return;
        }

        let transform = self.canvas_state.borrow().state.borrow().transform;
        self.canvas_state.borrow().state.borrow_mut().transform = transform.pre_mul(
            &Transform2D::row_major(a as f32, b as f32, c as f32, d as f32, e as f32, f as f32),
        );
        self.update_transform()
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-settransform
    fn SetTransform(&self, a: f64, b: f64, c: f64, d: f64, e: f64, f: f64) {
        if !(a.is_finite() &&
            b.is_finite() &&
            c.is_finite() &&
            d.is_finite() &&
            e.is_finite() &&
            f.is_finite())
        {
            return;
        }

        self.canvas_state.borrow().state.borrow_mut().transform =
            Transform2D::row_major(a as f32, b as f32, c as f32, d as f32, e as f32, f as f32);
        self.update_transform()
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-resettransform
    fn ResetTransform(&self) {
        self.canvas_state.borrow().state.borrow_mut().transform = Transform2D::identity();
        self.update_transform()
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

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-fillrect
    fn FillRect(&self, x: f64, y: f64, width: f64, height: f64) {
        self.canvas_state.borrow().FillRect(x, y, width, height);
        self.mark_as_dirty();
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-clearrect
    fn ClearRect(&self, x: f64, y: f64, width: f64, height: f64) {
        self.canvas_state.borrow().ClearRect(x, y, width, height);
        self.mark_as_dirty();
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-strokerect
    fn StrokeRect(&self, x: f64, y: f64, width: f64, height: f64) {
        self.canvas_state.borrow().StrokeRect(x, y, width, height);
        self.mark_as_dirty();
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-beginpath
    fn BeginPath(&self) {
        self.send_canvas_2d_msg(Canvas2dMsg::BeginPath);
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-closepath
    fn ClosePath(&self) {
        self.send_canvas_2d_msg(Canvas2dMsg::ClosePath);
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-fill
    fn Fill(&self, _: CanvasFillRule) {
        // TODO: Process fill rule
        self.send_canvas_2d_msg(Canvas2dMsg::Fill);
        self.mark_as_dirty();
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-stroke
    fn Stroke(&self) {
        self.send_canvas_2d_msg(Canvas2dMsg::Stroke);
        self.mark_as_dirty();
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-clip
    fn Clip(&self, _: CanvasFillRule) {
        // TODO: Process fill rule
        self.send_canvas_2d_msg(Canvas2dMsg::Clip);
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-ispointinpath
    fn IsPointInPath(&self, x: f64, y: f64, fill_rule: CanvasFillRule) -> bool {
        let fill_rule = match fill_rule {
            CanvasFillRule::Nonzero => FillRule::Nonzero,
            CanvasFillRule::Evenodd => FillRule::Evenodd,
        };
        let (sender, receiver) =
            profiled_ipc::channel::<bool>(self.global().time_profiler_chan().clone()).unwrap();
        self.send_canvas_2d_msg(Canvas2dMsg::IsPointInPath(x, y, fill_rule, sender));
        receiver.recv().unwrap()
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-filltext
    fn FillText(&self, text: DOMString, x: f64, y: f64, max_width: Option<f64>) {
        self.canvas_state.borrow().FillText(text,x,y,max_width)
        self.mark_as_dirty();
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-drawimage
    fn DrawImage(&self, image: CanvasImageSource, dx: f64, dy: f64) -> ErrorResult {
        if !(dx.is_finite() && dy.is_finite()) {
            return Ok(());
        }

        self.draw_image(image, 0f64, 0f64, None, None, dx, dy, None, None)
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
        if !(dx.is_finite() && dy.is_finite() && dw.is_finite() && dh.is_finite()) {
            return Ok(());
        }

        self.draw_image(image, 0f64, 0f64, None, None, dx, dy, Some(dw), Some(dh))
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
        if !(sx.is_finite() &&
            sy.is_finite() &&
            sw.is_finite() &&
            sh.is_finite() &&
            dx.is_finite() &&
            dy.is_finite() &&
            dw.is_finite() &&
            dh.is_finite())
        {
            return Ok(());
        }

        self.draw_image(
            image,
            sx,
            sy,
            Some(sw),
            Some(sh),
            dx,
            dy,
            Some(dw),
            Some(dh),
        )
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-moveto
    fn MoveTo(&self, x: f64, y: f64) {
        if !(x.is_finite() && y.is_finite()) {
            return;
        }
        self.send_canvas_2d_msg(Canvas2dMsg::MoveTo(Point2D::new(x as f32, y as f32)));
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-lineto
    fn LineTo(&self, x: f64, y: f64) {
        if !(x.is_finite() && y.is_finite()) {
            return;
        }
        self.send_canvas_2d_msg(Canvas2dMsg::LineTo(Point2D::new(x as f32, y as f32)));
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-rect
    fn Rect(&self, x: f64, y: f64, width: f64, height: f64) {
        if [x, y, width, height].iter().all(|val| val.is_finite()) {
            let rect = Rect::new(
                Point2D::new(x as f32, y as f32),
                Size2D::new(width as f32, height as f32),
            );
            self.send_canvas_2d_msg(Canvas2dMsg::Rect(rect));
        }
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-quadraticcurveto
    fn QuadraticCurveTo(&self, cpx: f64, cpy: f64, x: f64, y: f64) {
        if !(cpx.is_finite() && cpy.is_finite() && x.is_finite() && y.is_finite()) {
            return;
        }
        self.send_canvas_2d_msg(Canvas2dMsg::QuadraticCurveTo(
            Point2D::new(cpx as f32, cpy as f32),
            Point2D::new(x as f32, y as f32),
        ));
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-beziercurveto
    fn BezierCurveTo(&self, cp1x: f64, cp1y: f64, cp2x: f64, cp2y: f64, x: f64, y: f64) {
        if !(cp1x.is_finite() &&
            cp1y.is_finite() &&
            cp2x.is_finite() &&
            cp2y.is_finite() &&
            x.is_finite() &&
            y.is_finite())
        {
            return;
        }
        self.send_canvas_2d_msg(Canvas2dMsg::BezierCurveTo(
            Point2D::new(cp1x as f32, cp1y as f32),
            Point2D::new(cp2x as f32, cp2y as f32),
            Point2D::new(x as f32, y as f32),
        ));
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-arc
    fn Arc(&self, x: f64, y: f64, r: f64, start: f64, end: f64, ccw: bool) -> ErrorResult {
        if !([x, y, r, start, end].iter().all(|x| x.is_finite())) {
            return Ok(());
        }

        if r < 0.0 {
            return Err(Error::IndexSize);
        }

        self.send_canvas_2d_msg(Canvas2dMsg::Arc(
            Point2D::new(x as f32, y as f32),
            r as f32,
            start as f32,
            end as f32,
            ccw,
        ));
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

        self.send_canvas_2d_msg(Canvas2dMsg::ArcTo(
            Point2D::new(cp1x as f32, cp1y as f32),
            Point2D::new(cp2x as f32, cp2y as f32),
            r as f32,
        ));
        Ok(())
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
        if !([x, y, rx, ry, rotation, start, end]
            .iter()
            .all(|x| x.is_finite()))
        {
            return Ok(());
        }
        if rx < 0.0 || ry < 0.0 {
            return Err(Error::IndexSize);
        }

        self.send_canvas_2d_msg(Canvas2dMsg::Ellipse(
            Point2D::new(x as f32, y as f32),
            rx as f32,
            ry as f32,
            rotation as f32,
            start as f32,
            end as f32,
            ccw,
        ));
        Ok(())
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

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-createimagedata
    fn CreateImageData(&self, sw: i32, sh: i32) -> Fallible<DomRoot<ImageData>> {
        if sw == 0 || sh == 0 {
            return Err(Error::IndexSize);
        }
        ImageData::new(&self.global(), sw.abs() as u32, sh.abs() as u32, None)
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-createimagedata
    fn CreateImageData_(&self, imagedata: &ImageData) -> Fallible<DomRoot<ImageData>> {
        ImageData::new(&self.global(), imagedata.Width(), imagedata.Height(), None)
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-getimagedata
    fn GetImageData(&self, sx: i32, sy: i32, sw: i32, sh: i32) -> Fallible<DomRoot<ImageData>> {
        // FIXME(nox): There are many arithmetic operations here that can
        // overflow or underflow, this should probably be audited.

        if sw == 0 || sh == 0 {
            return Err(Error::IndexSize);
        }

        if !self.origin_is_clean() {
            return Err(Error::Security);
        }

        let (origin, size) = adjust_size_sign(Point2D::new(sx, sy), Size2D::new(sw, sh));
        // FIXME(nox): This is probably wrong when this is a context for an
        // offscreen canvas.
        let canvas_size = self
            .canvas
            .as_ref()
            .map_or(Size2D::zero(), |c| c.get_size());
        let read_rect = match pixels::clip(origin, size, canvas_size) {
            Some(rect) => rect,
            None => {
                // All the pixels are outside the canvas surface.
                return ImageData::new(&self.global(), size.width, size.height, None);
            },
        };

        ImageData::new(
            &self.global(),
            size.width,
            size.height,
            Some(self.get_rect(read_rect)),
        )
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-putimagedata
    fn PutImageData(&self, imagedata: &ImageData, dx: i32, dy: i32) {
        self.PutImageData_(
            imagedata,
            dx,
            dy,
            0,
            0,
            imagedata.Width() as i32,
            imagedata.Height() as i32,
        )
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
        // FIXME(nox): There are many arithmetic operations here that can
        // overflow or underflow, this should probably be audited.

        let imagedata_size = Size2D::new(imagedata.Width(), imagedata.Height());
        if imagedata_size.area() == 0 {
            return;
        }

        // Step 1.
        // Done later.

        // Step 2.
        // TODO: throw InvalidState if buffer is detached.

        // FIXME(nox): This is probably wrong when this is a context for an
        // offscreen canvas.
        let canvas_size = self
            .canvas
            .as_ref()
            .map_or(Size2D::zero(), |c| c.get_size());

        // Steps 3-6.
        let (src_origin, src_size) = adjust_size_sign(
            Point2D::new(dirty_x, dirty_y),
            Size2D::new(dirty_width, dirty_height),
        );
        let src_rect = match pixels::clip(src_origin, src_size, imagedata_size) {
            Some(rect) => rect,
            None => return,
        };
        let (dst_origin, _) = adjust_size_sign(
            Point2D::new(dirty_x.saturating_add(dx), dirty_y.saturating_add(dy)),
            Size2D::new(dirty_width, dirty_height),
        );
        // By clipping to the canvas surface, we avoid sending any pixel
        // that would fall outside it.
        let dst_rect = match pixels::clip(dst_origin, src_rect.size, canvas_size) {
            Some(rect) => rect,
            None => return,
        };

        // Step 7.
        let (sender, receiver) = ipc::bytes_channel().unwrap();
        let pixels = unsafe { &imagedata.get_rect(Rect::new(src_rect.origin, dst_rect.size)) };
        self.send_canvas_2d_msg(Canvas2dMsg::PutImageData(dst_rect, receiver));
        sender.send(pixels).unwrap();
        self.mark_as_dirty();
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

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-linewidth
    fn LineWidth(&self) -> f64 {
        self.canvas_state.borrow().state.borrow().line_width
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-linewidth
    fn SetLineWidth(&self, width: f64) {
        if !width.is_finite() || width <= 0.0 {
            return;
        }

        self.canvas_state.borrow().state.borrow_mut().line_width = width;
        self.send_canvas_2d_msg(Canvas2dMsg::SetLineWidth(width as f32))
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-linecap
    fn LineCap(&self) -> CanvasLineCap {
        match self.canvas_state.borrow().state.borrow().line_cap {
            LineCapStyle::Butt => CanvasLineCap::Butt,
            LineCapStyle::Round => CanvasLineCap::Round,
            LineCapStyle::Square => CanvasLineCap::Square,
        }
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-linecap
    fn SetLineCap(&self, cap: CanvasLineCap) {
        let line_cap = match cap {
            CanvasLineCap::Butt => LineCapStyle::Butt,
            CanvasLineCap::Round => LineCapStyle::Round,
            CanvasLineCap::Square => LineCapStyle::Square,
        };
        self.canvas_state.borrow().state.borrow_mut().line_cap = line_cap;
        self.send_canvas_2d_msg(Canvas2dMsg::SetLineCap(line_cap));
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-linejoin
    fn LineJoin(&self) -> CanvasLineJoin {
        match self.canvas_state.borrow().state.borrow().line_join {
            LineJoinStyle::Round => CanvasLineJoin::Round,
            LineJoinStyle::Bevel => CanvasLineJoin::Bevel,
            LineJoinStyle::Miter => CanvasLineJoin::Miter,
        }
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-linejoin
    fn SetLineJoin(&self, join: CanvasLineJoin) {
        let line_join = match join {
            CanvasLineJoin::Round => LineJoinStyle::Round,
            CanvasLineJoin::Bevel => LineJoinStyle::Bevel,
            CanvasLineJoin::Miter => LineJoinStyle::Miter,
        };
        self.canvas_state.borrow().state.borrow_mut().line_join = line_join;
        self.send_canvas_2d_msg(Canvas2dMsg::SetLineJoin(line_join));
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-miterlimit
    fn MiterLimit(&self) -> f64 {
        self.canvas_state.borrow().state.borrow().miter_limit
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-miterlimit
    fn SetMiterLimit(&self, limit: f64) {
        if !limit.is_finite() || limit <= 0.0 {
            return;
        }

        self.canvas_state.borrow().state.borrow_mut().miter_limit = limit;
        self.send_canvas_2d_msg(Canvas2dMsg::SetMiterLimit(limit as f32))
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
}

impl Drop for CanvasRenderingContext2D {
    fn drop(&mut self) {
        if let Err(err) = self
            .canvas_state
            .borrow()
            .ipc_renderer
            .send(CanvasMsg::Close(self.canvas_state.borrow().get_canvas_id()))
        {
            warn!("Could not close canvas: {}", err)
        }
    }
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

// Used by drawImage to determine if a source or destination rectangle is valid
// Origin coordinates and size cannot be negative. Size has to be greater than zero
fn is_rect_valid(rect: Rect<f64>) -> bool {
    rect.size.width > 0.0 && rect.size.height > 0.0
}

// https://html.spec.whatwg.org/multipage/#serialisation-of-a-color
fn serialize<W>(color: &RGBA, dest: &mut W) -> fmt::Result
where
    W: fmt::Write,
{
    let red = color.red;
    let green = color.green;
    let blue = color.blue;

    if color.alpha == 255 {
        write!(
            dest,
            "#{:x}{:x}{:x}{:x}{:x}{:x}",
            red >> 4,
            red & 0xF,
            green >> 4,
            green & 0xF,
            blue >> 4,
            blue & 0xF
        )
    } else {
        write!(
            dest,
            "rgba({}, {}, {}, {})",
            red,
            green,
            blue,
            color.alpha_f32()
        )
    }
}

fn adjust_size_sign(
    mut origin: Point2D<i32>,
    mut size: Size2D<i32>,
) -> (Point2D<i32>, Size2D<u32>) {
    if size.width < 0 {
        size.width = -size.width;
        origin.x = origin.x.saturating_sub(size.width);
    }
    if size.height < 0 {
        size.height = -size.height;
        origin.y = origin.y.saturating_sub(size.height);
    }
    (origin, size.to_u32())
}

/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::Cell;
use std::fmt;
use std::str::FromStr;
use std::sync::Arc;

use canvas_traits::canvas::{
    Canvas2dMsg, CanvasId, CanvasMsg, CompositionOrBlending, Direction, FillOrStrokeStyle,
    FillRule, LineCapStyle, LineJoinStyle, LinearGradientStyle, PathSegment, RadialGradientStyle,
    RepetitionStyle, TextAlign, TextBaseline, TextMetrics as CanvasTextMetrics,
};
use constellation_traits::ScriptToConstellationMessage;
use cssparser::color::clamp_unit_f32;
use cssparser::{Parser, ParserInput};
use euclid::default::{Point2D, Rect, Size2D, Transform2D};
use euclid::vec2;
use ipc_channel::ipc::{self, IpcSender};
use net_traits::image_cache::{ImageCache, ImageResponse};
use net_traits::request::CorsSettings;
use pixels::{PixelFormat, Snapshot, SnapshotAlphaMode, SnapshotPixelFormat};
use profile_traits::ipc as profiled_ipc;
use servo_url::{ImmutableOrigin, ServoUrl};
use style::color::{AbsoluteColor, ColorFlags, ColorSpace};
use style::context::QuirksMode;
use style::parser::ParserContext;
use style::properties::longhands::font_variant_caps::computed_value::T as FontVariantCaps;
use style::properties::style_structs::Font;
use style::stylesheets::{CssRuleType, Origin};
use style::values::computed::font::FontStyle;
use style::values::specified::color::Color;
use style_traits::values::ToCss;
use style_traits::{CssWriter, ParsingMode};
use url::Url;
use webrender_api::ImageKey;

use crate::canvas_context::{OffscreenRenderingContext, RenderingContext};
use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::CanvasRenderingContext2DBinding::{
    CanvasDirection, CanvasFillRule, CanvasImageSource, CanvasLineCap, CanvasLineJoin,
    CanvasTextAlign, CanvasTextBaseline, ImageDataMethods,
};
use crate::dom::bindings::codegen::UnionTypes::StringOrCanvasGradientOrCanvasPattern;
use crate::dom::bindings::error::{Error, ErrorResult, Fallible};
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::num::Finite;
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::bindings::str::DOMString;
use crate::dom::canvasgradient::{CanvasGradient, CanvasGradientStyle, ToFillOrStrokeStyle};
use crate::dom::canvaspattern::CanvasPattern;
use crate::dom::dommatrix::DOMMatrix;
use crate::dom::element::{Element, cors_setting_for_element};
use crate::dom::globalscope::GlobalScope;
use crate::dom::htmlcanvaselement::HTMLCanvasElement;
use crate::dom::htmlvideoelement::HTMLVideoElement;
use crate::dom::imagebitmap::ImageBitmap;
use crate::dom::imagedata::ImageData;
use crate::dom::node::{Node, NodeTraits};
use crate::dom::offscreencanvas::OffscreenCanvas;
use crate::dom::paintworkletglobalscope::PaintWorkletGlobalScope;
use crate::dom::textmetrics::TextMetrics;
use crate::script_runtime::CanGc;

#[cfg_attr(crown, crown::unrooted_must_root_lint::must_root)]
#[derive(Clone, JSTraceable, MallocSizeOf)]
#[allow(dead_code)]
pub(crate) enum CanvasFillOrStrokeStyle {
    Color(#[no_trace] AbsoluteColor),
    Gradient(Dom<CanvasGradient>),
    Pattern(Dom<CanvasPattern>),
}

impl CanvasFillOrStrokeStyle {
    fn to_fill_or_stroke_style(&self) -> FillOrStrokeStyle {
        match self {
            CanvasFillOrStrokeStyle::Color(rgba) => FillOrStrokeStyle::Color(*rgba),
            CanvasFillOrStrokeStyle::Gradient(gradient) => gradient.to_fill_or_stroke_style(),
            CanvasFillOrStrokeStyle::Pattern(pattern) => pattern.to_fill_or_stroke_style(),
        }
    }
}

#[cfg_attr(crown, crown::unrooted_must_root_lint::must_root)]
#[derive(Clone, JSTraceable, MallocSizeOf)]
pub(crate) struct CanvasContextState {
    global_alpha: f64,
    #[no_trace]
    global_composition: CompositionOrBlending,
    image_smoothing_enabled: bool,
    fill_style: CanvasFillOrStrokeStyle,
    stroke_style: CanvasFillOrStrokeStyle,
    line_width: f64,
    #[no_trace]
    line_cap: LineCapStyle,
    #[no_trace]
    line_join: LineJoinStyle,
    miter_limit: f64,
    line_dash: Vec<f64>,
    line_dash_offset: f64,
    #[no_trace]
    transform: Transform2D<f32>,
    shadow_offset_x: f64,
    shadow_offset_y: f64,
    shadow_blur: f64,
    #[no_trace]
    shadow_color: AbsoluteColor,
    #[no_trace]
    font_style: Option<Font>,
    #[no_trace]
    text_align: TextAlign,
    #[no_trace]
    text_baseline: TextBaseline,
    #[no_trace]
    direction: Direction,
}

impl CanvasContextState {
    const DEFAULT_FONT_STYLE: &'static str = "10px sans-serif";

    pub(crate) fn new() -> CanvasContextState {
        CanvasContextState {
            global_alpha: 1.0,
            global_composition: CompositionOrBlending::default(),
            image_smoothing_enabled: true,
            fill_style: CanvasFillOrStrokeStyle::Color(AbsoluteColor::BLACK),
            stroke_style: CanvasFillOrStrokeStyle::Color(AbsoluteColor::BLACK),
            line_width: 1.0,
            line_cap: LineCapStyle::Butt,
            line_join: LineJoinStyle::Miter,
            miter_limit: 10.0,
            transform: Transform2D::identity(),
            shadow_offset_x: 0.0,
            shadow_offset_y: 0.0,
            shadow_blur: 0.0,
            shadow_color: AbsoluteColor::TRANSPARENT_BLACK,
            font_style: None,
            text_align: Default::default(),
            text_baseline: Default::default(),
            direction: Default::default(),
            line_dash: Vec::new(),
            line_dash_offset: 0.0,
        }
    }
}

#[cfg_attr(crown, crown::unrooted_must_root_lint::must_root)]
#[derive(JSTraceable, MallocSizeOf)]
pub(crate) struct CanvasState {
    #[ignore_malloc_size_of = "Defined in ipc-channel"]
    #[no_trace]
    ipc_renderer: IpcSender<CanvasMsg>,
    #[no_trace]
    canvas_id: CanvasId,
    #[no_trace]
    image_key: ImageKey,
    #[no_trace]
    size: Cell<Size2D<u64>>,
    state: DomRefCell<CanvasContextState>,
    origin_clean: Cell<bool>,
    #[ignore_malloc_size_of = "Arc"]
    #[no_trace]
    image_cache: Arc<dyn ImageCache>,
    /// The base URL for resolving CSS image URL values.
    /// Needed because of <https://github.com/servo/servo/issues/17625>
    #[no_trace]
    base_url: ServoUrl,
    #[no_trace]
    origin: ImmutableOrigin,
    /// Any missing image URLs.
    #[no_trace]
    missing_image_urls: DomRefCell<Vec<ServoUrl>>,
    saved_states: DomRefCell<Vec<CanvasContextState>>,
}

impl CanvasState {
    pub(crate) fn new(global: &GlobalScope, size: Size2D<u64>) -> CanvasState {
        debug!("Creating new canvas rendering context.");
        let (sender, receiver) =
            profiled_ipc::channel(global.time_profiler_chan().clone()).unwrap();
        let script_to_constellation_chan = global.script_to_constellation_chan();
        debug!("Asking constellation to create new canvas thread.");
        let size = adjust_canvas_size(size);
        script_to_constellation_chan
            .send(ScriptToConstellationMessage::CreateCanvasPaintThread(
                size, sender,
            ))
            .unwrap();
        let (ipc_renderer, canvas_id, image_key) = receiver.recv().unwrap();
        debug!("Done.");
        // Worklets always receive a unique origin. This messes with fetching
        // cached images in the case of paint worklets, since the image cache
        // is keyed on the origin requesting the image data.
        let origin = if global.is::<PaintWorkletGlobalScope>() {
            global.api_base_url().origin()
        } else {
            global.origin().immutable().clone()
        };
        CanvasState {
            ipc_renderer,
            canvas_id,
            size: Cell::new(size),
            state: DomRefCell::new(CanvasContextState::new()),
            origin_clean: Cell::new(true),
            image_cache: global.image_cache(),
            base_url: global.api_base_url(),
            missing_image_urls: DomRefCell::new(Vec::new()),
            saved_states: DomRefCell::new(Vec::new()),
            image_key,
            origin,
        }
    }

    pub(crate) fn get_ipc_renderer(&self) -> &IpcSender<CanvasMsg> {
        &self.ipc_renderer
    }

    pub(crate) fn image_key(&self) -> ImageKey {
        self.image_key
    }

    pub(crate) fn get_missing_image_urls(&self) -> &DomRefCell<Vec<ServoUrl>> {
        &self.missing_image_urls
    }

    pub(crate) fn get_canvas_id(&self) -> CanvasId {
        self.canvas_id
    }

    pub(crate) fn is_paintable(&self) -> bool {
        !self.size.get().is_empty()
    }

    pub(crate) fn send_canvas_2d_msg(&self, msg: Canvas2dMsg) {
        if !self.is_paintable() {
            return;
        }

        self.ipc_renderer
            .send(CanvasMsg::Canvas2d(msg, self.get_canvas_id()))
            .unwrap()
    }

    /// Updates WR image and blocks on completion
    pub(crate) fn update_rendering(&self) {
        if !self.is_paintable() {
            return;
        }

        let (sender, receiver) = ipc::channel().unwrap();
        self.ipc_renderer
            .send(CanvasMsg::Canvas2d(
                Canvas2dMsg::UpdateImage(sender),
                self.canvas_id,
            ))
            .unwrap();
        receiver.recv().unwrap();
    }

    /// <https://html.spec.whatwg.org/multipage/#concept-canvas-set-bitmap-dimensions>
    pub(crate) fn set_bitmap_dimensions(&self, size: Size2D<u64>) {
        self.reset_to_initial_state();

        self.size.replace(adjust_canvas_size(size));

        self.ipc_renderer
            .send(CanvasMsg::Recreate(
                Some(self.size.get()),
                self.get_canvas_id(),
            ))
            .unwrap();
    }

    pub(crate) fn reset(&self) {
        self.reset_to_initial_state();

        if !self.is_paintable() {
            return;
        }

        self.ipc_renderer
            .send(CanvasMsg::Recreate(None, self.get_canvas_id()))
            .unwrap();
    }

    pub(crate) fn reset_to_initial_state(&self) {
        self.saved_states.borrow_mut().clear();
        *self.state.borrow_mut() = CanvasContextState::new();
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

    pub(crate) fn origin_is_clean(&self) -> bool {
        self.origin_clean.get()
    }

    fn set_origin_unclean(&self) {
        self.origin_clean.set(false)
    }

    /// <https://html.spec.whatwg.org/multipage/#the-image-argument-is-not-origin-clean>
    fn is_origin_clean(&self, source: CanvasImageSource) -> bool {
        match source {
            CanvasImageSource::HTMLImageElement(image) => {
                image.same_origin(GlobalScope::entry().origin())
            },
            CanvasImageSource::HTMLVideoElement(video) => video.origin_is_clean(),
            CanvasImageSource::HTMLCanvasElement(canvas) => canvas.origin_is_clean(),
            CanvasImageSource::ImageBitmap(bitmap) => bitmap.origin_is_clean(),
            CanvasImageSource::OffscreenCanvas(canvas) => canvas.origin_is_clean(),
            CanvasImageSource::CSSStyleValue(_) => true,
        }
    }

    fn fetch_image_data(
        &self,
        url: ServoUrl,
        cors_setting: Option<CorsSettings>,
    ) -> Option<Snapshot> {
        let img = match self.request_image_from_cache(url, cors_setting) {
            ImageResponse::Loaded(image, _) => {
                if let Some(image) = image.as_raster_image() {
                    image
                } else {
                    // TODO: https://html.spec.whatwg.org/multipage/#dom-context-2d-drawimage
                    warn!("Vector images are not supported as image source in canvas2d");
                    return None;
                }
            },
            ImageResponse::PlaceholderLoaded(_, _) |
            ImageResponse::None |
            ImageResponse::MetadataLoaded(_) => {
                return None;
            },
        };

        let size = Size2D::new(img.metadata.width, img.metadata.height);
        let format = match img.format {
            PixelFormat::BGRA8 => SnapshotPixelFormat::BGRA,
            PixelFormat::RGBA8 => SnapshotPixelFormat::RGBA,
            pixel_format => unimplemented!("unsupported pixel format ({:?})", pixel_format),
        };
        let alpha_mode = SnapshotAlphaMode::Transparent {
            premultiplied: false,
        };

        Some(Snapshot::from_vec(
            size.cast(),
            format,
            alpha_mode,
            img.first_frame().bytes.to_vec(),
        ))
    }

    fn request_image_from_cache(
        &self,
        url: ServoUrl,
        cors_setting: Option<CorsSettings>,
    ) -> ImageResponse {
        match self
            .image_cache
            .get_image(url.clone(), self.origin.clone(), cors_setting)
        {
            Some(image) => ImageResponse::Loaded(image, url),
            None => {
                // Rather annoyingly, we get the same response back from
                // A load which really failed and from a load which hasn't started yet.
                self.missing_image_urls.borrow_mut().push(url);
                ImageResponse::None
            },
        }
    }

    pub(crate) fn get_rect(&self, canvas_size: Size2D<u32>, rect: Rect<u32>) -> Vec<u8> {
        assert!(self.origin_is_clean());
        assert!(Rect::from_size(canvas_size).contains_rect(&rect));

        let (sender, receiver) = ipc::channel().unwrap();
        self.send_canvas_2d_msg(Canvas2dMsg::GetImageData(rect, canvas_size, sender));
        let mut snapshot = receiver.recv().unwrap().to_owned();
        snapshot.transform(
            SnapshotAlphaMode::Transparent {
                premultiplied: false,
            },
            SnapshotPixelFormat::RGBA,
        );
        snapshot.to_vec()
    }

    ///
    /// drawImage coordinates explained
    ///
    /// ```
    ///  Source Image      Destination Canvas
    /// +-------------+     +-------------+
    /// |             |     |             |
    /// |(sx,sy)      |     |(dx,dy)      |
    /// |   +----+    |     |   +----+    |
    /// |   |    |    |     |   |    |    |
    /// |   |    |sh  |---->|   |    |dh  |
    /// |   |    |    |     |   |    |    |
    /// |   +----+    |     |   +----+    |
    /// |     sw      |     |     dw      |
    /// |             |     |             |
    /// +-------------+     +-------------+
    /// ```
    ///
    /// The rectangle (sx, sy, sw, sh) from the source image
    /// is copied on the rectangle (dx, dy, dh, dw) of the destination canvas
    ///
    /// <https://html.spec.whatwg.org/multipage/#dom-context-2d-drawimage>
    #[allow(clippy::too_many_arguments)]
    fn draw_image_internal(
        &self,
        htmlcanvas: Option<&HTMLCanvasElement>,
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
        if !self.is_paintable() {
            return Ok(());
        }

        let result = match image {
            CanvasImageSource::HTMLVideoElement(ref video) => {
                // <https://html.spec.whatwg.org/multipage/#check-the-usability-of-the-image-argument>
                // Step 2. Let usability be the result of checking the usability of image.
                // Step 3. If usability is bad, then return (without drawing anything).
                if !video.is_usable() {
                    return Ok(());
                }

                self.draw_html_video_element(video, htmlcanvas, sx, sy, sw, sh, dx, dy, dw, dh);
                Ok(())
            },
            CanvasImageSource::HTMLCanvasElement(ref canvas) => {
                // <https://html.spec.whatwg.org/multipage/#check-the-usability-of-the-image-argument>
                if canvas.get_size().is_empty() {
                    return Err(Error::InvalidState);
                }

                self.draw_html_canvas_element(canvas, htmlcanvas, sx, sy, sw, sh, dx, dy, dw, dh)
            },
            CanvasImageSource::ImageBitmap(ref bitmap) => {
                // <https://html.spec.whatwg.org/multipage/#check-the-usability-of-the-image-argument>
                if bitmap.is_detached() {
                    return Err(Error::InvalidState);
                }

                self.draw_image_bitmap(bitmap, htmlcanvas, sx, sy, sw, sh, dx, dy, dw, dh);
                Ok(())
            },
            CanvasImageSource::OffscreenCanvas(ref canvas) => {
                // <https://html.spec.whatwg.org/multipage/#check-the-usability-of-the-image-argument>
                if canvas.get_size().is_empty() {
                    return Err(Error::InvalidState);
                }

                self.draw_offscreen_canvas(canvas, htmlcanvas, sx, sy, sw, sh, dx, dy, dw, dh)
            },
            CanvasImageSource::HTMLImageElement(ref image) => {
                // https://html.spec.whatwg.org/multipage/#drawing-images
                // 2. Let usability be the result of checking the usability of image.
                // 3. If usability is bad, then return (without drawing anything).
                if !image.is_usable()? {
                    return Ok(());
                }

                // TODO(pylbrecht): is it possible for image.get_url() to return None after the usability check?
                // https://html.spec.whatwg.org/multipage/#img-error
                // If the image argument is an HTMLImageElement object that is in the broken state,
                // then throw an InvalidStateError exception
                let url = image.get_url().ok_or(Error::InvalidState)?;
                let cors_setting = cors_setting_for_element(image.upcast());
                self.fetch_and_draw_image_data(
                    htmlcanvas,
                    url,
                    cors_setting,
                    sx,
                    sy,
                    sw,
                    sh,
                    dx,
                    dy,
                    dw,
                    dh,
                )
            },
            CanvasImageSource::CSSStyleValue(ref value) => {
                let url = value
                    .get_url(self.base_url.clone())
                    .ok_or(Error::InvalidState)?;
                self.fetch_and_draw_image_data(
                    htmlcanvas, url, None, sx, sy, sw, sh, dx, dy, dw, dh,
                )
            },
        };

        if result.is_ok() && !self.is_origin_clean(image) {
            self.set_origin_unclean()
        }
        result
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-context-2d-drawimage>
    #[allow(clippy::too_many_arguments)]
    fn draw_html_video_element(
        &self,
        video: &HTMLVideoElement,
        canvas: Option<&HTMLCanvasElement>,
        sx: f64,
        sy: f64,
        sw: Option<f64>,
        sh: Option<f64>,
        dx: f64,
        dy: f64,
        dw: Option<f64>,
        dh: Option<f64>,
    ) {
        let Some(snapshot) = video.get_current_frame_data() else {
            return;
        };

        // Step 4. Establish the source and destination rectangles.
        let video_size = snapshot.size();
        let dw = dw.unwrap_or(video_size.width as f64);
        let dh = dh.unwrap_or(video_size.height as f64);
        let sw = sw.unwrap_or(video_size.width as f64);
        let sh = sh.unwrap_or(video_size.height as f64);

        let (source_rect, dest_rect) =
            self.adjust_source_dest_rects(video_size, sx, sy, sw, sh, dx, dy, dw, dh);

        // Step 5. If one of the sw or sh arguments is zero, then return. Nothing is painted.
        if !is_rect_valid(source_rect) || !is_rect_valid(dest_rect) {
            return;
        }

        let smoothing_enabled = self.state.borrow().image_smoothing_enabled;

        self.send_canvas_2d_msg(Canvas2dMsg::DrawImage(
            snapshot.as_ipc(),
            dest_rect,
            source_rect,
            smoothing_enabled,
        ));

        self.mark_as_dirty(canvas);
    }

    #[allow(clippy::too_many_arguments)]
    fn draw_offscreen_canvas(
        &self,
        canvas: &OffscreenCanvas,
        htmlcanvas: Option<&HTMLCanvasElement>,
        sx: f64,
        sy: f64,
        sw: Option<f64>,
        sh: Option<f64>,
        dx: f64,
        dy: f64,
        dw: Option<f64>,
        dh: Option<f64>,
    ) -> ErrorResult {
        let canvas_size = canvas.get_size();
        let dw = dw.unwrap_or(canvas_size.width as f64);
        let dh = dh.unwrap_or(canvas_size.height as f64);
        let sw = sw.unwrap_or(canvas_size.width as f64);
        let sh = sh.unwrap_or(canvas_size.height as f64);

        let image_size = Size2D::new(canvas_size.width, canvas_size.height);
        // 2. Establish the source and destination rectangles
        let (source_rect, dest_rect) =
            self.adjust_source_dest_rects(image_size, sx, sy, sw, sh, dx, dy, dw, dh);

        if !is_rect_valid(source_rect) || !is_rect_valid(dest_rect) {
            return Ok(());
        }

        let smoothing_enabled = self.state.borrow().image_smoothing_enabled;

        if let Some(context) = canvas.context() {
            match *context {
                OffscreenRenderingContext::Context2d(ref context) => {
                    context.send_canvas_2d_msg(Canvas2dMsg::DrawImageInOther(
                        self.get_canvas_id(),
                        image_size,
                        dest_rect,
                        source_rect,
                        smoothing_enabled,
                    ));
                },
            }
        } else {
            self.send_canvas_2d_msg(Canvas2dMsg::DrawEmptyImage(
                image_size,
                dest_rect,
                source_rect,
            ));
        }

        self.mark_as_dirty(htmlcanvas);
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    fn draw_html_canvas_element(
        &self,
        canvas: &HTMLCanvasElement,             // source canvas
        htmlcanvas: Option<&HTMLCanvasElement>, // destination canvas
        sx: f64,
        sy: f64,
        sw: Option<f64>,
        sh: Option<f64>,
        dx: f64,
        dy: f64,
        dw: Option<f64>,
        dh: Option<f64>,
    ) -> ErrorResult {
        let canvas_size = canvas.get_size();
        let dw = dw.unwrap_or(canvas_size.width as f64);
        let dh = dh.unwrap_or(canvas_size.height as f64);
        let sw = sw.unwrap_or(canvas_size.width as f64);
        let sh = sh.unwrap_or(canvas_size.height as f64);

        let image_size = Size2D::new(canvas_size.width, canvas_size.height);
        // 2. Establish the source and destination rectangles
        let (source_rect, dest_rect) =
            self.adjust_source_dest_rects(image_size, sx, sy, sw, sh, dx, dy, dw, dh);

        if !is_rect_valid(source_rect) || !is_rect_valid(dest_rect) {
            return Ok(());
        }

        let smoothing_enabled = self.state.borrow().image_smoothing_enabled;

        if let Some(context) = canvas.context() {
            match *context {
                RenderingContext::Context2d(ref context) => {
                    context.send_canvas_2d_msg(Canvas2dMsg::DrawImageInOther(
                        self.get_canvas_id(),
                        image_size,
                        dest_rect,
                        source_rect,
                        smoothing_enabled,
                    ));
                },
                RenderingContext::Placeholder(ref context) => {
                    let Some(context) = context.context() else {
                        return Err(Error::InvalidState);
                    };
                    match *context {
                        OffscreenRenderingContext::Context2d(ref context) => context
                            .send_canvas_2d_msg(Canvas2dMsg::DrawImageInOther(
                                self.get_canvas_id(),
                                image_size,
                                dest_rect,
                                source_rect,
                                smoothing_enabled,
                            )),
                    }
                },
                _ => return Err(Error::InvalidState),
            }
        } else {
            self.send_canvas_2d_msg(Canvas2dMsg::DrawEmptyImage(
                image_size,
                dest_rect,
                source_rect,
            ));
        }

        self.mark_as_dirty(htmlcanvas);
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    fn fetch_and_draw_image_data(
        &self,
        canvas: Option<&HTMLCanvasElement>,
        url: ServoUrl,
        cors_setting: Option<CorsSettings>,
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
        let snapshot = self
            .fetch_image_data(url, cors_setting)
            .ok_or(Error::InvalidState)?;
        let image_size = snapshot.size();

        let dw = dw.unwrap_or(image_size.width as f64);
        let dh = dh.unwrap_or(image_size.height as f64);
        let sw = sw.unwrap_or(image_size.width as f64);
        let sh = sh.unwrap_or(image_size.height as f64);

        // Establish the source and destination rectangles
        let (source_rect, dest_rect) =
            self.adjust_source_dest_rects(image_size, sx, sy, sw, sh, dx, dy, dw, dh);

        if !is_rect_valid(source_rect) || !is_rect_valid(dest_rect) {
            return Ok(());
        }

        let smoothing_enabled = self.state.borrow().image_smoothing_enabled;
        self.send_canvas_2d_msg(Canvas2dMsg::DrawImage(
            snapshot.as_ipc(),
            dest_rect,
            source_rect,
            smoothing_enabled,
        ));
        self.mark_as_dirty(canvas);
        Ok(())
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-context-2d-drawimage>
    #[allow(clippy::too_many_arguments)]
    fn draw_image_bitmap(
        &self,
        bitmap: &ImageBitmap,
        canvas: Option<&HTMLCanvasElement>,
        sx: f64,
        sy: f64,
        sw: Option<f64>,
        sh: Option<f64>,
        dx: f64,
        dy: f64,
        dw: Option<f64>,
        dh: Option<f64>,
    ) {
        let Some(snapshot) = bitmap.bitmap_data().clone() else {
            return;
        };

        // Step 4. Establish the source and destination rectangles.
        let bitmap_size = snapshot.size();
        let dw = dw.unwrap_or(bitmap_size.width as f64);
        let dh = dh.unwrap_or(bitmap_size.height as f64);
        let sw = sw.unwrap_or(bitmap_size.width as f64);
        let sh = sh.unwrap_or(bitmap_size.height as f64);

        let (source_rect, dest_rect) =
            self.adjust_source_dest_rects(bitmap_size, sx, sy, sw, sh, dx, dy, dw, dh);

        // Step 5. If one of the sw or sh arguments is zero, then return. Nothing is painted.
        if !is_rect_valid(source_rect) || !is_rect_valid(dest_rect) {
            return;
        }

        let smoothing_enabled = self.state.borrow().image_smoothing_enabled;

        self.send_canvas_2d_msg(Canvas2dMsg::DrawImage(
            snapshot.as_ipc(),
            dest_rect,
            source_rect,
            smoothing_enabled,
        ));

        self.mark_as_dirty(canvas);
    }

    pub(crate) fn mark_as_dirty(&self, canvas: Option<&HTMLCanvasElement>) {
        if let Some(canvas) = canvas {
            canvas.mark_as_dirty();
        }
    }

    /// It is used by DrawImage to calculate the size of the source and destination rectangles based
    /// on the drawImage call arguments
    /// source rectangle = area of the original image to be copied
    /// destination rectangle = area of the destination canvas where the source image is going to be drawn
    #[allow(clippy::too_many_arguments)]
    fn adjust_source_dest_rects(
        &self,
        image_size: Size2D<u32>,
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
            Point2D::zero(),
            Size2D::new(image_size.width, image_size.height),
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
            .intersection(&image_rect.to_f64())
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

    fn update_transform(&self) {
        self.send_canvas_2d_msg(Canvas2dMsg::SetTransform(self.state.borrow().transform))
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-fillrect
    pub(crate) fn fill_rect(&self, x: f64, y: f64, width: f64, height: f64) {
        if let Some(rect) = self.create_drawable_rect(x, y, width, height) {
            let style = self.state.borrow().fill_style.to_fill_or_stroke_style();
            self.send_canvas_2d_msg(Canvas2dMsg::FillRect(rect, style));
        }
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-clearrect
    pub(crate) fn clear_rect(&self, x: f64, y: f64, width: f64, height: f64) {
        if let Some(rect) = self.create_drawable_rect(x, y, width, height) {
            self.send_canvas_2d_msg(Canvas2dMsg::ClearRect(rect));
        }
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-strokerect
    pub(crate) fn stroke_rect(&self, x: f64, y: f64, width: f64, height: f64) {
        if let Some(rect) = self.create_drawable_rect(x, y, width, height) {
            let style = self.state.borrow().stroke_style.to_fill_or_stroke_style();
            self.send_canvas_2d_msg(Canvas2dMsg::StrokeRect(rect, style));
        }
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-shadowoffsetx
    pub(crate) fn shadow_offset_x(&self) -> f64 {
        self.state.borrow().shadow_offset_x
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-shadowoffsetx
    pub(crate) fn set_shadow_offset_x(&self, value: f64) {
        if !value.is_finite() || value == self.state.borrow().shadow_offset_x {
            return;
        }
        self.state.borrow_mut().shadow_offset_x = value;
        self.send_canvas_2d_msg(Canvas2dMsg::SetShadowOffsetX(value))
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-shadowoffsety
    pub(crate) fn shadow_offset_y(&self) -> f64 {
        self.state.borrow().shadow_offset_y
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-shadowoffsety
    pub(crate) fn set_shadow_offset_y(&self, value: f64) {
        if !value.is_finite() || value == self.state.borrow().shadow_offset_y {
            return;
        }
        self.state.borrow_mut().shadow_offset_y = value;
        self.send_canvas_2d_msg(Canvas2dMsg::SetShadowOffsetY(value))
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-shadowblur
    pub(crate) fn shadow_blur(&self) -> f64 {
        self.state.borrow().shadow_blur
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-shadowblur
    pub(crate) fn set_shadow_blur(&self, value: f64) {
        if !value.is_finite() || value < 0f64 || value == self.state.borrow().shadow_blur {
            return;
        }
        self.state.borrow_mut().shadow_blur = value;
        self.send_canvas_2d_msg(Canvas2dMsg::SetShadowBlur(value))
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-shadowcolor
    pub(crate) fn shadow_color(&self) -> DOMString {
        let mut result = String::new();
        serialize(&self.state.borrow().shadow_color, &mut result).unwrap();
        DOMString::from(result)
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-shadowcolor
    pub(crate) fn set_shadow_color(
        &self,
        canvas: Option<&HTMLCanvasElement>,
        value: DOMString,
        can_gc: CanGc,
    ) {
        if let Ok(rgba) = parse_color(canvas, &value, can_gc) {
            self.state.borrow_mut().shadow_color = rgba;
            self.send_canvas_2d_msg(Canvas2dMsg::SetShadowColor(rgba))
        }
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-strokestyle
    pub(crate) fn stroke_style(&self) -> StringOrCanvasGradientOrCanvasPattern {
        match self.state.borrow().stroke_style {
            CanvasFillOrStrokeStyle::Color(ref rgba) => {
                let mut result = String::new();
                serialize(rgba, &mut result).unwrap();
                StringOrCanvasGradientOrCanvasPattern::String(DOMString::from(result))
            },
            CanvasFillOrStrokeStyle::Gradient(ref gradient) => {
                StringOrCanvasGradientOrCanvasPattern::CanvasGradient(DomRoot::from_ref(gradient))
            },
            CanvasFillOrStrokeStyle::Pattern(ref pattern) => {
                StringOrCanvasGradientOrCanvasPattern::CanvasPattern(DomRoot::from_ref(pattern))
            },
        }
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-strokestyle
    pub(crate) fn set_stroke_style(
        &self,
        canvas: Option<&HTMLCanvasElement>,
        value: StringOrCanvasGradientOrCanvasPattern,
        can_gc: CanGc,
    ) {
        match value {
            StringOrCanvasGradientOrCanvasPattern::String(string) => {
                if let Ok(rgba) = parse_color(canvas, &string, can_gc) {
                    self.state.borrow_mut().stroke_style = CanvasFillOrStrokeStyle::Color(rgba);
                }
            },
            StringOrCanvasGradientOrCanvasPattern::CanvasGradient(gradient) => {
                self.state.borrow_mut().stroke_style =
                    CanvasFillOrStrokeStyle::Gradient(Dom::from_ref(&*gradient));
            },
            StringOrCanvasGradientOrCanvasPattern::CanvasPattern(pattern) => {
                self.state.borrow_mut().stroke_style =
                    CanvasFillOrStrokeStyle::Pattern(Dom::from_ref(&*pattern));
                if !pattern.origin_is_clean() {
                    self.set_origin_unclean();
                }
            },
        }
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-strokestyle
    pub(crate) fn fill_style(&self) -> StringOrCanvasGradientOrCanvasPattern {
        match self.state.borrow().fill_style {
            CanvasFillOrStrokeStyle::Color(ref rgba) => {
                let mut result = String::new();
                serialize(rgba, &mut result).unwrap();
                StringOrCanvasGradientOrCanvasPattern::String(DOMString::from(result))
            },
            CanvasFillOrStrokeStyle::Gradient(ref gradient) => {
                StringOrCanvasGradientOrCanvasPattern::CanvasGradient(DomRoot::from_ref(gradient))
            },
            CanvasFillOrStrokeStyle::Pattern(ref pattern) => {
                StringOrCanvasGradientOrCanvasPattern::CanvasPattern(DomRoot::from_ref(pattern))
            },
        }
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-strokestyle
    pub(crate) fn set_fill_style(
        &self,
        canvas: Option<&HTMLCanvasElement>,
        value: StringOrCanvasGradientOrCanvasPattern,
        can_gc: CanGc,
    ) {
        match value {
            StringOrCanvasGradientOrCanvasPattern::String(string) => {
                if let Ok(rgba) = parse_color(canvas, &string, can_gc) {
                    self.state.borrow_mut().fill_style = CanvasFillOrStrokeStyle::Color(rgba);
                }
            },
            StringOrCanvasGradientOrCanvasPattern::CanvasGradient(gradient) => {
                self.state.borrow_mut().fill_style =
                    CanvasFillOrStrokeStyle::Gradient(Dom::from_ref(&*gradient));
            },
            StringOrCanvasGradientOrCanvasPattern::CanvasPattern(pattern) => {
                self.state.borrow_mut().fill_style =
                    CanvasFillOrStrokeStyle::Pattern(Dom::from_ref(&*pattern));
                if !pattern.origin_is_clean() {
                    self.set_origin_unclean();
                }
            },
        }
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-createlineargradient
    pub(crate) fn create_linear_gradient(
        &self,
        global: &GlobalScope,
        x0: Finite<f64>,
        y0: Finite<f64>,
        x1: Finite<f64>,
        y1: Finite<f64>,
        can_gc: CanGc,
    ) -> DomRoot<CanvasGradient> {
        CanvasGradient::new(
            global,
            CanvasGradientStyle::Linear(LinearGradientStyle::new(*x0, *y0, *x1, *y1, Vec::new())),
            can_gc,
        )
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-context-2d-createradialgradient>
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn create_radial_gradient(
        &self,
        global: &GlobalScope,
        x0: Finite<f64>,
        y0: Finite<f64>,
        r0: Finite<f64>,
        x1: Finite<f64>,
        y1: Finite<f64>,
        r1: Finite<f64>,
        can_gc: CanGc,
    ) -> Fallible<DomRoot<CanvasGradient>> {
        if *r0 < 0. || *r1 < 0. {
            return Err(Error::IndexSize);
        }

        Ok(CanvasGradient::new(
            global,
            CanvasGradientStyle::Radial(RadialGradientStyle::new(
                *x0,
                *y0,
                *r0,
                *x1,
                *y1,
                *r1,
                Vec::new(),
            )),
            can_gc,
        ))
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-context-2d-createpattern>
    pub(crate) fn create_pattern(
        &self,
        global: &GlobalScope,
        image: CanvasImageSource,
        mut repetition: DOMString,
        can_gc: CanGc,
    ) -> Fallible<Option<DomRoot<CanvasPattern>>> {
        let snapshot = match image {
            CanvasImageSource::HTMLImageElement(ref image) => {
                // <https://html.spec.whatwg.org/multipage/#check-the-usability-of-the-image-argument>
                if !image.is_usable()? {
                    return Ok(None);
                }

                image
                    .get_url()
                    .and_then(|url| {
                        self.fetch_image_data(url, cors_setting_for_element(image.upcast()))
                    })
                    .ok_or(Error::InvalidState)?
            },
            CanvasImageSource::HTMLVideoElement(ref video) => {
                // <https://html.spec.whatwg.org/multipage/#check-the-usability-of-the-image-argument>
                if !video.is_usable() {
                    return Ok(None);
                }

                video.get_current_frame_data().ok_or(Error::InvalidState)?
            },
            CanvasImageSource::HTMLCanvasElement(ref canvas) => {
                // <https://html.spec.whatwg.org/multipage/#check-the-usability-of-the-image-argument>
                if canvas.get_size().is_empty() {
                    return Err(Error::InvalidState);
                }

                canvas.get_image_data().ok_or(Error::InvalidState)?
            },
            CanvasImageSource::ImageBitmap(ref bitmap) => {
                // <https://html.spec.whatwg.org/multipage/#check-the-usability-of-the-image-argument>
                if bitmap.is_detached() {
                    return Err(Error::InvalidState);
                }

                bitmap.bitmap_data().clone().ok_or(Error::InvalidState)?
            },
            CanvasImageSource::OffscreenCanvas(ref canvas) => {
                // <https://html.spec.whatwg.org/multipage/#check-the-usability-of-the-image-argument>
                if canvas.get_size().is_empty() {
                    return Err(Error::InvalidState);
                }

                canvas.get_image_data().ok_or(Error::InvalidState)?
            },
            CanvasImageSource::CSSStyleValue(ref value) => value
                .get_url(self.base_url.clone())
                .and_then(|url| self.fetch_image_data(url, None))
                .ok_or(Error::InvalidState)?,
        };

        if repetition.is_empty() {
            repetition.push_str("repeat");
        }

        if let Ok(rep) = RepetitionStyle::from_str(&repetition) {
            let size = snapshot.size();
            Ok(Some(CanvasPattern::new(
                global,
                snapshot.to_vec(),
                size.cast(),
                rep,
                self.is_origin_clean(image),
                can_gc,
            )))
        } else {
            Err(Error::Syntax)
        }
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-save
    pub(crate) fn save(&self) {
        self.saved_states
            .borrow_mut()
            .push(self.state.borrow().clone());
        self.send_canvas_2d_msg(Canvas2dMsg::SaveContext);
    }

    #[cfg_attr(crown, allow(crown::unrooted_must_root))]
    // https://html.spec.whatwg.org/multipage/#dom-context-2d-restore
    pub(crate) fn restore(&self) {
        let mut saved_states = self.saved_states.borrow_mut();
        if let Some(state) = saved_states.pop() {
            self.state.borrow_mut().clone_from(&state);
            self.send_canvas_2d_msg(Canvas2dMsg::RestoreContext);
        }
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-globalalpha
    pub(crate) fn global_alpha(&self) -> f64 {
        self.state.borrow().global_alpha
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-globalalpha
    pub(crate) fn set_global_alpha(&self, alpha: f64) {
        if !alpha.is_finite() || !(0.0..=1.0).contains(&alpha) {
            return;
        }

        self.state.borrow_mut().global_alpha = alpha;
        self.send_canvas_2d_msg(Canvas2dMsg::SetGlobalAlpha(alpha as f32))
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-globalcompositeoperation
    pub(crate) fn global_composite_operation(&self) -> DOMString {
        match self.state.borrow().global_composition {
            CompositionOrBlending::Composition(op) => DOMString::from(op.to_str()),
            CompositionOrBlending::Blending(op) => DOMString::from(op.to_str()),
        }
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-globalcompositeoperation
    pub(crate) fn set_global_composite_operation(&self, op_str: DOMString) {
        if let Ok(op) = CompositionOrBlending::from_str(&op_str) {
            self.state.borrow_mut().global_composition = op;
            self.send_canvas_2d_msg(Canvas2dMsg::SetGlobalComposition(op))
        }
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-imagesmoothingenabled
    pub(crate) fn image_smoothing_enabled(&self) -> bool {
        self.state.borrow().image_smoothing_enabled
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-imagesmoothingenabled
    pub(crate) fn set_image_smoothing_enabled(&self, value: bool) {
        self.state.borrow_mut().image_smoothing_enabled = value;
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-filltext
    pub(crate) fn fill_text(
        &self,
        canvas: Option<&HTMLCanvasElement>,
        text: DOMString,
        x: f64,
        y: f64,
        max_width: Option<f64>,
        can_gc: CanGc,
    ) {
        if !x.is_finite() || !y.is_finite() {
            return;
        }
        if max_width.is_some_and(|max_width| !max_width.is_finite() || max_width <= 0.) {
            return;
        }
        if self.state.borrow().font_style.is_none() {
            self.set_font(
                canvas,
                CanvasContextState::DEFAULT_FONT_STYLE.into(),
                can_gc,
            )
        }

        let is_rtl = match self.state.borrow().direction {
            Direction::Ltr => false,
            Direction::Rtl => true,
            Direction::Inherit => false, // TODO: resolve direction wrt to canvas element
        };

        let style = self.state.borrow().fill_style.to_fill_or_stroke_style();
        self.send_canvas_2d_msg(Canvas2dMsg::FillText(
            text.into(),
            x,
            y,
            max_width,
            style,
            is_rtl,
        ));
    }

    // https://html.spec.whatwg.org/multipage/#textmetrics
    pub(crate) fn measure_text(
        &self,
        global: &GlobalScope,
        canvas: Option<&HTMLCanvasElement>,
        text: DOMString,
        can_gc: CanGc,
    ) -> DomRoot<TextMetrics> {
        if self.state.borrow().font_style.is_none() {
            self.set_font(
                canvas,
                CanvasContextState::DEFAULT_FONT_STYLE.into(),
                can_gc,
            );
        }

        let (sender, receiver) = ipc::channel::<CanvasTextMetrics>().unwrap();
        self.send_canvas_2d_msg(Canvas2dMsg::MeasureText(text.into(), sender));
        let metrics = receiver.recv().unwrap();

        TextMetrics::new(
            global,
            metrics.width.into(),
            metrics.actual_boundingbox_left.into(),
            metrics.actual_boundingbox_right.into(),
            metrics.font_boundingbox_ascent.into(),
            metrics.font_boundingbox_descent.into(),
            metrics.actual_boundingbox_ascent.into(),
            metrics.actual_boundingbox_descent.into(),
            metrics.em_height_ascent.into(),
            metrics.em_height_descent.into(),
            metrics.hanging_baseline.into(),
            metrics.alphabetic_baseline.into(),
            metrics.ideographic_baseline.into(),
            can_gc,
        )
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-font
    pub(crate) fn set_font(
        &self,
        canvas: Option<&HTMLCanvasElement>,
        value: DOMString,
        can_gc: CanGc,
    ) {
        let canvas = match canvas {
            Some(element) => element,
            None => return, // offscreen canvas doesn't have a placeholder canvas
        };
        let node = canvas.upcast::<Node>();
        let window = canvas.owner_window();
        let resolved_font_style =
            match window.resolved_font_style_query(node, value.to_string(), can_gc) {
                Some(value) => value,
                None => return, // syntax error
            };
        self.state.borrow_mut().font_style = Some((*resolved_font_style).clone());
        self.send_canvas_2d_msg(Canvas2dMsg::SetFont((*resolved_font_style).clone()));
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-font
    pub(crate) fn font(&self) -> DOMString {
        self.state.borrow().font_style.as_ref().map_or_else(
            || CanvasContextState::DEFAULT_FONT_STYLE.into(),
            |style| {
                let mut result = String::new();
                serialize_font(style, &mut result).unwrap();
                DOMString::from(result)
            },
        )
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-textalign
    pub(crate) fn text_align(&self) -> CanvasTextAlign {
        match self.state.borrow().text_align {
            TextAlign::Start => CanvasTextAlign::Start,
            TextAlign::End => CanvasTextAlign::End,
            TextAlign::Left => CanvasTextAlign::Left,
            TextAlign::Right => CanvasTextAlign::Right,
            TextAlign::Center => CanvasTextAlign::Center,
        }
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-textalign
    pub(crate) fn set_text_align(&self, value: CanvasTextAlign) {
        let text_align = match value {
            CanvasTextAlign::Start => TextAlign::Start,
            CanvasTextAlign::End => TextAlign::End,
            CanvasTextAlign::Left => TextAlign::Left,
            CanvasTextAlign::Right => TextAlign::Right,
            CanvasTextAlign::Center => TextAlign::Center,
        };
        self.state.borrow_mut().text_align = text_align;
        self.send_canvas_2d_msg(Canvas2dMsg::SetTextAlign(text_align));
    }

    pub(crate) fn text_baseline(&self) -> CanvasTextBaseline {
        match self.state.borrow().text_baseline {
            TextBaseline::Top => CanvasTextBaseline::Top,
            TextBaseline::Hanging => CanvasTextBaseline::Hanging,
            TextBaseline::Middle => CanvasTextBaseline::Middle,
            TextBaseline::Alphabetic => CanvasTextBaseline::Alphabetic,
            TextBaseline::Ideographic => CanvasTextBaseline::Ideographic,
            TextBaseline::Bottom => CanvasTextBaseline::Bottom,
        }
    }

    pub(crate) fn set_text_baseline(&self, value: CanvasTextBaseline) {
        let text_baseline = match value {
            CanvasTextBaseline::Top => TextBaseline::Top,
            CanvasTextBaseline::Hanging => TextBaseline::Hanging,
            CanvasTextBaseline::Middle => TextBaseline::Middle,
            CanvasTextBaseline::Alphabetic => TextBaseline::Alphabetic,
            CanvasTextBaseline::Ideographic => TextBaseline::Ideographic,
            CanvasTextBaseline::Bottom => TextBaseline::Bottom,
        };
        self.state.borrow_mut().text_baseline = text_baseline;
        self.send_canvas_2d_msg(Canvas2dMsg::SetTextBaseline(text_baseline));
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-direction
    pub(crate) fn direction(&self) -> CanvasDirection {
        match self.state.borrow().direction {
            Direction::Ltr => CanvasDirection::Ltr,
            Direction::Rtl => CanvasDirection::Rtl,
            Direction::Inherit => CanvasDirection::Inherit,
        }
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-direction
    pub(crate) fn set_direction(&self, value: CanvasDirection) {
        let direction = match value {
            CanvasDirection::Ltr => Direction::Ltr,
            CanvasDirection::Rtl => Direction::Rtl,
            CanvasDirection::Inherit => Direction::Inherit,
        };
        self.state.borrow_mut().direction = direction;
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-linewidth
    pub(crate) fn line_width(&self) -> f64 {
        self.state.borrow().line_width
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-linewidth
    pub(crate) fn set_line_width(&self, width: f64) {
        if !width.is_finite() || width <= 0.0 {
            return;
        }

        self.state.borrow_mut().line_width = width;
        self.send_canvas_2d_msg(Canvas2dMsg::SetLineWidth(width as f32))
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-linecap
    pub(crate) fn line_cap(&self) -> CanvasLineCap {
        match self.state.borrow().line_cap {
            LineCapStyle::Butt => CanvasLineCap::Butt,
            LineCapStyle::Round => CanvasLineCap::Round,
            LineCapStyle::Square => CanvasLineCap::Square,
        }
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-linecap
    pub(crate) fn set_line_cap(&self, cap: CanvasLineCap) {
        let line_cap = match cap {
            CanvasLineCap::Butt => LineCapStyle::Butt,
            CanvasLineCap::Round => LineCapStyle::Round,
            CanvasLineCap::Square => LineCapStyle::Square,
        };
        self.state.borrow_mut().line_cap = line_cap;
        self.send_canvas_2d_msg(Canvas2dMsg::SetLineCap(line_cap));
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-linejoin
    pub(crate) fn line_join(&self) -> CanvasLineJoin {
        match self.state.borrow().line_join {
            LineJoinStyle::Round => CanvasLineJoin::Round,
            LineJoinStyle::Bevel => CanvasLineJoin::Bevel,
            LineJoinStyle::Miter => CanvasLineJoin::Miter,
        }
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-linejoin
    pub(crate) fn set_line_join(&self, join: CanvasLineJoin) {
        let line_join = match join {
            CanvasLineJoin::Round => LineJoinStyle::Round,
            CanvasLineJoin::Bevel => LineJoinStyle::Bevel,
            CanvasLineJoin::Miter => LineJoinStyle::Miter,
        };
        self.state.borrow_mut().line_join = line_join;
        self.send_canvas_2d_msg(Canvas2dMsg::SetLineJoin(line_join));
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-miterlimit
    pub(crate) fn miter_limit(&self) -> f64 {
        self.state.borrow().miter_limit
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-miterlimit
    pub(crate) fn set_miter_limit(&self, limit: f64) {
        if !limit.is_finite() || limit <= 0.0 {
            return;
        }

        self.state.borrow_mut().miter_limit = limit;
        self.send_canvas_2d_msg(Canvas2dMsg::SetMiterLimit(limit as f32))
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-context-2d-getlinedash>
    pub(crate) fn line_dash(&self) -> Vec<f64> {
        // > return a sequence whose values are the values of
        // > the object's dash list, in the same order.
        self.state.borrow().line_dash.clone()
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-context-2d-setlinedash>
    pub(crate) fn set_line_dash(&self, segments: Vec<f64>) {
        // > If any value in segments is not finite (e.g. an Infinity or a NaN value),
        // > or if any value is negative (less than zero), then return (without throwing
        // > an exception; user agents could show a message on a developer console,
        // > though, as that would be helpful for debugging).
        if segments
            .iter()
            .any(|segment| !segment.is_finite() || *segment < 0.0)
        {
            return;
        }

        // > If the number of elements in segments is odd, then let segments
        // > be the concatenation of two copies of segments.
        let mut line_dash: Vec<_> = segments.clone();
        if segments.len() & 1 == 1 {
            line_dash.extend(line_dash.clone());
        }

        // > Let the object's dash list be segments.
        self.state.borrow_mut().line_dash = line_dash.clone();
        self.send_canvas_2d_msg(Canvas2dMsg::SetLineDash(
            line_dash.into_iter().map(|dash| dash as f32).collect(),
        ))
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-context-2d-linedashoffset>
    pub(crate) fn line_dash_offset(&self) -> f64 {
        // > On getting, it must return the current value.
        self.state.borrow().line_dash_offset
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-context-2d-linedashoffset?
    pub(crate) fn set_line_dash_offset(&self, offset: f64) {
        // > On setting, infinite and NaN values must be ignored,
        // > leaving the value unchanged;
        if !offset.is_finite() {
            return;
        }

        // > other values must change the current value to the new value.
        self.state.borrow_mut().line_dash_offset = offset;
        self.send_canvas_2d_msg(Canvas2dMsg::SetLineDashOffset(offset as f32));
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-createimagedata
    pub(crate) fn create_image_data(
        &self,
        global: &GlobalScope,
        sw: i32,
        sh: i32,
        can_gc: CanGc,
    ) -> Fallible<DomRoot<ImageData>> {
        if sw == 0 || sh == 0 {
            return Err(Error::IndexSize);
        }
        ImageData::new(global, sw.unsigned_abs(), sh.unsigned_abs(), None, can_gc)
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-createimagedata
    pub(crate) fn create_image_data_(
        &self,
        global: &GlobalScope,
        imagedata: &ImageData,
        can_gc: CanGc,
    ) -> Fallible<DomRoot<ImageData>> {
        ImageData::new(global, imagedata.Width(), imagedata.Height(), None, can_gc)
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-getimagedata
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn get_image_data(
        &self,
        canvas_size: Size2D<u32>,
        global: &GlobalScope,
        sx: i32,
        sy: i32,
        sw: i32,
        sh: i32,
        can_gc: CanGc,
    ) -> Fallible<DomRoot<ImageData>> {
        // FIXME(nox): There are many arithmetic operations here that can
        // overflow or underflow, this should probably be audited.

        if sw == 0 || sh == 0 {
            return Err(Error::IndexSize);
        }

        if !self.origin_is_clean() {
            return Err(Error::Security);
        }

        let (origin, size) = adjust_size_sign(Point2D::new(sx, sy), Size2D::new(sw, sh));
        let read_rect = match pixels::clip(origin, size.to_u32(), canvas_size) {
            Some(rect) => rect,
            None => {
                // All the pixels are outside the canvas surface.
                return ImageData::new(global, size.width, size.height, None, can_gc);
            },
        };

        let data = if self.is_paintable() {
            Some(self.get_rect(canvas_size, read_rect))
        } else {
            None
        };

        ImageData::new(global, size.width, size.height, data, can_gc)
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-putimagedata
    pub(crate) fn put_image_data(
        &self,
        canvas_size: Size2D<u32>,
        imagedata: &ImageData,
        dx: i32,
        dy: i32,
    ) {
        self.put_image_data_(
            canvas_size,
            imagedata,
            dx,
            dy,
            0,
            0,
            imagedata.Width() as i32,
            imagedata.Height() as i32,
        )
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-context-2d-putimagedata>
    #[allow(unsafe_code, clippy::too_many_arguments)]
    pub(crate) fn put_image_data_(
        &self,
        canvas_size: Size2D<u32>,
        imagedata: &ImageData,
        dx: i32,
        dy: i32,
        dirty_x: i32,
        dirty_y: i32,
        dirty_width: i32,
        dirty_height: i32,
    ) {
        if !self.is_paintable() {
            return;
        }

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

        // Steps 3-6.
        let (src_origin, src_size) = adjust_size_sign(
            Point2D::new(dirty_x, dirty_y),
            Size2D::new(dirty_width, dirty_height),
        );
        let src_rect = match pixels::clip(src_origin, src_size.to_u32(), imagedata_size.to_u32()) {
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
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-drawimage
    pub(crate) fn draw_image(
        &self,
        canvas: Option<&HTMLCanvasElement>,
        image: CanvasImageSource,
        dx: f64,
        dy: f64,
    ) -> ErrorResult {
        if !(dx.is_finite() && dy.is_finite()) {
            return Ok(());
        }

        self.draw_image_internal(canvas, image, 0f64, 0f64, None, None, dx, dy, None, None)
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-drawimage
    pub(crate) fn draw_image_(
        &self,
        canvas: Option<&HTMLCanvasElement>,
        image: CanvasImageSource,
        dx: f64,
        dy: f64,
        dw: f64,
        dh: f64,
    ) -> ErrorResult {
        if !(dx.is_finite() && dy.is_finite() && dw.is_finite() && dh.is_finite()) {
            return Ok(());
        }

        self.draw_image_internal(
            canvas,
            image,
            0f64,
            0f64,
            None,
            None,
            dx,
            dy,
            Some(dw),
            Some(dh),
        )
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-context-2d-drawimage>
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn draw_image__(
        &self,
        canvas: Option<&HTMLCanvasElement>,
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

        self.draw_image_internal(
            canvas,
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

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-beginpath
    pub(crate) fn begin_path(&self) {
        self.send_canvas_2d_msg(Canvas2dMsg::BeginPath);
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-fill
    pub(crate) fn fill(&self, _fill_rule: CanvasFillRule) {
        // TODO: Process fill rule
        let style = self.state.borrow().fill_style.to_fill_or_stroke_style();
        self.send_canvas_2d_msg(Canvas2dMsg::Fill(style));
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-fill
    pub(crate) fn fill_(&self, path: Vec<PathSegment>, _fill_rule: CanvasFillRule) {
        // TODO: Process fill rule
        let style = self.state.borrow().fill_style.to_fill_or_stroke_style();
        self.send_canvas_2d_msg(Canvas2dMsg::FillPath(style, path));
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-stroke
    pub(crate) fn stroke(&self) {
        let style = self.state.borrow().stroke_style.to_fill_or_stroke_style();
        self.send_canvas_2d_msg(Canvas2dMsg::Stroke(style));
    }

    pub(crate) fn stroke_(&self, path: Vec<PathSegment>) {
        let style = self.state.borrow().stroke_style.to_fill_or_stroke_style();
        self.send_canvas_2d_msg(Canvas2dMsg::StrokePath(style, path));
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-clip
    pub(crate) fn clip(&self, _fill_rule: CanvasFillRule) {
        // TODO: Process fill rule
        self.send_canvas_2d_msg(Canvas2dMsg::Clip);
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-clip
    pub(crate) fn clip_(&self, path: Vec<PathSegment>, _fill_rule: CanvasFillRule) {
        // TODO: Process fill rule
        self.send_canvas_2d_msg(Canvas2dMsg::ClipPath(path));
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-ispointinpath
    pub(crate) fn is_point_in_path(
        &self,
        global: &GlobalScope,
        x: f64,
        y: f64,
        fill_rule: CanvasFillRule,
    ) -> bool {
        if !(x.is_finite() && y.is_finite()) {
            return false;
        }

        let fill_rule = match fill_rule {
            CanvasFillRule::Nonzero => FillRule::Nonzero,
            CanvasFillRule::Evenodd => FillRule::Evenodd,
        };
        let (sender, receiver) =
            profiled_ipc::channel::<bool>(global.time_profiler_chan().clone()).unwrap();
        self.send_canvas_2d_msg(Canvas2dMsg::IsPointInCurrentPath(x, y, fill_rule, sender));
        receiver.recv().unwrap()
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-ispointinpath
    pub(crate) fn is_point_in_path_(
        &self,
        global: &GlobalScope,
        path: Vec<PathSegment>,
        x: f64,
        y: f64,
        fill_rule: CanvasFillRule,
    ) -> bool {
        if !(x.is_finite() && y.is_finite()) {
            return false;
        }

        let fill_rule = match fill_rule {
            CanvasFillRule::Nonzero => FillRule::Nonzero,
            CanvasFillRule::Evenodd => FillRule::Evenodd,
        };
        let (sender, receiver) =
            profiled_ipc::channel::<bool>(global.time_profiler_chan().clone()).unwrap();
        self.send_canvas_2d_msg(Canvas2dMsg::IsPointInPath(path, x, y, fill_rule, sender));
        receiver.recv().unwrap()
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-scale
    pub(crate) fn scale(&self, x: f64, y: f64) {
        if !(x.is_finite() && y.is_finite()) {
            return;
        }

        let transform = self.state.borrow().transform;
        self.state.borrow_mut().transform = transform.pre_scale(x as f32, y as f32);
        self.update_transform()
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-rotate
    pub(crate) fn rotate(&self, angle: f64) {
        if angle == 0.0 || !angle.is_finite() {
            return;
        }

        let (sin, cos) = (angle.sin(), angle.cos());
        let transform = self.state.borrow().transform;
        self.state.borrow_mut().transform =
            Transform2D::new(cos as f32, sin as f32, -sin as f32, cos as f32, 0.0, 0.0)
                .then(&transform);
        self.update_transform()
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-translate
    pub(crate) fn translate(&self, x: f64, y: f64) {
        if !(x.is_finite() && y.is_finite()) {
            return;
        }

        let transform = self.state.borrow().transform;
        self.state.borrow_mut().transform = transform.pre_translate(vec2(x as f32, y as f32));
        self.update_transform()
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-transform
    pub(crate) fn transform(&self, a: f64, b: f64, c: f64, d: f64, e: f64, f: f64) {
        if !(a.is_finite() &&
            b.is_finite() &&
            c.is_finite() &&
            d.is_finite() &&
            e.is_finite() &&
            f.is_finite())
        {
            return;
        }

        let transform = self.state.borrow().transform;
        self.state.borrow_mut().transform =
            Transform2D::new(a as f32, b as f32, c as f32, d as f32, e as f32, f as f32)
                .then(&transform);
        self.update_transform()
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-gettransform
    pub(crate) fn get_transform(&self, global: &GlobalScope, can_gc: CanGc) -> DomRoot<DOMMatrix> {
        let (sender, receiver) = ipc::channel::<Transform2D<f32>>().unwrap();
        self.send_canvas_2d_msg(Canvas2dMsg::GetTransform(sender));
        let transform = receiver.recv().unwrap();

        DOMMatrix::new(global, true, transform.cast::<f64>().to_3d(), can_gc)
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-settransform
    pub(crate) fn set_transform(&self, a: f64, b: f64, c: f64, d: f64, e: f64, f: f64) {
        if !(a.is_finite() &&
            b.is_finite() &&
            c.is_finite() &&
            d.is_finite() &&
            e.is_finite() &&
            f.is_finite())
        {
            return;
        }

        self.state.borrow_mut().transform =
            Transform2D::new(a as f32, b as f32, c as f32, d as f32, e as f32, f as f32);
        self.update_transform()
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-resettransform
    pub(crate) fn reset_transform(&self) {
        self.state.borrow_mut().transform = Transform2D::identity();
        self.update_transform()
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-closepath
    pub(crate) fn close_path(&self) {
        self.send_canvas_2d_msg(Canvas2dMsg::ClosePath);
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-moveto
    pub(crate) fn move_to(&self, x: f64, y: f64) {
        if !(x.is_finite() && y.is_finite()) {
            return;
        }
        self.send_canvas_2d_msg(Canvas2dMsg::MoveTo(Point2D::new(x as f32, y as f32)));
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-lineto
    pub(crate) fn line_to(&self, x: f64, y: f64) {
        if !(x.is_finite() && y.is_finite()) {
            return;
        }
        self.send_canvas_2d_msg(Canvas2dMsg::LineTo(Point2D::new(x as f32, y as f32)));
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-rect
    pub(crate) fn rect(&self, x: f64, y: f64, width: f64, height: f64) {
        if [x, y, width, height].iter().all(|val| val.is_finite()) {
            let rect = Rect::new(
                Point2D::new(x as f32, y as f32),
                Size2D::new(width as f32, height as f32),
            );
            self.send_canvas_2d_msg(Canvas2dMsg::Rect(rect));
        }
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-quadraticcurveto
    pub(crate) fn quadratic_curve_to(&self, cpx: f64, cpy: f64, x: f64, y: f64) {
        if !(cpx.is_finite() && cpy.is_finite() && x.is_finite() && y.is_finite()) {
            return;
        }
        self.send_canvas_2d_msg(Canvas2dMsg::QuadraticCurveTo(
            Point2D::new(cpx as f32, cpy as f32),
            Point2D::new(x as f32, y as f32),
        ));
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-beziercurveto
    pub(crate) fn bezier_curve_to(
        &self,
        cp1x: f64,
        cp1y: f64,
        cp2x: f64,
        cp2y: f64,
        x: f64,
        y: f64,
    ) {
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
    pub(crate) fn arc(
        &self,
        x: f64,
        y: f64,
        r: f64,
        start: f64,
        end: f64,
        ccw: bool,
    ) -> ErrorResult {
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
    pub(crate) fn arc_to(&self, cp1x: f64, cp1y: f64, cp2x: f64, cp2y: f64, r: f64) -> ErrorResult {
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

    /// <https://html.spec.whatwg.org/multipage/#dom-context-2d-ellipse>
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn ellipse(
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
}

pub(crate) fn parse_color(
    canvas: Option<&HTMLCanvasElement>,
    string: &str,
    can_gc: CanGc,
) -> Result<AbsoluteColor, ()> {
    let mut input = ParserInput::new(string);
    let mut parser = Parser::new(&mut input);
    let url = Url::parse("about:blank").unwrap().into();
    let context = ParserContext::new(
        Origin::Author,
        &url,
        Some(CssRuleType::Style),
        ParsingMode::DEFAULT,
        QuirksMode::NoQuirks,
        /* namespaces = */ Default::default(),
        None,
        None,
    );
    match Color::parse_and_compute(&context, &mut parser, None) {
        Some(color) => {
            // TODO: https://github.com/whatwg/html/issues/1099
            // Reconsider how to calculate currentColor in a display:none canvas

            // TODO: will need to check that the context bitmap mode is fixed
            // once we implement CanvasProxy
            let current_color = match canvas {
                // https://drafts.css-houdini.org/css-paint-api/#2d-rendering-context
                // Whenever "currentColor" is used as a color in the PaintRenderingContext2D API,
                // it is treated as opaque black.
                None => AbsoluteColor::BLACK,
                Some(canvas) => {
                    let canvas_element = canvas.upcast::<Element>();
                    match canvas_element.style(can_gc) {
                        Some(ref s) if canvas_element.has_css_layout_box(can_gc) => {
                            s.get_inherited_text().color
                        },
                        _ => AbsoluteColor::BLACK,
                    }
                },
            };

            Ok(color.resolve_to_absolute(&current_color))
        },
        None => Err(()),
    }
}

// Used by drawImage to determine if a source or destination rectangle is valid
// Origin coordinates and size cannot be negative. Size has to be greater than zero
pub(crate) fn is_rect_valid(rect: Rect<f64>) -> bool {
    rect.size.width > 0.0 && rect.size.height > 0.0
}

// https://html.spec.whatwg.org/multipage/#serialisation-of-a-color
pub(crate) fn serialize<W>(color: &AbsoluteColor, dest: &mut W) -> fmt::Result
where
    W: fmt::Write,
{
    let srgb = match color.color_space {
        ColorSpace::Srgb if color.flags.contains(ColorFlags::IS_LEGACY_SRGB) => *color,
        ColorSpace::Hsl | ColorSpace::Hwb => color.into_srgb_legacy(),
        _ => return color.to_css(&mut CssWriter::new(dest)),
    };
    debug_assert!(srgb.flags.contains(ColorFlags::IS_LEGACY_SRGB));
    let red = clamp_unit_f32(srgb.components.0);
    let green = clamp_unit_f32(srgb.components.1);
    let blue = clamp_unit_f32(srgb.components.2);
    let alpha = srgb.alpha;
    if alpha == 1.0 {
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
        write!(dest, "rgba({}, {}, {}, {})", red, green, blue, alpha)
    }
}

pub(crate) fn adjust_size_sign(
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

fn serialize_font<W>(style: &Font, dest: &mut W) -> fmt::Result
where
    W: fmt::Write,
{
    if style.font_style == FontStyle::ITALIC {
        write!(dest, "{} ", style.font_style.to_css_string())?;
    }
    if style.font_weight.is_bold() {
        write!(dest, "{} ", style.font_weight.to_css_string())?;
    }
    if style.font_variant_caps == FontVariantCaps::SmallCaps {
        write!(dest, "{} ", style.font_variant_caps.to_css_string())?;
    }
    write!(
        dest,
        "{} {}",
        style.font_size.to_css_string(),
        style.font_family.to_css_string()
    )
}

fn adjust_canvas_size(size: Size2D<u64>) -> Size2D<u64> {
    // Firefox limits width/height to 32767 pixels and Chromium to 65535 pixels,
    // but slows down dramatically before it reaches that limit.
    // We limit by area instead, giving us larger maximum dimensions,
    // in exchange for a smaller maximum canvas size.
    const MAX_CANVAS_AREA: u64 = 32768 * 8192;
    // Max width/height to 65535 in CSS pixels.
    const MAX_CANVAS_SIZE: u64 = 65535;

    if !size.is_empty() &&
        size.greater_than(Size2D::new(MAX_CANVAS_SIZE, MAX_CANVAS_SIZE))
            .none() &&
        size.area() < MAX_CANVAS_AREA
    {
        size
    } else {
        Size2D::zero()
    }
}

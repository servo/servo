/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::Cell;
use std::fmt;
use std::str::FromStr;
use std::sync::Arc;

use app_units::Au;
use base::generic_channel::GenericSender;
use base::{Epoch, generic_channel};
use canvas_traits::canvas::{
    Canvas2dMsg, CanvasFont, CanvasId, CanvasMsg, CompositionOptions, CompositionOrBlending,
    FillOrStrokeStyle, FillRule, GlyphAndPosition, LineCapStyle, LineJoinStyle, LineOptions,
    LinearGradientStyle, Path, RadialGradientStyle, RepetitionStyle, ShadowOptions, TextRun,
};
use constellation_traits::ScriptToConstellationMessage;
use cssparser::color::clamp_unit_f32;
use cssparser::{Parser, ParserInput};
use euclid::default::{Point2D, Rect, Size2D, Transform2D};
use euclid::{Vector2D, vec2};
use fonts::{
    FontBaseline, FontContext, FontGroup, FontIdentifier, FontMetrics, FontRef,
    LAST_RESORT_GLYPH_ADVANCE, ShapingFlags, ShapingOptions,
};
use net_traits::image_cache::{ImageCache, ImageResponse};
use net_traits::request::CorsSettings;
use pixels::{Snapshot, SnapshotAlphaMode, SnapshotPixelFormat};
use servo_arc::Arc as ServoArc;
use servo_url::{ImmutableOrigin, ServoUrl};
use style::color::{AbsoluteColor, ColorFlags, ColorSpace};
use style::properties::longhands::font_variant_caps::computed_value::T as FontVariantCaps;
use style::properties::style_structs::Font;
use style::stylesheets::CssRuleType;
use style::values::computed::XLang;
use style::values::computed::font::FontStyle;
use style::values::specified::color::Color;
use style_traits::values::ToCss;
use style_traits::{CssWriter, ParsingMode};
use unicode_script::Script;
use url::Url;
use webrender_api::ImageKey;

use crate::canvas_context::{CanvasContext, OffscreenRenderingContext, RenderingContext};
use crate::conversions::Convert;
use crate::css::parser_context_for_anonymous_content;
use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::CanvasRenderingContext2DBinding::{
    CanvasDirection, CanvasFillRule, CanvasImageSource, CanvasLineCap, CanvasLineJoin,
    CanvasTextAlign, CanvasTextBaseline, ImageDataMethods,
};
use crate::dom::bindings::codegen::Bindings::DOMMatrixBinding::DOMMatrix2DInit;
use crate::dom::bindings::codegen::UnionTypes::StringOrCanvasGradientOrCanvasPattern;
use crate::dom::bindings::error::{Error, ErrorResult, Fallible};
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::num::Finite;
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::bindings::str::DOMString;
use crate::dom::canvasgradient::{CanvasGradient, CanvasGradientStyle, ToFillOrStrokeStyle};
use crate::dom::canvaspattern::CanvasPattern;
use crate::dom::dommatrix::DOMMatrix;
use crate::dom::dommatrixreadonly::dommatrix2dinit_to_matrix;
use crate::dom::element::Element;
use crate::dom::globalscope::GlobalScope;
use crate::dom::html::htmlcanvaselement::HTMLCanvasElement;
use crate::dom::html::htmlimageelement::HTMLImageElement;
use crate::dom::html::htmlvideoelement::HTMLVideoElement;
use crate::dom::imagebitmap::ImageBitmap;
use crate::dom::imagedata::ImageData;
use crate::dom::node::{Node, NodeTraits};
use crate::dom::offscreencanvas::OffscreenCanvas;
use crate::dom::paintworkletglobalscope::PaintWorkletGlobalScope;
use crate::dom::textmetrics::TextMetrics;
use crate::script_runtime::CanGc;

const HANGING_BASELINE_DEFAULT: f64 = 0.8;
const IDEOGRAPHIC_BASELINE_DEFAULT: f64 = 0.5;

#[cfg_attr(crown, crown::unrooted_must_root_lint::must_root)]
#[derive(Clone, JSTraceable, MallocSizeOf)]
pub(super) enum CanvasFillOrStrokeStyle {
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
pub(super) struct CanvasContextState {
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
    transform: Transform2D<f64>,
    shadow_offset_x: f64,
    shadow_offset_y: f64,
    shadow_blur: f64,
    #[no_trace]
    shadow_color: AbsoluteColor,
    #[no_trace]
    #[conditional_malloc_size_of]
    font_style: Option<ServoArc<Font>>,
    text_align: CanvasTextAlign,
    text_baseline: CanvasTextBaseline,
    direction: CanvasDirection,
    /// The number of clips pushed onto the context while in this state.
    /// When restoring old state, same number of clips will be popped to restore state.
    clips_pushed: usize,
}

impl CanvasContextState {
    const DEFAULT_FONT_STYLE: &'static str = "10px sans-serif";

    pub(super) fn new() -> CanvasContextState {
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
            text_align: CanvasTextAlign::Start,
            text_baseline: CanvasTextBaseline::Alphabetic,
            direction: CanvasDirection::Inherit,
            line_dash: Vec::new(),
            line_dash_offset: 0.0,
            clips_pushed: 0,
        }
    }

    fn composition_options(&self) -> CompositionOptions {
        CompositionOptions {
            alpha: self.global_alpha,
            composition_operation: self.global_composition,
        }
    }

    fn shadow_options(&self) -> ShadowOptions {
        ShadowOptions {
            offset_x: self.shadow_offset_x,
            offset_y: self.shadow_offset_y,
            blur: self.shadow_blur,
            color: self.shadow_color,
        }
    }

    fn line_options(&self) -> LineOptions {
        LineOptions {
            width: self.line_width,
            cap_style: self.line_cap,
            join_style: self.line_join,
            miter_limit: self.miter_limit,
            dash: self.line_dash.iter().map(|x| *x as f32).collect(),
            dash_offset: self.line_dash_offset,
        }
    }
}

#[cfg_attr(crown, crown::unrooted_must_root_lint::must_root)]
#[derive(JSTraceable, MallocSizeOf)]
pub(super) struct CanvasState {
    #[no_trace]
    canvas_thread_sender: GenericSender<CanvasMsg>,
    #[no_trace]
    canvas_id: CanvasId,
    #[no_trace]
    size: Cell<Size2D<u64>>,
    state: DomRefCell<CanvasContextState>,
    origin_clean: Cell<bool>,
    #[ignore_malloc_size_of = "ImageCache"]
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
    /// <https://html.spec.whatwg.org/multipage/#current-default-path>
    #[no_trace]
    current_default_path: DomRefCell<Path>,
}

impl CanvasState {
    pub(super) fn new(global: &GlobalScope, size: Size2D<u64>) -> Option<CanvasState> {
        debug!("Creating new canvas rendering context.");
        let (sender, receiver) =
            profile_traits::generic_channel::channel(global.time_profiler_chan().clone()).unwrap();
        let script_to_constellation_chan = global.script_to_constellation_chan();
        debug!("Asking constellation to create new canvas thread.");
        let size = adjust_canvas_size(size);
        script_to_constellation_chan
            .send(ScriptToConstellationMessage::CreateCanvasPaintThread(
                size, sender,
            ))
            .unwrap();
        let (canvas_thread_sender, canvas_id) = receiver.recv().ok()??;
        debug!("Done.");
        // Worklets always receive a unique origin. This messes with fetching
        // cached images in the case of paint worklets, since the image cache
        // is keyed on the origin requesting the image data.
        let origin = if global.is::<PaintWorkletGlobalScope>() {
            global.api_base_url().origin()
        } else {
            global.origin().immutable().clone()
        };
        Some(CanvasState {
            canvas_thread_sender,
            canvas_id,
            size: Cell::new(size),
            state: DomRefCell::new(CanvasContextState::new()),
            origin_clean: Cell::new(true),
            image_cache: global.image_cache(),
            base_url: global.api_base_url(),
            missing_image_urls: DomRefCell::new(Vec::new()),
            saved_states: DomRefCell::new(Vec::new()),
            origin,
            current_default_path: DomRefCell::new(Path::new()),
        })
    }

    pub(super) fn set_image_key(&self, image_key: ImageKey) {
        self.send_canvas_2d_msg(Canvas2dMsg::SetImageKey(image_key));
    }

    pub(super) fn get_missing_image_urls(&self) -> &DomRefCell<Vec<ServoUrl>> {
        &self.missing_image_urls
    }

    pub(super) fn get_canvas_id(&self) -> CanvasId {
        self.canvas_id
    }

    pub(super) fn is_paintable(&self) -> bool {
        !self.size.get().is_empty()
    }

    pub(super) fn send_canvas_2d_msg(&self, msg: Canvas2dMsg) {
        if !self.is_paintable() {
            return;
        }

        self.canvas_thread_sender
            .send(CanvasMsg::Canvas2d(msg, self.get_canvas_id()))
            .unwrap()
    }

    /// Updates WR image and blocks on completion
    pub(super) fn update_rendering(&self, canvas_epoch: Option<Epoch>) -> bool {
        if !self.is_paintable() {
            return false;
        }

        self.canvas_thread_sender
            .send(CanvasMsg::Canvas2d(
                Canvas2dMsg::UpdateImage(canvas_epoch),
                self.canvas_id,
            ))
            .unwrap();
        true
    }

    /// <https://html.spec.whatwg.org/multipage/#concept-canvas-set-bitmap-dimensions>
    pub(super) fn set_bitmap_dimensions(&self, size: Size2D<u64>) {
        // Step 1. Reset the rendering context to its default state.
        self.reset_to_initial_state();

        // Step 2. Resize the output bitmap to the new width and height.
        self.size.replace(adjust_canvas_size(size));

        self.canvas_thread_sender
            .send(CanvasMsg::Recreate(
                Some(self.size.get()),
                self.get_canvas_id(),
            ))
            .unwrap();
    }

    /// <https://html.spec.whatwg.org/multipage/#reset-the-rendering-context-to-its-default-state>
    pub(super) fn reset(&self) {
        self.reset_to_initial_state();

        if !self.is_paintable() {
            return;
        }

        // Step 1. Clear canvas's bitmap to transparent black.
        self.canvas_thread_sender
            .send(CanvasMsg::Recreate(None, self.get_canvas_id()))
            .unwrap();
    }

    /// <https://html.spec.whatwg.org/multipage/#reset-the-rendering-context-to-its-default-state>
    fn reset_to_initial_state(&self) {
        // Step 2. Empty the list of subpaths in context's current default path.
        *self.current_default_path.borrow_mut() = Path::new();

        // Step 3. Clear the context's drawing state stack.
        self.saved_states.borrow_mut().clear();

        // Step 4. Reset everything that drawing state consists of to their initial values.
        *self.state.borrow_mut() = CanvasContextState::new();

        // <https://html.spec.whatwg.org/multipage/#security-with-canvas-elements>
        // The flag can be reset in certain situations; for example, when changing the value of the
        // width or the height content attribute of the canvas element to which a
        // CanvasRenderingContext2D is bound, the bitmap is cleared and its origin-clean flag is
        // reset.
        self.set_origin_clean(true);
    }

    pub(super) fn reset_bitmap(&self) {
        if !self.is_paintable() {
            return;
        }

        self.send_canvas_2d_msg(Canvas2dMsg::ClearRect(
            self.size.get().to_f32().into(),
            Transform2D::identity(),
        ));
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

    pub(super) fn origin_is_clean(&self) -> bool {
        self.origin_clean.get()
    }

    fn set_origin_clean(&self, origin_clean: bool) {
        self.origin_clean.set(origin_clean);
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
        let raster_image = match self.request_image_from_cache(url, cors_setting) {
            ImageResponse::Loaded(image, _) => {
                if let Some(image) = image.as_raster_image() {
                    image
                } else {
                    // TODO: https://html.spec.whatwg.org/multipage/#dom-context-2d-drawimage
                    warn!("Vector images are not supported as image source in canvas2d");
                    return None;
                }
            },
            ImageResponse::FailedToLoadOrDecode | ImageResponse::MetadataLoaded(_) => {
                return None;
            },
        };

        Some(raster_image.as_snapshot())
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
                ImageResponse::FailedToLoadOrDecode
            },
        }
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
    #[expect(clippy::too_many_arguments)]
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
            CanvasImageSource::HTMLImageElement(ref image) => {
                // https://html.spec.whatwg.org/multipage/#drawing-images
                // 2. Let usability be the result of checking the usability of image.
                // 3. If usability is bad, then return (without drawing anything).
                if !image.is_usable()? {
                    return Ok(());
                }

                self.draw_html_image_element(image, htmlcanvas, sx, sy, sw, sh, dx, dy, dw, dh);
                Ok(())
            },
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
                    return Err(Error::InvalidState(None));
                }

                self.draw_html_canvas_element(canvas, htmlcanvas, sx, sy, sw, sh, dx, dy, dw, dh)
            },
            CanvasImageSource::ImageBitmap(ref bitmap) => {
                // <https://html.spec.whatwg.org/multipage/#check-the-usability-of-the-image-argument>
                if bitmap.is_detached() {
                    return Err(Error::InvalidState(None));
                }

                self.draw_image_bitmap(bitmap, htmlcanvas, sx, sy, sw, sh, dx, dy, dw, dh);
                Ok(())
            },
            CanvasImageSource::OffscreenCanvas(ref canvas) => {
                // <https://html.spec.whatwg.org/multipage/#check-the-usability-of-the-image-argument>
                if canvas.get_size().is_empty() {
                    return Err(Error::InvalidState(None));
                }

                self.draw_offscreen_canvas(canvas, htmlcanvas, sx, sy, sw, sh, dx, dy, dw, dh)
            },
            CanvasImageSource::CSSStyleValue(ref value) => {
                let url = value
                    .get_url(self.base_url.clone())
                    .ok_or(Error::InvalidState(None))?;
                self.fetch_and_draw_image_data(
                    htmlcanvas, url, None, sx, sy, sw, sh, dx, dy, dw, dh,
                )
            },
        };

        if result.is_ok() && !self.is_origin_clean(image) {
            self.set_origin_clean(false);
        }
        result
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-context-2d-drawimage>
    #[expect(clippy::too_many_arguments)]
    fn draw_html_image_element(
        &self,
        image: &HTMLImageElement,
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
        let Some(snapshot) = image.get_raster_image_data() else {
            return;
        };

        // Step 4. Establish the source and destination rectangles.
        let image_size = snapshot.size();
        let dw = dw.unwrap_or(image_size.width as f64);
        let dh = dh.unwrap_or(image_size.height as f64);
        let sw = sw.unwrap_or(image_size.width as f64);
        let sh = sh.unwrap_or(image_size.height as f64);

        let (source_rect, dest_rect) =
            self.adjust_source_dest_rects(image_size, sx, sy, sw, sh, dx, dy, dw, dh);

        // Step 5. If one of the sw or sh arguments is zero, then return. Nothing is painted.
        if !is_rect_valid(source_rect) || !is_rect_valid(dest_rect) {
            return;
        }

        let smoothing_enabled = self.state.borrow().image_smoothing_enabled;

        self.send_canvas_2d_msg(Canvas2dMsg::DrawImage(
            snapshot.to_shared(),
            dest_rect,
            source_rect,
            smoothing_enabled,
            self.state.borrow().shadow_options(),
            self.state.borrow().composition_options(),
            self.state.borrow().transform,
        ));

        self.mark_as_dirty(canvas);
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-context-2d-drawimage>
    #[expect(clippy::too_many_arguments)]
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
            snapshot.to_shared(),
            dest_rect,
            source_rect,
            smoothing_enabled,
            self.state.borrow().shadow_options(),
            self.state.borrow().composition_options(),
            self.state.borrow().transform,
        ));

        self.mark_as_dirty(canvas);
    }

    #[expect(clippy::too_many_arguments)]
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
        let canvas_size = canvas
            .context()
            .map_or_else(|| canvas.get_size(), |context| context.size());

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
                        dest_rect,
                        source_rect,
                        smoothing_enabled,
                        self.state.borrow().shadow_options(),
                        self.state.borrow().composition_options(),
                        self.state.borrow().transform,
                    ));
                },
                OffscreenRenderingContext::BitmapRenderer(ref context) => {
                    let Some(snapshot) = context.get_image_data() else {
                        return Ok(());
                    };

                    self.send_canvas_2d_msg(Canvas2dMsg::DrawImage(
                        snapshot.to_shared(),
                        dest_rect,
                        source_rect,
                        smoothing_enabled,
                        self.state.borrow().shadow_options(),
                        self.state.borrow().composition_options(),
                        self.state.borrow().transform,
                    ));
                },
                OffscreenRenderingContext::Detached => return Err(Error::InvalidState(None)),
            }
        } else {
            self.send_canvas_2d_msg(Canvas2dMsg::DrawEmptyImage(
                image_size,
                dest_rect,
                source_rect,
                self.state.borrow().shadow_options(),
                self.state.borrow().composition_options(),
                self.state.borrow().transform,
            ));
        }

        self.mark_as_dirty(htmlcanvas);
        Ok(())
    }

    #[expect(clippy::too_many_arguments)]
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
        let canvas_size = canvas
            .context()
            .map_or_else(|| canvas.get_size(), |context| context.size());

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
                        dest_rect,
                        source_rect,
                        smoothing_enabled,
                        self.state.borrow().shadow_options(),
                        self.state.borrow().composition_options(),
                        self.state.borrow().transform,
                    ));
                },
                RenderingContext::BitmapRenderer(ref context) => {
                    let Some(snapshot) = context.get_image_data() else {
                        return Ok(());
                    };

                    self.send_canvas_2d_msg(Canvas2dMsg::DrawImage(
                        snapshot.to_shared(),
                        dest_rect,
                        source_rect,
                        smoothing_enabled,
                        self.state.borrow().shadow_options(),
                        self.state.borrow().composition_options(),
                        self.state.borrow().transform,
                    ));
                },
                RenderingContext::Placeholder(ref context) => {
                    let Some(context) = context.context() else {
                        return Err(Error::InvalidState(None));
                    };
                    match *context {
                        OffscreenRenderingContext::Context2d(ref context) => context
                            .send_canvas_2d_msg(Canvas2dMsg::DrawImageInOther(
                                self.get_canvas_id(),
                                dest_rect,
                                source_rect,
                                smoothing_enabled,
                                self.state.borrow().shadow_options(),
                                self.state.borrow().composition_options(),
                                self.state.borrow().transform,
                            )),
                        OffscreenRenderingContext::BitmapRenderer(ref context) => {
                            let Some(snapshot) = context.get_image_data() else {
                                return Ok(());
                            };

                            self.send_canvas_2d_msg(Canvas2dMsg::DrawImage(
                                snapshot.to_shared(),
                                dest_rect,
                                source_rect,
                                smoothing_enabled,
                                self.state.borrow().shadow_options(),
                                self.state.borrow().composition_options(),
                                self.state.borrow().transform,
                            ));
                        },
                        OffscreenRenderingContext::Detached => {
                            return Err(Error::InvalidState(None));
                        },
                    }
                },
                _ => return Err(Error::InvalidState(None)),
            }
        } else {
            self.send_canvas_2d_msg(Canvas2dMsg::DrawEmptyImage(
                image_size,
                dest_rect,
                source_rect,
                self.state.borrow().shadow_options(),
                self.state.borrow().composition_options(),
                self.state.borrow().transform,
            ));
        }

        self.mark_as_dirty(htmlcanvas);
        Ok(())
    }

    #[expect(clippy::too_many_arguments)]
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
            .ok_or(Error::InvalidState(None))?;
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
            snapshot.to_shared(),
            dest_rect,
            source_rect,
            smoothing_enabled,
            self.state.borrow().shadow_options(),
            self.state.borrow().composition_options(),
            self.state.borrow().transform,
        ));
        self.mark_as_dirty(canvas);
        Ok(())
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-context-2d-drawimage>
    #[expect(clippy::too_many_arguments)]
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
            snapshot.to_shared(),
            dest_rect,
            source_rect,
            smoothing_enabled,
            self.state.borrow().shadow_options(),
            self.state.borrow().composition_options(),
            self.state.borrow().transform,
        ));

        self.mark_as_dirty(canvas);
    }

    pub(super) fn mark_as_dirty(&self, canvas: Option<&HTMLCanvasElement>) {
        if let Some(canvas) = canvas {
            canvas.mark_as_dirty();
        }
    }

    /// It is used by DrawImage to calculate the size of the source and destination rectangles based
    /// on the drawImage call arguments
    /// source rectangle = area of the original image to be copied
    /// destination rectangle = area of the destination canvas where the source image is going to be drawn
    #[expect(clippy::too_many_arguments)]
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

    fn update_transform(&self, transform: Transform2D<f64>) {
        let mut state = self.state.borrow_mut();
        self.current_default_path
            .borrow_mut()
            .transform(state.transform.cast());
        state.transform = transform;
        if let Some(inverse) = transform.inverse() {
            self.current_default_path
                .borrow_mut()
                .transform(inverse.cast());
        }
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-fillrect
    pub(super) fn fill_rect(&self, x: f64, y: f64, width: f64, height: f64) {
        if let Some(rect) = self.create_drawable_rect(x, y, width, height) {
            let style = self.state.borrow().fill_style.to_fill_or_stroke_style();
            self.send_canvas_2d_msg(Canvas2dMsg::FillRect(
                rect,
                style,
                self.state.borrow().shadow_options(),
                self.state.borrow().composition_options(),
                self.state.borrow().transform,
            ));
        }
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-clearrect
    pub(super) fn clear_rect(&self, x: f64, y: f64, width: f64, height: f64) {
        if let Some(rect) = self.create_drawable_rect(x, y, width, height) {
            self.send_canvas_2d_msg(Canvas2dMsg::ClearRect(rect, self.state.borrow().transform));
        }
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-strokerect
    pub(super) fn stroke_rect(&self, x: f64, y: f64, width: f64, height: f64) {
        if let Some(rect) = self.create_drawable_rect(x, y, width, height) {
            let style = self.state.borrow().stroke_style.to_fill_or_stroke_style();
            self.send_canvas_2d_msg(Canvas2dMsg::StrokeRect(
                rect,
                style,
                self.state.borrow().line_options(),
                self.state.borrow().shadow_options(),
                self.state.borrow().composition_options(),
                self.state.borrow().transform,
            ));
        }
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-shadowoffsetx
    pub(super) fn shadow_offset_x(&self) -> f64 {
        self.state.borrow().shadow_offset_x
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-shadowoffsetx
    pub(super) fn set_shadow_offset_x(&self, value: f64) {
        if !value.is_finite() || value == self.state.borrow().shadow_offset_x {
            return;
        }
        self.state.borrow_mut().shadow_offset_x = value;
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-shadowoffsety
    pub(super) fn shadow_offset_y(&self) -> f64 {
        self.state.borrow().shadow_offset_y
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-shadowoffsety
    pub(super) fn set_shadow_offset_y(&self, value: f64) {
        if !value.is_finite() || value == self.state.borrow().shadow_offset_y {
            return;
        }
        self.state.borrow_mut().shadow_offset_y = value;
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-shadowblur
    pub(super) fn shadow_blur(&self) -> f64 {
        self.state.borrow().shadow_blur
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-shadowblur
    pub(super) fn set_shadow_blur(&self, value: f64) {
        if !value.is_finite() || value < 0f64 || value == self.state.borrow().shadow_blur {
            return;
        }
        self.state.borrow_mut().shadow_blur = value;
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-shadowcolor
    pub(super) fn shadow_color(&self) -> DOMString {
        let mut result = String::new();
        serialize(&self.state.borrow().shadow_color, &mut result).unwrap();
        DOMString::from(result)
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-shadowcolor
    pub(super) fn set_shadow_color(&self, canvas: Option<&HTMLCanvasElement>, value: DOMString) {
        if let Ok(rgba) = parse_color(canvas, &value) {
            self.state.borrow_mut().shadow_color = rgba;
        }
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-strokestyle
    pub(super) fn stroke_style(&self) -> StringOrCanvasGradientOrCanvasPattern {
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
    pub(super) fn set_stroke_style(
        &self,
        canvas: Option<&HTMLCanvasElement>,
        value: StringOrCanvasGradientOrCanvasPattern,
    ) {
        match value {
            StringOrCanvasGradientOrCanvasPattern::String(string) => {
                if let Ok(rgba) = parse_color(canvas, &string) {
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
                    self.set_origin_clean(false);
                }
            },
        }
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-strokestyle
    pub(super) fn fill_style(&self) -> StringOrCanvasGradientOrCanvasPattern {
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
    pub(super) fn set_fill_style(
        &self,
        canvas: Option<&HTMLCanvasElement>,
        value: StringOrCanvasGradientOrCanvasPattern,
    ) {
        match value {
            StringOrCanvasGradientOrCanvasPattern::String(string) => {
                if let Ok(rgba) = parse_color(canvas, &string) {
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
                    self.set_origin_clean(false);
                }
            },
        }
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-createlineargradient
    pub(super) fn create_linear_gradient(
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
    #[expect(clippy::too_many_arguments)]
    pub(super) fn create_radial_gradient(
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
            return Err(Error::IndexSize(None));
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
    pub(super) fn create_pattern(
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
                    .get_raster_image_data()
                    .ok_or(Error::InvalidState(None))?
            },
            CanvasImageSource::HTMLVideoElement(ref video) => {
                // <https://html.spec.whatwg.org/multipage/#check-the-usability-of-the-image-argument>
                if !video.is_usable() {
                    return Ok(None);
                }

                video
                    .get_current_frame_data()
                    .ok_or(Error::InvalidState(None))?
            },
            CanvasImageSource::HTMLCanvasElement(ref canvas) => {
                // <https://html.spec.whatwg.org/multipage/#check-the-usability-of-the-image-argument>
                if canvas.get_size().is_empty() {
                    return Err(Error::InvalidState(None));
                }

                canvas.get_image_data().ok_or(Error::InvalidState(None))?
            },
            CanvasImageSource::ImageBitmap(ref bitmap) => {
                // <https://html.spec.whatwg.org/multipage/#check-the-usability-of-the-image-argument>
                if bitmap.is_detached() {
                    return Err(Error::InvalidState(None));
                }

                bitmap
                    .bitmap_data()
                    .clone()
                    .ok_or(Error::InvalidState(None))?
            },
            CanvasImageSource::OffscreenCanvas(ref canvas) => {
                // <https://html.spec.whatwg.org/multipage/#check-the-usability-of-the-image-argument>
                if canvas.get_size().is_empty() {
                    return Err(Error::InvalidState(None));
                }

                canvas.get_image_data().ok_or(Error::InvalidState(None))?
            },
            CanvasImageSource::CSSStyleValue(ref value) => value
                .get_url(self.base_url.clone())
                .and_then(|url| self.fetch_image_data(url, None))
                .ok_or(Error::InvalidState(None))?,
        };

        if repetition.is_empty() {
            repetition.push_str("repeat");
        }

        if let Ok(rep) = RepetitionStyle::from_str(&repetition.str()) {
            let size = snapshot.size();
            Ok(Some(CanvasPattern::new(
                global,
                snapshot,
                size.cast(),
                rep,
                self.is_origin_clean(image),
                can_gc,
            )))
        } else {
            Err(Error::Syntax(None))
        }
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-save
    pub(super) fn save(&self) {
        self.saved_states
            .borrow_mut()
            .push(self.state.borrow().clone());
    }

    #[cfg_attr(crown, expect(crown::unrooted_must_root))]
    /// <https://html.spec.whatwg.org/multipage/#dom-context-2d-restore>
    pub(super) fn restore(&self) {
        let mut saved_states = self.saved_states.borrow_mut();
        if let Some(state) = saved_states.pop() {
            let clips_to_pop = self.state.borrow().clips_pushed;
            if clips_to_pop != 0 {
                self.send_canvas_2d_msg(Canvas2dMsg::PopClips(clips_to_pop));
            }
            self.state.borrow_mut().clone_from(&state);
        }
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-globalalpha
    pub(super) fn global_alpha(&self) -> f64 {
        self.state.borrow().global_alpha
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-globalalpha
    pub(super) fn set_global_alpha(&self, alpha: f64) {
        if !alpha.is_finite() || !(0.0..=1.0).contains(&alpha) {
            return;
        }

        self.state.borrow_mut().global_alpha = alpha;
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-globalcompositeoperation
    pub(super) fn global_composite_operation(&self) -> DOMString {
        match self.state.borrow().global_composition {
            CompositionOrBlending::Composition(op) => DOMString::from(op.to_string()),
            CompositionOrBlending::Blending(op) => DOMString::from(op.to_string()),
        }
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-globalcompositeoperation
    pub(super) fn set_global_composite_operation(&self, op_str: DOMString) {
        if let Ok(op) = CompositionOrBlending::from_str(&op_str.str()) {
            self.state.borrow_mut().global_composition = op;
        }
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-imagesmoothingenabled
    pub(super) fn image_smoothing_enabled(&self) -> bool {
        self.state.borrow().image_smoothing_enabled
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-imagesmoothingenabled
    pub(super) fn set_image_smoothing_enabled(&self, value: bool) {
        self.state.borrow_mut().image_smoothing_enabled = value;
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-filltext
    pub(super) fn fill_text(
        &self,
        global_scope: &GlobalScope,
        canvas: Option<&HTMLCanvasElement>,
        text: DOMString,
        x: f64,
        y: f64,
        max_width: Option<f64>,
    ) {
        // Step 1: If any of the arguments are infinite or NaN, then return.
        if !x.is_finite() ||
            !y.is_finite() ||
            max_width.is_some_and(|max_width| !max_width.is_finite())
        {
            return;
        }

        if self.state.borrow().font_style.is_none() {
            self.set_font(canvas, CanvasContextState::DEFAULT_FONT_STYLE.into())
        }
        // This may be `None` if if this is offscreen canvas, in which case just use
        // the initial values for the text style.
        let size = self.font_style().font_size.computed_size().px() as f64;

        let Some((bounds, text_run)) = self.text_with_size(
            global_scope,
            &text.str(),
            Point2D::new(x, y),
            size,
            max_width,
        ) else {
            return;
        };
        self.send_canvas_2d_msg(Canvas2dMsg::FillText(
            bounds,
            text_run,
            self.state.borrow().fill_style.to_fill_or_stroke_style(),
            self.state.borrow().shadow_options(),
            self.state.borrow().composition_options(),
            self.state.borrow().transform,
        ));
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-stroketext
    pub(super) fn stroke_text(
        &self,
        global_scope: &GlobalScope,
        canvas: Option<&HTMLCanvasElement>,
        text: DOMString,
        x: f64,
        y: f64,
        max_width: Option<f64>,
    ) {
        // Step 1: If any of the arguments are infinite or NaN, then return.
        if !x.is_finite() ||
            !y.is_finite() ||
            max_width.is_some_and(|max_width| !max_width.is_finite())
        {
            return;
        }

        if self.state.borrow().font_style.is_none() {
            self.set_font(canvas, CanvasContextState::DEFAULT_FONT_STYLE.into())
        }
        // This may be `None` if if this is offscreen canvas, in which case just use
        // the initial values for the text style.
        let size = self.font_style().font_size.computed_size().px() as f64;

        let Some((bounds, text_run)) = self.text_with_size(
            global_scope,
            &text.str(),
            Point2D::new(x, y),
            size,
            max_width,
        ) else {
            return;
        };
        self.send_canvas_2d_msg(Canvas2dMsg::StrokeText(
            bounds,
            text_run,
            self.state.borrow().stroke_style.to_fill_or_stroke_style(),
            self.state.borrow().line_options(),
            self.state.borrow().shadow_options(),
            self.state.borrow().composition_options(),
            self.state.borrow().transform,
        ));
    }

    /// <https://html.spec.whatwg.org/multipage/#text-preparation-algorithm>
    /// <https://html.spec.whatwg.org/multipage/#dom-context-2d-measuretext>
    /// <https://html.spec.whatwg.org/multipage/#textmetrics>
    pub(super) fn measure_text(
        &self,
        global: &GlobalScope,
        canvas: Option<&HTMLCanvasElement>,
        text: DOMString,
        can_gc: CanGc,
    ) -> DomRoot<TextMetrics> {
        // > Step 1: If maxWidth was provided but is less than or equal to zero or equal to NaN, then return an empty array.0
        // Max width is not provided for `measureText()`.

        // > Step 2: Replace all ASCII whitespace in text with U+0020 SPACE characters.
        let text = replace_ascii_whitespace(&text.str());

        // > Step 3: Let font be the current font of target, as given by that object's font
        // > attribute.
        if self.state.borrow().font_style.is_none() {
            self.set_font(canvas, CanvasContextState::DEFAULT_FONT_STYLE.into());
        }

        let Some(font_context) = global.font_context() else {
            warn!("Tried to paint to a canvas of GlobalScope without a FontContext.");
            return TextMetrics::default(global, can_gc);
        };

        let font_style = self.font_style();
        let font_group = font_context.font_group(font_style.clone());
        let font = font_group.first(font_context).expect("couldn't find font");
        let ascent = font.metrics.ascent.to_f64_px();
        let descent = font.metrics.descent.to_f64_px();
        let runs = self.build_unshaped_text_runs(font_context, &text, &font_group);

        let mut total_advance = 0.0;
        let shaped_runs: Vec<_> = runs
            .into_iter()
            .filter_map(|unshaped_text_run| {
                let text_run = unshaped_text_run.into_shaped_text_run(total_advance)?;
                total_advance += text_run.advance;
                Some(text_run)
            })
            .collect();

        let bounding_box = shaped_runs
            .iter()
            .map(|text_run| text_run.bounds)
            .reduce(|a, b| a.union(&b))
            .unwrap_or_default();

        let baseline = font.baseline().unwrap_or_else(|| FontBaseline {
            hanging_baseline: (ascent * HANGING_BASELINE_DEFAULT) as f32,
            ideographic_baseline: (-descent * IDEOGRAPHIC_BASELINE_DEFAULT) as f32,
            alphabetic_baseline: 0.,
        });
        let ideographic_baseline = baseline.ideographic_baseline as f64;
        let alphabetic_baseline = baseline.alphabetic_baseline as f64;
        let hanging_baseline = baseline.hanging_baseline as f64;

        let state = self.state.borrow();
        let anchor_x = match state.text_align {
            CanvasTextAlign::End => total_advance,
            CanvasTextAlign::Center => total_advance / 2.,
            CanvasTextAlign::Right => total_advance,
            _ => 0.,
        } as f64;
        let anchor_y = match state.text_baseline {
            CanvasTextBaseline::Top => ascent,
            CanvasTextBaseline::Hanging => hanging_baseline,
            CanvasTextBaseline::Ideographic => ideographic_baseline,
            CanvasTextBaseline::Middle => (ascent - descent) / 2.,
            CanvasTextBaseline::Alphabetic => alphabetic_baseline,
            CanvasTextBaseline::Bottom => -descent,
        };

        TextMetrics::new(
            global,
            total_advance as f64,
            anchor_x - bounding_box.min_x(),
            bounding_box.max_x() - anchor_x,
            bounding_box.max_y() - anchor_y,
            anchor_y - bounding_box.min_y(),
            ascent - anchor_y,
            descent + anchor_y,
            ascent - anchor_y,
            descent + anchor_y,
            hanging_baseline - anchor_y,
            alphabetic_baseline - anchor_y,
            ideographic_baseline - anchor_y,
            can_gc,
        )
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-font
    pub(super) fn set_font(&self, canvas: Option<&HTMLCanvasElement>, value: DOMString) {
        let canvas = match canvas {
            Some(element) => element,
            None => return, // offscreen canvas doesn't have a placeholder canvas
        };
        let node = canvas.upcast::<Node>();
        let window = canvas.owner_window();

        let Some(resolved_font_style) = window.resolved_font_style_query(node, value.to_string())
        else {
            // This will happen when there is a syntax error.
            return;
        };
        self.state.borrow_mut().font_style = Some(resolved_font_style);
    }

    fn font_style(&self) -> ServoArc<Font> {
        self.state
            .borrow()
            .font_style
            .clone()
            .unwrap_or_else(|| ServoArc::new(Font::initial_values()))
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-font
    pub(super) fn font(&self) -> DOMString {
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
    pub(super) fn text_align(&self) -> CanvasTextAlign {
        self.state.borrow().text_align
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-textalign
    pub(super) fn set_text_align(&self, value: CanvasTextAlign) {
        self.state.borrow_mut().text_align = value;
    }

    pub(super) fn text_baseline(&self) -> CanvasTextBaseline {
        self.state.borrow().text_baseline
    }

    pub(super) fn set_text_baseline(&self, value: CanvasTextBaseline) {
        self.state.borrow_mut().text_baseline = value;
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-direction
    pub(super) fn direction(&self) -> CanvasDirection {
        self.state.borrow().direction
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-direction
    pub(super) fn set_direction(&self, value: CanvasDirection) {
        self.state.borrow_mut().direction = value;
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-linewidth
    pub(super) fn line_width(&self) -> f64 {
        self.state.borrow().line_width
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-linewidth
    pub(super) fn set_line_width(&self, width: f64) {
        if !width.is_finite() || width <= 0.0 {
            return;
        }

        self.state.borrow_mut().line_width = width;
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-linecap
    pub(super) fn line_cap(&self) -> CanvasLineCap {
        match self.state.borrow().line_cap {
            LineCapStyle::Butt => CanvasLineCap::Butt,
            LineCapStyle::Round => CanvasLineCap::Round,
            LineCapStyle::Square => CanvasLineCap::Square,
        }
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-linecap
    pub(super) fn set_line_cap(&self, cap: CanvasLineCap) {
        let line_cap = match cap {
            CanvasLineCap::Butt => LineCapStyle::Butt,
            CanvasLineCap::Round => LineCapStyle::Round,
            CanvasLineCap::Square => LineCapStyle::Square,
        };
        self.state.borrow_mut().line_cap = line_cap;
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-linejoin
    pub(super) fn line_join(&self) -> CanvasLineJoin {
        match self.state.borrow().line_join {
            LineJoinStyle::Round => CanvasLineJoin::Round,
            LineJoinStyle::Bevel => CanvasLineJoin::Bevel,
            LineJoinStyle::Miter => CanvasLineJoin::Miter,
        }
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-linejoin
    pub(super) fn set_line_join(&self, join: CanvasLineJoin) {
        let line_join = match join {
            CanvasLineJoin::Round => LineJoinStyle::Round,
            CanvasLineJoin::Bevel => LineJoinStyle::Bevel,
            CanvasLineJoin::Miter => LineJoinStyle::Miter,
        };
        self.state.borrow_mut().line_join = line_join;
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-miterlimit
    pub(super) fn miter_limit(&self) -> f64 {
        self.state.borrow().miter_limit
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-miterlimit
    pub(super) fn set_miter_limit(&self, limit: f64) {
        if !limit.is_finite() || limit <= 0.0 {
            return;
        }

        self.state.borrow_mut().miter_limit = limit;
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-context-2d-getlinedash>
    pub(super) fn line_dash(&self) -> Vec<f64> {
        // > return a sequence whose values are the values of
        // > the object's dash list, in the same order.
        self.state.borrow().line_dash.clone()
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-context-2d-setlinedash>
    pub(super) fn set_line_dash(&self, segments: Vec<f64>) {
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
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-context-2d-linedashoffset>
    pub(super) fn line_dash_offset(&self) -> f64 {
        // > On getting, it must return the current value.
        self.state.borrow().line_dash_offset
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-context-2d-linedashoffset?
    pub(super) fn set_line_dash_offset(&self, offset: f64) {
        // > On setting, infinite and NaN values must be ignored,
        // > leaving the value unchanged;
        if !offset.is_finite() {
            return;
        }

        // > other values must change the current value to the new value.
        self.state.borrow_mut().line_dash_offset = offset;
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-createimagedata
    pub(super) fn create_image_data(
        &self,
        global: &GlobalScope,
        sw: i32,
        sh: i32,
        can_gc: CanGc,
    ) -> Fallible<DomRoot<ImageData>> {
        if sw == 0 || sh == 0 {
            return Err(Error::IndexSize(None));
        }
        ImageData::new(global, sw.unsigned_abs(), sh.unsigned_abs(), None, can_gc)
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-createimagedata
    pub(super) fn create_image_data_(
        &self,
        global: &GlobalScope,
        imagedata: &ImageData,
        can_gc: CanGc,
    ) -> Fallible<DomRoot<ImageData>> {
        ImageData::new(global, imagedata.Width(), imagedata.Height(), None, can_gc)
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-getimagedata
    #[expect(clippy::too_many_arguments)]
    pub(super) fn get_image_data(
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
            return Err(Error::IndexSize(None));
        }

        if !self.origin_is_clean() {
            return Err(Error::Security(None));
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
            let (sender, receiver) = generic_channel::channel().unwrap();
            self.send_canvas_2d_msg(Canvas2dMsg::GetImageData(Some(read_rect), sender));

            let mut snapshot = receiver.recv().unwrap().to_owned();
            snapshot.transform(
                SnapshotAlphaMode::Transparent {
                    premultiplied: false,
                },
                SnapshotPixelFormat::RGBA,
            );
            Some(snapshot.into())
        } else {
            None
        };

        ImageData::new(global, size.width, size.height, data, can_gc)
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-putimagedata
    pub(super) fn put_image_data(
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
    #[expect(clippy::too_many_arguments)]
    pub(super) fn put_image_data_(
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
        let snapshot = imagedata.get_snapshot_rect(Rect::new(src_rect.origin, dst_rect.size));
        self.send_canvas_2d_msg(Canvas2dMsg::PutImageData(dst_rect, snapshot.to_shared()));
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-drawimage
    pub(super) fn draw_image(
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
    pub(super) fn draw_image_(
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
    #[expect(clippy::too_many_arguments)]
    pub(super) fn draw_image__(
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
    pub(super) fn begin_path(&self) {
        *self.current_default_path.borrow_mut() = Path::new();
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-fill
    pub(super) fn fill(&self, fill_rule: CanvasFillRule) {
        let path = self.current_default_path.borrow().clone();
        self.fill_(path, fill_rule);
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-fill
    pub(super) fn fill_(&self, path: Path, fill_rule: CanvasFillRule) {
        let style = self.state.borrow().fill_style.to_fill_or_stroke_style();
        self.send_canvas_2d_msg(Canvas2dMsg::FillPath(
            style,
            path,
            fill_rule.convert(),
            self.state.borrow().shadow_options(),
            self.state.borrow().composition_options(),
            self.state.borrow().transform,
        ));
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-stroke
    pub(super) fn stroke(&self) {
        let path = self.current_default_path.borrow().clone();
        self.stroke_(path);
    }

    pub(super) fn stroke_(&self, path: Path) {
        let style = self.state.borrow().stroke_style.to_fill_or_stroke_style();
        self.send_canvas_2d_msg(Canvas2dMsg::StrokePath(
            path,
            style,
            self.state.borrow().line_options(),
            self.state.borrow().shadow_options(),
            self.state.borrow().composition_options(),
            self.state.borrow().transform,
        ));
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-clip
    pub(super) fn clip(&self, fill_rule: CanvasFillRule) {
        let path = self.current_default_path.borrow().clone();
        self.clip_(path, fill_rule);
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-clip
    pub(super) fn clip_(&self, path: Path, fill_rule: CanvasFillRule) {
        self.state.borrow_mut().clips_pushed += 1;
        self.send_canvas_2d_msg(Canvas2dMsg::ClipPath(
            path,
            fill_rule.convert(),
            self.state.borrow().transform,
        ));
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-ispointinpath
    pub(super) fn is_point_in_path(
        &self,
        global: &GlobalScope,
        x: f64,
        y: f64,
        fill_rule: CanvasFillRule,
    ) -> bool {
        let mut path = self.current_default_path.borrow().clone();
        path.transform(self.state.borrow().transform.cast());
        self.is_point_in_path_(global, path, x, y, fill_rule)
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-ispointinpath
    pub(super) fn is_point_in_path_(
        &self,
        _global: &GlobalScope,
        path: Path,
        x: f64,
        y: f64,
        fill_rule: CanvasFillRule,
    ) -> bool {
        let fill_rule = match fill_rule {
            CanvasFillRule::Nonzero => FillRule::Nonzero,
            CanvasFillRule::Evenodd => FillRule::Evenodd,
        };
        path.is_point_in_path(x, y, fill_rule)
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-scale
    pub(super) fn scale(&self, x: f64, y: f64) {
        if !(x.is_finite() && y.is_finite()) {
            return;
        }

        let transform = self.state.borrow().transform;
        self.update_transform(transform.pre_scale(x, y))
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-rotate
    pub(super) fn rotate(&self, angle: f64) {
        if angle == 0.0 || !angle.is_finite() {
            return;
        }

        let (sin, cos) = (angle.sin(), angle.cos());
        let transform = self.state.borrow().transform;
        self.update_transform(Transform2D::new(cos, sin, -sin, cos, 0.0, 0.0).then(&transform))
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-translate
    pub(super) fn translate(&self, x: f64, y: f64) {
        if !(x.is_finite() && y.is_finite()) {
            return;
        }

        let transform = self.state.borrow().transform;
        self.update_transform(transform.pre_translate(vec2(x, y)))
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-transform
    pub(super) fn transform(&self, a: f64, b: f64, c: f64, d: f64, e: f64, f: f64) {
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
        self.update_transform(Transform2D::new(a, b, c, d, e, f).then(&transform))
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-gettransform
    pub(super) fn get_transform(&self, global: &GlobalScope, can_gc: CanGc) -> DomRoot<DOMMatrix> {
        let transform = self.state.borrow_mut().transform;
        DOMMatrix::new(global, true, transform.to_3d(), can_gc)
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-context-2d-settransform>
    pub(super) fn set_transform(&self, a: f64, b: f64, c: f64, d: f64, e: f64, f: f64) {
        // Step 1. If any of the arguments are infinite or NaN, then return.
        if !a.is_finite() ||
            !b.is_finite() ||
            !c.is_finite() ||
            !d.is_finite() ||
            !e.is_finite() ||
            !f.is_finite()
        {
            return;
        }

        // Step 2. Reset the current transformation matrix to the matrix described by:
        self.update_transform(Transform2D::new(a, b, c, d, e, f))
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-context-2d-settransform-matrix>
    pub(super) fn set_transform_(&self, transform: &DOMMatrix2DInit) -> ErrorResult {
        // Step 1. Let matrix be the result of creating a DOMMatrix from the 2D
        // dictionary transform.
        let matrix = dommatrix2dinit_to_matrix(transform)?;

        // Step 2. If one or more of matrix's m11 element, m12 element, m21
        // element, m22 element, m41 element, or m42 element are infinite or
        // NaN, then return.
        if !matrix.m11.is_finite() ||
            !matrix.m12.is_finite() ||
            !matrix.m21.is_finite() ||
            !matrix.m22.is_finite() ||
            !matrix.m31.is_finite() ||
            !matrix.m32.is_finite()
        {
            return Ok(());
        }

        // Step 3. Reset the current transformation matrix to matrix.
        self.update_transform(matrix.cast());
        Ok(())
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-resettransform
    pub(super) fn reset_transform(&self) {
        self.update_transform(Transform2D::identity())
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-closepath
    pub(super) fn close_path(&self) {
        self.current_default_path.borrow_mut().close_path();
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-moveto
    pub(super) fn move_to(&self, x: f64, y: f64) {
        self.current_default_path.borrow_mut().move_to(x, y);
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-lineto
    pub(super) fn line_to(&self, x: f64, y: f64) {
        self.current_default_path.borrow_mut().line_to(x, y);
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-rect
    pub(super) fn rect(&self, x: f64, y: f64, width: f64, height: f64) {
        self.current_default_path
            .borrow_mut()
            .rect(x, y, width, height);
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-quadraticcurveto
    pub(super) fn quadratic_curve_to(&self, cpx: f64, cpy: f64, x: f64, y: f64) {
        self.current_default_path
            .borrow_mut()
            .quadratic_curve_to(cpx, cpy, x, y);
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-beziercurveto
    pub(super) fn bezier_curve_to(
        &self,
        cp1x: f64,
        cp1y: f64,
        cp2x: f64,
        cp2y: f64,
        x: f64,
        y: f64,
    ) {
        self.current_default_path
            .borrow_mut()
            .bezier_curve_to(cp1x, cp1y, cp2x, cp2y, x, y);
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-arc
    pub(super) fn arc(
        &self,
        x: f64,
        y: f64,
        r: f64,
        start: f64,
        end: f64,
        ccw: bool,
    ) -> ErrorResult {
        self.current_default_path
            .borrow_mut()
            .arc(x, y, r, start, end, ccw)
            .map_err(|_| Error::IndexSize(None))
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-arcto
    pub(super) fn arc_to(&self, cp1x: f64, cp1y: f64, cp2x: f64, cp2y: f64, r: f64) -> ErrorResult {
        self.current_default_path
            .borrow_mut()
            .arc_to(cp1x, cp1y, cp2x, cp2y, r)
            .map_err(|_| Error::IndexSize(None))
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-context-2d-ellipse>
    #[expect(clippy::too_many_arguments)]
    pub(super) fn ellipse(
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
        self.current_default_path
            .borrow_mut()
            .ellipse(x, y, rx, ry, rotation, start, end, ccw)
            .map_err(|_| Error::IndexSize(None))
    }

    fn text_with_size(
        &self,
        global_scope: &GlobalScope,
        text: &str,
        origin: Point2D<f64>,
        size: f64,
        max_width: Option<f64>,
    ) -> Option<(Rect<f64>, Vec<TextRun>)> {
        let Some(font_context) = global_scope.font_context() else {
            warn!("Tried to paint to a canvas of GlobalScope without a FontContext.");
            return None;
        };

        // Step 1: If maxWidth was provided but is less than or equal to zero or equal to NaN, then return an empty array.
        if max_width.is_some_and(|max_width| max_width.is_nan() || max_width <= 0.) {
            return None;
        }

        // > Step 2: Replace all ASCII whitespace in text with U+0020 SPACE characters.
        let text = replace_ascii_whitespace(text);

        // > Step 3: Let font be the current font of target, as given by that object's font
        // > attribute.
        let font_style = self.font_style();
        let font_group = font_context.font_group_with_size(font_style, Au::from_f64_px(size));
        let Some(first_font) = font_group.first(font_context) else {
            warn!("Could not render canvas text, because there was no first font.");
            return None;
        };

        let runs = self.build_unshaped_text_runs(font_context, &text, &font_group);

        // TODO: This doesn't do any kind of line layout at all. In particular, there needs
        // to be some alignment along a baseline and also support for bidi text.
        let mut total_advance = 0.0;
        let mut shaped_runs: Vec<_> = runs
            .into_iter()
            .filter_map(|unshaped_text_run| {
                let text_run = unshaped_text_run.into_shaped_text_run(total_advance)?;
                total_advance += text_run.advance;
                Some(text_run)
            })
            .collect();

        // > Step 6: If maxWidth was provided and the hypothetical width of the inline box in the
        // > hypothetical line box is greater than maxWidth CSS pixels, then change font to have a
        // > more condensed font (if one is available or if a reasonably readable one can be
        // > synthesized by applying a horizontal scale factor to the font) or a smaller font, and
        // > return to the previous step.
        //
        // TODO: We only try decreasing the font size here. Eventually it would make sense to use
        // other methods to try to decrease the size, such as finding a narrower font or decreasing
        // spacing.
        let total_advance = total_advance as f64;
        if let Some(max_width) = max_width {
            let new_size = (max_width / total_advance * size).floor().max(5.);
            if total_advance > max_width && new_size != size {
                return self.text_with_size(global_scope, &text, origin, new_size, Some(max_width));
            }
        }

        // > Step 7: Find the anchor point for the line of text.
        let start =
            self.find_anchor_point_for_line_of_text(origin, &first_font.metrics, total_advance);

        // > Step 8: Let result be an array constructed by iterating over each glyph in the inline box
        // > from left to right (if any), adding to the array, for each glyph, the shape of the glyph
        // > as it is in the inline box, positioned on a coordinate space using CSS pixels with its
        // > origin is at the anchor point.
        let mut bounds = None;
        for text_run in shaped_runs.iter_mut() {
            for glyph_and_position in text_run.glyphs_and_positions.iter_mut() {
                glyph_and_position.point += Vector2D::new(start.x as f32, start.y as f32);
            }
            bounds
                .get_or_insert(text_run.bounds)
                .union(&text_run.bounds);
        }

        Some((
            bounds
                .unwrap_or_default()
                .translate(start.to_vector().cast_unit()),
            shaped_runs,
        ))
    }

    fn build_unshaped_text_runs<'text>(
        &self,
        font_context: &FontContext,
        text: &'text str,
        font_group: &FontGroup,
    ) -> Vec<UnshapedTextRun<'text>> {
        let mut runs = Vec::new();
        let mut current_text_run = UnshapedTextRun::default();
        let mut current_text_run_start_index = 0;

        for (index, character) in text.char_indices() {
            // TODO: This should ultimately handle emoji variation selectors, but raqote does not yet
            // have support for color glyphs.
            let script = Script::from(character);
            let font =
                font_group.find_by_codepoint(font_context, character, None, XLang("".into()));

            if !current_text_run.script_and_font_compatible(script, &font) {
                let previous_text_run = std::mem::replace(
                    &mut current_text_run,
                    UnshapedTextRun {
                        font: font.clone(),
                        script,
                        ..Default::default()
                    },
                );
                current_text_run_start_index = index;
                runs.push(previous_text_run)
            }

            current_text_run.string =
                &text[current_text_run_start_index..index + character.len_utf8()];
        }

        runs.push(current_text_run);
        runs
    }

    /// Find the *anchor_point* for the given parameters of a line of text.
    /// See <https://html.spec.whatwg.org/multipage/#text-preparation-algorithm>.
    fn find_anchor_point_for_line_of_text(
        &self,
        origin: Point2D<f64>,
        metrics: &FontMetrics,
        width: f64,
    ) -> Point2D<f64> {
        let state = self.state.borrow();
        let is_rtl = match state.direction {
            CanvasDirection::Ltr => false,
            CanvasDirection::Rtl => true,
            CanvasDirection::Inherit => false, // TODO: resolve direction wrt to canvas element
        };

        let text_align = match self.text_align() {
            CanvasTextAlign::Start if is_rtl => CanvasTextAlign::Right,
            CanvasTextAlign::Start => CanvasTextAlign::Left,
            CanvasTextAlign::End if is_rtl => CanvasTextAlign::Left,
            CanvasTextAlign::End => CanvasTextAlign::Right,
            text_align => text_align,
        };
        let anchor_x = match text_align {
            CanvasTextAlign::Center => -width / 2.,
            CanvasTextAlign::Right => -width,
            _ => 0.,
        };

        let ascent = metrics.ascent.to_f64_px();
        let descent = metrics.descent.to_f64_px();
        let anchor_y = match self.text_baseline() {
            CanvasTextBaseline::Top => ascent,
            CanvasTextBaseline::Hanging => ascent * HANGING_BASELINE_DEFAULT,
            CanvasTextBaseline::Ideographic => -descent * IDEOGRAPHIC_BASELINE_DEFAULT,
            CanvasTextBaseline::Middle => (ascent - descent) / 2.,
            CanvasTextBaseline::Alphabetic => 0.,
            CanvasTextBaseline::Bottom => -descent,
        };

        origin + Vector2D::new(anchor_x, anchor_y)
    }
}

impl Drop for CanvasState {
    fn drop(&mut self) {
        if let Err(err) = self
            .canvas_thread_sender
            .send(CanvasMsg::Close(self.canvas_id))
        {
            warn!("Could not close canvas: {}", err)
        }
    }
}

#[derive(Default)]
struct UnshapedTextRun<'a> {
    font: Option<FontRef>,
    script: Script,
    string: &'a str,
}

impl UnshapedTextRun<'_> {
    fn script_and_font_compatible(&self, script: Script, other_font: &Option<FontRef>) -> bool {
        if self.script != script {
            return false;
        }

        match (&self.font, other_font) {
            (Some(font_a), Some(font_b)) => font_a.identifier() == font_b.identifier(),
            (None, None) => true,
            _ => false,
        }
    }

    fn into_shaped_text_run(self, previous_advance: f32) -> Option<TextRun> {
        let font = self.font?;
        if self.string.is_empty() {
            return None;
        }

        let word_spacing = Au::from_f64_px(
            font.glyph_index(' ')
                .map(|glyph_id| font.glyph_h_advance(glyph_id))
                .unwrap_or(LAST_RESORT_GLYPH_ADVANCE),
        );
        let options = ShapingOptions {
            letter_spacing: None,
            word_spacing,
            script: self.script,
            flags: ShapingFlags::empty(),
        };

        let glyphs = font.shape_text(self.string, &options);

        let mut advance = 0.0;
        let mut bounds = None;
        let glyphs_and_positions = glyphs
            .glyphs()
            .map(|glyph| {
                let glyph_offset = glyph.offset().unwrap_or(Point2D::zero());
                let glyph_and_position = GlyphAndPosition {
                    id: glyph.id(),
                    point: Point2D::new(previous_advance + advance, glyph_offset.y.to_f32_px()),
                };

                let glyph_bounds = font
                    .typographic_bounds(glyph.id())
                    .translate(Vector2D::new(advance + previous_advance, 0.0));
                bounds = Some(bounds.get_or_insert(glyph_bounds).union(&glyph_bounds));

                advance += glyph.advance().to_f32_px();

                glyph_and_position
            })
            .collect();

        let identifier = font.identifier();
        let font_data = match &identifier {
            FontIdentifier::Local(_) => None,
            FontIdentifier::Web(_) => Some(font.font_data_and_index().ok()?),
        }
        .cloned();
        let canvas_font = CanvasFont {
            identifier,
            data: font_data,
        };

        Some(TextRun {
            font: canvas_font,
            pt_size: font.descriptor.pt_size.to_f32_px(),
            glyphs_and_positions,
            advance,
            bounds: bounds.unwrap_or_default().cast(),
        })
    }
}

pub(super) fn parse_color(
    canvas: Option<&HTMLCanvasElement>,
    string: &DOMString,
) -> Result<AbsoluteColor, ()> {
    let string = string.str();
    let mut input = ParserInput::new(&string);
    let mut parser = Parser::new(&mut input);
    let url = Url::parse("about:blank").unwrap().into();
    let context =
        parser_context_for_anonymous_content(CssRuleType::Style, ParsingMode::DEFAULT, &url);
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
                    match canvas_element.style() {
                        Some(ref s) if canvas_element.has_css_layout_box() => {
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
pub(super) fn is_rect_valid(rect: Rect<f64>) -> bool {
    rect.size.width > 0.0 && rect.size.height > 0.0
}

// https://html.spec.whatwg.org/multipage/#serialisation-of-a-color
pub(super) fn serialize<W>(color: &AbsoluteColor, dest: &mut W) -> fmt::Result
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

pub(super) fn adjust_size_sign(
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

impl Convert<FillRule> for CanvasFillRule {
    fn convert(self) -> FillRule {
        match self {
            CanvasFillRule::Nonzero => FillRule::Nonzero,
            CanvasFillRule::Evenodd => FillRule::Evenodd,
        }
    }
}

fn replace_ascii_whitespace(text: &str) -> String {
    text.chars()
        .map(|c| match c {
            ' ' | '\t' | '\n' | '\r' | '\x0C' => '\x20',
            _ => c,
        })
        .collect()
}

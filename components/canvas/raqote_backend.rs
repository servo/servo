/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::borrow::Cow;
use std::cell::RefCell;
use std::collections::HashMap;

use canvas_traits::canvas::*;
use cssparser::color::clamp_unit_f32;
use euclid::default::{Point2D, Rect, Size2D, Transform2D, Vector2D};
use font_kit::font::Font;
use fonts::{ByteIndex, FontIdentifier, FontTemplateRefMethods};
use log::warn;
use range::Range;
use raqote::PathOp;
use style::color::AbsoluteColor;

use crate::backend::{
    Backend, DrawOptionsHelpers, GenericDrawTarget, GenericPathBuilder, PathHelpers,
    PatternHelpers, StrokeOptionsHelpers,
};
use crate::canvas_data::{CanvasPaintState, Filter, TextRun};

thread_local! {
    /// The shared font cache used by all canvases that render on a thread. It would be nicer
    /// to have a global cache, but it looks like font-kit uses a per-thread FreeType, so
    /// in order to ensure that fonts are particular to a thread we have to make our own
    /// cache thread local as well.
    static SHARED_FONT_CACHE: RefCell<HashMap<FontIdentifier, Font>> = RefCell::default();
}

#[derive(Clone, Default)]
pub(crate) struct RaqoteBackend;

impl Backend for RaqoteBackend {
    type Pattern<'a> = Pattern<'a>;
    type StrokeOptions = raqote::StrokeStyle;
    type Color = raqote::SolidSource;
    type DrawOptions = raqote::DrawOptions;
    type CompositionOp = raqote::BlendMode;
    type DrawTarget = raqote::DrawTarget;
    type PathBuilder = PathBuilder;
    type SourceSurface = Vec<u8>; // TODO: See if we can avoid the alloc (probably?)
    type Path = raqote::Path;
    type GradientStop = raqote::GradientStop;
    type GradientStops = Vec<raqote::GradientStop>;

    fn get_composition_op(&self, opts: &Self::DrawOptions) -> Self::CompositionOp {
        opts.blend_mode
    }

    fn need_to_draw_shadow(&self, color: &Self::Color) -> bool {
        color.a != 0
    }

    fn set_shadow_color(&mut self, color: AbsoluteColor, state: &mut CanvasPaintState<'_, Self>) {
        state.shadow_color = color.to_raqote_style();
    }

    fn set_fill_style(
        &mut self,
        style: FillOrStrokeStyle,
        state: &mut CanvasPaintState<'_, Self>,
        _drawtarget: &Self::DrawTarget,
    ) {
        if let Some(pattern) = style.to_raqote_pattern() {
            state.fill_style = pattern;
        }
    }

    fn set_stroke_style(
        &mut self,
        style: FillOrStrokeStyle,
        state: &mut CanvasPaintState<'_, Self>,
        _drawtarget: &Self::DrawTarget,
    ) {
        if let Some(pattern) = style.to_raqote_pattern() {
            state.stroke_style = pattern;
        }
    }

    fn set_global_composition(
        &mut self,
        op: CompositionOrBlending,
        state: &mut CanvasPaintState<'_, Self>,
    ) {
        state.draw_options.blend_mode = op.to_raqote_style();
    }

    fn create_drawtarget(&self, size: Size2D<u64>) -> Self::DrawTarget {
        raqote::DrawTarget::new(size.width as i32, size.height as i32)
    }

    fn new_paint_state<'a>(&self) -> CanvasPaintState<'a, Self> {
        let pattern = Pattern::Color(255, 0, 0, 0);
        CanvasPaintState {
            draw_options: raqote::DrawOptions::new(),
            fill_style: pattern.clone(),
            stroke_style: pattern,
            stroke_opts: Default::default(),
            transform: Transform2D::identity(),
            shadow_offset_x: 0.0,
            shadow_offset_y: 0.0,
            shadow_blur: 0.0,
            shadow_color: raqote::SolidSource::from_unpremultiplied_argb(0, 0, 0, 0),
            font_style: None,
            text_align: TextAlign::default(),
            text_baseline: TextBaseline::default(),
            _backend: std::marker::PhantomData,
        }
    }
}

#[derive(Clone)]
pub enum Pattern<'a> {
    // argb
    Color(u8, u8, u8, u8),
    LinearGradient(LinearGradientPattern),
    RadialGradient(RadialGradientPattern),
    Surface(SurfacePattern<'a>),
}

impl Pattern<'_> {
    fn set_transform(&mut self, transform: Transform2D<f32>) {
        match self {
            Pattern::Surface(pattern) => pattern.set_transform(transform),
            Pattern::LinearGradient(..) | Pattern::RadialGradient(..) | Pattern::Color(..) => {
                warn!("transform not supported")
            },
        }
    }
}

#[derive(Clone)]
pub struct LinearGradientPattern {
    gradient: raqote::Gradient,
    start: Point2D<f32>,
    end: Point2D<f32>,
}

impl LinearGradientPattern {
    fn new(start: Point2D<f32>, end: Point2D<f32>, stops: Vec<raqote::GradientStop>) -> Self {
        LinearGradientPattern {
            gradient: raqote::Gradient { stops },
            start,
            end,
        }
    }
}

#[derive(Clone)]
pub struct RadialGradientPattern {
    gradient: raqote::Gradient,
    center1: Point2D<f32>,
    radius1: f32,
    center2: Point2D<f32>,
    radius2: f32,
}

impl RadialGradientPattern {
    fn new(
        center1: Point2D<f32>,
        radius1: f32,
        center2: Point2D<f32>,
        radius2: f32,
        stops: Vec<raqote::GradientStop>,
    ) -> Self {
        RadialGradientPattern {
            gradient: raqote::Gradient { stops },
            center1,
            radius1,
            center2,
            radius2,
        }
    }
}

#[derive(Clone)]
pub struct SurfacePattern<'a> {
    image: raqote::Image<'a>,
    filter: raqote::FilterMode,
    extend: raqote::ExtendMode,
    repeat: Repetition,
    transform: Transform2D<f32>,
}

impl<'a> SurfacePattern<'a> {
    fn new(image: raqote::Image<'a>, filter: raqote::FilterMode, repeat: Repetition) -> Self {
        let extend = match repeat {
            Repetition::NoRepeat => raqote::ExtendMode::Pad,
            Repetition::RepeatX | Repetition::RepeatY | Repetition::Repeat => {
                raqote::ExtendMode::Repeat
            },
        };
        SurfacePattern {
            image,
            filter,
            extend,
            repeat,
            transform: Transform2D::identity(),
        }
    }
    fn set_transform(&mut self, transform: Transform2D<f32>) {
        self.transform = transform;
    }
    pub fn size(&self) -> Size2D<f32> {
        Size2D::new(self.image.width as f32, self.image.height as f32)
    }
    pub fn repetition(&self) -> &Repetition {
        &self.repeat
    }
}

#[derive(Clone)]
pub enum Repetition {
    Repeat,
    RepeatX,
    RepeatY,
    NoRepeat,
}

impl Repetition {
    fn from_xy(repeat_x: bool, repeat_y: bool) -> Self {
        if repeat_x && repeat_y {
            Repetition::Repeat
        } else if repeat_x {
            Repetition::RepeatX
        } else if repeat_y {
            Repetition::RepeatY
        } else {
            Repetition::NoRepeat
        }
    }
}

pub fn source<'a>(pattern: &Pattern<'a>) -> raqote::Source<'a> {
    match pattern {
        Pattern::Color(a, r, g, b) => raqote::Source::Solid(
            raqote::SolidSource::from_unpremultiplied_argb(*a, *r, *g, *b),
        ),
        Pattern::LinearGradient(pattern) => raqote::Source::new_linear_gradient(
            pattern.gradient.clone(),
            pattern.start,
            pattern.end,
            raqote::Spread::Pad,
        ),
        Pattern::RadialGradient(pattern) => raqote::Source::new_two_circle_radial_gradient(
            pattern.gradient.clone(),
            pattern.center1,
            pattern.radius1,
            pattern.center2,
            pattern.radius2,
            raqote::Spread::Pad,
        ),
        Pattern::Surface(pattern) => raqote::Source::Image(
            pattern.image,
            pattern.extend,
            pattern.filter,
            pattern.transform,
        ),
    }
}

impl PatternHelpers for Pattern<'_> {
    fn is_zero_size_gradient(&self) -> bool {
        match self {
            Pattern::RadialGradient(pattern) => {
                let centers_equal = pattern.center1 == pattern.center2;
                let radii_equal = pattern.radius1 == pattern.radius2;
                (centers_equal && radii_equal) || pattern.gradient.stops.is_empty()
            },
            Pattern::LinearGradient(pattern) => {
                (pattern.start == pattern.end) || pattern.gradient.stops.is_empty()
            },
            Pattern::Color(..) | Pattern::Surface(..) => false,
        }
    }

    fn draw_rect(&self, rect: &Rect<f32>) -> Rect<f32> {
        match self {
            Pattern::Surface(pattern) => {
                let pattern_rect = Rect::new(Point2D::origin(), pattern.size());
                let mut draw_rect = rect.intersection(&pattern_rect).unwrap_or(Rect::zero());

                match pattern.repetition() {
                    Repetition::NoRepeat => {
                        draw_rect.size.width = draw_rect.size.width.min(pattern_rect.size.width);
                        draw_rect.size.height = draw_rect.size.height.min(pattern_rect.size.height);
                    },
                    Repetition::RepeatX => {
                        draw_rect.size.width = rect.size.width;
                        draw_rect.size.height = draw_rect.size.height.min(pattern_rect.size.height);
                    },
                    Repetition::RepeatY => {
                        draw_rect.size.height = rect.size.height;
                        draw_rect.size.width = draw_rect.size.width.min(pattern_rect.size.width);
                    },
                    Repetition::Repeat => {
                        draw_rect = *rect;
                    },
                }

                draw_rect
            },
            Pattern::Color(..) | Pattern::LinearGradient(..) | Pattern::RadialGradient(..) => *rect,
        }
    }
}

impl StrokeOptionsHelpers for raqote::StrokeStyle {
    fn set_line_width(&mut self, _val: f32) {
        self.width = _val;
    }
    fn set_miter_limit(&mut self, _val: f32) {
        self.miter_limit = _val;
    }
    fn set_line_join(&mut self, val: LineJoinStyle) {
        self.join = val.to_raqote_style();
    }
    fn set_line_cap(&mut self, val: LineCapStyle) {
        self.cap = val.to_raqote_style();
    }
    fn set_line_dash(&mut self, items: Vec<f32>) {
        self.dash_array = items;
    }
    fn set_line_dash_offset(&mut self, offset: f32) {
        self.dash_offset = offset;
    }
}

impl DrawOptionsHelpers for raqote::DrawOptions {
    fn set_alpha(&mut self, val: f32) {
        self.alpha = val;
    }
}

impl PathHelpers<RaqoteBackend> for raqote::Path {
    fn transformed_copy_to_builder(&self, transform: &Transform2D<f32>) -> PathBuilder {
        PathBuilder(Some(raqote::PathBuilder::from(
            self.clone().transform(transform),
        )))
    }

    fn contains_point(&self, x: f64, y: f64, path_transform: &Transform2D<f32>) -> bool {
        self.clone()
            .transform(path_transform)
            .contains_point(0.1, x as f32, y as f32)
    }

    fn copy_to_builder(&self) -> PathBuilder {
        PathBuilder(Some(raqote::PathBuilder::from(self.clone())))
    }
}

fn create_gradient_stops(gradient_stops: Vec<CanvasGradientStop>) -> Vec<raqote::GradientStop> {
    let mut stops = gradient_stops
        .into_iter()
        .map(|item| item.to_raqote())
        .collect::<Vec<raqote::GradientStop>>();
    // https://www.w3.org/html/test/results/2dcontext/annotated-spec/canvas.html#testrefs.2d.gradient.interpolate.overlap
    stops.sort_by(|a, b| a.position.partial_cmp(&b.position).unwrap());
    stops
}

impl GenericDrawTarget<RaqoteBackend> for raqote::DrawTarget {
    fn clear_rect(&mut self, rect: &Rect<f32>) {
        let mut pb = raqote::PathBuilder::new();
        pb.rect(
            rect.origin.x,
            rect.origin.y,
            rect.size.width,
            rect.size.height,
        );
        let mut options = raqote::DrawOptions::new();
        options.blend_mode = raqote::BlendMode::Clear;
        let pattern = Pattern::Color(0, 0, 0, 0);
        <Self as GenericDrawTarget<RaqoteBackend>>::fill(self, &pb.finish(), pattern, &options);
    }
    #[allow(unsafe_code)]
    fn copy_surface(
        &mut self,
        surface: <RaqoteBackend as Backend>::SourceSurface,
        source: Rect<i32>,
        destination: Point2D<i32>,
    ) {
        let mut dt = raqote::DrawTarget::new(source.size.width, source.size.height);
        let data = surface;
        let s = unsafe { std::slice::from_raw_parts(data.as_ptr() as *const u32, data.len() / 4) };
        dt.get_data_mut().copy_from_slice(s);
        raqote::DrawTarget::copy_surface(self, &dt, source.to_box2d(), destination);
    }

    fn create_path_builder(&self) -> <RaqoteBackend as Backend>::PathBuilder {
        PathBuilder::new()
    }
    fn create_similar_draw_target(
        &self,
        size: &Size2D<i32>,
    ) -> <RaqoteBackend as Backend>::DrawTarget {
        raqote::DrawTarget::new(size.width, size.height)
    }
    fn create_source_surface_from_data(
        &self,
        data: &[u8],
    ) -> Option<<RaqoteBackend as Backend>::SourceSurface> {
        Some(data.to_vec())
    }
    #[allow(unsafe_code)]
    fn draw_surface(
        &mut self,
        surface: <RaqoteBackend as Backend>::SourceSurface,
        dest: Rect<f64>,
        source: Rect<f64>,
        filter: Filter,
        draw_options: &<RaqoteBackend as Backend>::DrawOptions,
    ) {
        let surface_data = surface;
        let image = raqote::Image {
            width: source.size.width as i32,
            height: source.size.height as i32,
            data: unsafe {
                std::slice::from_raw_parts(
                    surface_data.as_ptr() as *const u32,
                    surface_data.len() / std::mem::size_of::<u32>(),
                )
            },
        };

        let mut pattern = Pattern::Surface(SurfacePattern::new(
            image,
            filter.to_raqote(),
            Repetition::NoRepeat,
        ));
        let transform =
            raqote::Transform::translation(-dest.origin.x as f32, -dest.origin.y as f32)
                .then_scale(
                    image.width as f32 / dest.size.width as f32,
                    image.height as f32 / dest.size.height as f32,
                );
        pattern.set_transform(transform);

        let mut pb = raqote::PathBuilder::new();
        pb.rect(
            dest.origin.x as f32,
            dest.origin.y as f32,
            dest.size.width as f32,
            dest.size.height as f32,
        );

        <Self as GenericDrawTarget<RaqoteBackend>>::fill(self, &pb.finish(), pattern, draw_options);
    }
    fn draw_surface_with_shadow(
        &self,
        _surface: <RaqoteBackend as Backend>::SourceSurface,
        _dest: &Point2D<f32>,
        _color: &<RaqoteBackend as Backend>::Color,
        _offset: &Vector2D<f32>,
        _sigma: f32,
        _operator: <RaqoteBackend as Backend>::CompositionOp,
    ) {
        warn!("no support for drawing shadows");
    }
    fn fill(
        &mut self,
        path: &<RaqoteBackend as Backend>::Path,
        pattern: <RaqoteBackend as Backend>::Pattern<'_>,
        draw_options: &<RaqoteBackend as Backend>::DrawOptions,
    ) {
        match draw_options.blend_mode {
            raqote::BlendMode::Src => {
                self.clear(raqote::SolidSource::from_unpremultiplied_argb(0, 0, 0, 0));
                self.fill(path, &source(&pattern), draw_options);
            },
            raqote::BlendMode::Clear |
            raqote::BlendMode::SrcAtop |
            raqote::BlendMode::DstOut |
            raqote::BlendMode::Add |
            raqote::BlendMode::Xor |
            raqote::BlendMode::DstOver |
            raqote::BlendMode::SrcOver => {
                self.fill(path, &source(&pattern), draw_options);
            },
            raqote::BlendMode::SrcIn |
            raqote::BlendMode::SrcOut |
            raqote::BlendMode::DstIn |
            raqote::BlendMode::DstAtop => {
                let mut options = *draw_options;
                self.push_layer_with_blend(1., options.blend_mode);
                options.blend_mode = raqote::BlendMode::SrcOver;
                self.fill(path, &source(&pattern), &options);
                self.pop_layer();
            },
            _ => warn!("unrecognized blend mode: {:?}", draw_options.blend_mode),
        }
    }

    fn fill_text(
        &mut self,
        text_runs: Vec<TextRun>,
        start: Point2D<f32>,
        pattern: &<RaqoteBackend as Backend>::Pattern<'_>,
        draw_options: &<RaqoteBackend as Backend>::DrawOptions,
    ) {
        let mut advance = 0.;
        for run in text_runs.iter() {
            let mut positions = Vec::new();
            let glyphs = &run.glyphs;
            let ids: Vec<_> = glyphs
                .iter_glyphs_for_byte_range(&Range::new(ByteIndex(0), glyphs.len()))
                .map(|glyph| {
                    let glyph_offset = glyph.offset().unwrap_or(Point2D::zero());
                    positions.push(Point2D::new(
                        advance + start.x + glyph_offset.x.to_f32_px(),
                        start.y + glyph_offset.y.to_f32_px(),
                    ));
                    advance += glyph.advance().to_f32_px();
                    glyph.id()
                })
                .collect();

            // TODO: raqote uses font-kit to rasterize glyphs, but font-kit fails an assertion when
            // using color bitmap fonts in the FreeType backend. For now, simply do not render these
            // type of fonts.
            if run.font.has_color_bitmap_or_colr_table() {
                continue;
            }

            let template = &run.font.template;

            SHARED_FONT_CACHE.with(|font_cache| {
                let identifier = template.identifier();
                if !font_cache.borrow().contains_key(&identifier) {
                    let data = std::sync::Arc::new(run.font.data().as_ref().to_vec());
                    let Ok(font) = Font::from_bytes(data, identifier.index()) else {
                        return;
                    };
                    font_cache.borrow_mut().insert(identifier.clone(), font);
                }

                let font_cache = font_cache.borrow();
                let Some(font) = font_cache.get(&identifier) else {
                    return;
                };

                self.draw_glyphs(
                    font,
                    run.font.descriptor.pt_size.to_f32_px(),
                    &ids,
                    &positions,
                    &source(pattern),
                    draw_options,
                );
            })
        }
    }

    fn fill_rect(
        &mut self,
        rect: &Rect<f32>,
        pattern: <RaqoteBackend as Backend>::Pattern<'_>,
        draw_options: Option<&<RaqoteBackend as Backend>::DrawOptions>,
    ) {
        let mut pb = raqote::PathBuilder::new();
        pb.rect(
            rect.origin.x,
            rect.origin.y,
            rect.size.width,
            rect.size.height,
        );
        let draw_options = if let Some(options) = draw_options {
            *options
        } else {
            raqote::DrawOptions::new()
        };

        <Self as GenericDrawTarget<RaqoteBackend>>::fill(
            self,
            &pb.finish(),
            pattern,
            &draw_options,
        );
    }
    fn get_size(&self) -> Size2D<i32> {
        Size2D::new(self.width(), self.height())
    }
    fn get_transform(&self) -> Transform2D<f32> {
        *self.get_transform()
    }
    fn pop_clip(&mut self) {
        self.pop_clip();
    }
    fn push_clip(&mut self, path: &<RaqoteBackend as Backend>::Path) {
        self.push_clip(path);
    }
    fn set_transform(&mut self, matrix: &Transform2D<f32>) {
        self.set_transform(matrix);
    }
    fn surface(&self) -> <RaqoteBackend as Backend>::SourceSurface {
        self.bytes().to_vec()
    }
    fn stroke(
        &mut self,
        path: &<RaqoteBackend as Backend>::Path,
        pattern: Pattern<'_>,
        stroke_options: &<RaqoteBackend as Backend>::StrokeOptions,
        draw_options: &<RaqoteBackend as Backend>::DrawOptions,
    ) {
        self.stroke(path, &source(&pattern), stroke_options, draw_options);
    }
    fn stroke_line(
        &mut self,
        start: Point2D<f32>,
        end: Point2D<f32>,
        pattern: <RaqoteBackend as Backend>::Pattern<'_>,
        stroke_options: &<RaqoteBackend as Backend>::StrokeOptions,
        draw_options: &<RaqoteBackend as Backend>::DrawOptions,
    ) {
        let mut pb = raqote::PathBuilder::new();
        pb.move_to(start.x, start.y);
        pb.line_to(end.x, end.y);
        let mut stroke_options = stroke_options.clone();
        let cap = match stroke_options.join {
            raqote::LineJoin::Round => raqote::LineCap::Round,
            _ => raqote::LineCap::Butt,
        };
        stroke_options.cap = cap;

        self.stroke(
            &pb.finish(),
            &source(&pattern),
            &stroke_options,
            draw_options,
        );
    }
    fn stroke_rect(
        &mut self,
        rect: &Rect<f32>,
        pattern: <RaqoteBackend as Backend>::Pattern<'_>,
        stroke_options: &<RaqoteBackend as Backend>::StrokeOptions,
        draw_options: &<RaqoteBackend as Backend>::DrawOptions,
    ) {
        let mut pb = raqote::PathBuilder::new();
        pb.rect(
            rect.origin.x,
            rect.origin.y,
            rect.size.width,
            rect.size.height,
        );

        self.stroke(
            &pb.finish(),
            &source(&pattern),
            stroke_options,
            draw_options,
        );
    }
    #[allow(unsafe_code)]
    fn bytes(&self) -> Cow<[u8]> {
        let v = self.get_data();
        Cow::Borrowed(unsafe {
            std::slice::from_raw_parts(v.as_ptr() as *const u8, std::mem::size_of_val(v))
        })
    }
}

impl Filter {
    fn to_raqote(self) -> raqote::FilterMode {
        match self {
            Filter::Bilinear => raqote::FilterMode::Bilinear,
            Filter::Nearest => raqote::FilterMode::Nearest,
        }
    }
}

pub(crate) struct PathBuilder(Option<raqote::PathBuilder>);

impl PathBuilder {
    fn new() -> PathBuilder {
        PathBuilder(Some(raqote::PathBuilder::new()))
    }
}

impl GenericPathBuilder<RaqoteBackend> for PathBuilder {
    fn bezier_curve_to(
        &mut self,
        control_point1: &Point2D<f32>,
        control_point2: &Point2D<f32>,
        control_point3: &Point2D<f32>,
    ) {
        self.0.as_mut().unwrap().cubic_to(
            control_point1.x,
            control_point1.y,
            control_point2.x,
            control_point2.y,
            control_point3.x,
            control_point3.y,
        );
    }

    fn close(&mut self) {
        self.0.as_mut().unwrap().close();
    }

    fn get_current_point(&mut self) -> Option<Point2D<f32>> {
        let path = self.finish();
        self.0 = Some(path.clone().into());

        path.ops.iter().last().and_then(|op| match op {
            PathOp::MoveTo(point) | PathOp::LineTo(point) => Some(Point2D::new(point.x, point.y)),
            PathOp::CubicTo(_, _, point) => Some(Point2D::new(point.x, point.y)),
            PathOp::QuadTo(_, point) => Some(Point2D::new(point.x, point.y)),
            PathOp::Close => None,
        })
    }

    fn line_to(&mut self, point: Point2D<f32>) {
        self.0.as_mut().unwrap().line_to(point.x, point.y);
    }
    fn move_to(&mut self, point: Point2D<f32>) {
        self.0.as_mut().unwrap().move_to(point.x, point.y);
    }
    fn quadratic_curve_to(&mut self, control_point: &Point2D<f32>, end_point: &Point2D<f32>) {
        self.0.as_mut().unwrap().quad_to(
            control_point.x,
            control_point.y,
            end_point.x,
            end_point.y,
        );
    }
    fn finish(&mut self) -> raqote::Path {
        self.0.take().unwrap().finish()
    }
}

pub trait ToRaqoteStyle {
    type Target;

    fn to_raqote_style(self) -> Self::Target;
}

impl ToRaqoteStyle for LineJoinStyle {
    type Target = raqote::LineJoin;

    fn to_raqote_style(self) -> raqote::LineJoin {
        match self {
            LineJoinStyle::Round => raqote::LineJoin::Round,
            LineJoinStyle::Bevel => raqote::LineJoin::Bevel,
            LineJoinStyle::Miter => raqote::LineJoin::Miter,
        }
    }
}

impl ToRaqoteStyle for LineCapStyle {
    type Target = raqote::LineCap;

    fn to_raqote_style(self) -> raqote::LineCap {
        match self {
            LineCapStyle::Butt => raqote::LineCap::Butt,
            LineCapStyle::Round => raqote::LineCap::Round,
            LineCapStyle::Square => raqote::LineCap::Square,
        }
    }
}

pub trait ToRaqotePattern<'a> {
    fn to_raqote_pattern(self) -> Option<Pattern<'a>>;
}

pub trait ToRaqoteGradientStop {
    fn to_raqote(self) -> raqote::GradientStop;
}

impl ToRaqoteGradientStop for CanvasGradientStop {
    fn to_raqote(self) -> raqote::GradientStop {
        let srgb = self.color.into_srgb_legacy();
        let color = raqote::Color::new(
            clamp_unit_f32(srgb.alpha),
            clamp_unit_f32(srgb.components.0),
            clamp_unit_f32(srgb.components.1),
            clamp_unit_f32(srgb.components.2),
        );
        let position = self.offset as f32;
        raqote::GradientStop { position, color }
    }
}

impl ToRaqotePattern<'_> for FillOrStrokeStyle {
    #[allow(unsafe_code)]
    fn to_raqote_pattern(self) -> Option<Pattern<'static>> {
        use canvas_traits::canvas::FillOrStrokeStyle::*;

        match self {
            Color(color) => {
                let srgb = color.into_srgb_legacy();
                Some(Pattern::Color(
                    clamp_unit_f32(srgb.alpha),
                    clamp_unit_f32(srgb.components.0),
                    clamp_unit_f32(srgb.components.1),
                    clamp_unit_f32(srgb.components.2),
                ))
            },
            LinearGradient(style) => {
                let start = Point2D::new(style.x0 as f32, style.y0 as f32);
                let end = Point2D::new(style.x1 as f32, style.y1 as f32);
                let stops = create_gradient_stops(style.stops);
                Some(Pattern::LinearGradient(LinearGradientPattern::new(
                    start, end, stops,
                )))
            },
            RadialGradient(style) => {
                let center1 = Point2D::new(style.x0 as f32, style.y0 as f32);
                let center2 = Point2D::new(style.x1 as f32, style.y1 as f32);
                let stops = create_gradient_stops(style.stops);
                Some(Pattern::RadialGradient(RadialGradientPattern::new(
                    center1,
                    style.r0 as f32,
                    center2,
                    style.r1 as f32,
                    stops,
                )))
            },
            Surface(ref style) => {
                let repeat = Repetition::from_xy(style.repeat_x, style.repeat_y);
                let data = &style.surface_data[..];

                let image = raqote::Image {
                    width: style.surface_size.width as i32,
                    height: style.surface_size.height as i32,
                    data: unsafe {
                        std::slice::from_raw_parts(data.as_ptr() as *const u32, data.len() / 4)
                    },
                };
                Some(Pattern::Surface(SurfacePattern::new(
                    image,
                    raqote::FilterMode::Nearest,
                    repeat,
                )))
            },
        }
    }
}

impl ToRaqoteStyle for AbsoluteColor {
    type Target = raqote::SolidSource;

    fn to_raqote_style(self) -> Self::Target {
        let srgb = self.into_srgb_legacy();
        raqote::SolidSource::from_unpremultiplied_argb(
            clamp_unit_f32(srgb.alpha),
            clamp_unit_f32(srgb.components.0),
            clamp_unit_f32(srgb.components.1),
            clamp_unit_f32(srgb.components.2),
        )
    }
}

impl ToRaqoteStyle for CompositionOrBlending {
    type Target = raqote::BlendMode;

    fn to_raqote_style(self) -> Self::Target {
        match self {
            CompositionOrBlending::Composition(op) => op.to_raqote_style(),
            CompositionOrBlending::Blending(op) => op.to_raqote_style(),
        }
    }
}

impl ToRaqoteStyle for BlendingStyle {
    type Target = raqote::BlendMode;

    fn to_raqote_style(self) -> Self::Target {
        match self {
            BlendingStyle::Multiply => raqote::BlendMode::Multiply,
            BlendingStyle::Screen => raqote::BlendMode::Screen,
            BlendingStyle::Overlay => raqote::BlendMode::Overlay,
            BlendingStyle::Darken => raqote::BlendMode::Darken,
            BlendingStyle::Lighten => raqote::BlendMode::Lighten,
            BlendingStyle::ColorDodge => raqote::BlendMode::ColorDodge,
            BlendingStyle::HardLight => raqote::BlendMode::HardLight,
            BlendingStyle::SoftLight => raqote::BlendMode::SoftLight,
            BlendingStyle::Difference => raqote::BlendMode::Difference,
            BlendingStyle::Exclusion => raqote::BlendMode::Exclusion,
            BlendingStyle::Hue => raqote::BlendMode::Hue,
            BlendingStyle::Saturation => raqote::BlendMode::Saturation,
            BlendingStyle::Color => raqote::BlendMode::Color,
            BlendingStyle::Luminosity => raqote::BlendMode::Luminosity,
            BlendingStyle::ColorBurn => raqote::BlendMode::ColorBurn,
        }
    }
}

impl ToRaqoteStyle for CompositionStyle {
    type Target = raqote::BlendMode;

    fn to_raqote_style(self) -> Self::Target {
        match self {
            CompositionStyle::SrcIn => raqote::BlendMode::SrcIn,
            CompositionStyle::SrcOut => raqote::BlendMode::SrcOut,
            CompositionStyle::SrcOver => raqote::BlendMode::SrcOver,
            CompositionStyle::SrcAtop => raqote::BlendMode::SrcAtop,
            CompositionStyle::DestIn => raqote::BlendMode::DstIn,
            CompositionStyle::DestOut => raqote::BlendMode::DstOut,
            CompositionStyle::DestOver => raqote::BlendMode::DstOver,
            CompositionStyle::DestAtop => raqote::BlendMode::DstAtop,
            CompositionStyle::Copy => raqote::BlendMode::Src,
            CompositionStyle::Lighter => raqote::BlendMode::Add,
            CompositionStyle::Xor => raqote::BlendMode::Xor,
            CompositionStyle::Clear => raqote::BlendMode::Clear,
        }
    }
}

/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::canvas_data::{
    Backend, CanvasPaintState, Color, CompositionOp, DrawOptions, ExtendMode, Filter,
    GenericDrawTarget, GenericPathBuilder, GradientStop, GradientStops, Path, Pattern,
    SourceSurface, StrokeOptions, SurfaceFormat,
};
use crate::canvas_paint_thread::AntialiasMode;
use azure::azure::{AzFloat, AzGradientStop, AzPoint};
use azure::azure_hl;
use azure::azure_hl::SurfacePattern;
use azure::azure_hl::{BackendType, ColorPattern, DrawTarget};
use azure::azure_hl::{CapStyle, JoinStyle};
use azure::azure_hl::{LinearGradientPattern, RadialGradientPattern};
use canvas_traits::canvas::*;
use cssparser::RGBA;
use euclid::default::{Point2D, Rect, Size2D, Transform2D, Vector2D};

use std::marker::PhantomData;

pub struct AzureBackend;

impl Backend for AzureBackend {
    fn get_composition_op(&self, opts: &DrawOptions) -> CompositionOp {
        CompositionOp::Azure(opts.as_azure().composition)
    }

    fn need_to_draw_shadow(&self, color: &Color) -> bool {
        color.as_azure().a != 0.0f32
    }

    fn size_from_pattern(&self, rect: &Rect<f32>, pattern: &Pattern) -> Option<Size2D<f32>> {
        match pattern {
            Pattern::Azure(azure_hl::Pattern::Surface(ref surface), _) => {
                let surface_size = surface.size();
                let size = match (surface.repeat_x, surface.repeat_y) {
                    (true, true) => rect.size,
                    (true, false) => Size2D::new(rect.size.width, surface_size.height as f32),
                    (false, true) => Size2D::new(surface_size.width as f32, rect.size.height),
                    (false, false) => {
                        Size2D::new(surface_size.width as f32, surface_size.height as f32)
                    },
                };
                Some(size)
            },
            Pattern::Azure(..) => None,
        }
    }

    fn set_shadow_color<'a>(&mut self, color: RGBA, state: &mut CanvasPaintState<'a>) {
        state.shadow_color = Color::Azure(color.to_azure_style());
    }

    fn set_fill_style<'a>(
        &mut self,
        style: FillOrStrokeStyle,
        state: &mut CanvasPaintState<'a>,
        drawtarget: &dyn GenericDrawTarget,
    ) {
        if let Some(pattern) = style.to_azure_pattern(drawtarget) {
            state.fill_style = Pattern::Azure(pattern, PhantomData::default())
        }
    }

    fn set_stroke_style<'a>(
        &mut self,
        style: FillOrStrokeStyle,
        state: &mut CanvasPaintState<'a>,
        drawtarget: &dyn GenericDrawTarget,
    ) {
        if let Some(pattern) = style.to_azure_pattern(drawtarget) {
            state.stroke_style = Pattern::Azure(pattern, PhantomData::default())
        }
    }

    fn set_global_composition<'a>(
        &mut self,
        op: CompositionOrBlending,
        state: &mut CanvasPaintState<'a>,
    ) {
        state
            .draw_options
            .as_azure_mut()
            .set_composition_op(op.to_azure_style());
    }

    fn create_drawtarget(&self, size: Size2D<u64>) -> Box<dyn GenericDrawTarget> {
        // FIXME(nox): Why is the size made of i32 values?
        Box::new(DrawTarget::new(
            BackendType::Skia,
            size.to_i32(),
            azure_hl::SurfaceFormat::B8G8R8A8,
        ))
    }

    fn recreate_paint_state<'a>(&self, state: &CanvasPaintState<'a>) -> CanvasPaintState<'a> {
        CanvasPaintState::new(AntialiasMode::from_azure(
            state.draw_options.as_azure().antialias,
        ))
    }
}

impl<'a> CanvasPaintState<'a> {
    pub fn new(antialias: AntialiasMode) -> CanvasPaintState<'a> {
        CanvasPaintState {
            draw_options: DrawOptions::Azure(azure_hl::DrawOptions::new(
                1.0,
                azure_hl::CompositionOp::Over,
                antialias.into_azure(),
            )),
            fill_style: Pattern::Azure(
                azure_hl::Pattern::Color(ColorPattern::new(azure_hl::Color::black())),
                PhantomData::default(),
            ),
            stroke_style: Pattern::Azure(
                azure_hl::Pattern::Color(ColorPattern::new(azure_hl::Color::black())),
                PhantomData::default(),
            ),
            stroke_opts: StrokeOptions::Azure(azure_hl::StrokeOptions::new(
                1.0,
                JoinStyle::MiterOrBevel,
                CapStyle::Butt,
                10.0,
                &[],
            )),
            transform: Transform2D::identity(),
            shadow_offset_x: 0.0,
            shadow_offset_y: 0.0,
            shadow_blur: 0.0,
            shadow_color: Color::Azure(azure_hl::Color::transparent()),
        }
    }
}

impl GenericPathBuilder for azure_hl::PathBuilder {
    fn arc(
        &mut self,
        origin: Point2D<f32>,
        radius: f32,
        start_angle: f32,
        end_angle: f32,
        anticlockwise: bool,
    ) {
        azure_hl::PathBuilder::arc(
            self,
            origin as Point2D<AzFloat>,
            radius as AzFloat,
            start_angle as AzFloat,
            end_angle as AzFloat,
            anticlockwise,
        );
    }
    fn bezier_curve_to(
        &mut self,
        control_point1: &Point2D<f32>,
        control_point2: &Point2D<f32>,
        control_point3: &Point2D<f32>,
    ) {
        azure_hl::PathBuilder::bezier_curve_to(
            self,
            control_point1 as &Point2D<AzFloat>,
            control_point2 as &Point2D<AzFloat>,
            control_point3 as &Point2D<AzFloat>,
        );
    }
    fn close(&mut self) {
        azure_hl::PathBuilder::close(self);
    }
    fn ellipse(
        &mut self,
        origin: Point2D<f32>,
        radius_x: f32,
        radius_y: f32,
        rotation_angle: f32,
        start_angle: f32,
        end_angle: f32,
        anticlockwise: bool,
    ) {
        azure_hl::PathBuilder::ellipse(
            self,
            origin as Point2D<AzFloat>,
            radius_x as AzFloat,
            radius_y as AzFloat,
            rotation_angle as AzFloat,
            start_angle as AzFloat,
            end_angle as AzFloat,
            anticlockwise,
        );
    }
    fn get_current_point(&mut self) -> Option<Point2D<f32>> {
        let AzPoint { x, y } = azure_hl::PathBuilder::get_current_point(self);
        Some(Point2D::new(x as f32, y as f32))
    }
    fn line_to(&mut self, point: Point2D<f32>) {
        azure_hl::PathBuilder::line_to(self, point as Point2D<AzFloat>);
    }
    fn move_to(&mut self, point: Point2D<f32>) {
        azure_hl::PathBuilder::move_to(self, point as Point2D<AzFloat>);
    }
    fn quadratic_curve_to(&mut self, control_point: &Point2D<f32>, end_point: &Point2D<f32>) {
        azure_hl::PathBuilder::quadratic_curve_to(
            self,
            control_point as &Point2D<AzFloat>,
            end_point as &Point2D<AzFloat>,
        );
    }
    fn finish(&mut self) -> Path {
        Path::Azure(azure_hl::PathBuilder::finish(self))
    }
}

impl GenericDrawTarget for azure_hl::DrawTarget {
    fn clear_rect(&mut self, rect: &Rect<f32>) {
        azure_hl::DrawTarget::clear_rect(self, rect as &Rect<AzFloat>);
    }

    fn copy_surface(
        &mut self,
        surface: SourceSurface,
        source: Rect<i32>,
        destination: Point2D<i32>,
    ) {
        azure_hl::DrawTarget::copy_surface(self, surface.into_azure(), source, destination);
    }

    fn create_gradient_stops(
        &self,
        gradient_stops: Vec<GradientStop>,
        extend_mode: ExtendMode,
    ) -> GradientStops {
        let gradient_stops: Vec<AzGradientStop> =
            gradient_stops.into_iter().map(|x| x.into_azure()).collect();
        GradientStops::Azure(self.create_gradient_stops(&gradient_stops, extend_mode.into_azure()))
    }

    fn create_path_builder(&self) -> Box<dyn GenericPathBuilder> {
        Box::new(self.create_path_builder())
    }

    fn create_similar_draw_target(
        &self,
        size: &Size2D<i32>,
        format: SurfaceFormat,
    ) -> Box<dyn GenericDrawTarget> {
        Box::new(self.create_similar_draw_target(size, format.into_azure()))
    }
    fn create_source_surface_from_data(
        &self,
        data: &[u8],
        size: Size2D<i32>,
        stride: i32,
    ) -> Option<SourceSurface> {
        self.create_source_surface_from_data(data, size, stride, azure_hl::SurfaceFormat::B8G8R8A8)
            .map(|s| SourceSurface::Azure(s))
    }
    fn draw_surface(
        &mut self,
        surface: SourceSurface,
        dest: Rect<f64>,
        source: Rect<f64>,
        filter: Filter,
        draw_options: &DrawOptions,
    ) {
        let surf_options = azure_hl::DrawSurfaceOptions::new(filter.as_azure(), true);
        let draw_options = azure_hl::DrawOptions::new(
            draw_options.as_azure().alpha,
            draw_options.as_azure().composition,
            azure_hl::AntialiasMode::None,
        );
        azure_hl::DrawTarget::draw_surface(
            self,
            surface.into_azure(),
            dest.to_azure_style(),
            source.to_azure_style(),
            surf_options,
            draw_options,
        );
    }
    fn draw_surface_with_shadow(
        &self,
        surface: SourceSurface,
        dest: &Point2D<f32>,
        color: &Color,
        offset: &Vector2D<f32>,
        sigma: f32,
        operator: CompositionOp,
    ) {
        self.draw_surface_with_shadow(
            surface.into_azure(),
            dest as &Point2D<AzFloat>,
            color.as_azure(),
            offset as &Vector2D<AzFloat>,
            sigma as AzFloat,
            operator.into_azure(),
        );
    }
    fn fill(&mut self, path: &Path, pattern: Pattern, draw_options: &DrawOptions) {
        azure_hl::DrawTarget::fill(
            self,
            path.as_azure(),
            pattern.as_azure().to_pattern_ref(),
            draw_options.as_azure(),
        );
    }
    fn fill_rect(
        &mut self,
        rect: &Rect<f32>,
        pattern: Pattern,
        draw_options: Option<&DrawOptions>,
    ) {
        azure_hl::DrawTarget::fill_rect(
            self,
            rect as &Rect<AzFloat>,
            pattern.as_azure().to_pattern_ref(),
            draw_options.map(|x| x.as_azure()),
        );
    }
    fn get_format(&self) -> SurfaceFormat {
        SurfaceFormat::Azure(self.get_format())
    }
    fn get_size(&self) -> Size2D<i32> {
        let size = self.get_size();
        Size2D::new(size.width, size.height)
    }
    fn get_transform(&self) -> Transform2D<f32> {
        self.get_transform() as Transform2D<f32>
    }
    fn pop_clip(&mut self) {
        azure_hl::DrawTarget::pop_clip(self);
    }
    fn push_clip(&mut self, path: &Path) {
        azure_hl::DrawTarget::push_clip(self, path.as_azure());
    }
    fn set_transform(&mut self, matrix: &Transform2D<f32>) {
        azure_hl::DrawTarget::set_transform(self, matrix as &Transform2D<AzFloat>);
    }
    fn snapshot(&self) -> SourceSurface {
        SourceSurface::Azure(self.snapshot())
    }
    fn stroke(
        &mut self,
        path: &Path,
        pattern: Pattern,
        stroke_options: &StrokeOptions,
        draw_options: &DrawOptions,
    ) {
        azure_hl::DrawTarget::stroke(
            self,
            path.as_azure(),
            pattern.as_azure().to_pattern_ref(),
            stroke_options.as_azure(),
            draw_options.as_azure(),
        );
    }
    fn stroke_line(
        &mut self,
        start: Point2D<f32>,
        end: Point2D<f32>,
        pattern: Pattern,
        stroke_options: &StrokeOptions,
        draw_options: &DrawOptions,
    ) {
        let stroke_options = stroke_options.as_azure();
        let cap = match stroke_options.line_join {
            JoinStyle::Round => CapStyle::Round,
            _ => CapStyle::Butt,
        };

        let stroke_opts = azure_hl::StrokeOptions::new(
            stroke_options.line_width,
            stroke_options.line_join,
            cap,
            stroke_options.miter_limit,
            stroke_options.mDashPattern,
        );

        azure_hl::DrawTarget::stroke_line(
            self,
            start,
            end,
            pattern.as_azure().to_pattern_ref(),
            &stroke_opts,
            draw_options.as_azure(),
        );
    }
    fn stroke_rect(
        &mut self,
        rect: &Rect<f32>,
        pattern: Pattern,
        stroke_options: &StrokeOptions,
        draw_options: &DrawOptions,
    ) {
        azure_hl::DrawTarget::stroke_rect(
            self,
            rect as &Rect<AzFloat>,
            pattern.as_azure().to_pattern_ref(),
            stroke_options.as_azure(),
            draw_options.as_azure(),
        );
    }

    #[allow(unsafe_code)]
    fn snapshot_data(&self, f: &dyn Fn(&[u8]) -> Vec<u8>) -> Vec<u8> {
        unsafe { f(self.snapshot().get_data_surface().data()) }
    }

    #[allow(unsafe_code)]
    fn snapshot_data_owned(&self) -> Vec<u8> {
        unsafe { self.snapshot().get_data_surface().data().into() }
    }
}

impl AntialiasMode {
    fn into_azure(self) -> azure_hl::AntialiasMode {
        match self {
            AntialiasMode::Default => azure_hl::AntialiasMode::Default,
            AntialiasMode::None => azure_hl::AntialiasMode::None,
        }
    }

    fn from_azure(val: azure_hl::AntialiasMode) -> AntialiasMode {
        match val {
            azure_hl::AntialiasMode::Default => AntialiasMode::Default,
            azure_hl::AntialiasMode::None => AntialiasMode::None,
            v => unimplemented!("{:?} is unsupported", v),
        }
    }
}

impl ExtendMode {
    fn into_azure(self) -> azure_hl::ExtendMode {
        match self {
            ExtendMode::Azure(m) => m,
        }
    }
}

impl GradientStop {
    fn into_azure(self) -> AzGradientStop {
        match self {
            GradientStop::Azure(s) => s,
        }
    }
}

impl GradientStops {
    fn into_azure(self) -> azure_hl::GradientStops {
        match self {
            GradientStops::Azure(s) => s,
        }
    }
}

impl Color {
    fn as_azure(&self) -> &azure_hl::Color {
        match self {
            Color::Azure(s) => s,
        }
    }
}

impl CompositionOp {
    fn into_azure(self) -> azure_hl::CompositionOp {
        match self {
            CompositionOp::Azure(s) => s,
        }
    }
}

impl SurfaceFormat {
    fn into_azure(self) -> azure_hl::SurfaceFormat {
        match self {
            SurfaceFormat::Azure(s) => s,
        }
    }
}

impl SourceSurface {
    fn into_azure(self) -> azure_hl::SourceSurface {
        match self {
            SourceSurface::Azure(s) => s,
        }
    }
}

impl Path {
    fn as_azure(&self) -> &azure_hl::Path {
        match self {
            Path::Azure(p) => p,
        }
    }
}

impl Pattern<'_> {
    fn as_azure(&self) -> &azure_hl::Pattern {
        match self {
            Pattern::Azure(p, _) => p,
        }
    }
}

impl DrawOptions {
    fn as_azure(&self) -> &azure_hl::DrawOptions {
        match self {
            DrawOptions::Azure(options) => options,
        }
    }
    fn as_azure_mut(&mut self) -> &mut azure_hl::DrawOptions {
        match self {
            DrawOptions::Azure(options) => options,
        }
    }
    pub fn set_alpha(&mut self, val: f32) {
        match self {
            DrawOptions::Azure(options) => options.alpha = val as AzFloat,
        }
    }
}

impl<'a> StrokeOptions<'a> {
    pub fn as_azure(&self) -> &azure_hl::StrokeOptions<'a> {
        match self {
            StrokeOptions::Azure(options) => options,
        }
    }
    pub fn set_line_width(&mut self, val: f32) {
        match self {
            StrokeOptions::Azure(options) => options.line_width = val as AzFloat,
        }
    }
    pub fn set_miter_limit(&mut self, val: f32) {
        match self {
            StrokeOptions::Azure(options) => options.miter_limit = val as AzFloat,
        }
    }
    pub fn set_line_join(&mut self, val: LineJoinStyle) {
        match self {
            StrokeOptions::Azure(options) => options.line_join = val.to_azure_style(),
        }
    }
    pub fn set_line_cap(&mut self, val: LineCapStyle) {
        match self {
            StrokeOptions::Azure(options) => options.line_cap = val.to_azure_style(),
        }
    }
}

pub trait ToAzureStyle {
    type Target;
    fn to_azure_style(self) -> Self::Target;
}

impl ToAzureStyle for Rect<f64> {
    type Target = Rect<f32>;

    fn to_azure_style(self) -> Rect<f32> {
        Rect::new(
            Point2D::new(self.origin.x as f32, self.origin.y as f32),
            Size2D::new(self.size.width as f32, self.size.height as f32),
        )
    }
}

impl ToAzureStyle for LineCapStyle {
    type Target = CapStyle;

    fn to_azure_style(self) -> CapStyle {
        match self {
            LineCapStyle::Butt => CapStyle::Butt,
            LineCapStyle::Round => CapStyle::Round,
            LineCapStyle::Square => CapStyle::Square,
        }
    }
}

impl ToAzureStyle for LineJoinStyle {
    type Target = JoinStyle;

    fn to_azure_style(self) -> JoinStyle {
        match self {
            LineJoinStyle::Round => JoinStyle::Round,
            LineJoinStyle::Bevel => JoinStyle::Bevel,
            LineJoinStyle::Miter => JoinStyle::Miter,
        }
    }
}

impl ToAzureStyle for CompositionStyle {
    type Target = azure_hl::CompositionOp;

    fn to_azure_style(self) -> azure_hl::CompositionOp {
        match self {
            CompositionStyle::SrcIn => azure_hl::CompositionOp::In,
            CompositionStyle::SrcOut => azure_hl::CompositionOp::Out,
            CompositionStyle::SrcOver => azure_hl::CompositionOp::Over,
            CompositionStyle::SrcAtop => azure_hl::CompositionOp::Atop,
            CompositionStyle::DestIn => azure_hl::CompositionOp::DestIn,
            CompositionStyle::DestOut => azure_hl::CompositionOp::DestOut,
            CompositionStyle::DestOver => azure_hl::CompositionOp::DestOver,
            CompositionStyle::DestAtop => azure_hl::CompositionOp::DestAtop,
            CompositionStyle::Copy => azure_hl::CompositionOp::Source,
            CompositionStyle::Lighter => azure_hl::CompositionOp::Add,
            CompositionStyle::Xor => azure_hl::CompositionOp::Xor,
        }
    }
}

impl ToAzureStyle for BlendingStyle {
    type Target = azure_hl::CompositionOp;

    fn to_azure_style(self) -> azure_hl::CompositionOp {
        match self {
            BlendingStyle::Multiply => azure_hl::CompositionOp::Multiply,
            BlendingStyle::Screen => azure_hl::CompositionOp::Screen,
            BlendingStyle::Overlay => azure_hl::CompositionOp::Overlay,
            BlendingStyle::Darken => azure_hl::CompositionOp::Darken,
            BlendingStyle::Lighten => azure_hl::CompositionOp::Lighten,
            BlendingStyle::ColorDodge => azure_hl::CompositionOp::ColorDodge,
            BlendingStyle::ColorBurn => azure_hl::CompositionOp::ColorBurn,
            BlendingStyle::HardLight => azure_hl::CompositionOp::HardLight,
            BlendingStyle::SoftLight => azure_hl::CompositionOp::SoftLight,
            BlendingStyle::Difference => azure_hl::CompositionOp::Difference,
            BlendingStyle::Exclusion => azure_hl::CompositionOp::Exclusion,
            BlendingStyle::Hue => azure_hl::CompositionOp::Hue,
            BlendingStyle::Saturation => azure_hl::CompositionOp::Saturation,
            BlendingStyle::Color => azure_hl::CompositionOp::Color,
            BlendingStyle::Luminosity => azure_hl::CompositionOp::Luminosity,
        }
    }
}

impl ToAzureStyle for CompositionOrBlending {
    type Target = azure_hl::CompositionOp;

    fn to_azure_style(self) -> azure_hl::CompositionOp {
        match self {
            CompositionOrBlending::Composition(op) => op.to_azure_style(),
            CompositionOrBlending::Blending(op) => op.to_azure_style(),
        }
    }
}

pub trait ToAzurePattern {
    fn to_azure_pattern(&self, drawtarget: &dyn GenericDrawTarget) -> Option<azure_hl::Pattern>;
}

impl ToAzurePattern for FillOrStrokeStyle {
    fn to_azure_pattern(&self, drawtarget: &dyn GenericDrawTarget) -> Option<azure_hl::Pattern> {
        Some(match *self {
            FillOrStrokeStyle::Color(ref color) => {
                azure_hl::Pattern::Color(ColorPattern::new(color.to_azure_style()))
            },
            FillOrStrokeStyle::LinearGradient(ref linear_gradient_style) => {
                let gradient_stops: Vec<GradientStop> = linear_gradient_style
                    .stops
                    .iter()
                    .map(|s| {
                        GradientStop::Azure(azure_hl::GradientStop {
                            offset: s.offset as f32,
                            color: s.color.to_azure_style(),
                        })
                    })
                    .collect();

                azure_hl::Pattern::LinearGradient(LinearGradientPattern::new(
                    &Point2D::new(
                        linear_gradient_style.x0 as f32,
                        linear_gradient_style.y0 as f32,
                    ),
                    &Point2D::new(
                        linear_gradient_style.x1 as f32,
                        linear_gradient_style.y1 as f32,
                    ),
                    drawtarget
                        .create_gradient_stops(
                            gradient_stops,
                            ExtendMode::Azure(azure_hl::ExtendMode::Clamp),
                        )
                        .into_azure(),
                    &Transform2D::identity(),
                ))
            },
            FillOrStrokeStyle::RadialGradient(ref radial_gradient_style) => {
                let gradient_stops: Vec<GradientStop> = radial_gradient_style
                    .stops
                    .iter()
                    .map(|s| {
                        GradientStop::Azure(azure_hl::GradientStop {
                            offset: s.offset as f32,
                            color: s.color.to_azure_style(),
                        })
                    })
                    .collect();

                azure_hl::Pattern::RadialGradient(RadialGradientPattern::new(
                    &Point2D::new(
                        radial_gradient_style.x0 as f32,
                        radial_gradient_style.y0 as f32,
                    ),
                    &Point2D::new(
                        radial_gradient_style.x1 as f32,
                        radial_gradient_style.y1 as f32,
                    ),
                    radial_gradient_style.r0 as f32,
                    radial_gradient_style.r1 as f32,
                    drawtarget
                        .create_gradient_stops(
                            gradient_stops,
                            ExtendMode::Azure(azure_hl::ExtendMode::Clamp),
                        )
                        .into_azure(),
                    &Transform2D::identity(),
                ))
            },
            FillOrStrokeStyle::Surface(ref surface_style) => {
                let source_surface = drawtarget
                    .create_source_surface_from_data(
                        &surface_style.surface_data,
                        // FIXME(nox): Why are those i32 values?
                        surface_style.surface_size.to_i32(),
                        surface_style.surface_size.width as i32 * 4,
                    )?
                    .into_azure();
                azure_hl::Pattern::Surface(SurfacePattern::new(
                    source_surface.azure_source_surface,
                    surface_style.repeat_x,
                    surface_style.repeat_y,
                    &Transform2D::identity(),
                ))
            },
        })
    }
}

impl ToAzureStyle for RGBA {
    type Target = azure_hl::Color;

    fn to_azure_style(self) -> azure_hl::Color {
        azure_hl::Color::rgba(
            self.red_f32() as f32,
            self.green_f32() as f32,
            self.blue_f32() as f32,
            self.alpha_f32() as f32,
        )
    }
}

impl Pattern<'_> {
    pub fn is_zero_size_gradient(&self) -> bool {
        match *self {
            Pattern::Azure(azure_hl::Pattern::LinearGradient(ref gradient), _) => {
                gradient.is_zero_size()
            },
            _ => false,
        }
    }
}

impl Filter {
    fn as_azure(&self) -> azure_hl::Filter {
        match *self {
            Filter::Linear => azure_hl::Filter::Linear,
            Filter::Point => azure_hl::Filter::Point,
        }
    }
}

impl Path {
    pub fn transformed_copy_to_builder(
        &self,
        transform: &Transform2D<f32>,
    ) -> Box<dyn GenericPathBuilder> {
        Box::new(self.as_azure().transformed_copy_to_builder(transform))
    }

    pub fn contains_point(&self, x: f64, y: f64, path_transform: &Transform2D<f32>) -> bool {
        self.as_azure().contains_point(x, y, path_transform)
    }

    pub fn copy_to_builder(&self) -> Box<dyn GenericPathBuilder> {
        Box::new(self.as_azure().copy_to_builder())
    }
}

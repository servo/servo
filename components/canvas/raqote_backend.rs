/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::canvas_data::{
    Backend, CanvasPaintState, Color, CompositionOp, DrawOptions, ExtendMode, Filter,
    GenericDrawTarget, GenericPathBuilder, GradientStop, GradientStops, Path, Pattern,
    SourceSurface, StrokeOptions, SurfaceFormat,
};
use crate::canvas_paint_thread::AntialiasMode;
use canvas_traits::canvas::*;
use cssparser::RGBA;
use euclid::default::{Point2D, Rect, Size2D, Transform2D, Vector2D};
use raqote::PathOp;
use std::f32::consts::PI;
use std::marker::PhantomData;

pub struct RaqoteBackend;

impl Backend for RaqoteBackend {
    fn get_composition_op(&self, opts: &DrawOptions) -> CompositionOp {
        CompositionOp::Raqote(opts.as_raqote().blend_mode)
    }

    fn need_to_draw_shadow(&self, color: &Color) -> bool {
        color.as_raqote().a != 0
    }

    fn size_from_pattern(&self, rect: &Rect<f32>, pattern: &Pattern) -> Option<Size2D<f32>> {
        match pattern {
            Pattern::Raqote(raqote::Source::Image(image, extend, ..)) => match extend {
                raqote::ExtendMode::Repeat => Some(rect.size),
                _ => Some(Size2D::new(image.width as f32, image.height as f32)),
            },
            _ => None,
        }
    }

    fn set_shadow_color<'a>(&mut self, color: RGBA, state: &mut CanvasPaintState<'a>) {
        state.shadow_color = Color::Raqote(color.to_raqote_style());
    }

    fn set_fill_style<'a>(
        &mut self,
        style: FillOrStrokeStyle,
        state: &mut CanvasPaintState<'a>,
        _drawtarget: &dyn GenericDrawTarget,
    ) {
        if let Some(source) = style.to_raqote_source() {
            state.fill_style = Pattern::Raqote(source);
        }
    }

    fn set_stroke_style<'a>(
        &mut self,
        style: FillOrStrokeStyle,
        state: &mut CanvasPaintState<'a>,
        _drawtarget: &dyn GenericDrawTarget,
    ) {
        if let Some(pattern) = style.to_raqote_source() {
            state.stroke_style = Pattern::Raqote(pattern)
        }
    }

    fn set_global_composition<'a>(
        &mut self,
        op: CompositionOrBlending,
        state: &mut CanvasPaintState<'a>,
    ) {
        state.draw_options.as_raqote_mut().blend_mode = op.to_raqote_style();
    }

    fn create_drawtarget(&self, size: Size2D<u64>) -> Box<dyn GenericDrawTarget> {
        Box::new(raqote::DrawTarget::new(
            size.width as i32,
            size.height as i32,
        ))
    }

    fn recreate_paint_state<'a>(&self, _state: &CanvasPaintState<'a>) -> CanvasPaintState<'a> {
        CanvasPaintState::new(AntialiasMode::Default)
    }
}

impl<'a> CanvasPaintState<'a> {
    pub fn new(_antialias: AntialiasMode) -> CanvasPaintState<'a> {
        let solid_src = raqote::SolidSource {
            r: 0,
            g: 0,
            b: 0,
            a: 255,
        };
        CanvasPaintState {
            draw_options: DrawOptions::Raqote(raqote::DrawOptions::new()),
            fill_style: Pattern::Raqote(raqote::Source::Solid(solid_src)),
            stroke_style: Pattern::Raqote(raqote::Source::Solid(solid_src)),
            stroke_opts: StrokeOptions::Raqote(Default::default(), PhantomData),
            transform: Transform2D::identity(),
            shadow_offset_x: 0.0,
            shadow_offset_y: 0.0,
            shadow_blur: 0.0,
            shadow_color: Color::Raqote(raqote::SolidSource {
                r: 0,
                g: 0,
                b: 0,
                a: 0,
            }),
        }
    }
}

impl Pattern<'_> {
    pub fn is_zero_size_gradient(&self) -> bool {
        match self {
            Pattern::Raqote(p) => {
                use raqote::Source::*;

                match p {
                    LinearGradient(g, ..) |
                    RadialGradient(g, ..) |
                    TwoCircleRadialGradient(g, ..) => g.stops.is_empty(),
                    _ => false,
                }
            },
        }
    }
    pub fn as_raqote(&self) -> &raqote::Source {
        match self {
            Pattern::Raqote(p) => p,
        }
    }
}

impl<'a> StrokeOptions<'a> {
    pub fn set_line_width(&mut self, _val: f32) {
        match self {
            StrokeOptions::Raqote(options, _) => options.width = _val,
        }
    }
    pub fn set_miter_limit(&mut self, _val: f32) {
        match self {
            StrokeOptions::Raqote(options, _) => options.miter_limit = _val,
        }
    }
    pub fn set_line_join(&mut self, val: LineJoinStyle) {
        match self {
            StrokeOptions::Raqote(options, _) => options.join = val.to_raqote_style(),
        }
    }
    pub fn set_line_cap(&mut self, val: LineCapStyle) {
        match self {
            StrokeOptions::Raqote(options, _) => options.cap = val.to_raqote_style(),
        }
    }
    pub fn as_raqote(&self) -> &raqote::StrokeStyle {
        match self {
            StrokeOptions::Raqote(options, _) => options,
        }
    }
}

impl DrawOptions {
    pub fn set_alpha(&mut self, val: f32) {
        match self {
            DrawOptions::Raqote(draw_options) => draw_options.alpha = val,
        }
    }
    pub fn as_raqote(&self) -> &raqote::DrawOptions {
        match self {
            DrawOptions::Raqote(options) => options,
        }
    }
    fn as_raqote_mut(&mut self) -> &mut raqote::DrawOptions {
        match self {
            DrawOptions::Raqote(options) => options,
        }
    }
}

impl Path {
    pub fn transformed_copy_to_builder(
        &self,
        transform: &Transform2D<f32>,
    ) -> Box<dyn GenericPathBuilder> {
        Box::new(PathBuilder(Some(raqote::PathBuilder::from(
            self.as_raqote().clone().transform(transform),
        ))))
    }

    pub fn contains_point(&self, x: f64, y: f64, path_transform: &Transform2D<f32>) -> bool {
        self.as_raqote()
            .clone()
            .transform(path_transform)
            .contains_point(0., x as f32, y as f32)
    }

    pub fn copy_to_builder(&self) -> Box<dyn GenericPathBuilder> {
        Box::new(PathBuilder(Some(raqote::PathBuilder::from(
            self.as_raqote().clone(),
        ))))
    }

    pub fn as_raqote(&self) -> &raqote::Path {
        match self {
            Path::Raqote(p) => p,
        }
    }
}

impl GenericDrawTarget for raqote::DrawTarget {
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
        raqote::DrawTarget::fill(
            self,
            &pb.finish(),
            &raqote::Source::Solid(raqote::SolidSource {
                r: 0,
                g: 0,
                b: 0,
                a: 0,
            }),
            &options,
        );
    }
    #[allow(unsafe_code)]
    fn copy_surface(
        &mut self,
        surface: SourceSurface,
        source: Rect<i32>,
        destination: Point2D<i32>,
    ) {
        let mut dt = raqote::DrawTarget::new(source.size.width, source.size.height);
        let data = surface.as_raqote();
        let s = unsafe { std::slice::from_raw_parts(data.as_ptr() as *const u32, data.len() / 4) };
        dt.get_data_mut().copy_from_slice(s);
        raqote::DrawTarget::copy_surface(self, &dt, source.to_box2d(), destination);
    }
    fn create_gradient_stops(
        &self,
        gradient_stops: Vec<GradientStop>,
        _extend_mode: ExtendMode,
    ) -> GradientStops {
        let stops = gradient_stops
            .into_iter()
            .map(|item| item.as_raqote().clone())
            .collect();
        GradientStops::Raqote(stops)
    }
    fn create_path_builder(&self) -> Box<dyn GenericPathBuilder> {
        Box::new(PathBuilder::new())
    }
    fn create_similar_draw_target(
        &self,
        size: &Size2D<i32>,
        _format: SurfaceFormat,
    ) -> Box<dyn GenericDrawTarget> {
        Box::new(raqote::DrawTarget::new(size.width, size.height))
    }
    fn create_source_surface_from_data(
        &self,
        data: &[u8],
        _size: Size2D<i32>,
        _stride: i32,
    ) -> Option<SourceSurface> {
        Some(SourceSurface::Raqote(data.to_vec()))
    }
    #[allow(unsafe_code)]
    fn draw_surface(
        &mut self,
        surface: SourceSurface,
        dest: Rect<f64>,
        source: Rect<f64>,
        _filter: Filter,
        draw_options: &DrawOptions,
    ) {
        let v = surface.as_raqote();
        let image = raqote::Image {
            width: source.size.width as i32,
            height: source.size.height as i32,
            data: unsafe {
                std::slice::from_raw_parts(
                    v.as_ptr() as *const u32,
                    v.len() * std::mem::size_of::<u8>(),
                )
            },
        };
        raqote::DrawTarget::draw_image_with_size_at(
            self,
            dest.size.width as f32,
            dest.size.height as f32,
            dest.origin.x as f32,
            dest.origin.y as f32,
            &image,
            draw_options.as_raqote(),
        );
    }
    fn draw_surface_with_shadow(
        &self,
        _surface: SourceSurface,
        _dest: &Point2D<f32>,
        _color: &Color,
        _offset: &Vector2D<f32>,
        _sigma: f32,
        _operator: CompositionOp,
    ) {
        warn!("no support for drawing shadows");
    }
    fn fill(&mut self, path: &Path, pattern: Pattern, draw_options: &DrawOptions) {
        self.fill(
            path.as_raqote(),
            pattern.as_raqote(),
            draw_options.as_raqote(),
        );
    }
    fn fill_rect(
        &mut self,
        rect: &Rect<f32>,
        pattern: Pattern,
        draw_options: Option<&DrawOptions>,
    ) {
        let mut pb = raqote::PathBuilder::new();
        pb.rect(
            rect.origin.x,
            rect.origin.y,
            rect.size.width,
            rect.size.height,
        );
        let draw_options = if let Some(options) = draw_options {
            *options.as_raqote()
        } else {
            raqote::DrawOptions::new()
        };

        raqote::DrawTarget::fill(self, &pb.finish(), pattern.as_raqote(), &draw_options);
    }
    fn get_format(&self) -> SurfaceFormat {
        SurfaceFormat::Raqote(())
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
    fn push_clip(&mut self, path: &Path) {
        self.push_clip(path.as_raqote());
    }
    fn set_transform(&mut self, matrix: &Transform2D<f32>) {
        self.set_transform(matrix);
    }
    fn snapshot(&self) -> SourceSurface {
        SourceSurface::Raqote(self.snapshot_data_owned())
    }
    fn stroke(
        &mut self,
        path: &Path,
        pattern: Pattern,
        stroke_options: &StrokeOptions,
        draw_options: &DrawOptions,
    ) {
        self.stroke(
            path.as_raqote(),
            pattern.as_raqote(),
            stroke_options.as_raqote(),
            draw_options.as_raqote(),
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
        let mut pb = raqote::PathBuilder::new();
        pb.move_to(start.x, start.y);
        pb.line_to(end.x, end.y);
        let mut stroke_options = stroke_options.as_raqote().clone();
        let cap = match stroke_options.join {
            raqote::LineJoin::Round => raqote::LineCap::Round,
            _ => raqote::LineCap::Butt,
        };
        stroke_options.cap = cap;

        self.stroke(
            &pb.finish(),
            pattern.as_raqote(),
            &stroke_options,
            draw_options.as_raqote(),
        );
    }
    fn stroke_rect(
        &mut self,
        rect: &Rect<f32>,
        pattern: Pattern,
        stroke_options: &StrokeOptions,
        draw_options: &DrawOptions,
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
            pattern.as_raqote(),
            stroke_options.as_raqote(),
            draw_options.as_raqote(),
        );
    }
    #[allow(unsafe_code)]
    fn snapshot_data(&self, f: &dyn Fn(&[u8]) -> Vec<u8>) -> Vec<u8> {
        let v = self.get_data();
        f(unsafe {
            std::slice::from_raw_parts(
                v.as_ptr() as *const u8,
                v.len() * std::mem::size_of::<u32>(),
            )
        })
    }
    #[allow(unsafe_code)]
    fn snapshot_data_owned(&self) -> Vec<u8> {
        let v = self.get_data();
        unsafe {
            std::slice::from_raw_parts(
                v.as_ptr() as *const u8,
                v.len() * std::mem::size_of::<u32>(),
            )
            .into()
        }
    }
}

struct PathBuilder(Option<raqote::PathBuilder>);

impl PathBuilder {
    fn new() -> PathBuilder {
        PathBuilder(Some(raqote::PathBuilder::new()))
    }
}

impl GenericPathBuilder for PathBuilder {
    fn arc(
        &mut self,
        origin: Point2D<f32>,
        radius: f32,
        start_angle: f32,
        end_angle: f32,
        _anticlockwise: bool,
    ) {
        self.0
            .as_mut()
            .unwrap()
            .arc(origin.x, origin.y, radius, start_angle, end_angle);
    }
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
    fn ellipse(
        &mut self,
        origin: Point2D<f32>,
        radius_x: f32,
        radius_y: f32,
        _rotation_angle: f32,
        start_angle: f32,
        mut end_angle: f32,
        anticlockwise: bool,
    ) {
        let start_point = Point2D::new(
            origin.x + start_angle.cos() * radius_x,
            origin.y + end_angle.sin() * radius_y,
        );
        self.line_to(start_point);

        if !anticlockwise && (end_angle < start_angle) {
            let correction = ((start_angle - end_angle) / (2.0 * PI)).ceil();
            end_angle += correction * 2.0 * PI;
        } else if anticlockwise && (start_angle < end_angle) {
            let correction = ((end_angle - start_angle) / (2.0 * PI)).ceil();
            end_angle += correction * 2.0 * PI;
        }
        // Sweeping more than 2 * pi is a full circle.
        if !anticlockwise && (end_angle - start_angle > 2.0 * PI) {
            end_angle = start_angle + 2.0 * PI;
        } else if anticlockwise && (start_angle - end_angle > 2.0 * PI) {
            end_angle = start_angle - 2.0 * PI;
        }

        // Calculate the total arc we're going to sweep.
        let mut arc_sweep_left = (end_angle - start_angle).abs();
        let sweep_direction = match anticlockwise {
            true => -1.0,
            false => 1.0,
        };
        let mut current_start_angle = start_angle;
        while arc_sweep_left > 0.0 {
            // We guarantee here the current point is the start point of the next
            // curve segment.
            let current_end_angle;
            if arc_sweep_left > PI / 2.0 {
                current_end_angle = current_start_angle + PI / 2.0 * sweep_direction;
            } else {
                current_end_angle = current_start_angle + arc_sweep_left * sweep_direction;
            }
            let current_start_point = Point2D::new(
                origin.x + current_start_angle.cos() * radius_x,
                origin.y + current_start_angle.sin() * radius_y,
            );
            let current_end_point = Point2D::new(
                origin.x + current_end_angle.cos() * radius_x,
                origin.y + current_end_angle.sin() * radius_y,
            );
            // Calculate kappa constant for partial curve. The sign of angle in the
            // tangent will actually ensure this is negative for a counter clockwise
            // sweep, so changing signs later isn't needed.
            let kappa_factor =
                (4.0 / 3.0) * ((current_end_angle - current_start_angle) / 4.0).tan();
            let kappa_x = kappa_factor * radius_x;
            let kappa_y = kappa_factor * radius_y;

            let tangent_start =
                Point2D::new(-(current_start_angle.sin()), current_start_angle.cos());
            let mut cp1 = current_start_point;
            cp1 += Point2D::new(tangent_start.x * kappa_x, tangent_start.y * kappa_y).to_vector();
            let rev_tangent_end = Point2D::new(current_end_angle.sin(), -(current_end_angle.cos()));
            let mut cp2 = current_end_point;
            cp2 +=
                Point2D::new(rev_tangent_end.x * kappa_x, rev_tangent_end.y * kappa_y).to_vector();

            self.bezier_curve_to(&cp1, &cp2, &current_end_point);

            arc_sweep_left -= PI / 2.0;
            current_start_angle = current_end_angle;
        }
    }

    fn get_current_point(&mut self) -> Point2D<f32> {
        let path = self.finish();
        self.0 = Some(path.as_raqote().clone().into());

        for op in path.as_raqote().ops.iter().rev() {
            match op {
                PathOp::MoveTo(point) | PathOp::LineTo(point) => {
                    return Point2D::new(point.x, point.y)
                },
                PathOp::CubicTo(_, _, point) => return Point2D::new(point.x, point.y),
                PathOp::QuadTo(_, point) => return Point2D::new(point.x, point.y),
                PathOp::Close => {},
            };
        }
        panic!("dead end");
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
    fn finish(&mut self) -> Path {
        Path::Raqote(self.0.take().unwrap().finish())
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

pub trait ToRaqoteSource<'a> {
    fn to_raqote_source(self) -> Option<raqote::Source<'a>>;
}

pub trait ToRaqoteGradientStop {
    fn to_raqote(&self) -> raqote::GradientStop;
}

impl ToRaqoteGradientStop for CanvasGradientStop {
    fn to_raqote(&self) -> raqote::GradientStop {
        let color = raqote::Color::new(
            self.color.alpha,
            self.color.red,
            self.color.green,
            self.color.blue,
        );
        let position = self.offset as f32;
        raqote::GradientStop { position, color }
    }
}

impl<'a> ToRaqoteSource<'a> for FillOrStrokeStyle {
    #[allow(unsafe_code)]
    fn to_raqote_source(self) -> Option<raqote::Source<'a>> {
        use canvas_traits::canvas::FillOrStrokeStyle::*;

        match self {
            Color(rgba) => Some(raqote::Source::Solid(raqote::SolidSource {
                r: rgba.red,
                g: rgba.green,
                b: rgba.blue,
                a: rgba.alpha,
            })),
            LinearGradient(style) => {
                let stops = style.stops.into_iter().map(|s| s.to_raqote()).collect();
                let gradient = raqote::Gradient { stops };
                let start = Point2D::new(style.x0 as f32, style.y0 as f32);
                let end = Point2D::new(style.x1 as f32, style.y1 as f32);
                Some(raqote::Source::new_linear_gradient(
                    gradient,
                    start,
                    end,
                    raqote::Spread::Pad,
                ))
            },
            RadialGradient(style) => {
                let stops = style.stops.into_iter().map(|s| s.to_raqote()).collect();
                let gradient = raqote::Gradient { stops };
                let center1 = Point2D::new(style.x0 as f32, style.y0 as f32);
                let center2 = Point2D::new(style.x1 as f32, style.y1 as f32);
                Some(raqote::Source::new_two_circle_radial_gradient(
                    gradient,
                    center1,
                    style.r0 as f32,
                    center2,
                    style.r1 as f32,
                    raqote::Spread::Pad,
                ))
            },
            Surface(ref surface) => {
                let data = &surface.surface_data[..];
                Some(raqote::Source::Image(
                    raqote::Image {
                        data: unsafe {
                            std::slice::from_raw_parts(data.as_ptr() as *const u32, data.len() / 4)
                        },
                        width: surface.surface_size.width as i32,
                        height: surface.surface_size.height as i32,
                    },
                    raqote::ExtendMode::Repeat, // TODO: repeat-x, repeat-y ?
                    raqote::FilterMode::Bilinear,
                    raqote::Transform::identity(),
                ))
            },
        }
    }
}

impl Color {
    fn as_raqote(&self) -> &raqote::SolidSource {
        match self {
            Color::Raqote(s) => s,
        }
    }
}

impl ToRaqoteStyle for RGBA {
    type Target = raqote::SolidSource;

    fn to_raqote_style(self) -> Self::Target {
        raqote::SolidSource {
            r: self.red,
            g: self.green,
            b: self.blue,
            a: self.alpha,
        }
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
        }
    }
}

impl SourceSurface {
    fn as_raqote(&self) -> &Vec<u8> {
        match self {
            SourceSurface::Raqote(s) => s,
        }
    }
}

impl GradientStop {
    fn as_raqote(&self) -> &raqote::GradientStop {
        match self {
            GradientStop::Raqote(s) => s,
        }
    }
}

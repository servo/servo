/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use canvas_traits::canvas::{
    CompositionOrBlending, FillOrStrokeStyle, LineCapStyle, LineJoinStyle,
};
use euclid::Angle;
use euclid::default::{Point2D, Rect, Size2D, Transform2D, Vector2D};
use lyon_geom::Arc;
use style::color::AbsoluteColor;

use crate::canvas_data::{CanvasPaintState, Filter, TextRun};

pub(crate) trait Backend: Clone + Sized {
    type Pattern<'a>: PatternHelpers + Clone;
    type StrokeOptions: StrokeOptionsHelpers + Clone;
    type Color: Clone;
    type DrawOptions: DrawOptionsHelpers + Clone;
    type CompositionOp;
    type DrawTarget: GenericDrawTarget<Self>;
    type PathBuilder: GenericPathBuilder<Self>;
    type SourceSurface;
    type Bytes<'a>: AsRef<[u8]>;
    type Path: PathHelpers<Self> + Clone;
    type GradientStop;
    type GradientStops;

    fn get_composition_op(&self, opts: &Self::DrawOptions) -> Self::CompositionOp;
    fn need_to_draw_shadow(&self, color: &Self::Color) -> bool;
    fn set_shadow_color(&mut self, color: AbsoluteColor, state: &mut CanvasPaintState<'_, Self>);
    fn set_fill_style(
        &mut self,
        style: FillOrStrokeStyle,
        state: &mut CanvasPaintState<'_, Self>,
        drawtarget: &Self::DrawTarget,
    );
    fn set_stroke_style(
        &mut self,
        style: FillOrStrokeStyle,
        state: &mut CanvasPaintState<'_, Self>,
        drawtarget: &Self::DrawTarget,
    );
    fn set_global_composition(
        &mut self,
        op: CompositionOrBlending,
        state: &mut CanvasPaintState<'_, Self>,
    );
    fn create_drawtarget(&self, size: Size2D<u64>) -> Self::DrawTarget;
    fn new_paint_state<'a>(&self) -> CanvasPaintState<'a, Self>;
}

// This defines required methods for a DrawTarget (currently only implemented for raqote).  The
// prototypes are derived from the now-removed Azure backend's methods.
pub(crate) trait GenericDrawTarget<B: Backend> {
    fn clear_rect(&mut self, rect: &Rect<f32>);
    fn copy_surface(
        &mut self,
        surface: B::SourceSurface,
        source: Rect<i32>,
        destination: Point2D<i32>,
    );
    fn create_path_builder(&self) -> B::PathBuilder;
    fn create_similar_draw_target(&self, size: &Size2D<i32>) -> Self;
    fn create_source_surface_from_data(&self, data: &[u8]) -> Option<B::SourceSurface>;
    fn draw_surface(
        &mut self,
        surface: B::SourceSurface,
        dest: Rect<f64>,
        source: Rect<f64>,
        filter: Filter,
        draw_options: &B::DrawOptions,
    );
    fn draw_surface_with_shadow(
        &self,
        surface: B::SourceSurface,
        dest: &Point2D<f32>,
        color: &B::Color,
        offset: &Vector2D<f32>,
        sigma: f32,
        operator: B::CompositionOp,
    );
    fn fill(&mut self, path: &B::Path, pattern: B::Pattern<'_>, draw_options: &B::DrawOptions);
    fn fill_text(
        &mut self,
        text_runs: Vec<TextRun>,
        start: Point2D<f32>,
        pattern: &B::Pattern<'_>,
        draw_options: &B::DrawOptions,
    );
    fn fill_rect(
        &mut self,
        rect: &Rect<f32>,
        pattern: B::Pattern<'_>,
        draw_options: Option<&B::DrawOptions>,
    );
    fn get_size(&self) -> Size2D<i32>;
    fn get_transform(&self) -> Transform2D<f32>;
    fn pop_clip(&mut self);
    fn push_clip(&mut self, path: &B::Path);
    fn set_transform(&mut self, matrix: &Transform2D<f32>);
    fn stroke(
        &mut self,
        path: &B::Path,
        pattern: B::Pattern<'_>,
        stroke_options: &B::StrokeOptions,
        draw_options: &B::DrawOptions,
    );
    fn stroke_line(
        &mut self,
        start: Point2D<f32>,
        end: Point2D<f32>,
        pattern: B::Pattern<'_>,
        stroke_options: &B::StrokeOptions,
        draw_options: &B::DrawOptions,
    );
    fn stroke_rect(
        &mut self,
        rect: &Rect<f32>,
        pattern: B::Pattern<'_>,
        stroke_options: &B::StrokeOptions,
        draw_options: &B::DrawOptions,
    );
    fn surface(&self) -> B::SourceSurface;
    fn bytes(&'_ self) -> B::Bytes<'_>;
}

/// A generic PathBuilder that abstracts the interface for azure's and raqote's PathBuilder.
pub(crate) trait GenericPathBuilder<B: Backend> {
    fn arc(
        &mut self,
        origin: Point2D<f32>,
        radius: f32,
        start_angle: f32,
        end_angle: f32,
        anticlockwise: bool,
    );
    fn bezier_curve_to(
        &mut self,
        control_point1: &Point2D<f32>,
        control_point2: &Point2D<f32>,
        control_point3: &Point2D<f32>,
    );
    fn close(&mut self);
    #[allow(clippy::too_many_arguments)]
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
        let mut start = Angle::radians(start_angle);
        let mut end = Angle::radians(end_angle);

        // Wrap angles mod 2 * PI if necessary
        if !anticlockwise && start > end + Angle::two_pi() ||
            anticlockwise && end > start + Angle::two_pi()
        {
            start = start.positive();
            end = end.positive();
        }

        // Calculate the total arc we're going to sweep.
        let sweep = match anticlockwise {
            true => {
                if end - start == Angle::two_pi() {
                    -Angle::two_pi()
                } else if end > start {
                    -(Angle::two_pi() - (end - start))
                } else {
                    -(start - end)
                }
            },
            false => {
                if start - end == Angle::two_pi() {
                    Angle::two_pi()
                } else if start > end {
                    Angle::two_pi() - (start - end)
                } else {
                    end - start
                }
            },
        };

        let arc: Arc<f32> = Arc {
            center: origin,
            radii: Vector2D::new(radius_x, radius_y),
            start_angle: start,
            sweep_angle: sweep,
            x_rotation: Angle::radians(rotation_angle),
        };

        self.line_to(arc.from());

        arc.for_each_quadratic_bezier(&mut |q| {
            self.quadratic_curve_to(&q.ctrl, &q.to);
        });
    }
    fn get_current_point(&mut self) -> Option<Point2D<f32>>;
    fn line_to(&mut self, point: Point2D<f32>);
    fn move_to(&mut self, point: Point2D<f32>);
    fn quadratic_curve_to(&mut self, control_point: &Point2D<f32>, end_point: &Point2D<f32>);
    fn svg_arc(
        &mut self,
        radius_x: f32,
        radius_y: f32,
        rotation_angle: f32,
        large_arc: bool,
        sweep: bool,
        end_point: Point2D<f32>,
    );
    fn finish(&mut self) -> B::Path;
}

pub(crate) trait PatternHelpers {
    fn is_zero_size_gradient(&self) -> bool;
    fn draw_rect(&self, rect: &Rect<f32>) -> Rect<f32>;
}

pub(crate) trait StrokeOptionsHelpers {
    fn set_line_width(&mut self, _val: f32);
    fn set_miter_limit(&mut self, _val: f32);
    fn set_line_join(&mut self, val: LineJoinStyle);
    fn set_line_cap(&mut self, val: LineCapStyle);
    fn set_line_dash(&mut self, items: Vec<f32>);
    fn set_line_dash_offset(&mut self, offset: f32);
}

pub(crate) trait DrawOptionsHelpers {
    fn set_alpha(&mut self, val: f32);
}

pub(crate) trait PathHelpers<B: Backend> {
    fn transformed_copy_to_builder(&self, transform: &Transform2D<f32>) -> B::PathBuilder;

    fn contains_point(&self, x: f64, y: f64, path_transform: &Transform2D<f32>) -> bool;

    fn copy_to_builder(&self) -> B::PathBuilder;
}

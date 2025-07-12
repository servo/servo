/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use canvas_traits::canvas::{
    CompositionOrBlending, FillOrStrokeStyle, LineCapStyle, LineJoinStyle, Path,
};
use compositing_traits::SerializableImageData;
use euclid::default::{Point2D, Rect, Size2D, Transform2D, Vector2D};
use pixels::Snapshot;
use style::color::AbsoluteColor;
use webrender_api::ImageDescriptor;

use crate::canvas_data::{CanvasPaintState, Filter, TextRun};

pub(crate) trait Backend: Clone + Sized {
    type Pattern<'a>: PatternHelpers + Clone;
    type StrokeOptions: StrokeOptionsHelpers + Clone;
    type Color: Clone;
    type DrawOptions: DrawOptionsHelpers + Clone;
    type CompositionOp;
    type DrawTarget: GenericDrawTarget<Self>;
    type SourceSurface;
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
    fn create_similar_draw_target(&self, size: &Size2D<i32>) -> Self;
    fn create_source_surface_from_data(&self, data: Snapshot) -> Option<B::SourceSurface>;
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
    fn fill(&mut self, path: &Path, pattern: &B::Pattern<'_>, draw_options: &B::DrawOptions);
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
        pattern: &B::Pattern<'_>,
        draw_options: &B::DrawOptions,
    );
    fn get_size(&self) -> Size2D<i32>;
    fn get_transform(&self) -> Transform2D<f32>;
    fn pop_clip(&mut self);
    fn push_clip(&mut self, path: &Path);
    fn push_clip_rect(&mut self, rect: &Rect<i32>);
    fn set_transform(&mut self, matrix: &Transform2D<f32>);
    fn stroke(
        &mut self,
        path: &Path,
        pattern: &B::Pattern<'_>,
        stroke_options: &B::StrokeOptions,
        draw_options: &B::DrawOptions,
    );
    fn stroke_rect(
        &mut self,
        rect: &Rect<f32>,
        pattern: &B::Pattern<'_>,
        stroke_options: &B::StrokeOptions,
        draw_options: &B::DrawOptions,
    );
    fn surface(&self) -> B::SourceSurface;
    fn image_descriptor_and_serializable_data(&self) -> (ImageDescriptor, SerializableImageData);
    fn snapshot(&self) -> Snapshot;
}

pub(crate) trait PatternHelpers {
    fn is_zero_size_gradient(&self) -> bool;
    fn x_bound(&self) -> Option<u32>;
    fn y_bound(&self) -> Option<u32>;
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
    fn is_clear(&self) -> bool;
}

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
use std::marker::PhantomData;

pub struct RaqoteBackend;

impl Backend for RaqoteBackend {
    fn get_composition_op(&self, _opts: &DrawOptions) -> CompositionOp {
        unimplemented!()
    }

    fn need_to_draw_shadow(&self, _color: &Color) -> bool {
        unimplemented!()
    }

    fn size_from_pattern(&self, _rect: &Rect<f32>, _pattern: &Pattern) -> Option<Size2D<f32>> {
        unimplemented!()
    }

    fn set_shadow_color<'a>(&mut self, _color: RGBA, _state: &mut CanvasPaintState<'a>) {
        unimplemented!()
    }

    fn set_fill_style<'a>(
        &mut self,
        _style: FillOrStrokeStyle,
        _state: &mut CanvasPaintState<'a>,
        _drawtarget: &dyn GenericDrawTarget,
    ) {
        unimplemented!()
    }

    fn set_stroke_style<'a>(
        &mut self,
        _style: FillOrStrokeStyle,
        _state: &mut CanvasPaintState<'a>,
        _drawtarget: &dyn GenericDrawTarget,
    ) {
        unimplemented!()
    }

    fn set_global_composition<'a>(
        &mut self,
        _op: CompositionOrBlending,
        _state: &mut CanvasPaintState<'a>,
    ) {
        unimplemented!()
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
        CanvasPaintState {
            draw_options: DrawOptions::Raqote(()),
            fill_style: Pattern::Raqote(()),
            stroke_style: Pattern::Raqote(()),
            stroke_opts: StrokeOptions::Raqote(PhantomData),
            transform: Transform2D::identity(),
            shadow_offset_x: 0.0,
            shadow_offset_y: 0.0,
            shadow_blur: 0.0,
            shadow_color: Color::Raqote(()),
        }
    }
}

impl Pattern {
    pub fn is_zero_size_gradient(&self) -> bool {
        match *self {
            Pattern::Raqote(()) => unimplemented!(),
        }
    }
}

impl<'a> StrokeOptions<'a> {
    pub fn set_line_width(&mut self, _val: f32) {
        unimplemented!()
    }
    pub fn set_miter_limit(&mut self, _val: f32) {
        unimplemented!()
    }
    pub fn set_line_join(&mut self, _val: LineJoinStyle) {
        unimplemented!()
    }
    pub fn set_line_cap(&mut self, _val: LineCapStyle) {
        unimplemented!()
    }
}

impl DrawOptions {
    pub fn set_alpha(&mut self, _val: f32) {
        match self {
            DrawOptions::Raqote(()) => unimplemented!(),
        }
    }
}

impl Path {
    pub fn transformed_copy_to_builder(
        &self,
        _transform: &Transform2D<f32>,
    ) -> Box<dyn GenericPathBuilder> {
        unimplemented!()
    }

    pub fn contains_point(&self, _x: f64, _y: f64, _path_transform: &Transform2D<f32>) -> bool {
        unimplemented!()
    }

    pub fn copy_to_builder(&self) -> Box<dyn GenericPathBuilder> {
        unimplemented!()
    }
}

impl GenericDrawTarget for raqote::DrawTarget {
    fn clear_rect(&self, _rect: &Rect<f32>) {
        unimplemented!()
    }

    fn copy_surface(
        &self,
        _surface: SourceSurface,
        _source: Rect<i32>,
        _destination: Point2D<i32>,
    ) {
        unimplemented!()
    }

    fn create_gradient_stops(
        &self,
        _gradient_stops: Vec<GradientStop>,
        _extend_mode: ExtendMode,
    ) -> GradientStops {
        unimplemented!()
    }

    fn create_path_builder(&self) -> Box<dyn GenericPathBuilder> {
        unimplemented!()
    }

    fn create_similar_draw_target(
        &self,
        _size: &Size2D<i32>,
        _format: SurfaceFormat,
    ) -> Box<dyn GenericDrawTarget> {
        unimplemented!()
    }
    fn create_source_surface_from_data(
        &self,
        _data: &[u8],
        _size: Size2D<i32>,
        _stride: i32,
    ) -> Option<SourceSurface> {
        unimplemented!()
    }
    fn draw_surface(
        &self,
        _surface: SourceSurface,
        _dest: Rect<f64>,
        _source: Rect<f64>,
        _filter: Filter,
        _draw_options: &DrawOptions,
    ) {
        unimplemented!()
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
        unimplemented!()
    }
    fn fill(&self, _path: &Path, _pattern: Pattern, _draw_options: &DrawOptions) {
        unimplemented!()
    }
    fn fill_rect(&self, _rect: &Rect<f32>, _pattern: Pattern, _draw_options: Option<&DrawOptions>) {
        unimplemented!()
    }
    fn get_format(&self) -> SurfaceFormat {
        unimplemented!()
    }
    fn get_size(&self) -> Size2D<i32> {
        unimplemented!()
    }
    fn get_transform(&self) -> Transform2D<f32> {
        unimplemented!()
    }
    fn pop_clip(&self) {
        unimplemented!()
    }
    fn push_clip(&self, _path: &Path) {
        unimplemented!()
    }
    fn set_transform(&self, _matrix: &Transform2D<f32>) {
        unimplemented!()
    }
    fn snapshot(&self) -> SourceSurface {
        unimplemented!()
    }
    fn stroke(
        &self,
        _path: &Path,
        _pattern: Pattern,
        _stroke_options: &StrokeOptions,
        _draw_options: &DrawOptions,
    ) {
        unimplemented!()
    }
    fn stroke_line(
        &self,
        _start: Point2D<f32>,
        _end: Point2D<f32>,
        _pattern: Pattern,
        _stroke_options: &StrokeOptions,
        _draw_options: &DrawOptions,
    ) {
        unimplemented!()
    }
    fn stroke_rect(
        &self,
        _rect: &Rect<f32>,
        _pattern: Pattern,
        _stroke_options: &StrokeOptions<'_>,
        _draw_options: &DrawOptions,
    ) {
        unimplemented!()
    }

    fn snapshot_data(&self, _f: &dyn Fn(&[u8]) -> Vec<u8>) -> Vec<u8> {
        unimplemented!()
    }

    fn snapshot_data_owned(&self) -> Vec<u8> {
        unimplemented!()
    }
}

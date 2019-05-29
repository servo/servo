/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use cssparser::RGBA;
use crate::canvas_data::{
    Backend, CanvasPaintState, Color, CompositionOp, DrawOptions, GenericDrawTarget,
    GenericPathBuilder, Path, Pattern, StrokeOptions,
};
use crate::canvas_paint_thread::AntialiasMode;
use canvas_traits::canvas::*;
use euclid::{Rect, Size2D, Transform2D};
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
        _drawtarget: &GenericDrawTarget,
    ) {
        unimplemented!()
    }

    fn set_stroke_style<'a>(
        &mut self,
        _style: FillOrStrokeStyle,
        _state: &mut CanvasPaintState<'a>,
        _drawtarget: &GenericDrawTarget)
    {
        unimplemented!()
    }

    fn set_global_composition<'a>(&mut self, _op: CompositionOrBlending, _state: &mut CanvasPaintState<'a>) {
        unimplemented!()
    }

    fn create_drawtarget(&self, _size: Size2D<u64>) -> Box<GenericDrawTarget> {
        unimplemented!()
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
        match *self  {
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
    pub fn transformed_copy_to_builder(&self, _transform: &Transform2D<f32>) -> Box<GenericPathBuilder> {
        unimplemented!()
    }

    pub fn contains_point(&self, _x: f64, _y: f64, _path_transform: &Transform2D<f32>) -> bool {
        unimplemented!()
    }

    pub fn copy_to_builder(&self) -> Box<GenericPathBuilder> {
        unimplemented!()
    }
}

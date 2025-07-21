/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use canvas_traits::canvas::{
    CompositionOptions, FillOrStrokeStyle, LineOptions, Path, ShadowOptions,
};
use compositing_traits::SerializableImageData;
use euclid::default::{Point2D, Rect, Size2D, Transform2D};
use pixels::Snapshot;
use webrender_api::ImageDescriptor;

use crate::canvas_data::{Filter, TextRun};

pub(crate) trait Backend: Clone + Sized {
    type DrawTarget: GenericDrawTarget<Self>;
    type SourceSurface;

    fn create_drawtarget(&self, size: Size2D<u64>) -> Self::DrawTarget;
}

// This defines required methods for a DrawTarget (currently only implemented for raqote).  The
// prototypes are derived from the now-removed Azure backend's methods.
pub(crate) trait GenericDrawTarget<B: Backend> {
    fn clear_rect(&mut self, rect: &Rect<f32>, transform: Transform2D<f32>);
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
        composition_options: CompositionOptions,
        transform: Transform2D<f32>,
    );
    fn draw_surface_with_shadow(
        &self,
        surface: B::SourceSurface,
        dest: &Point2D<f32>,
        shadow_options: ShadowOptions,
        composition_options: CompositionOptions,
    );
    fn fill(
        &mut self,
        path: &Path,
        style: FillOrStrokeStyle,
        composition_options: CompositionOptions,
        transform: Transform2D<f32>,
    );
    fn fill_text(
        &mut self,
        text_runs: Vec<TextRun>,
        start: Point2D<f32>,
        style: FillOrStrokeStyle,
        composition_options: CompositionOptions,
        transform: Transform2D<f32>,
    );
    fn fill_rect(
        &mut self,
        rect: &Rect<f32>,
        style: FillOrStrokeStyle,
        composition_options: CompositionOptions,
        transform: Transform2D<f32>,
    );
    fn get_size(&self) -> Size2D<i32>;
    fn pop_clip(&mut self);
    fn push_clip(&mut self, path: &Path, transform: Transform2D<f32>);
    fn push_clip_rect(&mut self, rect: &Rect<i32>);
    fn stroke(
        &mut self,
        path: &Path,
        style: FillOrStrokeStyle,
        line_options: LineOptions,
        composition_options: CompositionOptions,
        transform: Transform2D<f32>,
    );
    fn stroke_rect(
        &mut self,
        rect: &Rect<f32>,
        style: FillOrStrokeStyle,
        line_options: LineOptions,
        composition_options: CompositionOptions,
        transform: Transform2D<f32>,
    );
    fn surface(&self) -> B::SourceSurface;
    fn image_descriptor_and_serializable_data(&self) -> (ImageDescriptor, SerializableImageData);
    fn snapshot(&self) -> Snapshot;
}

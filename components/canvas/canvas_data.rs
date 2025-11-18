/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use base::Epoch;
use canvas_traits::canvas::*;
use compositing_traits::CrossProcessCompositorApi;
use euclid::default::{Point2D, Rect, Size2D, Transform2D};
use pixels::Snapshot;
use webrender_api::ImageKey;

use crate::backend::GenericDrawTarget;

// Asserts on WR texture cache update for zero sized image with raw data.
// https://github.com/servo/webrender/blob/main/webrender/src/texture_cache.rs#L1475
const MIN_WR_IMAGE_SIZE: Size2D<u64> = Size2D::new(1, 1);

#[derive(Clone, Copy)]
pub(crate) enum Filter {
    Bilinear,
    Nearest,
}

pub(crate) struct CanvasData<DrawTarget: GenericDrawTarget> {
    draw_target: DrawTarget,
    compositor_api: CrossProcessCompositorApi,
    image_key: Option<ImageKey>,
}

impl<DrawTarget: GenericDrawTarget> CanvasData<DrawTarget> {
    pub(crate) fn new(
        size: Size2D<u64>,
        compositor_api: CrossProcessCompositorApi,
    ) -> CanvasData<DrawTarget> {
        CanvasData {
            draw_target: DrawTarget::new(size.max(MIN_WR_IMAGE_SIZE).cast()),
            compositor_api,
            image_key: None,
        }
    }

    pub(crate) fn set_image_key(&mut self, image_key: ImageKey) {
        let (descriptor, data) = self.draw_target.image_descriptor_and_serializable_data();
        self.compositor_api.add_image(image_key, descriptor, data);

        if let Some(old_image_key) = self.image_key.replace(image_key) {
            self.compositor_api.delete_image(old_image_key);
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn draw_image(
        &mut self,
        snapshot: Snapshot,
        dest_rect: Rect<f64>,
        source_rect: Rect<f64>,
        smoothing_enabled: bool,
        shadow_options: ShadowOptions,
        composition_options: CompositionOptions,
        transform: Transform2D<f64>,
    ) {
        // We round up the floating pixel values to draw the pixels
        let source_rect = source_rect.ceil();
        // It discards the extra pixels (if any) that won't be painted
        let snapshot = if Rect::from_size(snapshot.size().to_f64()).contains_rect(&source_rect) {
            snapshot.get_rect(source_rect.to_u32())
        } else {
            snapshot
        };

        let writer = |draw_target: &mut DrawTarget, transform| {
            write_image::<DrawTarget>(
                draw_target,
                snapshot,
                dest_rect,
                smoothing_enabled,
                composition_options,
                transform,
            );
        };

        if shadow_options.need_to_draw_shadow() {
            let rect = Rect::new(
                Point2D::new(dest_rect.origin.x as f32, dest_rect.origin.y as f32),
                Size2D::new(dest_rect.size.width as f32, dest_rect.size.height as f32),
            );

            self.draw_with_shadow(
                &rect,
                shadow_options,
                composition_options,
                transform,
                writer,
            );
        } else {
            writer(&mut self.draw_target, transform);
        }
    }

    pub(crate) fn fill_text(
        &mut self,
        text_bounds: Rect<f64>,
        text_runs: Vec<TextRun>,
        fill_or_stroke_style: FillOrStrokeStyle,
        _shadow_options: ShadowOptions,
        composition_options: CompositionOptions,
        transform: Transform2D<f64>,
    ) {
        self.maybe_bound_shape_with_pattern(
            fill_or_stroke_style,
            composition_options,
            &text_bounds,
            transform,
            |self_, style| {
                self_
                    .draw_target
                    .fill_text(text_runs, style, composition_options, transform);
            },
        );
    }

    pub(crate) fn stroke_text(
        &mut self,
        text_bounds: Rect<f64>,
        text_runs: Vec<TextRun>,
        fill_or_stroke_style: FillOrStrokeStyle,
        line_options: LineOptions,
        _shadow_options: ShadowOptions,
        composition_options: CompositionOptions,
        transform: Transform2D<f64>,
    ) {
        self.maybe_bound_shape_with_pattern(
            fill_or_stroke_style,
            composition_options,
            &text_bounds,
            transform,
            |self_, style| {
                self_.draw_target.stroke_text(
                    text_runs,
                    style,
                    line_options,
                    composition_options,
                    transform,
                );
            },
        );
    }

    pub(crate) fn fill_rect(
        &mut self,
        rect: &Rect<f32>,
        style: FillOrStrokeStyle,
        shadow_options: ShadowOptions,
        composition_options: CompositionOptions,
        transform: Transform2D<f64>,
    ) {
        if style.is_zero_size_gradient() {
            return; // Paint nothing if gradient size is zero.
        }

        if shadow_options.need_to_draw_shadow() {
            self.draw_with_shadow(
                rect,
                shadow_options,
                composition_options,
                transform,
                |new_draw_target, transform| {
                    new_draw_target.fill_rect(rect, style, composition_options, transform);
                },
            );
        } else {
            self.maybe_bound_shape_with_pattern(
                style,
                composition_options,
                &rect.cast(),
                transform,
                |self_, style| {
                    self_
                        .draw_target
                        .fill_rect(rect, style, composition_options, transform);
                },
            );
        }
    }

    pub(crate) fn clear_rect(&mut self, rect: &Rect<f32>, transform: Transform2D<f64>) {
        self.draw_target.clear_rect(rect, transform);
    }

    pub(crate) fn stroke_rect(
        &mut self,
        rect: &Rect<f32>,
        style: FillOrStrokeStyle,
        line_options: LineOptions,
        shadow_options: ShadowOptions,
        composition_options: CompositionOptions,
        transform: Transform2D<f64>,
    ) {
        if style.is_zero_size_gradient() {
            return; // Paint nothing if gradient size is zero.
        }

        if shadow_options.need_to_draw_shadow() {
            self.draw_with_shadow(
                rect,
                shadow_options,
                composition_options,
                transform,
                |new_draw_target, transform| {
                    new_draw_target.stroke_rect(
                        rect,
                        style,
                        line_options,
                        composition_options,
                        transform,
                    );
                },
            );
        } else {
            self.maybe_bound_shape_with_pattern(
                style,
                composition_options,
                &rect.cast(),
                transform,
                |self_, style| {
                    self_.draw_target.stroke_rect(
                        rect,
                        style,
                        line_options,
                        composition_options,
                        transform,
                    );
                },
            )
        }
    }

    pub(crate) fn fill_path(
        &mut self,
        path: &Path,
        fill_rule: FillRule,
        style: FillOrStrokeStyle,
        _shadow_options: ShadowOptions,
        composition_options: CompositionOptions,
        transform: Transform2D<f64>,
    ) {
        if style.is_zero_size_gradient() {
            return; // Paint nothing if gradient size is zero.
        }

        self.maybe_bound_shape_with_pattern(
            style,
            composition_options,
            &path.bounding_box(),
            transform,
            |self_, style| {
                self_
                    .draw_target
                    .fill(path, fill_rule, style, composition_options, transform)
            },
        )
    }

    pub(crate) fn stroke_path(
        &mut self,
        path: &Path,
        style: FillOrStrokeStyle,
        line_options: LineOptions,
        _shadow_options: ShadowOptions,
        composition_options: CompositionOptions,
        transform: Transform2D<f64>,
    ) {
        if style.is_zero_size_gradient() {
            return; // Paint nothing if gradient size is zero.
        }

        self.maybe_bound_shape_with_pattern(
            style,
            composition_options,
            &path.bounding_box(),
            transform,
            |self_, style| {
                self_
                    .draw_target
                    .stroke(path, style, line_options, composition_options, transform);
            },
        )
    }

    pub(crate) fn clip_path(
        &mut self,
        path: &Path,
        fill_rule: FillRule,
        transform: Transform2D<f64>,
    ) {
        self.draw_target.push_clip(path, fill_rule, transform);
    }

    /// <https://html.spec.whatwg.org/multipage/#reset-the-rendering-context-to-its-default-state>
    pub(crate) fn recreate(&mut self, size: Option<Size2D<u64>>) {
        let size = size
            .unwrap_or_else(|| self.draw_target.get_size().to_u64())
            .max(MIN_WR_IMAGE_SIZE);

        // Step 1. Clear canvas's bitmap to transparent black.
        self.draw_target = self
            .draw_target
            .create_similar_draw_target(&Size2D::new(size.width, size.height).cast());

        self.update_image_rendering(None);
    }

    /// Update image in WebRender
    pub(crate) fn update_image_rendering(&mut self, canvas_epoch: Option<Epoch>) {
        let Some(image_key) = self.image_key else {
            return;
        };

        let (descriptor, data) = {
            let _span =
                profile_traits::trace_span!("image_descriptor_and_serializable_data",).entered();
            self.draw_target.image_descriptor_and_serializable_data()
        };

        self.compositor_api
            .update_image(image_key, descriptor, data, canvas_epoch);
    }

    // https://html.spec.whatwg.org/multipage/#dom-context-2d-putimagedata
    pub(crate) fn put_image_data(&mut self, snapshot: Snapshot, rect: Rect<u32>) {
        assert_eq!(rect.size, snapshot.size());
        let source_surface = self
            .draw_target
            .create_source_surface_from_data(snapshot)
            .unwrap();
        self.draw_target.copy_surface(
            source_surface,
            Rect::from_size(rect.size.to_i32()),
            rect.origin.to_i32(),
        );
    }

    fn create_draw_target_for_shadow(&self, source_rect: &Rect<f32>) -> DrawTarget {
        self.draw_target.create_similar_draw_target(&Size2D::new(
            source_rect.size.width as i32,
            source_rect.size.height as i32,
        ))
    }

    fn draw_with_shadow<F>(
        &self,
        rect: &Rect<f32>,
        shadow_options: ShadowOptions,
        composition_options: CompositionOptions,
        transform: Transform2D<f64>,
        draw_shadow_source: F,
    ) where
        F: FnOnce(&mut DrawTarget, Transform2D<f64>),
    {
        let shadow_src_rect = transform.outer_transformed_rect(&rect.cast());
        // Because this comes from the rect on f32 precision, casting it down should be ok.
        let mut new_draw_target = self.create_draw_target_for_shadow(&shadow_src_rect.cast());
        let shadow_transform = transform
            .then(&Transform2D::identity().pre_translate(-shadow_src_rect.origin.to_vector()));
        draw_shadow_source(&mut new_draw_target, shadow_transform);
        self.draw_target.draw_surface_with_shadow(
            new_draw_target.surface(),
            &Point2D::new(
                shadow_src_rect.origin.x as f32,
                shadow_src_rect.origin.y as f32,
            ),
            shadow_options,
            composition_options,
        );
    }

    /// Push a clip to the draw target to respect the non-repeating bound (either x, y, or both)
    /// of the given pattern.
    fn maybe_bound_shape_with_pattern<F>(
        &mut self,
        style: FillOrStrokeStyle,
        composition_options: CompositionOptions,
        path_bound_box: &Rect<f64>,
        transform: Transform2D<f64>,
        draw_shape: F,
    ) where
        F: FnOnce(&mut Self, FillOrStrokeStyle),
    {
        let x_bound = style.x_bound();
        let y_bound = style.y_bound();
        // Clear operations are also unbounded.
        if matches!(
            composition_options.composition_operation,
            CompositionOrBlending::Composition(CompositionStyle::Clear)
        ) || (x_bound.is_none() && y_bound.is_none())
        {
            draw_shape(self, style);
            return;
        }
        let rect = Rect::from_size(Size2D::new(
            x_bound.unwrap_or(path_bound_box.size.width.ceil() as u32),
            y_bound.unwrap_or(path_bound_box.size.height.ceil() as u32),
        ))
        .cast();
        let rect = transform.outer_transformed_rect(&rect);
        self.draw_target.push_clip_rect(&rect.cast());
        draw_shape(self, style);
        self.draw_target.pop_clip();
    }

    /// It reads image data from the canvas
    /// canvas_size: The size of the canvas we're reading from
    /// read_rect: The area of the canvas we want to read from
    #[servo_tracing::instrument(skip_all)]
    pub(crate) fn read_pixels(&mut self, read_rect: Option<Rect<u32>>) -> Snapshot {
        let canvas_size = self.draw_target.get_size().cast();

        if let Some(read_rect) = read_rect {
            let canvas_rect = Rect::from_size(canvas_size);
            if canvas_rect
                .intersection(&read_rect)
                .is_none_or(|rect| rect.is_empty())
            {
                Snapshot::empty()
            } else {
                self.draw_target.snapshot().get_rect(read_rect)
            }
        } else {
            self.draw_target.snapshot()
        }
    }

    pub(crate) fn pop_clips(&mut self, clips: usize) {
        for _ in 0..clips {
            self.draw_target.pop_clip();
        }
    }
}

impl<D: GenericDrawTarget> Drop for CanvasData<D> {
    fn drop(&mut self) {
        if let Some(image_key) = self.image_key {
            self.compositor_api.delete_image(image_key);
        }
    }
}

/// It writes an image to the destination target
/// draw_target: the destination target where the image_data will be copied
/// image_data: Pixel information of the image to be written. It takes RGBA8
/// image_size: The size of the image to be written
/// dest_rect: Area of the destination target where the pixels will be copied
/// smoothing_enabled: It determines if smoothing is applied to the image result
/// premultiply: Determines whenever the image data should be premultiplied or not
fn write_image<DrawTarget: GenericDrawTarget>(
    draw_target: &mut DrawTarget,
    snapshot: Snapshot,
    dest_rect: Rect<f64>,
    smoothing_enabled: bool,
    composition_options: CompositionOptions,
    transform: Transform2D<f64>,
) {
    if snapshot.size().is_empty() {
        return;
    }

    let image_rect = Rect::new(Point2D::zero(), snapshot.size().cast());

    // From spec https://html.spec.whatwg.org/multipage/#dom-context-2d-drawimage
    // When scaling up, if the imageSmoothingEnabled attribute is set to true, the user agent should attempt
    // to apply a smoothing algorithm to the image data when it is scaled.
    // Otherwise, the image must be rendered using nearest-neighbor interpolation.
    let filter = if smoothing_enabled {
        Filter::Bilinear
    } else {
        Filter::Nearest
    };

    let source_surface = draw_target
        .create_source_surface_from_data(snapshot)
        .unwrap();

    draw_target.draw_surface(
        source_surface,
        dest_rect,
        image_rect,
        filter,
        composition_options,
        transform,
    );
}

pub(crate) trait RectToi32 {
    fn ceil(&self) -> Rect<f64>;
}

impl RectToi32 for Rect<f64> {
    fn ceil(&self) -> Rect<f64> {
        Rect::new(
            Point2D::new(self.origin.x.ceil(), self.origin.y.ceil()),
            Size2D::new(self.size.width.ceil(), self.size.height.ceil()),
        )
    }
}

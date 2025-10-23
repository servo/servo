/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use euclid::{Point2D, Rect, Scale, Transform2D, Vector2D};
use webrender_api::ScrollLocation;
use webrender_api::units::{DevicePixel, DevicePoint, DeviceRect, DeviceSize, DeviceVector2D};

/// A [`PinchZoom`] describes the pinch zoom viewport of a `WebView`. This is used to
/// track the current pinch zoom transformation and to clamp all pinching and panning
/// to the unscaled `WebView` viewport.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct PinchZoom {
    zoom_factor: f32,
    transform: Transform2D<f32, DevicePixel, DevicePixel>,
    unscaled_viewport_size: DeviceSize,
}

impl PinchZoom {
    pub(crate) fn new(webview_rect: DeviceRect) -> Self {
        Self {
            zoom_factor: 1.0,
            unscaled_viewport_size: webview_rect.size(),
            transform: Transform2D::identity(),
        }
    }

    pub(crate) fn transform(&self) -> Transform2D<f32, DevicePixel, DevicePixel> {
        self.transform
    }

    pub(crate) fn zoom_factor(&self) -> Scale<f32, DevicePixel, DevicePixel> {
        Scale::new(self.zoom_factor)
    }

    fn set_transform(&mut self, transform: Transform2D<f32, DevicePixel, DevicePixel>) {
        let rect = Rect::new(
            Point2D::origin(),
            self.unscaled_viewport_size.to_vector().to_size(),
        )
        .cast_unit();
        let mut rect = transform
            .inverse()
            .expect("Should always be able to invert provided transform")
            .outer_transformed_rect(&rect);
        rect.origin = rect.origin.clamp(
            Point2D::origin(),
            (self.unscaled_viewport_size - rect.size)
                .to_vector()
                .to_point(),
        );
        let scale = self.unscaled_viewport_size.width / rect.width();
        self.transform = Transform2D::identity()
            .then_translate(Vector2D::new(-rect.origin.x, -rect.origin.y))
            .then_scale(scale, scale);
    }

    pub(crate) fn zoom(&mut self, magnification: f32, new_center: DevicePoint) {
        const MINIMUM_PINCH_ZOOM: f32 = 1.0;
        const MAXIMUM_PINCH_ZOOM: f32 = 10.0;
        let new_factor =
            (self.zoom_factor * magnification).clamp(MINIMUM_PINCH_ZOOM, MAXIMUM_PINCH_ZOOM);
        let old_factor = std::mem::replace(&mut self.zoom_factor, new_factor);

        if self.zoom_factor <= 1.0 {
            self.transform = Transform2D::identity();
            return;
        }

        let magnification = self.zoom_factor / old_factor;
        let transform = self
            .transform
            .then_translate(Vector2D::new(-new_center.x, -new_center.y))
            .then_scale(magnification, magnification)
            .then_translate(Vector2D::new(new_center.x, new_center.y));
        self.set_transform(transform);
    }

    /// Pan the pinch zoom viewoprt by the given [`ScrollLocation`] and if it is a delta,
    /// modify the delta to reflect the remaining unused scroll delta.
    pub(crate) fn pan(&mut self, scroll_location: &mut ScrollLocation) {
        // TODO: The delta passed help in `ScrollLocation` is a LayoutVector2D, but is actually
        // in DevicePixels! This should reflect reality.
        match scroll_location {
            ScrollLocation::Delta(delta) => {
                let remaining = self.pan_with_device_scroll(DeviceScroll::Delta(
                    DeviceVector2D::new(delta.x, delta.y),
                ));
                *delta = Vector2D::new(remaining.x, remaining.y)
            },
            ScrollLocation::Start => {
                self.pan_with_device_scroll(DeviceScroll::Start);
            },
            ScrollLocation::End => {
                self.pan_with_device_scroll(DeviceScroll::End);
            },
        }
    }

    /// Pan the pinch zoom viewport by the given delta and return the remaining device
    /// pixel value that was unused.
    pub(crate) fn pan_with_device_scroll(&mut self, scroll: DeviceScroll) -> DeviceVector2D {
        let current_viewport = Rect::new(
            Point2D::origin(),
            self.unscaled_viewport_size.to_vector().to_size(),
        );
        let layout_viewport_in_device_pixels =
            self.transform.outer_transformed_rect(&current_viewport);
        let max_viewport_offset = -(layout_viewport_in_device_pixels.size -
            self.unscaled_viewport_size.to_vector().to_size());
        let max_delta = layout_viewport_in_device_pixels.origin - max_viewport_offset;

        let delta = match scroll {
            DeviceScroll::Delta(delta) => delta,
            DeviceScroll::Start => DeviceVector2D::new(0.0, max_delta.y),
            DeviceScroll::End => {
                DeviceVector2D::new(0.0, -layout_viewport_in_device_pixels.origin.y)
            },
        };

        let mut remaining = Vector2D::zero();
        if delta.x < 0.0 {
            remaining.x = (delta.x - layout_viewport_in_device_pixels.origin.x).min(0.0);
        }
        if delta.y < 0.0 {
            remaining.y = (delta.y - layout_viewport_in_device_pixels.origin.y).min(0.0);
        }
        if delta.x > 0.0 {
            remaining.x = (delta.x - max_delta.x).max(0.0);
        }
        if delta.y > 0.0 {
            remaining.y = (delta.y - max_delta.y).max(0.0);
        }

        self.set_transform(
            self.transform
                .then_translate(Vector2D::new(-delta.x, -delta.y)),
        );

        remaining
    }
}

pub(crate) enum DeviceScroll {
    Delta(DeviceVector2D),
    Start,
    End,
}

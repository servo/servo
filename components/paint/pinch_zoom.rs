/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use embedder_traits::Scroll;
use euclid::{Point2D, Rect, Scale, Transform2D, Vector2D};
use paint_api::PinchZoomInfos;
use paint_api::viewport_description::ViewportDescription;
use style_traits::CSSPixel;
use webrender_api::units::{DevicePixel, DevicePoint, DeviceRect, DeviceSize, DeviceVector2D};

/// A [`PinchZoom`] describes the pinch zoom viewport of a `WebView`. This is used to
/// track the current pinch zoom transformation and to clamp all pinching and panning
/// to the unscaled `WebView` viewport.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct PinchZoom {
    zoom_factor: f32,
    transform: Transform2D<f32, DevicePixel, DevicePixel>,
    unscaled_viewport_size: DeviceSize,
    /// A [`ViewportDescription`] for the [`WebViewRenderer`], which contains the limitations
    /// and initial values for zoom derived from the `viewport` meta tag in web content.
    viewport_description: ViewportDescription,
}

impl PinchZoom {
    pub(crate) fn new(webview_rect: DeviceRect) -> Self {
        Self {
            zoom_factor: 1.0,
            unscaled_viewport_size: webview_rect.size(),
            transform: Transform2D::identity(),
            viewport_description: Default::default(),
        }
    }

    pub(crate) fn transform(&self) -> Transform2D<f32, DevicePixel, DevicePixel> {
        self.transform
    }

    pub(crate) fn zoom_factor(&self) -> Scale<f32, DevicePixel, DevicePixel> {
        Scale::new(self.zoom_factor)
    }

    pub(crate) fn resize_unscaled_viewport(&mut self, webview_rect: DeviceRect) {
        self.unscaled_viewport_size = webview_rect.size();
    }

    /// The boundary of the pinch zoom viewport relative to the unscaled viewport. For the
    /// script's `VisualViewport` interface and calculations (such as scroll).
    fn pinch_zoom_rect_relative_to_unscaled_viewport(&self) -> Rect<f32, DevicePixel> {
        let rect = Rect::new(
            Point2D::origin(),
            self.unscaled_viewport_size.to_vector().to_size(),
        )
        .cast_unit();
        self.transform
            .inverse()
            .expect("Should always be able to invert provided transform")
            .outer_transformed_rect(&rect)
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
        let new_factor = self
            .viewport_description
            .clamp_zoom(self.zoom_factor * magnification);
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

    /// Pan the pinch zoom viewoprt by the given [`Scroll`] and if it is a delta,
    /// modify the delta to reflect the remaining unused scroll delta.
    pub(crate) fn pan(&mut self, scroll: &mut Scroll, scale: Scale<f32, CSSPixel, DevicePixel>) {
        let remaining = self.pan_with_device_scroll(*scroll, scale);

        if let Scroll::Delta(delta) = scroll {
            *delta = remaining.into();
        }
    }

    /// Pan the pinch zoom viewport by the given delta and return the remaining device
    /// pixel value that was unused.
    pub(crate) fn pan_with_device_scroll(
        &mut self,
        scroll: Scroll,
        scale: Scale<f32, CSSPixel, DevicePixel>,
    ) -> DeviceVector2D {
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
            Scroll::Delta(delta) => delta.as_device_vector(scale),
            Scroll::Start => DeviceVector2D::new(0.0, max_delta.y),
            Scroll::End => DeviceVector2D::new(0.0, -layout_viewport_in_device_pixels.origin.y),
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

    /// Get the [`PinchZoomInfos`] from this [`PinchZoom`] state.
    pub(crate) fn get_pinch_zoom_infos_for_script(
        &self,
        viewport_scale: Scale<f32, CSSPixel, DevicePixel>,
    ) -> PinchZoomInfos {
        PinchZoomInfos {
            zoom_factor: Scale::new(self.zoom_factor),
            rect: self.pinch_zoom_rect_relative_to_unscaled_viewport() / viewport_scale,
        }
    }

    pub(crate) fn set_viewport_description(&mut self, viewport_description: ViewportDescription) {
        self.viewport_description = viewport_description;
    }
}

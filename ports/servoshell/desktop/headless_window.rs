/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! A headless window implementation.

#![deny(clippy::panic)]
#![deny(clippy::unwrap_used)]

use std::cell::Cell;
use std::rc::Rc;

use euclid::{Point2D, Scale, Size2D};
use servo::servo_geometry::{
    DeviceIndependentIntRect, DeviceIndependentPixel, convert_rect_to_css_pixel,
};
use servo::webrender_api::units::{DeviceIntPoint, DeviceIntRect, DeviceIntSize, DevicePixel};
use servo::{RenderingContext, ScreenGeometry, SoftwareRenderingContext, WebView};
use winit::dpi::PhysicalSize;

use crate::prefs::ServoShellPreferences;
use crate::window::{MIN_WINDOW_INNER_SIZE, PlatformWindow, ServoShellWindow, ServoShellWindowId};

pub struct Window {
    fullscreen: Cell<bool>,
    device_pixel_ratio_override: Option<Scale<f32, DeviceIndependentPixel, DevicePixel>>,
    inner_size: Cell<DeviceIntSize>,
    screen_size: Size2D<i32, DevicePixel>,
    // virtual top-left position of the window in device pixels.
    window_position: Cell<Point2D<i32, DevicePixel>>,
    rendering_context: Rc<SoftwareRenderingContext>,
}

impl Window {
    pub fn new(servoshell_preferences: &ServoShellPreferences) -> Rc<Self> {
        let size = servoshell_preferences.initial_window_size;

        let device_pixel_ratio_override = servoshell_preferences.device_pixel_ratio_override;
        let device_pixel_ratio_override: Option<Scale<f32, DeviceIndependentPixel, DevicePixel>> =
            device_pixel_ratio_override.map(Scale::new);
        let hidpi_factor = device_pixel_ratio_override.unwrap_or_else(Scale::identity);

        let inner_size = (size.to_f32() * hidpi_factor).to_i32();
        let physical_size = PhysicalSize::new(inner_size.width as u32, inner_size.height as u32);
        let rendering_context =
            SoftwareRenderingContext::new(physical_size).expect("Failed to create WR surfman");

        let screen_size = servoshell_preferences
            .screen_size_override
            .map_or(inner_size * 2, |screen_size_override| {
                (screen_size_override.to_f32() * hidpi_factor).to_i32()
            });

        let window = Window {
            fullscreen: Cell::new(false),
            device_pixel_ratio_override,
            inner_size: Cell::new(inner_size),
            screen_size,
            window_position: Cell::new(Point2D::zero()),
            rendering_context: Rc::new(rendering_context),
        };

        Rc::new(window)
    }
}

impl PlatformWindow for Window {
    fn id(&self) -> ServoShellWindowId {
        0.into()
    }

    fn screen_geometry(&self) -> servo::ScreenGeometry {
        ScreenGeometry {
            size: self.screen_size,
            available_size: self.screen_size,
            window_rect: DeviceIntRect::from_origin_and_size(
                self.window_position.get(),
                self.inner_size.get(),
            ),
        }
    }

    fn set_position(&self, point: DeviceIntPoint) {
        self.window_position.set(point);
    }

    fn request_repaint(&self, window: &ServoShellWindow) {
        window.repaint_webviews();
    }

    fn request_resize(&self, webview: &WebView, new_size: DeviceIntSize) -> Option<DeviceIntSize> {
        // Do not let the window size get smaller than `MIN_WINDOW_INNER_SIZE` or larger
        // than twice the screen size.
        let new_size = new_size.clamp(MIN_WINDOW_INNER_SIZE, self.screen_size * 2);
        if self.inner_size.get() == new_size {
            return Some(new_size);
        }

        self.inner_size.set(new_size);

        // Because we are managing the rendering surface ourselves, there will be no other
        // notification (such as from the display manager) that it has changed size, so we
        // must notify the compositor here.
        webview.move_resize(new_size.to_f32().into());
        webview.resize(PhysicalSize::new(
            new_size.width as u32,
            new_size.height as u32,
        ));

        Some(new_size)
    }

    fn device_hidpi_scale_factor(&self) -> Scale<f32, DeviceIndependentPixel, DevicePixel> {
        Scale::new(1.0)
    }

    fn hidpi_scale_factor(&self) -> Scale<f32, DeviceIndependentPixel, DevicePixel> {
        self.device_pixel_ratio_override
            .unwrap_or_else(|| self.device_hidpi_scale_factor())
    }

    fn set_fullscreen(&self, state: bool) {
        self.fullscreen.set(state);
    }

    fn get_fullscreen(&self) -> bool {
        self.fullscreen.get()
    }

    #[cfg(feature = "webxr")]
    fn new_glwindow(
        &self,
        _event_loop: &winit::event_loop::ActiveEventLoop,
    ) -> Rc<dyn servo::webxr::glwindow::GlWindow> {
        unimplemented!()
    }

    fn window_rect(&self) -> DeviceIndependentIntRect {
        convert_rect_to_css_pixel(
            DeviceIntRect::from_origin_and_size(self.window_position.get(), self.inner_size.get()),
            self.hidpi_scale_factor(),
        )
    }

    fn rendering_context(&self) -> Rc<dyn RenderingContext> {
        self.rendering_context.clone()
    }

    fn maximize(&self, webview: &WebView) {
        self.window_position.set(Point2D::zero());
        self.inner_size.set(self.screen_size);
        // Because we are managing the rendering surface ourselves, there will be no other
        // notification (such as from the display manager) that it has changed size, so we
        // must notify the compositor here.
        webview.move_resize(self.screen_size.to_f32().into());
        webview.resize(PhysicalSize::new(
            self.screen_size.width as u32,
            self.screen_size.height as u32,
        ));
    }

    fn focused(&self) -> bool {
        true
    }
}

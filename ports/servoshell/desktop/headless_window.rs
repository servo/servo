/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! A headless window implementation.

use std::cell::Cell;
use std::rc::Rc;

use euclid::num::Zero;
use euclid::{Length, Point2D, Scale, Size2D};
use servo::servo_geometry::{
    DeviceIndependentIntRect, DeviceIndependentPixel, convert_rect_to_css_pixel,
};
use servo::webrender_api::units::{DeviceIntPoint, DeviceIntRect, DeviceIntSize, DevicePixel};
use servo::{RenderingContext, ScreenGeometry, SoftwareRenderingContext, WebView};
use winit::dpi::PhysicalSize;

use super::app_state::RunningAppState;
use crate::desktop::window_trait::{MIN_INNER_HEIGHT, MIN_INNER_WIDTH, WindowPortsMethods};
use crate::prefs::ServoShellPreferences;

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
    #[allow(clippy::new_ret_no_self)]
    pub fn new(servoshell_preferences: &ServoShellPreferences) -> Rc<dyn WindowPortsMethods> {
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

impl WindowPortsMethods for Window {
    fn id(&self) -> winit::window::WindowId {
        winit::window::WindowId::dummy()
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

    fn request_resize(
        &self,
        webview: &WebView,
        outer_size: DeviceIntSize,
    ) -> Option<DeviceIntSize> {
        let new_size = DeviceIntSize::new(
            outer_size.width.max(MIN_INNER_WIDTH),
            outer_size.height.max(MIN_INNER_HEIGHT),
        );
        if self.inner_size.get() == new_size {
            return Some(new_size);
        }

        self.inner_size.set(new_size);

        // Because we are managing the rendering surface ourselves, there will be no other
        // notification (such as from the display manager) that it has changed size, so we
        // must notify the compositor here.
        webview.move_resize(outer_size.to_f32().into());
        webview.resize(PhysicalSize::new(
            outer_size.width as u32,
            outer_size.height as u32,
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

    fn page_height(&self) -> f32 {
        let height = self.inner_size.get().height;
        let dpr = self.hidpi_scale_factor();
        height as f32 * dpr.get()
    }

    fn set_fullscreen(&self, state: bool) {
        self.fullscreen.set(state);
    }

    fn get_fullscreen(&self) -> bool {
        self.fullscreen.get()
    }

    fn handle_winit_event(&self, _: Rc<RunningAppState>, _: winit::event::WindowEvent) {
        // Not expecting any winit events.
    }

    fn new_glwindow(
        &self,
        _events_loop: &winit::event_loop::ActiveEventLoop,
    ) -> Rc<dyn servo::webxr::glwindow::GlWindow> {
        unimplemented!()
    }

    fn winit_window(&self) -> Option<&winit::window::Window> {
        None
    }

    fn toolbar_height(&self) -> Length<f32, DeviceIndependentPixel> {
        Length::zero()
    }

    fn window_rect(&self) -> DeviceIndependentIntRect {
        convert_rect_to_css_pixel(
            DeviceIntRect::from_origin_and_size(self.window_position.get(), self.inner_size.get()),
            self.hidpi_scale_factor(),
        )
    }

    fn set_toolbar_height(&self, _height: Length<f32, DeviceIndependentPixel>) {
        unimplemented!("headless Window only")
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
}

/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! A headless window implementation.

use std::cell::Cell;
use std::rc::Rc;

use euclid::num::Zero;
use euclid::{Box2D, Length, Point2D, Scale, Size2D};
use servo::compositing::windowing::{AnimationState, EmbedderCoordinates, WindowMethods};
use servo::servo_geometry::DeviceIndependentPixel;
use servo::webrender_api::units::{DeviceIntSize, DevicePixel};
use servo::{RenderingContext, SoftwareRenderingContext};
use winit::dpi::PhysicalSize;

use super::app_state::RunningAppState;
use crate::desktop::window_trait::WindowPortsMethods;
use crate::prefs::ServoShellPreferences;

pub struct Window {
    animation_state: Cell<AnimationState>,
    fullscreen: Cell<bool>,
    device_pixel_ratio_override: Option<Scale<f32, DeviceIndependentPixel, DevicePixel>>,
    inner_size: Cell<DeviceIntSize>,
    screen_size: Size2D<i32, DeviceIndependentPixel>,
    window_rect: Box2D<i32, DeviceIndependentPixel>,
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

        let window_rect = Box2D::from_origin_and_size(Point2D::zero(), size.to_i32());

        let screen_size = servoshell_preferences.screen_size_override.map_or_else(
            || window_rect.size(),
            |screen_size_override| screen_size_override.to_i32(),
        );

        let window = Window {
            animation_state: Cell::new(AnimationState::Idle),
            fullscreen: Cell::new(false),
            device_pixel_ratio_override,
            inner_size: Cell::new(inner_size),
            screen_size,
            window_rect,
            rendering_context: Rc::new(rendering_context),
        };

        Rc::new(window)
    }
}

impl WindowPortsMethods for Window {
    fn id(&self) -> winit::window::WindowId {
        winit::window::WindowId::dummy()
    }

    fn request_resize(
        &self,
        webview: &::servo::WebView,
        size: DeviceIntSize,
    ) -> Option<DeviceIntSize> {
        // Surfman doesn't support zero-sized surfaces.
        let new_size = DeviceIntSize::new(size.width.max(1), size.height.max(1));
        if self.inner_size.get() == new_size {
            return Some(new_size);
        }

        self.inner_size.set(new_size);

        // Because we are managing the rendering surface ourselves, there will be no other
        // notification (such as from the display manager) that it has changed size, so we
        // must notify the compositor here.
        webview.resize(PhysicalSize::new(size.width as u32, size.height as u32));

        Some(new_size)
    }

    fn device_hidpi_factor(&self) -> Scale<f32, DeviceIndependentPixel, DevicePixel> {
        Scale::new(1.0)
    }

    fn device_pixel_ratio_override(
        &self,
    ) -> Option<Scale<f32, DeviceIndependentPixel, DevicePixel>> {
        self.device_pixel_ratio_override
    }

    fn page_height(&self) -> f32 {
        let height = self.inner_size.get().height;
        let dpr = self.hidpi_factor();
        height as f32 * dpr.get()
    }

    fn set_fullscreen(&self, state: bool) {
        self.fullscreen.set(state);
    }

    fn get_fullscreen(&self) -> bool {
        self.fullscreen.get()
    }

    fn is_animating(&self) -> bool {
        self.animation_state.get() == AnimationState::Animating
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

    fn set_toolbar_height(&self, _height: Length<f32, DeviceIndependentPixel>) {
        unimplemented!("headless Window only")
    }

    fn rendering_context(&self) -> Rc<dyn RenderingContext> {
        self.rendering_context.clone()
    }
}

impl WindowMethods for Window {
    fn get_coordinates(&self) -> EmbedderCoordinates {
        EmbedderCoordinates {
            window_rect: self.window_rect,
            screen_size: self.screen_size,
            available_screen_size: self.screen_size,
            hidpi_factor: self.hidpi_factor(),
        }
    }

    fn set_animation_state(&self, state: AnimationState) {
        self.animation_state.set(state);
    }
}

/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! A headless window implementation.

use std::cell::Cell;
use std::rc::Rc;
use std::sync::RwLock;

use euclid::num::Zero;
use euclid::{Box2D, Length, Point2D, Scale, Size2D};
use servo::compositing::windowing::{
    AnimationState, EmbedderCoordinates, EmbedderEvent, WindowMethods,
};
use servo::config::opts;
use servo::servo_geometry::DeviceIndependentPixel;
use servo::webrender_api::units::{DeviceIntSize, DevicePixel};

use crate::desktop::window_trait::WindowPortsMethods;

pub struct Window {
    animation_state: Cell<AnimationState>,
    fullscreen: Cell<bool>,
    device_pixel_ratio_override: Option<Scale<f32, DeviceIndependentPixel, DevicePixel>>,
    inner_size: Cell<DeviceIntSize>,
    screen_size: Size2D<i32, DeviceIndependentPixel>,
    window_rect: Box2D<i32, DeviceIndependentPixel>,
    event_queue: RwLock<Vec<EmbedderEvent>>,
}

impl Window {
    #[allow(clippy::new_ret_no_self)]
    pub fn new(
        size: Size2D<u32, DeviceIndependentPixel>,
        device_pixel_ratio_override: Option<f32>,
    ) -> Rc<dyn WindowPortsMethods> {
        let device_pixel_ratio_override: Option<Scale<f32, DeviceIndependentPixel, DevicePixel>> =
            device_pixel_ratio_override.map(Scale::new);
        let hidpi_factor = device_pixel_ratio_override.unwrap_or_else(Scale::identity);

        let size = size.to_i32();
        let inner_size = Cell::new((size.to_f32() * hidpi_factor).to_i32());
        let window_rect = Box2D::from_origin_and_size(Point2D::zero(), size);

        let screen_size = opts::get().screen_size_override.map_or_else(
            || window_rect.size(),
            |screen_size_override| screen_size_override.to_i32(),
        );

        let window = Window {
            animation_state: Cell::new(AnimationState::Idle),
            fullscreen: Cell::new(false),
            device_pixel_ratio_override,
            inner_size,
            screen_size,
            window_rect,
            event_queue: RwLock::new(Vec::new()),
        };

        Rc::new(window)
    }
}

impl WindowPortsMethods for Window {
    fn get_events(&self) -> Vec<EmbedderEvent> {
        match self.event_queue.write() {
            Ok(ref mut event_queue) => std::mem::take(event_queue),
            Err(_) => vec![],
        }
    }

    fn id(&self) -> winit::window::WindowId {
        winit::window::WindowId::dummy()
    }

    fn request_inner_size(&self, size: DeviceIntSize) -> Option<DeviceIntSize> {
        // Surfman doesn't support zero-sized surfaces.
        let new_size = DeviceIntSize::new(size.width.max(1), size.height.max(1));
        if self.inner_size.get() == new_size {
            return Some(new_size);
        }

        self.inner_size.set(new_size);
        if let Ok(ref mut queue) = self.event_queue.write() {
            queue.push(EmbedderEvent::WindowResize);
        }
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

    fn queue_embedder_events_for_winit_event(&self, _event: winit::event::WindowEvent) {
        // Not expecting any winit events.
    }

    fn new_glwindow(
        &self,
        _events_loop: &winit::event_loop::ActiveEventLoop,
    ) -> Rc<dyn webxr::glwindow::GlWindow> {
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
}

impl WindowMethods for Window {
    fn get_coordinates(&self) -> EmbedderCoordinates {
        let inner_size = self.inner_size.get();
        EmbedderCoordinates {
            viewport: Box2D::from_origin_and_size(Point2D::zero(), inner_size),
            framebuffer: inner_size,
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

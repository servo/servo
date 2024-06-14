/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Definition of Window.
//! Implemented by headless and headed windows.

use euclid::{Length, Scale};
use servo::compositing::windowing::{EmbedderEvent, WindowMethods};
use servo::config::opts;
use servo::embedder_traits::Cursor;
use servo::servo_geometry::DeviceIndependentPixel;
use servo::style_traits::DevicePixel;
use servo::webrender_api::units::{DeviceIntPoint, DeviceIntSize};

use super::events_loop::WakerEvent;

// This should vary by zoom level and maybe actual text size (focused or under cursor)
pub const LINE_HEIGHT: f32 = 38.0;

pub trait WindowPortsMethods: WindowMethods {
    fn get_events(&self) -> Vec<EmbedderEvent>;
    fn id(&self) -> winit::window::WindowId;
    fn hidpi_factor(&self) -> Scale<f32, DeviceIndependentPixel, DevicePixel> {
        self.device_pixel_ratio_override()
            .unwrap_or_else(|| match opts::get().output_file {
                Some(_) => Scale::new(1.0),
                None => self.device_hidpi_factor(),
            })
    }
    fn device_hidpi_factor(&self) -> Scale<f32, DeviceIndependentPixel, DevicePixel>;
    fn device_pixel_ratio_override(
        &self,
    ) -> Option<Scale<f32, DeviceIndependentPixel, DevicePixel>>;
    fn page_height(&self) -> f32;
    fn get_fullscreen(&self) -> bool;
    fn queue_embedder_events_for_winit_event(&self, event: winit::event::WindowEvent);
    fn is_animating(&self) -> bool;
    fn set_title(&self, _title: &str) {}
    fn request_inner_size(&self, size: DeviceIntSize) -> Option<DeviceIntSize>;
    fn set_position(&self, _point: DeviceIntPoint) {}
    fn set_fullscreen(&self, _state: bool) {}
    fn set_cursor(&self, _cursor: Cursor) {}
    fn new_glwindow(
        &self,
        events_loop: &winit::event_loop::EventLoopWindowTarget<WakerEvent>,
    ) -> Box<dyn webxr::glwindow::GlWindow>;
    fn winit_window(&self) -> Option<&winit::window::Window>;
    fn toolbar_height(&self) -> Length<f32, DeviceIndependentPixel>;
    fn set_toolbar_height(&self, height: Length<f32, DeviceIndependentPixel>);
}

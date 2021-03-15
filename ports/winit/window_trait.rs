/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Definition of Window.
//! Implemented by headless and headed windows.

use crate::events_loop::ServoEvent;
use servo::compositing::windowing::{WindowEvent, WindowMethods};
use servo::embedder_traits::Cursor;
use servo::webrender_api::units::{DeviceIntPoint, DeviceIntSize};
use winit;

// This should vary by zoom level and maybe actual text size (focused or under cursor)
pub const LINE_HEIGHT: f32 = 38.0;

pub trait WindowPortsMethods: WindowMethods {
    fn get_events(&self) -> Vec<WindowEvent>;
    fn id(&self) -> winit::window::WindowId;
    fn has_events(&self) -> bool;
    fn page_height(&self) -> f32;
    fn get_fullscreen(&self) -> bool;
    fn winit_event_to_servo_event(&self, event: winit::event::WindowEvent);
    fn is_animating(&self) -> bool;
    fn set_title(&self, _title: &str) {}
    fn set_inner_size(&self, _size: DeviceIntSize) {}
    fn set_position(&self, _point: DeviceIntPoint) {}
    fn set_fullscreen(&self, _state: bool) {}
    fn set_cursor(&self, _cursor: Cursor) {}
    fn new_glwindow(
        &self,
        events_loop: &winit::event_loop::EventLoopWindowTarget<ServoEvent>
    ) -> Box<dyn webxr::glwindow::GlWindow>;
}

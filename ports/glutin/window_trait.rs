/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Definition of a Window.

use glutin;
use servo::compositing::windowing::{WindowEvent, WindowMethods};
use servo::embedder_traits::Cursor;
use servo::webrender_api::{DeviceIntPoint, DeviceIntSize};

// This should vary by zoom level and maybe actual text size (focused or under cursor)
pub const LINE_HEIGHT: f32 = 38.0;

pub trait WindowPortsMethods: WindowMethods {
    fn get_events(&self) -> Vec<WindowEvent>;
    fn id(&self) -> Option<glutin::WindowId>;
    fn has_events(&self) -> bool;
    fn page_height(&self) -> f32;
    fn get_fullscreen(&self) -> bool;
    fn winit_event_to_servo_event(&self, event: glutin::WindowEvent);
    fn is_animating(&self) -> bool;
    fn set_title(&self, _title: &str) {}
    fn set_inner_size(&self, _size: DeviceIntSize) {}
    fn set_position(&self, _point: DeviceIntPoint) {}
    fn set_fullscreen(&self, _state: bool) {}
    fn set_cursor(&self, _cursor: Cursor) {}
}

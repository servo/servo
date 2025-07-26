/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Definition of Window.
//! Implemented by headless and headed windows.

use std::rc::Rc;

use euclid::{Length, Scale};
use servo::servo_geometry::{DeviceIndependentIntRect, DeviceIndependentPixel};
use servo::webrender_api::units::{DeviceIntPoint, DeviceIntSize, DevicePixel};
use servo::{Cursor, RenderingContext, ScreenGeometry, WebView};

use super::app_state::RunningAppState;

// This should vary by zoom level and maybe actual text size (focused or under cursor)
pub(crate) const LINE_HEIGHT: f32 = 76.0;
pub(crate) const LINE_WIDTH: f32 = 76.0;
// MouseScrollDelta::PixelDelta is default for MacOS, which is high precision and very slow
// in winit. Therefore we use a factor of 4.0 to make it more usable.
// See https://github.com/servo/servo/pull/34063#discussion_r2197729507
pub(crate) const PIXEL_DELTA_FACTOR: f64 = 4.0;

/// <https://github.com/web-platform-tests/wpt/blob/9320b1f724632c52929a3fdb11bdaf65eafc7611/webdriver/tests/classic/set_window_rect/set.py#L287-L290>
/// "A window size of 10x10px shouldn't be supported by any browser."
pub(crate) const MIN_INNER_WIDTH: i32 = 20;
pub(crate) const MIN_INNER_HEIGHT: i32 = 20;

pub trait WindowPortsMethods {
    fn id(&self) -> winit::window::WindowId;
    fn screen_geometry(&self) -> ScreenGeometry;
    fn device_hidpi_scale_factor(&self) -> Scale<f32, DeviceIndependentPixel, DevicePixel>;
    fn hidpi_scale_factor(&self) -> Scale<f32, DeviceIndependentPixel, DevicePixel>;
    fn page_height(&self) -> f32;
    fn get_fullscreen(&self) -> bool;
    fn handle_winit_event(&self, state: Rc<RunningAppState>, event: winit::event::WindowEvent);
    fn set_title(&self, _title: &str) {}
    /// Request a new outer size for the window, including external decorations.
    /// This should be the same as `window.outerWidth` and `window.outerHeight``
    fn request_resize(&self, webview: &WebView, outer_size: DeviceIntSize)
    -> Option<DeviceIntSize>;
    fn set_position(&self, _point: DeviceIntPoint) {}
    fn set_fullscreen(&self, _state: bool) {}
    fn set_cursor(&self, _cursor: Cursor) {}
    fn new_glwindow(
        &self,
        event_loop: &winit::event_loop::ActiveEventLoop,
    ) -> Rc<dyn servo::webxr::glwindow::GlWindow>;
    fn winit_window(&self) -> Option<&winit::window::Window>;
    fn toolbar_height(&self) -> Length<f32, DeviceIndependentPixel>;
    fn set_toolbar_height(&self, height: Length<f32, DeviceIndependentPixel>);
    /// This returns [`RenderingContext`] matching the viewport.
    fn rendering_context(&self) -> Rc<dyn RenderingContext>;
    fn show_ime(
        &self,
        _input_type: servo::InputMethodType,
        _text: Option<(String, i32)>,
        _multiline: bool,
        _position: servo::webrender_api::units::DeviceIntRect,
    ) {
    }
    fn hide_ime(&self) {}
    fn theme(&self) -> servo::Theme {
        servo::Theme::Light
    }
    fn window_rect(&self) -> DeviceIndependentIntRect;
    fn maximize(&self, webview: &WebView);
}

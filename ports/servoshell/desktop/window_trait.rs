/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Definition of Window.
//! Implemented by headless and headed windows.

use std::rc::Rc;

use euclid::{Length, Scale};
use servo::servo_geometry::{DeviceIndependentIntRect, DeviceIndependentPixel};
use servo::webrender_api::units::{DeviceIntPoint, DeviceIntSize, DevicePixel};
use servo::{
    Cursor, InputEventId, InputEventResult, InputMethodControl, RenderingContext, ScreenGeometry,
    WebView,
};
use winit::event::WindowEvent;

use super::app_state::RunningAppState;
use crate::desktop::events_loop::AppEvent;

// This should vary by zoom level and maybe actual text size (focused or under cursor)
pub(crate) const LINE_HEIGHT: f32 = 76.0;
pub(crate) const LINE_WIDTH: f32 = 76.0;

/// <https://github.com/web-platform-tests/wpt/blob/9320b1f724632c52929a3fdb11bdaf65eafc7611/webdriver/tests/classic/set_window_rect/set.py#L287-L290>
/// "A window size of 10x10px shouldn't be supported by any browser."
pub(crate) const MIN_WINDOW_INNER_SIZE: DeviceIntSize = DeviceIntSize::new(100, 100);

pub trait WindowPortsMethods {
    fn id(&self) -> winit::window::WindowId;
    fn screen_geometry(&self) -> ScreenGeometry;
    fn device_hidpi_scale_factor(&self) -> Scale<f32, DeviceIndependentPixel, DevicePixel>;
    fn hidpi_scale_factor(&self) -> Scale<f32, DeviceIndependentPixel, DevicePixel>;
    fn get_fullscreen(&self) -> bool;
    /// Request that the `Window` rebuild its user interface, if it has one. This should
    /// not repaint, but should prepare the user interface for painting when it is
    /// actually requested.
    fn rebuild_user_interface(&self, _: &RunningAppState) {}
    /// Inform the `Window` that the state of a `WebView` has changed and that it should
    /// do an incremental update of user interface state. Returns `true` if the user
    /// interface actually changed and a rebuild  and repaint is needed, `false` otherwise.
    fn update_user_interface_state(&self, _: &RunningAppState) -> bool {
        false
    }
    /// Handle a winit [`WindowEvent`]. Returns `true` if the event loop should continue
    /// and `false` otherwise.
    ///
    /// TODO: This should be handled internally in the winit window if possible so that it
    /// makes more sense when we are mixing headed and headless windows.
    fn handle_winit_window_event(&self, _: Rc<RunningAppState>, _: WindowEvent) -> bool {
        false
    }
    /// Handle a winit [`AppEvent`]. Returns `true` if the event loop should continue and
    /// `false` otherwise.
    ///
    /// TODO: This should be handled internally in the winit window if possible so that it
    /// makes more sense when we are mixing headed and headless windows.
    fn handle_winit_app_event(&self, _: Rc<RunningAppState>, _: AppEvent) -> bool {
        false
    }
    /// Request a new outer size for the window, including external decorations.
    /// This should be the same as `window.outerWidth` and `window.outerHeight``
    fn request_resize(&self, webview: &WebView, outer_size: DeviceIntSize)
    -> Option<DeviceIntSize>;
    fn set_position(&self, _point: DeviceIntPoint) {}
    fn set_fullscreen(&self, _state: bool) {}
    fn set_cursor(&self, _cursor: Cursor) {}
    #[cfg(feature = "webxr")]
    fn new_glwindow(
        &self,
        event_loop: &winit::event_loop::ActiveEventLoop,
    ) -> Rc<dyn servo::webxr::glwindow::GlWindow>;
    fn toolbar_height(&self) -> Length<f32, DeviceIndependentPixel>;
    /// This returns [`RenderingContext`] matching the viewport.
    fn rendering_context(&self) -> Rc<dyn RenderingContext>;
    fn show_ime(&self, _input_method: InputMethodControl) {}
    fn hide_ime(&self) {}
    fn theme(&self) -> servo::Theme {
        servo::Theme::Light
    }
    fn window_rect(&self) -> DeviceIndependentIntRect;
    fn maximize(&self, webview: &WebView);

    fn notify_input_event_handled(
        &self,
        _webview: &WebView,
        _id: InputEventId,
        _result: InputEventResult,
    ) {
    }
}

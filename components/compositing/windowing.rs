/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Abstract windowing methods. The concrete implementations of these can be found in `platform/`.

use geom::point::TypedPoint2D;
use geom::scale_factor::ScaleFactor;
use geom::size::TypedSize2D;
use layers::geometry::DevicePixel;
use servo_msg::compositor_msg::{ReadyState, RenderState};
use servo_util::geometry::ScreenPx;

pub enum MouseWindowEvent {
    MouseWindowClickEvent(uint, TypedPoint2D<DevicePixel, f32>),
    MouseWindowMouseDownEvent(uint, TypedPoint2D<DevicePixel, f32>),
    MouseWindowMouseUpEvent(uint, TypedPoint2D<DevicePixel, f32>),
}

pub enum WindowNavigateMsg {
    Forward,
    Back,
}

/// Events that the windowing system sends to Servo.
pub enum WindowEvent {
    /// Sent when no message has arrived.
    ///
    /// FIXME: This is a bogus event and is only used because we don't have the new
    /// scheduler integrated with the platform event loop.
    IdleWindowEvent,
    /// Sent when part of the window is marked dirty and needs to be redrawn.
    RefreshWindowEvent,
    /// Sent when the window is resized.
    ResizeWindowEvent(TypedSize2D<DevicePixel, uint>),
    /// Sent when a new URL is to be loaded.
    LoadUrlWindowEvent(String),
    /// Sent when a mouse hit test is to be performed.
    MouseWindowEventClass(MouseWindowEvent),
    /// Sent when a mouse move.
    MouseWindowMoveEventClass(TypedPoint2D<DevicePixel, f32>),
    /// Sent when the user scrolls. Includes the current cursor position.
    ScrollWindowEvent(TypedPoint2D<DevicePixel, f32>, TypedPoint2D<DevicePixel, i32>),
    /// Sent when the user zooms.
    ZoomWindowEvent(f32),
    /// Simulated "pinch zoom" gesture for non-touch platforms (e.g. ctrl-scrollwheel).
    PinchZoomWindowEvent(f32),
    /// Sent when the user uses chrome navigation (i.e. backspace or shift-backspace).
    NavigationWindowEvent(WindowNavigateMsg),
    /// Sent when rendering is finished.
    FinishedWindowEvent,
    /// Sent when the user quits the application
    QuitWindowEvent,
}

pub trait WindowMethods {
    /// Returns the size of the window in hardware pixels.
    fn framebuffer_size(&self) -> TypedSize2D<DevicePixel, uint>;
    /// Returns the size of the window in density-independent "px" units.
    fn size(&self) -> TypedSize2D<ScreenPx, f32>;
    /// Presents the window to the screen (perhaps by page flipping).
    fn present(&self);

    /// Spins the event loop and returns the next event.
    fn recv(&self) -> WindowEvent;

    /// Sets the ready state of the current page.
    fn set_ready_state(&self, ready_state: ReadyState);
    /// Sets the render state of the current page.
    fn set_render_state(&self, render_state: RenderState);

    /// Returns the hidpi factor of the monitor.
    fn hidpi_factor(&self) -> ScaleFactor<ScreenPx, DevicePixel, f32>;
}


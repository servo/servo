/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Abstract windowing methods. The concrete implementations of these can be found in `platform/`.

use geom::point::Point2D;
use geom::size::Size2D;
use gfx::compositor::RenderState;
use script::compositor_interface::ReadyState;

pub enum MouseWindowEvent {
    MouseWindowClickEvent(uint, Point2D<f32>),
    MouseWindowMouseDownEvent(uint, Point2D<f32>),
    MouseWindowMouseUpEvent(uint, Point2D<f32>),
}

/// Events that the windowing system sends to Servo.
pub enum WindowEvent {
    /// Sent when no message has arrived.
    ///
    /// FIXME(pcwalton): This is a bogus event and is only used because we don't have the new
    /// scheduler integrated with the platform event loop.
    IdleWindowEvent,
    /// Sent when the window is to be redisplayed.
    CompositeWindowEvent,
    /// Sent when the window is resized.
    ResizeWindowEvent(Size2D<uint>),
    /// Sent when a new URL is to be loaded.
    NavigateWindowEvent(~str),
    /// Sent when a mouse event occurs.
    MouseWindowEventClass(MouseWindowEvent),
    /// Sent when the user scrolls.
    ScrollWindowEvent(Point2D<f32>),
    /// Sent when the user zooms.
    ZoomWindowEvent(f32),
    /// Sent when the user quits the application.
    QuitWindowEvent,
}

/// Methods for an abstract Application.
pub trait ApplicationMethods {
    fn new() -> Self;
}

pub trait WindowMethods<A> {
    /// Creates a new window.
    pub fn new(app: &A) -> @mut Self;
    /// Returns the size of the window.
    pub fn size(&self) -> Size2D<f32>;
    /// Presents the window to the screen (perhaps by page flipping).
    pub fn present(&mut self);

    /// Spins the event loop and returns the next event.
    pub fn recv(@mut self) -> WindowEvent;

    /// Schedules a redisplay at the next turn of the event loop.
    pub fn set_needs_display(@mut self);
    /// Sets the ready state of the current page.
    pub fn set_ready_state(@mut self, ready_state: ReadyState);
    /// Sets the render state of the current page.
    pub fn set_render_state(@mut self, render_state: RenderState);
}


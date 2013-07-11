/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Abstract windowing methods. The concrete implementations of these can be found in `platform/`.

use geom::point::Point2D;
use geom::size::Size2D;
use servo_msg::compositor_msg::{ReadyState, RenderState};

pub enum WindowMouseEvent {
    WindowClickEvent(uint, Point2D<f32>),
    WindowMouseDownEvent(uint, Point2D<f32>),
    WindowMouseUpEvent(uint, Point2D<f32>),
}

pub enum WindowNavigateMsg {
    Forward,
    Back,
}

/// Type of the function that is called when the window is resized.
pub type ResizeCallback = @fn(uint, uint);

/// Type of the function that is called when a new URL is to be loaded.
pub type LoadUrlCallback = @fn(&str);

/// Type of the function that is called when a mouse hit test is to be performed.
pub type MouseCallback = @fn(WindowMouseEvent);

/// Type of the function that is called when the user scrolls.
pub type ScrollCallback = @fn(Point2D<f32>);

/// Type of the function that is called when the user zooms.
pub type ZoomCallback = @fn(f32);

/// Type of the function that is called when the user clicks backspace or shift-backspace
pub type NavigationCallback = @fn(WindowNavigateMsg);

/// Type of the function that is called when the rendering is finished
pub type FinishedCallback = @fn();

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

    /// Registers a callback to run when a resize event occurs.
    pub fn set_resize_callback(&mut self, new_resize_callback: ResizeCallback);
    /// Registers a callback to run when a new URL is to be loaded.
    pub fn set_load_url_callback(&mut self, new_load_url_callback: LoadUrlCallback);
    /// Registers a callback to run when the user clicks.
    pub fn set_mouse_callback(&mut self, new_mouse_callback: MouseCallback);
    /// Registers a callback to run when the user scrolls.
    pub fn set_scroll_callback(&mut self, new_scroll_callback: ScrollCallback);
    /// Registers a callback to run when the user zooms.
    pub fn set_zoom_callback(&mut self, new_zoom_callback: ZoomCallback);
    /// Registers a callback to run when the user presses backspace or shift-backspace.
    pub fn set_navigation_callback(&mut self, new_navigation_callback: NavigationCallback);
    /// Registers a callback to run when rendering is finished.
    pub fn set_finished_callback(&mut self, new_finish_callback: FinishedCallback);

    /// Spins the event loop. Returns whether the window should close.
    pub fn check_loop(@mut self) -> bool;
    /// Sets the ready state of the current page.
    pub fn set_ready_state(@mut self, ready_state: ReadyState);
    /// Sets the render state of the current page.
    pub fn set_render_state(@mut self, render_state: RenderState);
}


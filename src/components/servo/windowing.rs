/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Abstract windowing methods. The concrete implementations of these can be found in `platform/`.

use geom::point::Point2D;
use geom::size::Size2D;

/// Type of the function that is called when the screen is to be redisplayed.
pub type CompositeCallback = @fn();

/// Type of the function that is called when the window is resized.
pub type ResizeCallback = @fn(uint, uint);

/// Type of the function that is called when a new URL is to be loaded.
pub type LoadUrlCallback = @fn(&str);

/// Type of the function that is called when the user scrolls.
pub type ScrollCallback = @fn(Point2D<f32>);

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

    /// Registers a callback to run when a composite event occurs.
    pub fn set_composite_callback(&mut self, new_composite_callback: CompositeCallback);
    /// Registers a callback to run when a resize event occurs.
    pub fn set_resize_callback(&mut self, new_resize_callback: ResizeCallback);
    /// Registers a callback to run when a new URL is to be loaded.
    pub fn set_load_url_callback(&mut self, new_load_url_callback: LoadUrlCallback);
    /// Registers a callback to run when the user scrolls.
    pub fn set_scroll_callback(&mut self, new_scroll_callback: ScrollCallback);

    /// Spins the event loop.
    pub fn check_loop(@mut self);
    /// Schedules a redisplay at the next turn of the event loop.
    pub fn set_needs_display(@mut self);
}


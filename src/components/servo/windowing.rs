/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Abstract windowing methods. The concrete implementations of these can be found in `platform/`.

use geom::size::Size2D;

/// Type of the function that is called when the screen is to be redisplayed.
pub type CompositeCallback = @fn();

/// Type of the function that is called when the window is resized.
pub type ResizeCallback = @fn(uint, uint);

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
    /// Spins the event loop.
    pub fn check_loop(@mut self);
}


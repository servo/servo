/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! A windowing implementation using shared OpenGL textures.
///
/// In this setup, Servo renders to an OpenGL texture and uses IPC to share that texture with
/// another application. It also uses IPC to handle events.
///
/// This is designed for sandboxing scenarios which the OpenGL graphics driver is either sandboxed
/// along with the Servo process or trusted. If the OpenGL driver itself is untrusted, then this
/// windowing implementation is not appropriate.

use windowing::{CompositeCallback, ResizeCallback};

use geom::size::Size2D;
use sharegl::base::ShareContext;
use sharegl::platform::Context;

/// A structure responsible for setting up and tearing down the entire windowing system.
pub struct Application;

impl ApplicationMethods for Application {
    pub fn new() -> Application {
        Application
    }
}

/// The type of a window.
pub struct Window(Context);

impl WindowingMethods<Application> for Window {
    /// Creates a new window.
    pub fn new(_: &Application) -> @mut Window {
        let share_context: Context = ShareContext::new(Size2D(800, 600));
        println(fmt!("Sharing ID is %d", share_context.id()));
        @mut Window(share_context)
    }

    /// Returns the size of the window.
    pub fn size(&mut self) -> Size2D<f32> {
        Size2D(800.0, 600.0)
    }

    /// Presents the window to the screen (perhaps by page flipping).
    pub fn present(&mut self) {
        (*self).flush();
    }

    /// Registers a callback to run when a composite event occurs.
    pub fn set_composite_callback(&mut self, _: CompositeCallback) {}

    /// Registers a callback to run when a resize event occurs.
    pub fn set_resize_callback(&mut self, _: ResizeCallback) {}

    /// Returns the next event.
    pub fn check_loop(@mut self) {}
}


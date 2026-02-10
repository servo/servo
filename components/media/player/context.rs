// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

//! `PlayerGLContext` is a trait to be used to pass the GL context for
//! rendering purposes.
//!
//! The current consumer of this trait is the GL rendering mechanism
//! for the GStreamer backend.
//!
//! The client application should implement this trait and pass the
//! trait object to its `player` instance.

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum GlContext {
    /// The EGL platform used primarily with the X11, Wayland and
    /// Android window systems as well as on embedded Linux.
    Egl(usize),
    /// The GLX platform used primarily with the X11 window system.
    Glx(usize),
    Unknown,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum NativeDisplay {
    /// The EGLDisplay memory address
    Egl(usize),
    /// XDisplay memory address
    X11(usize),
    /// wl_display memory address
    Wayland(usize),
    Headless,
    Unknown,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum GlApi {
    OpenGL,
    OpenGL3,
    Gles1,
    Gles2,
    None,
}

pub trait PlayerGLContext {
    /// Returns the GL context living pointer wrapped by `GlContext`
    fn get_gl_context(&self) -> GlContext;
    /// Returns the living pointer to the native display structure
    /// wrapped by `NativeDisplay`.
    fn get_native_display(&self) -> NativeDisplay;
    /// Returns the GL API of the context
    fn get_gl_api(&self) -> GlApi;
}

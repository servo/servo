/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
#![allow(non_camel_case_types)]
#![allow(unused_imports)]

pub type ServoGl = std::rc::Rc<dyn servo::gl::Gl>;

use std::ffi::CString;
use std::os::raw::c_void;

use log::info;
use servo::gl::GlesFns;

pub type EGLNativeWindowType = *const libc::c_void;
pub type khronos_utime_nanoseconds_t = khronos_uint64_t;
pub type khronos_uint64_t = u64;
pub type khronos_ssize_t = libc::c_long;
pub type EGLint = i32;
pub type EGLContext = *const libc::c_void;
pub type EGLNativeDisplayType = *const libc::c_void;
pub type EGLNativePixmapType = *const libc::c_void;
pub type NativeDisplayType = EGLNativeDisplayType;
pub type NativePixmapType = EGLNativePixmapType;
pub type NativeWindowType = EGLNativeWindowType;

include!(concat!(env!("OUT_DIR"), "/egl_bindings.rs"));

pub struct EGLInitResult {
    pub gl_wrapper: ServoGl,
    pub gl_context: EGLContext,
    pub display: EGLNativeDisplayType,
}

pub fn init() -> Result<EGLInitResult, &'static str> {
    info!("Loading EGL...");
    unsafe {
        let egl = Egl;
        let display = egl.GetCurrentDisplay();
        egl.SwapInterval(display, 1);
        let egl = GlesFns::load_with(|addr| {
            let addr = CString::new(addr.as_bytes()).unwrap();
            let addr = addr.as_ptr();
            let egl = Egl;
            egl.GetProcAddress(addr) as *const c_void
        });
        info!("EGL loaded");
        Ok(EGLInitResult {
            gl_wrapper: egl,
            gl_context: Egl.GetCurrentContext(),
            display,
        })
    }
}

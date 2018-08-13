/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#[cfg(any(target_os = "android", target_os = "windows"))]
#[allow(non_camel_case_types)]
pub mod egl {
    use libc;
    use servo::gl::{Gl, GlesFns};
    use std::ffi::CString;
    #[cfg(not(target_os = "windows"))]
    use std::os::raw::c_void;
    use std::rc::Rc;
    #[cfg(target_os = "windows")]
    use winapi;
    #[cfg(target_os = "windows")]
    use winapi::um::libloaderapi::{GetProcAddress, LoadLibraryA};

    #[cfg(target_os = "windows")]
    pub type EGLNativeWindowType = winapi::shared::windef::HWND;
    #[cfg(target_os = "linux")]
    pub type EGLNativeWindowType = *const libc::c_void;
    #[cfg(target_os = "android")]
    pub type EGLNativeWindowType = *const libc::c_void;
    #[cfg(any(target_os = "dragonfly", target_os = "freebsd", target_os = "openbsd"))]
    pub type EGLNativeWindowType = *const libc::c_void;

    pub type khronos_utime_nanoseconds_t = khronos_uint64_t;
    pub type khronos_uint64_t = libc::uint64_t;
    pub type khronos_ssize_t = libc::c_long;
    pub type EGLint = libc::int32_t;
    pub type EGLNativeDisplayType = *const libc::c_void;
    pub type EGLNativePixmapType = *const libc::c_void;
    pub type NativeDisplayType = EGLNativeDisplayType;
    pub type NativePixmapType = EGLNativePixmapType;
    pub type NativeWindowType = EGLNativeWindowType;

    include!(concat!(env!("OUT_DIR"), "/egl_bindings.rs"));

    #[cfg(target_os = "android")]
    pub fn init() -> Result<Rc<Gl>, &'static str> {
        info!("Loading EGL…");
        unsafe {
            let egl = Egl;
            let d = egl.GetCurrentDisplay();
            egl.SwapInterval(d, 1);
            let egl = GlesFns::load_with(|addr| {
                let addr = CString::new(addr.as_bytes()).unwrap();
                let addr = addr.as_ptr();
                let egl = Egl;
                egl.GetProcAddress(addr) as *const c_void
            });
            info!("EGL loaded");
            Ok(egl)
        }
    }

    #[cfg(target_os = "windows")]
    pub fn init() -> Result<Rc<Gl>, &'static str> {
        info!("Loading EGL…");

        let dll = b"libEGL.dll\0" as &[u8];
        let dll = unsafe { LoadLibraryA(dll.as_ptr() as *const _) };
        if dll.is_null() {
            Err("Can't find libEGL.dll")
        } else {
            unsafe {
                let egl = GlesFns::load_with(|addr| {
                    let addr = CString::new(addr.as_bytes()).unwrap();
                    let addr = addr.as_ptr();
                    GetProcAddress(dll, addr) as *const _
                });
                info!("EGL loaded");
                Ok(egl)
            }
        }
    }
}

#[cfg(any(target_os = "windows", target_os = "linux", target_os = "macos"))]
pub mod gl {
    use servo::gl::Gl;
    use std::rc::Rc;
    pub fn init() -> Result<Rc<Gl>, &'static str> {
        // FIXME: Add an OpenGL version
        unimplemented!()
    }
}

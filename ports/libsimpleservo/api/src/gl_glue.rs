/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use servo::gl::GlType::Gl;
use std::rc::Rc;

pub type ServoGl = Rc<dyn servo::gl::Gl>;

#[cfg(any(target_os = "android", target_os = "windows"))]
#[allow(non_camel_case_types)]
pub mod egl {
    use servo::gl::GlesFns;
    use std::ffi::CString;
    #[cfg(not(target_os = "windows"))]
    use std::os::raw::c_void;
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
    pub fn init() -> Result<crate::gl_glue::ServoGl, &'static str> {
        info!("Loading EGL...");
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
    use winapi::shared::windef::HGLRC;
    use winapi::um::libloaderapi::*;

    pub fn init() -> Result<Rc<Gl>, &'static str> {
        info!("Loading EGL...");

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

//#[cfg(any(target_os = "windows", target_os = "linux", target_os = "macos"))]
//pub mod gl {
//   pub fn init() -> Result<crate::gl_glue::ServoGl, &'static str> {
// FIXME: Add an OpenGL version
//unimplemented!()
//   }
//}

#[cfg(target_os = "macos")]
pub mod gl {
    use core_foundation::base::TCFType;
    use core_foundation::bundle::{
        CFBundleGetBundleWithIdentifier, CFBundleGetFunctionPointerForName,
    };
    use core_foundation::string::CFString;
    use servo::gl::GlType::Gl;
    use std::os::raw::c_void;
    use std::rc::Rc;
    use std::str;

    #[cfg(target_os = "macos")]
    pub fn init() -> Result<Rc<Gl>, &'static str> {
        info!("Loading OpenGL...");
        let opengl_pointer;
        opengl_pointer = unsafe {
            gl::GlFns::load_with(|addr| {
                let symbol_name: CFString = str::FromStr::from_str(addr).unwrap();
                let framework_name: CFString = str::FromStr::from_str("com.apple.opengl").unwrap();
                let framework =
                    CFBundleGetBundleWithIdentifier(framework_name.as_concrete_TypeRef());
                let symbol =
                    CFBundleGetFunctionPointerForName(framework, symbol_name.as_concrete_TypeRef());
                symbol as *const c_void
            })
        };
        if opengl_pointer.is_null() {
            Err("OpenGL isn't configured/installed")
        }
        info!("OpenGL loaded");
        Ok(opengl_pointer)
    }
}

#[cfg(target_os = "windows")]
pub mod gl {
    use servo::gl::GlType::Gl;
    use std::ffi::CString;
    use std::rc::Rc;
    use winapi::um::libloaderapi::{GetProcAddress, LoadLibraryA};

    pub fn init() -> Result<Rc<Gl>, &'static str> {
        info!("Loading OpenGL...");

        let opengl_pointer;
        let dll = b"OpenGL32.dll\0" as &[u8];
        let dll = unsafe { LoadLibraryA(dll.as_ptr() as *const _) };
        if dll.is_null {
            Err("Can't find Opengl32.dll, OpenGL isn't configured/installed")
        } else {
            unsafe {
                opengl_pointer = gl::GlFns::load_with(|addr| {
                    let addr = CString::new(addr.as_bytes()).unwrap();
                    let addr = addr.as_ptr();
                    GetProcAddress(dll, addr) as *const _
                });
            }
        }
        info!("OpenGL loaded");
        Ok(opengl_pointer)
    }
}

#[cfg(any(
    target_os = "dragonfly",
    target_os = "linux",
    target_os = "freebsd",
    target_os = "openbsd"
))]

pub mod gl {
    use api::dlopen;
    use servo::gl::GlType::Gl;
    use std::ffi::CString;
    use std::rc::Rc;

    pub fn init() -> Result<Rc<Gl>, &'static str> {
        info!("Loading OpenGL");
        //Glx bindings
        pub mod glx {
            include!(concat!(env!("OUT_DIR"), "/glx_bindings.rs"));
        }

        let mut libglx =
            unsafe { dlopen::dlopen(b"libGL.so\0".as_ptr() as *const _, dlopen::RTLD_NOW) };
        if libglx.is_null() {
            Err("Can't find libGL.so, OpenGL isn't configured/installed")
        }
        //initializing glx instance
        let glx_instance = Glx::load_with(|sym| {
            let sym = CString::new(sym).unwrap();
            unsafe { dlopen::dlsym(libglx, sym.as_ptr()) }
        });
        //finding the opengl address loaded in memory and returning the pointer to memory address
        let opengl_instance = gl::GlFns::load_with(|addr| glx_instance.GetProcAddress(addr));
        info!("OpenGL is loaded");
        Ok(opengl_instance)
    }
}

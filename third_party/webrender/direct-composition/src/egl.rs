/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use mozangle::egl::ffi::*;
use std::os::raw::c_void;
use std::ptr;
use std::rc::Rc;
use winapi::um::d3d11::ID3D11Device;
use winapi::um::d3d11::ID3D11Texture2D;

pub use mozangle::egl::get_proc_address;

pub struct SharedEglThings {
    device: EGLDeviceEXT,
    display: types::EGLDisplay,
    config: types::EGLConfig,
    context: types::EGLContext,
}

fn cast_attributes(slice: &[types::EGLenum]) -> &EGLint {
    unsafe {
        &*(slice.as_ptr() as *const EGLint)
    }
}

macro_rules! attributes {
    ($( $key: expr => $value: expr, )*) => {
        cast_attributes(&[
            $( $key, $value, )*
            NONE,
        ])
    }
}

impl SharedEglThings {
    pub unsafe fn new(d3d_device: *mut ID3D11Device) -> Rc<Self> {
        let device = eglCreateDeviceANGLE(
            D3D11_DEVICE_ANGLE,
            d3d_device as *mut c_void,
            ptr::null(),
        ).check();
        let display = GetPlatformDisplayEXT(
            PLATFORM_DEVICE_EXT,
            device,
            attributes! [
                EXPERIMENTAL_PRESENT_PATH_ANGLE => EXPERIMENTAL_PRESENT_PATH_FAST_ANGLE,
            ],
        ).check();
        Initialize(display, ptr::null_mut(), ptr::null_mut()).check();

        // Adapted from
        // https://searchfox.org/mozilla-central/rev/056a4057/gfx/gl/GLContextProviderEGL.cpp#635
        let mut configs = [ptr::null(); 64];
        let mut num_configs = 0;
        ChooseConfig(
            display,
            attributes! [
                SURFACE_TYPE => WINDOW_BIT,
                RENDERABLE_TYPE => OPENGL_ES2_BIT,
                RED_SIZE => 8,
                GREEN_SIZE => 8,
                BLUE_SIZE => 8,
                ALPHA_SIZE => 8,
            ],
            configs.as_mut_ptr(),
            configs.len() as i32,
            &mut num_configs,
        ).check();
        let config = pick_config(&configs[..num_configs as usize]);

        let context = CreateContext(
            display, config, NO_CONTEXT,
            attributes![
                 CONTEXT_CLIENT_VERSION => 3,
            ]
        ).check();
        MakeCurrent(display, NO_SURFACE, NO_SURFACE, context).check();

        Rc::new(SharedEglThings { device, display, config, context })
    }
}

fn pick_config(configs: &[types::EGLConfig]) -> types::EGLConfig {
    // FIXME: better criteria to make this choice?
    // Firefox uses GetConfigAttrib to find a config that has the requested r/g/b/a sizes
    // https://searchfox.org/mozilla-central/rev/056a4057/gfx/gl/GLContextProviderEGL.cpp#662-685

    configs[0]
}

impl Drop for SharedEglThings {
    fn drop(&mut self) {
        unsafe {
            // FIXME does EGLDisplay or EGLConfig need clean up? How?
            DestroyContext(self.display, self.context).check();
            eglReleaseDeviceANGLE(self.device).check();
        }
    }
}

pub struct PerVisualEglThings {
    shared: Rc<SharedEglThings>,
    surface: types::EGLSurface,
}

impl PerVisualEglThings {
    pub unsafe fn new(shared: Rc<SharedEglThings>, buffer: *const ID3D11Texture2D,
           width: u32, height: u32)
           -> Self {
        let surface = CreatePbufferFromClientBuffer(
            shared.display,
            D3D_TEXTURE_ANGLE,
            buffer as types::EGLClientBuffer,
            shared.config,
            attributes! [
                WIDTH => width,
                HEIGHT => height,
                FLEXIBLE_SURFACE_COMPATIBILITY_SUPPORTED_ANGLE => TRUE,
            ],
        ).check();

        PerVisualEglThings { shared, surface }
    }

    pub fn make_current(&self) {
        unsafe {
            MakeCurrent(self.shared.display, self.surface, self.surface, self.shared.context).check();
        }
    }
}

impl Drop for PerVisualEglThings {
    fn drop(&mut self) {
        unsafe {
            DestroySurface(self.shared.display, self.surface).check();
        }
    }
}

fn check_error() {
    unsafe {
        let error = GetError() as types::EGLenum;
        assert_eq!(error, SUCCESS, "0x{:x} != 0x{:x}", error, SUCCESS);
    }
}

trait Check {
    fn check(self) -> Self;
}

impl Check for *const c_void {
    fn check(self) -> Self {
        check_error();
        assert!(!self.is_null());
        self
    }
}

impl Check for *mut c_void {
    fn check(self) -> Self {
        check_error();
        assert!(!self.is_null());
        self
    }
}

impl Check for types::EGLBoolean {
    fn check(self) -> Self {
        check_error();
        assert_eq!(self, TRUE);
        self
    }
}

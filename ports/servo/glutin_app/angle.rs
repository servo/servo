/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use glutin::{
    Api, ContextError, GlAttributes, GlContext, PixelFormatRequirements
};
#[cfg(target_os = "windows")]
use glutin::Context as GlutinContext;
#[cfg(target_os = "windows")]
use crate::platform::windows::egl::EglContext;
use winit::Window;
#[cfg(target_os = "windows")]
use winit::os::windows::WindowExt;

#[cfg(target_os = "windows")]
pub struct Context {
    context: EglContext,
}

#[cfg(target_os = "windows")]
pub fn create_context(
    pf_reqs: &PixelFormatRequirements,
    gl_attr: &GlAttributes<&GlutinContext>,
    window: &Window,
) -> Context {
    let context = EglContext::new(pf_reqs, gl_attr)
        .and_then(|p| p.finish(window.get_hwnd() as _))
        .expect("Couldn't create ANGLE context");
    Context { context }
}

#[cfg(target_os = "windows")]
impl GlContext for Context {
    unsafe fn make_current(&self) -> Result<(), ContextError> {
        self.context.make_current()
    }

    fn is_current(&self) -> bool {
        self.context.is_current()
    }

    fn get_proc_address(&self, addr: &str) -> *const () {
        self.context.get_proc_address(addr)
    }

    fn get_api(&self) -> Api {
        self.context.get_api()
    }
}

#[cfg(target_os = "windows")]
impl Context {
    pub fn swap_buffers(&self) -> Result<(), ContextError> {
        self.context.swap_buffers()
    }
}

#[cfg(not(target_os = "windows"))]
pub struct Context;

#[cfg(not(target_os = "windows"))]
impl GlContext for Context {
    unsafe fn make_current(&self) -> Result<(), ContextError> {
        unimplemented!()
    }

    fn is_current(&self) -> bool {
        unimplemented!()
    }

    fn get_proc_address(&self, _addr: &str) -> *const () {
        unimplemented!()
    }

    fn get_api(&self) -> Api {
        unimplemented!()
    }
}

#[cfg(not(target_os = "windows"))]
impl Context {
    pub fn swap_buffers(&self) -> Result<(), ContextError> {
        unimplemented!()
    }
}

#[cfg(not(target_os = "windows"))]
pub fn create_context(
    _pf_reqs: &PixelFormatRequirements,
    _gl_attr: &GlAttributes<&Context>,
    _window: &Window,
) -> Context {
    unimplemented!("ANGLE is only supported on Windows platforms")
}

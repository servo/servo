// Licensed under the Apache License, Version 2.0.
// This file may not be copied, modified, or distributed except according to those terms.

//! Based on https://github.com/tomaka/glutin/blob/1b2d62c0e9/src/api/egl/mod.rs
#![cfg(windows)]
#![allow(unused_variables)]

use glutin::ContextError;
use glutin::CreationError;
use glutin::GlAttributes;
use glutin::GlRequest;
use glutin::PixelFormat;
use glutin::PixelFormatRequirements;
use glutin::ReleaseBehavior;
use glutin::Robustness;
use glutin::Api;

use std::ffi::{CStr, CString};
use std::os::raw::c_int;
use std::ptr;
use std::cell::Cell;

use mozangle::egl::ffi as egl;
mod ffi {
    pub use mozangle::egl::ffi as egl;
    pub use mozangle::egl::ffi::*;
}

pub struct Context {
    display: ffi::egl::types::EGLDisplay,
    context: ffi::egl::types::EGLContext,
    surface: Cell<ffi::egl::types::EGLSurface>,
    api: Api,
    pixel_format: PixelFormat,
}

impl Context {
    /// Start building an EGL context.
    ///
    /// This function initializes some things and chooses the pixel format.
    ///
    /// To finish the process, you must call `.finish(window)` on the `ContextPrototype`.
    pub fn new<'a>(
        pf_reqs: &PixelFormatRequirements,
        opengl: &'a GlAttributes<&'a Context>,
    ) -> Result<ContextPrototype<'a>, CreationError>
    {
        if opengl.sharing.is_some() {
            unimplemented!()
        }

        // calling `eglGetDisplay` or equivalent
        let display = unsafe { egl::GetDisplay(ptr::null_mut()) };

        if display.is_null() {
            return Err(CreationError::PlatformSpecific("Could not create EGL display object".to_string()));
        }

        let egl_version = unsafe {
            let mut major: ffi::egl::types::EGLint = 0; // out param
            let mut minor: ffi::egl::types::EGLint = 0; // out param

            if egl::Initialize(display, &mut major, &mut minor) == 0 {
                return Err(CreationError::OsError(format!("eglInitialize failed")))
            }

            (major, minor)
        };

        // the list of extensions supported by the client once initialized is different from the
        // list of extensions obtained earlier
        let extensions = if egl_version >= (1, 2) {
            let p = unsafe { CStr::from_ptr(egl::QueryString(display, ffi::egl::EXTENSIONS as i32)) };
            let list = String::from_utf8(p.to_bytes().to_vec()).unwrap_or_else(|_| format!(""));
            list.split(' ').map(|e| e.to_string()).collect::<Vec<_>>()

        } else {
            vec![]
        };

        // binding the right API and choosing the version
        let (version, api) = unsafe {
            match opengl.version {
                GlRequest::Latest => {
                    if egl_version >= (1, 4) {
                        if egl::BindAPI(ffi::egl::OPENGL_API) != 0 {
                            (None, Api::OpenGl)
                        } else if egl::BindAPI(ffi::egl::OPENGL_ES_API) != 0 {
                            (None, Api::OpenGlEs)
                        } else {
                            return Err(CreationError::OpenGlVersionNotSupported);
                        }
                    } else {
                        (None, Api::OpenGlEs)
                    }
                },
                GlRequest::Specific(Api::OpenGlEs, version) => {
                    if egl_version >= (1, 2) {
                        if egl::BindAPI(ffi::egl::OPENGL_ES_API) == 0 {
                            return Err(CreationError::OpenGlVersionNotSupported);
                        }
                    }
                    (Some(version), Api::OpenGlEs)
                },
                GlRequest::Specific(Api::OpenGl, version) => {
                    if egl_version < (1, 4) {
                        return Err(CreationError::OpenGlVersionNotSupported);
                    }
                    if egl::BindAPI(ffi::egl::OPENGL_API) == 0 {
                        return Err(CreationError::OpenGlVersionNotSupported);
                    }
                    (Some(version), Api::OpenGl)
                },
                GlRequest::Specific(_, _) => return Err(CreationError::OpenGlVersionNotSupported),
                GlRequest::GlThenGles { opengles_version, opengl_version } => {
                    if egl_version >= (1, 4) {
                        if egl::BindAPI(ffi::egl::OPENGL_API) != 0 {
                            (Some(opengl_version), Api::OpenGl)
                        } else if egl::BindAPI(ffi::egl::OPENGL_ES_API) != 0 {
                            (Some(opengles_version), Api::OpenGlEs)
                        } else {
                            return Err(CreationError::OpenGlVersionNotSupported);
                        }
                    } else {
                        (Some(opengles_version), Api::OpenGlEs)
                    }
                },
            }
        };

        let (config_id, pixel_format) = unsafe {
            choose_fbconfig(display, &egl_version, api, version, pf_reqs)?
        };

        Ok(ContextPrototype {
            opengl: opengl,
            display: display,
            egl_version: egl_version,
            extensions: extensions,
            api: api,
            version: version,
            config_id: config_id,
            pixel_format: pixel_format,
        })
    }

    #[inline]
    pub fn swap_buffers(&self) -> Result<(), ContextError> {
        if self.surface.get() == ffi::egl::NO_SURFACE {
            return Err(ContextError::ContextLost);
        }

        let ret = unsafe {
            egl::SwapBuffers(self.display, self.surface.get())
        };

        if ret == 0 {
            match unsafe { egl::GetError() } as u32 {
                ffi::egl::CONTEXT_LOST => return Err(ContextError::ContextLost),
                err => panic!("eglSwapBuffers failed (eglGetError returned 0x{:x})", err)
            }

        } else {
            Ok(())
        }
    }

    pub unsafe fn make_current(&self) -> Result<(), ContextError> {
        let ret = egl::MakeCurrent(self.display, self.surface.get(), self.surface.get(), self.context);

        if ret == 0 {
            match egl::GetError() as u32 {
                ffi::egl::CONTEXT_LOST => return Err(ContextError::ContextLost),
                err => panic!("eglMakeCurrent failed (eglGetError returned 0x{:x})", err)
            }

        } else {
            Ok(())
        }
    }

    #[inline]
    pub fn is_current(&self) -> bool {
        unsafe { egl::GetCurrentContext() == self.context }
    }

    pub fn get_proc_address(&self, addr: &str) -> *const () {
        let addr = CString::new(addr.as_bytes()).unwrap();
        let addr = addr.as_ptr();
        unsafe {
            egl::GetProcAddress(addr) as *const _
        }
    }

    #[inline]
    pub fn get_api(&self) -> Api {
        self.api
    }

    #[inline]
    pub fn get_pixel_format(&self) -> PixelFormat {
        self.pixel_format.clone()
    }
}

unsafe impl Send for Context {}
unsafe impl Sync for Context {}

impl Drop for Context {
    fn drop(&mut self) {
        unsafe {
            // we don't call MakeCurrent(0, 0) because we are not sure that the context
            // is still the current one
            egl::DestroyContext(self.display, self.context);
            egl::DestroySurface(self.display, self.surface.get());
            egl::Terminate(self.display);
        }
    }
}

pub struct ContextPrototype<'a> {
    opengl: &'a GlAttributes<&'a Context>,
    display: ffi::egl::types::EGLDisplay,
    egl_version: (ffi::egl::types::EGLint, ffi::egl::types::EGLint),
    extensions: Vec<String>,
    api: Api,
    version: Option<(u8, u8)>,
    config_id: ffi::egl::types::EGLConfig,
    pixel_format: PixelFormat,
}

impl<'a> ContextPrototype<'a> {
    pub fn get_native_visual_id(&self) -> ffi::egl::types::EGLint {
        let mut value = 0;
        let ret = unsafe { egl::GetConfigAttrib(self.display, self.config_id,
                                                    ffi::egl::NATIVE_VISUAL_ID
                                                    as ffi::egl::types::EGLint, &mut value) };
        if ret == 0 { panic!("eglGetConfigAttrib failed") };
        value
    }

    pub fn finish(self, native_window: ffi::EGLNativeWindowType)
                  -> Result<Context, CreationError>
    {
        let surface = unsafe {
            let surface = egl::CreateWindowSurface(self.display, self.config_id, native_window,
                                                       ptr::null());
            if surface.is_null() {
                return Err(CreationError::OsError(format!("eglCreateWindowSurface failed")))
            }
            surface
        };

        self.finish_impl(surface)
    }

    pub fn finish_pbuffer(self, dimensions: (u32, u32)) -> Result<Context, CreationError> {
        let attrs = &[
            ffi::egl::WIDTH as c_int, dimensions.0 as c_int,
            ffi::egl::HEIGHT as c_int, dimensions.1 as c_int,
            ffi::egl::NONE as c_int,
        ];

        let surface = unsafe {
            let surface = egl::CreatePbufferSurface(self.display, self.config_id,
                                                        attrs.as_ptr());
            if surface.is_null() {
                return Err(CreationError::OsError(format!("eglCreatePbufferSurface failed")))
            }
            surface
        };

        self.finish_impl(surface)
    }

    fn finish_impl(self, surface: ffi::egl::types::EGLSurface)
                   -> Result<Context, CreationError>
    {
        let context = unsafe {
            if let Some(version) = self.version {
                create_context(self.display, &self.egl_version,
                               &self.extensions, self.api, version, self.config_id,
                               self.opengl.debug, self.opengl.robustness)?

            } else if self.api == Api::OpenGlEs {
                if let Ok(ctxt) = create_context(self.display, &self.egl_version,
                                                 &self.extensions, self.api, (2, 0), self.config_id,
                                                 self.opengl.debug, self.opengl.robustness)
                {
                    ctxt
                } else if let Ok(ctxt) = create_context(self.display, &self.egl_version,
                                                        &self.extensions, self.api, (1, 0),
                                                        self.config_id, self.opengl.debug,
                                                        self.opengl.robustness)
                {
                    ctxt
                } else {
                    return Err(CreationError::OpenGlVersionNotSupported);
                }

            } else {
                if let Ok(ctxt) = create_context(self.display, &self.egl_version,
                                                 &self.extensions, self.api, (3, 2), self.config_id,
                                                 self.opengl.debug, self.opengl.robustness)
                {
                    ctxt
                } else if let Ok(ctxt) = create_context(self.display, &self.egl_version,
                                                        &self.extensions, self.api, (3, 1),
                                                        self.config_id, self.opengl.debug,
                                                        self.opengl.robustness)
                {
                    ctxt
                } else if let Ok(ctxt) = create_context(self.display, &self.egl_version,
                                                        &self.extensions, self.api, (1, 0),
                                                        self.config_id, self.opengl.debug,
                                                        self.opengl.robustness)
                {
                    ctxt
                } else {
                    return Err(CreationError::OpenGlVersionNotSupported);
                }
            }
        };

        Ok(Context {
            display: self.display,
            context: context,
            surface: Cell::new(surface),
            api: self.api,
            pixel_format: self.pixel_format,
        })
    }
}

unsafe fn choose_fbconfig(display: ffi::egl::types::EGLDisplay,
                          egl_version: &(ffi::egl::types::EGLint, ffi::egl::types::EGLint),
                          api: Api, version: Option<(u8, u8)>, reqs: &PixelFormatRequirements)
                          -> Result<(ffi::egl::types::EGLConfig, PixelFormat), CreationError>
{
    let descriptor = {
        let mut out: Vec<c_int> = Vec::with_capacity(37);

        if egl_version >= &(1, 2) {
            out.push(ffi::egl::COLOR_BUFFER_TYPE as c_int);
            out.push(ffi::egl::RGB_BUFFER as c_int);
        }

        out.push(ffi::egl::SURFACE_TYPE as c_int);
        // TODO: Some versions of Mesa report a BAD_ATTRIBUTE error
        // if we ask for PBUFFER_BIT as well as WINDOW_BIT
        out.push((ffi::egl::WINDOW_BIT) as c_int);

        match (api, version) {
            (Api::OpenGlEs, Some((3, _))) => {
                if egl_version < &(1, 3) { return Err(CreationError::NoAvailablePixelFormat); }
                out.push(ffi::egl::RENDERABLE_TYPE as c_int);
                out.push(ffi::egl::OPENGL_ES3_BIT as c_int);
                out.push(ffi::egl::CONFORMANT as c_int);
                out.push(ffi::egl::OPENGL_ES3_BIT as c_int);
            },
            (Api::OpenGlEs, Some((2, _))) => {
                if egl_version < &(1, 3) { return Err(CreationError::NoAvailablePixelFormat); }
                out.push(ffi::egl::RENDERABLE_TYPE as c_int);
                out.push(ffi::egl::OPENGL_ES2_BIT as c_int);
                out.push(ffi::egl::CONFORMANT as c_int);
                out.push(ffi::egl::OPENGL_ES2_BIT as c_int);
            },
            (Api::OpenGlEs, Some((1, _))) => {
                if egl_version >= &(1, 3) {
                    out.push(ffi::egl::RENDERABLE_TYPE as c_int);
                    out.push(ffi::egl::OPENGL_ES_BIT as c_int);
                    out.push(ffi::egl::CONFORMANT as c_int);
                    out.push(ffi::egl::OPENGL_ES_BIT as c_int);
                }
            },
            (Api::OpenGlEs, _) => unimplemented!(),
            (Api::OpenGl, _) => {
                if egl_version < &(1, 3) { return Err(CreationError::NoAvailablePixelFormat); }
                out.push(ffi::egl::RENDERABLE_TYPE as c_int);
                out.push(ffi::egl::OPENGL_BIT as c_int);
                out.push(ffi::egl::CONFORMANT as c_int);
                out.push(ffi::egl::OPENGL_BIT as c_int);
            },
            (_, _) => unimplemented!(),
        };

        if let Some(hardware_accelerated) = reqs.hardware_accelerated {
            out.push(ffi::egl::CONFIG_CAVEAT as c_int);
            out.push(if hardware_accelerated {
                ffi::egl::NONE as c_int
            } else {
                ffi::egl::SLOW_CONFIG as c_int
            });
        }

        if let Some(color) = reqs.color_bits {
            out.push(ffi::egl::RED_SIZE as c_int);
            out.push((color / 3) as c_int);
            out.push(ffi::egl::GREEN_SIZE as c_int);
            out.push((color / 3 + if color % 3 != 0 { 1 } else { 0 }) as c_int);
            out.push(ffi::egl::BLUE_SIZE as c_int);
            out.push((color / 3 + if color % 3 == 2 { 1 } else { 0 }) as c_int);
        }

        if let Some(alpha) = reqs.alpha_bits {
            out.push(ffi::egl::ALPHA_SIZE as c_int);
            out.push(alpha as c_int);
        }

        if let Some(depth) = reqs.depth_bits {
            out.push(ffi::egl::DEPTH_SIZE as c_int);
            out.push(depth as c_int);
        }

        if let Some(stencil) = reqs.stencil_bits {
            out.push(ffi::egl::STENCIL_SIZE as c_int);
            out.push(stencil as c_int);
        }

        if let Some(true) = reqs.double_buffer {
            return Err(CreationError::NoAvailablePixelFormat);
        }

        if let Some(multisampling) = reqs.multisampling {
            out.push(ffi::egl::SAMPLES as c_int);
            out.push(multisampling as c_int);
        }

        if reqs.stereoscopy {
            return Err(CreationError::NoAvailablePixelFormat);
        }

        // FIXME: srgb is not taken into account

        match reqs.release_behavior {
            ReleaseBehavior::Flush => (),
            ReleaseBehavior::None => {
                // TODO: with EGL you need to manually set the behavior
                unimplemented!()
            },
        }

        out.push(ffi::egl::NONE as c_int);
        out
    };

    // calling `eglChooseConfig`
    let mut config_id = ptr::null(); // out param
    let mut num_configs = 0;         // out param
    if egl::ChooseConfig(display, descriptor.as_ptr(), &mut config_id, 1, &mut num_configs) == 0 {
        return Err(CreationError::OsError(format!("eglChooseConfig failed")));
    }
    if num_configs == 0 {
        return Err(CreationError::NoAvailablePixelFormat);
    }

    // analyzing each config
    macro_rules! attrib {
        ($display:expr, $config:expr, $attr:expr) => (
            {
                let mut value = 0; // out param
                let res = egl::GetConfigAttrib($display, $config,
                                               $attr as ffi::egl::types::EGLint, &mut value);
                if res == 0 {
                    return Err(CreationError::OsError(format!("eglGetConfigAttrib failed")));
                }
                value
            }
        )
    }

    let desc = PixelFormat {
        hardware_accelerated: attrib!(display, config_id, ffi::egl::CONFIG_CAVEAT)
                                      != ffi::egl::SLOW_CONFIG as i32,
        color_bits: attrib!(display, config_id, ffi::egl::RED_SIZE) as u8 +
                    attrib!(display, config_id, ffi::egl::BLUE_SIZE) as u8 +
                    attrib!(display, config_id, ffi::egl::GREEN_SIZE) as u8,
        alpha_bits: attrib!(display, config_id, ffi::egl::ALPHA_SIZE) as u8,
        depth_bits: attrib!(display, config_id, ffi::egl::DEPTH_SIZE) as u8,
        stencil_bits: attrib!(display, config_id, ffi::egl::STENCIL_SIZE) as u8,
        stereoscopy: false,
        double_buffer: true,
        multisampling: match attrib!(display, config_id, ffi::egl::SAMPLES) {
            0 | 1 => None,
            a => Some(a as u16),
        },
        srgb: false,        // TODO: use EGL_KHR_gl_colorspace to know that
    };

    Ok((config_id, desc))
}

unsafe fn create_context(display: ffi::egl::types::EGLDisplay,
                         egl_version: &(ffi::egl::types::EGLint, ffi::egl::types::EGLint),
                         extensions: &[String], api: Api, version: (u8, u8),
                         config_id: ffi::egl::types::EGLConfig, gl_debug: bool,
                         gl_robustness: Robustness)
                         -> Result<ffi::egl::types::EGLContext, CreationError>
{
    let mut context_attributes = Vec::with_capacity(10);
    let mut flags = 0;

    if egl_version >= &(1, 5) || extensions.iter().find(|s| s == &"EGL_KHR_create_context")
                                                  .is_some()
    {
        context_attributes.push(ffi::egl::CONTEXT_MAJOR_VERSION as i32);
        context_attributes.push(version.0 as i32);
        context_attributes.push(ffi::egl::CONTEXT_MINOR_VERSION as i32);
        context_attributes.push(version.1 as i32);

        // handling robustness
        let supports_robustness = egl_version >= &(1, 5) ||
                                  extensions.iter()
                                            .find(|s| s == &"EGL_EXT_create_context_robustness")
                                            .is_some();

        match gl_robustness {
            Robustness::NotRobust => (),

            Robustness::NoError => {
                if extensions.iter().find(|s| s == &"EGL_KHR_create_context_no_error").is_some() {
                    context_attributes.push(ffi::egl::CONTEXT_OPENGL_NO_ERROR_KHR as c_int);
                    context_attributes.push(1);
                }
            },

            Robustness::RobustNoResetNotification => {
                if supports_robustness {
                    context_attributes.push(ffi::egl::CONTEXT_OPENGL_RESET_NOTIFICATION_STRATEGY
                                            as c_int);
                    context_attributes.push(ffi::egl::NO_RESET_NOTIFICATION as c_int);
                    flags = flags | ffi::egl::CONTEXT_OPENGL_ROBUST_ACCESS as c_int;
                } else {
                    return Err(CreationError::RobustnessNotSupported);
                }
            },

            Robustness::TryRobustNoResetNotification => {
                if supports_robustness {
                    context_attributes.push(ffi::egl::CONTEXT_OPENGL_RESET_NOTIFICATION_STRATEGY
                                            as c_int);
                    context_attributes.push(ffi::egl::NO_RESET_NOTIFICATION as c_int);
                    flags = flags | ffi::egl::CONTEXT_OPENGL_ROBUST_ACCESS as c_int;
                }
            },

            Robustness::RobustLoseContextOnReset => {
                if supports_robustness {
                    context_attributes.push(ffi::egl::CONTEXT_OPENGL_RESET_NOTIFICATION_STRATEGY
                                            as c_int);
                    context_attributes.push(ffi::egl::LOSE_CONTEXT_ON_RESET as c_int);
                    flags = flags | ffi::egl::CONTEXT_OPENGL_ROBUST_ACCESS as c_int;
                } else {
                    return Err(CreationError::RobustnessNotSupported);
                }
            },

            Robustness::TryRobustLoseContextOnReset => {
                if supports_robustness {
                    context_attributes.push(ffi::egl::CONTEXT_OPENGL_RESET_NOTIFICATION_STRATEGY
                                            as c_int);
                    context_attributes.push(ffi::egl::LOSE_CONTEXT_ON_RESET as c_int);
                    flags = flags | ffi::egl::CONTEXT_OPENGL_ROBUST_ACCESS as c_int;
                }
            },
        }

        if gl_debug {
            if egl_version >= &(1, 5) {
                context_attributes.push(ffi::egl::CONTEXT_OPENGL_DEBUG as i32);
                context_attributes.push(ffi::egl::TRUE as i32);
            }

            // TODO: using this flag sometimes generates an error
            //       there was a change in the specs that added this flag, so it may not be
            //       supported everywhere ; however it is not possible to know whether it is
            //       supported or not
            //flags = flags | ffi::egl::CONTEXT_OPENGL_DEBUG_BIT_KHR as i32;
        }

        context_attributes.push(ffi::egl::CONTEXT_FLAGS_KHR as i32);
        context_attributes.push(flags);

    } else if egl_version >= &(1, 3) && api == Api::OpenGlEs {
        // robustness is not supported
        match gl_robustness {
            Robustness::RobustNoResetNotification | Robustness::RobustLoseContextOnReset => {
                return Err(CreationError::RobustnessNotSupported);
            },
            _ => ()
        }

        context_attributes.push(ffi::egl::CONTEXT_CLIENT_VERSION as i32);
        context_attributes.push(version.0 as i32);
    }

    context_attributes.push(ffi::egl::NONE as i32);

    let context = egl::CreateContext(display, config_id, ptr::null(),
                                    context_attributes.as_ptr());

    if context.is_null() {
        match egl::GetError() as u32 {
            ffi::egl::BAD_ATTRIBUTE => return Err(CreationError::OpenGlVersionNotSupported),
            e => panic!("eglCreateContext failed: 0x{:x}", e),
        }
    }

    Ok(context)
}


/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use super::webgl_thread::{GLState, WebGLImpl};
use canvas_traits::webgl::{
    GLContextAttributes, GLLimits, WebGLCommand, WebGLCommandBacktrace, WebGLVersion,
};
use euclid::default::Size2D;
use gleam::gl;
use offscreen_gl_context::{
    ColorAttachmentType, DrawBuffer, GLContext, GLContextAttributes as RawGLContextAttributes,
    GLContextDispatcher,
};
use offscreen_gl_context::{GLLimits as RawGLLimits, GLVersion};
use offscreen_gl_context::{NativeGLContext, NativeGLContextHandle, NativeGLContextMethods};
use offscreen_gl_context::{OSMesaContext, OSMesaContextHandle};
use servo_config::opts;

pub trait CloneableDispatcher: GLContextDispatcher {
    fn clone(&self) -> Box<dyn GLContextDispatcher>;
}

/// The GLContextFactory is used to create shared GL contexts with the main thread GL context.
/// Currently, shared textures are used to render WebGL textures into the WR compositor.
/// In order to create a shared context, the GLContextFactory stores the handle of the main GL context.
pub enum GLContextFactory {
    Native(
        NativeGLContextHandle,
        Option<Box<dyn CloneableDispatcher + Send>>,
        gl::GlType,
    ),
    OSMesa(OSMesaContextHandle),
}

impl GLContextFactory {
    /// Creates a new GLContextFactory that uses the currently bound GL context to create shared contexts.
    pub fn current_native_handle(
        dispatcher: Box<dyn CloneableDispatcher + Send>,
        api_type: gl::GlType,
    ) -> Option<GLContextFactory> {
        let dispatcher = if cfg!(target_os = "windows") {
            // Used to dispatch functions from the GLContext thread to the main thread's
            // event loop. Required to allow WGL GLContext sharing in Windows.
            Some(dispatcher)
        } else {
            None
        };
        // FIXME(emilio): This assumes a single GL backend per platform which is
        // not true on Linux, we probably need a third `Egl` variant or abstract
        // it a bit more...
        NativeGLContext::current_handle()
            .map(|handle| GLContextFactory::Native(handle, dispatcher, api_type))
    }

    /// Creates a new GLContextFactory that uses the currently bound OSMesa context to create shared contexts.
    pub fn current_osmesa_handle() -> Option<GLContextFactory> {
        OSMesaContext::current_handle().map(GLContextFactory::OSMesa)
    }

    /// Creates a new shared GLContext with the main GLContext
    pub fn new_shared_context(
        &self,
        webgl_version: WebGLVersion,
        size: Size2D<u32>,
        attributes: GLContextAttributes,
    ) -> Result<GLContextWrapper, &'static str> {
        let attributes = map_attrs(attributes);
        Ok(match *self {
            GLContextFactory::Native(ref _handle, ref _dispatcher, ref api_type) => {
                #[cfg(target_os = "macos")]
                {
                    if opts::get().with_io_surface {
                        return GLContext::new_shared_with_dispatcher(
                            // FIXME(nox): Why are those i32 values?
                            size.to_i32(),
                            attributes,
                            ColorAttachmentType::IOSurface,
                            *api_type,
                            Self::gl_version(webgl_version),
                            None,
                            None,
                        )
                        .map(|ctx| GLContextWrapper::NativeWithIOSurface(ctx));
                    }
                }
                GLContextWrapper::Native(GLContext::new_shared_with_dispatcher(
                    // FIXME(nox): Why are those i32 values?
                    size.to_i32(),
                    attributes,
                    ColorAttachmentType::Texture,
                    *api_type,
                    Self::gl_version(webgl_version),
                    Some(_handle),
                    _dispatcher.as_ref().map(|d| (**d).clone()),
                )?)
            },
            GLContextFactory::OSMesa(ref _handle) => {
                {
                    GLContextWrapper::OSMesa(GLContext::new_shared_with_dispatcher(
                        // FIXME(nox): Why are those i32 values?
                        size.to_i32(),
                        attributes,
                        ColorAttachmentType::Texture,
                        gl::GlType::default(),
                        Self::gl_version(webgl_version),
                        Some(_handle),
                        None,
                    )?)
                }
            },
        })
    }

    /// Creates a new non-shared GLContext
    pub fn new_context(
        &self,
        webgl_version: WebGLVersion,
        size: Size2D<u32>,
        attributes: GLContextAttributes,
    ) -> Result<GLContextWrapper, &'static str> {
        let attributes = map_attrs(attributes);
        Ok(match *self {
            GLContextFactory::Native(_, _, ref api_type) => {
                GLContextWrapper::Native(GLContext::new_shared_with_dispatcher(
                    // FIXME(nox): Why are those i32 values?
                    size.to_i32(),
                    attributes,
                    ColorAttachmentType::Texture,
                    *api_type,
                    Self::gl_version(webgl_version),
                    None,
                    None,
                )?)
            },
            GLContextFactory::OSMesa(_) => {
                GLContextWrapper::OSMesa(GLContext::new_shared_with_dispatcher(
                    // FIXME(nox): Why are those i32 values?
                    size.to_i32(),
                    attributes,
                    ColorAttachmentType::Texture,
                    gl::GlType::default(),
                    Self::gl_version(webgl_version),
                    None,
                    None,
                )?)
            },
        })
    }

    fn gl_version(webgl_version: WebGLVersion) -> GLVersion {
        match webgl_version {
            WebGLVersion::WebGL1 => GLVersion::Major(2),
            WebGLVersion::WebGL2 => GLVersion::Major(3),
        }
    }
}

/// GLContextWrapper used to abstract NativeGLContext and OSMesaContext types
pub enum GLContextWrapper {
    Native(GLContext<NativeGLContext>),
    OSMesa(GLContext<OSMesaContext>),
    #[cfg(target_os = "macos")]
    NativeWithIOSurface(GLContext<NativeGLContext>),
}

impl GLContextWrapper {
    pub fn make_current(&self) {
        match *self {
            GLContextWrapper::Native(ref ctx) => {
                ctx.make_current().unwrap();
            },
            GLContextWrapper::OSMesa(ref ctx) => {
                ctx.make_current().unwrap();
            },
            #[cfg(target_os = "macos")]
            GLContextWrapper::NativeWithIOSurface(ref ctx) => {
                ctx.make_current().unwrap();
            },
        }
    }

    pub fn apply_command(
        &self,
        cmd: WebGLCommand,
        backtrace: WebGLCommandBacktrace,
        state: &mut GLState,
    ) {
        match *self {
            GLContextWrapper::Native(ref ctx) => {
                WebGLImpl::apply(ctx, state, cmd, backtrace);
            },
            GLContextWrapper::OSMesa(ref ctx) => {
                WebGLImpl::apply(ctx, state, cmd, backtrace);
            },
            #[cfg(target_os = "macos")]
            GLContextWrapper::NativeWithIOSurface(ref ctx) => {
                WebGLImpl::apply(ctx, state, cmd, backtrace);
            },
        }
    }

    pub fn gl(&self) -> &dyn gl::Gl {
        match *self {
            GLContextWrapper::Native(ref ctx) => ctx.gl(),
            GLContextWrapper::OSMesa(ref ctx) => ctx.gl(),
            #[cfg(target_os = "macos")]
            GLContextWrapper::NativeWithIOSurface(ref ctx) => ctx.gl(),
        }
    }

    pub fn get_info(&self) -> (Size2D<i32>, u32, Option<u32>, GLLimits) {
        match *self {
            GLContextWrapper::Native(ref ctx) => {
                let (real_size, texture_id, io_surface_id) = {
                    let draw_buffer = ctx.borrow_draw_buffer().unwrap();
                    (
                        draw_buffer.size(),
                        draw_buffer.get_bound_texture_id().unwrap(),
                        None,
                    )
                };

                let limits = ctx.borrow_limits().clone();

                (real_size, texture_id, io_surface_id, map_limits(limits))
            },
            GLContextWrapper::OSMesa(ref ctx) => {
                let (real_size, texture_id, io_surface_id) = {
                    let draw_buffer = ctx.borrow_draw_buffer().unwrap();
                    (
                        draw_buffer.size(),
                        draw_buffer.get_bound_texture_id().unwrap(),
                        None,
                    )
                };

                let limits = ctx.borrow_limits().clone();

                (real_size, texture_id, io_surface_id, map_limits(limits))
            },
            #[cfg(target_os = "macos")]
            GLContextWrapper::NativeWithIOSurface(ref ctx) => {
                let (real_size, texture_id, io_surface_id) = {
                    let draw_buffer = ctx.borrow_draw_buffer().unwrap();
                    (
                        draw_buffer.size(),
                        draw_buffer.get_bound_texture_id().unwrap(),
                        draw_buffer.get_active_io_surface_id(),
                    )
                };

                let limits = ctx.borrow_limits().clone();

                (real_size, texture_id, io_surface_id, map_limits(limits))
            },
        }
    }

    pub fn get_active_io_surface_id(&self) -> Option<u32> {
        match *self {
            #[cfg(target_os = "macos")]
            GLContextWrapper::NativeWithIOSurface(ref ctx) => {
                ctx.borrow_draw_buffer().unwrap().get_active_io_surface_id()
            },
            _ => None,
        }
    }

    /// Swap the backing texture for the draw buffer, returning the id of the IOsurface
    /// now used for reading.
    pub fn swap_draw_buffer(
        &mut self,
        _clear_color: (f32, f32, f32, f32),
        _mask: u32,
    ) -> Option<u32> {
        match *self {
            #[cfg(target_os = "macos")]
            GLContextWrapper::NativeWithIOSurface(ref mut ctx) => {
                ctx.swap_draw_buffer(_clear_color, _mask)
            },
            _ => None,
        }
    }

    pub fn handle_lock(&mut self) -> Option<u32> {
        match *self {
            #[cfg(target_os = "macos")]
            GLContextWrapper::NativeWithIOSurface(ref mut ctx) => ctx.handle_lock(),
            _ => None,
        }
    }

    pub fn resize(&mut self, size: Size2D<u32>) -> Result<DrawBuffer, &'static str> {
        match *self {
            GLContextWrapper::Native(ref mut ctx) => {
                // FIXME(nox): Why are those i32 values?
                ctx.resize(size.to_i32())
            },
            GLContextWrapper::OSMesa(ref mut ctx) => {
                // FIXME(nox): Why are those i32 values?
                ctx.resize(size.to_i32())
            },
            #[cfg(target_os = "macos")]
            GLContextWrapper::NativeWithIOSurface(ref mut ctx) => {
                // FIXME(nox): Why are those i32 values?
                ctx.resize(size.to_i32())
            },
        }
    }
}

fn map_limits(limits: RawGLLimits) -> GLLimits {
    GLLimits {
        max_vertex_attribs: limits.max_vertex_attribs,
        max_tex_size: limits.max_tex_size,
        max_cube_map_tex_size: limits.max_cube_map_tex_size,
        max_combined_texture_image_units: limits.max_combined_texture_image_units,
        max_fragment_uniform_vectors: limits.max_fragment_uniform_vectors,
        max_renderbuffer_size: limits.max_renderbuffer_size,
        max_texture_image_units: limits.max_texture_image_units,
        max_varying_vectors: limits.max_varying_vectors,
        max_vertex_texture_image_units: limits.max_vertex_texture_image_units,
        max_vertex_uniform_vectors: limits.max_vertex_uniform_vectors,
    }
}

pub fn map_attrs(attrs: GLContextAttributes) -> RawGLContextAttributes {
    RawGLContextAttributes {
        alpha: attrs.alpha,
        depth: attrs.depth,
        stencil: attrs.stencil,
        antialias: attrs.antialias,
        premultiplied_alpha: attrs.premultiplied_alpha,
        preserve_drawing_buffer: attrs.preserve_drawing_buffer,
    }
}

pub fn map_attrs_to_script_attrs(attrs: RawGLContextAttributes) -> GLContextAttributes {
    GLContextAttributes {
        alpha: attrs.alpha,
        depth: attrs.depth,
        stencil: attrs.stencil,
        antialias: attrs.antialias,
        premultiplied_alpha: attrs.premultiplied_alpha,
        preserve_drawing_buffer: attrs.preserve_drawing_buffer,
    }
}

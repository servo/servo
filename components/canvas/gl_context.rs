/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use super::webgl_thread::{GLState, WebGLImpl};
use canvas_traits::webgl::{
    GLContextAttributes, GLLimits, WebGLCommand, WebGLCommandBacktrace, WebGLVersion,
};
use compositing::compositor_thread::{self, CompositorProxy};
use euclid::Size2D;
use gleam::gl;
use offscreen_gl_context::{
    ColorAttachmentType, DrawBuffer, GLContext, GLContextAttributes as RawGLContextAttributes,
    GLContextDispatcher,
};
use offscreen_gl_context::{GLLimits as RawGLLimits, GLVersion};
use offscreen_gl_context::{NativeGLContext, NativeGLContextHandle, NativeGLContextMethods};
use offscreen_gl_context::{OSMesaContext, OSMesaContextHandle};
use std::sync::{Arc, Mutex};

/// The GLContextFactory is used to create shared GL contexts with the main thread GL context.
/// Currently, shared textures are used to render WebGL textures into the WR compositor.
/// In order to create a shared context, the GLContextFactory stores the handle of the main GL context.
pub enum GLContextFactory {
    Native(NativeGLContextHandle, Option<MainThreadDispatcher>),
    OSMesa(OSMesaContextHandle),
}

impl GLContextFactory {
    /// Creates a new GLContextFactory that uses the currently bound GL context to create shared contexts.
    pub fn current_native_handle(proxy: &CompositorProxy) -> Option<GLContextFactory> {
        // FIXME(emilio): This assumes a single GL backend per platform which is
        // not true on Linux, we probably need a third `Egl` variant or abstract
        // it a bit more...
        NativeGLContext::current_handle().map(|handle| {
            if cfg!(target_os = "windows") {
                // Used to dispatch functions from the GLContext thread to the main thread's event loop.
                // Required to allow WGL GLContext sharing in Windows.
                GLContextFactory::Native(handle, Some(MainThreadDispatcher::new(proxy.clone())))
            } else {
                GLContextFactory::Native(handle, None)
            }
        })
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
            GLContextFactory::Native(ref handle, ref dispatcher) => {
                let dispatcher = dispatcher.as_ref().map(|d| Box::new(d.clone()) as Box<_>);
                GLContextWrapper::Native(GLContext::new_shared_with_dispatcher(
                    // FIXME(nox): Why are those i32 values?
                    size.to_i32(),
                    attributes,
                    ColorAttachmentType::Texture,
                    gl::GlType::default(),
                    Self::gl_version(webgl_version),
                    Some(handle),
                    dispatcher,
                )?)
            },
            GLContextFactory::OSMesa(ref handle) => {
                GLContextWrapper::OSMesa(GLContext::new_shared_with_dispatcher(
                    // FIXME(nox): Why are those i32 values?
                    size.to_i32(),
                    attributes,
                    ColorAttachmentType::Texture,
                    gl::GlType::default(),
                    Self::gl_version(webgl_version),
                    Some(handle),
                    None,
                )?)
            },
        })
    }

    /// Creates a new non-shared GLContext
    pub fn new_context(
        &self,
        webgl_version: WebGLVersion,
        size: Size2D<u32>,
        attributes: GLContextAttributes,
        color_attachment_type: ColorAttachmentType,
    ) -> Result<GLContextWrapper, &'static str> {
        if ColorAttachmentType::IOSurface == color_attachment_type
            && cfg!(not(target_os="macos")) {
                return Err("IOSurface is not supported on this platform")
        }
        let attributes = map_attrs(attributes);
        Ok(match *self {
            GLContextFactory::Native(..) => {
                GLContextWrapper::Native(GLContext::new_shared_with_dispatcher(
                    // FIXME(nox): Why are those i32 values?
                    size.to_i32(),
                    attributes,
                    color_attachment_type,
                    gl::GlType::default(),
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
                    color_attachment_type,
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
        }
    }

    pub fn gl(&self) -> &dyn gl::Gl {
        match *self {
            GLContextWrapper::Native(ref ctx) => ctx.gl(),
            GLContextWrapper::OSMesa(ref ctx) => ctx.gl(),
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
                        draw_buffer.get_complete_io_surface_id(),
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
                        draw_buffer.get_complete_io_surface_id(),
                    )
                };

                let limits = ctx.borrow_limits().clone();

                (real_size, texture_id, io_surface_id, map_limits(limits))
            },
        }
    }

    /// Swap the backing texture for the draw buffer, returning the id of the texture
    /// now used for reading.
    pub fn swap_draw_buffer(&mut self) -> Option<u32> {
        match *self {
            GLContextWrapper::Native(ref mut ctx) => ctx.swap_draw_buffer(),
            GLContextWrapper::OSMesa(ref mut ctx) => ctx.swap_draw_buffer(),
        }
    }

    pub fn draw_buffer_is_bound(&self) -> bool {
        match *self {
            GLContextWrapper::Native(ref ctx) => ctx.draw_buffer_is_bound(),
            GLContextWrapper::OSMesa(ref ctx) => ctx.draw_buffer_is_bound(),
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
        }
    }

    pub fn get_framebuffer(&self) -> gl::GLuint {
        match *self {
            GLContextWrapper::Native(ref ctx) => {
                ctx.get_framebuffer()
            },
            GLContextWrapper::OSMesa(ref ctx) => {
                ctx.get_framebuffer()
            },
        }
    }
}

/// Implements GLContextDispatcher to dispatch functions from GLContext threads to the main thread's event loop.
/// It's used in Windows to allow WGL GLContext sharing.
#[derive(Clone)]
pub struct MainThreadDispatcher {
    compositor_proxy: Arc<Mutex<CompositorProxy>>,
}

impl MainThreadDispatcher {
    fn new(proxy: CompositorProxy) -> Self {
        Self {
            compositor_proxy: Arc::new(Mutex::new(proxy)),
        }
    }
}
impl GLContextDispatcher for MainThreadDispatcher {
    fn dispatch(&self, f: Box<dyn Fn() + Send>) {
        self.compositor_proxy
            .lock()
            .unwrap()
            .send(compositor_thread::Msg::Dispatch(f));
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

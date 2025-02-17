/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#![deny(unsafe_code)]

use std::cell::{Cell, RefCell};
use std::ffi::c_void;
use std::num::NonZeroU32;
use std::rc::Rc;

use euclid::default::{Rect, Size2D};
use euclid::Point2D;
use gleam::gl::{self, Gl};
use glow::NativeFramebuffer;
use image::RgbaImage;
use log::{debug, trace, warn};
use servo_media::player::context::{GlContext, NativeDisplay};
use surfman::chains::{PreserveBuffer, SwapChain};
#[cfg(all(target_os = "linux", not(target_env = "ohos")))]
use surfman::platform::generic::multi::connection::NativeConnection as LinuxNativeConnection;
#[cfg(all(target_os = "linux", not(target_env = "ohos")))]
use surfman::platform::generic::multi::context::NativeContext as LinuxNativeContext;
pub use surfman::Error;
use surfman::{
    Adapter, Connection, Context, ContextAttributeFlags, ContextAttributes, Device, GLApi,
    NativeContext, NativeDevice, NativeWidget, Surface, SurfaceAccess, SurfaceInfo, SurfaceTexture,
    SurfaceType,
};

/// Describes the OpenGL version that is requested when a context is created.
pub enum GLVersion {
    GL(u8, u8),
    GLES(u8, u8),
}

/// The `RenderingContext` trait defines a set of methods for managing
/// an OpenGL or GLES rendering context.
/// Implementors of this trait are responsible for handling the creation,
/// management, and destruction of the rendering context and its associated
/// resources.
pub trait RenderingContext {
    /// Prepare this [`RenderingContext`] to be rendered upon by Servo. For instance,
    /// by binding a framebuffer to the current OpenGL context.
    fn prepare_for_rendering(&self) {}
    /// Read the contents of this [`Renderingcontext`] into an in-memory image. If the
    /// image cannot be read (for instance, if no rendering has taken place yet), then
    /// `None` is returned.
    ///
    /// In a double-buffered [`RenderingContext`] this is expected to read from the back
    /// buffer. That means that once Servo renders to the context, this should return those
    /// results, even before [`RenderingContext::present`] is called.
    fn read_to_image(&self, source_rectangle: Rect<u32>) -> Option<RgbaImage>;
    /// Resizes the rendering surface to the given size.
    fn resize(&self, size: Size2D<i32>);
    /// Presents the rendered frame to the screen. In a double-buffered context, this would
    /// swap buffers.
    fn present(&self);
    /// Makes the context the current OpenGL context for this thread.
    /// After calling this function, it is valid to use OpenGL rendering
    /// commands.
    fn make_current(&self) -> Result<(), Error>;
    /// Returns the OpenGL or GLES API.
    fn gl_api(&self) -> Rc<dyn gleam::gl::Gl>;
    /// Describes the OpenGL version that is requested when a context is created.
    fn gl_version(&self) -> GLVersion;
    /// Returns the GL Context used by servo media player. Default to `GlContext::Unknown`.
    fn gl_context(&self) -> GlContext {
        GlContext::Unknown
    }
    /// Returns the GL Display used by servo media player. Default to `NativeDisplay::Unknown`.
    fn gl_display(&self) -> NativeDisplay {
        NativeDisplay::Unknown
    }
    /// Creates a texture from a given surface and returns the surface texture,
    /// the OpenGL texture object, and the size of the surface. Default to `None`.
    fn create_texture(&self, _surface: Surface) -> Option<(SurfaceTexture, u32, Size2D<i32>)> {
        None
    }
    /// Destroys the texture and returns the surface. Default to `None`.
    fn destroy_texture(&self, _surface_texture: SurfaceTexture) -> Option<Surface> {
        None
    }

    /// The connection to the display server for WebGL. Default to `None`.
    fn connection(&self) -> Option<Connection> {
        None
    }
}

/// A rendering context that uses the Surfman library to create and manage
/// the OpenGL context and surface. This struct provides the default implementation
/// of the `RenderingContext` trait, handling the creation, management, and destruction
/// of the rendering context and its associated resources.
///
/// The `SurfmanRenderingContext` struct encapsulates the necessary data and methods
/// to interact with the Surfman library, including creating surfaces, binding surfaces,
/// resizing surfaces, presenting rendered frames, and managing the OpenGL context state.
#[derive(Clone)]
pub struct SurfmanRenderingContext(Rc<RenderingContextData>);

struct RenderingContextData {
    gl: Rc<dyn Gl>,
    device: RefCell<Device>,
    context: RefCell<Context>,
    // We either render to a swap buffer or to a native widget
    swap_chain: Option<SwapChain<Device>>,
}

impl Drop for RenderingContextData {
    fn drop(&mut self) {
        let device = &mut self.device.borrow_mut();
        let context = &mut self.context.borrow_mut();
        if let Some(ref swap_chain) = self.swap_chain {
            let _ = swap_chain.destroy(device, context);
        }
        let _ = device.destroy_context(context);
    }
}

impl RenderingContext for SurfmanRenderingContext {
    fn gl_context(&self) -> GlContext {
        #[cfg(all(target_os = "linux", not(target_env = "ohos")))]
        {
            match self.native_context() {
                NativeContext::Default(LinuxNativeContext::Default(native_context)) => {
                    GlContext::Egl(native_context.egl_context as usize)
                },
                NativeContext::Default(LinuxNativeContext::Alternate(native_context)) => {
                    GlContext::Egl(native_context.egl_context as usize)
                },
                NativeContext::Alternate(_) => GlContext::Unknown,
            }
        }
        #[cfg(target_os = "windows")]
        {
            #[cfg(feature = "no-wgl")]
            {
                GlContext::Egl(self.native_context().egl_context as usize)
            }
            #[cfg(not(feature = "no-wgl"))]
            GlContext::Unknown
        }
        #[cfg(not(any(
            target_os = "windows",
            all(target_os = "linux", not(target_env = "ohos"))
        )))]
        {
            GlContext::Unknown
        }
    }
    fn gl_display(&self) -> NativeDisplay {
        #[cfg(all(target_os = "linux", not(target_env = "ohos")))]
        {
            match self.connection().native_connection() {
                surfman::NativeConnection::Default(LinuxNativeConnection::Default(connection)) => {
                    NativeDisplay::Egl(connection.0 as usize)
                },
                surfman::NativeConnection::Default(LinuxNativeConnection::Alternate(
                    connection,
                )) => NativeDisplay::X11(connection.x11_display as usize),
                surfman::NativeConnection::Alternate(_) => NativeDisplay::Unknown,
            }
        }
        #[cfg(target_os = "windows")]
        {
            #[cfg(feature = "no-wgl")]
            {
                NativeDisplay::Egl(self.native_device().egl_display as usize)
            }
            #[cfg(not(feature = "no-wgl"))]
            NativeDisplay::Unknown
        }
        #[cfg(not(any(
            target_os = "windows",
            all(target_os = "linux", not(target_env = "ohos"))
        )))]
        {
            NativeDisplay::Unknown
        }
    }

    fn prepare_for_rendering(&self) {
        self.0.gl.bind_framebuffer(
            gleam::gl::FRAMEBUFFER,
            self.framebuffer()
                .map_or(0, |framebuffer| framebuffer.0.into()),
        );
    }

    fn read_to_image(&self, source_rectangle: Rect<u32>) -> Option<RgbaImage> {
        let framebuffer_id = self
            .framebuffer()
            .map(|framebuffer| framebuffer.0.into())
            .unwrap_or(0);
        Framebuffer::read_framebuffer_to_image(self.gl_api(), framebuffer_id, source_rectangle)
    }

    fn resize(&self, size: Size2D<i32>) {
        if let Err(err) = self.resize(size) {
            warn!("Failed to resize surface: {:?}", err);
        }
    }
    fn present(&self) {
        if let Err(err) = self.present() {
            warn!("Failed to present surface: {:?}", err);
        }
    }
    fn make_current(&self) -> Result<(), Error> {
        self.make_gl_context_current()
    }
    #[allow(unsafe_code)]
    fn gl_api(&self) -> Rc<dyn gleam::gl::Gl> {
        self.0.gl.clone()
    }
    fn gl_version(&self) -> GLVersion {
        let device = self.0.device.borrow();
        let context = self.0.context.borrow();
        let descriptor = device.context_descriptor(&context);
        let attributes = device.context_descriptor_attributes(&descriptor);
        let major = attributes.version.major;
        let minor = attributes.version.minor;
        match self.connection().gl_api() {
            GLApi::GL => GLVersion::GL(major, minor),
            GLApi::GLES => GLVersion::GLES(major, minor),
        }
    }

    fn create_texture(&self, surface: Surface) -> Option<(SurfaceTexture, u32, Size2D<i32>)> {
        let device = &self.0.device.borrow();
        let context = &mut self.0.context.borrow_mut();
        let SurfaceInfo {
            id: front_buffer_id,
            size,
            ..
        } = device.surface_info(&surface);
        debug!("... getting texture for surface {:?}", front_buffer_id);
        let surface_texture = device.create_surface_texture(context, surface).unwrap();
        let gl_texture = device
            .surface_texture_object(&surface_texture)
            .map(|tex| tex.0.get())
            .unwrap_or(0);
        Some((surface_texture, gl_texture, size))
    }

    fn destroy_texture(&self, surface_texture: SurfaceTexture) -> Option<Surface> {
        self.destroy_surface_texture(surface_texture).ok()
    }

    fn connection(&self) -> Option<Connection> {
        Some(self.connection())
    }
}

impl SurfmanRenderingContext {
    pub fn create(
        connection: &Connection,
        adapter: &Adapter,
        headless: Option<Size2D<i32>>,
    ) -> Result<Self, Error> {
        let mut device = connection.create_device(adapter)?;
        let flags = ContextAttributeFlags::ALPHA |
            ContextAttributeFlags::DEPTH |
            ContextAttributeFlags::STENCIL;
        let version = match connection.gl_api() {
            GLApi::GLES => surfman::GLVersion { major: 3, minor: 0 },
            GLApi::GL => surfman::GLVersion { major: 3, minor: 2 },
        };
        let context_attributes = ContextAttributes { flags, version };
        let context_descriptor = device.create_context_descriptor(&context_attributes)?;
        let mut context = device.create_context(&context_descriptor, None)?;
        let surface_access = SurfaceAccess::GPUOnly;
        let swap_chain = if let Some(size) = headless {
            let surface_type = SurfaceType::Generic { size };
            let surface = device.create_surface(&context, surface_access, surface_type)?;
            device
                .bind_surface_to_context(&mut context, surface)
                .map_err(|(err, mut surface)| {
                    let _ = device.destroy_surface(&mut context, &mut surface);
                    err
                })?;
            device.make_context_current(&context)?;
            Some(SwapChain::create_attached(
                &mut device,
                &mut context,
                surface_access,
            )?)
        } else {
            None
        };

        #[allow(unsafe_code)]
        let gl = {
            match connection.gl_api() {
                GLApi::GL => unsafe {
                    gl::GlFns::load_with(|s| device.get_proc_address(&context, s))
                },
                GLApi::GLES => unsafe {
                    gl::GlesFns::load_with(|s| device.get_proc_address(&context, s))
                },
            }
        };

        let device = RefCell::new(device);
        let context = RefCell::new(context);
        let data = RenderingContextData {
            gl,
            device,
            context,
            swap_chain,
        };
        Ok(SurfmanRenderingContext(Rc::new(data)))
    }

    pub fn create_surface(
        &self,
        surface_type: SurfaceType<NativeWidget>,
    ) -> Result<Surface, Error> {
        let device = &mut self.0.device.borrow_mut();
        let context = &self.0.context.borrow();
        let surface_access = SurfaceAccess::GPUOnly;
        device.create_surface(context, surface_access, surface_type)
    }

    pub fn bind_surface(&self, surface: Surface) -> Result<(), Error> {
        let device = &self.0.device.borrow();
        let context = &mut self.0.context.borrow_mut();
        device
            .bind_surface_to_context(context, surface)
            .map_err(|(err, mut surface)| {
                let _ = device.destroy_surface(context, &mut surface);
                err
            })?;

        device.make_context_current(context)?;
        Ok(())
    }

    pub fn destroy_surface(&self, mut surface: Surface) -> Result<(), Error> {
        let device = &self.0.device.borrow();
        let context = &mut self.0.context.borrow_mut();
        device.destroy_surface(context, &mut surface)
    }

    pub fn create_surface_texture(&self, surface: Surface) -> Result<SurfaceTexture, Error> {
        let device = &self.0.device.borrow();
        let context = &mut self.0.context.borrow_mut();
        device
            .create_surface_texture(context, surface)
            .map_err(|(error, _)| error)
    }

    pub fn destroy_surface_texture(
        &self,
        surface_texture: SurfaceTexture,
    ) -> Result<Surface, Error> {
        let device = &self.0.device.borrow();
        let context = &mut self.0.context.borrow_mut();
        device
            .destroy_surface_texture(context, surface_texture)
            .map_err(|(error, _)| error)
    }

    pub fn make_gl_context_current(&self) -> Result<(), Error> {
        let device = &self.0.device.borrow();
        let context = &self.0.context.borrow();
        device.make_context_current(context)
    }

    pub fn swap_chain(&self) -> Result<&SwapChain<Device>, Error> {
        self.0.swap_chain.as_ref().ok_or(Error::WidgetAttached)
    }

    pub fn resize(&self, size: Size2D<i32>) -> Result<(), Error> {
        let device = &mut self.0.device.borrow_mut();
        let context = &mut self.0.context.borrow_mut();
        if let Some(swap_chain) = self.0.swap_chain.as_ref() {
            return swap_chain.resize(device, context, size);
        }
        let mut surface = device.unbind_surface_from_context(context)?.unwrap();
        device.resize_surface(context, &mut surface, size)?;
        device
            .bind_surface_to_context(context, surface)
            .map_err(|(err, mut surface)| {
                let _ = device.destroy_surface(context, &mut surface);
                err
            })
    }

    pub fn present(&self) -> Result<(), Error> {
        let device = &mut self.0.device.borrow_mut();
        let context = &mut self.0.context.borrow_mut();
        if let Some(ref swap_chain) = self.0.swap_chain {
            return swap_chain.swap_buffers(device, context, PreserveBuffer::No);
        }
        let mut surface = device.unbind_surface_from_context(context)?.unwrap();
        device.present_surface(context, &mut surface)?;
        device
            .bind_surface_to_context(context, surface)
            .map_err(|(err, mut surface)| {
                let _ = device.destroy_surface(context, &mut surface);
                err
            })
    }

    /// Invoke a closure with the surface associated with the current front buffer.
    /// This can be used to create a surfman::SurfaceTexture to blit elsewhere.
    pub fn with_front_buffer<F: FnOnce(&Device, Surface) -> Surface>(&self, f: F) {
        let device = &mut self.0.device.borrow_mut();
        let context = &mut self.0.context.borrow_mut();
        let surface = device
            .unbind_surface_from_context(context)
            .unwrap()
            .unwrap();
        let surface = f(device, surface);
        device.bind_surface_to_context(context, surface).unwrap();
    }

    pub fn device(&self) -> std::cell::Ref<Device> {
        self.0.device.borrow()
    }

    pub fn connection(&self) -> Connection {
        let device = &self.0.device.borrow();
        device.connection()
    }

    pub fn adapter(&self) -> Adapter {
        let device = &self.0.device.borrow();
        device.adapter()
    }

    pub fn native_context(&self) -> NativeContext {
        let device = &self.0.device.borrow();
        let context = &self.0.context.borrow();
        device.native_context(context)
    }

    pub fn native_device(&self) -> NativeDevice {
        let device = &self.0.device.borrow();
        device.native_device()
    }

    pub fn context_attributes(&self) -> ContextAttributes {
        let device = &self.0.device.borrow();
        let context = &self.0.context.borrow();
        let descriptor = &device.context_descriptor(context);
        device.context_descriptor_attributes(descriptor)
    }

    pub fn context_surface_info(&self) -> Result<Option<SurfaceInfo>, Error> {
        let device = &self.0.device.borrow();
        let context = &self.0.context.borrow();
        device.context_surface_info(context)
    }

    pub fn surface_info(&self, surface: &Surface) -> SurfaceInfo {
        let device = &self.0.device.borrow();
        device.surface_info(surface)
    }

    pub fn surface_texture_object(&self, surface: &SurfaceTexture) -> u32 {
        let device = &self.0.device.borrow();
        device
            .surface_texture_object(surface)
            .map(|t| t.0.get())
            .unwrap_or_default()
    }

    pub fn get_proc_address(&self, name: &str) -> *const c_void {
        let device = &self.0.device.borrow();
        let context = &self.0.context.borrow();
        device.get_proc_address(context, name)
    }

    pub fn unbind_native_surface_from_context(&self) -> Result<(), Error> {
        let device = self.0.device.borrow_mut();
        let mut context = self.0.context.borrow_mut();
        let mut surface = device.unbind_surface_from_context(&mut context)?.unwrap();
        device.destroy_surface(&mut context, &mut surface)?;
        Ok(())
    }

    pub fn bind_native_surface_to_context(&self, native_widget: NativeWidget) -> Result<(), Error> {
        let mut device = self.0.device.borrow_mut();
        let mut context = self.0.context.borrow_mut();
        let surface_access = SurfaceAccess::GPUOnly;
        let surface_type = SurfaceType::Widget { native_widget };
        let surface = device.create_surface(&context, surface_access, surface_type)?;
        device
            .bind_surface_to_context(&mut context, surface)
            .map_err(|(err, mut surface)| {
                let _ = device.destroy_surface(&mut context, &mut surface);
                err
            })?;
        device.make_context_current(&context)?;
        Ok(())
    }

    pub fn framebuffer(&self) -> Option<NativeFramebuffer> {
        self.context_surface_info()
            .unwrap_or(None)
            .and_then(|info| info.framebuffer_object)
    }

    /// Create a new offscreen context that is compatible with this [`SurfmanRenderingContext`].
    /// The contents of the resulting [`OffscreenRenderingContext`] are guaranteed to be blit
    /// compatible with the this context.
    pub fn offscreen_context(&self, size: Size2D<u32>) -> OffscreenRenderingContext {
        OffscreenRenderingContext::new(SurfmanRenderingContext(self.0.clone()), size)
    }
}

struct Framebuffer {
    gl: Rc<dyn Gl>,
    size: Size2D<u32>,
    framebuffer_id: gl::GLuint,
    renderbuffer_id: gl::GLuint,
    texture_id: gl::GLuint,
}

impl Framebuffer {
    fn bind(&self) {
        trace!("Binding FBO {}", self.framebuffer_id);
        self.gl
            .bind_framebuffer(gl::FRAMEBUFFER, self.framebuffer_id)
    }
}

impl Drop for Framebuffer {
    fn drop(&mut self) {
        self.gl.bind_framebuffer(gl::FRAMEBUFFER, 0);
        self.gl.delete_textures(&[self.texture_id]);
        self.gl.delete_renderbuffers(&[self.renderbuffer_id]);
        self.gl.delete_framebuffers(&[self.framebuffer_id]);
    }
}

impl Framebuffer {
    fn new(gl: Rc<dyn Gl>, size: Size2D<u32>) -> Self {
        let framebuffer_ids = gl.gen_framebuffers(1);
        gl.bind_framebuffer(gl::FRAMEBUFFER, framebuffer_ids[0]);

        let texture_ids = gl.gen_textures(1);
        gl.bind_texture(gl::TEXTURE_2D, texture_ids[0]);
        gl.tex_image_2d(
            gl::TEXTURE_2D,
            0,
            gl::RGBA as gl::GLint,
            size.width as gl::GLsizei,
            size.height as gl::GLsizei,
            0,
            gl::RGBA,
            gl::UNSIGNED_BYTE,
            None,
        );
        gl.tex_parameter_i(
            gl::TEXTURE_2D,
            gl::TEXTURE_MAG_FILTER,
            gl::NEAREST as gl::GLint,
        );
        gl.tex_parameter_i(
            gl::TEXTURE_2D,
            gl::TEXTURE_MIN_FILTER,
            gl::NEAREST as gl::GLint,
        );

        gl.framebuffer_texture_2d(
            gl::FRAMEBUFFER,
            gl::COLOR_ATTACHMENT0,
            gl::TEXTURE_2D,
            texture_ids[0],
            0,
        );

        gl.bind_texture(gl::TEXTURE_2D, 0);

        let renderbuffer_ids = gl.gen_renderbuffers(1);
        let depth_rb = renderbuffer_ids[0];
        gl.bind_renderbuffer(gl::RENDERBUFFER, depth_rb);
        gl.renderbuffer_storage(
            gl::RENDERBUFFER,
            gl::DEPTH_COMPONENT24,
            size.width as gl::GLsizei,
            size.height as gl::GLsizei,
        );
        gl.framebuffer_renderbuffer(
            gl::FRAMEBUFFER,
            gl::DEPTH_ATTACHMENT,
            gl::RENDERBUFFER,
            depth_rb,
        );

        Self {
            gl,
            size,
            framebuffer_id: *framebuffer_ids
                .first()
                .expect("Guaranteed by GL operations"),
            renderbuffer_id: *renderbuffer_ids
                .first()
                .expect("Guaranteed by GL operations"),
            texture_id: *texture_ids.first().expect("Guaranteed by GL operations"),
        }
    }

    fn read_to_image(&self, source_rectangle: Rect<u32>) -> Option<RgbaImage> {
        Self::read_framebuffer_to_image(self.gl.clone(), self.framebuffer_id, source_rectangle)
    }

    fn read_framebuffer_to_image(
        gl: Rc<dyn Gl>,
        framebuffer_id: u32,
        source_rectangle: Rect<u32>,
    ) -> Option<RgbaImage> {
        gl.bind_framebuffer(gl::FRAMEBUFFER, framebuffer_id);

        // For some reason, OSMesa fails to render on the 3rd
        // attempt in headless mode, under some conditions.
        // I think this can only be some kind of synchronization
        // bug in OSMesa, but explicitly un-binding any vertex
        // array here seems to work around that bug.
        // See https://github.com/servo/servo/issues/18606.
        gl.bind_vertex_array(0);

        let mut pixels = gl.read_pixels(
            source_rectangle.origin.x as i32,
            source_rectangle.origin.y as i32,
            source_rectangle.width() as gl::GLsizei,
            source_rectangle.height() as gl::GLsizei,
            gl::RGBA,
            gl::UNSIGNED_BYTE,
        );
        let gl_error = gl.get_error();
        if gl_error != gl::NO_ERROR {
            warn!("GL error code 0x{gl_error:x} set after read_pixels");
        }

        // flip image vertically (texture is upside down)
        let source_rectangle = source_rectangle.to_usize();
        let orig_pixels = pixels.clone();
        let stride = source_rectangle.width() * 4;
        for y in 0..source_rectangle.height() {
            let dst_start = y * stride;
            let src_start = (source_rectangle.height() - y - 1) * stride;
            let src_slice = &orig_pixels[src_start..src_start + stride];
            pixels[dst_start..dst_start + stride].clone_from_slice(&src_slice[..stride]);
        }

        RgbaImage::from_raw(
            source_rectangle.width() as u32,
            source_rectangle.height() as u32,
            pixels,
        )
    }
}

pub struct OffscreenRenderingContext {
    parent_context: SurfmanRenderingContext,
    size: Cell<Size2D<u32>>,
    back_framebuffer: RefCell<Framebuffer>,
    front_framebuffer: RefCell<Option<Framebuffer>>,
}

type RenderToParentCallback = Box<dyn Fn(&glow::Context, Rect<i32>) + Send + Sync>;

impl OffscreenRenderingContext {
    fn new(parent_context: SurfmanRenderingContext, size: Size2D<u32>) -> Self {
        let next_framebuffer = Framebuffer::new(parent_context.gl_api(), size);
        Self {
            parent_context,
            size: Cell::new(size),
            back_framebuffer: RefCell::new(next_framebuffer),
            front_framebuffer: Default::default(),
        }
    }

    pub fn parent_context(&self) -> &SurfmanRenderingContext {
        &self.parent_context
    }

    pub fn front_framebuffer_id(&self) -> Option<gl::GLuint> {
        self.front_framebuffer
            .borrow()
            .as_ref()
            .map(|framebuffer| framebuffer.framebuffer_id)
    }

    pub fn render_to_parent_callback(&self) -> Option<RenderToParentCallback> {
        // Don't accept a `None` context for the read framebuffer.
        let front_framebuffer_id =
            NonZeroU32::new(self.front_framebuffer_id()?).map(NativeFramebuffer)?;
        let parent_context_framebuffer_id = self.parent_context.framebuffer();
        let size = self.size.get();
        Some(Box::new(move |gl, target_rect| {
            Self::render_framebuffer_to_parent_context(
                gl,
                Rect::new(Point2D::origin(), size.to_i32()),
                front_framebuffer_id,
                target_rect,
                parent_context_framebuffer_id,
            );
        }))
    }

    #[allow(unsafe_code)]
    fn render_framebuffer_to_parent_context(
        gl: &glow::Context,
        source_rect: Rect<i32>,
        source_framebuffer_id: NativeFramebuffer,
        target_rect: Rect<i32>,
        target_framebuffer_id: Option<NativeFramebuffer>,
    ) {
        use glow::HasContext as _;
        unsafe {
            gl.clear_color(0.0, 0.0, 0.0, 0.0);
            gl.scissor(
                target_rect.origin.x,
                target_rect.origin.y,
                target_rect.width(),
                target_rect.height(),
            );
            gl.enable(gl::SCISSOR_TEST);
            gl.clear(gl::COLOR_BUFFER_BIT);
            gl.disable(gl::SCISSOR_TEST);

            gl.bind_framebuffer(gl::READ_FRAMEBUFFER, Some(source_framebuffer_id));
            gl.bind_framebuffer(gl::DRAW_FRAMEBUFFER, target_framebuffer_id);

            gl.blit_framebuffer(
                source_rect.origin.x,
                source_rect.origin.y,
                source_rect.origin.x + source_rect.width(),
                source_rect.origin.y + source_rect.height(),
                target_rect.origin.x,
                target_rect.origin.y,
                target_rect.origin.x + target_rect.width(),
                target_rect.origin.y + target_rect.height(),
                gl::COLOR_BUFFER_BIT,
                gl::NEAREST,
            );
            gl.bind_framebuffer(gl::FRAMEBUFFER, target_framebuffer_id);
        }
    }
}

impl RenderingContext for OffscreenRenderingContext {
    fn resize(&self, size: Size2D<i32>) {
        // We do not resize any buffers right now. The current buffers might be too big or too
        // small, but we only want to ensure (later) that next buffer that we draw to is the
        // correct size.
        self.size.set(size.to_u32());
    }

    fn prepare_for_rendering(&self) {
        self.back_framebuffer.borrow().bind();
    }

    fn present(&self) {
        trace!(
            "Unbinding FBO {}",
            self.back_framebuffer.borrow().framebuffer_id
        );
        self.gl_api().bind_framebuffer(gl::FRAMEBUFFER, 0);

        let new_back_framebuffer = match self.front_framebuffer.borrow_mut().take() {
            Some(framebuffer) if framebuffer.size == self.size.get() => framebuffer,
            _ => Framebuffer::new(self.gl_api(), self.size.get()),
        };

        let new_front_framebuffer = std::mem::replace(
            &mut *self.back_framebuffer.borrow_mut(),
            new_back_framebuffer,
        );
        *self.front_framebuffer.borrow_mut() = Some(new_front_framebuffer);
    }

    fn make_current(&self) -> Result<(), surfman::Error> {
        self.parent_context.make_gl_context_current()
    }

    fn gl_api(&self) -> Rc<dyn gleam::gl::Gl> {
        self.parent_context.gl_api()
    }

    fn gl_version(&self) -> GLVersion {
        self.parent_context.gl_version()
    }

    fn create_texture(&self, surface: Surface) -> Option<(SurfaceTexture, u32, Size2D<i32>)> {
        self.parent_context.create_texture(surface)
    }

    fn destroy_texture(&self, surface_texture: SurfaceTexture) -> Option<Surface> {
        self.parent_context.destroy_texture(surface_texture)
    }

    fn connection(&self) -> Option<Connection> {
        Some(self.parent_context.connection())
    }

    fn read_to_image(&self, source_rectangle: Rect<u32>) -> Option<RgbaImage> {
        self.back_framebuffer
            .borrow()
            .read_to_image(source_rectangle)
    }
}

#[cfg(test)]
mod test {
    use euclid::{Point2D, Rect, Size2D};
    use gleam::gl;
    use image::Rgba;
    use surfman::{Connection, ContextAttributeFlags, ContextAttributes, Error, GLApi, GLVersion};

    use super::Framebuffer;

    #[test]
    #[allow(unsafe_code)]
    fn test_read_pixels() -> Result<(), Error> {
        let connection = Connection::new()?;
        let adapter = connection.create_software_adapter()?;
        let mut device = connection.create_device(&adapter)?;
        let context_descriptor = device.create_context_descriptor(&ContextAttributes {
            version: GLVersion::new(3, 0),
            flags: ContextAttributeFlags::empty(),
        })?;
        let mut context = device.create_context(&context_descriptor, None)?;

        let gl = match connection.gl_api() {
            GLApi::GL => unsafe { gl::GlFns::load_with(|s| device.get_proc_address(&context, s)) },
            GLApi::GLES => unsafe {
                gl::GlesFns::load_with(|s| device.get_proc_address(&context, s))
            },
        };

        device.make_context_current(&context)?;

        {
            const SIZE: u32 = 16;
            let framebuffer = Framebuffer::new(gl, Size2D::new(SIZE, SIZE));
            framebuffer.bind();
            framebuffer
                .gl
                .clear_color(12.0 / 255.0, 34.0 / 255.0, 56.0 / 255.0, 78.0 / 255.0);
            framebuffer.gl.clear(gl::COLOR_BUFFER_BIT);

            let img = framebuffer
                .read_to_image(Rect::new(Point2D::zero(), Size2D::new(SIZE, SIZE)))
                .expect("Should have been able to read back image.");
            assert_eq!(img.width(), SIZE);
            assert_eq!(img.height(), SIZE);

            let expected_pixel: Rgba<u8> = Rgba([12, 34, 56, 78]);
            assert!(img.pixels().all(|&p| p == expected_pixel));
        }

        device.destroy_context(&mut context)?;

        Ok(())
    }
}

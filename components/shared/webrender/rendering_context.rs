/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#![deny(unsafe_code)]

use std::cell::RefCell;
use std::ffi::c_void;
use std::rc::Rc;

use euclid::default::Size2D;
use gleam::gl;
use log::{debug, warn};
use surfman::chains::{PreserveBuffer, SwapChain};
pub use surfman::Error;
use surfman::{
    Adapter, Connection, Context, ContextAttributeFlags, ContextAttributes, Device, GLApi,
    GLVersion, NativeContext, NativeDevice, NativeWidget, Surface, SurfaceAccess, SurfaceInfo,
    SurfaceTexture, SurfaceType,
};

/// The `RenderingContext` trait defines a set of methods for managing
/// an OpenGL or GLES rendering context.
/// Implementors of this trait are responsible for handling the creation,
/// management, and destruction of the rendering context and its associated
/// resources.
pub trait RenderingContext {
    /// Returns the native OpenGL or GLES device handle
    fn device(&self) -> NativeDevice;
    /// Returns the native OpenGL or GLES context handle.
    fn context(&self) -> NativeContext;
    /// Resizes the rendering surface to the given size.
    fn resize(&self, size: Size2D<i32>);
    /// Presents the rendered frame to the screen.
    fn present(&self);
    /// Binds a native widget to the rendering context.
    fn bind_native_surface_to_context(&self, native_widget: NativeWidget);
    /// The connection to the display server.
    fn connection(&self) -> Connection;
    /// Represents a hardware display adapter that can be used for
    /// rendering (including the CPU).
    fn adapter(&self) -> Adapter;
    /// Makes the context the current OpenGL context for this thread.
    /// After calling this function, it is valid to use OpenGL rendering
    /// commands.
    fn make_current(&self) -> Result<(), Error>;
    /// Returns the OpenGL framebuffer object needed to render to the surface.
    fn framebuffer_object(&self) -> u32;
    /// Returns the OpenGL or GLES API.
    fn gl_api(&self) -> Rc<dyn gleam::gl::Gl>;
    /// Describes the OpenGL version that is requested when a context is created.
    fn gl_version(&self) -> GLVersion;
    /// Invalidates the native surface by unbinding it from the context.
    /// This is used only on Android for when the underlying native surface
    /// can be lost during servo's lifetime.
    /// For example, this happens when the app is sent to background.
    /// We need to unbind the surface so that we don't try to use it again.
    fn invalidate_native_surface(&self);
    /// Replaces the native surface with a new one.
    /// This is used only on Android for when the app moves to foreground
    /// and the system creates a new native surface that needs to bound to
    /// the current context.
    fn replace_native_surface(
        &self,
        native_widget: *mut c_void,
        coords: euclid::Size2D<i32, webrender_api::units::DevicePixel>,
    );
    /// Creates a texture from a given surface and returns the surface texture,
    /// the OpenGL texture object, and the size of the surface.
    fn create_texture(&self, surface: Surface) -> (SurfaceTexture, u32, Size2D<i32>);
    /// Destroys the texture and returns the surface.
    fn destroy_texture(&self, surface_texture: SurfaceTexture) -> Surface;
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
    fn device(&self) -> NativeDevice {
        self.native_device()
    }
    fn context(&self) -> NativeContext {
        self.native_context()
    }
    fn connection(&self) -> Connection {
        self.connection()
    }
    fn adapter(&self) -> Adapter {
        self.adapter()
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
    fn bind_native_surface_to_context(&self, native_widget: NativeWidget) {
        if let Err(err) = self.bind_native_surface_to_context(native_widget) {
            warn!("Failed to bind native surface to context: {:?}", err);
        }
    }
    fn make_current(&self) -> Result<(), Error> {
        self.make_gl_context_current()
    }
    fn framebuffer_object(&self) -> u32 {
        self.context_surface_info()
            .unwrap_or(None)
            .map(|info| info.framebuffer_object)
            .unwrap_or(0)
    }
    #[allow(unsafe_code)]
    fn gl_api(&self) -> Rc<dyn gleam::gl::Gl> {
        let context = self.0.context.borrow();
        let device = self.0.device.borrow();
        match self.connection().gl_api() {
            GLApi::GL => unsafe { gl::GlFns::load_with(|s| device.get_proc_address(&context, s)) },
            GLApi::GLES => unsafe {
                gl::GlesFns::load_with(|s| device.get_proc_address(&context, s))
            },
        }
    }
    fn gl_version(&self) -> GLVersion {
        let device = self.0.device.borrow();
        let context = self.0.context.borrow();
        let descriptor = device.context_descriptor(&context);
        let attributes = device.context_descriptor_attributes(&descriptor);
        attributes.version
    }
    fn invalidate_native_surface(&self) {
        if let Err(e) = self.unbind_native_surface_from_context() {
            warn!("Unbinding native surface from context failed ({:?})", e);
        }
    }
    #[allow(unsafe_code)]
    #[allow(clippy::not_unsafe_ptr_arg_deref)] // It has an unsafe block inside
    fn replace_native_surface(
        &self,
        native_widget: *mut c_void,
        coords: euclid::Size2D<i32, webrender_api::units::DevicePixel>,
    ) {
        let connection = self.connection();
        let native_widget =
            unsafe { connection.create_native_widget_from_ptr(native_widget, coords.to_untyped()) };
        if let Err(e) = self.bind_native_surface_to_context(native_widget) {
            warn!("Binding native surface to context failed ({:?})", e);
        }
    }

    fn create_texture(&self, surface: Surface) -> (SurfaceTexture, u32, Size2D<i32>) {
        let device = &self.0.device.borrow();
        let context = &mut self.0.context.borrow_mut();
        let SurfaceInfo {
            id: front_buffer_id,
            size,
            ..
        } = device.surface_info(&surface);
        debug!("... getting texture for surface {:?}", front_buffer_id);
        let surface_texture = device.create_surface_texture(context, surface).unwrap();
        let gl_texture = device.surface_texture_object(&surface_texture);
        (surface_texture, gl_texture, size)
    }

    fn destroy_texture(&self, surface_texture: SurfaceTexture) -> Surface {
        self.destroy_surface_texture(surface_texture).unwrap()
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
            GLApi::GLES => GLVersion { major: 3, minor: 0 },
            GLApi::GL => GLVersion { major: 3, minor: 2 },
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
        let device = RefCell::new(device);
        let context = RefCell::new(context);
        let data = RenderingContextData {
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
        device.surface_texture_object(surface)
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
}

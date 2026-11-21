/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use euclid::{Size2D, UnknownUnit};
use glow as gl;
use openxr::{
    ExtensionSet, FrameStream, FrameWaiter, Graphics, Instance, Session, SwapchainUsageFlags,
    SystemId,
};
use surfman::{
    Context as SurfmanContext, Device as SurfmanDevice, Error as SurfmanError, SurfaceTexture,
};
use webxr_api::Error;

pub enum GraphicsProvider {}

#[derive(Default)]
pub struct SessionGraphics {}

#[derive(Default)]
pub struct LayerGraphics {}

impl LayerGraphics {
    pub fn destroy(self, _gl: Option<&gl::Context>) {}
}

pub enum SwapchainTexture {
    Surface(SurfaceTexture),
}

impl SwapchainTexture {
    pub fn gl_texture(&self, device: &SurfmanDevice) -> Option<gl::NativeTexture> {
        match self {
            SwapchainTexture::Surface(surface_texture) => {
                device.surface_texture_object(surface_texture)
            },
        }
    }

    pub fn gl_texture_target(&self, device: &SurfmanDevice) -> u32 {
        match self {
            SwapchainTexture::Surface(_) => device.surface_gl_texture_target(),
        }
    }

    pub fn destroy(self, device: &SurfmanDevice, context: &mut SurfmanContext) {
        let SwapchainTexture::Surface(surface_texture) = self;
        if let Ok(mut surface) = device.destroy_surface_texture(context, surface_texture) {
            let _ = device.destroy_surface(context, &mut surface);
        }
    }
}

pub trait GraphicsProviderMethods<G: Graphics> {
    /// Whether swapchain textures use a top-left origin relative to OpenGL/WebGL's
    /// bottom-left origin without the graphics backend flipping during presentation.
    fn needs_y_flip() -> bool {
        true
    }

    fn swapchain_usage_flags() -> SwapchainUsageFlags {
        SwapchainUsageFlags::COLOR_ATTACHMENT | SwapchainUsageFlags::SAMPLED
    }

    fn enable_graphics_extensions(_exts: &mut ExtensionSet) {}

    fn enable_graphics_extensions_for(
        _supported: &ExtensionSet,
        exts: &mut ExtensionSet,
    ) -> Result<(), String> {
        Self::enable_graphics_extensions(exts);
        Ok(())
    }

    fn pick_format(formats: &[u32]) -> u32 {
        formats[0]
    }

    fn try_pick_format(formats: &[u32]) -> Result<u32, Error> {
        Ok(Self::pick_format(formats))
    }

    fn create_session(
        _instance: &Instance,
        _system: SystemId,
        _device: &SurfmanDevice,
    ) -> Result<(Session<G>, FrameWaiter, FrameStream<G>), Error> {
        Err(Error::NoMatchingDevice)
    }

    fn create_session_with_graphics(
        instance: &Instance,
        system: SystemId,
        device: &SurfmanDevice,
    ) -> Result<(Session<G>, FrameWaiter, FrameStream<G>, SessionGraphics), Error> {
        let (session, waiter, stream) = Self::create_session(instance, system, device)?;
        Ok((session, waiter, stream, SessionGraphics::default()))
    }

    fn create_layer_graphics(
        _session_graphics: &SessionGraphics,
        _gl: &gl::Context,
        _size: &Size2D<i32, UnknownUnit>,
        _format: u32,
    ) -> Result<LayerGraphics, Error> {
        Ok(LayerGraphics::default())
    }

    fn surface_texture_from_swapchain_texture(
        _image: <G as Graphics>::SwapchainImage,
        _device: &SurfmanDevice,
        _context: &mut SurfmanContext,
        _size: &Size2D<i32, UnknownUnit>,
    ) -> Result<SurfaceTexture, SurfmanError> {
        Err(SurfmanError::Unimplemented)
    }

    fn create_swapchain_texture(
        image: <G as Graphics>::SwapchainImage,
        device: &SurfmanDevice,
        context: &mut SurfmanContext,
        size: &Size2D<i32, UnknownUnit>,
        _layer_graphics: &LayerGraphics,
    ) -> Result<SwapchainTexture, SurfmanError> {
        Self::surface_texture_from_swapchain_texture(image, device, context, size)
            .map(SwapchainTexture::Surface)
    }

    fn present_layer(
        _layer_graphics: &LayerGraphics,
        _image: <G as Graphics>::SwapchainImage,
        _gl: &gl::Context,
    ) -> Result<(), Error> {
        Ok(())
    }
}

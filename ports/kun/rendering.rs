/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::ffi::CString;
use std::num::NonZeroU32;
use std::rc::Rc;

use euclid::default::Size2D;
use gleam::gl;
use glutin::config::{Config, GetGlConfig, GlConfig};
use glutin::context::{ContextApi, ContextAttributesBuilder, PossiblyCurrentContext, Version};
use glutin::display::GetGlDisplay;
use glutin::prelude::{GlContext, GlDisplay, NotCurrentGlContext, PossiblyCurrentGlContext};
use glutin::surface::{
    GlSurface, ResizeableSurface, Surface, SurfaceTypeTrait, SwapInterval, WindowSurface,
};
use glutin_winit::GlWindow;
use raw_window_handle::HasWindowHandle;
use winit::window::Window;

/// A Servo rendering context, which holds all of the information needed
/// to render Servo's layout, and bridges WebRender and glutin.
pub struct RenderingContext {
    context: PossiblyCurrentContext,
    pub(crate) gl: Rc<dyn gl::Gl>,
}

impl RenderingContext {
    /// Create a rendering context instance.
    pub fn create(
        window: &Window,
        gl_config: &Config,
    ) -> Result<(Self, Surface<WindowSurface>), Box<dyn std::error::Error>> {
        // XXX This will panic on Android, but we care about Desktop for now.
        let raw_window_handle = window.window_handle().ok().map(|handle| handle.as_raw());
        // XXX The display could be obtained from any object created by it, so we can
        // query it from the config.
        let gl_display = gl_config.display();
        // The context creation part.
        let context_attributes = ContextAttributesBuilder::new().build(raw_window_handle);
        // Since glutin by default tries to create OpenGL core context, which may not be
        // present we should try GLES.
        let fallback_context_attributes = ContextAttributesBuilder::new()
            .with_context_api(ContextApi::Gles(None))
            .build(raw_window_handle);
        // There are also some old devices that support neither modern OpenGL nor GLES.
        // To support these we can try and create a 2.1 context.
        let legacy_context_attributes = ContextAttributesBuilder::new()
            .with_context_api(ContextApi::OpenGl(Some(Version::new(2, 1))))
            .build(raw_window_handle);
        let not_current_gl_context = unsafe {
            gl_display
                .create_context(gl_config, &context_attributes)
                .unwrap_or_else(|_| {
                    gl_display
                        .create_context(gl_config, &fallback_context_attributes)
                        .unwrap_or_else(|_| {
                            gl_display
                                .create_context(gl_config, &legacy_context_attributes)
                                .expect("failed to create context")
                        })
                })
        };

        // Create surface
        let attrs = window
            .build_surface_attributes(Default::default())
            .expect("Failed to build surface attributes");
        let surface = unsafe {
            gl_config
                .display()
                .create_window_surface(gl_config, &attrs)
                .unwrap()
        };

        // Make it current.
        let context = not_current_gl_context.make_current(&surface).unwrap();

        // Try setting vsync.
        if let Err(res) =
            surface.set_swap_interval(&context, SwapInterval::Wait(NonZeroU32::new(1).unwrap()))
        {
            log::error!("Error setting vsync: {res:?}");
        }

        let gl = match context.context_api() {
            ContextApi::OpenGl(_) => unsafe {
                gleam::gl::GlFns::load_with(|symbol| {
                    let symbol = CString::new(symbol).unwrap();
                    gl_display.get_proc_address(symbol.as_c_str()) as *const _
                })
            },
            ContextApi::Gles(_) => unsafe {
                gleam::gl::GlesFns::load_with(|symbol| {
                    let symbol = CString::new(symbol).unwrap();
                    gl_display.get_proc_address(symbol.as_c_str()) as *const _
                })
            },
        };

        println!("Running on {}", gl.get_string(gl::RENDERER));
        println!("OpenGL Version {}", gl.get_string(gl::VERSION));
        println!(
            "Shaders version on {}",
            gl.get_string(gl::SHADING_LANGUAGE_VERSION)
        );

        Ok((Self { context, gl }, surface))
    }

    /// Create a surface based on provided window.
    pub fn create_surface(
        &self,
        window: &Window,
    ) -> Result<Surface<WindowSurface>, crate::errors::Error> {
        let attrs = window
            .build_surface_attributes(Default::default())
            .expect("Failed to build surface attributes");
        let config = self.context.config();
        unsafe { Ok(config.display().create_window_surface(&config, &attrs)?) }
    }

    /// Make GL context current.
    pub fn make_gl_context_current(
        &self,
        surface: &Surface<impl SurfaceTypeTrait>,
    ) -> Result<(), crate::errors::Error> {
        self.context.make_current(surface)?;
        Ok(())
    }

    /// Resize the rendering context.
    pub fn resize(
        &self,
        surface: &Surface<impl SurfaceTypeTrait + ResizeableSurface>,
        size: Size2D<i32>,
    ) {
        surface.resize(
            &self.context,
            NonZeroU32::new(size.width as u32).unwrap(),
            NonZeroU32::new(size.height as u32).unwrap(),
        );
        self.gl.viewport(0, 0, size.width, size.height);
    }

    /// Present the surface of the rendering context.
    pub fn present(
        &self,
        surface: &Surface<impl SurfaceTypeTrait>,
    ) -> Result<(), crate::errors::Error> {
        self.context.make_current(&surface)?;
        surface.swap_buffers(&self.context)?;
        Ok(())
    }
}

/// Find the config with the maximum number of samples, so our triangle will be
/// smooth.
pub fn gl_config_picker(configs: Box<dyn Iterator<Item = Config> + '_>) -> Config {
    configs
        .reduce(|accum, config| {
            let transparency_check = config.supports_transparency().unwrap_or(false) &
                !accum.supports_transparency().unwrap_or(false);

            if transparency_check || config.num_samples() > accum.num_samples() {
                config
            } else {
                accum
            }
        })
        .unwrap()
}

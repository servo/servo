/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! A headless window implementation.

use std::cell::Cell;
use std::rc::Rc;
use std::sync::RwLock;

use euclid::{Length, Point2D, Rotation3D, Scale, Size2D, UnknownUnit, Vector3D};
use log::warn;
use servo::compositing::windowing::{
    AnimationState, EmbedderCoordinates, EmbedderEvent, WindowMethods,
};
use servo::rendering_context::RenderingContext;
use servo::servo_geometry::DeviceIndependentPixel;
use servo::style_traits::DevicePixel;
use servo::webrender_api::units::{DeviceIntRect, DeviceIntSize};
use surfman::{Connection, Context, Device, SurfaceType};

use crate::events_loop::WakerEvent;
use crate::window_trait::WindowPortsMethods;

pub struct Window {
    rendering_context: RenderingContext,
    animation_state: Cell<AnimationState>,
    fullscreen: Cell<bool>,
    device_pixel_ratio_override: Option<f32>,
    inner_size: Cell<Size2D<i32, UnknownUnit>>,
    event_queue: RwLock<Vec<EmbedderEvent>>,
}

impl Window {
    pub fn new(
        size: Size2D<u32, DeviceIndependentPixel>,
        device_pixel_ratio_override: Option<f32>,
    ) -> Rc<dyn WindowPortsMethods> {
        // Initialize surfman
        let connection = Connection::new().expect("Failed to create connection");
        let adapter = connection
            .create_software_adapter()
            .expect("Failed to create adapter");
        let size = size.to_untyped().to_i32();
        let surface_type = SurfaceType::Generic { size };
        let rendering_context = RenderingContext::create(&connection, &adapter, surface_type)
            .expect("Failed to create WR surfman");

        let window = Window {
            rendering_context,
            animation_state: Cell::new(AnimationState::Idle),
            fullscreen: Cell::new(false),
            device_pixel_ratio_override,
            inner_size: Cell::new(size.to_i32()),
            event_queue: RwLock::new(Vec::new()),
        };

        Rc::new(window)
    }
}

impl WindowPortsMethods for Window {
    fn get_events(&self) -> Vec<EmbedderEvent> {
        match self.event_queue.write() {
            Ok(ref mut event_queue) => std::mem::take(event_queue),
            Err(_) => vec![],
        }
    }

    fn has_events(&self) -> bool {
        self.event_queue
            .read()
            .ok()
            .map(|queue| !queue.is_empty())
            .unwrap_or(false)
    }

    fn id(&self) -> winit::window::WindowId {
        unsafe { winit::window::WindowId::dummy() }
    }

    fn set_inner_size(&self, size: DeviceIntSize) {
        let (width, height) = size.into();

        // Surfman doesn't support zero-sized surfaces.
        let new_size = Size2D::new(width.max(1), height.max(1));
        if self.inner_size.get() == new_size {
            return;
        }

        match self.rendering_context.resize(new_size.to_untyped()) {
            Ok(()) => {
                self.inner_size.set(new_size);
                if let Ok(ref mut queue) = self.event_queue.write() {
                    queue.push(EmbedderEvent::Resize);
                }
            },
            Err(error) => warn!("Could not resize window: {error:?}"),
        }
    }

    fn device_hidpi_factor(&self) -> Scale<f32, DeviceIndependentPixel, DevicePixel> {
        Scale::new(1.0)
    }

    fn device_pixel_ratio_override(
        &self,
    ) -> Option<Scale<f32, DeviceIndependentPixel, DevicePixel>> {
        self.device_pixel_ratio_override.map(Scale::new)
    }

    fn page_height(&self) -> f32 {
        let height = self
            .rendering_context
            .context_surface_info()
            .unwrap_or(None)
            .map(|info| info.size.height)
            .unwrap_or(0);
        let dpr = self.hidpi_factor();
        height as f32 * dpr.get()
    }

    fn set_fullscreen(&self, state: bool) {
        self.fullscreen.set(state);
    }

    fn get_fullscreen(&self) -> bool {
        self.fullscreen.get()
    }

    fn is_animating(&self) -> bool {
        self.animation_state.get() == AnimationState::Animating
    }

    fn queue_embedder_events_for_winit_event(&self, _event: winit::event::WindowEvent<'_>) {
        // Not expecting any winit events.
    }

    fn new_glwindow(
        &self,
        _events_loop: &winit::event_loop::EventLoopWindowTarget<WakerEvent>,
    ) -> Box<dyn webxr::glwindow::GlWindow> {
        unimplemented!()
    }

    fn winit_window(&self) -> Option<&winit::window::Window> {
        None
    }

    fn set_toolbar_height(&self, _height: Length<f32, DeviceIndependentPixel>) {
        unimplemented!("headless Window only")
    }
}

impl WindowMethods for Window {
    fn get_coordinates(&self) -> EmbedderCoordinates {
        let dpr = self.hidpi_factor();
        let size = self
            .rendering_context
            .context_surface_info()
            .unwrap_or(None)
            .map(|info| Size2D::from_untyped(info.size))
            .unwrap_or(Size2D::new(0, 0));
        let viewport = DeviceIntRect::from_origin_and_size(Point2D::zero(), size);
        EmbedderCoordinates {
            viewport,
            framebuffer: size,
            window: (size, Point2D::zero()),
            screen: size,
            screen_avail: size,
            hidpi_factor: dpr,
        }
    }

    fn set_animation_state(&self, state: AnimationState) {
        self.animation_state.set(state);
    }

    fn rendering_context(&self) -> RenderingContext {
        self.rendering_context.clone()
    }
}

impl webxr::glwindow::GlWindow for Window {
    fn get_render_target(
        &self,
        _device: &mut Device,
        _context: &mut Context,
    ) -> webxr::glwindow::GlWindowRenderTarget {
        unimplemented!()
    }

    fn get_rotation(&self) -> Rotation3D<f32, UnknownUnit, UnknownUnit> {
        Rotation3D::identity()
    }

    fn get_translation(&self) -> Vector3D<f32, UnknownUnit> {
        Vector3D::zero()
    }
}

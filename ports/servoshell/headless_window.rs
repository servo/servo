/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! A headless window implementation.

use std::cell::Cell;
use std::rc::Rc;

use euclid::{Length, Point2D, Rotation3D, Scale, Size2D, UnknownUnit, Vector3D};
use servo::compositing::windowing::{
    AnimationState, EmbedderCoordinates, EmbedderEvent, WindowMethods,
};
use servo::servo_geometry::DeviceIndependentPixel;
use servo::style_traits::DevicePixel;
use servo::webrender_api::units::DeviceIntRect;
use servo::webrender_surfman::WebrenderSurfman;
use servo_media::player::context as MediaPlayerCtxt;
use surfman::{Connection, Context, Device, SurfaceType};

use crate::events_loop::WakerEvent;
use crate::window_trait::WindowPortsMethods;

pub struct Window {
    webrender_surfman: WebrenderSurfman,
    animation_state: Cell<AnimationState>,
    fullscreen: Cell<bool>,
    device_pixel_ratio_override: Option<f32>,
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
        let webrender_surfman = WebrenderSurfman::create(&connection, &adapter, surface_type)
            .expect("Failed to create WR surfman");

        let window = Window {
            webrender_surfman,
            animation_state: Cell::new(AnimationState::Idle),
            fullscreen: Cell::new(false),
            device_pixel_ratio_override,
        };

        Rc::new(window)
    }
}

impl WindowPortsMethods for Window {
    fn get_events(&self) -> Vec<EmbedderEvent> {
        vec![]
    }

    fn has_events(&self) -> bool {
        false
    }

    fn id(&self) -> winit::window::WindowId {
        unsafe { winit::window::WindowId::dummy() }
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
            .webrender_surfman
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
            .webrender_surfman
            .context_surface_info()
            .unwrap_or(None)
            .map(|info| Size2D::from_untyped(info.size))
            .unwrap_or(Size2D::new(0, 0));
        let viewport = DeviceIntRect::new(Point2D::zero(), size);
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

    fn get_gl_context(&self) -> MediaPlayerCtxt::GlContext {
        MediaPlayerCtxt::GlContext::Unknown
    }

    fn get_native_display(&self) -> MediaPlayerCtxt::NativeDisplay {
        MediaPlayerCtxt::NativeDisplay::Unknown
    }

    fn get_gl_api(&self) -> MediaPlayerCtxt::GlApi {
        MediaPlayerCtxt::GlApi::None
    }

    fn webrender_surfman(&self) -> WebrenderSurfman {
        self.webrender_surfman.clone()
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

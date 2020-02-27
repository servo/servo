/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Implements the global methods required by Servo (not window/gl/compositor related).

use crate::app;
use crate::events_loop::EventsLoop;
use crate::window_trait::WindowPortsMethods;
use gleam::gl;
use glutin;
use glutin::dpi::LogicalSize;
use glutin::EventsLoopClosed;
use rust_webvr::GlWindowVRService;
use servo::canvas::{SurfaceProviders, WebGlExecutor};
use servo::compositing::windowing::EmbedderMethods;
use servo::embedder_traits::EventLoopWaker;
use servo::servo_config::{opts, pref};
use servo::webvr::VRServiceManager;
use servo::webvr_traits::WebVRMainThreadHeartbeat;
use std::cell::RefCell;
use std::rc::Rc;

pub struct EmbedderCallbacks {
    window: Rc<dyn WindowPortsMethods>,
    events_loop: Rc<RefCell<EventsLoop>>,
    gl: Rc<dyn gl::Gl>,
    angle: bool,
}

impl EmbedderCallbacks {
    pub fn new(
        window: Rc<dyn WindowPortsMethods>,
        events_loop: Rc<RefCell<EventsLoop>>,
        gl: Rc<dyn gl::Gl>,
        angle: bool,
    ) -> EmbedderCallbacks {
        EmbedderCallbacks {
            window,
            events_loop,
            gl,
            angle,
        }
    }
}

impl EmbedderMethods for EmbedderCallbacks {
    fn create_event_loop_waker(&mut self) -> Box<dyn EventLoopWaker> {
        self.events_loop.borrow().create_event_loop_waker()
    }

    fn register_vr_services(
        &mut self,
        services: &mut VRServiceManager,
        heartbeats: &mut Vec<Box<dyn WebVRMainThreadHeartbeat>>,
    ) {
        if !opts::get().headless {
            if pref!(dom.webvr.test) {
                warn!("Creating test VR display");
                // This is safe, because register_vr_services is called from the main thread.
                let name = String::from("Test VR Display");
                let size = opts::get().initial_window_size.to_f64();
                let size = LogicalSize::new(size.width, size.height);
                let events_loop_clone = self.events_loop.clone();
                let events_loop_factory = Box::new(move || {
                    events_loop_clone
                        .borrow_mut()
                        .take()
                        .ok_or(EventsLoopClosed)
                });
                let window_builder = glutin::WindowBuilder::new()
                    .with_title(name.clone())
                    .with_dimensions(size)
                    .with_visibility(false)
                    .with_multitouch();
                let context = glutin::ContextBuilder::new()
                    .with_gl(app::gl_version(self.angle))
                    .with_vsync(false) // Assume the browser vsync is the same as the test VR window vsync
                    .build_windowed(window_builder, &*self.events_loop.borrow().as_winit())
                    .expect("Failed to create window.");
                let gl = self.gl.clone();
                let (service, heartbeat) =
                    GlWindowVRService::new(name, context, events_loop_factory, gl);

                services.register(Box::new(service));
                heartbeats.push(Box::new(heartbeat));
            }
        } else {
            // FIXME: support headless mode
        }
    }

    fn register_webxr(
        &mut self,
        xr: &mut webxr::MainThreadRegistry,
        _executor: WebGlExecutor,
        _surface_provider_registration: SurfaceProviders
    ) {
        if pref!(dom.webxr.test) {
            xr.register_mock(webxr::headless::HeadlessMockDiscovery::new());
        } else if !opts::get().headless && pref!(dom.webxr.glwindow) {
            warn!("Creating test XR device");
            let gl = self.gl.clone();
            let window = self.window.clone();
            let factory = Box::new(move || window.new_window());
            let discovery = webxr::glwindow::GlWindowDiscovery::new(gl, factory);
            xr.register(discovery);
        }
    }
}

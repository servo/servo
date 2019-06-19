/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Implements the global methods required by Servo (not window/gl/compositor related).

use crate::app;
use crate::events_loop::EventsLoop;
use gleam::gl;
use glutin;
use glutin::dpi::LogicalSize;
use rust_webvr::GlWindowVRService;
use servo::compositing::windowing::EmbedderMethods;
use servo::embedder_traits::EventLoopWaker;
use servo::servo_config::{opts, pref};
use servo::webvr::VRServiceManager;
use servo::webvr_traits::WebVRMainThreadHeartbeat;
use std::cell::RefCell;
use std::rc::Rc;

pub struct EmbedderCallbacks {
    events_loop: Rc<RefCell<EventsLoop>>,
    gl: Rc<dyn gl::Gl>,
}

impl EmbedderCallbacks {
    pub fn new(events_loop: Rc<RefCell<EventsLoop>>, gl: Rc<dyn gl::Gl>) -> EmbedderCallbacks {
        EmbedderCallbacks { events_loop, gl }
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
                let window_builder = glutin::WindowBuilder::new()
                    .with_title(name.clone())
                    .with_dimensions(size)
                    .with_visibility(false)
                    .with_multitouch();
                let context = glutin::ContextBuilder::new()
                    .with_gl(app::gl_version())
                    .with_vsync(false) // Assume the browser vsync is the same as the test VR window vsync
                    .build_windowed(window_builder, &*self.events_loop.borrow().as_winit())
                    .expect("Failed to create window.");
                let gl = self.gl.clone();
                let (service, heartbeat) = GlWindowVRService::new(name, context, gl);

                services.register(Box::new(service));
                heartbeats.push(Box::new(heartbeat));
            }
        } else {
            // FIXME: support headless mode
        }
    }
}

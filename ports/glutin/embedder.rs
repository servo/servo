/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Implements the global methods required by Servo (not window/gl/compositor related).

use crate::events_loop::EventsLoop;
use crate::window_trait::WindowPortsMethods;
use gleam::gl;
use servo::canvas::{SurfaceProviders, WebGlExecutor};
use servo::compositing::windowing::EmbedderMethods;
use servo::embedder_traits::{EmbedderProxy, EventLoopWaker};
use servo::servo_config::{opts, pref};
use std::cell::RefCell;
use std::rc::Rc;

pub struct EmbedderCallbacks {
    window: Rc<dyn WindowPortsMethods>,
    events_loop: Rc<RefCell<EventsLoop>>,
    gl: Rc<dyn gl::Gl>,
}

impl EmbedderCallbacks {
    pub fn new(
        window: Rc<dyn WindowPortsMethods>,
        events_loop: Rc<RefCell<EventsLoop>>,
        gl: Rc<dyn gl::Gl>,
    ) -> EmbedderCallbacks {
        EmbedderCallbacks {
            window,
            events_loop,
            gl,
        }
    }
}

impl EmbedderMethods for EmbedderCallbacks {
    fn create_event_loop_waker(&mut self) -> Box<dyn EventLoopWaker> {
        self.events_loop.borrow().create_event_loop_waker()
    }

    fn register_webxr(
        &mut self,
        xr: &mut webxr::MainThreadRegistry,
        _executor: WebGlExecutor,
        _surface_provider_registration: SurfaceProviders,
        _embedder_proxy: EmbedderProxy,
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

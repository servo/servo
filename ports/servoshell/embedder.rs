/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Implements the global methods required by Servo (not window/gl/compositor related).

use servo::compositing::windowing::EmbedderMethods;
use servo::embedder_traits::{EmbedderProxy, EventLoopWaker};
use servo::servo_config::pref;
use webxr::glwindow::GlWindowDiscovery;

pub struct EmbedderCallbacks {
    event_loop_waker: Box<dyn EventLoopWaker>,
    xr_discovery: Option<GlWindowDiscovery>,
}

impl EmbedderCallbacks {
    pub fn new(
        event_loop_waker: Box<dyn EventLoopWaker>,
        xr_discovery: Option<GlWindowDiscovery>,
    ) -> EmbedderCallbacks {
        EmbedderCallbacks {
            event_loop_waker,
            xr_discovery,
        }
    }
}

impl EmbedderMethods for EmbedderCallbacks {
    fn create_event_loop_waker(&mut self) -> Box<dyn EventLoopWaker> {
        self.event_loop_waker.clone()
    }

    fn register_webxr(
        &mut self,
        xr: &mut webxr::MainThreadRegistry,
        _embedder_proxy: EmbedderProxy,
    ) {
        if pref!(dom.webxr.test) {
            xr.register_mock(webxr::headless::HeadlessMockDiscovery::new());
        } else if let Some(xr_discovery) = self.xr_discovery.take() {
            xr.register(xr_discovery);
        }
    }
}

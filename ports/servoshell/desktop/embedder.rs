/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Implements the global methods required by Servo (not window/gl/compositor related).

use servo::compositing::windowing::EmbedderMethods;
use servo::embedder_traits::{EmbedderProxy, EventLoopWaker};
use servo::servo_config::pref;
use webxr::glwindow::GlWindowDiscovery;
#[cfg(target_os = "windows")]
use webxr::openxr::OpenXrDiscovery;

pub enum XrDiscovery {
    GlWindow(GlWindowDiscovery),
    #[cfg(target_os = "windows")]
    OpenXr(OpenXrDiscovery),
}

pub struct EmbedderCallbacks {
    event_loop_waker: Box<dyn EventLoopWaker>,
    xr_discovery: Option<XrDiscovery>,
}

impl EmbedderCallbacks {
    pub fn new(
        event_loop_waker: Box<dyn EventLoopWaker>,
        xr_discovery: Option<XrDiscovery>,
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
            match xr_discovery {
                XrDiscovery::GlWindow(discovery) => xr.register(discovery),
                #[cfg(target_os = "windows")]
                XrDiscovery::OpenXr(discovery) => xr.register(discovery),
            }
        }
    }
}

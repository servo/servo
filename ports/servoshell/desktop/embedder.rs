/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Implements the global methods required by Servo (not window/gl/compositor related).

use net::protocols::ProtocolRegistry;
use servo::compositing::windowing::EmbedderMethods;
use servo::servo_config::pref;
use servo::webxr::glwindow::GlWindowDiscovery;
#[cfg(target_os = "windows")]
use servo::webxr::openxr::OpenXrDiscovery;
use servo::{EmbedderProxy, EventLoopWaker};

use crate::desktop::protocols::{resource, servo as servo_handler, urlinfo};

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

    #[cfg(feature = "webxr")]
    fn register_webxr(
        &mut self,
        xr: &mut servo::webxr::MainThreadRegistry,
        _embedder_proxy: EmbedderProxy,
    ) {
        use servo::webxr::headless::HeadlessMockDiscovery;

        if pref!(dom_webxr_test) {
            xr.register_mock(HeadlessMockDiscovery::default());
        } else if let Some(xr_discovery) = self.xr_discovery.take() {
            match xr_discovery {
                XrDiscovery::GlWindow(discovery) => xr.register(discovery),
                #[cfg(target_os = "windows")]
                XrDiscovery::OpenXr(discovery) => xr.register(discovery),
            }
        }
    }

    fn get_protocol_handlers(&self) -> ProtocolRegistry {
        let mut registry = ProtocolRegistry::default();
        registry.register("urlinfo", urlinfo::UrlInfoProtocolHander::default());
        registry.register("servo", servo_handler::ServoProtocolHandler::default());
        registry.register("resource", resource::ResourceProtocolHandler::default());
        registry
    }

    fn get_version_string(&self) -> Option<String> {
        crate::servo_version().into()
    }
}

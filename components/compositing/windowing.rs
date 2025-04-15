/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Abstract windowing methods. The concrete implementations of these can be found in `platform/`.

use embedder_traits::EventLoopWaker;
use net::protocols::ProtocolRegistry;

/// Various debug and profiling flags that WebRender supports.
#[derive(Clone)]
pub enum WebRenderDebugOption {
    Profiler,
    TextureCacheDebug,
    RenderTargetDebug,
}

pub trait EmbedderMethods {
    /// Returns a thread-safe object to wake up the window's event loop.
    fn create_event_loop_waker(&mut self) -> Box<dyn EventLoopWaker>;

    #[cfg(feature = "webxr")]
    /// Register services with a WebXR Registry.
    fn register_webxr(
        &mut self,
        _: &mut webxr::MainThreadRegistry,
        _: embedder_traits::EmbedderProxy,
    ) {
    }

    /// Returns the protocol handlers implemented by that embedder.
    /// They will be merged with the default internal ones.
    fn get_protocol_handlers(&self) -> ProtocolRegistry {
        ProtocolRegistry::default()
    }
}

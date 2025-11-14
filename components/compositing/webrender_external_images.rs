/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::rc::Rc;

use canvas_traits::webgl::WebGLContextId;
use compositing_traits::rendering_context::RenderingContext;
use compositing_traits::{ExternalImageSource, WebRenderExternalImageApi};
use euclid::default::Size2D;
use log::debug;
use rustc_hash::FxHashMap;
use surfman::chains::{SwapChainAPI, SwapChains, SwapChainsAPI};
use surfman::{Device, SurfaceTexture};

/// Bridge between the webrender::ExternalImage callbacks and the WebGLThreads.
pub struct WebGLExternalImages {
    rendering_context: Rc<dyn RenderingContext>,
    swap_chains: SwapChains<WebGLContextId, Device>,
    locked_front_buffers: FxHashMap<WebGLContextId, SurfaceTexture>,
}

impl WebGLExternalImages {
    pub fn new(
        rendering_context: Rc<dyn RenderingContext>,
        swap_chains: SwapChains<WebGLContextId, Device>,
    ) -> Self {
        Self {
            rendering_context,
            swap_chains,
            locked_front_buffers: FxHashMap::default(),
        }
    }

    fn lock_swap_chain(&mut self, id: WebGLContextId) -> Option<(u32, Size2D<i32>)> {
        debug!("... locking chain {:?}", id);
        let front_buffer = self.swap_chains.get(id)?.take_surface()?;
        let (surface_texture, gl_texture, size) =
            self.rendering_context.create_texture(front_buffer)?;
        self.locked_front_buffers.insert(id, surface_texture);

        Some((gl_texture, size))
    }

    fn unlock_swap_chain(&mut self, id: WebGLContextId) -> Option<()> {
        debug!("... unlocked chain {:?}", id);
        let locked_front_buffer = self.locked_front_buffers.remove(&id)?;
        let locked_front_buffer = self
            .rendering_context
            .destroy_texture(locked_front_buffer)?;

        // TODO: This has the potential to drop a surface without calling `destroy`, if
        // the WebGLThread has already cleaned up the SwapChain. It should probably just
        // ask the WebGLThread to destroy the surface.
        self.swap_chains
            .get(id)?
            .recycle_surface(locked_front_buffer);
        Some(())
    }
}

impl WebRenderExternalImageApi for WebGLExternalImages {
    fn lock(&mut self, id: u64) -> (ExternalImageSource<'_>, Size2D<i32>) {
        let (texture_id, size) = self.lock_swap_chain(WebGLContextId(id)).unwrap_or_default();
        (ExternalImageSource::NativeTexture(texture_id), size)
    }

    fn unlock(&mut self, id: u64) {
        self.unlock_swap_chain(WebGLContextId(id));
    }
}

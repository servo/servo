/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::default::Default;
use std::sync::{Arc, Mutex};

use canvas_traits::webgl::{webgl_channel, WebGLContextId, WebGLMsg, WebGLThreads};
use euclid::default::Size2D;
use fnv::FnvHashMap;
use log::debug;
use sparkle::gl::GlType;
use surfman::chains::{SwapChainAPI, SwapChains, SwapChainsAPI};
use surfman::{Device, SurfaceInfo, SurfaceTexture};
use webrender::RenderApiSender;
use webrender_api::DocumentId;
use webrender_traits::{
    RenderingContext, WebrenderExternalImageApi, WebrenderExternalImageRegistry,
    WebrenderImageSource,
};
use webxr::SurfmanGL as WebXRSurfman;
use webxr_api::LayerGrandManager as WebXRLayerGrandManager;

use crate::webgl_thread::{WebGLThread, WebGLThreadInit, WebXRBridgeInit};

pub struct WebGLComm {
    pub webgl_threads: WebGLThreads,
    pub image_handler: Box<dyn WebrenderExternalImageApi>,
    pub webxr_layer_grand_manager: WebXRLayerGrandManager<WebXRSurfman>,
}

impl WebGLComm {
    /// Creates a new `WebGLComm` object.
    pub fn new(
        surfman: RenderingContext,
        webrender_api_sender: RenderApiSender,
        webrender_doc: DocumentId,
        external_images: Arc<Mutex<WebrenderExternalImageRegistry>>,
        api_type: GlType,
    ) -> WebGLComm {
        debug!("WebGLThreads::new()");
        let (sender, receiver) = webgl_channel::<WebGLMsg>().unwrap();
        let webrender_swap_chains = SwapChains::new();
        let webxr_init = WebXRBridgeInit::new(sender.clone());
        let webxr_layer_grand_manager = webxr_init.layer_grand_manager();

        // This implementation creates a single `WebGLThread` for all the pipelines.
        let init = WebGLThreadInit {
            webrender_api_sender,
            webrender_doc,
            external_images,
            sender: sender.clone(),
            receiver,
            webrender_swap_chains: webrender_swap_chains.clone(),
            connection: surfman.connection(),
            adapter: surfman.adapter(),
            api_type,
            webxr_init,
        };

        let external = WebGLExternalImages::new(surfman, webrender_swap_chains);

        WebGLThread::run_on_own_thread(init);

        WebGLComm {
            webgl_threads: WebGLThreads(sender),
            image_handler: Box::new(external),
            webxr_layer_grand_manager,
        }
    }
}

/// Bridge between the webrender::ExternalImage callbacks and the WebGLThreads.
struct WebGLExternalImages {
    surfman: RenderingContext,
    swap_chains: SwapChains<WebGLContextId, Device>,
    locked_front_buffers: FnvHashMap<WebGLContextId, SurfaceTexture>,
}

impl WebGLExternalImages {
    fn new(surfman: RenderingContext, swap_chains: SwapChains<WebGLContextId, Device>) -> Self {
        Self {
            surfman,
            swap_chains,
            locked_front_buffers: FnvHashMap::default(),
        }
    }

    fn lock_swap_chain(&mut self, id: WebGLContextId) -> Option<(u32, Size2D<i32>)> {
        debug!("... locking chain {:?}", id);
        let front_buffer = self.swap_chains.get(id)?.take_surface()?;

        let SurfaceInfo {
            id: front_buffer_id,
            size,
            ..
        } = self.surfman.surface_info(&front_buffer);
        debug!("... getting texture for surface {:?}", front_buffer_id);
        let front_buffer_texture = self.surfman.create_surface_texture(front_buffer).unwrap();
        let gl_texture = self.surfman.surface_texture_object(&front_buffer_texture);

        self.locked_front_buffers.insert(id, front_buffer_texture);

        Some((gl_texture, size))
    }

    fn unlock_swap_chain(&mut self, id: WebGLContextId) -> Option<()> {
        let locked_front_buffer = self.locked_front_buffers.remove(&id)?;
        let locked_front_buffer = self
            .surfman
            .destroy_surface_texture(locked_front_buffer)
            .unwrap();

        debug!("... unlocked chain {:?}", id);
        self.swap_chains
            .get(id)?
            .recycle_surface(locked_front_buffer);
        Some(())
    }
}

impl WebrenderExternalImageApi for WebGLExternalImages {
    fn lock(&mut self, id: u64) -> (WebrenderImageSource, Size2D<i32>) {
        let id = WebGLContextId(id);
        let (texture_id, size) = self.lock_swap_chain(id).unwrap_or_default();
        (WebrenderImageSource::TextureHandle(texture_id), size)
    }

    fn unlock(&mut self, id: u64) {
        let id = WebGLContextId(id);
        self.unlock_swap_chain(id);
    }
}

/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::default::Default;
use std::rc::Rc;
use std::sync::{Arc, Mutex};

use canvas_traits::webgl::{webgl_channel, GlType, WebGLContextId, WebGLMsg, WebGLThreads};
use euclid::default::Size2D;
use fnv::FnvHashMap;
use log::debug;
use surfman::chains::{SwapChainAPI, SwapChains, SwapChainsAPI};
use surfman::{Device, SurfaceTexture};
use webrender::RenderApiSender;
use webrender_api::DocumentId;
use webrender_traits::rendering_context::RenderingContext;
use webrender_traits::{
    WebrenderExternalImageApi, WebrenderExternalImageRegistry, WebrenderImageSource,
};
#[cfg(feature = "webxr")]
use webxr::SurfmanGL as WebXRSurfman;
#[cfg(feature = "webxr")]
use webxr_api::LayerGrandManager as WebXRLayerGrandManager;

use crate::webgl_thread::{WebGLThread, WebGLThreadInit};

pub struct WebGLComm {
    pub webgl_threads: WebGLThreads,
    pub image_handler: Box<dyn WebrenderExternalImageApi>,
    #[cfg(feature = "webxr")]
    pub webxr_layer_grand_manager: WebXRLayerGrandManager<WebXRSurfman>,
}

impl WebGLComm {
    /// Creates a new `WebGLComm` object.
    pub fn new(
        rendering_context: Rc<dyn RenderingContext>,
        webrender_api_sender: RenderApiSender,
        webrender_doc: DocumentId,
        external_images: Arc<Mutex<WebrenderExternalImageRegistry>>,
        api_type: GlType,
    ) -> WebGLComm {
        debug!("WebGLThreads::new()");
        let (sender, receiver) = webgl_channel::<WebGLMsg>().unwrap();
        let webrender_swap_chains = SwapChains::new();
        #[cfg(feature = "webxr")]
        let webxr_init = crate::webxr::WebXRBridgeInit::new(sender.clone());
        #[cfg(feature = "webxr")]
        let webxr_layer_grand_manager = webxr_init.layer_grand_manager();
        let connection = rendering_context
            .connection()
            .expect("Failed to get connection");
        let adapter = connection
            .create_adapter()
            .expect("Failed to create adapter");

        // This implementation creates a single `WebGLThread` for all the pipelines.
        let init = WebGLThreadInit {
            webrender_api_sender,
            webrender_doc,
            external_images,
            sender: sender.clone(),
            receiver,
            webrender_swap_chains: webrender_swap_chains.clone(),
            connection,
            adapter,
            api_type,
            #[cfg(feature = "webxr")]
            webxr_init,
        };

        let external = WebGLExternalImages::new(rendering_context, webrender_swap_chains);

        WebGLThread::run_on_own_thread(init);

        WebGLComm {
            webgl_threads: WebGLThreads(sender),
            image_handler: Box::new(external),
            #[cfg(feature = "webxr")]
            webxr_layer_grand_manager,
        }
    }
}

/// Bridge between the webrender::ExternalImage callbacks and the WebGLThreads.
struct WebGLExternalImages {
    rendering_context: Rc<dyn RenderingContext>,
    swap_chains: SwapChains<WebGLContextId, Device>,
    locked_front_buffers: FnvHashMap<WebGLContextId, SurfaceTexture>,
}

impl WebGLExternalImages {
    fn new(
        rendering_context: Rc<dyn RenderingContext>,
        swap_chains: SwapChains<WebGLContextId, Device>,
    ) -> Self {
        Self {
            rendering_context,
            swap_chains,
            locked_front_buffers: FnvHashMap::default(),
        }
    }

    fn lock_swap_chain(&mut self, id: WebGLContextId) -> Option<(u32, Size2D<i32>)> {
        debug!("... locking chain {:?}", id);
        let front_buffer = self.swap_chains.get(id)?.take_surface()?;

        if let Some((surface_texture, gl_texture, size)) =
            self.rendering_context.create_texture(front_buffer)
        {
            self.locked_front_buffers.insert(id, surface_texture);

            Some((gl_texture, size))
        } else {
            None
        }
    }

    fn unlock_swap_chain(&mut self, id: WebGLContextId) -> Option<()> {
        debug!("... unlocked chain {:?}", id);
        let locked_front_buffer = self.locked_front_buffers.remove(&id)?;
        if let Some(locked_front_buffer) =
            self.rendering_context.destroy_texture(locked_front_buffer)
        {
            self.swap_chains
                .get(id)?
                .recycle_surface(locked_front_buffer);
            Some(())
        } else {
            None
        }
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

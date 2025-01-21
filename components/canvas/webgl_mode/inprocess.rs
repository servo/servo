/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::default::Default;
use std::rc::Rc;
use std::sync::{Arc, Mutex};

use canvas_traits::webgl::{webgl_channel, GlType, WebGLContextId, WebGLMsg, WebGLThreads};
use euclid::default::Size2D;
use fnv::FnvHashMap;
use log::{debug, warn};
use surfman::chains::{SwapChainAPI, SwapChains, SwapChainsAPI};
use surfman::{Context, Device, SurfaceInfo, SurfaceTexture};
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

        // This implementation creates a single `WebGLThread` for all the pipelines.
        let init = WebGLThreadInit {
            webrender_api_sender,
            webrender_doc,
            external_images,
            sender: sender.clone(),
            receiver,
            webrender_swap_chains: webrender_swap_chains.clone(),
            connection: rendering_context.connection(),
            adapter: rendering_context.adapter(),
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
    device: Device,
    context: Context,
    swap_chains: SwapChains<WebGLContextId, Device>,
    locked_front_buffers: FnvHashMap<WebGLContextId, SurfaceTexture>,
}

impl WebGLExternalImages {
    #[allow(unsafe_code)]
    fn new(
        rendering_context: Rc<dyn RenderingContext>,
        swap_chains: SwapChains<WebGLContextId, Device>,
    ) -> Self {
        unsafe {
            let device = rendering_context
                .connection()
                .create_device_from_native_device(rendering_context.device())
                .unwrap();
            let context = device
                .create_context_from_native_context(rendering_context.context())
                .unwrap();
            Self {
                device,
                context,
                swap_chains,
                locked_front_buffers: FnvHashMap::default(),
            }
        }
    }

    fn lock_swap_chain(&mut self, id: WebGLContextId) -> Option<(u32, Size2D<i32>)> {
        debug!("... locking chain {:?}", id);
        let front_buffer = self.swap_chains.get(id)?.take_surface()?;

        let SurfaceInfo {
            id: front_buffer_id,
            size,
            ..
        } = self.device.surface_info(&front_buffer);
        debug!("... getting texture for surface {:?}", front_buffer_id);
        let front_buffer_texture = self
            .device
            .create_surface_texture(&mut self.context, front_buffer)
            .unwrap();
        let gl_texture = self.device.surface_texture_object(&front_buffer_texture);

        self.locked_front_buffers.insert(id, front_buffer_texture);

        Some((gl_texture, size))
    }

    fn unlock_swap_chain(&mut self, id: WebGLContextId) -> Option<()> {
        let locked_front_buffer = self.locked_front_buffers.remove(&id)?;
        let locked_front_buffer = self
            .device
            .destroy_surface_texture(&mut self.context, locked_front_buffer)
            .unwrap();

        debug!("... unlocked chain {:?}", id);
        self.swap_chains
            .get(id)?
            .recycle_surface(locked_front_buffer);
        Some(())
    }
}

impl Drop for WebGLExternalImages {
    fn drop(&mut self) {
        if let Err(err) = self.device.destroy_context(&mut self.context) {
            warn!("Failed to destroy context: {:?}", err);
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

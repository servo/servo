/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::webgl_thread::{WebGLThread, WebGLThreadInit, WebXRBridgeInit};
use canvas_traits::webgl::webgl_channel;
use canvas_traits::webgl::{WebGLContextId, WebGLMsg, WebGLThreads};
use euclid::default::Size2D;
use fnv::FnvHashMap;
use gleam;
use servo_config::pref;
use sparkle::gl;
use sparkle::gl::GlType;
use std::default::Default;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use surfman::Device;
use surfman::SurfaceInfo;
use surfman::SurfaceTexture;
use surfman_chains::SwapChains;
use surfman_chains_api::SwapChainAPI;
use surfman_chains_api::SwapChainsAPI;
use webrender_surfman::WebrenderSurfman;
use webrender_traits::{
    WebrenderExternalImageApi, WebrenderExternalImageRegistry, WebrenderImageSource,
};
use webxr::SurfmanGL as WebXRSurfman;
use webxr_api::LayerGrandManager as WebXRLayerGrandManager;

pub struct WebGLComm {
    pub webgl_threads: WebGLThreads,
    pub image_handler: Box<dyn WebrenderExternalImageApi>,
    pub output_handler: Option<Box<dyn webrender_api::OutputImageHandler>>,
    pub webxr_layer_grand_manager: WebXRLayerGrandManager<WebXRSurfman>,
}

impl WebGLComm {
    /// Creates a new `WebGLComm` object.
    pub fn new(
        surfman: WebrenderSurfman,
        webrender_gl: Rc<dyn gleam::gl::Gl>,
        webrender_api_sender: webrender_api::RenderApiSender,
        webrender_doc: webrender_api::DocumentId,
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

        let output_handler = if pref!(dom.webgl.dom_to_texture.enabled) {
            Some(Box::new(OutputHandler::new(webrender_gl.clone())))
        } else {
            None
        };

        let external = WebGLExternalImages::new(surfman, webrender_swap_chains);

        WebGLThread::run_on_own_thread(init);

        WebGLComm {
            webgl_threads: WebGLThreads(sender),
            image_handler: Box::new(external),
            output_handler: output_handler.map(|b| b as Box<_>),
            webxr_layer_grand_manager: webxr_layer_grand_manager,
        }
    }
}

/// Bridge between the webrender::ExternalImage callbacks and the WebGLThreads.
struct WebGLExternalImages {
    surfman: WebrenderSurfman,
    swap_chains: SwapChains<WebGLContextId, Device>,
    locked_front_buffers: FnvHashMap<WebGLContextId, SurfaceTexture>,
}

impl WebGLExternalImages {
    fn new(surfman: WebrenderSurfman, swap_chains: SwapChains<WebGLContextId, Device>) -> Self {
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

/// struct used to implement DOMToTexture feature and webrender::OutputImageHandler trait.
struct OutputHandler {
    webrender_gl: Rc<dyn gleam::gl::Gl>,
    sync_objects: FnvHashMap<webrender_api::PipelineId, gleam::gl::GLsync>,
}

impl OutputHandler {
    fn new(webrender_gl: Rc<dyn gleam::gl::Gl>) -> Self {
        OutputHandler {
            webrender_gl,
            sync_objects: Default::default(),
        }
    }
}

/// Bridge between the WR frame outputs and WebGL to implement DOMToTexture synchronization.
impl webrender_api::OutputImageHandler for OutputHandler {
    fn lock(
        &mut self,
        id: webrender_api::PipelineId,
    ) -> Option<(u32, webrender_api::units::FramebufferIntSize)> {
        // Insert a fence in the WR command queue
        let gl_sync = self
            .webrender_gl
            .fence_sync(gl::SYNC_GPU_COMMANDS_COMPLETE, 0);
        self.sync_objects.insert(id, gl_sync);
        // https://github.com/servo/servo/issues/24615
        None
    }

    fn unlock(&mut self, id: webrender_api::PipelineId) {
        if let Some(gl_sync) = self.sync_objects.remove(&id) {
            // Flush the Sync object into the GPU's command queue to guarantee that it it's signaled.
            self.webrender_gl.flush();
            // Mark the sync object for deletion.
            self.webrender_gl.delete_sync(gl_sync);
        }
    }
}

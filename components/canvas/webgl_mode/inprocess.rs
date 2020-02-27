/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::webgl_thread::{SurfaceProviders, WebGLThread, WebGLThreadInit, WebGlExecutor};
use canvas_traits::webgl::{webgl_channel, WebVRRenderHandler};
use canvas_traits::webgl::{WebGLContextId, WebGLMsg, WebGLThreads};
use euclid::default::Size2D;
use fnv::FnvHashMap;
use gleam;
use servo_config::pref;
use sparkle::gl;
use sparkle::gl::GlType;
use std::collections::HashMap;
use std::default::Default;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use surfman::platform::generic::universal::context::Context;
use surfman::platform::generic::universal::device::Device;
use surfman::platform::generic::universal::surface::SurfaceTexture;
use surfman::SurfaceInfo;
use surfman_chains::SwapChains;
use surfman_chains_api::SwapChainAPI;
use surfman_chains_api::SwapChainsAPI;
use webrender_traits::{WebrenderExternalImageApi, WebrenderExternalImageRegistry};
use webxr_api::SwapChainId as WebXRSwapChainId;

pub struct WebGLComm {
    pub webgl_threads: WebGLThreads,
    pub webxr_swap_chains: SwapChains<WebXRSwapChainId>,
    pub webxr_surface_providers: SurfaceProviders,
    pub image_handler: Box<dyn WebrenderExternalImageApi>,
    pub output_handler: Option<Box<dyn webrender_api::OutputImageHandler>>,
    pub webgl_executor: WebGlExecutor,
}

impl WebGLComm {
    /// Creates a new `WebGLComm` object.
    pub fn new(
        device: Device,
        context: Context,
        webrender_gl: Rc<dyn gleam::gl::Gl>,
        webrender_api_sender: webrender_api::RenderApiSender,
        webvr_compositor: Option<Box<dyn WebVRRenderHandler>>,
        external_images: Arc<Mutex<WebrenderExternalImageRegistry>>,
        api_type: GlType,
    ) -> WebGLComm {
        debug!("WebGLThreads::new()");
        let (sender, receiver) = webgl_channel::<WebGLMsg>().unwrap();
        let webrender_swap_chains = SwapChains::new();
        let webxr_swap_chains = SwapChains::new();
        let webxr_surface_providers = Arc::new(Mutex::new(HashMap::new()));
        let (runnable_sender, runnable_receiver) = crossbeam_channel::unbounded();

        // This implementation creates a single `WebGLThread` for all the pipelines.
        let init = WebGLThreadInit {
            webrender_api_sender,
            webvr_compositor,
            external_images,
            sender: sender.clone(),
            receiver,
            webrender_swap_chains: webrender_swap_chains.clone(),
            webxr_swap_chains: webxr_swap_chains.clone(),
            webxr_surface_providers: webxr_surface_providers.clone(),
            connection: device.connection(),
            adapter: device.adapter(),
            api_type,
            runnable_receiver,
        };

        let output_handler = if pref!(dom.webgl.dom_to_texture.enabled) {
            Some(Box::new(OutputHandler::new(webrender_gl.clone())))
        } else {
            None
        };

        let external = WebGLExternalImages::new(device, context, webrender_swap_chains);

        WebGLThread::run_on_own_thread(init);

        WebGLComm {
            webgl_threads: WebGLThreads(sender),
            webxr_swap_chains,
            webxr_surface_providers,
            image_handler: Box::new(external),
            output_handler: output_handler.map(|b| b as Box<_>),
            webgl_executor: runnable_sender,
        }
    }
}

/// Bridge between the webrender::ExternalImage callbacks and the WebGLThreads.
struct WebGLExternalImages {
    device: Device,
    context: Context,
    swap_chains: SwapChains<WebGLContextId>,
    locked_front_buffers: FnvHashMap<WebGLContextId, SurfaceTexture>,
}

impl Drop for WebGLExternalImages {
    fn drop(&mut self) {
        let _ = self.device.destroy_context(&mut self.context);
    }
}

impl WebGLExternalImages {
    fn new(device: Device, context: Context, swap_chains: SwapChains<WebGLContextId>) -> Self {
        Self {
            device,
            context,
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
        } = self.device.surface_info(&front_buffer);
        debug!("... getting texture for surface {:?}", front_buffer_id);
        let front_buffer_texture = self
            .device
            .create_surface_texture(&mut self.context, front_buffer)
            .unwrap();
        let gl_texture = front_buffer_texture.gl_texture();

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

impl WebrenderExternalImageApi for WebGLExternalImages {
    fn lock(&mut self, id: u64) -> (u32, Size2D<i32>) {
        let id = WebGLContextId(id);
        self.lock_swap_chain(id).unwrap_or_default()
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

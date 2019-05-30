/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::gl_context::GLContextFactory;
use crate::webgl_thread::{WebGLExternalImageApi, WebGLExternalImageHandler, WebGLThread};
use canvas_traits::webgl::webgl_channel;
use canvas_traits::webgl::DOMToTextureCommand;
use canvas_traits::webgl::{WebGLChan, WebGLContextId, WebGLMsg, WebGLPipeline, WebGLReceiver};
use canvas_traits::webgl::{WebGLSender, WebVRCommand, WebVRRenderHandler};
use euclid::Size2D;
use fnv::FnvHashMap;
use gl_traits::WebrenderImageHandlersMsg;
use gleam::gl;
use ipc_channel::ipc::IpcSender;
use servo_config::pref;
use std::rc::Rc;
use std::sync::Arc;

/// WebGL Threading API entry point that lives in the constellation.
pub struct WebGLThreads(WebGLSender<WebGLMsg>);

impl WebGLThreads {
    /// Creates a new WebGLThreads object
    pub fn new(
        gl_factory: GLContextFactory,
        webrender_gl: Rc<dyn gl::Gl>,
        webrender_api_sender: webrender_api::RenderApiSender,
        webvr_compositor: Option<Box<dyn WebVRRenderHandler>>,
        webrender_image_handlers_sender: IpcSender<WebrenderImageHandlersMsg>,
    ) -> WebGLThreads {
        // This implementation creates a single `WebGLThread` for all the pipelines.
        WebGLThreads(WebGLThread::start(
            gl_factory,
            webrender_api_sender,
            webvr_compositor.map(|c| WebVRRenderWrapper(c)),
            webrender_image_handlers_sender,
        ))
    }

    /// Gets the WebGLThread handle for each script pipeline.
    pub fn pipeline(&self) -> WebGLPipeline {
        // This mode creates a single thread, so the existing WebGLChan is just cloned.
        WebGLPipeline(WebGLChan(self.0.clone()))
    }

    /// Sends a exit message to close the WebGLThreads and release all WebGLContexts.
    pub fn exit(&self) -> Result<(), &'static str> {
        self.0
            .send(WebGLMsg::Exit)
            .map_err(|_| "Failed to send Exit message")
    }
}

/// Wrapper to send WebVR commands used in `WebGLThread`.
struct WebVRRenderWrapper(Box<dyn WebVRRenderHandler>);

impl WebVRRenderHandler for WebVRRenderWrapper {
    fn handle(
        &mut self,
        gl: &dyn gl::Gl,
        command: WebVRCommand,
        texture: Option<(u32, Size2D<i32>)>,
    ) {
        self.0.handle(gl, command, texture);
    }
}

/// struct used to implement DOMToTexture feature and webrender::OutputImageHandler trait.
type OutputHandlerData = Option<(u32, Size2D<i32>)>;
struct OutputHandler {
    webrender_gl: Rc<dyn gl::Gl>,
    webgl_channel: WebGLSender<WebGLMsg>,
    // Used to avoid creating a new channel on each received WebRender request.
    lock_channel: (
        WebGLSender<OutputHandlerData>,
        WebGLReceiver<OutputHandlerData>,
    ),
    sync_objects: FnvHashMap<webrender_api::PipelineId, gl::GLsync>,
}

impl OutputHandler {
    fn new(webrender_gl: Rc<dyn gl::Gl>, channel: WebGLSender<WebGLMsg>) -> Self {
        Self {
            webrender_gl,
            webgl_channel: channel,
            lock_channel: webgl_channel().unwrap(),
            sync_objects: Default::default(),
        }
    }
}

/// Bridge between the WR frame outputs and WebGL to implement DOMToTexture synchronization.
impl webrender::OutputImageHandler for OutputHandler {
    fn lock(
        &mut self,
        id: webrender_api::PipelineId,
    ) -> Option<(u32, webrender_api::FramebufferIntSize)> {
        // Insert a fence in the WR command queue
        let gl_sync = self
            .webrender_gl
            .fence_sync(gl::SYNC_GPU_COMMANDS_COMPLETE, 0);
        // The lock command adds a WaitSync call on the WebGL command flow.
        let command = DOMToTextureCommand::Lock(id, gl_sync as usize, self.lock_channel.0.clone());
        self.webgl_channel
            .send(WebGLMsg::DOMToTextureCommand(command))
            .unwrap();
        self.lock_channel.1.recv().unwrap().map(|(tex_id, size)| {
            (
                tex_id,
                webrender_api::FramebufferIntSize::new(size.width, size.height),
            )
        })
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

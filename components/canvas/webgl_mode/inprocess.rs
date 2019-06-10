/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::gl_context::GLContextFactory;
use crate::webgl_thread::WebGLThread;
use canvas_traits::webgl::{WebGLChan, WebGLMsg, WebGLPipeline};
use canvas_traits::webgl::{WebGLSender, WebVRCommand, WebVRRenderHandler};
use euclid::Size2D;
use gl_traits::WebrenderImageHandlersMsg;
use gleam::gl;
use ipc_channel::ipc::IpcSender;

/// WebGL Threading API entry point that lives in the constellation.
pub struct WebGLThreads(WebGLSender<WebGLMsg>);

impl WebGLThreads {
    /// Creates a new WebGLThreads object
    pub fn new(
        gl_factory: GLContextFactory,
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

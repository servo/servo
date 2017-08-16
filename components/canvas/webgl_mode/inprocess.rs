/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use ::gl_context::GLContextFactory;
use ::webgl_thread::{WebGLExternalImageApi, WebGLExternalImageHandler, WebGLThreadObserver, WebGLThread};
use canvas_traits::webgl::{WebGLChan, WebGLContextId, WebGLMsg, WebGLPipeline, WebGLReceiver};
use canvas_traits::webgl::{WebGLSender, WebVRCommand, WebVRRenderHandler};
use canvas_traits::webgl::webgl_channel;
use euclid::Size2D;
use std::marker::PhantomData;
use webrender;
use webrender_api;

/// WebGL Threading API entry point that lives in the constellation.
pub struct WebGLThreads(WebGLSender<WebGLMsg>);

impl WebGLThreads {
    /// Creates a new WebGLThreads object
    pub fn new(gl_factory: GLContextFactory,
               webrender_api_sender: webrender_api::RenderApiSender,
               webvr_compositor: Option<Box<WebVRRenderHandler>>)
               -> (WebGLThreads, Box<webrender::ExternalImageHandler>) {
        // This implementation creates a single `WebGLThread` for all the pipelines.
        let channel = WebGLThread::start(gl_factory,
                                         webrender_api_sender,
                                         webvr_compositor.map(|c| WebVRRenderWrapper(c)),
                                         PhantomData);
        let external = WebGLExternalImageHandler::new(WebGLExternalImages::new(channel.clone()));
        (WebGLThreads(channel), Box::new(external))
    }

    /// Gets the WebGLThread handle for each script pipeline.
    pub fn pipeline(&self) -> WebGLPipeline {
        // This mode creates a single thread, so the existing WebGLChan is just cloned.
        WebGLPipeline(WebGLChan(self.0.clone()))
    }

    /// Sends a exit message to close the WebGLThreads and release all WebGLContexts.
    pub fn exit(&self) -> Result<(), &'static str> {
        self.0.send(WebGLMsg::Exit).map_err(|_| "Failed to send Exit message")
    }
}

/// Bridge between the webrender::ExternalImage callbacks and the WebGLThreads.
struct WebGLExternalImages {
    webgl_channel: WebGLSender<WebGLMsg>,
    // Used to avoid creating a new channel on each received WebRender request.
    lock_channel: (WebGLSender<(u32, Size2D<i32>)>, WebGLReceiver<(u32, Size2D<i32>)>),
}

impl WebGLExternalImages {
    fn new(channel: WebGLSender<WebGLMsg>) -> Self {
        Self {
            webgl_channel: channel,
            lock_channel: webgl_channel().unwrap(),
        }
    }
}

impl WebGLExternalImageApi for WebGLExternalImages {
    fn lock(&mut self, ctx_id: WebGLContextId) -> (u32, Size2D<i32>) {
        self.webgl_channel.send(WebGLMsg::Lock(ctx_id, self.lock_channel.0.clone())).unwrap();
        self.lock_channel.1.recv().unwrap()
    }

    fn unlock(&mut self, ctx_id: WebGLContextId) {
        self.webgl_channel.send(WebGLMsg::Unlock(ctx_id)).unwrap();
    }
}

/// Custom observer used in a `WebGLThread`.
impl WebGLThreadObserver for PhantomData<()> {
    fn on_context_create(&mut self, ctx_id: WebGLContextId, texture_id: u32, size: Size2D<i32>) {
        debug!("WebGLContext created (ctx_id: {:?} texture_id: {:?} size: {:?}", ctx_id, texture_id, size);
    }

    fn on_context_resize(&mut self, ctx_id: WebGLContextId, texture_id: u32, size: Size2D<i32>) {
        debug!("WebGLContext resized (ctx_id: {:?} texture_id: {:?} size: {:?}", ctx_id, texture_id, size);
    }

    fn on_context_delete(&mut self, ctx_id: WebGLContextId) {
        debug!("WebGLContext deleted (ctx_id: {:?})", ctx_id);
    }
}


/// Wrapper to send WebVR commands used in `WebGLThread`.
struct WebVRRenderWrapper(Box<WebVRRenderHandler>);

impl WebVRRenderHandler for WebVRRenderWrapper {
    fn handle(&mut self, command: WebVRCommand, texture: Option<(u32, Size2D<i32>)>) {
        self.0.handle(command, texture);
    }
}

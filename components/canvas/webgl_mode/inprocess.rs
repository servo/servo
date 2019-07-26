/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::gl_context::GLContextFactory;
use crate::webgl_thread::{WebGLMainThread, WebGLThread, WebGLThreadInit};
use canvas_traits::webgl::webgl_channel;
use canvas_traits::webgl::DOMToTextureCommand;
use canvas_traits::webgl::{WebGLChan, WebGLContextId, WebGLMsg, WebGLPipeline, WebGLReceiver};
use canvas_traits::webgl::{WebGLSender, WebVRRenderHandler};
use embedder_traits::EventLoopWaker;
use euclid::default::Size2D;
use fnv::FnvHashMap;
use gleam::gl;
use servo_config::pref;
use std::default::Default;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use webrender_traits::{WebrenderExternalImageApi, WebrenderExternalImageRegistry};

/// WebGL Threading API entry point that lives in the constellation.
pub struct WebGLThreads(WebGLSender<WebGLMsg>);

pub enum ThreadMode {
    MainThread(Box<dyn EventLoopWaker>),
    OffThread,
}

impl WebGLThreads {
    /// Creates a new WebGLThreads object
    pub fn new(
        gl_factory: GLContextFactory,
        webrender_gl: Rc<dyn gl::Gl>,
        webrender_api_sender: webrender_api::RenderApiSender,
        webvr_compositor: Option<Box<dyn WebVRRenderHandler>>,
        external_images: Arc<Mutex<WebrenderExternalImageRegistry>>,
        mode: ThreadMode,
    ) -> (
        WebGLThreads,
        Option<Rc<WebGLMainThread>>,
        Box<dyn WebrenderExternalImageApi>,
        Option<Box<dyn webrender::OutputImageHandler>>,
    ) {
        let (sender, receiver) = webgl_channel::<WebGLMsg>().unwrap();
        // This implementation creates a single `WebGLThread` for all the pipelines.
        let init = WebGLThreadInit {
            gl_factory,
            webrender_api_sender,
            webvr_compositor,
            external_images,
            sender: sender.clone(),
            receiver,
        };

        let output_handler = if pref!(dom.webgl.dom_to_texture.enabled) {
            Some(Box::new(OutputHandler::new(
                webrender_gl.clone(),
                sender.clone(),
            )))
        } else {
            None
        };

        let external = WebGLExternalImages::new(webrender_gl, sender.clone());

        let webgl_thread = match mode {
            ThreadMode::MainThread(event_loop_waker) => {
                let thread = WebGLThread::run_on_current_thread(init, event_loop_waker);
                Some(thread)
            },
            ThreadMode::OffThread => {
                WebGLThread::run_on_own_thread(init);
                None
            },
        };

        (
            WebGLThreads(sender),
            webgl_thread,
            Box::new(external),
            output_handler.map(|b| b as Box<_>),
        )
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

/// Bridge between the webrender::ExternalImage callbacks and the WebGLThreads.
struct WebGLExternalImages {
    webrender_gl: Rc<dyn gl::Gl>,
    webgl_channel: WebGLSender<WebGLMsg>,
    // Used to avoid creating a new channel on each received WebRender request.
    lock_channel: (
        WebGLSender<(u32, Size2D<i32>, usize)>,
        WebGLReceiver<(u32, Size2D<i32>, usize)>,
    ),
}

impl WebGLExternalImages {
    fn new(webrender_gl: Rc<dyn gl::Gl>, channel: WebGLSender<WebGLMsg>) -> Self {
        WebGLExternalImages {
            webrender_gl,
            webgl_channel: channel,
            lock_channel: webgl_channel().unwrap(),
        }
    }
}

impl WebrenderExternalImageApi for WebGLExternalImages {
    fn lock(&mut self, id: u64) -> (u32, Size2D<i32>) {
        if let Some(main_thread) = WebGLMainThread::on_current_thread() {
            // If we're on the same thread as WebGL, we can get the data directly
            main_thread
                .thread_data
                .borrow_mut()
                .handle_lock_unsync(WebGLContextId(id as usize))
        } else {
            // WebGL Thread has it's own GL command queue that we need to synchronize with the WR GL command queue.
            // The WebGLMsg::Lock message inserts a fence in the WebGL command queue.
            self.webgl_channel
                .send(WebGLMsg::Lock(
                    WebGLContextId(id as usize),
                    self.lock_channel.0.clone(),
                ))
                .unwrap();
            let (image_id, size, gl_sync) = self.lock_channel.1.recv().unwrap();
            // The next glWaitSync call is run on the WR thread and it's used to synchronize the two
            // flows of OpenGL commands in order to avoid WR using a semi-ready WebGL texture.
            // glWaitSync doesn't block WR thread, it affects only internal OpenGL subsystem.
            self.webrender_gl
                .wait_sync(gl_sync as gl::GLsync, 0, gl::TIMEOUT_IGNORED);
            (image_id, size)
        }
    }

    fn unlock(&mut self, id: u64) {
        if let Some(main_thread) = WebGLMainThread::on_current_thread() {
            // If we're on the same thread as WebGL, we can unlock directly
            main_thread
                .thread_data
                .borrow_mut()
                .handle_unlock(WebGLContextId(id as usize))
        } else {
            self.webgl_channel
                .send(WebGLMsg::Unlock(WebGLContextId(id as usize)))
                .unwrap()
        }
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
        OutputHandler {
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
    ) -> Option<(u32, webrender_api::units::FramebufferIntSize)> {
        // Insert a fence in the WR command queue
        let gl_sync = self
            .webrender_gl
            .fence_sync(gl::SYNC_GPU_COMMANDS_COMPLETE, 0);
        self.sync_objects.insert(id, gl_sync);

        let result = if let Some(main_thread) = WebGLMainThread::on_current_thread() {
            main_thread
                .thread_data
                .borrow_mut()
                .handle_dom_to_texture_lock(id, gl_sync as usize)
        } else {
            // The lock command adds a WaitSync call on the WebGL command flow.
            let command =
                DOMToTextureCommand::Lock(id, gl_sync as usize, self.lock_channel.0.clone());
            self.webgl_channel
                .send(WebGLMsg::DOMToTextureCommand(command))
                .unwrap();
            self.lock_channel.1.recv().unwrap()
        };

        result.map(|(tex_id, size)| {
            (
                tex_id,
                webrender_api::units::FramebufferIntSize::new(size.width, size.height),
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

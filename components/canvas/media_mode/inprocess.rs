/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::media_thread::{GLPlayerExternalImageApi, GLPlayerExternalImageHandler, GLPlayerThread};
use canvas_traits::media::glplayer_channel;
use canvas_traits::media::{
    GLPlayerChan, GLPlayerMsg, GLPlayerPipeline, GLPlayerReceiver, GLPlayerSender,
};
use euclid::Size2D;

/// GLPlayer Threading API entry point that lives in the constellation.
pub struct GLPlayerThreads(GLPlayerSender<GLPlayerMsg>);

impl GLPlayerThreads {
    pub fn new() -> (GLPlayerThreads, Box<dyn webrender::ExternalImageHandler>) {
        let channel = GLPlayerThread::start();
        let external =
            GLPlayerExternalImageHandler::new(GLPlayerExternalImages::new(channel.clone()));
        (GLPlayerThreads(channel), Box::new(external))
    }

    /// Gets the GLPlayerThread handle for each script pipeline.
    pub fn pipeline(&self) -> GLPlayerPipeline {
        // This mode creates a single thread, so the existing
        // GLPlayerChan is just cloned.
        GLPlayerPipeline(GLPlayerChan(self.0.clone()))
    }

    /// Sends an exit message to close the GLPlayerThreads
    pub fn exit(&self) -> Result<(), &'static str> {
        self.0
            .send(GLPlayerMsg::Exit)
            .map_err(|_| "Failed to send Exit message")
    }
}

/// Bridge between the webrender::ExternalImage callbacks and the
/// GLPlayerThreads.
struct GLPlayerExternalImages {
    // @FIXME(victor): this should be added when GstGLSyncMeta is
    // added
    //webrender_gl: Rc<dyn gl::Gl>,
    glplayer_channel: GLPlayerSender<GLPlayerMsg>,
    // Used to avoid creating a new channel on each received WebRender
    // request.
    lock_channel: (
        GLPlayerSender<(u32, Size2D<i32>, usize)>,
        GLPlayerReceiver<(u32, Size2D<i32>, usize)>,
    ),
}

impl GLPlayerExternalImages {
    fn new(channel: GLPlayerSender<GLPlayerMsg>) -> Self {
        Self {
            glplayer_channel: channel,
            lock_channel: glplayer_channel().unwrap(),
        }
    }
}

impl GLPlayerExternalImageApi for GLPlayerExternalImages {
    fn lock(&mut self, id: u64) -> (u32, Size2D<i32>) {
        // The GLPlayerMsgForward::Lock message inserts a fence in the
        // GLPlayer command queue.
        self.glplayer_channel
            .send(GLPlayerMsg::Lock(id, self.lock_channel.0.clone()))
            .unwrap();
        let (image_id, size, _gl_sync) = self.lock_channel.1.recv().unwrap();
        // The next glWaitSync call is run on the WR thread and it's
        // used to synchronize the two flows of OpenGL commands in
        // order to avoid WR using a semi-ready GLPlayer texture.
        // glWaitSync doesn't block WR thread, it affects only
        // internal OpenGL subsystem.
        //self.webrender_gl
        //    .wait_sync(gl_sync as gl::GLsync, 0, gl::TIMEOUT_IGNORED);
        (image_id, size)
    }

    fn unlock(&mut self, id: u64) {
        self.glplayer_channel.send(GLPlayerMsg::Unlock(id)).unwrap();
    }
}

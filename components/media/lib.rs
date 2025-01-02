/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#![deny(unsafe_code)]
#![allow(clippy::type_complexity)]

mod media_channel;
mod media_thread;

use std::sync::{Arc, Mutex};

use euclid::default::Size2D;
use serde::{Deserialize, Serialize};
pub use servo_media::player::context::{GlApi, GlContext, NativeDisplay, PlayerGLContext};
use webrender_traits::{
    WebrenderExternalImageApi, WebrenderExternalImageRegistry, WebrenderImageSource,
};

pub use crate::media_channel::glplayer_channel;
use crate::media_channel::{GLPlayerChan, GLPlayerPipeline, GLPlayerReceiver, GLPlayerSender};
use crate::media_thread::GLPlayerThread;

/// These are the messages that the GLPlayer thread will forward to
/// the video player which lives in htmlmediaelement
#[derive(Debug, Deserialize, Serialize)]
pub enum GLPlayerMsgForward {
    PlayerId(u64),
    Lock(GLPlayerSender<(u32, Size2D<i32>, usize)>),
    Unlock(),
}

/// GLPlayer thread Message API
///
/// These are the messages that the thread will receive from the
/// constellation, the webrender::ExternalImageHandle demultiplexor
/// implementation, or a htmlmediaelement
#[derive(Debug, Deserialize, Serialize)]
pub enum GLPlayerMsg {
    /// Registers an instantiated player in DOM
    RegisterPlayer(GLPlayerSender<GLPlayerMsgForward>),
    /// Unregisters a player's ID
    UnregisterPlayer(u64),
    /// Locks a specific texture from a player. Lock messages are used
    /// for a correct synchronization with WebRender external image
    /// API.
    ///
    /// WR locks a external texture when it wants to use the shared
    /// texture contents.
    ///
    /// The WR client should not change the shared texture content
    /// until the Unlock call.
    ///
    /// Currently OpenGL Sync Objects are used to implement the
    /// synchronization mechanism.
    Lock(u64, GLPlayerSender<(u32, Size2D<i32>, usize)>),
    /// Unlocks a specific texture from a player. Unlock messages are
    /// used for a correct synchronization with WebRender external
    /// image API.
    ///
    /// The WR unlocks a context when it finished reading the shared
    /// texture contents.
    ///
    /// Unlock messages are always sent after a Lock message.
    Unlock(u64),
    /// Frees all resources and closes the thread.
    Exit,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct WindowGLContext {
    /// Application's GL Context
    pub gl_context: GlContext,
    /// Application's GL Api
    pub gl_api: GlApi,
    /// Application's native display
    pub native_display: NativeDisplay,
    /// A channel to the GLPlayer thread.
    pub glplayer_chan: Option<GLPlayerPipeline>,
}

impl PlayerGLContext for WindowGLContext {
    fn get_gl_context(&self) -> GlContext {
        self.gl_context.clone()
    }

    fn get_native_display(&self) -> NativeDisplay {
        self.native_display.clone()
    }

    fn get_gl_api(&self) -> GlApi {
        self.gl_api.clone()
    }
}

/// GLPlayer Threading API entry point that lives in the constellation.
pub struct GLPlayerThreads(GLPlayerSender<GLPlayerMsg>);

impl GLPlayerThreads {
    pub fn new(
        external_images: Arc<Mutex<WebrenderExternalImageRegistry>>,
    ) -> (GLPlayerThreads, Box<dyn WebrenderExternalImageApi>) {
        let channel = GLPlayerThread::start(external_images);
        let external = GLPlayerExternalImages::new(channel.clone());
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

impl WebrenderExternalImageApi for GLPlayerExternalImages {
    fn lock(&mut self, id: u64) -> (WebrenderImageSource, Size2D<i32>) {
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
        (WebrenderImageSource::TextureHandle(image_id), size)
    }

    fn unlock(&mut self, id: u64) {
        self.glplayer_channel.send(GLPlayerMsg::Unlock(id)).unwrap();
    }
}

/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#![deny(unsafe_code)]
#![allow(clippy::type_complexity)]

mod media_thread;

use std::sync::{Arc, Mutex};

use euclid::default::Size2D;
use ipc_channel::ipc::{channel, IpcReceiver, IpcSender};
use log::warn;
use serde::{Deserialize, Serialize};
use servo_config::pref;
pub use servo_media::player::context::{GlApi, GlContext, NativeDisplay, PlayerGLContext};
use webrender_traits::{
    WebrenderExternalImageApi, WebrenderExternalImageHandlers, WebrenderExternalImageRegistry,
    WebrenderImageHandlerType, WebrenderImageSource,
};

use crate::media_thread::GLPlayerThread;

/// A global version of the [`WindowGLContext`] to be shared between the embedder and the
/// constellation. This is only okay to do because OpenGL contexts cannot be used across processes
/// anyway.
///
/// This avoid having to establish a depenency on `media` in `*_traits` crates.
static WINDOW_GL_CONTEXT: Mutex<WindowGLContext> = Mutex::new(WindowGLContext::inactive());

/// These are the messages that the GLPlayer thread will forward to
/// the video player which lives in htmlmediaelement
#[derive(Debug, Deserialize, Serialize)]
pub enum GLPlayerMsgForward {
    PlayerId(u64),
    Lock(IpcSender<(u32, Size2D<i32>, usize)>),
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
    RegisterPlayer(IpcSender<GLPlayerMsgForward>),
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
    Lock(u64, IpcSender<(u32, Size2D<i32>, usize)>),
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

/// A [`PlayerGLContext`] that renders to a window. Note that if the background
/// thread is not started for this context, then it is inactive (returning
/// `Unknown` values in the trait implementation).
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct WindowGLContext {
    /// Application's GL Context
    pub context: GlContext,
    /// Application's GL Api
    pub api: GlApi,
    /// Application's native display
    pub display: NativeDisplay,
    /// A channel to the GLPlayer thread.
    pub glplayer_thread_sender: Option<IpcSender<GLPlayerMsg>>,
}

impl WindowGLContext {
    /// Create an inactive [`WindowGLContext`].
    pub const fn inactive() -> Self {
        WindowGLContext {
            context: GlContext::Unknown,
            api: GlApi::None,
            display: NativeDisplay::Unknown,
            glplayer_thread_sender: None,
        }
    }

    pub fn register(context: Self) {
        *WINDOW_GL_CONTEXT.lock().unwrap() = context;
    }

    pub fn get() -> Self {
        WINDOW_GL_CONTEXT.lock().unwrap().clone()
    }

    /// Sends an exit message to close the GLPlayerThread.
    pub fn exit(&self) {
        self.send(GLPlayerMsg::Exit);
    }

    #[inline]
    pub fn send(&self, message: GLPlayerMsg) {
        // Don't do anything if GL accelerated playback is disabled.
        let Some(sender) = self.glplayer_thread_sender.as_ref() else {
            return;
        };

        if let Err(error) = sender.send(message) {
            warn!("Could no longer communicate with GL accelerated media threads: {error}")
        }
    }

    pub fn initialize(display: NativeDisplay, api: GlApi, context: GlContext) {
        if matches!(display, NativeDisplay::Unknown) || matches!(context, GlContext::Unknown) {
            return;
        }

        let mut window_gl_context = WINDOW_GL_CONTEXT.lock().unwrap();
        if window_gl_context.glplayer_thread_sender.is_some() {
            warn!("Not going to initialize GL accelerated media playback more than once.");
            return;
        }

        window_gl_context.context = context;
        window_gl_context.display = display;
        window_gl_context.api = api;
    }

    pub fn initialize_image_handler(
        external_image_handlers: &mut WebrenderExternalImageHandlers,
        external_images: Arc<Mutex<WebrenderExternalImageRegistry>>,
    ) {
        if !pref!(media_glvideo_enabled) {
            return;
        }

        let mut window_gl_context = WINDOW_GL_CONTEXT.lock().unwrap();
        if window_gl_context.glplayer_thread_sender.is_some() {
            warn!("Not going to initialize GL accelerated media playback more than once.");
            return;
        }

        if matches!(window_gl_context.display, NativeDisplay::Unknown) ||
            matches!(window_gl_context.context, GlContext::Unknown)
        {
            return;
        }

        let thread_sender = GLPlayerThread::start(external_images);
        let image_handler = Box::new(GLPlayerExternalImages::new(thread_sender.clone()));
        external_image_handlers.set_handler(image_handler, WebrenderImageHandlerType::Media);
        window_gl_context.glplayer_thread_sender = Some(thread_sender);
    }
}

impl PlayerGLContext for WindowGLContext {
    fn get_gl_context(&self) -> GlContext {
        match self.glplayer_thread_sender {
            Some(..) => self.context.clone(),
            None => GlContext::Unknown,
        }
    }

    fn get_native_display(&self) -> NativeDisplay {
        match self.glplayer_thread_sender {
            Some(..) => self.display.clone(),
            None => NativeDisplay::Unknown,
        }
    }

    fn get_gl_api(&self) -> GlApi {
        self.api.clone()
    }
}

/// Bridge between the webrender::ExternalImage callbacks and the
/// GLPlayerThreads.
struct GLPlayerExternalImages {
    // @FIXME(victor): this should be added when GstGLSyncMeta is
    // added
    //webrender_gl: Rc<dyn gl::Gl>,
    glplayer_channel: IpcSender<GLPlayerMsg>,
    // Used to avoid creating a new channel on each received WebRender
    // request.
    lock_channel: (
        IpcSender<(u32, Size2D<i32>, usize)>,
        IpcReceiver<(u32, Size2D<i32>, usize)>,
    ),
}

impl GLPlayerExternalImages {
    fn new(sender: IpcSender<GLPlayerMsg>) -> Self {
        Self {
            glplayer_channel: sender,
            lock_channel: channel().unwrap(),
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

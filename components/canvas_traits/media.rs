/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use euclid::Size2D;
use servo_media::player::context::{GlApi, GlContext, NativeDisplay, PlayerGLContext};

/// Helper function that creates a GLPlayer channel (GLPlayerSender,
/// GLPlayerReceiver) to be used in GLPlayerMsg.
pub use crate::media_channel::glplayer_channel;
/// Entry point channel type used for sending GLPlayerMsg messages to
/// the GLPlayer thread.
pub use crate::media_channel::GLPlayerChan;
/// Entry point type used in a Script Pipeline to get the GLPlayerChan
/// to be used in that thread.
pub use crate::media_channel::GLPlayerPipeline;
/// Receiver type used in GLPlayerMsg.
pub use crate::media_channel::GLPlayerReceiver;
/// Result type for send()/recv() calls in in GLPlayerMsg.
pub use crate::media_channel::GLPlayerSendResult;
/// Sender type used in GLPlayerMsg.
pub use crate::media_channel::GLPlayerSender;

/// GLPlayer thread Message API
///
/// These are the message that the thread will receive from the
/// constellation, the webrender::ExternalImageHandle multiplexor
/// implementation, or a htmlmediaelement
#[derive(Debug, Deserialize, Serialize)]
pub enum GLPlayerMsg {
    /// Registers an instantiated player in DOM
    RegisterPlayer(GLPlayerSender<u64>),
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

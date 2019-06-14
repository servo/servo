/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use canvas_traits::media::*;
use euclid::Size2D;
use fnv::FnvHashMap;
use std::thread;

/// GL player threading API entry point that lives in the
/// constellation.
///
/// It allows to get a GLPlayerThead handle for each script pipeline.
pub use crate::media_mode::GLPlayerThreads;

/// A GLPlayerThrx1ead manages the life cycle and message multiplexign of
/// a set of video players with GL render.
pub struct GLPlayerThread {
    // Map of live players.
    players: FnvHashMap<u64, GLPlayerSender<GLPlayerMsgForward>>,
    /// Id generator for new WebGLContexts.
    next_player_id: u64,
}

impl GLPlayerThread {
    pub fn new() -> Self {
        GLPlayerThread {
            players: Default::default(),
            next_player_id: 1,
        }
    }

    pub fn start() -> GLPlayerSender<GLPlayerMsg> {
        let (sender, receiver) = glplayer_channel::<GLPlayerMsg>().unwrap();
        thread::Builder::new()
            .name("GLPlayerThread".to_owned())
            .spawn(move || {
                let mut renderer = GLPlayerThread::new();
                loop {
                    let msg = receiver.recv().unwrap();
                    let exit = renderer.handle_msg(msg);
                    if exit {
                        return;
                    }
                }
            })
            .expect("Thread spawning failed");

        sender
    }

    /// Handles a generic WebGLMsg message
    #[inline]
    fn handle_msg(&mut self, msg: GLPlayerMsg) -> bool {
        trace!("processing {:?}", msg);
        match msg {
            GLPlayerMsg::RegisterPlayer(sender) => {
                let id = self.next_player_id;
                self.players.insert(id, sender.clone());
                sender.send(GLPlayerMsgForward::PlayerId(id)).unwrap();
                self.next_player_id += 1;
            },
            GLPlayerMsg::UnregisterPlayer(id) => {
                if self.players.remove(&id).is_none() {
                    warn!("Tried to remove an unknown player");
                }
            },
            GLPlayerMsg::Lock(id, handler_sender) => {
                self.players.get(&id).map(|sender| {
                    sender.send(GLPlayerMsgForward::Lock(handler_sender)).ok();
                });
            },
            GLPlayerMsg::Unlock(id) => {
                self.players.get(&id).map(|sender| {
                    sender.send(GLPlayerMsgForward::Unlock()).ok();
                });
            },
            GLPlayerMsg::Exit => return true,
        }

        false
    }
}

/// This trait is used as a bridge between the `GLPlayerThreads`
/// implementation and the WR ExternalImageHandler API implemented in
/// the `GLPlayerExternalImageHandler` struct.
//
/// `GLPlayerExternalImageHandler<T>` takes care of type conversions
/// between WR and GLPlayer info (e.g keys, uvs).
//
/// It uses this trait to notify lock/unlock messages and get the
/// required info that WR needs.
//
/// `GLPlayerThreads` receives lock/unlock message notifications and
/// takes care of sending the unlock/lock messages to the appropiate
/// `GLPlayerThread`.
pub trait GLPlayerExternalImageApi {
    fn lock(&mut self, id: u64) -> (u32, Size2D<i32>);
    fn unlock(&mut self, id: u64);
}

/// WebRender External Image Handler implementation
pub struct GLPlayerExternalImageHandler<T: GLPlayerExternalImageApi> {
    handler: T,
}

impl<T: GLPlayerExternalImageApi> GLPlayerExternalImageHandler<T> {
    pub fn new(handler: T) -> Self {
        Self { handler: handler }
    }
}

impl<T: GLPlayerExternalImageApi> webrender::ExternalImageHandler
    for GLPlayerExternalImageHandler<T>
{
    /// Lock the external image. Then, WR could start to read the
    /// image content.
    /// The WR client should not change the image content until the
    /// unlock() call.
    fn lock(
        &mut self,
        key: webrender_api::ExternalImageId,
        _channel_index: u8,
        _rendering: webrender_api::ImageRendering,
    ) -> webrender::ExternalImage {
        let (texture_id, size) = self.handler.lock(key.0);

        webrender::ExternalImage {
            uv: webrender_api::TexelRect::new(0.0, 0.0, size.width as f32, size.height as f32),
            source: webrender::ExternalImageSource::NativeTexture(texture_id),
        }
    }

    /// Unlock the external image. The WR should not read the image
    /// content after this call.
    fn unlock(&mut self, key: webrender_api::ExternalImageId, _channel_index: u8) {
        self.handler.unlock(key.0);
    }
}

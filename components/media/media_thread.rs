/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::media_channel::{glplayer_channel, GLPlayerSender};
/// GL player threading API entry point that lives in the
/// constellation.
use crate::{GLPlayerMsg, GLPlayerMsgForward};
use fnv::FnvHashMap;
use std::thread;

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

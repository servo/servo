/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use canvas_traits::media::*;
use std::thread;

/// GL player threading API entry point that lives in the
/// constellation.
///
/// It allows to get a GLPlayerThead handle for each script pipeline.
pub use crate::media_mode::GLPlayerThreads;

/// A GLPlayerThrx1ead manages the life cycle and message multiplexign of
/// a set of video players with GL render.
pub struct GLPlayerThread ();

impl GLPlayerThread {
    pub fn new() -> Self {
        GLPlayerThread()
    }

    pub fn start() -> GLPlayerSender<GLPlayerMsg> {
        let (sender, receiver) = glplayer_channel::<GLPlayerMsg>().unwrap();
        thread::Builder::new()
            .name("GLPlayerThread".to_owned())
            .spawn(move || {
                let renderer = GLPlayerThread::new();
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
    fn handle_msg(&self, msg: GLPlayerMsg) -> bool {
        trace!("processing {:?}", msg);
        match msg {
            GLPlayerMsg::Exit => return true,
            _ => (),
        }

        false
    }
}

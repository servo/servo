/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::media_thread::GLPlayerThread;
use canvas_traits::media::{GLPlayerChan, GLPlayerMsg, GLPlayerPipeline, GLPlayerSender};

/// GLPlayer Threading API entry point that lives in the constellation.
pub struct GLPlayerThreads(GLPlayerSender<GLPlayerMsg>);

impl GLPlayerThreads {
    pub fn new() -> GLPlayerThreads {
        let channel = GLPlayerThread::start();
        GLPlayerThreads(channel)
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

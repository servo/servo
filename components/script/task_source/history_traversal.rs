/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use crate::script_runtime::{CommonScriptMsg, ScriptChan};
use crate::script_thread::MainThreadScriptMsg;
use crossbeam_channel::Sender;

#[derive(JSTraceable)]
pub struct HistoryTraversalTaskSource(pub Sender<MainThreadScriptMsg>);

impl ScriptChan for HistoryTraversalTaskSource {
    fn send(&self, msg: CommonScriptMsg) -> Result<(), ()> {
        self.0
            .send(MainThreadScriptMsg::Common(msg))
            .map_err(|_| ())
    }

    fn clone(&self) -> Box<dyn ScriptChan + Send> {
        Box::new(HistoryTraversalTaskSource((&self.0).clone()))
    }
}

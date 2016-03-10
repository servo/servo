/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use script_thread::{CommonScriptMsg, MainThreadScriptMsg, ScriptChan};
use std::sync::mpsc::Sender;

#[derive(JSTraceable)]
pub struct NetworkingTaskSource(pub Sender<MainThreadScriptMsg>);

impl ScriptChan for NetworkingTaskSource {
    fn send(&self, msg: CommonScriptMsg) -> Result<(), ()> {
        let NetworkingTaskSource(ref chan) = *self;
        chan.send(MainThreadScriptMsg::Common(msg)).map_err(|_| ())
    }

    fn clone(&self) -> Box<ScriptChan + Send> {
        let NetworkingTaskSource(ref chan) = *self;
        box NetworkingTaskSource((*chan).clone())
    }
}

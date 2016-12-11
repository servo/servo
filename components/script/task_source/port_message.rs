/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use script_runtime::{CommonScriptMsg, ScriptChan, ScriptThreadEventCategory};
use script_thread::{Runnable, RunnableWrapper};
use task_source::TaskSource;

#[derive(JSTraceable)]
pub struct PortMessageQueue(pub Box<ScriptChan + Send + 'static>);

impl Clone for PortMessageQueue {
    fn clone(&self) -> PortMessageQueue {
        PortMessageQueue(self.0.clone())
    }
}

impl TaskSource for PortMessageQueue {
    fn queue_with_wrapper<T>(&self,
                             msg: Box<T>,
                             wrapper: &RunnableWrapper)
                             -> Result<(), ()>
                             where T: Runnable + Send + 'static {
        self.0.send(CommonScriptMsg::RunnableMsg(ScriptThreadEventCategory::PortMessage,
                                                 wrapper.wrap_runnable(msg)))
    }
}

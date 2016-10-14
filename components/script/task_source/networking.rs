/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use script_runtime::{CommonScriptMsg, ScriptChan, ScriptThreadEventCategory};
use script_thread::{Runnable, RunnableWrapper};
use task_source::TaskSource;

#[derive(JSTraceable)]
pub struct NetworkingTaskSource(pub Box<ScriptChan + Send + 'static>);

impl Clone for NetworkingTaskSource {
    fn clone(&self) -> NetworkingTaskSource {
        NetworkingTaskSource(self.0.clone())
    }
}

impl TaskSource for NetworkingTaskSource {
    fn queue_with_wrapper<T>(&self,
                             msg: Box<T>,
                             wrapper: &RunnableWrapper)
                             -> Result<(), ()>
                             where T: Runnable + Send + 'static {
        self.0.send(CommonScriptMsg::RunnableMsg(ScriptThreadEventCategory::NetworkEvent,
                                                 wrapper.wrap_runnable(msg)))
    }
}

impl NetworkingTaskSource {
    pub fn queue_wrapperless<T: Runnable + Send + 'static>(&self, msg: Box<T>) -> Result<(), ()> {
        self.0.send(CommonScriptMsg::RunnableMsg(ScriptThreadEventCategory::NetworkEvent, msg))
    }
}

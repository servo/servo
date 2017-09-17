/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use script_runtime::{CommonScriptMsg, ScriptChan, ScriptThreadEventCategory};
use task::{Task, TaskCanceller};
use task_source::TaskSource;

#[derive(JSTraceable)]
pub struct NetworkingTaskSource(pub Box<ScriptChan + Send + 'static>);

impl Clone for NetworkingTaskSource {
    fn clone(&self) -> NetworkingTaskSource {
        NetworkingTaskSource(self.0.clone())
    }
}

impl TaskSource for NetworkingTaskSource {
    fn queue_with_canceller<T>(
        &self,
        msg: Box<T>,
        canceller: &TaskCanceller,
    ) -> Result<(), ()>
    where
        T: Send + Task + 'static,
    {
        self.0.send(CommonScriptMsg::Task(
            ScriptThreadEventCategory::NetworkEvent,
            canceller.wrap_task(msg),
        ))
    }
}

impl NetworkingTaskSource {
    /// This queues a task that will not be cancelled when its associated
    /// global scope gets destroyed.
    pub fn queue_unconditionally<T>(&self, msg: Box<T>) -> Result<(), ()>
    where
        T: Task + Send + 'static,
    {
        self.0.send(CommonScriptMsg::Task(ScriptThreadEventCategory::NetworkEvent, msg))
    }
}

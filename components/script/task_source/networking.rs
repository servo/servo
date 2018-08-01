/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use msg::constellation_msg::PipelineId;
use script_runtime::{CommonScriptMsg, ScriptChan, ScriptThreadEventCategory};
use task::{TaskCanceller, TaskOnce};
use task_source::{TaskSource, TaskSourceName};

#[derive(JSTraceable)]
pub struct NetworkingTaskSource(pub Box<ScriptChan + Send + 'static>, pub PipelineId);

impl Clone for NetworkingTaskSource {
    fn clone(&self) -> NetworkingTaskSource {
        NetworkingTaskSource(self.0.clone(), self.1.clone())
    }
}

impl TaskSource for NetworkingTaskSource {
    const NAME: TaskSourceName = TaskSourceName::Networking;

    fn queue_with_canceller<T>(
        &self,
        task: T,
        canceller: &TaskCanceller,
    )
    where
        T: TaskOnce + 'static,
    {
        self.0.send(CommonScriptMsg::Task(
            ScriptThreadEventCategory::NetworkEvent,
            Box::new(canceller.wrap_task(task)),
            Some(self.1),
        ))
    }
}

impl NetworkingTaskSource {
    /// This queues a task that will not be cancelled when its associated
    /// global scope gets destroyed.
    pub fn queue_unconditionally<T>(&self, task: T)
    where
        T: TaskOnce + 'static,
    {
        self.0.send(CommonScriptMsg::Task(
            ScriptThreadEventCategory::NetworkEvent,
            Box::new(task),
            Some(self.1),
        ))
    }
}

/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use msg::constellation_msg::PipelineId;
use script_runtime::{CommonScriptMsg, ScriptChan, ScriptThreadEventCategory};
use task::{TaskCanceller, TaskOnce};
use task_source::{TaskSource, TaskSourceName};
use typeholder::TypeHolderTrait;
use std::marker::PhantomData;

#[derive(JSTraceable)]
pub struct NetworkingTaskSource<TH: TypeHolderTrait>(pub Box<ScriptChan + Send + 'static>, pub PipelineId, pub PhantomData<TH>);

impl<TH: TypeHolderTrait> Clone for NetworkingTaskSource<TH> {
    fn clone(&self) -> NetworkingTaskSource<TH> {
        NetworkingTaskSource(self.0.clone(), self.1.clone(), Default::default())
    }
}

impl<TH: TypeHolderTrait> TaskSource for NetworkingTaskSource<TH> {
    const NAME: TaskSourceName = TaskSourceName::Networking;

    fn queue_with_canceller<T>(
        &self,
        task: T,
        canceller: &TaskCanceller,
    ) -> Result<(), ()>
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

impl<TH: TypeHolderTrait> NetworkingTaskSource<TH> {
    /// This queues a task that will not be cancelled when its associated
    /// global scope gets destroyed.
    pub fn queue_unconditionally<T>(&self, task: T) -> Result<(), ()>
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

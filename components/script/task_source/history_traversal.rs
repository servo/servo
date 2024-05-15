/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use base::id::PipelineId;
use crossbeam_channel::Sender;

use crate::script_runtime::{CommonScriptMsg, ScriptThreadEventCategory};
use crate::script_thread::MainThreadScriptMsg;
use crate::task::{TaskCanceller, TaskOnce};
use crate::task_source::{TaskSource, TaskSourceName};

#[derive(Clone, JSTraceable)]
pub struct HistoryTraversalTaskSource(
    #[no_trace] pub Sender<MainThreadScriptMsg>,
    #[no_trace] pub PipelineId,
);

impl TaskSource for HistoryTraversalTaskSource {
    const NAME: TaskSourceName = TaskSourceName::HistoryTraversal;

    fn queue_with_canceller<T>(&self, task: T, canceller: &TaskCanceller) -> Result<(), ()>
    where
        T: TaskOnce + 'static,
    {
        let msg = MainThreadScriptMsg::Common(CommonScriptMsg::Task(
            ScriptThreadEventCategory::HistoryEvent,
            Box::new(canceller.wrap_task(task)),
            Some(self.1),
            HistoryTraversalTaskSource::NAME,
        ));
        self.0.send(msg).map_err(|_| ())
    }
}

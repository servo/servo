/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use msg::constellation_msg::PipelineId;
use script_runtime::{CommonScriptMsg, ScriptThreadEventCategory};
use script_thread::MainThreadScriptMsg;
use servo_channel::Sender;
use task::{TaskCanceller, TaskOnce};
use task_source::{TaskSource, TaskSourceName};

#[derive(Clone, JSTraceable)]
pub struct HistoryTraversalTaskSource(pub Sender<MainThreadScriptMsg>, pub PipelineId);

impl TaskSource for HistoryTraversalTaskSource {
    const NAME: TaskSourceName = TaskSourceName::HistoryTraversal;

    fn queue_with_canceller<T>(&self, task: T, canceller: &TaskCanceller) -> Result<(), ()>
    where
        T: TaskOnce + 'static,
    {
        let msg = MainThreadScriptMsg::Common(CommonScriptMsg::Task(
            ScriptThreadEventCategory::DocumentEvent,
            Box::new(canceller.wrap_task(task)),
            Some(self.1),
            HistoryTraversalTaskSource::NAME,
        ));
        self.0.send(msg).map_err(|_| ())
    }
}

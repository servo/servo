/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::fmt;

use base::id::PipelineId;

use crate::script_runtime::{CommonScriptMsg, ScriptChan, ScriptThreadEventCategory};
use crate::task::{TaskCanceller, TaskOnce};
use crate::task_source::{TaskSource, TaskSourceName};

#[derive(JSTraceable)]
/// <https://html.spec.whatwg.org/multipage/#timer-task-source>
pub struct TimerTaskSource(
    pub Box<dyn ScriptChan + Send + 'static>,
    #[no_trace] pub PipelineId,
);

impl Clone for TimerTaskSource {
    fn clone(&self) -> TimerTaskSource {
        TimerTaskSource(self.0.clone(), self.1)
    }
}

impl fmt::Debug for TimerTaskSource {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "TimerTaskSource(...)")
    }
}

impl TaskSource for TimerTaskSource {
    const NAME: TaskSourceName = TaskSourceName::Timer;

    fn queue_with_canceller<T>(&self, task: T, canceller: &TaskCanceller) -> Result<(), ()>
    where
        T: TaskOnce + 'static,
    {
        let msg = CommonScriptMsg::Task(
            ScriptThreadEventCategory::TimerEvent,
            Box::new(canceller.wrap_task(task)),
            Some(self.1),
            Self::NAME,
        );
        self.0.send(msg).map_err(|_| ())
    }
}

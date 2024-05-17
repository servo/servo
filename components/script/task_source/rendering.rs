/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::fmt;
use std::result::Result;

use base::id::PipelineId;

use crate::script_runtime::{CommonScriptMsg, ScriptChan, ScriptThreadEventCategory};
use crate::task::{TaskCanceller, TaskOnce};
use crate::task_source::{TaskSource, TaskSourceName};

#[derive(JSTraceable)]
pub struct RenderingTaskSource(pub Box<dyn ScriptChan + Send>, #[no_trace] pub PipelineId);

impl Clone for RenderingTaskSource {
    fn clone(&self) -> RenderingTaskSource {
        RenderingTaskSource(self.0.clone(), self.1)
    }
}

impl fmt::Debug for RenderingTaskSource {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "RenderingTaskSource(...)")
    }
}

impl TaskSource for RenderingTaskSource {
    const NAME: TaskSourceName = TaskSourceName::Rendering;

    fn queue_with_canceller<T>(&self, task: T, canceller: &TaskCanceller) -> Result<(), ()>
    where
        T: TaskOnce + 'static,
    {
        let msg_task = CommonScriptMsg::Task(
            ScriptThreadEventCategory::ScriptEvent,
            Box::new(canceller.wrap_task(task)),
            Some(self.1),
            RenderingTaskSource::NAME,
        );

        self.0.send(msg_task).map_err(|_| ())
    }
}

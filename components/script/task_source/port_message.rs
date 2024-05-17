/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::fmt;

use base::id::PipelineId;

use crate::script_runtime::{CommonScriptMsg, ScriptChan, ScriptThreadEventCategory};
use crate::task::{TaskCanceller, TaskOnce};
use crate::task_source::{TaskSource, TaskSourceName};

#[derive(JSTraceable)]
pub struct PortMessageQueue(
    pub Box<dyn ScriptChan + Send + 'static>,
    #[no_trace] pub PipelineId,
);

impl Clone for PortMessageQueue {
    fn clone(&self) -> PortMessageQueue {
        PortMessageQueue(self.0.clone(), self.1)
    }
}

impl fmt::Debug for PortMessageQueue {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "PortMessageQueue(...)")
    }
}

impl TaskSource for PortMessageQueue {
    const NAME: TaskSourceName = TaskSourceName::PortMessage;

    fn queue_with_canceller<T>(&self, task: T, canceller: &TaskCanceller) -> Result<(), ()>
    where
        T: TaskOnce + 'static,
    {
        let msg = CommonScriptMsg::Task(
            ScriptThreadEventCategory::PortMessage,
            Box::new(canceller.wrap_task(task)),
            Some(self.1),
            Self::NAME,
        );
        self.0.send(msg).map_err(|_| ())
    }
}

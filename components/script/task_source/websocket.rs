/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use base::id::PipelineId;

use crate::script_runtime::{CommonScriptMsg, ScriptChan, ScriptThreadEventCategory};
use crate::task::{TaskCanceller, TaskOnce};
use crate::task_source::{TaskSource, TaskSourceName};

#[derive(JSTraceable)]
pub struct WebsocketTaskSource(
    pub Box<dyn ScriptChan + Send + 'static>,
    #[no_trace] pub PipelineId,
);

impl Clone for WebsocketTaskSource {
    fn clone(&self) -> WebsocketTaskSource {
        WebsocketTaskSource(self.0.clone(), self.1)
    }
}

impl TaskSource for WebsocketTaskSource {
    const NAME: TaskSourceName = TaskSourceName::Websocket;

    fn queue_with_canceller<T>(&self, task: T, canceller: &TaskCanceller) -> Result<(), ()>
    where
        T: TaskOnce + 'static,
    {
        self.0.send(CommonScriptMsg::Task(
            ScriptThreadEventCategory::NetworkEvent,
            Box::new(canceller.wrap_task(task)),
            Some(self.1),
            WebsocketTaskSource::NAME,
        ))
    }
}

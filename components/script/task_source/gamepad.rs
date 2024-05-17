/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::fmt;
use std::result::Result;

use base::id::PipelineId;

use crate::script_runtime::{CommonScriptMsg, ScriptChan, ScriptThreadEventCategory};
use crate::task::{TaskCanceller, TaskOnce};
use crate::task_source::{TaskSource, TaskSourceName};

#[derive(JSTraceable)]
pub struct GamepadTaskSource(
    pub Box<dyn ScriptChan + Send + 'static>,
    #[no_trace] pub PipelineId,
);

impl Clone for GamepadTaskSource {
    fn clone(&self) -> GamepadTaskSource {
        GamepadTaskSource(self.0.clone(), self.1)
    }
}

impl fmt::Debug for GamepadTaskSource {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "GamepadTaskSource(...)")
    }
}

impl TaskSource for GamepadTaskSource {
    const NAME: TaskSourceName = TaskSourceName::Gamepad;

    fn queue_with_canceller<T>(&self, task: T, canceller: &TaskCanceller) -> Result<(), ()>
    where
        T: TaskOnce + 'static,
    {
        let msg = CommonScriptMsg::Task(
            ScriptThreadEventCategory::InputEvent,
            Box::new(canceller.wrap_task(task)),
            Some(self.1),
            GamepadTaskSource::NAME,
        );
        self.0.send(msg).map_err(|_| ())
    }
}

/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::inheritance::Castable;
use dom::bindings::refcounted::Trusted;
use dom::event::{EventBubbles, EventCancelable, EventTask, SimpleEventTask};
use dom::eventtarget::EventTarget;
use dom::window::Window;
use msg::constellation_msg::PipelineId;
use script_runtime::{CommonScriptMsg, ScriptThreadEventCategory};
use script_thread::MainThreadScriptMsg;
use servo_atoms::Atom;
use std::fmt;
use std::result::Result;
use std::sync::mpsc::Sender;
use task::{TaskCanceller, TaskOnce};
use task_source::{TaskSource, TaskSourceName};

#[derive(Clone, JSTraceable)]
pub struct UnshippedPortMessageQueueTaskSource(pub Sender<MainThreadScriptMsg>, pub PipelineId);

impl fmt::Debug for UnshippedPortMessageQueueTaskSource {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "UnshippedPortMessageQueueTaskSource(...)")
    }
}

impl TaskSource for UnshippedPortMessageQueueTaskSource {
    const NAME: TaskSourceName = TaskSourceName::UnshippedPortMessageQueue;

    fn queue_with_canceller<T>(
        &self,
        task: T,
        canceller: &TaskCanceller,
    ) -> Result<(), ()>
    where
        T: TaskOnce + 'static,
    {
        let msg = MainThreadScriptMsg::Common(CommonScriptMsg::Task(
            ScriptThreadEventCategory::ScriptEvent,
            Box::new(canceller.wrap_task(task)),
            Some(self.1),
            UnshippedPortMessageQueueTaskSource::NAME,
        ));
        self.0.send(msg).map_err(|_| ())
    }
}

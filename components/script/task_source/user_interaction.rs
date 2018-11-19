/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::refcounted::Trusted;
use crate::dom::event::{EventBubbles, EventCancelable, EventTask};
use crate::dom::eventtarget::EventTarget;
use crate::dom::window::Window;
use crate::script_runtime::{CommonScriptMsg, ScriptThreadEventCategory};
use crate::script_thread::MainThreadScriptMsg;
use crate::task::{TaskCanceller, TaskOnce};
use crate::task_source::{TaskSource, TaskSourceName};
use crossbeam_channel::Sender;
use msg::constellation_msg::PipelineId;
use servo_atoms::Atom;
use std::fmt;
use std::result::Result;

#[derive(Clone, JSTraceable)]
pub struct UserInteractionTaskSource(pub Sender<MainThreadScriptMsg>, pub PipelineId);

impl fmt::Debug for UserInteractionTaskSource {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "UserInteractionTaskSource(...)")
    }
}

impl TaskSource for UserInteractionTaskSource {
    const NAME: TaskSourceName = TaskSourceName::UserInteraction;

    fn queue_with_canceller<T>(&self, task: T, canceller: &TaskCanceller) -> Result<(), ()>
    where
        T: TaskOnce + 'static,
    {
        let msg = MainThreadScriptMsg::Common(CommonScriptMsg::Task(
            ScriptThreadEventCategory::InputEvent,
            Box::new(canceller.wrap_task(task)),
            Some(self.1),
            UserInteractionTaskSource::NAME,
        ));
        self.0.send(msg).map_err(|_| ())
    }
}

impl UserInteractionTaskSource {
    pub fn queue_event(
        &self,
        target: &EventTarget,
        name: Atom,
        bubbles: EventBubbles,
        cancelable: EventCancelable,
        window: &Window,
    ) {
        let target = Trusted::new(target);
        let task = EventTask {
            target,
            name,
            bubbles,
            cancelable,
        };
        let _ = self.queue(task, window.upcast());
    }
}

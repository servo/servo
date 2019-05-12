/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::refcounted::Trusted;
use crate::dom::event::{EventBubbles, EventCancelable, EventTask, SimpleEventTask};
use crate::dom::eventtarget::EventTarget;
use crate::dom::window::Window;
use crate::script_runtime::{CommonScriptMsg, LocalScriptChan, ScriptThreadEventCategory};
use crate::task::{TaskCanceller, TaskOnce};
use crate::task_source::{TaskSource, TaskSourceName};
use msg::constellation_msg::PipelineId;
use servo_atoms::Atom;
use std::fmt;
use std::result::Result;

#[derive(JSTraceable)]
pub struct DOMManipulationTaskSource(pub Box<dyn LocalScriptChan>, pub PipelineId);

impl Clone for DOMManipulationTaskSource {
    fn clone(&self) -> DOMManipulationTaskSource {
        DOMManipulationTaskSource(self.0.clone(), self.1.clone())
    }
}

impl fmt::Debug for DOMManipulationTaskSource {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "DOMManipulationTaskSource(...)")
    }
}

impl TaskSource for DOMManipulationTaskSource {
    const NAME: TaskSourceName = TaskSourceName::DOMManipulation;

    fn queue_with_canceller<T>(&self, task: T, canceller: &TaskCanceller) -> Result<(), ()>
    where
        T: TaskOnce + 'static,
    {
        let msg_task = CommonScriptMsg::Task(
            ScriptThreadEventCategory::ScriptEvent,
            Box::new(canceller.wrap_task(task)),
            Some(self.1),
            DOMManipulationTaskSource::NAME,
        );

        self.0.send(msg_task).map_err(|_| ())
    }
}

impl DOMManipulationTaskSource {
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
            target: target,
            name: name,
            bubbles: bubbles,
            cancelable: cancelable,
        };
        let _ = self.queue(task, window.upcast());
    }

    pub fn queue_simple_event(&self, target: &EventTarget, name: Atom, window: &Window) {
        let target = Trusted::new(target);
        let _ = self.queue(SimpleEventTask { target, name }, window.upcast());
    }
}

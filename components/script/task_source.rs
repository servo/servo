/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::fmt;
use std::result::Result;

use base::id::PipelineId;
use malloc_size_of_derive::MallocSizeOf;
use servo_atoms::Atom;

use crate::dom::bindings::refcounted::Trusted;
use crate::dom::event::{EventBubbles, EventCancelable, EventTask, SimpleEventTask};
use crate::dom::eventtarget::EventTarget;
use crate::script_runtime::{CommonScriptMsg, ScriptChan, ScriptThreadEventCategory};
use crate::task::{TaskCanceller, TaskOnce};

/// The names of all task sources, used to differentiate TaskCancellers. Note: When adding a task
/// source, update this enum. Note: The HistoryTraversalTaskSource is not part of this, because it
/// doesn't implement TaskSource.
///
/// Note: When adding or removing a [`TaskSourceName`], be sure to also update the return value of
/// [`TaskSourceName::all`].
#[derive(Clone, Copy, Debug, Eq, Hash, JSTraceable, MallocSizeOf, PartialEq)]
pub enum TaskSourceName {
    DOMManipulation,
    FileReading,
    HistoryTraversal,
    Networking,
    PerformanceTimeline,
    PortMessage,
    UserInteraction,
    RemoteEvent,
    /// <https://html.spec.whatwg.org/multipage/#rendering-task-source>
    Rendering,
    MediaElement,
    WebSocket,
    Timer,
    /// <https://www.w3.org/TR/gamepad/#dfn-gamepad-task-source>
    Gamepad,
}

impl From<TaskSourceName> for ScriptThreadEventCategory {
    fn from(value: TaskSourceName) -> Self {
        match value {
            TaskSourceName::DOMManipulation => ScriptThreadEventCategory::ScriptEvent,
            TaskSourceName::FileReading => ScriptThreadEventCategory::FileRead,
            TaskSourceName::HistoryTraversal => ScriptThreadEventCategory::HistoryEvent,
            TaskSourceName::Networking => ScriptThreadEventCategory::NetworkEvent,
            TaskSourceName::PerformanceTimeline => {
                ScriptThreadEventCategory::PerformanceTimelineTask
            },
            TaskSourceName::PortMessage => ScriptThreadEventCategory::PortMessage,
            TaskSourceName::UserInteraction => ScriptThreadEventCategory::InputEvent,
            TaskSourceName::RemoteEvent => ScriptThreadEventCategory::NetworkEvent,
            TaskSourceName::Rendering => ScriptThreadEventCategory::Rendering,
            TaskSourceName::MediaElement => ScriptThreadEventCategory::ScriptEvent,
            TaskSourceName::WebSocket => ScriptThreadEventCategory::WebSocketEvent,
            TaskSourceName::Timer => ScriptThreadEventCategory::TimerEvent,
            TaskSourceName::Gamepad => ScriptThreadEventCategory::InputEvent,
        }
    }
}

impl TaskSourceName {
    pub fn all() -> &'static [TaskSourceName] {
        &[
            TaskSourceName::DOMManipulation,
            TaskSourceName::FileReading,
            TaskSourceName::HistoryTraversal,
            TaskSourceName::Networking,
            TaskSourceName::PerformanceTimeline,
            TaskSourceName::PortMessage,
            TaskSourceName::UserInteraction,
            TaskSourceName::RemoteEvent,
            TaskSourceName::Rendering,
            TaskSourceName::MediaElement,
            TaskSourceName::WebSocket,
            TaskSourceName::Timer,
            TaskSourceName::Gamepad,
        ]
    }
}

#[derive(JSTraceable, MallocSizeOf)]
pub(crate) struct TaskSource {
    #[ignore_malloc_size_of = "Need to push MallocSizeOf down into the ScriptChan trait implementations"]
    pub sender: Box<dyn ScriptChan + Send + 'static>,
    #[no_trace]
    pub pipeline_id: PipelineId,
    pub name: TaskSourceName,
    pub canceller: TaskCanceller,
}

impl TaskSource {
    pub(crate) fn queue<T>(&self, task: T) -> Result<(), ()>
    where
        T: TaskOnce + 'static,
    {
        let msg = CommonScriptMsg::Task(
            self.name.into(),
            Box::new(self.canceller.wrap_task(task)),
            Some(self.pipeline_id),
            self.name,
        );
        self.sender.send(msg).map_err(|_| ())
    }

    /// This queues a task that will not be cancelled when its associated global scope gets destroyed.
    pub(crate) fn queue_unconditionally<T>(&self, task: T) -> Result<(), ()>
    where
        T: TaskOnce + 'static,
    {
        self.sender.send(CommonScriptMsg::Task(
            self.name.into(),
            Box::new(task),
            Some(self.pipeline_id),
            self.name,
        ))
    }

    pub(crate) fn queue_simple_event(&self, target: &EventTarget, name: Atom) {
        let target = Trusted::new(target);
        let _ = self.queue(SimpleEventTask { target, name });
    }

    pub(crate) fn queue_event(
        &self,
        target: &EventTarget,
        name: Atom,
        bubbles: EventBubbles,
        cancelable: EventCancelable,
    ) {
        let target = Trusted::new(target);
        let _ = self.queue(EventTask {
            target,
            name,
            bubbles,
            cancelable,
        });
    }
}

impl Clone for TaskSource {
    fn clone(&self) -> Self {
        Self {
            sender: self.sender.as_boxed(),
            pipeline_id: self.pipeline_id,
            name: self.name,
            canceller: self.canceller.clone(),
        }
    }
}

impl fmt::Debug for TaskSource {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}(...)", self.name)
    }
}

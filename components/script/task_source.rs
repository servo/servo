/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::fmt;

use base::id::PipelineId;
use malloc_size_of_derive::MallocSizeOf;
use servo_atoms::Atom;

use crate::dom::bindings::refcounted::Trusted;
use crate::dom::event::{EventBubbles, EventCancelable, EventTask, SimpleEventTask};
use crate::dom::eventtarget::EventTarget;
use crate::messaging::{CommonScriptMsg, ScriptEventLoopSender};
use crate::script_runtime::ScriptThreadEventCategory;
use crate::task::{TaskCanceller, TaskOnce};
use crate::task_manager::TaskManager;

/// The names of all task sources, used to differentiate TaskCancellers. Note: When adding a task
/// source, update this enum. Note: The HistoryTraversalTaskSource is not part of this, because it
/// doesn't implement TaskSource.
///
/// Note: When adding or removing a [`TaskSourceName`], be sure to also update the return value of
/// [`TaskSourceName::all`].
#[derive(Clone, Copy, Debug, Eq, Hash, JSTraceable, MallocSizeOf, PartialEq)]
pub(crate) enum TaskSourceName {
    Canvas,
    DOMManipulation,
    FileReading,
    /// <https://drafts.csswg.org/css-font-loading/#task-source>
    FontLoading,
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
            TaskSourceName::Canvas => ScriptThreadEventCategory::ScriptEvent,
            TaskSourceName::DOMManipulation => ScriptThreadEventCategory::ScriptEvent,
            TaskSourceName::FileReading => ScriptThreadEventCategory::FileRead,
            TaskSourceName::FontLoading => ScriptThreadEventCategory::FontLoading,
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
    pub(crate) fn all() -> &'static [TaskSourceName] {
        &[
            TaskSourceName::Canvas,
            TaskSourceName::DOMManipulation,
            TaskSourceName::FileReading,
            TaskSourceName::FontLoading,
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

pub(crate) struct TaskSource<'task_manager> {
    pub(crate) task_manager: &'task_manager TaskManager,
    pub(crate) name: TaskSourceName,
}

impl TaskSource<'_> {
    /// Queue a task with the default canceller for this [`TaskSource`].
    pub(crate) fn queue(&self, task: impl TaskOnce + 'static) {
        let canceller = self.task_manager.canceller(self.name);
        if canceller.cancelled() {
            return;
        }

        self.queue_unconditionally(canceller.wrap_task(task))
    }

    /// This queues a task that will not be cancelled when its associated global scope gets destroyed.
    pub(crate) fn queue_unconditionally(&self, task: impl TaskOnce + 'static) {
        let sender = self.task_manager.sender();
        sender
            .as_ref()
            .expect("Tried to enqueue task for DedicatedWorker while not handling a message.")
            .send(CommonScriptMsg::Task(
                self.name.into(),
                Box::new(task),
                Some(self.task_manager.pipeline_id()),
                self.name,
            ))
            .expect("Tried to send a task on a task queue after shutdown.");
    }

    pub(crate) fn queue_simple_event(&self, target: &EventTarget, name: Atom) {
        let target = Trusted::new(target);
        self.queue(SimpleEventTask { target, name });
    }

    pub(crate) fn queue_event(
        &self,
        target: &EventTarget,
        name: Atom,
        bubbles: EventBubbles,
        cancelable: EventCancelable,
    ) {
        let target = Trusted::new(target);
        self.queue(EventTask {
            target,
            name,
            bubbles,
            cancelable,
        });
    }

    /// Convert this [`TaskSource`] into a [`SendableTaskSource`] suitable for sending
    /// to different threads.
    pub(crate) fn to_sendable(&self) -> SendableTaskSource {
        let sender = self.task_manager.sender();
        let sender = sender
            .as_ref()
            .expect("Tried to enqueue task for DedicatedWorker while not handling a message.")
            .clone();
        SendableTaskSource {
            sender,
            pipeline_id: self.task_manager.pipeline_id(),
            name: self.name,
            canceller: self.task_manager.canceller(self.name),
        }
    }
}

impl<'task_manager> From<TaskSource<'task_manager>> for SendableTaskSource {
    fn from(task_source: TaskSource<'task_manager>) -> Self {
        task_source.to_sendable()
    }
}

#[derive(JSTraceable, MallocSizeOf)]
pub(crate) struct SendableTaskSource {
    pub(crate) sender: ScriptEventLoopSender,
    #[no_trace]
    pub(crate) pipeline_id: PipelineId,
    pub(crate) name: TaskSourceName,
    pub(crate) canceller: TaskCanceller,
}

impl SendableTaskSource {
    pub(crate) fn queue(&self, task: impl TaskOnce + 'static) {
        if !self.canceller.cancelled() {
            self.queue_unconditionally(self.canceller.wrap_task(task))
        }
    }

    /// This queues a task that will not be cancelled when its associated global scope gets destroyed.
    pub(crate) fn queue_unconditionally(&self, task: impl TaskOnce + 'static) {
        if self
            .sender
            .send(CommonScriptMsg::Task(
                self.name.into(),
                Box::new(task),
                Some(self.pipeline_id),
                self.name,
            ))
            .is_err()
        {
            warn!(
                "Could not queue non-main-thread task {:?}. Likely tried to queue during shutdown.",
                self.name
            );
        }
    }
}

impl Clone for SendableTaskSource {
    fn clone(&self) -> Self {
        Self {
            sender: self.sender.clone(),
            pipeline_id: self.pipeline_id,
            name: self.name,
            canceller: self.canceller.clone(),
        }
    }
}

impl fmt::Debug for SendableTaskSource {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}(...)", self.name)
    }
}

/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use core::cell::RefCell;
use core::sync::atomic::Ordering;
use std::cell::Ref;
use std::collections::HashMap;

use base::id::PipelineId;

use crate::messaging::ScriptEventLoopSender;
use crate::task::TaskCanceller;
use crate::task_source::{TaskSource, TaskSourceName};

#[derive(JSTraceable, MallocSizeOf)]
enum TaskCancellers {
    /// A shared canceller that is used for workers, which can create multiple TaskManagers, but all
    /// of them need to have the same canceller flag for all task sources.
    Shared(TaskCanceller),
    /// For `Window` each `TaskSource` has its own canceller.
    OnePerTaskSource(RefCell<HashMap<TaskSourceName, TaskCanceller>>),
}

impl TaskCancellers {
    fn get(&self, name: TaskSourceName) -> TaskCanceller {
        match self {
            Self::Shared(canceller) => canceller.clone(),
            Self::OnePerTaskSource(map) => map.borrow_mut().entry(name).or_default().clone(),
        }
    }

    fn cancel_all_tasks_and_ignore_future_tasks(&self) {
        match self {
            Self::Shared(canceller) => canceller.cancelled.store(true, Ordering::SeqCst),
            Self::OnePerTaskSource(..) => {
                // We must create the canceller if they aren't created because we want future
                // tasks to be ignored completely.
                for task_source_name in TaskSourceName::all() {
                    self.get(*task_source_name)
                        .cancelled
                        .store(true, Ordering::SeqCst)
                }
            },
        }
    }

    fn cancel_pending_tasks_for_source(&self, task_source_name: TaskSourceName) {
        let Self::OnePerTaskSource(map) = self else {
            unreachable!(
                "It isn't possible to cancel pending tasks for Worker \
                 TaskManager's without ignoring future tasks."
            )
        };

        // Remove the canceller from the map so that the next time a task like this is
        // queued, it has a fresh, uncancelled canceller.
        if let Some(canceller) = map.borrow_mut().remove(&task_source_name) {
            // Cancel any tasks that use the current canceller.
            canceller.cancelled.store(true, Ordering::SeqCst);
        }
    }
}

macro_rules! task_source_functions {
    ($self:ident, $task_source:ident, $task_source_name:ident) => {
        pub(crate) fn $task_source(&$self) -> TaskSource {
            TaskSource {
                task_manager: $self,
                name: TaskSourceName::$task_source_name,
            }
        }
    };
}

#[derive(JSTraceable, MallocSizeOf)]
pub(crate) struct TaskManager {
    sender: RefCell<Option<ScriptEventLoopSender>>,
    #[no_trace]
    pipeline_id: PipelineId,
    cancellers: TaskCancellers,
}

impl TaskManager {
    pub(crate) fn new(
        sender: Option<ScriptEventLoopSender>,
        pipeline_id: PipelineId,
        shared_canceller: Option<TaskCanceller>,
    ) -> Self {
        let cancellers = match shared_canceller {
            Some(shared_canceller) => TaskCancellers::Shared(shared_canceller),
            None => TaskCancellers::OnePerTaskSource(Default::default()),
        };
        let sender = RefCell::new(sender);

        TaskManager {
            sender,
            pipeline_id,
            cancellers,
        }
    }

    pub(crate) fn pipeline_id(&self) -> PipelineId {
        self.pipeline_id
    }

    pub(crate) fn sender(&self) -> Ref<Option<ScriptEventLoopSender>> {
        self.sender.borrow()
    }

    pub(crate) fn canceller(&self, name: TaskSourceName) -> TaskCanceller {
        self.cancellers.get(name)
    }

    /// Update the sender for this [`TaskSource`]. This is used by dedicated workers, which only have a
    /// sender while handling messages (as their sender prevents the main thread Worker object from being
    /// garbage collected).
    pub(crate) fn set_sender(&self, sender: Option<ScriptEventLoopSender>) {
        *self.sender.borrow_mut() = sender;
    }

    /// Cancel all queued but unexecuted tasks and ignore all subsequently queued tasks.
    pub(crate) fn cancel_all_tasks_and_ignore_future_tasks(&self) {
        self.cancellers.cancel_all_tasks_and_ignore_future_tasks();
    }

    /// Cancel all queued but unexecuted tasks for the given task source, but subsequently queued
    /// tasks will not be ignored.
    pub(crate) fn cancel_pending_tasks_for_source(&self, task_source_name: TaskSourceName) {
        self.cancellers
            .cancel_pending_tasks_for_source(task_source_name);
    }

    task_source_functions!(self, canvas_blob_task_source, Canvas);
    task_source_functions!(self, dom_manipulation_task_source, DOMManipulation);
    task_source_functions!(self, file_reading_task_source, FileReading);
    task_source_functions!(self, font_loading_task_source, FontLoading);
    task_source_functions!(self, gamepad_task_source, Gamepad);
    task_source_functions!(self, media_element_task_source, MediaElement);
    task_source_functions!(self, networking_task_source, Networking);
    task_source_functions!(self, performance_timeline_task_source, PerformanceTimeline);
    task_source_functions!(self, port_message_queue, PortMessage);
    task_source_functions!(self, remote_event_task_source, RemoteEvent);
    task_source_functions!(self, rendering_task_source, Rendering);
    task_source_functions!(self, timer_task_source, Timer);
    task_source_functions!(self, user_interaction_task_source, UserInteraction);
    task_source_functions!(self, websocket_task_source, WebSocket);
}

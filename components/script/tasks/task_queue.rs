/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Machinery for [task-queue](https://html.spec.whatwg.org/multipage/#task-queue).

use std::collections::VecDeque;
use std::default::Default;

use crossbeam_channel::{self, Receiver};
use rustc_hash::{FxHashMap, FxHashSet};
use script_bindings::cell::DomRefCell;
use servo_base::id::PipelineId;

use crate::dom::worker::TrustedWorkerAddress;
use crate::runtime::script_runtime::ScriptThreadEventCategory;
use crate::tasks::task::TaskBox;
use crate::tasks::task_source::TaskSourceName;

#[derive(MallocSizeOf)]
pub(crate) struct QueuedTask {
    pub(crate) worker: Option<TrustedWorkerAddress>,
    pub(crate) event_category: ScriptThreadEventCategory,
    #[ignore_malloc_size_of = "TaskBox is difficult"]
    pub(crate) task: Box<dyn TaskBox>,
    pub(crate) pipeline_id: Option<PipelineId>,
    pub(crate) task_source: TaskSourceName,
}

/// Defining the operations used to convert from a msg T to a QueuedTask.
pub(crate) trait QueuedTaskConversion {
    fn pipeline_id(&self) -> Option<PipelineId>;
    fn into_queued_task(self) -> Option<QueuedTask>;
    fn from_queued_task(queued_task: QueuedTask) -> Self;
    fn inactive_msg() -> Self;
    fn wake_up_msg() -> Self;
    fn is_wake_up(&self) -> bool;
}

#[derive(MallocSizeOf)]
pub(crate) struct TaskQueue<T> {
    /// The original port on which the task-sources send tasks as messages.
    port: Receiver<T>,
    /// A queue from which the event-loop can drain tasks.
    msg_queue: DomRefCell<VecDeque<T>>,
    /// Tasks for not fully-active documents.
    inactive: DomRefCell<FxHashMap<PipelineId, VecDeque<QueuedTask>>>,
}

impl<T: QueuedTaskConversion> TaskQueue<T> {
    pub(crate) fn new(port: Receiver<T>) -> TaskQueue<T> {
        TaskQueue {
            port,
            msg_queue: DomRefCell::new(VecDeque::new()),
            inactive: Default::default(),
        }
    }

    /// Remove any tasks for an exiting pipeline. This only clears tasks that
    /// were stored from a previously inactive pipeline, as others will be drained
    /// the next time the queue is processed.
    pub(crate) fn remove_tasks_for_exiting_pipeline(&self, pipeline_id: &PipelineId) {
        self.inactive.borrow_mut().remove(pipeline_id);
    }

    /// Release previously held-back tasks for documents that are now fully-active.
    /// <https://html.spec.whatwg.org/multipage/#event-loop-processing-model:fully-active>
    fn release_tasks_for_fully_active_documents(
        &self,
        fully_active: &FxHashSet<PipelineId>,
    ) -> Vec<T> {
        self.inactive
            .borrow_mut()
            .iter_mut()
            .filter(|(pipeline_id, _)| fully_active.contains(pipeline_id))
            .flat_map(|(_, inactive_queue)| {
                inactive_queue
                    .drain(0..)
                    .map(|queued_task| T::from_queued_task(queued_task))
            })
            .collect()
    }

    /// Hold back tasks for currently not fully-active documents.
    /// <https://html.spec.whatwg.org/multipage/#event-loop-processing-model:fully-active>
    fn store_task_for_inactive_pipeline(&self, msg: T, pipeline_id: &PipelineId) {
        let mut inactive = self.inactive.borrow_mut();
        let inactive_queue = inactive.entry(*pipeline_id).or_default();
        inactive_queue.push_back(
            msg.into_queued_task()
                .expect("Incoming messages should always be convertible into queued tasks"),
        );
        let mut msg_queue = self.msg_queue.borrow_mut();
        if msg_queue.is_empty() {
            // Ensure there is at least one message.
            // Otherwise if the just stored inactive message
            // was the first and last of this iteration,
            // it will result in a spurious wake-up of the event-loop.
            msg_queue.push_back(T::inactive_msg());
        }
    }

    /// Drain the queue for the current iteration of the event-loop.
    pub(crate) fn take_tasks(&self, first_message: T, fully_active: &FxHashSet<PipelineId>) {
        // 1. Make any previously stored task from now fully-active document available.
        let mut incoming = self.release_tasks_for_fully_active_documents(fully_active);

        // 2. Process the first message (artifact of the fact that select always returns a message).
        if !first_message.is_wake_up() {
            incoming.push(first_message);
        }

        // 3. Process any other incoming message.
        while let Ok(message) = self.port.try_recv() {
            if !message.is_wake_up() {
                incoming.push(message);
            }
        }

        for msg in incoming {
            if let Some(pipeline_id) = msg.pipeline_id() &&
                !fully_active.contains(&pipeline_id)
            {
                self.store_task_for_inactive_pipeline(msg, &pipeline_id);
                continue;
            }
            self.msg_queue.borrow_mut().push_back(msg);
        }
    }

    /// Reset the queue for a new iteration of the event-loop,
    /// returning the port about whose readiness we want to be notified.
    pub(crate) fn select(&self) -> &crossbeam_channel::Receiver<T> {
        // We want to be notified when the script-port is ready to receive.
        // Hence that's the one we need to include in the select.
        &self.port
    }

    /// Take a message from the front of the queue, without waiting if empty.
    pub(crate) fn recv(&self) -> Result<T, ()> {
        self.msg_queue.borrow_mut().pop_front().ok_or(())
    }

    /// Take all tasks again and then run `recv()`.
    pub(crate) fn take_tasks_and_recv(
        &self,
        fully_active: &FxHashSet<PipelineId>,
    ) -> Result<T, ()> {
        self.take_tasks(T::wake_up_msg(), fully_active);
        self.recv()
    }
}

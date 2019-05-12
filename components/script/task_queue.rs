/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Machinery for [task-queue](https://html.spec.whatwg.org/multipage/#task-queue).

use crate::dom::bindings::cell::DomRefCell;
use crate::dom::worker::TrustedWorkerAddress;
use crate::script_runtime::ScriptThreadEventCategory;
use crate::script_thread::ScriptThread;
use crate::task::TaskBox;
use crate::task_source::TaskSourceName;
use crossbeam_channel::{self, Receiver, Sender};
use msg::constellation_msg::PipelineId;
use std::cell::Cell;
use std::collections::{HashMap, HashSet, VecDeque};
use std::default::Default;
use std::rc::Rc;

pub type QueuedTask = (
    Option<TrustedWorkerAddress>,
    ScriptThreadEventCategory,
    Box<dyn TaskBox>,
    Option<PipelineId>,
    TaskSourceName,
);

/// Defining the operations used to convert from a msg T to a QueuedTask.
pub trait QueuedTaskConversion {
    fn task_source_name(&self) -> Option<&TaskSourceName>;
    fn pipeline_id(&self) -> Option<PipelineId>;
    fn into_queued_task(self) -> Option<QueuedTask>;
    fn from_queued_task(queued_task: QueuedTask) -> Self;
    fn inactive_msg() -> Self;
    fn wake_up_msg() -> Self;
    fn is_wake_up(&self) -> bool;
}

pub struct TaskQueue<T> {
    /// The original port on which the task-sources send tasks as messages.
    port: Receiver<T>,
    /// A thread-local version of port.
    local_port: Rc<DomRefCell<VecDeque<T>>>,
    /// A sender to ensure the port doesn't block on select while there are throttled tasks.
    wake_up_sender: Sender<T>,
    /// A queue from which the event-loop can drain tasks.
    msg_queue: DomRefCell<VecDeque<T>>,
    /// A "business" counter, reset for each iteration of the event-loop
    taken_task_counter: Cell<u64>,
    /// Tasks that will be throttled for as long as we are "busy".
    throttled: DomRefCell<HashMap<TaskSourceName, VecDeque<QueuedTask>>>,
    /// Tasks for not fully-active documents.
    inactive: DomRefCell<HashMap<PipelineId, VecDeque<QueuedTask>>>,
}

impl<T: QueuedTaskConversion> TaskQueue<T> {
    pub fn new(port: Receiver<T>, wake_up_sender: Sender<T>) -> TaskQueue<T> {
        TaskQueue {
            port,
            local_port: Rc::new(DomRefCell::new(VecDeque::new())),
            wake_up_sender,
            msg_queue: DomRefCell::new(VecDeque::new()),
            taken_task_counter: Default::default(),
            throttled: Default::default(),
            inactive: Default::default(),
        }
    }

    pub fn local_port(&self) -> Rc<DomRefCell<VecDeque<T>>> {
        self.local_port.clone()
    }

    /// Release previously held-back tasks for documents that are now fully-active.
    /// https://html.spec.whatwg.org/multipage/#event-loop-processing-model:fully-active
    fn release_tasks_for_fully_active_documents(
        &self,
        fully_active: &HashSet<PipelineId>,
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
    /// https://html.spec.whatwg.org/multipage/#event-loop-processing-model:fully-active
    fn store_task_for_inactive_pipeline(&self, msg: T, pipeline_id: &PipelineId) {
        let mut inactive = self.inactive.borrow_mut();
        let inactive_queue = inactive.entry(pipeline_id.clone()).or_default();
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

    /// Process incoming tasks, immediately sending priority ones downstream,
    /// and categorizing potential throttles.
    fn process_incoming_tasks(&self, first_msg: T, fully_active: &HashSet<PipelineId>) {
        // 1. Make any previously stored task from now fully-active document available.
        let mut incoming = self.release_tasks_for_fully_active_documents(fully_active);

        // 2. Process the first message(artifact of the fact that select always returns a message).
        if !first_msg.is_wake_up() {
            incoming.push(first_msg);
        }

        // 3. Process any other incoming message.
        while let Ok(msg) = self.port.try_recv() {
            if !msg.is_wake_up() {
                incoming.push(msg);
            }
        }
        let mut locally_enqueued_tasks: Vec<T> = self.local_port.borrow_mut().drain(0..).collect();
        incoming.append(&mut locally_enqueued_tasks);

        // 4. Filter tasks from non-priority task-sources.
        let to_be_throttled: Vec<T> = incoming
            .drain_filter(|msg| {
                let task_source = match msg.task_source_name() {
                    Some(task_source) => task_source,
                    None => return false,
                };
                match task_source {
                    TaskSourceName::PerformanceTimeline => return true,
                    _ => {
                        // A task that will not be throttled, start counting "business"
                        self.taken_task_counter
                            .set(self.taken_task_counter.get() + 1);
                        return false;
                    },
                }
            })
            .collect();

        for msg in incoming {
            if let Some(pipeline_id) = msg.pipeline_id() {
                if !fully_active.contains(&pipeline_id) {
                    self.store_task_for_inactive_pipeline(msg, &pipeline_id);
                    continue;
                }
            }
            // Immediately send non-throttled tasks for processing.
            let _ = self.msg_queue.borrow_mut().push_back(msg);
        }

        for msg in to_be_throttled {
            // Categorize tasks per task queue.
            let (worker, category, boxed, pipeline_id, task_source) = match msg.into_queued_task() {
                Some(queued_task) => queued_task,
                None => unreachable!(
                    "A message to be throttled should always be convertible into a queued task"
                ),
            };
            let mut throttled_tasks = self.throttled.borrow_mut();
            throttled_tasks
                .entry(task_source.clone())
                .or_default()
                .push_back((worker, category, boxed, pipeline_id, task_source));
        }
    }

    /// Reset the queue for a new iteration of the event-loop,
    /// returning the port about whose readiness we want to be notified.
    pub fn select(&self) -> &crossbeam_channel::Receiver<T> {
        // This is a new iteration of the event-loop, so we reset the "business" counter.
        self.taken_task_counter.set(0);
        // We want to be notified when the script-port is ready to receive.
        // Hence that's the one we need to include in the select.
        &self.port
    }

    /// Ensure the event-loop wakes-up,
    /// in the case we only have tasks available from the local queue.
    pub fn ensure_wake_up(&self) {
        if !self.port.is_empty() {
            // The channel is not empty, so the event-loop will wake-up.
            return;
        }
        if self.local_port.borrow().len() > 0 {
            // We have locally enqueued tasks, yet the channel is empty.
            // Schedule a wake-up of the event-loop.
            let _ = self.wake_up_sender.send(T::wake_up_msg());
        }
    }

    /// Take a message from the front of the queue, without waiting if empty.
    pub fn recv(&self) -> Result<T, ()> {
        self.msg_queue.borrow_mut().pop_front().ok_or(())
    }

    /// Same as recv.
    pub fn try_recv(&self) -> Result<T, ()> {
        self.recv()
    }

    /// Drain the queue for the current iteration of the event-loop.
    /// Holding-back throttles above a given high-water mark.
    pub fn take_tasks(&self, first_msg: T) {
        // High-watermark: once reached, throttled tasks will be held-back.
        const PER_ITERATION_MAX: u64 = 5;
        let fully_active = ScriptThread::get_fully_active_document_ids();
        // Always first check for new tasks, but don't reset 'taken_task_counter'.
        self.process_incoming_tasks(first_msg, &fully_active);
        let mut throttled = self.throttled.borrow_mut();
        let mut throttled_length: usize = throttled.values().map(|queue| queue.len()).sum();
        let task_source_names = TaskSourceName::all();
        let mut task_source_cycler = task_source_names.iter().cycle();
        // "being busy", is defined as having more than x tasks for this loop's iteration.
        // As long as we're not busy, and there are throttled tasks left:
        loop {
            let max_reached = self.taken_task_counter.get() > PER_ITERATION_MAX;
            let none_left = throttled_length == 0;
            match (max_reached, none_left) {
                (_, true) => break,
                (true, false) => {
                    // We have reached the high-watermark for this iteration of the event-loop,
                    // yet also have throttled messages left in the queue.
                    // Ensure the select wakes up in the next iteration of the event-loop
                    let _ = self.wake_up_sender.send(T::wake_up_msg());
                    break;
                },
                (false, false) => {
                    // Cycle through non-priority task sources, taking one throttled task from each.
                    let task_source = task_source_cycler.next().unwrap();
                    let throttled_queue = match throttled.get_mut(&task_source) {
                        Some(queue) => queue,
                        None => continue,
                    };
                    let queued_task = match throttled_queue.pop_front() {
                        Some(queued_task) => queued_task,
                        None => continue,
                    };
                    let msg = T::from_queued_task(queued_task);

                    // Hold back tasks for currently inactive documents.
                    if let Some(pipeline_id) = msg.pipeline_id() {
                        if !fully_active.contains(&pipeline_id) {
                            self.store_task_for_inactive_pipeline(msg, &pipeline_id);
                            // Reduce the length of throttles,
                            // but don't add the task to "msg_queue",
                            // and neither increment "taken_task_counter".
                            throttled_length = throttled_length - 1;
                            continue;
                        }
                    }

                    // Make the task available for the event-loop to handle as a message.
                    let _ = self.msg_queue.borrow_mut().push_back(msg);
                    self.taken_task_counter
                        .set(self.taken_task_counter.get() + 1);
                    throttled_length = throttled_length - 1;
                },
            }
        }
    }
}

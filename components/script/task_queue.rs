/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Machinery for [task-queue](https://html.spec.whatwg.org/multipage/#task-queue).

use crate::dom::bindings::cell::DomRefCell;
use crate::dom::worker::TrustedWorkerAddress;
use crate::script_runtime::ScriptThreadEventCategory;
use crate::task::TaskBox;
use crate::task_source::TaskSourceName;
use msg::constellation_msg::PipelineId;
use servo_channel::{Receiver, Sender, base_channel};
use std::cell::Cell;
use std::collections::{HashMap, VecDeque};
use std::default::Default;

pub type QueuedTask = (
    Option<TrustedWorkerAddress>,
    ScriptThreadEventCategory,
    Box<TaskBox>,
    Option<PipelineId>,
    TaskSourceName,
);

/// Defining the operations used to convert from a msg T to a QueuedTask.
pub trait QueuedTaskConversion {
    fn task_source_name(&self) -> Option<&TaskSourceName>;
    fn into_queued_task(self) -> Option<QueuedTask>;
    fn from_queued_task(queued_task: QueuedTask) -> Self;
    fn wake_up_msg() -> Self;
    fn is_wake_up(&self) -> bool;
}

pub struct TaskQueue<T> {
    /// The original port on which the task-sources send tasks as messages.
    port: Receiver<T>,
    /// A sender to ensure the port doesn't block on select while there are throttled tasks.
    wake_up_sender: Sender<T>,
    /// A queue from which the event-loop can drain tasks.
    msg_queue: DomRefCell<VecDeque<T>>,
    /// A "business" counter, reset for each iteration of the event-loop
    taken_task_counter: Cell<u64>,
    /// Tasks that will be throttled for as long as we are "busy".
    throttled: DomRefCell<HashMap<TaskSourceName, VecDeque<QueuedTask>>>,
}

impl<T: QueuedTaskConversion> TaskQueue<T> {
    pub fn new(port: Receiver<T>, wake_up_sender: Sender<T>) -> TaskQueue<T> {
        TaskQueue {
            port,
            wake_up_sender,
            msg_queue: DomRefCell::new(VecDeque::new()),
            taken_task_counter: Default::default(),
            throttled: Default::default(),
        }
    }

    /// Process incoming tasks, immediately sending priority ones downstream,
    /// and categorizing potential throttles.
    fn process_incoming_tasks(&self, first_msg: T) {
        let mut incoming = Vec::with_capacity(self.port.len() + 1);
        if !first_msg.is_wake_up() {
            incoming.push(first_msg);
        }
        while let Some(msg) = self.port.try_recv() {
            if !msg.is_wake_up() {
                incoming.push(msg);
            }
        }

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
            }).collect();

        for msg in incoming {
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
                .or_insert(VecDeque::new())
                .push_back((worker, category, boxed, pipeline_id, task_source));
        }
    }

    /// Reset the queue for a new iteration of the event-loop,
    /// returning the port about whose readiness we want to be notified.
    pub fn select(&self) -> &base_channel::Receiver<T> {
        // This is a new iteration of the event-loop, so we reset the "business" counter.
        self.taken_task_counter.set(0);
        // We want to be notified when the script-port is ready to receive.
        // Hence that's the one we need to include in the select.
        self.port.select()
    }

    /// Take a message from the front of the queue, without waiting if empty.
    pub fn recv(&self) -> Option<T> {
        self.msg_queue.borrow_mut().pop_front()
    }

    /// Same as recv.
    pub fn try_recv(&self) -> Option<T> {
        self.recv()
    }

    /// Drain the queue for the current iteration of the event-loop.
    /// Holding-back throttles above a given high-water mark.
    pub fn take_tasks(&self, first_msg: T) {
        // High-watermark: once reached, throttled tasks will be held-back.
        const PER_ITERATION_MAX: u64 = 5;
        // Always first check for new tasks, but don't reset 'taken_task_counter'.
        self.process_incoming_tasks(first_msg);
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
                    let _ = self.msg_queue.borrow_mut().push_back(msg);
                    self.taken_task_counter
                        .set(self.taken_task_counter.get() + 1);
                    throttled_length = throttled_length - 1;
                },
            }
        }
    }
}

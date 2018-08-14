/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Machinery for [task-queue](https://html.spec.whatwg.org/multipage/#task-queue).

use dom::bindings::cell::DomRefCell;
use dom::bindings::trace::JSTraceable;
use js::jsapi::JSTracer;
use msg::constellation_msg::PipelineId;
use script_runtime::{CommonScriptMsg, ScriptThreadEventCategory};
use std::cell::Cell;
use std::collections::{HashMap, VecDeque};
use std::default::Default;
use std::sync::mpsc::Receiver;
use std::time::Instant;
use task::TaskBox;
use task_source::TaskSourceName;


type ThrottledTask = (ScriptThreadEventCategory, Box<TaskBox>, Option<PipelineId>);

#[allow(unsafe_code)]
unsafe impl JSTraceable for ThrottledTask {
    #[allow(unsafe_code)]
    unsafe fn trace(&self, _trc: *mut JSTracer) {
        // Do nothing
    }
}

// Channel-like interface, for use within a single thread.
pub struct MsgQueue <T> {
    internal: DomRefCell<VecDeque<T>>
}

#[allow(unsafe_code)]
unsafe impl<T> JSTraceable for MsgQueue<T> {
    #[allow(unsafe_code)]
    unsafe fn trace(&self, _trc: *mut JSTracer) {
        // Do nothing
    }
}

impl<T> MsgQueue <T> {
    fn new() -> MsgQueue<T> {
        MsgQueue {
            internal: DomRefCell::new(VecDeque::new())
        }
    }

    // Add a message to the back of the queue.
    fn send(&self, msg: T) {
        self.internal.borrow_mut().push_back(msg);
    }

    // Take a message from the front, without waiting if empty.
    pub fn recv(&self) -> Result<T, ()> {
        self.internal.borrow_mut().pop_front().ok_or(())
    }

    // Same as recv.
    pub fn try_recv(&self) -> Result<T, ()> {
        self.recv()
    }
}

pub trait CommonScriptMsgVariant {
    fn common_script_msg(&self) -> Option<&CommonScriptMsg>;
    fn into_common_script_msg(self) -> Option<CommonScriptMsg>;
    fn set_common_script_msg(script_msg: CommonScriptMsg) -> Self;
}

pub trait TaskPort<A, B> {
    fn recv_tasks(&self) -> Vec<A>;
    fn receiver(&self) -> &Receiver<B>;
}

pub struct TaskQueue<A, B> {
    // The original "script-port", on which the task-sources send tasks as messages.
    port: Box<TaskPort<A, B>>,
    // A queue from which the event-loop can drain tasks.
    msg_queue: MsgQueue<A>,
    // A "business" counter, reset for each iteration of the event-loop
    taken_task_counter: Cell<u64>,
    // The start of the previous iteration of the event-loop, used for timing.
    last_iteration: Cell<Option<Instant>>,
    // Tasks that will be throttled for as long as we are "busy".
    throttled: DomRefCell<HashMap<TaskSourceName, VecDeque<ThrottledTask>>>
}

#[allow(unsafe_code)]
unsafe impl<A, B> JSTraceable for TaskQueue<A, B> {
    #[allow(unsafe_code)]
    unsafe fn trace(&self, _trc: *mut JSTracer) {
        // Do nothing
    }
}

impl<A: CommonScriptMsgVariant, B: 'static> TaskQueue<A, B> {
    pub fn new(port: Receiver<B>) -> TaskQueue<A, B>
        where Receiver<B>: TaskPort<A, B>
    {
        TaskQueue {
            port: Box::new(port),
            msg_queue: MsgQueue::new(),
            taken_task_counter: Default::default(),
            last_iteration: Cell::new(None),
            throttled: Default::default(),
        }
    }

    // Process incoming tasks, immediately sending priority ones downstream,
    // and categorizing potential throttles.
    fn process_incoming_tasks(&self) {
        let mut non_throttled = self.port.recv_tasks();

        let to_be_throttled: Vec<A> = non_throttled.drain_filter(|msg|{
            let script_msg = match msg.common_script_msg() {
                Some(script_msg) => script_msg,
                None => return false,
            };
            let category = match script_msg {
                CommonScriptMsg::Task(category, _boxed, _pipeline_id) => category,
                _ => return false,
            };
            match category {
                ScriptThreadEventCategory::PerformanceTimelineTask => return true,
                _ => {
                    // A task that will not be throttled, start counting "business"
                    self.taken_task_counter.set(self.taken_task_counter.get() + 1);
                    return false
                },
            }
        }).collect();

        for msg in non_throttled {
            // Immediately send non-throttled tasks for processing.
            let _ = self.msg_queue.send(msg);
        }

        for msg in to_be_throttled {
            let script_msg = match msg.into_common_script_msg() {
                Some(script_msg) => script_msg,
                None => unreachable!(),
            };
            // Categorize throtted tasks per task queue.
            let (category, boxed, pipeline_id) = match script_msg {
                CommonScriptMsg::Task(category, boxed, pipeline_id) =>
                    (category, boxed, pipeline_id),
                _ => unreachable!(),
            };
            let task_source = match category {
                ScriptThreadEventCategory::PerformanceTimelineTask => TaskSourceName::PerformanceTimeline,
                _ => unreachable!(),
            };
            let mut throttled_tasks = self.throttled.borrow_mut();
            throttled_tasks
                .entry(task_source)
                .or_insert(VecDeque::new())
                .push_back((category, boxed, pipeline_id));
        }
    }

    // Time an iteration of the event-loop.
    fn time_event_loop(&self) {
        if let Some(instant) = self.last_iteration.get() {
            if instant.elapsed().as_secs() > 1 {
                warn!("Script thread was blocked, or idle, for more than one sec.");
            }
        }
        self.last_iteration.set(Some(Instant::now()));
    }

    // Reset the queue for a new iteration of the event-loop,
    // returning the port about whose readiness we want to be notified.
    pub fn select(&self) -> &Receiver<B> {
        // This is a new iterations of the event-loop, so we reset the "business" counter.
        self.taken_task_counter.set(0);
        // Time each iteration of the event-loop.
        self.time_event_loop();
        // We want to be notified when the script-port is ready to receive.
        // Hence that's the one we need to include in the select.
        self.port.receiver()
    }

    // Drain the queue for the current iteration of the event-loop.
    // Holding-back throttles above a given high-water mark.
    pub fn take_tasks(&self) -> &MsgQueue<A> {
        // High-watermark: once reached, throttled tasks will be held-back.
        let per_iteration_max = 5;
        // Always first check for new tasks, but don't reset 'taken_task_counter'.
        self.process_incoming_tasks();
        let mut throttled = self.throttled.borrow_mut();
        let mut throttled_length = throttled.values().fold(0, |acc, queue| acc + queue.len());
        let mut max_reached = self.taken_task_counter.get() > per_iteration_max;
        let mut none_left = throttled_length == 0;
        let task_source_names = TaskSourceName::all();
        let mut task_source_cycler = task_source_names.iter().cycle();
        // "being busy", is defined as having more than x tasks for this loop's iteration.
        // As long as we're not busy, and there are throttled tasks left:
        while !(max_reached || none_left) {
            // Cycle through non-priority task sources, taking one throttled task from each.
            let task_source = task_source_cycler.next().unwrap();
            let throttled_queue = match throttled.get_mut(&task_source) {
                Some(queue) => queue,
                None => continue,
            };
            let (category, boxed, pipeline_id) = match throttled_queue.pop_front() {
                Some((category, boxed, pipeline_id)) => (category, boxed, pipeline_id),
                None => continue,
            };
            let task = CommonScriptMsg::Task(category, boxed, pipeline_id);
            let msg = A::set_common_script_msg(task);
            let _ = self.msg_queue.send(msg);
            self.taken_task_counter.set(self.taken_task_counter.get() + 1);
            throttled_length = throttled_length - 1;
            max_reached = self.taken_task_counter.get() > per_iteration_max;
            none_left = throttled_length == 0;
        }
        &self.msg_queue
    }
}

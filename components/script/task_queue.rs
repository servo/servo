/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Machinery for [task-queue](https://html.spec.whatwg.org/multipage/#task-queue).

use dom::bindings::cell::DomRefCell;
use dom::bindings::trace::JSTraceable;
use js::jsapi::JSTracer;
use msg::constellation_msg::PipelineId;
use script_runtime::{CommonScriptMsg, ScriptThreadEventCategory};
use script_thread::MainThreadScriptMsg;
use std::cell::Cell;
use std::collections::{HashMap, VecDeque};
use std::default::Default;
use std::sync::mpsc::{channel, Receiver, Sender};
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

#[derive(JSTraceable)]
pub struct TaskQueue {
    script_port: Receiver<MainThreadScriptMsg>,
    task_sender: Sender<MainThreadScriptMsg>,
    task_port: Receiver<MainThreadScriptMsg>,
    taken_task_counter: Cell<u64>,
    throttled: DomRefCell<HashMap<TaskSourceName, VecDeque<ThrottledTask>>>
}

impl TaskQueue {
    pub fn new(script_port: Receiver<MainThreadScriptMsg>) -> TaskQueue {
        let (task_sender, task_port) = channel();
        TaskQueue {
            script_port,
            task_sender,
            task_port,
            taken_task_counter: Default::default(),
            throttled: Default::default(),
        }
    }

    // Process incoming tasks, immediately sending priority ones downstream,
    // and categorizing potential throttles.
    fn process_incoming_tasks(&self) {
        let mut non_throttled: Vec<MainThreadScriptMsg> = self.script_port.try_iter().collect();

        let to_be_throttled: Vec<MainThreadScriptMsg> = non_throttled.drain_filter(|msg|{
            let script_msg = match msg {
                MainThreadScriptMsg::Common(script_msg) => script_msg,
                _ => return false,
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
            let _ = self.task_sender.send(msg);
        }

        for msg in to_be_throttled {
            // Categorize throtted tasks per task queue.
            let (category, boxed, pipeline_id) = match msg {
                MainThreadScriptMsg::Common(CommonScriptMsg::Task(category, boxed, pipeline_id)) =>
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

    // Reset the queue for a new iteration of the event-loop,
    // returning the port about whose readiness we want to be notified.
    pub fn select(&self) -> &Receiver<MainThreadScriptMsg> {
        // This is a new iterations of the event-loop, so we reset the "business" counter.
        self.taken_task_counter.set(0);
        // We want to be notified when the script-port is ready to receive.
        // Hence that's the one we need to include in the select.
        &self.script_port
    }

    // Drain the queue for the current iteration of the event-loop.
    // Holding-back throttles above a given high-water mark.
    pub fn take_tasks(&self) -> &Receiver<MainThreadScriptMsg> {
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
            let msg = MainThreadScriptMsg::Common(task);
            let _ = self.task_sender.send(msg);
            self.taken_task_counter.set(self.taken_task_counter.get() + 1);
            throttled_length = throttled_length - 1;
            max_reached = self.taken_task_counter.get() > per_iteration_max;
            none_left = throttled_length == 0;
        }
        &self.task_port
    }
}

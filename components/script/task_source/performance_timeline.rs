/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// XXX The spec says that the performance timeline task source should be
//     a low priority task and it should be processed during idle periods.
//     We are currently treating this task queue as a normal priority queue.

use dom::bindings::inheritance::Castable;
use dom::bindings::refcounted::Trusted;
use dom::performance::{NotifyPerformanceObserverRunnable, Performance};
use dom::window::Window;
use script_thread::{MainThreadScriptMsg, Runnable, RunnableWrapper, ScriptThread};
use std::fmt;
use std::result::Result;
use std::sync::mpsc::Sender;
use task_source::TaskSource;

#[derive(Clone, JSTraceable)]
pub struct PerformanceTimelineTaskSource(pub Sender<MainThreadScriptMsg>);

impl fmt::Debug for PerformanceTimelineTaskSource {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "PerformanceTimelineTaskSource(...)")
    }
}

impl TaskSource for PerformanceTimelineTaskSource {
    fn queue_with_wrapper<T>(&self,
                             msg: Box<T>,
                             wrapper: &RunnableWrapper) -> Result<(), ()>
                             where T: Runnable + Send + 'static {
        let msg = PerformanceTimelineTask(wrapper.wrap_runnable(msg));
        self.0.send(MainThreadScriptMsg::PerformanceTimeline(msg)).map_err(|_| ())
    }
}

impl PerformanceTimelineTaskSource {
    pub fn queue_notification(&self, owner: &Performance, window: &Window) {
        let owner = Trusted::new(owner);
        let runnable = box NotifyPerformanceObserverRunnable::new(owner);
        let _ = self.queue(runnable, window.upcast());
    }
}

pub struct PerformanceTimelineTask(pub Box<Runnable + Send>);

impl fmt::Debug for PerformanceTimelineTask {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "PerformanceTimelineTask(...)")
    }
}

impl PerformanceTimelineTask {
    pub fn handle_task(self, script_thread: &ScriptThread) {
        self.0.main_thread_handler(script_thread);
    }
}

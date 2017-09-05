/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// XXX The spec says that the performance timeline task source should be
//     a low priority task and it should be processed during idle periods.
//     We are currently treating this task queue as a normal priority queue.

use dom::bindings::refcounted::Trusted;
use dom::globalscope::GlobalScope;
use dom::performance::Performance;
use script_runtime::{CommonScriptMsg, ScriptChan, ScriptThreadEventCategory};
use script_thread::{Runnable, RunnableWrapper};
use std::fmt;
use std::result::Result;
use task_source::TaskSource;

pub struct NotifyPerformanceObserverRunnable {
    owner: Trusted<Performance>,
}

impl NotifyPerformanceObserverRunnable {
    pub fn new(owner: Trusted<Performance>) -> Self {
        NotifyPerformanceObserverRunnable {
            owner,
        }
    }
}

impl Runnable for NotifyPerformanceObserverRunnable {
    fn handler(self: Box<NotifyPerformanceObserverRunnable>) {
        self.owner.root().notify_observers();
    }
}

#[derive(JSTraceable)]
pub struct PerformanceTimelineTaskSource(pub Box<ScriptChan + Send + 'static>);

impl Clone for PerformanceTimelineTaskSource {
    fn clone(&self) -> PerformanceTimelineTaskSource {
        PerformanceTimelineTaskSource(self.0.clone())
    }
}

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
        let msg = CommonScriptMsg::RunnableMsg(
            ScriptThreadEventCategory::PerformanceTimelineTask,
            wrapper.wrap_runnable(msg)
        );
        self.0.send(msg).map_err(|_| ())
    }
}

impl PerformanceTimelineTaskSource {
    pub fn queue_notification(&self, global: &GlobalScope) {
        let owner = Trusted::new(&*global.performance());
        let runnable = box NotifyPerformanceObserverRunnable::new(owner);
        let _ = self.queue(runnable, global);
    }
}

/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// XXX The spec says that the performance timeline task source should be
//     a low priority task and it should be processed during idle periods.
//     We are currently treating this task queue as a normal priority queue.

use dom::bindings::refcounted::Trusted;
use dom::globalscope::GlobalScope;
use script_runtime::{CommonScriptMsg, ScriptChan, ScriptThreadEventCategory};
use std::fmt;
use std::result::Result;
use task::{TaskCanceller, TaskOnce};
use task_source::TaskSource;

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
    fn queue_with_canceller<T>(
        &self,
        task: T,
        canceller: &TaskCanceller,
    ) -> Result<(), ()>
    where
        T: TaskOnce + 'static,
    {
        let msg = CommonScriptMsg::Task(
            ScriptThreadEventCategory::PerformanceTimelineTask,
            Box::new(canceller.wrap_task(task))
        );
        self.0.send(msg).map_err(|_| ())
    }
}

impl PerformanceTimelineTaskSource {
    pub fn queue_notification(&self, global: &GlobalScope) {
        let owner = Trusted::new(&*global.performance());
        // FIXME(nox): Why are errors silenced here?
        let _ = self.queue(
            task!(notify_performance_observers: move || {
                owner.root().notify_observers();
            }),
            global,
        );
    }
}

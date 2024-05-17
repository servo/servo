/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// XXX The spec says that the performance timeline task source should be
//     a low priority task and it should be processed during idle periods.
//     We are currently treating this task queue as a normal priority queue.

use std::fmt;
use std::result::Result;

use base::id::PipelineId;

use crate::dom::bindings::refcounted::Trusted;
use crate::dom::globalscope::GlobalScope;
use crate::script_runtime::{CommonScriptMsg, ScriptChan, ScriptThreadEventCategory};
use crate::task::{TaskCanceller, TaskOnce};
use crate::task_source::{TaskSource, TaskSourceName};

#[derive(JSTraceable)]
pub struct PerformanceTimelineTaskSource(
    pub Box<dyn ScriptChan + Send + 'static>,
    #[no_trace] pub PipelineId,
);

impl Clone for PerformanceTimelineTaskSource {
    fn clone(&self) -> PerformanceTimelineTaskSource {
        PerformanceTimelineTaskSource(self.0.clone(), self.1)
    }
}

impl fmt::Debug for PerformanceTimelineTaskSource {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "PerformanceTimelineTaskSource(...)")
    }
}

impl TaskSource for PerformanceTimelineTaskSource {
    const NAME: TaskSourceName = TaskSourceName::PerformanceTimeline;

    fn queue_with_canceller<T>(&self, task: T, canceller: &TaskCanceller) -> Result<(), ()>
    where
        T: TaskOnce + 'static,
    {
        let msg = CommonScriptMsg::Task(
            ScriptThreadEventCategory::PerformanceTimelineTask,
            Box::new(canceller.wrap_task(task)),
            Some(self.1),
            PerformanceTimelineTaskSource::NAME,
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

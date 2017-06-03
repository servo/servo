/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use devtools_traits::ScriptToDevtoolsControlMsg;
use dom::bindings::inheritance::Castable;
use dom::bindings::js::Root;
use dom::globalscope::GlobalScope;
use dom::paintworkletglobalscope::PaintWorkletGlobalScope;
use dom::paintworkletglobalscope::PaintWorkletTask;
use dom::testworkletglobalscope::TestWorkletGlobalScope;
use dom::testworkletglobalscope::TestWorkletTask;
use dom_struct::dom_struct;
use ipc_channel::ipc;
use ipc_channel::ipc::IpcSender;
use js::jsval::UndefinedValue;
use js::rust::Runtime;
use microtask::Microtask;
use microtask::MicrotaskQueue;
use msg::constellation_msg::PipelineId;
use net_traits::ResourceThreads;
use profile_traits::mem;
use profile_traits::time;
use script_traits::ScriptMsg;
use script_traits::TimerSchedulerMsg;
use servo_url::ImmutableOrigin;
use servo_url::MutableOrigin;
use servo_url::ServoUrl;

#[dom_struct]
/// https://drafts.css-houdini.org/worklets/#workletglobalscope
pub struct WorkletGlobalScope {
    /// The global for this worklet.
    globalscope: GlobalScope,
    /// The base URL for this worklet.
    base_url: ServoUrl,
    /// The microtask queue for this worklet
    microtask_queue: MicrotaskQueue,
}

impl WorkletGlobalScope {
    /// Create a new stack-allocated `WorkletGlobalScope`.
    pub fn new_inherited(pipeline_id: PipelineId,
                         base_url: ServoUrl,
                         init: &WorkletGlobalScopeInit)
                         -> WorkletGlobalScope {
        // Any timer events fired on this global are ignored.
        let (timer_event_chan, _) = ipc::channel().unwrap();
        WorkletGlobalScope {
            globalscope: GlobalScope::new_inherited(pipeline_id,
                                                    init.devtools_chan.clone(),
                                                    init.mem_profiler_chan.clone(),
                                                    init.time_profiler_chan.clone(),
                                                    init.constellation_chan.clone(),
                                                    init.scheduler_chan.clone(),
                                                    init.resource_threads.clone(),
                                                    timer_event_chan,
                                                    MutableOrigin::new(ImmutableOrigin::new_opaque())),
            base_url: base_url,
            microtask_queue: MicrotaskQueue::default(),
        }
    }

    /// Evaluate a JS script in this global.
    pub fn evaluate_js(&self, script: &str) -> bool {
        debug!("Evaluating JS.");
        rooted!(in (self.globalscope.get_cx()) let mut rval = UndefinedValue());
        self.globalscope.evaluate_js_on_global_with_result(&*script, rval.handle_mut())
    }

    /// The base URL of this global.
    pub fn base_url(&self) -> ServoUrl {
        self.base_url.clone()
    }

    /// Queue up a microtask to be executed in this global.
    pub fn enqueue_microtask(&self, job: Microtask) {
        self.microtask_queue.enqueue(job);
    }

    /// Perform any queued microtasks.
    pub fn perform_a_microtask_checkpoint(&self) {
        self.microtask_queue.checkpoint(|id| {
            let global = self.upcast::<GlobalScope>();
            assert_eq!(global.pipeline_id(), id);
            Some(Root::from_ref(global))
        });
    }

    /// Perform a worklet task
    pub fn perform_a_worklet_task(&self, task: WorkletTask) {
        match task {
            WorkletTask::Test(task) => match self.downcast::<TestWorkletGlobalScope>() {
                Some(global) => global.perform_a_worklet_task(task),
                None => warn!("This is not a test worklet."),
            },
            WorkletTask::Paint(task) => match self.downcast::<PaintWorkletGlobalScope>() {
                Some(global) => global.perform_a_worklet_task(task),
                None => warn!("This is not a paint worklet."),
            },
        }
    }
}

/// Resources required by workletglobalscopes
#[derive(Clone)]
pub struct WorkletGlobalScopeInit {
    /// Channel to a resource thread
    pub resource_threads: ResourceThreads,
    /// Channel to the memory profiler
    pub mem_profiler_chan: mem::ProfilerChan,
    /// Channel to the time profiler
    pub time_profiler_chan: time::ProfilerChan,
    /// Channel to devtools
    pub devtools_chan: Option<IpcSender<ScriptToDevtoolsControlMsg>>,
    /// Messages to send to constellation
    pub constellation_chan: IpcSender<ScriptMsg>,
    /// Message to send to the scheduler
    pub scheduler_chan: IpcSender<TimerSchedulerMsg>,
}

/// https://drafts.css-houdini.org/worklets/#worklet-global-scope-type
#[derive(Clone, Copy, Debug, HeapSizeOf, JSTraceable)]
pub enum WorkletGlobalScopeType {
    /// A servo-specific testing worklet
    Test,
    /// A paint worklet
    Paint,
}

impl WorkletGlobalScopeType {
    /// Create a new heap-allocated `WorkletGlobalScope`.
    pub fn new(&self,
               runtime: &Runtime,
               pipeline_id: PipelineId,
               base_url: ServoUrl,
               init: &WorkletGlobalScopeInit)
               -> Root<WorkletGlobalScope>
    {
        match *self {
            WorkletGlobalScopeType::Test =>
                Root::upcast(TestWorkletGlobalScope::new(runtime, pipeline_id, base_url, init)),
            WorkletGlobalScopeType::Paint =>
                Root::upcast(PaintWorkletGlobalScope::new(runtime, pipeline_id, base_url, init)),
        }
    }
}

/// A task which can be performed in the context of a worklet global.
pub enum WorkletTask {
    Test(TestWorkletTask),
    Paint(PaintWorkletTask),
}

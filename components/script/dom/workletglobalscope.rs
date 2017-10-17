/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use devtools_traits::ScriptToDevtoolsControlMsg;
use dom::bindings::inheritance::Castable;
use dom::bindings::root::DomRoot;
use dom::globalscope::GlobalScope;
use dom::paintworkletglobalscope::PaintWorkletGlobalScope;
use dom::paintworkletglobalscope::PaintWorkletTask;
use dom::testworkletglobalscope::TestWorkletGlobalScope;
use dom::testworkletglobalscope::TestWorkletTask;
use dom::worklet::WorkletExecutor;
use dom_struct::dom_struct;
use ipc_channel::ipc;
use ipc_channel::ipc::IpcSender;
use js::jsapi::JSContext;
use js::jsval::UndefinedValue;
use js::rust::Runtime;
use msg::constellation_msg::PipelineId;
use net_traits::ResourceThreads;
use net_traits::image_cache::ImageCache;
use profile_traits::mem;
use profile_traits::time;
use script_thread::MainThreadScriptMsg;
use script_traits::{Painter, ScriptMsg};
use script_traits::{ScriptToConstellationChan, TimerSchedulerMsg};
use servo_atoms::Atom;
use servo_url::ImmutableOrigin;
use servo_url::MutableOrigin;
use servo_url::ServoUrl;
use std::sync::Arc;
use std::sync::mpsc::Sender;

#[dom_struct]
/// <https://drafts.css-houdini.org/worklets/#workletglobalscope>
pub struct WorkletGlobalScope {
    /// The global for this worklet.
    globalscope: GlobalScope,
    /// The base URL for this worklet.
    base_url: ServoUrl,
    /// Sender back to the script thread
    #[ignore_heap_size_of = "channels are hard"]
    to_script_thread_sender: Sender<MainThreadScriptMsg>,
    /// Worklet task executor
    executor: WorkletExecutor,
}

impl WorkletGlobalScope {
    /// Create a new stack-allocated `WorkletGlobalScope`.
    pub fn new_inherited(
        pipeline_id: PipelineId,
        base_url: ServoUrl,
        executor: WorkletExecutor,
        init: &WorkletGlobalScopeInit,
    ) -> Self {
        // Any timer events fired on this global are ignored.
        let (timer_event_chan, _) = ipc::channel().unwrap();
        let script_to_constellation_chan = ScriptToConstellationChan {
            sender: init.to_constellation_sender.clone(),
            pipeline_id,
        };
        Self {
            globalscope: GlobalScope::new_inherited(
                pipeline_id,
                init.devtools_chan.clone(),
                init.mem_profiler_chan.clone(),
                init.time_profiler_chan.clone(),
                script_to_constellation_chan,
                init.scheduler_chan.clone(),
                init.resource_threads.clone(),
                timer_event_chan,
                MutableOrigin::new(ImmutableOrigin::new_opaque()),
                Default::default(),
            ),
            base_url,
            to_script_thread_sender: init.to_script_thread_sender.clone(),
            executor,
        }
    }

    /// Get the JS context.
    pub fn get_cx(&self) -> *mut JSContext {
        self.globalscope.get_cx()
    }

    /// Evaluate a JS script in this global.
    pub fn evaluate_js(&self, script: &str) -> bool {
        debug!("Evaluating Dom.");
        rooted!(in (self.globalscope.get_cx()) let mut rval = UndefinedValue());
        self.globalscope.evaluate_js_on_global_with_result(&*script, rval.handle_mut())
    }

    /// Register a paint worklet to the script thread.
    pub fn register_paint_worklet(
        &self,
        name: Atom,
        properties: Vec<Atom>,
        painter: Box<Painter>,
    ) {
        self.to_script_thread_sender
            .send(MainThreadScriptMsg::RegisterPaintWorklet {
                pipeline_id: self.globalscope.pipeline_id(),
                name,
                properties,
                painter,
            })
            .expect("Worklet thread outlived script thread.");
    }

    /// The base URL of this global.
    pub fn base_url(&self) -> ServoUrl {
        self.base_url.clone()
    }

    /// The worklet executor.
    pub fn executor(&self) -> WorkletExecutor {
        self.executor.clone()
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
    /// Channel to the main script thread
    pub to_script_thread_sender: Sender<MainThreadScriptMsg>,
    /// Channel to a resource thread
    pub resource_threads: ResourceThreads,
    /// Channel to the memory profiler
    pub mem_profiler_chan: mem::ProfilerChan,
    /// Channel to the time profiler
    pub time_profiler_chan: time::ProfilerChan,
    /// Channel to devtools
    pub devtools_chan: Option<IpcSender<ScriptToDevtoolsControlMsg>>,
    /// Messages to send to constellation
    pub to_constellation_sender: IpcSender<(PipelineId, ScriptMsg)>,
    /// Message to send to the scheduler
    pub scheduler_chan: IpcSender<TimerSchedulerMsg>,
    /// The image cache
    pub image_cache: Arc<ImageCache>,
}

/// <https://drafts.css-houdini.org/worklets/#worklet-global-scope-type>
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
               executor: WorkletExecutor,
               init: &WorkletGlobalScopeInit)
               -> DomRoot<WorkletGlobalScope>
    {
        match *self {
            WorkletGlobalScopeType::Test =>
                DomRoot::upcast(TestWorkletGlobalScope::new(runtime, pipeline_id, base_url, executor, init)),
            WorkletGlobalScopeType::Paint =>
                DomRoot::upcast(PaintWorkletGlobalScope::new(runtime, pipeline_id, base_url, executor, init)),
        }
    }
}

/// A task which can be performed in the context of a worklet global.
pub enum WorkletTask {
    Test(TestWorkletTask),
    Paint(PaintWorkletTask),
}

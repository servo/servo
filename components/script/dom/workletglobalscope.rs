/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::borrow::Cow;
use std::sync::Arc;

use base::id::PipelineId;
use crossbeam_channel::Sender;
use devtools_traits::ScriptToDevtoolsControlMsg;
use dom_struct::dom_struct;
use ipc_channel::ipc::IpcSender;
use js::jsval::UndefinedValue;
use js::rust::Runtime;
use net_traits::image_cache::ImageCache;
use net_traits::ResourceThreads;
use profile_traits::{mem, time};
use script_traits::{Painter, ScriptMsg, ScriptToConstellationChan, TimerSchedulerMsg};
use servo_atoms::Atom;
use servo_url::{ImmutableOrigin, MutableOrigin, ServoUrl};

use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::root::DomRoot;
use crate::dom::globalscope::GlobalScope;
use crate::dom::identityhub::Identities;
use crate::dom::paintworkletglobalscope::{PaintWorkletGlobalScope, PaintWorkletTask};
use crate::dom::testworkletglobalscope::{TestWorkletGlobalScope, TestWorkletTask};
use crate::dom::worklet::WorkletExecutor;
use crate::script_module::ScriptFetchOptions;
use crate::script_runtime::JSContext;
use crate::script_thread::MainThreadScriptMsg;

#[dom_struct]
/// <https://drafts.css-houdini.org/worklets/#workletglobalscope>
pub struct WorkletGlobalScope {
    /// The global for this worklet.
    globalscope: GlobalScope,
    /// The base URL for this worklet.
    #[no_trace]
    base_url: ServoUrl,
    /// Sender back to the script thread
    #[ignore_malloc_size_of = "channels are hard"]
    #[no_trace]
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
                MutableOrigin::new(ImmutableOrigin::new_opaque()),
                None,
                Default::default(),
                init.is_headless,
                init.user_agent.clone(),
                init.gpu_id_hub.clone(),
                init.inherited_secure_context,
            ),
            base_url,
            to_script_thread_sender: init.to_script_thread_sender.clone(),
            executor,
        }
    }

    /// Get the JS context.
    pub fn get_cx() -> JSContext {
        GlobalScope::get_cx()
    }

    /// Evaluate a JS script in this global.
    pub fn evaluate_js(&self, script: &str) -> bool {
        debug!("Evaluating Dom in a worklet.");
        rooted!(in (*GlobalScope::get_cx()) let mut rval = UndefinedValue());
        self.globalscope.evaluate_js_on_global_with_result(
            script,
            rval.handle_mut(),
            ScriptFetchOptions::default_classic_script(&self.globalscope),
            self.globalscope.api_base_url(),
        )
    }

    /// Register a paint worklet to the script thread.
    pub fn register_paint_worklet(
        &self,
        name: Atom,
        properties: Vec<Atom>,
        painter: Box<dyn Painter>,
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
    pub image_cache: Arc<dyn ImageCache>,
    /// True if in headless mode
    pub is_headless: bool,
    /// An optional string allowing the user agent to be set for testing
    pub user_agent: Cow<'static, str>,
    /// Identity manager for WebGPU resources
    pub gpu_id_hub: Arc<Identities>,
    /// Is considered secure
    pub inherited_secure_context: Option<bool>,
}

/// <https://drafts.css-houdini.org/worklets/#worklet-global-scope-type>
#[derive(Clone, Copy, Debug, JSTraceable, MallocSizeOf)]
pub enum WorkletGlobalScopeType {
    /// A servo-specific testing worklet
    Test,
    /// A paint worklet
    Paint,
}

impl WorkletGlobalScopeType {
    /// Create a new heap-allocated `WorkletGlobalScope`.
    pub fn new(
        &self,
        runtime: &Runtime,
        pipeline_id: PipelineId,
        base_url: ServoUrl,
        executor: WorkletExecutor,
        init: &WorkletGlobalScopeInit,
    ) -> DomRoot<WorkletGlobalScope> {
        match *self {
            WorkletGlobalScopeType::Test => DomRoot::upcast(TestWorkletGlobalScope::new(
                runtime,
                pipeline_id,
                base_url,
                executor,
                init,
            )),
            WorkletGlobalScopeType::Paint => DomRoot::upcast(PaintWorkletGlobalScope::new(
                runtime,
                pipeline_id,
                base_url,
                executor,
                init,
            )),
        }
    }
}

/// A task which can be performed in the context of a worklet global.
pub enum WorkletTask {
    Test(TestWorkletTask),
    Paint(PaintWorkletTask),
}

/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::sync::Arc;

use base::id::PipelineId;
use constellation_traits::{ScriptToConstellationChan, ScriptToConstellationMessage};
use crossbeam_channel::Sender;
use devtools_traits::ScriptToDevtoolsControlMsg;
use dom_struct::dom_struct;
use ipc_channel::ipc::IpcSender;
use js::jsval::UndefinedValue;
use js::rust::Runtime;
use net_traits::ResourceThreads;
use net_traits::image_cache::ImageCache;
use profile_traits::{mem, time};
use script_bindings::realms::InRealm;
use script_traits::Painter;
use servo_url::{ImmutableOrigin, MutableOrigin, ServoUrl};
use stylo_atoms::Atom;

use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::trace::CustomTraceable;
use crate::dom::bindings::utils::define_all_exposed_interfaces;
use crate::dom::globalscope::GlobalScope;
use crate::dom::paintworkletglobalscope::{PaintWorkletGlobalScope, PaintWorkletTask};
#[cfg(feature = "testbinding")]
use crate::dom::testworkletglobalscope::{TestWorkletGlobalScope, TestWorkletTask};
#[cfg(feature = "webgpu")]
use crate::dom::webgpu::identityhub::IdentityHub;
use crate::dom::worklet::WorkletExecutor;
use crate::messaging::MainThreadScriptMsg;
use crate::realms::enter_realm;
use crate::script_module::ScriptFetchOptions;
use crate::script_runtime::{CanGc, IntroductionType, JSContext};

#[dom_struct]
/// <https://drafts.css-houdini.org/worklets/#workletglobalscope>
pub(crate) struct WorkletGlobalScope {
    /// The global for this worklet.
    globalscope: GlobalScope,
    /// The base URL for this worklet.
    #[no_trace]
    base_url: ServoUrl,
    /// Sender back to the script thread
    to_script_thread_sender: Sender<MainThreadScriptMsg>,
    /// Worklet task executor
    executor: WorkletExecutor,
}

impl WorkletGlobalScope {
    /// Create a new heap-allocated `WorkletGlobalScope`.
    pub(crate) fn new(
        scope_type: WorkletGlobalScopeType,
        runtime: &Runtime,
        pipeline_id: PipelineId,
        base_url: ServoUrl,
        executor: WorkletExecutor,
        init: &WorkletGlobalScopeInit,
    ) -> DomRoot<WorkletGlobalScope> {
        let scope: DomRoot<WorkletGlobalScope> = match scope_type {
            #[cfg(feature = "testbinding")]
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
        };

        let realm = enter_realm(&*scope);
        define_all_exposed_interfaces(scope.upcast(), InRealm::entered(&realm), CanGc::note());

        scope
    }

    /// Create a new stack-allocated `WorkletGlobalScope`.
    pub(crate) fn new_inherited(
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
                init.resource_threads.clone(),
                MutableOrigin::new(ImmutableOrigin::new_opaque()),
                base_url.clone(),
                None,
                Default::default(),
                #[cfg(feature = "webgpu")]
                init.gpu_id_hub.clone(),
                init.inherited_secure_context,
                false,
            ),
            base_url,
            to_script_thread_sender: init.to_script_thread_sender.clone(),
            executor,
        }
    }

    /// Get the JS context.
    pub(crate) fn get_cx() -> JSContext {
        GlobalScope::get_cx()
    }

    /// Evaluate a JS script in this global.
    pub(crate) fn evaluate_js(&self, script: &str, can_gc: CanGc) -> bool {
        debug!("Evaluating Dom in a worklet.");
        rooted!(in (*GlobalScope::get_cx()) let mut rval = UndefinedValue());
        self.globalscope.evaluate_js_on_global_with_result(
            script,
            rval.handle_mut(),
            ScriptFetchOptions::default_classic_script(&self.globalscope),
            self.globalscope.api_base_url(),
            can_gc,
            Some(IntroductionType::WORKLET),
        )
    }

    /// Register a paint worklet to the script thread.
    pub(crate) fn register_paint_worklet(
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
    pub(crate) fn base_url(&self) -> ServoUrl {
        self.base_url.clone()
    }

    /// The worklet executor.
    pub(crate) fn executor(&self) -> WorkletExecutor {
        self.executor.clone()
    }

    /// Perform a worklet task
    pub(crate) fn perform_a_worklet_task(&self, task: WorkletTask) {
        match task {
            #[cfg(feature = "testbinding")]
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
pub(crate) struct WorkletGlobalScopeInit {
    /// Channel to the main script thread
    pub(crate) to_script_thread_sender: Sender<MainThreadScriptMsg>,
    /// Channel to a resource thread
    pub(crate) resource_threads: ResourceThreads,
    /// Channel to the memory profiler
    pub(crate) mem_profiler_chan: mem::ProfilerChan,
    /// Channel to the time profiler
    pub(crate) time_profiler_chan: time::ProfilerChan,
    /// Channel to devtools
    pub(crate) devtools_chan: Option<IpcSender<ScriptToDevtoolsControlMsg>>,
    /// Messages to send to constellation
    pub(crate) to_constellation_sender: IpcSender<(PipelineId, ScriptToConstellationMessage)>,
    /// The image cache
    pub(crate) image_cache: Arc<dyn ImageCache>,
    /// Identity manager for WebGPU resources
    #[cfg(feature = "webgpu")]
    pub(crate) gpu_id_hub: Arc<IdentityHub>,
    /// Is considered secure
    pub(crate) inherited_secure_context: Option<bool>,
}

/// <https://drafts.css-houdini.org/worklets/#worklet-global-scope-type>
#[derive(Clone, Copy, Debug, JSTraceable, MallocSizeOf)]
pub(crate) enum WorkletGlobalScopeType {
    /// A servo-specific testing worklet
    #[cfg(feature = "testbinding")]
    Test,
    /// A paint worklet
    Paint,
}

/// A task which can be performed in the context of a worklet global.
pub(crate) enum WorkletTask {
    #[cfg(feature = "testbinding")]
    Test(TestWorkletTask),
    Paint(PaintWorkletTask),
}

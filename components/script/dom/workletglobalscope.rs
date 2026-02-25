/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::borrow::Cow;
use std::sync::Arc;

use base::generic_channel::{GenericCallback, GenericSender};
use base::id::{PipelineId, WebViewId};
use constellation_traits::{ScriptToConstellationChan, ScriptToConstellationMessage};
use crossbeam_channel::Sender;
use devtools_traits::ScriptToDevtoolsControlMsg;
use dom_struct::dom_struct;
use embedder_traits::{JavaScriptEvaluationError, ScriptToEmbedderChan};
use js::jsval::UndefinedValue;
use net_traits::ResourceThreads;
use net_traits::image_cache::ImageCache;
use profile_traits::{mem, time};
use script_traits::Painter;
use servo_url::{ImmutableOrigin, MutableOrigin, ServoUrl};
use storage_traits::StorageThreads;
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
use crate::realms::enter_auto_realm;
use crate::script_runtime::{IntroductionType, JSContext};

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
    #[expect(clippy::too_many_arguments)]
    /// Create a new heap-allocated `WorkletGlobalScope`.
    pub(crate) fn new(
        scope_type: WorkletGlobalScopeType,
        webview_id: WebViewId,
        pipeline_id: PipelineId,
        base_url: ServoUrl,
        inherited_secure_context: Option<bool>,
        executor: WorkletExecutor,
        init: &WorkletGlobalScopeInit,
        cx: &mut js::context::JSContext,
    ) -> DomRoot<WorkletGlobalScope> {
        let scope: DomRoot<WorkletGlobalScope> = match scope_type {
            #[cfg(feature = "testbinding")]
            WorkletGlobalScopeType::Test => DomRoot::upcast(TestWorkletGlobalScope::new(
                webview_id,
                pipeline_id,
                base_url,
                inherited_secure_context,
                executor,
                init,
                cx,
            )),
            WorkletGlobalScopeType::Paint => DomRoot::upcast(PaintWorkletGlobalScope::new(
                webview_id,
                pipeline_id,
                base_url,
                inherited_secure_context,
                executor,
                init,
                cx,
            )),
        };

        let mut realm = enter_auto_realm(cx, &*scope);
        let mut realm = realm.current_realm();
        define_all_exposed_interfaces(&mut realm, scope.upcast());

        scope
    }

    /// Create a new stack-allocated `WorkletGlobalScope`.
    pub(crate) fn new_inherited(
        webview_id: WebViewId,
        pipeline_id: PipelineId,
        base_url: ServoUrl,
        inherited_secure_context: Option<bool>,
        executor: WorkletExecutor,
        init: &WorkletGlobalScopeInit,
    ) -> Self {
        let script_to_constellation_chan = ScriptToConstellationChan {
            sender: init.to_constellation_sender.clone(),
            webview_id,
            pipeline_id,
        };
        Self {
            globalscope: GlobalScope::new_inherited(
                pipeline_id,
                init.devtools_chan.clone(),
                init.mem_profiler_chan.clone(),
                init.time_profiler_chan.clone(),
                script_to_constellation_chan,
                init.to_embedder_sender.clone(),
                init.resource_threads.clone(),
                init.storage_threads.clone(),
                MutableOrigin::new(ImmutableOrigin::new_opaque()),
                base_url.clone(),
                None,
                #[cfg(feature = "webgpu")]
                init.gpu_id_hub.clone(),
                inherited_secure_context,
                false,
                None, // font_context
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
    pub(crate) fn evaluate_js(
        &self,
        script: Cow<'_, str>,
        cx: &mut js::context::JSContext,
    ) -> Result<(), JavaScriptEvaluationError> {
        let mut realm = enter_auto_realm(cx, self);
        let cx = &mut realm.current_realm();

        debug!("Evaluating Dom in a worklet.");
        rooted!(&in(cx) let mut rval = UndefinedValue());
        self.globalscope.evaluate_js_on_global(
            cx,
            script,
            "",
            Some(IntroductionType::WORKLET),
            rval.handle_mut(),
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
    /// Channels to the [`StorageThreads`].
    pub(crate) storage_threads: StorageThreads,
    /// Channel to the memory profiler
    pub(crate) mem_profiler_chan: mem::ProfilerChan,
    /// Channel to the time profiler
    pub(crate) time_profiler_chan: time::ProfilerChan,
    /// Channel to devtools
    pub(crate) devtools_chan: Option<GenericCallback<ScriptToDevtoolsControlMsg>>,
    /// Messages to send to constellation
    pub(crate) to_constellation_sender:
        GenericSender<(WebViewId, PipelineId, ScriptToConstellationMessage)>,
    /// Messages to send to the Embedder
    pub(crate) to_embedder_sender: ScriptToEmbedderChan,
    /// The image cache
    pub(crate) image_cache: Arc<dyn ImageCache>,
    /// Identity manager for WebGPU resources
    #[cfg(feature = "webgpu")]
    pub(crate) gpu_id_hub: Arc<IdentityHub>,
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

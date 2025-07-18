/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::sync::Arc;

use base::id::PipelineId;
use constellation_traits::ScriptToConstellationChan;
use dom_struct::dom_struct;
use js::jsval::UndefinedValue;
use js::rust::Runtime;
use net_traits::ResourceThreads;
use profile_traits::{mem, time};
use servo_url::{ImmutableOrigin, MutableOrigin, ServoUrl};

use crate::dom::bindings::codegen::Bindings::DebuggerGlobalScopeBinding;
use crate::dom::bindings::root::DomRoot;
use crate::dom::globalscope::GlobalScope;
#[cfg(feature = "testbinding")]
#[cfg(feature = "webgpu")]
use crate::dom::webgpu::identityhub::IdentityHub;
use crate::script_module::ScriptFetchOptions;
use crate::script_runtime::{CanGc, JSContext};

#[dom_struct]
/// Global scope for interacting with the devtools Debugger API.
///
/// <https://firefox-source-docs.mozilla.org/js/Debugger/>
pub(crate) struct DebuggerGlobalScope {
    global_scope: GlobalScope,
}

impl DebuggerGlobalScope {
    /// Create a new heap-allocated `DebuggerGlobalScope`.
    #[allow(unsafe_code)]
    pub(crate) fn new(
        runtime: &Runtime,
        mem_profiler_chan: mem::ProfilerChan,
        time_profiler_chan: time::ProfilerChan,
        script_to_constellation_chan: ScriptToConstellationChan,
        resource_threads: ResourceThreads,
        #[cfg(feature = "webgpu")] gpu_id_hub: Arc<IdentityHub>,
    ) -> DomRoot<Self> {
        let global = Box::new(Self {
            global_scope: GlobalScope::new_inherited(
                PipelineId::new(), // ??? or TEST_PIPELINE_ID, but that seems worse
                None,              // ? if needed, see script_thread:745
                mem_profiler_chan,
                time_profiler_chan,
                script_to_constellation_chan, // wrap it in a ScriptToConstellationChan
                resource_threads,
                MutableOrigin::new(ImmutableOrigin::new_opaque()),
                ServoUrl::parse_with_base(None, "about:internal/debugger")
                    .expect("Guaranteed by argument"), // ???
                None,
                Default::default(),
                gpu_id_hub,
                None, // ? if needed, see script_thread:745
                false,
            ),
        });

        unsafe {
            DebuggerGlobalScopeBinding::Wrap::<crate::DomTypeHolder>(
                JSContext::from_ptr(runtime.cx()),
                global,
            )
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
        self.global_scope.evaluate_js_on_global_with_result(
            script,
            rval.handle_mut(),
            ScriptFetchOptions::default_classic_script(&self.global_scope),
            self.global_scope.api_base_url(),
            can_gc,
        )
    }
}

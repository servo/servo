/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::sync::Arc;
use std::time::Duration;

use base::id::PipelineId;
use constellation_traits::ScriptToConstellationChan;
use crossbeam_channel::Sender;
use dom_struct::dom_struct;
use js::gc::HandleValue;
use js::jsval::UndefinedValue;
use js::rust::Runtime;
use js::rust::wrappers::JS_DefineDebuggerObject;
use net_traits::ResourceThreads;
use profile_traits::{mem, time};
use script_bindings::codegen::GenericBindings::DebuggerGlobalScopeBinding::DebuggerGlobalScopeMethods;
use script_bindings::realms::InRealm;
use script_bindings::reflector::DomObject;
use servo_url::{ImmutableOrigin, MutableOrigin, ServoUrl};

use crate::dom::bindings::codegen::Bindings::DebuggerGlobalScopeBinding;
use crate::dom::bindings::codegen::UnionTypes::StringOrFunction;
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::trace::CustomTraceable;
use crate::dom::bindings::utils::define_all_exposed_interfaces;
use crate::dom::globalscope::GlobalScope;
#[cfg(feature = "testbinding")]
#[cfg(feature = "webgpu")]
use crate::dom::webgpu::identityhub::IdentityHub;
use crate::messaging::{MainThreadScriptMsg, ScriptEventLoopSender};
use crate::realms::enter_realm;
use crate::script_module::ScriptFetchOptions;
use crate::script_runtime::{CanGc, JSContext};
use crate::timers::{IsInterval, TimerCallback};

#[dom_struct]
/// Global scope for interacting with the devtools Debugger API.
///
/// <https://firefox-source-docs.mozilla.org/js/Debugger/>
pub(crate) struct DebuggerGlobalScope {
    global_scope: GlobalScope,
    script_chan: Sender<MainThreadScriptMsg>,
}

impl DebuggerGlobalScope {
    /// Create a new heap-allocated `DebuggerGlobalScope`.
    #[allow(unsafe_code)]
    pub(crate) fn new(
        runtime: &Runtime,
        script_chan: Sender<MainThreadScriptMsg>,
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
            script_chan,
        });
        let global = unsafe {
            DebuggerGlobalScopeBinding::Wrap::<crate::DomTypeHolder>(
                JSContext::from_ptr(runtime.cx()),
                global,
            )
        };

        let realm = enter_realm(&*global);
        define_all_exposed_interfaces(global.upcast(), InRealm::entered(&realm), CanGc::note());
        // TODO: what invariants do we need to uphold for the unsafe call?
        assert!(unsafe {
            JS_DefineDebuggerObject(
                *Self::get_cx(),
                global.global_scope.reflector().get_jsobject(),
            )
        });

        global
    }

    /// Get the JS context.
    pub(crate) fn get_cx() -> JSContext {
        GlobalScope::get_cx()
    }

    pub(crate) fn as_global_scope(&self) -> &GlobalScope {
        self.upcast::<GlobalScope>()
    }

    pub(crate) fn event_loop_sender(&self) -> ScriptEventLoopSender {
        ScriptEventLoopSender::MainThread(self.script_chan.clone())
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

impl DebuggerGlobalScopeMethods<crate::DomTypeHolder> for DebuggerGlobalScope {
    // https://html.spec.whatwg.org/multipage/#dom-windowtimers-settimeout
    fn SetTimeout(
        &self,
        _cx: JSContext,
        callback: StringOrFunction,
        timeout: i32,
        args: Vec<HandleValue>,
    ) -> i32 {
        let callback = match callback {
            StringOrFunction::String(i) => TimerCallback::StringTimerCallback(i),
            StringOrFunction::Function(i) => TimerCallback::FunctionTimerCallback(i),
        };
        self.as_global_scope().set_timeout_or_interval(
            callback,
            args,
            Duration::from_millis(timeout.max(0) as u64),
            IsInterval::NonInterval,
        )
    }

    // https://html.spec.whatwg.org/multipage/#dom-windowtimers-cleartimeout
    fn ClearTimeout(&self, handle: i32) {
        self.as_global_scope().clear_timeout_or_interval(handle);
    }

    // https://html.spec.whatwg.org/multipage/#dom-windowtimers-setinterval
    fn SetInterval(
        &self,
        _cx: JSContext,
        callback: StringOrFunction,
        timeout: i32,
        args: Vec<HandleValue>,
    ) -> i32 {
        let callback = match callback {
            StringOrFunction::String(i) => TimerCallback::StringTimerCallback(i),
            StringOrFunction::Function(i) => TimerCallback::FunctionTimerCallback(i),
        };
        self.as_global_scope().set_timeout_or_interval(
            callback,
            args,
            Duration::from_millis(timeout.max(0) as u64),
            IsInterval::Interval,
        )
    }

    // https://html.spec.whatwg.org/multipage/#dom-windowtimers-clearinterval
    fn ClearInterval(&self, handle: i32) {
        self.ClearTimeout(handle);
    }
}

/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use base::id::PipelineId;
use constellation_traits::ScriptToConstellationChan;
use crossbeam_channel::Sender;
use devtools_traits::ScriptToDevtoolsControlMsg;
use dom_struct::dom_struct;
use embedder_traits::resources::{self, Resource};
use ipc_channel::ipc::IpcSender;
use js::jsval::UndefinedValue;
use js::rust::Runtime;
use js::rust::wrappers::JS_DefineDebuggerObject;
use net_traits::ResourceThreads;
use profile_traits::{mem, time};
use script_bindings::realms::InRealm;
use script_bindings::reflector::DomObject;
use servo_url::{ImmutableOrigin, MutableOrigin, ServoUrl};

use crate::dom::bindings::codegen::Bindings::DebuggerGlobalScopeBinding;
use crate::dom::bindings::error::report_pending_exception;
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::trace::CustomTraceable;
use crate::dom::bindings::utils::define_all_exposed_interfaces;
use crate::dom::event::EventStatus;
use crate::dom::globalscope::GlobalScope;
use crate::dom::types::{DebuggerEvent, Event};
#[cfg(feature = "testbinding")]
#[cfg(feature = "webgpu")]
use crate::dom::webgpu::identityhub::IdentityHub;
use crate::messaging::MainThreadScriptMsg;
use crate::realms::enter_realm;
use crate::script_module::ScriptFetchOptions;
use crate::script_runtime::{CanGc, JSContext};

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
    #[allow(unsafe_code, clippy::too_many_arguments)]
    pub(crate) fn new(
        runtime: &Runtime,
        script_chan: Sender<MainThreadScriptMsg>,
        devtools_chan: Option<IpcSender<ScriptToDevtoolsControlMsg>>,
        mem_profiler_chan: mem::ProfilerChan,
        time_profiler_chan: time::ProfilerChan,
        script_to_constellation_chan: ScriptToConstellationChan,
        resource_threads: ResourceThreads,
        #[cfg(feature = "webgpu")] gpu_id_hub: std::sync::Arc<IdentityHub>,
        can_gc: CanGc,
    ) -> DomRoot<Self> {
        let global = Box::new(Self {
            global_scope: GlobalScope::new_inherited(
                PipelineId::new(),
                devtools_chan,
                mem_profiler_chan,
                time_profiler_chan,
                script_to_constellation_chan,
                resource_threads,
                MutableOrigin::new(ImmutableOrigin::new_opaque()),
                ServoUrl::parse_with_base(None, "about:internal/debugger")
                    .expect("Guaranteed by argument"),
                None,
                Default::default(),
                #[cfg(feature = "webgpu")]
                gpu_id_hub,
                None,
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
        define_all_exposed_interfaces(global.upcast(), InRealm::entered(&realm), can_gc);
        assert!(unsafe {
            // Invariants: `cx` must be a non-null, valid JSContext pointer,
            // and `obj` must be a handle to a JS global object.
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

    fn evaluate_js(&self, script: &str, can_gc: CanGc) -> bool {
        rooted!(in (*Self::get_cx()) let mut rval = UndefinedValue());
        self.global_scope.evaluate_js_on_global_with_result(
            script,
            rval.handle_mut(),
            ScriptFetchOptions::default_classic_script(&self.global_scope),
            self.global_scope.api_base_url(),
            can_gc,
            None,
        )
    }

    pub(crate) fn execute(&self, can_gc: CanGc) {
        if !self.evaluate_js(&resources::read_string(Resource::DebuggerJS), can_gc) {
            let ar = enter_realm(self);
            report_pending_exception(Self::get_cx(), true, InRealm::Entered(&ar), can_gc);
        }
    }

    pub(crate) fn fire_add_debuggee(&self, can_gc: CanGc, global: &GlobalScope) {
        assert_eq!(
            DomRoot::upcast::<Event>(DebuggerEvent::new(self.upcast(), global, can_gc))
                .fire(self.upcast(), can_gc),
            EventStatus::NotCanceled,
            "Guaranteed by DebuggerEvent::new"
        );
    }
}

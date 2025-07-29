/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::sync::Arc;

use base::id::{Index, PipelineId, PipelineNamespaceId};
use constellation_traits::ScriptToConstellationChan;
use crossbeam_channel::Sender;
use devtools_traits::{ScriptToDevtoolsControlMsg, SourceInfo};
use dom_struct::dom_struct;
use embedder_traits::resources::{self, Resource};
use ipc_channel::ipc::IpcSender;
use js::gc::HandleValue;
use js::jsval::{UInt32Value, UndefinedValue};
use js::rust::Runtime;
use js::rust::wrappers::JS_DefineDebuggerObject;
use net_traits::ResourceThreads;
use profile_traits::{mem, time};
use script_bindings::codegen::GenericBindings::DebuggerGlobalScopeBinding::{
    DebuggerGlobalScopeMethods, NotifyNewSource,
};
use script_bindings::conversions::SafeToJSValConvertible;
use script_bindings::realms::InRealm;
use script_bindings::reflector::DomObject;
use script_bindings::utils::set_dictionary_property;
use servo_url::{ImmutableOrigin, MutableOrigin, ServoUrl};

use crate::dom::bindings::codegen::Bindings::DebuggerGlobalScopeBinding;
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::trace::CustomTraceable;
use crate::dom::bindings::utils::define_all_exposed_interfaces;
use crate::dom::globalscope::GlobalScope;
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
    #[allow(unsafe_code)]
    pub(crate) fn new(
        runtime: &Runtime,
        script_chan: Sender<MainThreadScriptMsg>,
        devtools_chan: Option<IpcSender<ScriptToDevtoolsControlMsg>>,
        mem_profiler_chan: mem::ProfilerChan,
        time_profiler_chan: time::ProfilerChan,
        script_to_constellation_chan: ScriptToConstellationChan,
        resource_threads: ResourceThreads,
        #[cfg(feature = "webgpu")] gpu_id_hub: Arc<IdentityHub>,
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

    fn evaluate_js(&self, script: &str, can_gc: CanGc) -> bool {
        rooted!(in (*Self::get_cx()) let mut rval = UndefinedValue());
        self.global_scope.evaluate_js_on_global_with_result(
            script,
            rval.handle_mut(),
            ScriptFetchOptions::default_classic_script(&self.global_scope),
            self.global_scope.api_base_url(),
            can_gc,
        )
    }

    pub(crate) fn execute(&self, can_gc: CanGc) {
        if !self.evaluate_js(&resources::read_string(Resource::DebuggerJS), can_gc) {
            warn!("Failed to execute debugger request");
        }
    }

    #[allow(unsafe_code)]
    pub(crate) fn execute_new_global(
        &self,
        can_gc: CanGc,
        global: &GlobalScope,
        pipeline_id: PipelineId,
    ) {
        let cx = Self::get_cx();
        rooted!(in(*cx) let pipeline_namespace_id = UInt32Value(pipeline_id.namespace_id.0));
        rooted!(in(*cx) let pipeline_index = UInt32Value(pipeline_id.index.0.get()));
        rooted!(in(*cx) let mut debuggee = UndefinedValue());

        let _realm = enter_realm(self);
        // Convert the debuggee global’s reflector to a Value, wrapping it from its originating realm (debuggee realm)
        // into the active realm (debugger realm) so that it can be passed across compartments.
        global.reflector().safe_to_jsval(cx, debuggee.handle_mut());

        // TODO: what invariants do we need to uphold for the unsafe call?
        if let Err(()) = (|| -> Result<(), ()> {
            unsafe {
                set_dictionary_property(
                    *cx,
                    self.global_scope.reflector().get_jsobject(),
                    "pipelineNamespaceId",
                    pipeline_namespace_id.handle(),
                )?;
                set_dictionary_property(
                    *cx,
                    self.global_scope.reflector().get_jsobject(),
                    "pipelineIndex",
                    pipeline_index.handle(),
                )?;
                set_dictionary_property(
                    *cx,
                    self.global_scope.reflector().get_jsobject(),
                    "debuggee",
                    debuggee.handle(),
                )
            }
        })() {
            warn!("Failed to set properties");
            return;
        }
        self.execute(can_gc);
    }
}

impl DebuggerGlobalScopeMethods<crate::DomTypeHolder> for DebuggerGlobalScope {
    // check-tidy: no specs after this line
    fn NotifyNewSource(&self, args: &NotifyNewSource) {
        info!(
            "NotifyNewSource: ({},{}) {} {} {}",
            args.pipelineId.namespaceId,
            args.pipelineId.index,
            args.spidermonkeyId,
            args.url,
            args.text
        );
        if let Some(devtools_chan) = self.as_global_scope().devtools_chan() {
            let pipeline_id = PipelineId {
                namespace_id: PipelineNamespaceId(args.pipelineId.namespaceId),
                index: Index::new(args.pipelineId.index)
                    .expect("`pipelineId.index` must not be zero"),
            };
            let source_info = SourceInfo {
                url: ServoUrl::parse(args.url.str()).expect("Failed to parse url"),
                external: true,  // TODO
                worker_id: None, // TODO
                content: Some(args.text.to_string()),
                content_type: None, // TODO
                spidermonkey_id: args.spidermonkeyId,
            };
            devtools_chan
                .send(ScriptToDevtoolsControlMsg::CreateSourceActor(
                    pipeline_id,
                    source_info,
                ))
                .expect("Failed to send to devtools server");
        }
    }
}

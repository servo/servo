/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use base::id::{Index, PipelineId, PipelineNamespaceId};
use constellation_traits::ScriptToConstellationChan;
use devtools_traits::{ScriptToDevtoolsControlMsg, SourceInfo, WorkerId};
use dom_struct::dom_struct;
use embedder_traits::JavaScriptEvaluationError;
use embedder_traits::resources::{self, Resource};
use ipc_channel::ipc::IpcSender;
use js::jsval::UndefinedValue;
use js::rust::Runtime;
use js::rust::wrappers::JS_DefineDebuggerObject;
use net_traits::ResourceThreads;
use profile_traits::{mem, time};
use script_bindings::codegen::GenericBindings::DebuggerGlobalScopeBinding::{
    DebuggerGlobalScopeMethods, NotifyNewSource,
};
use script_bindings::realms::InRealm;
use script_bindings::reflector::DomObject;
use servo_url::{ImmutableOrigin, MutableOrigin, ServoUrl};

use crate::dom::bindings::codegen::Bindings::DebuggerGlobalScopeBinding;
use crate::dom::bindings::error::report_pending_exception;
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::utils::define_all_exposed_interfaces;
use crate::dom::globalscope::GlobalScope;
use crate::dom::types::{DebuggerAddDebuggeeEvent, Event};
#[cfg(feature = "testbinding")]
#[cfg(feature = "webgpu")]
use crate::dom::webgpu::identityhub::IdentityHub;
use crate::realms::enter_realm;
use crate::script_module::ScriptFetchOptions;
use crate::script_runtime::{CanGc, IntroductionType, JSContext};

#[dom_struct]
/// Global scope for interacting with the devtools Debugger API.
///
/// <https://firefox-source-docs.mozilla.org/js/Debugger/>
pub(crate) struct DebuggerGlobalScope {
    global_scope: GlobalScope,
}

impl DebuggerGlobalScope {
    /// Create a new heap-allocated `DebuggerGlobalScope`.
    ///
    /// `debugger_pipeline_id` is the pipeline id to use when creating the debugger’s [`GlobalScope`]:
    /// - in normal script threads, it should be set to `PipelineId::new()`, because those threads can generate
    ///   pipeline ids, and they may contain debuggees from more than one pipeline
    /// - in web worker threads, it should be set to the pipeline id of the page that created the thread, because
    ///   those threads can’t generate pipeline ids, and they only contain one debuggee from one pipeline
    #[allow(unsafe_code, clippy::too_many_arguments)]
    pub(crate) fn new(
        runtime: &Runtime,
        debugger_pipeline_id: PipelineId,
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
                debugger_pipeline_id,
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

    pub(crate) fn as_global_scope(&self) -> &GlobalScope {
        self.upcast::<GlobalScope>()
    }

    fn evaluate_js(&self, script: &str, can_gc: CanGc) -> Result<(), JavaScriptEvaluationError> {
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
        if self
            .evaluate_js(&resources::read_string(Resource::DebuggerJS), can_gc)
            .is_err()
        {
            let ar = enter_realm(self);
            report_pending_exception(Self::get_cx(), true, InRealm::Entered(&ar), can_gc);
        }
    }

    pub(crate) fn fire_add_debuggee(
        &self,
        can_gc: CanGc,
        debuggee_global: &GlobalScope,
        debuggee_pipeline_id: PipelineId,
        debuggee_worker_id: Option<WorkerId>,
    ) {
        let debuggee_pipeline_id =
            crate::dom::pipelineid::PipelineId::new(self.upcast(), debuggee_pipeline_id, can_gc);
        let event = DomRoot::upcast::<Event>(DebuggerAddDebuggeeEvent::new(
            self.upcast(),
            debuggee_global,
            &debuggee_pipeline_id,
            debuggee_worker_id.map(|id| id.to_string().into()),
            can_gc,
        ));
        assert!(
            DomRoot::upcast::<Event>(event).fire(self.upcast(), can_gc),
            "Guaranteed by DebuggerAddDebuggeeEvent::new"
        );
    }
}

impl DebuggerGlobalScopeMethods<crate::DomTypeHolder> for DebuggerGlobalScope {
    // check-tidy: no specs after this line
    fn NotifyNewSource(&self, args: &NotifyNewSource) {
        let Some(devtools_chan) = self.as_global_scope().devtools_chan() else {
            return;
        };
        let pipeline_id = PipelineId {
            namespace_id: PipelineNamespaceId(args.pipelineId.namespaceId),
            index: Index::new(args.pipelineId.index).expect("`pipelineId.index` must not be zero"),
        };

        if let Some(introduction_type) = args.introductionType.as_ref() {
            // Check the `introductionType` and `url`, decide whether or not to create a source actor, and if so,
            // tell the devtools server to create a source actor. Based on the Firefox impl in:
            // - getDebuggerSourceURL() <https://searchfox.org/mozilla-central/rev/85667ab51e4b2a3352f7077a9ee43513049ed2d6/devtools/server/actors/utils/source-url.js#7-42>
            // - getSourceURL() <https://searchfox.org/mozilla-central/rev/85667ab51e4b2a3352f7077a9ee43513049ed2d6/devtools/server/actors/source.js#67-109>
            // - resolveSourceURL() <https://searchfox.org/mozilla-central/rev/85667ab51e4b2a3352f7077a9ee43513049ed2d6/devtools/server/actors/source.js#48-66>
            // - SourceActor#_isInlineSource <https://searchfox.org/mozilla-central/rev/85667ab51e4b2a3352f7077a9ee43513049ed2d6/devtools/server/actors/source.js#130-143>
            // - SourceActor#url <https://searchfox.org/mozilla-central/rev/85667ab51e4b2a3352f7077a9ee43513049ed2d6/devtools/server/actors/source.js#157-162>

            // Firefox impl: getDebuggerSourceURL(), getSourceURL()
            // TODO: handle `about:srcdoc` case (see Firefox getDebuggerSourceURL())
            // TODO: remove trailing details that may have been appended by SpiderMonkey
            // (currently impossible to do robustly due to <https://bugzilla.mozilla.org/show_bug.cgi?id=1982001>)
            let url_original = args.url.str();
            // FIXME: use page/worker url as base here
            let url_original = ServoUrl::parse(url_original).ok();

            // If the source has a `urlOverride` (aka `displayURL` aka `//# sourceURL`), it should be a valid url,
            // possibly relative to the page/worker url, and we should treat the source as coming from that url for
            // devtools purposes, including the file tree in the Sources tab.
            // Firefox impl: getSourceURL()
            let url_override = args
                .urlOverride
                .as_ref()
                .map(|url| url.str())
                // FIXME: use page/worker url as base here, not `url_original`
                .and_then(|url| ServoUrl::parse_with_base(url_original.as_ref(), url).ok());

            // If the `introductionType` is “eval or eval-like”, the `url` won’t be meaningful, so ignore these
            // sources unless we have a `urlOverride` (aka `displayURL` aka `//# sourceURL`).
            // Firefox impl: getDebuggerSourceURL(), getSourceURL()
            if [
                IntroductionType::INJECTED_SCRIPT_STR,
                IntroductionType::EVAL_STR,
                IntroductionType::DEBUGGER_EVAL_STR,
                IntroductionType::FUNCTION_STR,
                IntroductionType::JAVASCRIPT_URL_STR,
                IntroductionType::EVENT_HANDLER_STR,
                IntroductionType::DOM_TIMER_STR,
            ]
            .contains(&introduction_type.str()) &&
                url_override.is_none()
            {
                debug!(
                    "Not creating debuggee: `introductionType` is `{introduction_type}` but no valid url"
                );
                return;
            }

            // Sources with an `introductionType` of `inlineScript` are generally inline, meaning their contents
            // are a substring of the page markup (hence not known to SpiderMonkey, requiring plumbing in Servo).
            // But sources with a `urlOverride` are not inline, since they get their own place in the Sources tree.
            // nor are sources created for `<iframe srcdoc>`, since they are not necessarily a substring of the
            // page markup as originally sent by the server.
            // Firefox impl: SourceActor#_isInlineSource
            // TODO: handle `about:srcdoc` case (see Firefox SourceActor#_isInlineSource)
            let inline = introduction_type.str() == "inlineScript" && url_override.is_none();
            let Some(url) = url_override.or(url_original) else {
                debug!("Not creating debuggee: no valid url");
                return;
            };

            let worker_id = args.workerId.as_ref().map(|id| id.parse().unwrap());

            let source_info = SourceInfo {
                url,
                introduction_type: introduction_type.str().to_owned(),
                inline,
                worker_id,
                content: (!inline).then(|| args.text.to_string()),
                content_type: None, // TODO
                spidermonkey_id: args.spidermonkeyId,
            };
            if let Err(error) = devtools_chan.send(ScriptToDevtoolsControlMsg::CreateSourceActor(
                pipeline_id,
                source_info,
            )) {
                warn!("Failed to send to devtools server: {error:?}");
            }
        } else {
            debug!("Not creating debuggee for script with no `introductionType`");
        }
    }
}

/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::borrow::Cow;
use std::cell::RefCell;

use base::generic_channel::{GenericCallback, GenericSender, channel};
use base::id::{Index, PipelineId, PipelineNamespaceId};
use constellation_traits::ScriptToConstellationChan;
use devtools_traits::{
    DevtoolScriptControlMsg, EvaluateJSReply, ScriptToDevtoolsControlMsg, SourceInfo, WorkerId,
};
use dom_struct::dom_struct;
use embedder_traits::resources::{self, Resource};
use embedder_traits::{JavaScriptEvaluationError, ScriptToEmbedderChan};
use js::context::JSContext;
use js::jsval::UndefinedValue;
use js::rust::wrappers2::JS_DefineDebuggerObject;
use net_traits::ResourceThreads;
use profile_traits::{mem, time};
use script_bindings::codegen::GenericBindings::DebuggerEvalEventBinding::EvalResultValue;
use script_bindings::codegen::GenericBindings::DebuggerGetPossibleBreakpointsEventBinding::RecommendedBreakpointLocation;
use script_bindings::codegen::GenericBindings::DebuggerGlobalScopeBinding::{
    DebuggerGlobalScopeMethods, NotifyNewSource, PipelineIdInit,
};
use script_bindings::realms::InRealm;
use script_bindings::reflector::DomObject;
use script_bindings::str::DOMString;
use servo_url::{ImmutableOrigin, MutableOrigin, ServoUrl};
use storage_traits::StorageThreads;

use crate::dom::bindings::codegen::Bindings::DebuggerGlobalScopeBinding;
use crate::dom::bindings::codegen::Bindings::DebuggerInterruptEventBinding::{
    FrameInfo, PauseReason,
};
use crate::dom::bindings::error::report_pending_exception;
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::utils::define_all_exposed_interfaces;
use crate::dom::debuggerclearbreakpointevent::DebuggerClearBreakpointEvent;
use crate::dom::debuggerinterruptevent::DebuggerInterruptEvent;
use crate::dom::debuggersetbreakpointevent::DebuggerSetBreakpointEvent;
use crate::dom::globalscope::GlobalScope;
use crate::dom::types::{
    DebuggerAddDebuggeeEvent, DebuggerEvalEvent, DebuggerGetPossibleBreakpointsEvent, Event,
};
#[cfg(feature = "webgpu")]
use crate::dom::webgpu::identityhub::IdentityHub;
use crate::realms::{enter_auto_realm, enter_realm};
use crate::script_runtime::{CanGc, IntroductionType};
use crate::script_thread::with_script_thread;

#[dom_struct]
/// Global scope for interacting with the devtools Debugger API.
///
/// <https://firefox-source-docs.mozilla.org/js/Debugger/>
pub(crate) struct DebuggerGlobalScope {
    global_scope: GlobalScope,
    #[no_trace]
    devtools_to_script_sender: GenericSender<DevtoolScriptControlMsg>,
    #[no_trace]
    get_possible_breakpoints_result_sender:
        RefCell<Option<GenericSender<Vec<devtools_traits::RecommendedBreakpointLocation>>>>,
    #[no_trace]
    eval_result_sender: RefCell<Option<GenericSender<EvaluateJSReply>>>,
}

impl DebuggerGlobalScope {
    /// Create a new heap-allocated `DebuggerGlobalScope`.
    ///
    /// `debugger_pipeline_id` is the pipeline id to use when creating the debugger’s [`GlobalScope`]:
    /// - in normal script threads, it should be set to `PipelineId::new()`, because those threads can generate
    ///   pipeline ids, and they may contain debuggees from more than one pipeline
    /// - in web worker threads, it should be set to the pipeline id of the page that created the thread, because
    ///   those threads can’t generate pipeline ids, and they only contain one debuggee from one pipeline
    #[expect(unsafe_code, clippy::too_many_arguments)]
    pub(crate) fn new(
        debugger_pipeline_id: PipelineId,
        script_to_devtools_sender: Option<GenericCallback<ScriptToDevtoolsControlMsg>>,
        devtools_to_script_sender: GenericSender<DevtoolScriptControlMsg>,
        mem_profiler_chan: mem::ProfilerChan,
        time_profiler_chan: time::ProfilerChan,
        script_to_constellation_chan: ScriptToConstellationChan,
        script_to_embedder_chan: ScriptToEmbedderChan,
        resource_threads: ResourceThreads,
        storage_threads: StorageThreads,
        #[cfg(feature = "webgpu")] gpu_id_hub: std::sync::Arc<IdentityHub>,
        cx: &mut JSContext,
    ) -> DomRoot<Self> {
        let global = Box::new(Self {
            global_scope: GlobalScope::new_inherited(
                debugger_pipeline_id,
                script_to_devtools_sender,
                mem_profiler_chan,
                time_profiler_chan,
                script_to_constellation_chan,
                script_to_embedder_chan,
                resource_threads,
                storage_threads,
                MutableOrigin::new(ImmutableOrigin::new_opaque()),
                ServoUrl::parse_with_base(None, "about:internal/debugger")
                    .expect("Guaranteed by argument"),
                None,
                #[cfg(feature = "webgpu")]
                gpu_id_hub,
                None,
                false,
                None, // font_context
            ),
            devtools_to_script_sender,
            get_possible_breakpoints_result_sender: RefCell::new(None),
            eval_result_sender: RefCell::new(None),
        });
        let global = DebuggerGlobalScopeBinding::Wrap::<crate::DomTypeHolder>(cx, global);

        let mut realm = enter_auto_realm(cx, &*global);
        let mut realm = realm.current_realm();
        define_all_exposed_interfaces(&mut realm, global.upcast());
        assert!(unsafe {
            // Invariants: `obj` must be a handle to a JS global object.
            JS_DefineDebuggerObject(&mut realm, global.global_scope.reflector().get_jsobject())
        });

        global
    }

    pub(crate) fn as_global_scope(&self) -> &GlobalScope {
        self.upcast::<GlobalScope>()
    }

    fn evaluate_js(
        &self,
        script: Cow<'_, str>,
        cx: &mut JSContext,
    ) -> Result<(), JavaScriptEvaluationError> {
        rooted!(&in(cx) let mut rval = UndefinedValue());
        self.global_scope.evaluate_js_on_global(
            script,
            "",
            None,
            rval.handle_mut(),
            CanGc::from_cx(cx),
        )
    }

    pub(crate) fn execute(&self, cx: &mut JSContext) {
        if self
            .evaluate_js(resources::read_string(Resource::DebuggerJS).into(), cx)
            .is_err()
        {
            let mut realm = enter_auto_realm(cx, self);
            let mut realm = realm.current_realm();
            let in_realm_proof = (&mut realm).into();
            let in_realm = InRealm::Already(&in_realm_proof);

            let cx = &mut realm;
            report_pending_exception(cx.into(), true, in_realm, CanGc::from_cx(cx));
        }
    }

    pub(crate) fn fire_add_debuggee(
        &self,
        can_gc: CanGc,
        debuggee_global: &GlobalScope,
        debuggee_pipeline_id: PipelineId,
        debuggee_worker_id: Option<WorkerId>,
    ) {
        let _realm = enter_realm(self);
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
            event.fire(self.upcast(), can_gc),
            "Guaranteed by DebuggerAddDebuggeeEvent::new"
        );
    }

    pub(crate) fn fire_eval(
        &self,
        can_gc: CanGc,
        code: DOMString,
        debuggee_pipeline_id: PipelineId,
        debuggee_worker_id: Option<WorkerId>,
        result_sender: GenericSender<EvaluateJSReply>,
    ) {
        assert!(
            self.eval_result_sender
                .replace(Some(result_sender))
                .is_none()
        );
        let _realm = enter_realm(self);
        let debuggee_pipeline_id =
            crate::dom::pipelineid::PipelineId::new(self.upcast(), debuggee_pipeline_id, can_gc);
        let event = DomRoot::upcast::<Event>(DebuggerEvalEvent::new(
            self.upcast(),
            code,
            &debuggee_pipeline_id,
            debuggee_worker_id.map(|id| id.to_string().into()),
            can_gc,
        ));
        assert!(
            event.fire(self.upcast(), can_gc),
            "Guaranteed by DebuggerEvalEvent::new"
        );
    }

    pub(crate) fn fire_get_possible_breakpoints(
        &self,
        can_gc: CanGc,
        spidermonkey_id: u32,
        result_sender: GenericSender<Vec<devtools_traits::RecommendedBreakpointLocation>>,
    ) {
        assert!(
            self.get_possible_breakpoints_result_sender
                .replace(Some(result_sender))
                .is_none()
        );
        let _realm = enter_realm(self);
        let event = DomRoot::upcast::<Event>(DebuggerGetPossibleBreakpointsEvent::new(
            self.upcast(),
            spidermonkey_id,
            can_gc,
        ));
        assert!(
            event.fire(self.upcast(), can_gc),
            "Guaranteed by DebuggerGetPossibleBreakpointsEvent::new"
        );
    }

    pub(crate) fn fire_set_breakpoint(
        &self,
        can_gc: CanGc,
        spidermonkey_id: u32,
        script_id: u32,
        offset: u32,
    ) {
        let event = DomRoot::upcast::<Event>(DebuggerSetBreakpointEvent::new(
            self.upcast(),
            spidermonkey_id,
            script_id,
            offset,
            can_gc,
        ));
        assert!(
            event.fire(self.upcast(), can_gc),
            "Guaranteed by DebuggerSetBreakpointEvent::new"
        );
    }

    pub(crate) fn fire_interrupt(&self, can_gc: CanGc) {
        let event = DomRoot::upcast::<Event>(DebuggerInterruptEvent::new(self.upcast(), can_gc));
        assert!(
            event.fire(self.upcast(), can_gc),
            "Guaranteed by DebuggerInterruptEvent::new"
        );
    }

    pub(crate) fn fire_clear_breakpoint(
        &self,
        can_gc: CanGc,
        spidermonkey_id: u32,
        script_id: u32,
        offset: u32,
    ) {
        let event = DomRoot::upcast::<Event>(DebuggerClearBreakpointEvent::new(
            self.upcast(),
            spidermonkey_id,
            script_id,
            offset,
            can_gc,
        ));
        assert!(
            event.fire(self.upcast(), can_gc),
            "Guaranteed by DebuggerClearBreakpointEvent::new"
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
            let url_original = ServoUrl::parse(&url_original).ok();

            // If the source has a `urlOverride` (aka `displayURL` aka `//# sourceURL`), it should be a valid url,
            // possibly relative to the page/worker url, and we should treat the source as coming from that url for
            // devtools purposes, including the file tree in the Sources tab.
            // Firefox impl: getSourceURL()
            let url_override = args
                .urlOverride
                .as_ref()
                .map(|url| url.str())
                // FIXME: use page/worker url as base here, not `url_original`
                .and_then(|url| ServoUrl::parse_with_base(url_original.as_ref(), &url).ok());

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
            .contains(&&*introduction_type.str()) &&
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
                self.devtools_to_script_sender.clone(),
                pipeline_id,
                source_info,
            )) {
                warn!("Failed to send to devtools server: {error:?}");
            }
        } else {
            debug!("Not creating debuggee for script with no `introductionType`");
        }
    }

    fn GetPossibleBreakpointsResult(
        &self,
        event: &DebuggerGetPossibleBreakpointsEvent,
        result: Vec<RecommendedBreakpointLocation>,
    ) {
        info!("GetPossibleBreakpointsResult: {event:?} {result:?}");
        let sender = self
            .get_possible_breakpoints_result_sender
            .take()
            .expect("Guaranteed by Self::fire_get_possible_breakpoints()");
        let _ = sender.send(
            result
                .into_iter()
                .map(|entry| devtools_traits::RecommendedBreakpointLocation {
                    script_id: entry.scriptId,
                    offset: entry.offset,
                    line_number: entry.lineNumber,
                    column_number: entry.columnNumber,
                    is_step_start: entry.isStepStart,
                })
                .collect(),
        );
    }

    /// Handle the result from debugger.js executeInGlobal() call.
    ///
    /// The result contains completion value information from the SpiderMonkey Debugger API:
    /// <https://firefox-source-docs.mozilla.org/js/Debugger/Conventions.html#completion-values>
    fn EvalResult(&self, _event: &DebuggerEvalEvent, result: &EvalResultValue) {
        let sender = self
            .eval_result_sender
            .take()
            .expect("Guaranteed by Self::fire_eval()");

        let reply = if result.completionType.str() == "terminated" {
            EvaluateJSReply::VoidValue
        } else {
            match &*result.valueType.str() {
                "undefined" => EvaluateJSReply::VoidValue,
                "null" => EvaluateJSReply::NullValue,
                "boolean" => {
                    EvaluateJSReply::BooleanValue(result.booleanValue.flatten().unwrap_or(false))
                },
                "number" => {
                    let num = result.numberValue.flatten().map(|f| *f).unwrap_or(0.0);
                    EvaluateJSReply::NumberValue(num)
                },
                "string" => EvaluateJSReply::StringValue(
                    result
                        .stringValue
                        .as_ref()
                        .and_then(|opt| opt.as_ref())
                        .map(|s| s.to_string())
                        .unwrap_or_default(),
                ),
                "object" => {
                    let class = result
                        .objectClass
                        .as_ref()
                        .and_then(|opt| opt.as_ref())
                        .map(|s| s.to_string())
                        .unwrap_or_else(|| "Object".to_string());
                    EvaluateJSReply::ActorValue {
                        class,
                        uuid: uuid::Uuid::new_v4().to_string(),
                    }
                },
                _ => unreachable!(),
            }
        };

        let _ = sender.send(reply);
    }

    fn PauseAndRespond(
        &self,
        pipeline_id: &PipelineIdInit,
        frame_actor_id: DOMString,
        pause_reason: &PauseReason,
    ) {
        let pipeline_id = PipelineId {
            namespace_id: PipelineNamespaceId(pipeline_id.namespaceId),
            index: Index::new(pipeline_id.index).expect("`pipelineId.index` must not be zero"),
        };

        let pause_reason = devtools_traits::PauseReason {
            type_: pause_reason.type_.clone().into(),
            on_next: pause_reason.onNext,
        };

        if let Some(chan) = self.upcast::<GlobalScope>().devtools_chan() {
            let msg = ScriptToDevtoolsControlMsg::DebuggerPause(
                pipeline_id,
                frame_actor_id.into(),
                pause_reason,
            );
            let _ = chan.send(msg);
        }

        with_script_thread(|script_thread| {
            script_thread.enter_debugger_pause_loop();
        });
    }

    fn RegisterFrameActor(
        &self,
        pipeline_id: &PipelineIdInit,
        result: &FrameInfo,
    ) -> Option<DOMString> {
        let pipeline_id = PipelineId {
            namespace_id: PipelineNamespaceId(pipeline_id.namespaceId),
            index: Index::new(pipeline_id.index).expect("`pipelineId.index` must not be zero"),
        };

        let chan = self.upcast::<GlobalScope>().devtools_chan()?;
        let (tx, rx) = channel::<String>().unwrap();

        let frame = devtools_traits::FrameInfo {
            column: result.column,
            display_name: result.displayName.clone().into(),
            line: result.line,
            on_stack: result.onStack,
            oldest: result.oldest,
            terminated: result.terminated,
            type_: result.type_.clone().into(),
            url: result.url.clone().into(),
        };
        let msg = ScriptToDevtoolsControlMsg::CreateFrameActor(tx, pipeline_id, frame);
        let _ = chan.send(msg);

        rx.recv().ok().map(DOMString::from)
    }
}

/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::fmt::Debug;

use devtools_traits::GetEnvironmentRequest;
use dom_struct::dom_struct;
use js::context::JSContext;
use script_bindings::inheritance::Castable;
use script_bindings::reflector::reflect_dom_object_with_cx;
use script_bindings::str::DOMString;

use crate::dom::GlobalScope;
use crate::dom::bindings::codegen::Bindings::DebuggerGetEnvironmentEventBinding::DebuggerGetEnvironmentEventMethods;
use crate::dom::bindings::codegen::Bindings::EventBinding::Event_Binding::EventMethods;
use crate::dom::bindings::root::DomRoot;
use crate::dom::event::Event;
use crate::dom::pipelineid::PipelineId;
use crate::dom::types::DebuggerGlobalScope;

#[dom_struct]
/// Event for Rust → JS calls in [`crate::dom::debugger::DebuggerGlobalScope`].
pub(crate) struct DebuggerGetEnvironmentEvent {
    event: Event,
    frame_actor_id: Option<DOMString>,
    pipeline_id: Option<DomRoot<PipelineId>>,
}

impl DebuggerGetEnvironmentEvent {
    pub(crate) fn new(
        cx: &mut JSContext,
        debugger_global_scope: &DebuggerGlobalScope,
        request: GetEnvironmentRequest,
    ) -> DomRoot<Self> {
        let result = Box::new(match request {
            GetEnvironmentRequest::Frame(frame_actor_id) => Self {
                event: Event::new_inherited(),
                frame_actor_id: Some(frame_actor_id.into()),
                pipeline_id: None,
            },
            GetEnvironmentRequest::Global(debuggee_pipeline_id) => {
                let debuggee_pipeline_id = crate::dom::pipelineid::PipelineId::new(
                    cx,
                    debugger_global_scope.upcast(),
                    debuggee_pipeline_id,
                );
                Self {
                    event: Event::new_inherited(),
                    frame_actor_id: None,
                    pipeline_id: Some(debuggee_pipeline_id),
                }
            },
        });
        let result =
            reflect_dom_object_with_cx(result, debugger_global_scope.upcast::<GlobalScope>(), cx);
        result
            .event
            .init_event("getEnvironment".into(), false, false);

        result
    }
}

impl DebuggerGetEnvironmentEventMethods<crate::DomTypeHolder> for DebuggerGetEnvironmentEvent {
    // check-tidy: no specs after this line
    fn GetFrameActorId(&self) -> Option<DOMString> {
        self.frame_actor_id.clone()
    }

    fn GetPipelineId(
        &self,
    ) -> Option<DomRoot<<crate::DomTypeHolder as script_bindings::DomTypes>::PipelineId>> {
        self.pipeline_id.clone()
    }

    fn IsTrusted(&self) -> bool {
        self.event.IsTrusted()
    }
}

impl Debug for DebuggerGetEnvironmentEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DebuggerGetEnvironmentEvent").finish()
    }
}

/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use script_bindings::str::DOMString;

use crate::dom::bindings::codegen::Bindings::DebuggerEvalEventBinding::DebuggerEvalEventMethods;
use crate::dom::bindings::codegen::Bindings::EventBinding::Event_Binding::EventMethods;
use crate::dom::bindings::reflector::reflect_dom_object;
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::event::Event;
use crate::dom::types::{GlobalScope, PipelineId};
use crate::script_runtime::CanGc;

#[dom_struct]
/// Event for Rust → JS calls in [`crate::dom::DebuggerGlobalScope`].
pub(crate) struct DebuggerEvalEvent {
    event: Event,
    code: DOMString,
    pipeline_id: Dom<PipelineId>,
    worker_id: Option<DOMString>,
}

impl DebuggerEvalEvent {
    pub(crate) fn new(
        debugger_global: &GlobalScope,
        code: DOMString,
        pipeline_id: &PipelineId,
        worker_id: Option<DOMString>,
        can_gc: CanGc,
    ) -> DomRoot<Self> {
        let result = Box::new(Self {
            event: Event::new_inherited(),
            code,
            pipeline_id: Dom::from_ref(pipeline_id),
            worker_id,
        });
        let result = reflect_dom_object(result, debugger_global, can_gc);
        result.event.init_event("eval".into(), false, false);

        result
    }
}

impl DebuggerEvalEventMethods<crate::DomTypeHolder> for DebuggerEvalEvent {
    // check-tidy: no specs after this line
    fn Code(&self) -> DOMString {
        self.code.clone()
    }

    fn PipelineId(
        &self,
    ) -> DomRoot<<crate::DomTypeHolder as script_bindings::DomTypes>::PipelineId> {
        DomRoot::from_ref(&self.pipeline_id)
    }

    fn GetWorkerId(&self) -> Option<DOMString> {
        self.worker_id.clone()
    }

    fn IsTrusted(&self) -> bool {
        self.event.IsTrusted()
    }
}

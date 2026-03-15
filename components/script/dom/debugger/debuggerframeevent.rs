/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::fmt::Debug;

use dom_struct::dom_struct;

use crate::dom::bindings::codegen::Bindings::DebuggerFrameEventBinding::DebuggerFrameEventMethods;
use crate::dom::bindings::codegen::Bindings::EventBinding::Event_Binding::EventMethods;
use crate::dom::bindings::reflector::reflect_dom_object;
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::event::Event;
use crate::dom::types::{GlobalScope, PipelineId};
use crate::script_runtime::CanGc;

#[dom_struct]
/// Event for Rust → JS calls in [`crate::dom::debugger::DebuggerGlobalScope`].
pub(crate) struct DebuggerFrameEvent {
    event: Event,
    pipeline_id: Dom<PipelineId>,
    start: u32,
    count: u32,
}

impl DebuggerFrameEvent {
    pub(crate) fn new(
        debugger_global: &GlobalScope,
        pipeline_id: &PipelineId,
        start: u32,
        count: u32,
        can_gc: CanGc,
    ) -> DomRoot<Self> {
        let result = Box::new(Self {
            event: Event::new_inherited(),
            pipeline_id: Dom::from_ref(pipeline_id),
            start,
            count,
        });
        let result = reflect_dom_object(result, debugger_global, can_gc);
        result.event.init_event("frames".into(), false, false);

        result
    }
}

impl DebuggerFrameEventMethods<crate::DomTypeHolder> for DebuggerFrameEvent {
    // check-tidy: no specs after this line
    fn PipelineId(
        &self,
    ) -> DomRoot<<crate::DomTypeHolder as script_bindings::DomTypes>::PipelineId> {
        DomRoot::from_ref(&self.pipeline_id)
    }

    fn Start(&self) -> u32 {
        self.start
    }

    fn Count(&self) -> u32 {
        self.count
    }

    fn IsTrusted(&self) -> bool {
        self.event.IsTrusted()
    }
}

impl Debug for DebuggerFrameEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DebuggerFrameEvent").finish()
    }
}

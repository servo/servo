/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::fmt::Debug;

use dom_struct::dom_struct;
use js::context::JSContext;
use script_bindings::reflector::reflect_dom_object_with_cx;
use script_bindings::str::DOMString;

use crate::dom::bindings::codegen::Bindings::DebuggerResumeEventBinding::DebuggerResumeEventMethods;
use crate::dom::bindings::codegen::Bindings::EventBinding::Event_Binding::EventMethods;
use crate::dom::bindings::root::DomRoot;
use crate::dom::event::Event;
use crate::dom::types::GlobalScope;

#[dom_struct]
/// Event for Rust → JS calls in [`crate::dom::debugger::DebuggerGlobalScope`].
pub(crate) struct DebuggerResumeEvent {
    event: Event,
    resume_limit_type: Option<DOMString>,
    frame_actor_id: Option<DOMString>,
}

impl DebuggerResumeEvent {
    pub(crate) fn new(
        cx: &mut JSContext,
        debugger_global: &GlobalScope,
        resume_limit_type: Option<DOMString>,
        frame_actor_id: Option<DOMString>,
    ) -> DomRoot<Self> {
        let result = Box::new(Self {
            event: Event::new_inherited(),
            resume_limit_type,
            frame_actor_id,
        });
        let result = reflect_dom_object_with_cx(result, debugger_global, cx);
        result.event.init_event("resume".into(), false, false);

        result
    }
}

impl DebuggerResumeEventMethods<crate::DomTypeHolder> for DebuggerResumeEvent {
    // check-tidy: no specs after this line
    fn IsTrusted(&self) -> bool {
        self.event.IsTrusted()
    }

    fn GetResumeLimitType(&self) -> Option<DOMString> {
        self.resume_limit_type.clone()
    }

    fn GetFrameActorID(&self) -> Option<DOMString> {
        self.frame_actor_id.clone()
    }
}

impl Debug for DebuggerResumeEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DebuggerResumeEvent").finish()
    }
}

/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::fmt::Debug;

use dom_struct::dom_struct;

use crate::dom::bindings::codegen::Bindings::DebuggerGetPossibleBreakpointsEventBinding::DebuggerGetPossibleBreakpointsEventMethods;
use crate::dom::bindings::codegen::Bindings::EventBinding::Event_Binding::EventMethods;
use crate::dom::bindings::reflector::reflect_dom_object;
use crate::dom::bindings::root::DomRoot;
use crate::dom::event::Event;
use crate::dom::types::GlobalScope;
use crate::script_runtime::CanGc;

#[dom_struct]
/// Event for Rust → JS calls in [`crate::dom::DebuggerGlobalScope`].
pub(crate) struct DebuggerGetPossibleBreakpointsEvent {
    event: Event,
    spidermonkey_id: u32,
}

impl DebuggerGetPossibleBreakpointsEvent {
    pub(crate) fn new(
        debugger_global: &GlobalScope,
        spidermonkey_id: u32,
        can_gc: CanGc,
    ) -> DomRoot<Self> {
        let result = Box::new(Self {
            event: Event::new_inherited(),
            spidermonkey_id,
        });
        let result = reflect_dom_object(result, debugger_global, can_gc);
        result
            .event
            .init_event("getPossibleBreakpoints".into(), false, false);

        result
    }
}

impl DebuggerGetPossibleBreakpointsEventMethods<crate::DomTypeHolder>
    for DebuggerGetPossibleBreakpointsEvent
{
    // check-tidy: no specs after this line
    fn SpidermonkeyId(&self) -> u32 {
        self.spidermonkey_id
    }

    fn IsTrusted(&self) -> bool {
        self.event.IsTrusted()
    }
}

impl Debug for DebuggerGetPossibleBreakpointsEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DebuggerGetPossibleBreakpointsEvent")
            .field("spidermonkey_id", &self.spidermonkey_id)
            .finish()
    }
}

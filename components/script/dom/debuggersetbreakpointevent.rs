/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::fmt::Debug;

use dom_struct::dom_struct;

use crate::dom::bindings::codegen::Bindings::DebuggerSetBreakpointEventBinding::DebuggerSetBreakpointEventMethods;
use crate::dom::bindings::codegen::Bindings::EventBinding::Event_Binding::EventMethods;
use crate::dom::bindings::reflector::reflect_dom_object;
use crate::dom::bindings::root::DomRoot;
use crate::dom::event::Event;
use crate::dom::types::GlobalScope;
use crate::script_runtime::CanGc;

#[dom_struct]
/// Event for Rust → JS calls in [`crate::dom::DebuggerGlobalScope`].
pub(crate) struct DebuggerSetBreakpointEvent {
    event: Event,
    spidermonkey_id: u32,
    script_id: u32,
    offset: u32,
}

impl DebuggerSetBreakpointEvent {
    pub(crate) fn new(
        debugger_global: &GlobalScope,
        spidermonkey_id: u32,
        script_id: u32,
        offset: u32,
        can_gc: CanGc,
    ) -> DomRoot<Self> {
        let result = Box::new(Self {
            event: Event::new_inherited(),
            spidermonkey_id,
            script_id,
            offset,
        });
        let result = reflect_dom_object(result, debugger_global, can_gc);
        result
            .event
            .init_event("setBreakpoint".into(), false, false);

        result
    }
}

impl DebuggerSetBreakpointEventMethods<crate::DomTypeHolder> for DebuggerSetBreakpointEvent {
    // check-tidy: no specs after this line
    fn SpidermonkeyId(&self) -> u32 {
        self.spidermonkey_id
    }

    fn ScriptId(&self) -> u32 {
        self.script_id
    }

    fn Offset(&self) -> u32 {
        self.offset
    }

    fn IsTrusted(&self) -> bool {
        self.event.IsTrusted()
    }
}

impl Debug for DebuggerSetBreakpointEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DebuggerSetBreakpointEvent")
            .field("spidermonkey_id", &self.spidermonkey_id)
            .field("script_id", &self.script_id)
            .field("offset", &self.offset)
            .finish()
    }
}

/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use script_bindings::codegen::GenericBindings::DebuggerUnblackboxEventBinding::DebuggerUnblackboxEventMethods;

use crate::dom::bindings::codegen::Bindings::DebuggerGlobalScopeBinding::DebuggerSourceLocation;
use crate::dom::bindings::codegen::Bindings::EventBinding::Event_Binding::EventMethods;
use crate::dom::bindings::reflector::reflect_dom_object;
use crate::dom::bindings::root::DomRoot;
use crate::dom::event::Event;
use crate::dom::types::GlobalScope;
use crate::script_runtime::CanGc;

#[dom_struct]
/// Event for Rust → JS calls in [`crate::dom::debugger::DebuggerGlobalScope`].
pub(crate) struct DebuggerUnblackboxEvent {
    event: Event,
    spidermonkey_id: u32,
    covers_full_source: bool,

    // Only relevant if covers_full_source is false
    start_line: u32,
    start_column: u32,
    end_line: u32,
    end_column: u32,
}

impl DebuggerUnblackboxEvent {
    pub(crate) fn new(
        debugger_global: &GlobalScope,
        spidermonkey_id: u32,
        range: Option<((u32, u32), (u32, u32))>,
        can_gc: CanGc,
    ) -> DomRoot<Self> {
        let result = match range {
            Some(((start_line, start_column), (end_line, end_column))) => Self {
                event: Event::new_inherited(),
                spidermonkey_id,
                covers_full_source: false,
                start_line,
                start_column,
                end_line,
                end_column,
            },
            None => Self {
                event: Event::new_inherited(),
                spidermonkey_id,
                covers_full_source: true,
                start_line: 0,
                start_column: 0,
                end_line: 0,
                end_column: 0,
            },
        };

        let result = reflect_dom_object(Box::new(result), debugger_global, can_gc);
        result.event.init_event("blackbox".into(), false, false);

        result
    }
}

impl DebuggerUnblackboxEventMethods<crate::DomTypeHolder> for DebuggerUnblackboxEvent {
    // check-tidy: no specs after this line
    fn SpidermonkeyId(&self) -> u32 {
        self.spidermonkey_id
    }

    fn CoversFullSource(&self) -> bool {
        self.covers_full_source
    }

    fn Start(&self) -> DebuggerSourceLocation {
        DebuggerSourceLocation {
            line: self.start_line,
            column: self.start_column,
        }
    }

    fn End(&self) -> DebuggerSourceLocation {
        DebuggerSourceLocation {
            line: self.end_line,
            column: self.end_column,
        }
    }

    fn IsTrusted(&self) -> bool {
        self.event.IsTrusted()
    }
}

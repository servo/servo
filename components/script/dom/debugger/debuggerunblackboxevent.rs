/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use devtools_traits::BlackboxCoverage;
use devtools_traits::BlackboxCoverage::{Full, Partial};
use dom_struct::dom_struct;
use js::context::JSContext;
use script_bindings::codegen::GenericBindings::DebuggerUnblackboxEventBinding::DebuggerUnblackboxEventMethods;
use script_bindings::reflector::reflect_dom_object_with_cx;

use crate::dom::bindings::codegen::Bindings::DebuggerGlobalScopeBinding::DebuggerSourceLocation;
use crate::dom::bindings::codegen::Bindings::EventBinding::Event_Binding::EventMethods;
use crate::dom::bindings::root::DomRoot;
use crate::dom::event::Event;
use crate::dom::types::GlobalScope;

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
        cx: &mut JSContext,
        debugger_global: &GlobalScope,
        spidermonkey_id: u32,
        coverage: BlackboxCoverage,
    ) -> DomRoot<Self> {
        let result = Box::new(match coverage {
            Partial((start_line, start_column), (end_line, end_column)) => Self {
                event: Event::new_inherited(),
                spidermonkey_id,
                covers_full_source: false,
                start_line,
                start_column,
                end_line,
                end_column,
            },
            Full => Self {
                event: Event::new_inherited(),
                spidermonkey_id,
                covers_full_source: true,
                start_line: 0,
                start_column: 0,
                end_line: 0,
                end_column: 0,
            },
        });

        let result = reflect_dom_object_with_cx(result, debugger_global, cx);
        result.event.init_event("unblackbox".into(), false, false);

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

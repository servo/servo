/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::ops::Deref;

use dom_struct::dom_struct;
use metrics::ToMs;

use crate::dom::bindings::codegen::Bindings::VisibilityStateEntryBinding::VisibilityStateEntryMethods;
use crate::dom::bindings::num::Finite;
use crate::dom::bindings::reflector::reflect_dom_object;
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::DOMString;
use crate::dom::globalscope::GlobalScope;
use crate::dom::performanceentry::PerformanceEntry;

use super::bindings::codegen::Bindings::DocumentBinding::DocumentVisibilityState;
use super::bindings::codegen::Bindings::PerformanceEntryBinding::PerformanceEntryMethods;

#[dom_struct]
pub struct VisibilityStateEntry {
    entry: PerformanceEntry,
}

impl VisibilityStateEntry {
    fn new_inherited(state: DocumentVisibilityState, timestamp: u64) -> VisibilityStateEntry {
        let name = match state {
            DocumentVisibilityState::Visible => DOMString::from("visible"),
            DocumentVisibilityState::Hidden => DOMString::from("hidden"),
        };
        VisibilityStateEntry {
            entry: PerformanceEntry::new_inherited(
                name,
                DOMString::from("visibility-state"),
                timestamp.to_ms(),
                0.,
            ),
        }
    }

    #[allow(crown::unrooted_must_root)]
    pub fn new(
        global: &GlobalScope,
        state: DocumentVisibilityState,
        timestamp: u64,
    ) -> DomRoot<VisibilityStateEntry> {
        let entry = VisibilityStateEntry::new_inherited(state, timestamp);
        reflect_dom_object(Box::new(entry), global)
    }
}

impl VisibilityStateEntryMethods for VisibilityStateEntry {
    fn Name(&self) -> DOMString {
        self.entry.Name()
    }

    fn EntryType(&self) -> DOMString {
        self.entry.EntryType()
    }

    fn StartTime(&self) -> Finite<f64> {
        self.entry.StartTime()
    }

    fn Duration(&self) -> u32 {
        *self.entry.Duration().deref() as u32
    }
}

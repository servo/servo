/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::ops::Deref;

use dom_struct::dom_struct;

use crate::dom::bindings::codegen::Bindings::DocumentBinding::DocumentVisibilityState;
use crate::dom::bindings::codegen::Bindings::PerformanceEntryBinding::PerformanceEntryMethods;
use crate::dom::bindings::codegen::Bindings::VisibilityStateEntryBinding::VisibilityStateEntryMethods;
use crate::dom::bindings::num::Finite;
use crate::dom::bindings::reflector::reflect_dom_object;
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::DOMString;
use crate::dom::globalscope::GlobalScope;
use crate::dom::performanceentry::PerformanceEntry;

#[dom_struct]
pub struct VisibilityStateEntry {
    entry: PerformanceEntry,
}

impl VisibilityStateEntry {
    #[allow(crown::unrooted_must_root)]
    fn new_inherited(state: DocumentVisibilityState, timestamp: f64) -> VisibilityStateEntry {
        let name = match state {
            DocumentVisibilityState::Visible => DOMString::from("visible"),
            DocumentVisibilityState::Hidden => DOMString::from("hidden"),
        };
        VisibilityStateEntry {
            entry: PerformanceEntry::new_inherited(
                name,
                DOMString::from("visibility-state"),
                timestamp,
                0.,
            ),
        }
    }

    pub fn new(
        global: &GlobalScope,
        state: DocumentVisibilityState,
        timestamp: f64,
    ) -> DomRoot<VisibilityStateEntry> {
        reflect_dom_object(
            Box::new(VisibilityStateEntry::new_inherited(state, timestamp)),
            global,
        )
    }
}

impl VisibilityStateEntryMethods for VisibilityStateEntry {
    /// <https://html.spec.whatwg.org/multipage/#visibilitystateentry-name>
    fn Name(&self) -> DOMString {
        self.entry.Name()
    }

    /// <https://html.spec.whatwg.org/multipage/#visibilitystateentry-entrytype>
    fn EntryType(&self) -> DOMString {
        self.entry.EntryType()
    }

    /// <https://html.spec.whatwg.org/multipage/#visibilitystateentry-starttime>
    fn StartTime(&self) -> Finite<f64> {
        self.entry.StartTime()
    }

    /// <https://html.spec.whatwg.org/multipage/#visibilitystateentry-duration>
    fn Duration(&self) -> u32 {
        *self.entry.Duration().deref() as u32
    }
}

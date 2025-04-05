/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::ops::Deref;

use base::cross_process_instant::CrossProcessInstant;
use dom_struct::dom_struct;
use time::Duration;

use crate::dom::bindings::codegen::Bindings::DocumentBinding::DocumentVisibilityState;
use crate::dom::bindings::codegen::Bindings::PerformanceEntryBinding::PerformanceEntryMethods;
use crate::dom::bindings::codegen::Bindings::VisibilityStateEntryBinding::VisibilityStateEntryMethods;
use crate::dom::bindings::num::Finite;
use crate::dom::bindings::reflector::reflect_dom_object;
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::DOMString;
use crate::dom::globalscope::GlobalScope;
use crate::dom::performanceentry::PerformanceEntry;
use crate::script_runtime::CanGc;

#[dom_struct]
pub(crate) struct VisibilityStateEntry {
    entry: PerformanceEntry,
}

impl VisibilityStateEntry {
    #[cfg_attr(crown, allow(crown::unrooted_must_root))]
    fn new_inherited(
        state: DocumentVisibilityState,
        timestamp: CrossProcessInstant,
    ) -> VisibilityStateEntry {
        let name = match state {
            DocumentVisibilityState::Visible => DOMString::from("visible"),
            DocumentVisibilityState::Hidden => DOMString::from("hidden"),
        };
        VisibilityStateEntry {
            entry: PerformanceEntry::new_inherited(
                name,
                DOMString::from("visibility-state"),
                Some(timestamp),
                Duration::ZERO,
            ),
        }
    }

    pub(crate) fn new(
        global: &GlobalScope,
        state: DocumentVisibilityState,
        timestamp: CrossProcessInstant,
        can_gc: CanGc,
    ) -> DomRoot<VisibilityStateEntry> {
        reflect_dom_object(
            Box::new(VisibilityStateEntry::new_inherited(state, timestamp)),
            global,
            can_gc,
        )
    }
}

impl VisibilityStateEntryMethods<crate::DomTypeHolder> for VisibilityStateEntry {
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

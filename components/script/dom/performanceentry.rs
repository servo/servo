/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use base::cross_process_instant::CrossProcessInstant;
use dom_struct::dom_struct;
use time::Duration;

use super::performance::ToDOMHighResTimeStamp;
use crate::dom::bindings::codegen::Bindings::PerformanceBinding::DOMHighResTimeStamp;
use crate::dom::bindings::codegen::Bindings::PerformanceEntryBinding::PerformanceEntryMethods;
use crate::dom::bindings::reflector::{reflect_dom_object, DomGlobal, Reflector};
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::DOMString;
use crate::dom::globalscope::GlobalScope;
use crate::script_runtime::CanGc;

#[dom_struct]
pub(crate) struct PerformanceEntry {
    reflector_: Reflector,
    name: DOMString,
    entry_type: DOMString,
    #[no_trace]
    start_time: Option<CrossProcessInstant>,
    /// The duration of this [`PerformanceEntry`]. This is a [`time::Duration`],
    /// because it can be negative and `std::time::Duration` cannot be.
    #[no_trace]
    #[ignore_malloc_size_of = "No MallocSizeOf support for `time` crate"]
    duration: Duration,
}

impl PerformanceEntry {
    pub(crate) fn new_inherited(
        name: DOMString,
        entry_type: DOMString,
        start_time: Option<CrossProcessInstant>,
        duration: Duration,
    ) -> PerformanceEntry {
        PerformanceEntry {
            reflector_: Reflector::new(),
            name,
            entry_type,
            start_time,
            duration,
        }
    }

    #[cfg_attr(crown, allow(crown::unrooted_must_root))]
    pub(crate) fn new(
        global: &GlobalScope,
        name: DOMString,
        entry_type: DOMString,
        start_time: CrossProcessInstant,
        duration: Duration,
        can_gc: CanGc,
    ) -> DomRoot<PerformanceEntry> {
        let entry = PerformanceEntry::new_inherited(name, entry_type, Some(start_time), duration);
        reflect_dom_object(Box::new(entry), global, can_gc)
    }

    pub(crate) fn entry_type(&self) -> &DOMString {
        &self.entry_type
    }

    pub(crate) fn name(&self) -> &DOMString {
        &self.name
    }

    pub(crate) fn start_time(&self) -> Option<CrossProcessInstant> {
        self.start_time
    }

    pub(crate) fn duration(&self) -> Duration {
        self.duration
    }
}

impl PerformanceEntryMethods<crate::DomTypeHolder> for PerformanceEntry {
    // https://w3c.github.io/performance-timeline/#dom-performanceentry-name
    fn Name(&self) -> DOMString {
        self.name.clone()
    }

    // https://w3c.github.io/performance-timeline/#dom-performanceentry-entrytype
    fn EntryType(&self) -> DOMString {
        self.entry_type.clone()
    }

    // https://w3c.github.io/performance-timeline/#dom-performanceentry-starttime
    fn StartTime(&self) -> DOMHighResTimeStamp {
        self.global()
            .performance()
            .maybe_to_dom_high_res_time_stamp(self.start_time)
    }

    // https://w3c.github.io/performance-timeline/#dom-performanceentry-duration
    fn Duration(&self) -> DOMHighResTimeStamp {
        self.duration.to_dom_high_res_time_stamp()
    }
}

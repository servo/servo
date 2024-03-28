/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;

use crate::dom::bindings::codegen::Bindings::PerformanceBinding::DOMHighResTimeStamp;
use crate::dom::bindings::codegen::Bindings::PerformanceEntryBinding::PerformanceEntryMethods;
use crate::dom::bindings::reflector::{reflect_dom_object, Reflector};
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::DOMString;
use crate::dom::globalscope::GlobalScope;
use crate::dom::performance::reduce_timing_resolution;

#[dom_struct]
pub struct PerformanceEntry {
    reflector_: Reflector,
    name: DOMString,
    entry_type: DOMString,
    start_time: f64,
    duration: f64,
}

impl PerformanceEntry {
    pub fn new_inherited(
        name: DOMString,
        entry_type: DOMString,
        start_time: f64,
        duration: f64,
    ) -> PerformanceEntry {
        PerformanceEntry {
            reflector_: Reflector::new(),
            name,
            entry_type,
            start_time,
            duration,
        }
    }

    #[allow(crown::unrooted_must_root)]
    pub fn new(
        global: &GlobalScope,
        name: DOMString,
        entry_type: DOMString,
        start_time: f64,
        duration: f64,
    ) -> DomRoot<PerformanceEntry> {
        let entry = PerformanceEntry::new_inherited(name, entry_type, start_time, duration);
        reflect_dom_object(Box::new(entry), global)
    }

    pub fn entry_type(&self) -> &DOMString {
        &self.entry_type
    }

    pub fn name(&self) -> &DOMString {
        &self.name
    }

    pub fn start_time(&self) -> f64 {
        self.start_time
    }

    pub fn duration(&self) -> f64 {
        self.duration
    }
}

impl PerformanceEntryMethods for PerformanceEntry {
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
        reduce_timing_resolution(self.start_time)
    }

    // https://w3c.github.io/performance-timeline/#dom-performanceentry-duration
    fn Duration(&self) -> DOMHighResTimeStamp {
        reduce_timing_resolution(self.duration)
    }
}

/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::PerformanceEntryBinding;
use dom::bindings::codegen::Bindings::PerformanceEntryBinding::PerformanceEntryMethods;
use dom::bindings::num::Finite;
use dom::bindings::reflector::{Reflector, reflect_dom_object};
use dom::bindings::root::DomRoot;
use dom::bindings::str::DOMString;
use dom::globalscope::GlobalScope;
use dom_struct::dom_struct;

#[dom_struct]
pub struct PerformanceEntry {
    reflector_: Reflector,
    name: DOMString,
    entry_type: DOMString,
    start_time: f64,
    duration: f64,
}

impl PerformanceEntry {
    pub fn new_inherited(name: DOMString,
                         entry_type: DOMString,
                         start_time: f64,
                         duration: f64) -> PerformanceEntry {
        PerformanceEntry {
            reflector_: Reflector::new(),
            name,
            entry_type,
            start_time,
            duration,
        }
    }

    #[allow(unrooted_must_root)]
    pub fn new(global: &GlobalScope,
               name: DOMString,
               entry_type: DOMString,
               start_time: f64,
               duration: f64) -> DomRoot<PerformanceEntry> {
        let entry = PerformanceEntry::new_inherited(name, entry_type, start_time, duration);
        reflect_dom_object(Box::new(entry), global, PerformanceEntryBinding::Wrap)
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
}

impl PerformanceEntryMethods for PerformanceEntry {
    // https://w3c.github.io/performance-timeline/#dom-performanceentry-name
    fn Name(&self) -> DOMString {
        DOMString::from(self.name.clone())
    }

    // https://w3c.github.io/performance-timeline/#dom-performanceentry-entrytype
    fn EntryType(&self) -> DOMString {
        DOMString::from(self.entry_type.clone())
    }

    // https://w3c.github.io/performance-timeline/#dom-performanceentry-starttime
    fn StartTime(&self) -> Finite<f64> {
        Finite::wrap(self.start_time)
    }

    // https://w3c.github.io/performance-timeline/#dom-performanceentry-duration
    fn Duration(&self) -> Finite<f64> {
        Finite::wrap(self.duration)
    }
}

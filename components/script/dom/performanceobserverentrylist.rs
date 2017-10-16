/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::cell::DomRefCell;
use dom::bindings::codegen::Bindings::PerformanceObserverEntryListBinding;
use dom::bindings::codegen::Bindings::PerformanceObserverEntryListBinding::PerformanceObserverEntryListMethods;
use dom::bindings::reflector::{Reflector, reflect_dom_object};
use dom::bindings::root::DomRoot;
use dom::bindings::str::DOMString;
use dom::globalscope::GlobalScope;
use dom::performance::PerformanceEntryList;
use dom::performanceentry::PerformanceEntry;
use dom_struct::dom_struct;

#[dom_struct]
pub struct PerformanceObserverEntryList {
    reflector_: Reflector,
    entries: DomRefCell<PerformanceEntryList>,
}

impl PerformanceObserverEntryList {
    fn new_inherited(entries: PerformanceEntryList) -> PerformanceObserverEntryList {
        PerformanceObserverEntryList {
            reflector_: Reflector::new(),
            entries: DomRefCell::new(entries),
        }
    }

    #[allow(unrooted_must_root)]
    pub fn new(global: &GlobalScope, entries: PerformanceEntryList)
        -> DomRoot<PerformanceObserverEntryList> {
        let observer_entry_list = PerformanceObserverEntryList::new_inherited(entries);
        reflect_dom_object(Box::new(observer_entry_list), global, PerformanceObserverEntryListBinding::Wrap)
    }
}

impl PerformanceObserverEntryListMethods for PerformanceObserverEntryList {
    // https://w3c.github.io/performance-timeline/#dom-performanceobserver
    fn GetEntries(&self) -> Vec<DomRoot<PerformanceEntry>> {
        self.entries.borrow().get_entries_by_name_and_type(None, None)
    }

    // https://w3c.github.io/performance-timeline/#dom-performanceobserver
    fn GetEntriesByType(&self, entry_type: DOMString) -> Vec<DomRoot<PerformanceEntry>> {
        self.entries.borrow().get_entries_by_name_and_type(None, Some(entry_type))
    }

    // https://w3c.github.io/performance-timeline/#dom-performanceobserver
    fn GetEntriesByName(&self, name: DOMString, entry_type: Option<DOMString>)
        -> Vec<DomRoot<PerformanceEntry>> {
        self.entries.borrow().get_entries_by_name_and_type(Some(name), entry_type)
    }
}

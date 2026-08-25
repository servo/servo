/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use js::context::JSContext;
use script_bindings::cell::DomRefCell;
use script_bindings::reflector::{Reflector, reflect_dom_object_with_cx};

use super::performance::PerformanceEntryList;
use super::performanceentry::{EntryType, PerformanceEntry};
use crate::dom::bindings::codegen::Bindings::PerformanceObserverEntryListBinding::PerformanceObserverEntryListMethods;
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::DOMString;
use crate::dom::globalscope::GlobalScope;

#[dom_struct]
pub(crate) struct PerformanceObserverEntryList {
    reflector_: Reflector,
    entries: DomRefCell<PerformanceEntryList>,
}

impl PerformanceObserverEntryList {
    fn new_inherited(entries: Vec<DomRoot<PerformanceEntry>>) -> PerformanceObserverEntryList {
        PerformanceObserverEntryList {
            reflector_: Reflector::new(),
            entries: DomRefCell::new(PerformanceEntryList::new(entries)),
        }
    }

    pub(crate) fn new(
        cx: &mut JSContext,
        global: &GlobalScope,
        entries: Vec<DomRoot<PerformanceEntry>>,
    ) -> DomRoot<PerformanceObserverEntryList> {
        reflect_dom_object_with_cx(
            Box::new(PerformanceObserverEntryList::new_inherited(entries)),
            global,
            cx,
        )
    }
}

impl PerformanceObserverEntryListMethods<crate::DomTypeHolder> for PerformanceObserverEntryList {
    /// <https://w3c.github.io/performance-timeline/#dom-performanceobserver>
    fn GetEntries(&self) -> Vec<DomRoot<PerformanceEntry>> {
        self.entries
            .borrow()
            .get_entries_by_name_and_type(None, None)
    }

    /// <https://w3c.github.io/performance-timeline/#dom-performanceobserver>
    fn GetEntriesByType(&self, entry_type: DOMString) -> Vec<DomRoot<PerformanceEntry>> {
        let Ok(entry_type) = EntryType::try_from(&*entry_type.str()) else {
            return Vec::new();
        };
        self.entries
            .borrow()
            .get_entries_by_name_and_type(None, Some(entry_type))
    }

    /// <https://w3c.github.io/performance-timeline/#dom-performanceobserver>
    fn GetEntriesByName(
        &self,
        name: DOMString,
        entry_type: Option<DOMString>,
    ) -> Vec<DomRoot<PerformanceEntry>> {
        let entry_type = match entry_type {
            Some(entry_type) => {
                let Ok(entry_type) = EntryType::try_from(&*entry_type.str()) else {
                    return Vec::new();
                };
                Some(entry_type)
            },
            None => None,
        };
        self.entries
            .borrow()
            .get_entries_by_name_and_type(Some(name), entry_type)
    }
}

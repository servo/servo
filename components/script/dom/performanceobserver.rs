/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::dom::bindings::callback::ExceptionHandling;
use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::PerformanceBinding::PerformanceEntryList as DOMPerformanceEntryList;
use crate::dom::bindings::codegen::Bindings::PerformanceObserverBinding;
use crate::dom::bindings::codegen::Bindings::PerformanceObserverBinding::PerformanceObserverCallback;
use crate::dom::bindings::codegen::Bindings::PerformanceObserverBinding::PerformanceObserverInit;
use crate::dom::bindings::codegen::Bindings::PerformanceObserverBinding::PerformanceObserverMethods;
use crate::dom::bindings::error::{Error, Fallible};
use crate::dom::bindings::reflector::{reflect_dom_object, DomObject, Reflector};
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::DOMString;
use crate::dom::globalscope::GlobalScope;
use crate::dom::performance::PerformanceEntryList;
use crate::dom::performanceentry::PerformanceEntry;
use crate::dom::performanceobserverentrylist::PerformanceObserverEntryList;
use dom_struct::dom_struct;
use std::rc::Rc;

/// List of allowed performance entry types.
const VALID_ENTRY_TYPES: &'static [&'static str] = &[
    "mark",    // User Timing API
    "measure", // User Timing API
    "resource", // Resource Timing API
    "navigation", // Navigation Timing API
    // "frame", //TODO Frame Timing API
    // "server", XXX Server Timing API
    "paint", // Paint Timing API
];

#[dom_struct]
pub struct PerformanceObserver {
    reflector_: Reflector,
    #[ignore_malloc_size_of = "can't measure Rc values"]
    callback: Rc<PerformanceObserverCallback>,
    entries: DomRefCell<DOMPerformanceEntryList>,
}

impl PerformanceObserver {
    fn new_inherited(
        callback: Rc<PerformanceObserverCallback>,
        entries: DomRefCell<DOMPerformanceEntryList>,
    ) -> PerformanceObserver {
        PerformanceObserver {
            reflector_: Reflector::new(),
            callback,
            entries,
        }
    }

    #[allow(unrooted_must_root)]
    pub fn new(
        global: &GlobalScope,
        callback: Rc<PerformanceObserverCallback>,
        entries: DOMPerformanceEntryList,
    ) -> DomRoot<PerformanceObserver> {
        let observer = PerformanceObserver::new_inherited(callback, DomRefCell::new(entries));
        reflect_dom_object(Box::new(observer), global, PerformanceObserverBinding::Wrap)
    }

    pub fn Constructor(
        global: &GlobalScope,
        callback: Rc<PerformanceObserverCallback>,
    ) -> Fallible<DomRoot<PerformanceObserver>> {
        Ok(PerformanceObserver::new(global, callback, Vec::new()))
    }

    /// Buffer a new performance entry.
    pub fn queue_entry(&self, entry: &PerformanceEntry) {
        self.entries.borrow_mut().push(DomRoot::from_ref(entry));
    }

    /// Trigger performance observer callback with the list of performance entries
    /// buffered since the last callback call.
    pub fn notify(&self) {
        let entries = self.entries.borrow();
        if entries.is_empty() {
            return;
        }
        let mut entries = entries.clone();
        let global = self.global();
        let entry_list = PerformanceEntryList::new(entries.drain(..).collect());
        let observer_entry_list = PerformanceObserverEntryList::new(&global, entry_list);
        let _ = self
            .callback
            .Call__(&observer_entry_list, self, ExceptionHandling::Report);
    }

    pub fn callback(&self) -> Rc<PerformanceObserverCallback> {
        self.callback.clone()
    }

    pub fn entries(&self) -> DOMPerformanceEntryList {
        self.entries.borrow().clone()
    }

    pub fn set_entries(&self, entries: DOMPerformanceEntryList) {
        *self.entries.borrow_mut() = entries;
    }
}

impl PerformanceObserverMethods for PerformanceObserver {
    // https://w3c.github.io/performance-timeline/#dom-performanceobserver-observe()
    fn Observe(&self, options: &PerformanceObserverInit) -> Fallible<()> {
        // step 1
        // Make sure the client is asking to observe events from allowed entry types.
        let entry_types = options
            .entryTypes
            .iter()
            .filter(|e| VALID_ENTRY_TYPES.contains(&e.as_ref()))
            .map(|e| e.clone())
            .collect::<Vec<DOMString>>();
        // step 2
        // There must be at least one valid entry type.
        if entry_types.is_empty() {
            return Err(Error::Type("entryTypes cannot be empty".to_string()));
        }

        // step 3-4-5
        self.global()
            .performance()
            .add_observer(self, entry_types, options.buffered);

        Ok(())
    }

    // https://w3c.github.io/performance-timeline/#dom-performanceobserver-disconnect()
    fn Disconnect(&self) {
        self.global().performance().remove_observer(self);
        self.entries.borrow_mut().clear();
    }
}

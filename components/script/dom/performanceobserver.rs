/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::callback::ExceptionHandling;
use dom::bindings::cell::DomRefCell;
use dom::bindings::codegen::Bindings::PerformanceBinding::PerformanceEntryList as DOMPerformanceEntryList;
use dom::bindings::codegen::Bindings::PerformanceObserverBinding;
use dom::bindings::codegen::Bindings::PerformanceObserverBinding::PerformanceObserverCallback;
use dom::bindings::codegen::Bindings::PerformanceObserverBinding::PerformanceObserverInit;
use dom::bindings::codegen::Bindings::PerformanceObserverBinding::PerformanceObserverMethods;
use dom::bindings::error::{Error, Fallible};
use dom::bindings::reflector::{DomObject, Reflector, reflect_dom_object};
use dom::bindings::root::DomRoot;
use dom::bindings::str::DOMString;
use dom::globalscope::GlobalScope;
use dom::performance::PerformanceEntryList;
use dom::performanceentry::PerformanceEntry;
use dom::performanceobserverentrylist::PerformanceObserverEntryList;
use dom_struct::dom_struct;
use std::rc::Rc;
use typeholder::TypeHolderTrait;

/// List of allowed performance entry types.
const VALID_ENTRY_TYPES: &'static [&'static str] = &[
    "mark", // User Timing API
    "measure", // User Timing API
    // "resource", XXX Resource Timing API
    // "server", XXX Server Timing API
    "paint", // Paint Timing API
];

#[dom_struct]
pub struct PerformanceObserver<TH: TypeHolderTrait> {
    reflector_: Reflector<TH>,
    #[ignore_malloc_size_of = "can't measure Rc values"]
    callback: Rc<PerformanceObserverCallback<TH>>,
    entries: DomRefCell<DOMPerformanceEntryList<TH>>,
}

impl<TH: TypeHolderTrait> PerformanceObserver<TH> {
    fn new_inherited(callback: Rc<PerformanceObserverCallback<TH>>,
                     entries: DomRefCell<DOMPerformanceEntryList<TH>>)
        -> PerformanceObserver<TH> {
        PerformanceObserver {
            reflector_: Reflector::new(),
            callback,
            entries,
        }
    }

    #[allow(unrooted_must_root)]
    pub fn new(global: &GlobalScope<TH>,
               callback: Rc<PerformanceObserverCallback<TH>>,
               entries: DOMPerformanceEntryList<TH>)
        -> DomRoot<PerformanceObserver<TH>> {
        let observer = PerformanceObserver::new_inherited(callback, DomRefCell::new(entries));
        reflect_dom_object(Box::new(observer), global, PerformanceObserverBinding::Wrap)
    }

    pub fn Constructor(global: &GlobalScope<TH>, callback: Rc<PerformanceObserverCallback<TH>>)
        -> Fallible<DomRoot<PerformanceObserver<TH>>> {
        Ok(PerformanceObserver::new(global, callback, Vec::new()))
    }

    /// Buffer a new performance entry.
    pub fn queue_entry(&self, entry: &PerformanceEntry<TH>) {
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
        let _ = self.callback.Call__(&observer_entry_list, self, ExceptionHandling::Report);
    }

    pub fn callback(&self) -> Rc<PerformanceObserverCallback<TH>> {
        self.callback.clone()
    }

    pub fn entries(&self) -> DOMPerformanceEntryList<TH> {
        self.entries.borrow().clone()
    }

    pub fn set_entries(&self, entries: DOMPerformanceEntryList<TH>) {
        *self.entries.borrow_mut() = entries;
    }
}

impl<TH: TypeHolderTrait> PerformanceObserverMethods for PerformanceObserver<TH> {
    // https://w3c.github.io/performance-timeline/#dom-performanceobserver-observe()
    fn Observe(&self, options: &PerformanceObserverInit) -> Fallible<()> {
        // step 1
        // Make sure the client is asking to observe events from allowed entry types.
        let entry_types = options.entryTypes.iter()
                                            .filter(|e| VALID_ENTRY_TYPES.contains(&e.as_ref()))
                                            .map(|e| e.clone())
                                            .collect::<Vec<DOMString>>();
        // step 2
        // There must be at least one valid entry type.
        if entry_types.is_empty() {
            return Err(Error::Type("entryTypes cannot be empty".to_string()));
        }

        // step 3-4-5
        self.global().performance().add_observer(self, entry_types, options.buffered);

        Ok(())
    }

    // https://w3c.github.io/performance-timeline/#dom-performanceobserver-disconnect()
    fn Disconnect(&self) {
        self.global().performance().remove_observer(self);
        self.entries.borrow_mut().clear();
    }
}

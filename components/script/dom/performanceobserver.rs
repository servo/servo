/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::Cell;
use std::rc::Rc;

use dom_struct::dom_struct;
use js::rust::{HandleObject, MutableHandleValue};

use crate::dom::bindings::callback::ExceptionHandling;
use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::PerformanceBinding::PerformanceEntryList as DOMPerformanceEntryList;
use crate::dom::bindings::codegen::Bindings::PerformanceObserverBinding::{
    PerformanceObserverCallback, PerformanceObserverInit, PerformanceObserverMethods,
};
use crate::dom::bindings::error::{Error, Fallible};
use crate::dom::bindings::reflector::{reflect_dom_object_with_proto, DomGlobal, Reflector};
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::DOMString;
use crate::dom::console::Console;
use crate::dom::globalscope::GlobalScope;
use crate::dom::performance::PerformanceEntryList;
use crate::dom::performanceentry::PerformanceEntry;
use crate::dom::performanceobserverentrylist::PerformanceObserverEntryList;
use crate::script_runtime::{CanGc, JSContext};

/// List of allowed performance entry types, in alphabetical order.
pub(crate) const VALID_ENTRY_TYPES: &[&str] = &[
    // "frame", //TODO Frame Timing API
    "mark",       // User Timing API
    "measure",    // User Timing API
    "navigation", // Navigation Timing API
    "paint",      // Paint Timing API
    "resource",   // Resource Timing API
                  // "server", XXX Server Timing API
];

#[derive(Clone, Copy, JSTraceable, MallocSizeOf, PartialEq)]
enum ObserverType {
    Undefined,
    Single,
    Multiple,
}

#[dom_struct]
pub(crate) struct PerformanceObserver {
    reflector_: Reflector,
    #[ignore_malloc_size_of = "can't measure Rc values"]
    callback: Rc<PerformanceObserverCallback>,
    entries: DomRefCell<DOMPerformanceEntryList>,
    observer_type: Cell<ObserverType>,
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
            observer_type: Cell::new(ObserverType::Undefined),
        }
    }

    pub(crate) fn new(
        global: &GlobalScope,
        callback: Rc<PerformanceObserverCallback>,
        entries: DOMPerformanceEntryList,
        can_gc: CanGc,
    ) -> DomRoot<PerformanceObserver> {
        Self::new_with_proto(global, None, callback, entries, can_gc)
    }

    #[cfg_attr(crown, allow(crown::unrooted_must_root))]
    fn new_with_proto(
        global: &GlobalScope,
        proto: Option<HandleObject>,
        callback: Rc<PerformanceObserverCallback>,
        entries: DOMPerformanceEntryList,
        can_gc: CanGc,
    ) -> DomRoot<PerformanceObserver> {
        let observer = PerformanceObserver::new_inherited(callback, DomRefCell::new(entries));
        reflect_dom_object_with_proto(Box::new(observer), global, proto, can_gc)
    }

    /// Buffer a new performance entry.
    pub(crate) fn queue_entry(&self, entry: &PerformanceEntry) {
        self.entries.borrow_mut().push(DomRoot::from_ref(entry));
    }

    /// Trigger performance observer callback with the list of performance entries
    /// buffered since the last callback call.
    pub(crate) fn notify(&self) {
        if self.entries.borrow().is_empty() {
            return;
        }
        let entry_list = PerformanceEntryList::new(self.entries.borrow_mut().drain(..).collect());
        let observer_entry_list =
            PerformanceObserverEntryList::new(&self.global(), entry_list, CanGc::note());
        // using self both as thisArg and as the second formal argument
        let _ = self
            .callback
            .Call_(self, &observer_entry_list, self, ExceptionHandling::Report);
    }

    pub(crate) fn callback(&self) -> Rc<PerformanceObserverCallback> {
        self.callback.clone()
    }

    pub(crate) fn entries(&self) -> DOMPerformanceEntryList {
        self.entries.borrow().clone()
    }

    pub(crate) fn set_entries(&self, entries: DOMPerformanceEntryList) {
        *self.entries.borrow_mut() = entries;
    }
}

impl PerformanceObserverMethods<crate::DomTypeHolder> for PerformanceObserver {
    // https://w3c.github.io/performance-timeline/#dom-performanceobserver-constructor
    fn Constructor(
        global: &GlobalScope,
        proto: Option<HandleObject>,
        can_gc: CanGc,
        callback: Rc<PerformanceObserverCallback>,
    ) -> Fallible<DomRoot<PerformanceObserver>> {
        Ok(PerformanceObserver::new_with_proto(
            global,
            proto,
            callback,
            Vec::new(),
            can_gc,
        ))
    }

    // https://w3c.github.io/performance-timeline/#supportedentrytypes-attribute
    fn SupportedEntryTypes(cx: JSContext, global: &GlobalScope, retval: MutableHandleValue) {
        // While this is exposed through a method of PerformanceObserver,
        // it is specified as associated with the global scope.
        global.supported_performance_entry_types(cx, retval)
    }

    // https://w3c.github.io/performance-timeline/#dom-performanceobserver-observe()
    fn Observe(&self, options: &PerformanceObserverInit) -> Fallible<()> {
        // Step 1 is self

        // Step 2 is self.global()

        // Step 3
        if options.entryTypes.is_none() && options.type_.is_none() {
            return Err(Error::Syntax);
        }

        // Step 4
        if options.entryTypes.is_some() && (options.buffered.is_some() || options.type_.is_some()) {
            return Err(Error::Syntax);
        }

        // If this point is reached, then one of options.entryTypes or options.type_
        // is_some, but not both.

        // Step 5
        match self.observer_type.get() {
            ObserverType::Undefined => {
                if options.entryTypes.is_some() {
                    self.observer_type.set(ObserverType::Multiple);
                } else {
                    self.observer_type.set(ObserverType::Single);
                }
            },
            ObserverType::Single => {
                if options.entryTypes.is_some() {
                    return Err(Error::InvalidModification);
                }
            },
            ObserverType::Multiple => {
                if options.type_.is_some() {
                    return Err(Error::InvalidModification);
                }
            },
        }

        // The entryTypes and type paths diverge here
        if let Some(entry_types) = &options.entryTypes {
            // Steps 6.1 - 6.2
            let entry_types = entry_types
                .iter()
                .filter(|e| VALID_ENTRY_TYPES.contains(&e.as_ref()))
                .cloned()
                .collect::<Vec<DOMString>>();

            // Step 6.3
            if entry_types.is_empty() {
                Console::internal_warn(
                    &self.global(),
                    DOMString::from("No valid entry type provided to observe()."),
                );
                return Ok(());
            }

            // Steps 6.4-6.5
            // This never pre-fills buffered entries, and
            // any existing types are replaced.
            self.global()
                .performance()
                .add_multiple_type_observer(self, entry_types);
            Ok(())
        } else if let Some(entry_type) = &options.type_ {
            // Step 7.2
            if !VALID_ENTRY_TYPES.contains(&entry_type.as_ref()) {
                Console::internal_warn(
                    &self.global(),
                    DOMString::from("No valid entry type provided to observe()."),
                );
                return Ok(());
            }

            // Steps 7.3-7.5
            // This may pre-fill buffered entries, and
            // existing types are appended to.
            self.global().performance().add_single_type_observer(
                self,
                entry_type,
                options.buffered.unwrap_or(false),
            );
            Ok(())
        } else {
            // Step 7.1
            unreachable!()
        }
    }

    // https://w3c.github.io/performance-timeline/#dom-performanceobserver-disconnect
    fn Disconnect(&self) {
        self.global().performance().remove_observer(self);
        self.entries.borrow_mut().clear();
    }

    // https://w3c.github.io/performance-timeline/#takerecords-method
    fn TakeRecords(&self) -> Vec<DomRoot<PerformanceEntry>> {
        let mut entries = self.entries.borrow_mut();
        let taken = entries
            .iter()
            .map(|entry| DomRoot::from_ref(&**entry))
            .collect();
        entries.clear();
        taken
    }
}

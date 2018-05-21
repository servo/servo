/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::cell::DomRefCell;
use dom::bindings::codegen::Bindings::PerformanceBinding;
use dom::bindings::codegen::Bindings::PerformanceBinding::{DOMHighResTimeStamp, PerformanceMethods};
use dom::bindings::codegen::Bindings::PerformanceBinding::PerformanceEntryList as DOMPerformanceEntryList;
use dom::bindings::error::{Error, Fallible};
use dom::bindings::inheritance::Castable;
use dom::bindings::num::Finite;
use dom::bindings::reflector::{DomObject, Reflector, reflect_dom_object};
use dom::bindings::root::{Dom, DomRoot};
use dom::bindings::str::DOMString;
use dom::globalscope::GlobalScope;
use dom::performanceentry::PerformanceEntry;
use dom::performancemark::PerformanceMark;
use dom::performancemeasure::PerformanceMeasure;
use dom::performanceobserver::PerformanceObserver as DOMPerformanceObserver;
use dom::performancetiming::PerformanceTiming;
use dom::window::Window;
use dom_struct::dom_struct;
use metrics::ToMs;
use std::cell::Cell;
use std::cmp::Ordering;
use time;

const INVALID_ENTRY_NAMES: &'static [&'static str] = &[
    "navigationStart",
    "unloadEventStart",
    "unloadEventEnd",
    "redirectStart",
    "redirectEnd",
    "fetchStart",
    "domainLookupStart",
    "domainLookupEnd",
    "connectStart",
    "connectEnd",
    "secureConnectionStart",
    "requestStart",
    "responseStart",
    "responseEnd",
    "domLoading",
    "domInteractive",
    "domContentLoadedEventStart",
    "domContentLoadedEventEnd",
    "domComplete",
    "loadEventStart",
    "loadEventEnd",
];

/// Implementation of a list of PerformanceEntry items shared by the
/// Performance and PerformanceObserverEntryList interfaces implementations.
#[derive(JSTraceable, MallocSizeOf)]
pub struct PerformanceEntryList {
    entries: DOMPerformanceEntryList,
}

impl PerformanceEntryList {
    pub fn new(entries: DOMPerformanceEntryList) -> Self {
        PerformanceEntryList {
            entries,
        }
    }

    pub fn get_entries_by_name_and_type(&self, name: Option<DOMString>, entry_type: Option<DOMString>)
        -> Vec<DomRoot<PerformanceEntry>> {
        let mut res = self.entries.iter().filter(|e|
            name.as_ref().map_or(true, |name_| *e.name() == *name_) &&
            entry_type.as_ref().map_or(true, |type_| *e.entry_type() == *type_)
        ).map(|e| e.clone()).collect::<Vec<DomRoot<PerformanceEntry>>>();
        res.sort_by(|a, b| a.start_time().partial_cmp(&b.start_time()).unwrap_or(Ordering::Equal));
        res
    }

    pub fn clear_entries_by_name_and_type(&mut self, name: Option<DOMString>,
                                          entry_type: Option<DOMString>) {
        self.entries.retain(|e|
            name.as_ref().map_or(true, |name_| *e.name() == *name_) &&
            entry_type.as_ref().map_or(true, |type_| *e.entry_type() == *type_)
        );
    }

    fn get_last_entry_start_time_with_name_and_type(&self, name: DOMString,
                                                    entry_type: DOMString) -> f64 {
        match self.entries.iter()
                          .rev()
                          .find(|e| *e.entry_type() == *entry_type &&
                                    *e.name() == *name) {
            Some(entry) => entry.start_time(),
            None => 0.,
        }
    }
}

impl IntoIterator for PerformanceEntryList {
    type Item = DomRoot<PerformanceEntry>;
    type IntoIter = ::std::vec::IntoIter<DomRoot<PerformanceEntry>>;

    fn into_iter(self) -> Self::IntoIter {
        self.entries.into_iter()
    }
}

#[derive(JSTraceable, MallocSizeOf)]
struct PerformanceObserver {
    observer: DomRoot<DOMPerformanceObserver>,
    entry_types: Vec<DOMString>,
}

#[dom_struct]
pub struct Performance {
    reflector_: Reflector,
    timing: Option<Dom<PerformanceTiming>>,
    entries: DomRefCell<PerformanceEntryList>,
    observers: DomRefCell<Vec<PerformanceObserver>>,
    pending_notification_observers_task: Cell<bool>,
    navigation_start_precise: u64,
}

impl Performance {
    fn new_inherited(global: &GlobalScope,
                     navigation_start: u64,
                     navigation_start_precise: u64) -> Performance {
        Performance {
            reflector_: Reflector::new(),
            timing: if global.is::<Window>() {
                Some(Dom::from_ref(&*PerformanceTiming::new(global.as_window(),
                                                           navigation_start,
                                                           navigation_start_precise)))
            } else {
                None
            },
            entries: DomRefCell::new(PerformanceEntryList::new(Vec::new())),
            observers: DomRefCell::new(Vec::new()),
            pending_notification_observers_task: Cell::new(false),
            navigation_start_precise,
        }
    }

    pub fn new(global: &GlobalScope,
               navigation_start: u64,
               navigation_start_precise: u64) -> DomRoot<Performance> {
        reflect_dom_object(
            Box::new(Performance::new_inherited(global, navigation_start, navigation_start_precise)),
            global,
            PerformanceBinding::Wrap
        )
    }

    /// Add a PerformanceObserver to the list of observers with a set of
    /// observed entry types.
    pub fn add_observer(&self,
                        observer: &DOMPerformanceObserver,
                        entry_types: Vec<DOMString>,
                        buffered: bool) {
        if buffered {
            let entries = self.entries.borrow();
            let mut new_entries = entry_types.iter()
                            .flat_map(|e| entries.get_entries_by_name_and_type(None, Some(e.clone())))
                            .collect::<DOMPerformanceEntryList>();
            let mut obs_entries = observer.entries();
            obs_entries.append(&mut new_entries);
            observer.set_entries(obs_entries);
        }
        let mut observers = self.observers.borrow_mut();
        match observers.iter().position(|o| *o.observer == *observer) {
            // If the observer is already in the list, we only update the observed
            // entry types.
            Some(p) => observers[p].entry_types = entry_types,
            // Otherwise, we create and insert the new PerformanceObserver.
            None => observers.push(PerformanceObserver {
                observer: DomRoot::from_ref(observer),
                entry_types
            })
        };
    }

    /// Remove a PerformanceObserver from the list of observers.
    pub fn remove_observer(&self, observer: &DOMPerformanceObserver) {
        let mut observers = self.observers.borrow_mut();
        let index = match observers.iter().position(|o| &(*o.observer) == observer) {
            Some(p) => p,
            None => return,
        };

        observers.remove(index);
    }

    /// Queue a notification for each performance observer interested in
    /// this type of performance entry and queue a low priority task to
    /// notify the observers if no other notification task is already queued.
    ///
    /// Algorithm spec:
    /// <https://w3c.github.io/performance-timeline/#queue-a-performanceentry>
    pub fn queue_entry(&self, entry: &PerformanceEntry,
                       add_to_performance_entries_buffer: bool) {
        // Steps 1-3.
        // Add the performance entry to the list of performance entries that have not
        // been notified to each performance observer owner, filtering the ones it's
        // interested in.
        for o in self.observers.borrow().iter().filter(|o| o.entry_types.contains(entry.entry_type())) {
            o.observer.queue_entry(entry);
        }

        // Step 4.
        // If the "add to performance entry buffer flag" is set, add the
        // new entry to the buffer.
        if add_to_performance_entries_buffer {
            self.entries.borrow_mut().entries.push(DomRoot::from_ref(entry));
        }

        // Step 5.
        // If there is already a queued notification task, we just bail out.
        if self.pending_notification_observers_task.get() {
            return;
        }

        // Step 6.
        // Queue a new notification task.
        self.pending_notification_observers_task.set(true);
        let task_source = self.global().performance_timeline_task_source();
        task_source.queue_notification(&self.global());
    }

    /// Observers notifications task.
    ///
    /// Algorithm spec (step 7):
    /// <https://w3c.github.io/performance-timeline/#queue-a-performanceentry>
    pub fn notify_observers(&self) {
        // Step 7.1.
        self.pending_notification_observers_task.set(false);

        // Step 7.2.
        // We have to operate over a copy of the performance observers to avoid
        // the risk of an observer's callback modifying the list of registered
        // observers.
        let observers: Vec<DomRoot<DOMPerformanceObserver>> =
            self.observers.borrow().iter()
                                   .map(|o| DOMPerformanceObserver::new(&self.global(),
                                                                        o.observer.callback(),
                                                                        o.observer.entries()))
                                   .collect();

        // Step 7.3.
        for o in observers.iter() {
            o.notify();
        }
    }

    fn now(&self) -> f64 {
        let nav_start = match self.timing {
            Some(ref timing) => timing.navigation_start_precise(),
            None => self.navigation_start_precise,
        };
        (time::precise_time_ns() - nav_start).to_ms()
    }
}

impl PerformanceMethods for Performance {
    // https://dvcs.w3.org/hg/webperf/raw-file/tip/specs/NavigationTiming/Overview.html#performance-timing-attribute
    fn Timing(&self) -> DomRoot<PerformanceTiming> {
        match self.timing {
            Some(ref timing) => DomRoot::from_ref(&*timing),
            None => unreachable!("Are we trying to expose Performance.timing in workers?"),
        }
    }

    // https://dvcs.w3.org/hg/webperf/raw-file/tip/specs/HighResolutionTime/Overview.html#dom-performance-now
    fn Now(&self) -> DOMHighResTimeStamp {
        Finite::wrap(self.now())
    }

    // https://www.w3.org/TR/performance-timeline-2/#dom-performance-getentries
    fn GetEntries(&self) -> Vec<DomRoot<PerformanceEntry>> {
        self.entries.borrow().get_entries_by_name_and_type(None, None)
    }

    // https://www.w3.org/TR/performance-timeline-2/#dom-performance-getentriesbytype
    fn GetEntriesByType(&self, entry_type: DOMString) -> Vec<DomRoot<PerformanceEntry>> {
        self.entries.borrow().get_entries_by_name_and_type(None, Some(entry_type))
    }

    // https://www.w3.org/TR/performance-timeline-2/#dom-performance-getentriesbyname
    fn GetEntriesByName(&self, name: DOMString, entry_type: Option<DOMString>)
        -> Vec<DomRoot<PerformanceEntry>> {
        self.entries.borrow().get_entries_by_name_and_type(Some(name), entry_type)
    }

    // https://w3c.github.io/user-timing/#dom-performance-mark
    fn Mark(&self, mark_name: DOMString) -> Fallible<()> {
        let global = self.global();
        // Step 1.
        if global.is::<Window>() && INVALID_ENTRY_NAMES.contains(&mark_name.as_ref()) {
            return Err(Error::Syntax);
        }

        // Steps 2 to 6.
        let entry = PerformanceMark::new(&global,
                                         mark_name,
                                         self.now(),
                                         0.);
        // Steps 7 and 8.
        self.queue_entry(&entry.upcast::<PerformanceEntry>(),
                         true /* buffer performance entry */);

        // Step 9.
        Ok(())
    }

    // https://w3c.github.io/user-timing/#dom-performance-clearmarks
    fn ClearMarks(&self, mark_name: Option<DOMString>) {
        self.entries.borrow_mut().clear_entries_by_name_and_type(mark_name,
                                                                 Some(DOMString::from("mark")));
    }

    // https://w3c.github.io/user-timing/#dom-performance-measure
    fn Measure(&self,
               measure_name: DOMString,
               start_mark: Option<DOMString>,
               end_mark: Option<DOMString>) -> Fallible<()> {
        // Steps 1 and 2.
        let end_time = match end_mark {
            Some(name) =>
                self.entries.borrow().get_last_entry_start_time_with_name_and_type(
                    DOMString::from("mark"), name),
            None => self.now(),
        };

        // Step 3.
        let start_time = match start_mark {
            Some(name) =>
                self.entries.borrow().get_last_entry_start_time_with_name_and_type(
                    DOMString::from("mark"), name),
            None => 0.,
        };

        // Steps 4 to 8.
        let entry = PerformanceMeasure::new(&self.global(),
                                            measure_name,
                                            start_time,
                                            end_time - start_time);

        // Step 9 and 10.
        self.queue_entry(&entry.upcast::<PerformanceEntry>(),
                         true /* buffer performance entry */);

        // Step 11.
        Ok(())
    }

    // https://w3c.github.io/user-timing/#dom-performance-clearmeasures
    fn ClearMeasures(&self, measure_name: Option<DOMString>) {
        self.entries.borrow_mut().clear_entries_by_name_and_type(measure_name,
                                                                 Some(DOMString::from("measure")));
    }
}

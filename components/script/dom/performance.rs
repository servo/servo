/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::cell::DOMRefCell;
use dom::bindings::codegen::Bindings::PerformanceBinding;
use dom::bindings::codegen::Bindings::PerformanceBinding::{DOMHighResTimeStamp, PerformanceMethods};
use dom::bindings::codegen::Bindings::PerformanceBinding::PerformanceEntryList as DOMPerformanceEntryList;
use dom::bindings::js::{JS, Root};
use dom::bindings::num::Finite;
use dom::bindings::refcounted::Trusted;
use dom::bindings::reflector::{DomObject, Reflector, reflect_dom_object};
use dom::bindings::str::DOMString;
use dom::performanceentry::PerformanceEntry;
use dom::performanceobserver::PerformanceObserver as DOMPerformanceObserver;
use dom::performancetiming::PerformanceTiming;
use dom::window::Window;
use dom_struct::dom_struct;
use script_thread::{Runnable, ScriptThread};
use std::cell::Cell;
use std::cmp::Ordering;
use time;

/// Implementation of a list of PerformanceEntry items shared by the
/// Performance and PerformanceObserverEntryList interfaces implementations.
#[derive(HeapSizeOf, JSTraceable)]
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
        -> Vec<Root<PerformanceEntry>> {
        let mut res = self.entries.iter().filter(|e|
            name.as_ref().map_or(true, |name_| *e.name() == *name_) &&
            entry_type.as_ref().map_or(true, |type_| *e.entry_type() == *type_)
        ).map(|e| e.clone()).collect::<Vec<Root<PerformanceEntry>>>();
        res.sort_by(|a, b| a.start_time().partial_cmp(&b.start_time()).unwrap_or(Ordering::Equal));
        res
    }
}

impl IntoIterator for PerformanceEntryList {
    type Item = Root<PerformanceEntry>;
    type IntoIter = ::std::vec::IntoIter<Root<PerformanceEntry>>;

    fn into_iter(self) -> Self::IntoIter {
        self.entries.into_iter()
    }
}

#[derive(HeapSizeOf, JSTraceable)]
struct PerformanceObserver {
    observer: Root<DOMPerformanceObserver>,
    entry_types: Vec<DOMString>,
}

#[dom_struct]
pub struct Performance {
    reflector_: Reflector,
    timing: JS<PerformanceTiming>,
    entries: DOMRefCell<PerformanceEntryList>,
    observers: DOMRefCell<Vec<PerformanceObserver>>,
    pending_notification_observers_task: Cell<bool>,
}

impl Performance {
    fn new_inherited(window: &Window,
                     navigation_start: u64,
                     navigation_start_precise: f64) -> Performance {
        Performance {
            reflector_: Reflector::new(),
            timing: JS::from_ref(&*PerformanceTiming::new(window,
                                                            navigation_start,
                                                            navigation_start_precise)),
            entries: DOMRefCell::new(PerformanceEntryList::new(Vec::new())),
            observers: DOMRefCell::new(Vec::new()),
            pending_notification_observers_task: Cell::new(false),
        }
    }

    pub fn new(window: &Window,
               navigation_start: u64,
               navigation_start_precise: f64) -> Root<Performance> {
        reflect_dom_object(box Performance::new_inherited(window,
                                                          navigation_start,
                                                          navigation_start_precise),
                           window,
                           PerformanceBinding::Wrap)
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
        match observers.iter().position(|o| &(*o.observer) == observer) {
            // If the observer is already in the list, we only update the observed
            // entry types.
            Some(p) => observers[p].entry_types = entry_types,
            // Otherwise, we create and insert the new PerformanceObserver.
            None => observers.push(PerformanceObserver {
                observer: Root::from_ref(observer),
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
    /// https://w3c.github.io/performance-timeline/#queue-a-performanceentry
    ///
    /// XXX This should be called at some point by the User Timing, Resource
    ///     Timing, Server Timing and Paint Timing APIs.
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
            self.entries.borrow_mut().entries.push(Root::from_ref(entry));
        }

        // Step 5.
        // If there is already a queued notification task, we just bail out.
        if self.pending_notification_observers_task.get() {
            return;
        }

        // Step 6.
        // Queue a new notification task.
        self.pending_notification_observers_task.set(true);
        let global = self.global();
        let window = global.as_window();
        let task_source = window.performance_timeline_task_source();
        task_source.queue_notification(self, window);
    }

    /// Observers notifications task.
    ///
    /// Algorithm spec (step 7):
    /// https://w3c.github.io/performance-timeline/#queue-a-performanceentry
    fn notify_observers(&self) {
        // Step 7.1.
        self.pending_notification_observers_task.set(false);

        // Step 7.2.
        // We have to operate over a copy of the performance observers to avoid
        // the risk of an observer's callback modifying the list of registered
        // observers.
        let observers: Vec<Root<DOMPerformanceObserver>> =
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
}

pub struct NotifyPerformanceObserverRunnable {
    owner: Trusted<Performance>,
}

impl NotifyPerformanceObserverRunnable {
    pub fn new(owner: Trusted<Performance>) -> Self {
        NotifyPerformanceObserverRunnable {
            owner,
        }
    }
}

impl Runnable for NotifyPerformanceObserverRunnable {
    fn main_thread_handler(self: Box<NotifyPerformanceObserverRunnable>,
                           _: &ScriptThread) {
        self.owner.root().notify_observers();
    }
}

impl PerformanceMethods for Performance {
    // https://dvcs.w3.org/hg/webperf/raw-file/tip/specs/NavigationTiming/Overview.html#performance-timing-attribute
    fn Timing(&self) -> Root<PerformanceTiming> {
        Root::from_ref(&*self.timing)
    }

    // https://dvcs.w3.org/hg/webperf/raw-file/tip/specs/HighResolutionTime/Overview.html#dom-performance-now
    fn Now(&self) -> DOMHighResTimeStamp {
        let nav_start = self.timing.navigation_start_precise();
        let now = (time::precise_time_ns() as f64 - nav_start) / 1000000 as f64;
        Finite::wrap(now)
    }

    // https://www.w3.org/TR/performance-timeline-2/#dom-performance-getentries
    fn GetEntries(&self) -> Vec<Root<PerformanceEntry>> {
        self.entries.borrow().get_entries_by_name_and_type(None, None)
    }

    // https://www.w3.org/TR/performance-timeline-2/#dom-performance-getentriesbytype
    fn GetEntriesByType(&self, entry_type: DOMString) -> Vec<Root<PerformanceEntry>> {
        self.entries.borrow().get_entries_by_name_and_type(None, Some(entry_type))
    }

    // https://www.w3.org/TR/performance-timeline-2/#dom-performance-getentriesbyname
    fn GetEntriesByName(&self, name: DOMString, entry_type: Option<DOMString>)
        -> Vec<Root<PerformanceEntry>> {
        self.entries.borrow().get_entries_by_name_and_type(Some(name), entry_type)
    }
}

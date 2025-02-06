/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::Cell;
use std::cmp::Ordering;
use std::collections::VecDeque;

use base::cross_process_instant::CrossProcessInstant;
use dom_struct::dom_struct;
use time::Duration;

use super::bindings::refcounted::Trusted;
use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::PerformanceBinding::{
    DOMHighResTimeStamp, PerformanceEntryList as DOMPerformanceEntryList, PerformanceMethods,
};
use crate::dom::bindings::error::{Error, Fallible};
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::num::Finite;
use crate::dom::bindings::reflector::{reflect_dom_object, DomGlobal};
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::DOMString;
use crate::dom::eventtarget::EventTarget;
use crate::dom::globalscope::GlobalScope;
use crate::dom::performanceentry::PerformanceEntry;
use crate::dom::performancemark::PerformanceMark;
use crate::dom::performancemeasure::PerformanceMeasure;
use crate::dom::performancenavigation::PerformanceNavigation;
use crate::dom::performancenavigationtiming::PerformanceNavigationTiming;
use crate::dom::performanceobserver::PerformanceObserver as DOMPerformanceObserver;
use crate::dom::window::Window;
use crate::script_runtime::CanGc;

const INVALID_ENTRY_NAMES: &[&str] = &[
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
pub(crate) struct PerformanceEntryList {
    /// <https://w3c.github.io/performance-timeline/#dfn-performance-entry-buffer>
    entries: DOMPerformanceEntryList,
}

impl PerformanceEntryList {
    pub(crate) fn new(entries: DOMPerformanceEntryList) -> Self {
        PerformanceEntryList { entries }
    }

    pub(crate) fn get_entries_by_name_and_type(
        &self,
        name: Option<DOMString>,
        entry_type: Option<DOMString>,
    ) -> Vec<DomRoot<PerformanceEntry>> {
        let mut res = self
            .entries
            .iter()
            .filter(|e| {
                name.as_ref().map_or(true, |name_| *e.name() == *name_) &&
                    entry_type
                        .as_ref()
                        .map_or(true, |type_| *e.entry_type() == *type_)
            })
            .cloned()
            .collect::<Vec<DomRoot<PerformanceEntry>>>();
        res.sort_by(|a, b| {
            a.start_time()
                .partial_cmp(&b.start_time())
                .unwrap_or(Ordering::Equal)
        });
        res
    }

    pub(crate) fn clear_entries_by_name_and_type(
        &mut self,
        name: Option<DOMString>,
        entry_type: DOMString,
    ) {
        self.entries.retain(|e| {
            *e.entry_type() != *entry_type || name.as_ref().is_some_and(|name_| *e.name() != *name_)
        });
    }

    fn get_last_entry_start_time_with_name_and_type(
        &self,
        name: DOMString,
        entry_type: DOMString,
    ) -> Option<CrossProcessInstant> {
        self.entries
            .iter()
            .rev()
            .find(|e| *e.entry_type() == *entry_type && *e.name() == *name)
            .and_then(|entry| entry.start_time())
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
pub(crate) struct Performance {
    eventtarget: EventTarget,
    buffer: DomRefCell<PerformanceEntryList>,
    observers: DomRefCell<Vec<PerformanceObserver>>,
    pending_notification_observers_task: Cell<bool>,
    #[no_trace]
    /// The `timeOrigin` as described in
    /// <https://html.spec.whatwg.org/multipage/#concept-settings-object-time-origin>.
    time_origin: CrossProcessInstant,
    /// <https://w3c.github.io/performance-timeline/#dfn-maxbuffersize>
    /// The max-size of the buffer, set to 0 once the pipeline exits.
    /// TODO: have one max-size per entry type.
    resource_timing_buffer_size_limit: Cell<usize>,
    resource_timing_buffer_current_size: Cell<usize>,
    resource_timing_buffer_pending_full_event: Cell<bool>,
    resource_timing_secondary_entries: DomRefCell<VecDeque<DomRoot<PerformanceEntry>>>,
}

impl Performance {
    fn new_inherited(time_origin: CrossProcessInstant) -> Performance {
        Performance {
            eventtarget: EventTarget::new_inherited(),
            buffer: DomRefCell::new(PerformanceEntryList::new(Vec::new())),
            observers: DomRefCell::new(Vec::new()),
            pending_notification_observers_task: Cell::new(false),
            time_origin,
            resource_timing_buffer_size_limit: Cell::new(250),
            resource_timing_buffer_current_size: Cell::new(0),
            resource_timing_buffer_pending_full_event: Cell::new(false),
            resource_timing_secondary_entries: DomRefCell::new(VecDeque::new()),
        }
    }

    pub(crate) fn new(
        global: &GlobalScope,
        navigation_start: CrossProcessInstant,
    ) -> DomRoot<Performance> {
        reflect_dom_object(
            Box::new(Performance::new_inherited(navigation_start)),
            global,
            CanGc::note(),
        )
    }

    pub(crate) fn to_dom_high_res_time_stamp(
        &self,
        instant: CrossProcessInstant,
    ) -> DOMHighResTimeStamp {
        (instant - self.time_origin).to_dom_high_res_time_stamp()
    }

    pub(crate) fn maybe_to_dom_high_res_time_stamp(
        &self,
        instant: Option<CrossProcessInstant>,
    ) -> DOMHighResTimeStamp {
        self.to_dom_high_res_time_stamp(instant.unwrap_or(self.time_origin))
    }

    /// Clear all buffered performance entries, and disable the buffer.
    /// Called as part of the window's "clear_js_runtime" workflow,
    /// performed when exiting a pipeline.
    pub(crate) fn clear_and_disable_performance_entry_buffer(&self) {
        let mut buffer = self.buffer.borrow_mut();
        buffer.entries.clear();
        self.resource_timing_buffer_size_limit.set(0);
    }

    // Add a PerformanceObserver to the list of observers with a set of
    // observed entry types.

    pub(crate) fn add_multiple_type_observer(
        &self,
        observer: &DOMPerformanceObserver,
        entry_types: Vec<DOMString>,
    ) {
        let mut observers = self.observers.borrow_mut();
        match observers.iter().position(|o| *o.observer == *observer) {
            // If the observer is already in the list, we only update the observed
            // entry types.
            Some(p) => observers[p].entry_types = entry_types,
            // Otherwise, we create and insert the new PerformanceObserver.
            None => observers.push(PerformanceObserver {
                observer: DomRoot::from_ref(observer),
                entry_types,
            }),
        };
    }

    pub(crate) fn add_single_type_observer(
        &self,
        observer: &DOMPerformanceObserver,
        entry_type: &DOMString,
        buffered: bool,
    ) {
        if buffered {
            let buffer = self.buffer.borrow();
            let mut new_entries =
                buffer.get_entries_by_name_and_type(None, Some(entry_type.clone()));
            if !new_entries.is_empty() {
                let mut obs_entries = observer.entries();
                obs_entries.append(&mut new_entries);
                observer.set_entries(obs_entries);
            }

            if !self.pending_notification_observers_task.get() {
                self.pending_notification_observers_task.set(true);
                let global = &self.global();
                let owner = Trusted::new(&*global.performance());
                self.global()
                    .task_manager()
                    .performance_timeline_task_source()
                    .queue(task!(notify_performance_observers: move || {
                        owner.root().notify_observers();
                    }));
            }
        }
        let mut observers = self.observers.borrow_mut();
        match observers.iter().position(|o| *o.observer == *observer) {
            // If the observer is already in the list, we only update
            // the observed entry types.
            Some(p) => {
                // Append the type if not already present, otherwise do nothing
                if !observers[p].entry_types.contains(entry_type) {
                    observers[p].entry_types.push(entry_type.clone())
                }
            },
            // Otherwise, we create and insert the new PerformanceObserver.
            None => observers.push(PerformanceObserver {
                observer: DomRoot::from_ref(observer),
                entry_types: vec![entry_type.clone()],
            }),
        };
    }

    /// Remove a PerformanceObserver from the list of observers.
    pub(crate) fn remove_observer(&self, observer: &DOMPerformanceObserver) {
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
    /// Also this algorithm has been extented according to :
    /// <https://w3c.github.io/resource-timing/#sec-extensions-performance-interface>
    pub(crate) fn queue_entry(&self, entry: &PerformanceEntry, can_gc: CanGc) -> Option<usize> {
        // https://w3c.github.io/performance-timeline/#dfn-determine-eligibility-for-adding-a-performance-entry
        if entry.entry_type() == "resource" && !self.should_queue_resource_entry(entry, can_gc) {
            return None;
        }

        // Steps 1-3.
        // Add the performance entry to the list of performance entries that have not
        // been notified to each performance observer owner, filtering the ones it's
        // interested in.
        for o in self
            .observers
            .borrow()
            .iter()
            .filter(|o| o.entry_types.contains(entry.entry_type()))
        {
            o.observer.queue_entry(entry);
        }

        // Step 4.
        //add the new entry to the buffer.
        self.buffer
            .borrow_mut()
            .entries
            .push(DomRoot::from_ref(entry));

        let entry_last_index = self.buffer.borrow_mut().entries.len() - 1;

        // Step 5.
        // If there is already a queued notification task, we just bail out.
        if self.pending_notification_observers_task.get() {
            return None;
        }

        // Step 6.
        // Queue a new notification task.
        self.pending_notification_observers_task.set(true);

        let global = &self.global();
        let owner = Trusted::new(&*global.performance());
        self.global()
            .task_manager()
            .performance_timeline_task_source()
            .queue(task!(notify_performance_observers: move || {
                owner.root().notify_observers();
            }));

        Some(entry_last_index)
    }

    /// Observers notifications task.
    ///
    /// Algorithm spec (step 7):
    /// <https://w3c.github.io/performance-timeline/#queue-a-performanceentry>
    pub(crate) fn notify_observers(&self) {
        // Step 7.1.
        self.pending_notification_observers_task.set(false);

        // Step 7.2.
        // We have to operate over a copy of the performance observers to avoid
        // the risk of an observer's callback modifying the list of registered
        // observers. This is a shallow copy, so observers can
        // disconnect themselves by using the argument of their own callback.
        let observers: Vec<DomRoot<DOMPerformanceObserver>> = self
            .observers
            .borrow()
            .iter()
            .map(|o| DomRoot::from_ref(&*o.observer))
            .collect();

        // Step 7.3.
        for o in observers.iter() {
            o.notify();
        }
    }

    fn can_add_resource_timing_entry(&self) -> bool {
        self.resource_timing_buffer_current_size.get() <=
            self.resource_timing_buffer_size_limit.get()
    }
    fn copy_secondary_resource_timing_buffer(&self, can_gc: CanGc) {
        while self.can_add_resource_timing_entry() {
            let entry = self
                .resource_timing_secondary_entries
                .borrow_mut()
                .pop_front();
            if let Some(ref entry) = entry {
                self.queue_entry(entry, can_gc);
            } else {
                break;
            }
        }
    }
    // `fire a buffer full event` paragraph of
    // https://w3c.github.io/resource-timing/#sec-extensions-performance-interface
    fn fire_buffer_full_event(&self, can_gc: CanGc) {
        while !self.resource_timing_secondary_entries.borrow().is_empty() {
            let no_of_excess_entries_before = self.resource_timing_secondary_entries.borrow().len();

            if !self.can_add_resource_timing_entry() {
                self.upcast::<EventTarget>()
                    .fire_event(atom!("resourcetimingbufferfull"), can_gc);
            }
            self.copy_secondary_resource_timing_buffer(can_gc);
            let no_of_excess_entries_after = self.resource_timing_secondary_entries.borrow().len();
            if no_of_excess_entries_before <= no_of_excess_entries_after {
                self.resource_timing_secondary_entries.borrow_mut().clear();
                break;
            }
        }
        self.resource_timing_buffer_pending_full_event.set(false);
    }
    /// `add a PerformanceResourceTiming entry` paragraph of
    /// <https://w3c.github.io/resource-timing/#sec-extensions-performance-interface>
    fn should_queue_resource_entry(&self, entry: &PerformanceEntry, can_gc: CanGc) -> bool {
        // Step 1 is done in the args list.
        if !self.resource_timing_buffer_pending_full_event.get() {
            // Step 2.
            if self.can_add_resource_timing_entry() {
                // Step 2.a is done in `queue_entry`
                // Step 2.b.
                self.resource_timing_buffer_current_size
                    .set(self.resource_timing_buffer_current_size.get() + 1);
                // Step 2.c.
                return true;
            }
            // Step 3.
            self.resource_timing_buffer_pending_full_event.set(true);
            self.fire_buffer_full_event(can_gc);
        }
        // Steps 4 and 5.
        self.resource_timing_secondary_entries
            .borrow_mut()
            .push_back(DomRoot::from_ref(entry));
        false
    }

    pub(crate) fn update_entry(&self, index: usize, entry: &PerformanceEntry) {
        if let Some(e) = self.buffer.borrow_mut().entries.get_mut(index) {
            *e = DomRoot::from_ref(entry);
        }
    }
}

impl PerformanceMethods<crate::DomTypeHolder> for Performance {
    // FIXME(avada): this should be deprecated in the future, but some sites still use it
    // https://dvcs.w3.org/hg/webperf/raw-file/tip/specs/NavigationTiming/Overview.html#performance-timing-attribute
    fn Timing(&self) -> DomRoot<PerformanceNavigationTiming> {
        let entries = self.GetEntriesByType(DOMString::from("navigation"));
        if !entries.is_empty() {
            return DomRoot::from_ref(
                entries[0]
                    .downcast::<PerformanceNavigationTiming>()
                    .unwrap(),
            );
        }
        unreachable!("Are we trying to expose Performance.timing in workers?");
    }

    // https://w3c.github.io/navigation-timing/#dom-performance-navigation
    fn Navigation(&self) -> DomRoot<PerformanceNavigation> {
        PerformanceNavigation::new(&self.global())
    }

    // https://dvcs.w3.org/hg/webperf/raw-file/tip/specs/HighResolutionTime/Overview.html#dom-performance-now
    fn Now(&self) -> DOMHighResTimeStamp {
        self.to_dom_high_res_time_stamp(CrossProcessInstant::now())
    }

    // https://www.w3.org/TR/hr-time-2/#dom-performance-timeorigin
    fn TimeOrigin(&self) -> DOMHighResTimeStamp {
        (self.time_origin - CrossProcessInstant::epoch()).to_dom_high_res_time_stamp()
    }

    // https://www.w3.org/TR/performance-timeline-2/#dom-performance-getentries
    fn GetEntries(&self) -> Vec<DomRoot<PerformanceEntry>> {
        self.buffer
            .borrow()
            .get_entries_by_name_and_type(None, None)
    }

    // https://www.w3.org/TR/performance-timeline-2/#dom-performance-getentriesbytype
    fn GetEntriesByType(&self, entry_type: DOMString) -> Vec<DomRoot<PerformanceEntry>> {
        self.buffer
            .borrow()
            .get_entries_by_name_and_type(None, Some(entry_type))
    }

    // https://www.w3.org/TR/performance-timeline-2/#dom-performance-getentriesbyname
    fn GetEntriesByName(
        &self,
        name: DOMString,
        entry_type: Option<DOMString>,
    ) -> Vec<DomRoot<PerformanceEntry>> {
        self.buffer
            .borrow()
            .get_entries_by_name_and_type(Some(name), entry_type)
    }

    // https://w3c.github.io/user-timing/#dom-performance-mark
    fn Mark(&self, mark_name: DOMString, can_gc: CanGc) -> Fallible<()> {
        let global = self.global();
        // Step 1.
        if global.is::<Window>() && INVALID_ENTRY_NAMES.contains(&mark_name.as_ref()) {
            return Err(Error::Syntax);
        }

        // Steps 2 to 6.
        let entry = PerformanceMark::new(
            &global,
            mark_name,
            CrossProcessInstant::now(),
            Duration::ZERO,
        );
        // Steps 7 and 8.
        self.queue_entry(entry.upcast::<PerformanceEntry>(), can_gc);

        // Step 9.
        Ok(())
    }

    // https://w3c.github.io/user-timing/#dom-performance-clearmarks
    fn ClearMarks(&self, mark_name: Option<DOMString>) {
        self.buffer
            .borrow_mut()
            .clear_entries_by_name_and_type(mark_name, DOMString::from("mark"));
    }

    // https://w3c.github.io/user-timing/#dom-performance-measure
    fn Measure(
        &self,
        measure_name: DOMString,
        start_mark: Option<DOMString>,
        end_mark: Option<DOMString>,
        can_gc: CanGc,
    ) -> Fallible<()> {
        // Steps 1 and 2.
        let end_time = end_mark
            .map(|name| {
                self.buffer
                    .borrow()
                    .get_last_entry_start_time_with_name_and_type(DOMString::from("mark"), name)
                    .unwrap_or(self.time_origin)
            })
            .unwrap_or_else(CrossProcessInstant::now);

        // Step 3.
        let start_time = start_mark
            .and_then(|name| {
                self.buffer
                    .borrow()
                    .get_last_entry_start_time_with_name_and_type(DOMString::from("mark"), name)
            })
            .unwrap_or(self.time_origin);

        // Steps 4 to 8.
        let entry = PerformanceMeasure::new(
            &self.global(),
            measure_name,
            start_time,
            end_time - start_time,
        );

        // Step 9 and 10.
        self.queue_entry(entry.upcast::<PerformanceEntry>(), can_gc);

        // Step 11.
        Ok(())
    }

    // https://w3c.github.io/user-timing/#dom-performance-clearmeasures
    fn ClearMeasures(&self, measure_name: Option<DOMString>) {
        self.buffer
            .borrow_mut()
            .clear_entries_by_name_and_type(measure_name, DOMString::from("measure"));
    }
    // https://w3c.github.io/resource-timing/#dom-performance-clearresourcetimings
    fn ClearResourceTimings(&self) {
        self.buffer
            .borrow_mut()
            .clear_entries_by_name_and_type(None, DOMString::from("resource"));
        self.resource_timing_buffer_current_size.set(0);
    }

    // https://w3c.github.io/resource-timing/#dom-performance-setresourcetimingbuffersize
    fn SetResourceTimingBufferSize(&self, max_size: u32) {
        self.resource_timing_buffer_size_limit
            .set(max_size as usize);
    }

    // https://w3c.github.io/resource-timing/#dom-performance-onresourcetimingbufferfull
    event_handler!(
        resourcetimingbufferfull,
        GetOnresourcetimingbufferfull,
        SetOnresourcetimingbufferfull
    );
}

pub(crate) trait ToDOMHighResTimeStamp {
    fn to_dom_high_res_time_stamp(&self) -> DOMHighResTimeStamp;
}

impl ToDOMHighResTimeStamp for Duration {
    fn to_dom_high_res_time_stamp(&self) -> DOMHighResTimeStamp {
        // https://www.w3.org/TR/hr-time-2/#clock-resolution
        // We need a granularity no finer than 5 microseconds. 5 microseconds isn't an
        // exactly representable f64 so WPT tests might occasionally corner-case on
        // rounding.  web-platform-tests/wpt#21526 wants us to use an integer number of
        // microseconds; the next divisor of milliseconds up from 5 microseconds is 10.
        let microseconds_rounded = (self.whole_microseconds() as f64 / 10.).floor() * 10.;
        Finite::wrap(microseconds_rounded / 1000.)
    }
}

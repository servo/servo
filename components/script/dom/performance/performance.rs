/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::Cell;
use std::cmp::Ordering;
use std::collections::VecDeque;

use dom_struct::dom_struct;
use script_bindings::cformat;
use script_bindings::codegen::GenericBindings::WindowBinding::WindowMethods;
use script_bindings::codegen::GenericUnionTypes::StringOrPerformanceMeasureOptions;
use servo_base::cross_process_instant::CrossProcessInstant;
use time::Duration;

use super::performanceentry::{EntryType, PerformanceEntry};
use super::performancemark::PerformanceMark;
use super::performancemeasure::PerformanceMeasure;
use super::performancenavigation::PerformanceNavigation;
use super::performancenavigationtiming::PerformanceNavigationTiming;
use super::performanceobserver::PerformanceObserver as DOMPerformanceObserver;
use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::PerformanceBinding::{
    DOMHighResTimeStamp, PerformanceEntryList as DOMPerformanceEntryList, PerformanceMethods,
};
use crate::dom::bindings::codegen::UnionTypes::StringOrDouble;
use crate::dom::bindings::error::{Error, Fallible};
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::num::Finite;
use crate::dom::bindings::refcounted::Trusted;
use crate::dom::bindings::reflector::{DomGlobal, reflect_dom_object};
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::DOMString;
use crate::dom::eventtarget::EventTarget;
use crate::dom::globalscope::GlobalScope;
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

    /// <https://www.w3.org/TR/performance-timeline/#dfn-filter-buffer-map-by-name-and-type>
    pub(crate) fn get_entries_by_name_and_type(
        &self,
        name: Option<DOMString>,
        entry_type: Option<EntryType>,
    ) -> Vec<DomRoot<PerformanceEntry>> {
        let mut result = self
            .entries
            .iter()
            .filter(|e| {
                name.as_ref().is_none_or(|name_| *e.name() == *name_) &&
                    entry_type
                        .as_ref()
                        .is_none_or(|type_| e.entry_type() == *type_)
            })
            .cloned()
            .collect::<Vec<DomRoot<PerformanceEntry>>>();

        // Step 6. Sort results's entries in chronological order with respect to startTime
        result.sort_by(|a, b| {
            a.start_time()
                .partial_cmp(&b.start_time())
                .unwrap_or(Ordering::Equal)
        });

        // Step 7. Return result.
        result
    }

    pub(crate) fn clear_entries_by_name_and_type(
        &mut self,
        name: Option<DOMString>,
        entry_type: EntryType,
    ) {
        self.entries.retain(|e| {
            e.entry_type() != entry_type || name.as_ref().is_some_and(|name_| e.name() != name_)
        });
    }

    fn get_last_entry_start_time_with_name_and_type(
        &self,
        name: DOMString,
        entry_type: EntryType,
    ) -> Option<CrossProcessInstant> {
        self.entries
            .iter()
            .rev()
            .find(|e| e.entry_type() == entry_type && *e.name() == name)
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
    entry_types: Vec<EntryType>,
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
    /// <https://w3c.github.io/resource-timing/#performance-resource-timing-buffer-size-limit>
    /// The max-size of the buffer, set to 0 once the pipeline exits.
    /// TODO: have one max-size per entry type.
    resource_timing_buffer_size_limit: Cell<usize>,
    /// <https://w3c.github.io/resource-timing/#performance-resource-timing-buffer-current-size>
    resource_timing_buffer_current_size: Cell<usize>,
    /// <https://w3c.github.io/resource-timing/#performance-resource-timing-buffer-full-event-pending-flag>
    resource_timing_buffer_pending_full_event: Cell<bool>,
    /// <https://w3c.github.io/resource-timing/#performance-resource-timing-secondary-buffer>
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
        can_gc: CanGc,
    ) -> DomRoot<Performance> {
        reflect_dom_object(
            Box::new(Performance::new_inherited(navigation_start)),
            global,
            can_gc,
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
        entry_types: Vec<EntryType>,
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
        entry_type: EntryType,
        buffered: bool,
    ) {
        if buffered {
            let buffer = self.buffer.borrow();
            let mut new_entries = buffer.get_entries_by_name_and_type(None, Some(entry_type));
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
                if !observers[p].entry_types.contains(&entry_type) {
                    observers[p].entry_types.push(entry_type)
                }
            },
            // Otherwise, we create and insert the new PerformanceObserver.
            None => observers.push(PerformanceObserver {
                observer: DomRoot::from_ref(observer),
                entry_types: vec![entry_type],
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
    pub(crate) fn queue_entry(&self, entry: &PerformanceEntry) -> Option<usize> {
        // https://w3c.github.io/performance-timeline/#dfn-determine-eligibility-for-adding-a-performance-entry
        if entry.entry_type() == EntryType::Resource && !self.should_queue_resource_entry(entry) {
            return None;
        }

        // Steps 1-3.
        // Add the performance entry to the list of performance entries that have not
        // been notified to each performance observer owner, filtering the ones it's
        // interested in.
        for observer in self
            .observers
            .borrow()
            .iter()
            .filter(|o| o.entry_types.contains(&entry.entry_type()))
        {
            observer.observer.queue_entry(entry);
        }

        // Step 4.
        // add the new entry to the buffer.
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
            o.notify(CanGc::deprecated_note());
        }
    }

    /// <https://w3c.github.io/resource-timing/#performance-can-add-resource-timing-entry>
    fn can_add_resource_timing_entry(&self) -> bool {
        // Step 1. If resource timing buffer current size is smaller than resource timing buffer size limit, return true.
        // Step 2. Return false.
        // TODO: Changing this to "<" (as per spec) does not result in passing tests, needs investigation
        self.resource_timing_buffer_current_size.get() <=
            self.resource_timing_buffer_size_limit.get()
    }

    /// <https://w3c.github.io/resource-timing/#dfn-copy-secondary-buffer>
    fn copy_secondary_resource_timing_buffer(&self) {
        // Step 1. While resource timing secondary buffer is not empty and can add resource timing entry returns true, run the following substeps:
        while self.can_add_resource_timing_entry() {
            // Step 1.1. Let entry be the oldest PerformanceResourceTiming in resource timing secondary buffer.
            let entry = self
                .resource_timing_secondary_entries
                .borrow_mut()
                .pop_front();
            if let Some(ref entry) = entry {
                // Step 1.2. Add entry to the end of performance entry buffer.
                self.buffer
                    .borrow_mut()
                    .entries
                    .push(DomRoot::from_ref(entry));
                // Step 1.3. Increment resource timing buffer current size by 1.
                self.resource_timing_buffer_current_size
                    .set(self.resource_timing_buffer_current_size.get() + 1);
                // Step 1.4. Remove entry from resource timing secondary buffer.
                // Step 1.5. Decrement resource timing secondary buffer current size by 1.
                // Handled by popping the entry earlier.
            } else {
                break;
            }
        }
    }
    // `fire a buffer full event` paragraph of
    /// <https://w3c.github.io/resource-timing/#sec-extensions-performance-interface>
    fn fire_buffer_full_event(&self, can_gc: CanGc) {
        while !self.resource_timing_secondary_entries.borrow().is_empty() {
            let no_of_excess_entries_before = self.resource_timing_secondary_entries.borrow().len();

            if !self.can_add_resource_timing_entry() {
                self.upcast::<EventTarget>()
                    .fire_event(atom!("resourcetimingbufferfull"), can_gc);
            }
            self.copy_secondary_resource_timing_buffer();
            let no_of_excess_entries_after = self.resource_timing_secondary_entries.borrow().len();
            if no_of_excess_entries_before <= no_of_excess_entries_after {
                self.resource_timing_secondary_entries.borrow_mut().clear();
                break;
            }
        }
        self.resource_timing_buffer_pending_full_event.set(false);
    }

    /// <https://w3c.github.io/resource-timing/#dfn-add-a-performanceresourcetiming-entry>
    fn should_queue_resource_entry(&self, entry: &PerformanceEntry) -> bool {
        // Step 1. If can add resource timing entry returns true and resource timing buffer full event pending flag is false, run the following substeps:
        if !self.resource_timing_buffer_pending_full_event.get() {
            if self.can_add_resource_timing_entry() {
                // Step 1.a.  Add new entry to the performance entry buffer.
                //   This is done in queue_entry, which calls this method.
                // Step 1.b. Increase resource timing buffer current size by 1.
                self.resource_timing_buffer_current_size
                    .set(self.resource_timing_buffer_current_size.get() + 1);
                // Step 1.c. Return.
                return true;
            }

            // Step 2.a. Set resource timing buffer full event pending flag to true.
            self.resource_timing_buffer_pending_full_event.set(true);
            // Step 2.b. Queue a task on the performance timeline task source to run fire a buffer full event.
            let performance = Trusted::new(self);
            self.global()
                .task_manager()
                .performance_timeline_task_source()
                .queue(task!(fire_a_buffer_full_event: move || {
                    performance.root().fire_buffer_full_event(CanGc::deprecated_note());
                }));
        }

        // Step 3. Add new entry to the resource timing secondary buffer.
        self.resource_timing_secondary_entries
            .borrow_mut()
            .push_back(DomRoot::from_ref(entry));

        // Step 4. Increase resource timing secondary buffer current size by 1.
        //   This is tracked automatically via `.len()`.
        false
    }

    pub(crate) fn update_entry(&self, index: usize, entry: &PerformanceEntry) {
        if let Some(e) = self.buffer.borrow_mut().entries.get_mut(index) {
            *e = DomRoot::from_ref(entry);
        }
    }

    /// <https://w3c.github.io/user-timing/#convert-a-name-to-a-timestamp>
    fn convert_a_name_to_a_timestamp(&self, name: &str) -> Fallible<CrossProcessInstant> {
        // Step 1. If the global object is not a Window object, throw a TypeError.
        let Some(window) = DomRoot::downcast::<Window>(self.global()) else {
            return Err(Error::Type(cformat!(
                "Cannot use {name} from non-window global"
            )));
        };

        // Step 2. If name is navigationStart, return 0.
        if name == "navigationStart" {
            return Ok(self.time_origin);
        }

        // Step 3. Let startTime be the value of navigationStart in the PerformanceTiming interface.
        // FIXME: We don't implement this value yet, so we assume it's zero (and then we don't need it at all)

        // Step 4. Let endTime be the value of name in the PerformanceTiming interface.
        // NOTE: We store all performance values on the document
        let document = window.Document();
        let end_time = match name {
            "unloadEventStart" => document.get_unload_event_start(),
            "unloadEventEnd" => document.get_unload_event_end(),
            "domInteractive" => document.get_dom_interactive(),
            "domContentLoadedEventStart" => document.get_dom_content_loaded_event_start(),
            "domContentLoadedEventEnd" => document.get_dom_content_loaded_event_end(),
            "domComplete" => document.get_dom_complete(),
            "loadEventStart" => document.get_load_event_start(),
            "loadEventEnd" => document.get_load_event_end(),
            other => {
                if cfg!(debug_assertions) {
                    unreachable!("{other:?} is not the name of a timestamp");
                }
                return Err(Error::Operation(None));
            },
        };
        // Step 5. If endTime is 0, throw an InvalidAccessError.
        let Some(end_time) = end_time else {
            return Err(Error::InvalidAccess(Some(format!(
                "{name} hasn't happened yet"
            ))));
        };

        // Step 6. Return result of subtracting startTime from endTime.
        Ok(end_time)
    }

    /// <https://w3c.github.io/user-timing/#convert-a-mark-to-a-timestamp>
    fn convert_a_mark_to_a_timestamp(
        &self,
        mark: &StringOrDouble,
    ) -> Fallible<CrossProcessInstant> {
        match mark {
            StringOrDouble::String(name) => {
                // Step 1. If mark is a DOMString and it has the same name as a read only attribute in the
                // PerformanceTiming interface, let end time be the value returned by running the convert
                // a name to a timestamp algorithm with name set to the value of mark.
                // TODO: These aren't all fields because servo doesn't support some of them yet
                if matches!(
                    &*name.str(),
                    "navigationStart" |
                        "unloadEventStart" |
                        "unloadEventEnd" |
                        "domInteractive" |
                        "domContentLoadedEventStart" |
                        "domContentLoadedEventEnd" |
                        "domComplete" |
                        "loadEventStart" |
                        "loadEventEnd"
                ) {
                    self.convert_a_name_to_a_timestamp(&name.str())
                }
                // Step 2. Otherwise, if mark is a DOMString, let end time be the value of the startTime
                // attribute from the most recent occurrence of a PerformanceMark object in the performance entry
                // buffer whose name is mark. If no matching entry is found, throw a SyntaxError.
                else {
                    self.buffer
                        .borrow()
                        .get_last_entry_start_time_with_name_and_type(name.clone(), EntryType::Mark)
                        .ok_or(Error::Syntax(Some(format!(
                            "No PerformanceMark named {name} exists"
                        ))))
                }
            },
            // Step 3. Otherwise, if mark is a DOMHighResTimeStamp:
            StringOrDouble::Double(timestamp) => {
                // Step 3.1 If mark is negative, throw a TypeError.
                if timestamp.is_sign_negative() {
                    return Err(Error::Type(c"Time stamps must not be negative".to_owned()));
                }

                // Step 3.2 Otherwise, let end time be mark.
                // NOTE: I think the spec wants us to return the value.
                Ok(self.time_origin + Duration::milliseconds(timestamp.round() as i64))
            },
        }
    }
}

impl PerformanceMethods<crate::DomTypeHolder> for Performance {
    /// <https://w3c.github.io/navigation-timing/#dom-performance-timing>
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

    /// <https://w3c.github.io/navigation-timing/#dom-performance-navigation>
    fn Navigation(&self) -> DomRoot<PerformanceNavigation> {
        PerformanceNavigation::new(&self.global(), CanGc::deprecated_note())
    }

    /// <https://w3c.github.io/hr-time/#dom-performance-now>
    fn Now(&self) -> DOMHighResTimeStamp {
        self.to_dom_high_res_time_stamp(CrossProcessInstant::now())
    }

    /// <https://www.w3.org/TR/hr-time-2/#dom-performance-timeorigin>
    fn TimeOrigin(&self) -> DOMHighResTimeStamp {
        (self.time_origin - CrossProcessInstant::epoch()).to_dom_high_res_time_stamp()
    }

    /// <https://www.w3.org/TR/performance-timeline-2/#dom-performance-getentries>
    fn GetEntries(&self) -> Vec<DomRoot<PerformanceEntry>> {
        // > Returns a PerformanceEntryList object returned by the filter buffer map by name and type
        // > algorithm with name and type set to null.
        self.buffer
            .borrow()
            .get_entries_by_name_and_type(None, None)
    }

    /// <https://www.w3.org/TR/performance-timeline-2/#dom-performance-getentriesbytype>
    fn GetEntriesByType(&self, entry_type: DOMString) -> Vec<DomRoot<PerformanceEntry>> {
        let Ok(entry_type) = EntryType::try_from(&*entry_type.str()) else {
            return Vec::new();
        };
        self.buffer
            .borrow()
            .get_entries_by_name_and_type(None, Some(entry_type))
    }

    /// <https://www.w3.org/TR/performance-timeline-2/#dom-performance-getentriesbyname>
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
        self.buffer
            .borrow()
            .get_entries_by_name_and_type(Some(name), entry_type)
    }

    /// <https://w3c.github.io/user -timing/#dom-performance-mark>
    fn Mark(&self, mark_name: DOMString) -> Fallible<DomRoot<PerformanceMark>> {
        let global = self.global();
        // NOTE: This should happen within the performancemark constructor
        if global.is::<Window>() && INVALID_ENTRY_NAMES.contains(&&*mark_name.str()) {
            return Err(Error::Syntax(None));
        }

        // Step 1. Run the PerformanceMark constructor and let entry be the newly created object.
        let entry = PerformanceMark::new(
            &global,
            mark_name,
            CrossProcessInstant::now(),
            Duration::ZERO,
        );

        // Step 2. Queue a PerformanceEntry entry.
        self.queue_entry(entry.upcast::<PerformanceEntry>());

        // TODO Step 3. Add entry to the performance entry buffer.

        // Step 4. Return entry.
        Ok(entry)
    }

    /// <https://w3c.github.io/user-timing/#dom-performance-clearmarks>
    fn ClearMarks(&self, mark_name: Option<DOMString>) {
        self.buffer
            .borrow_mut()
            .clear_entries_by_name_and_type(mark_name, EntryType::Mark);
    }

    /// <https://w3c.github.io/user-timing/#dom-performance-measure>
    fn Measure(
        &self,
        measure_name: DOMString,
        start_or_measure_options: StringOrPerformanceMeasureOptions,
        end_mark: Option<DOMString>,
    ) -> Fallible<DomRoot<PerformanceMeasure>> {
        // Step 1. If startOrMeasureOptions is a PerformanceMeasureOptions object and at least one of start,
        // end, duration, and detail exist, run the following checks:
        if let StringOrPerformanceMeasureOptions::PerformanceMeasureOptions(options) =
            &start_or_measure_options
        {
            if options.start.is_some() || options.duration.is_some() || options.end.is_some() {
                // Step 1.1 If endMark is given, throw a TypeError.
                if end_mark.is_some() {
                    return Err(Error::Type(
                        c"Must not provide endMark if PerformanceMeasureOptions is also provided"
                            .to_owned(),
                    ));
                }

                // Step 1.2 If startOrMeasureOptions’s start and end members are both omitted, throw a TypeError.
                if options.start.is_none() && options.end.is_none() {
                    return Err(Error::Type(c"Either 'start' or 'end' member of PerformanceMeasureOptions must be provided".to_owned()));
                }

                // Step 1.3 If startOrMeasureOptions’s start, duration, and end members all exist, throw a TypeError.
                if options.start.is_some() && options.duration.is_some() && options.end.is_some() {
                    return Err(Error::Type(c"Either 'start' or 'end' or 'duration' member of PerformanceMeasureOptions must be omitted".to_owned()));
                }
            }
        }

        // Step 2. Compute end time as follows:
        // Step 2.1 If endMark is given, let end time be the value returned
        // by running the convert a mark to a timestamp algorithm passing in endMark.
        let end_time = if let Some(end_mark) = end_mark {
            self.convert_a_mark_to_a_timestamp(&StringOrDouble::String(end_mark))?
        } else {
            match &start_or_measure_options {
                StringOrPerformanceMeasureOptions::PerformanceMeasureOptions(options) => {
                    // Step 2.2 Otherwise, if startOrMeasureOptions is a PerformanceMeasureOptions object,
                    // and if its end member exists, let end time be the value returned by running the
                    // convert a mark to a timestamp algorithm passing in startOrMeasureOptions’s end.
                    if let Some(end) = &options.end {
                        self.convert_a_mark_to_a_timestamp(end)?
                    }
                    // Step 2.3 Otherwise, if startOrMeasureOptions is a PerformanceMeasureOptions object,
                    // and if its start and duration members both exist:
                    else if let Some((start, duration)) =
                        options.start.as_ref().zip(options.duration)
                    {
                        // Step 2.3.1 Let start be the value returned by running the convert a mark to a timestamp
                        // algorithm passing in start.
                        let start = self.convert_a_mark_to_a_timestamp(start)?;

                        // Step 2.3.2 Let duration be the value returned by running the convert a mark to a timestamp
                        // algorithm passing in duration.
                        let duration = self
                            .convert_a_mark_to_a_timestamp(&StringOrDouble::Double(duration))? -
                            self.time_origin;

                        // Step 2.3.3 Let end time be start plus duration.
                        start + duration
                    } else {
                        // Step 2.4 Otherwise, let end time be the value that would be returned by the
                        // Performance object’s now() method.
                        CrossProcessInstant::now()
                    }
                },
                _ => {
                    // Step 2.4 Otherwise, let end time be the value that would be returned by the
                    // Performance object’s now() method.
                    CrossProcessInstant::now()
                },
            }
        };

        // Step 3. Compute start time as follows:
        let start_time = match start_or_measure_options {
            StringOrPerformanceMeasureOptions::PerformanceMeasureOptions(options) => {
                // Step 3.1 If startOrMeasureOptions is a PerformanceMeasureOptions object, and if its start member exists,
                // let start time be the value returned by running the convert a mark to a timestamp algorithm passing in
                // startOrMeasureOptions’s start.
                if let Some(start) = &options.start {
                    self.convert_a_mark_to_a_timestamp(start)?
                }
                // Step 3.2 Otherwise, if startOrMeasureOptions is a PerformanceMeasureOptions object,
                // and if its duration and end members both exist:
                else if let Some((duration, end)) = options.duration.zip(options.end.as_ref()) {
                    // Step 3.2.1 Let duration be the value returned by running the convert a mark to a timestamp
                    // algorithm passing in duration.
                    let duration = self
                        .convert_a_mark_to_a_timestamp(&StringOrDouble::Double(duration))? -
                        self.time_origin;

                    // Step 3.2.2 Let end be the value returned by running the convert a mark to a timestamp algorithm
                    // passing in end.
                    let end = self.convert_a_mark_to_a_timestamp(end)?;

                    // Step 3.3.3 Let start time be end minus duration.
                    end - duration
                }
                // Step 3.4 Otherwise, let start time be 0.
                else {
                    self.time_origin
                }
            },
            StringOrPerformanceMeasureOptions::String(string) => {
                // Step 3.3 Otherwise, if startOrMeasureOptions is a DOMString, let start time be the value returned
                // by running the convert a mark to a timestamp algorithm passing in startOrMeasureOptions.
                self.convert_a_mark_to_a_timestamp(&StringOrDouble::String(string))?
            },
        };

        // Step 4. Create a new PerformanceMeasure object (entry) with this’s relevant realm.
        // Step 5. Set entry’s name attribute to measureName.
        // Step 6. Set entry’s entryType attribute to DOMString "measure".
        // Step 7. Set entry’s startTime attribute to start time.
        // Step 8. Set entry’s duration attribute to the duration from start time to end time.
        // The resulting duration value MAY be negative.
        // TODO: Step 9. Set entry’s detail attribute as follows:
        let entry = PerformanceMeasure::new(
            &self.global(),
            measure_name,
            start_time,
            end_time - start_time,
        );

        // Step 10. Queue a PerformanceEntry entry.
        // Step 11. Add entry to the performance entry buffer.
        self.queue_entry(entry.upcast::<PerformanceEntry>());

        // Step 12. Return entry.
        Ok(entry)
    }

    /// <https://w3c.github.io/user-timing/#dom-performance-clearmeasures>
    fn ClearMeasures(&self, measure_name: Option<DOMString>) {
        self.buffer
            .borrow_mut()
            .clear_entries_by_name_and_type(measure_name, EntryType::Measure);
    }
    /// <https://w3c.github.io/resource-timing/#dom-performance-clearresourcetimings>
    fn ClearResourceTimings(&self) {
        self.buffer
            .borrow_mut()
            .clear_entries_by_name_and_type(None, EntryType::Resource);
        self.resource_timing_buffer_current_size.set(0);
    }

    /// <https://w3c.github.io/resource-timing/#performance-setresourcetimingbuffersize>
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

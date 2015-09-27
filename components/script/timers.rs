/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::callback::ExceptionHandling::Report;
use dom::bindings::cell::DOMRefCell;
use dom::bindings::codegen::Bindings::FunctionBinding::Function;
use dom::bindings::global::global_object_for_js_object;
use dom::bindings::utils::Reflectable;
use dom::window::ScriptHelpers;
use euclid::length::Length;
use script_traits::{MsDuration, precise_time_ms};
use script_traits::{TimerEventChan, TimerEventId, TimerEventRequest, TimerSource};

use util::mem::HeapSizeOf;
use util::str::DOMString;

use js::jsapi::{RootedValue, HandleValue, Heap};
use js::jsval::{JSVal, UndefinedValue};

use num::traits::Saturating;
use std::cell::Cell;
use std::cmp::{self, Ord, Ordering};
use std::collections::BinaryHeap;
use std::default::Default;
use std::rc::Rc;
use std::sync::mpsc::Sender;

#[derive(JSTraceable, PartialEq, Eq, Copy, Clone, HeapSizeOf, Hash, PartialOrd, Ord)]
pub struct TimerHandle(i32);

#[derive(JSTraceable, HeapSizeOf)]
#[privatize]
pub struct ActiveTimers {
    #[ignore_heap_size_of = "Defined in std"]
    timer_event_chan: Box<TimerEventChan + Send>,
    #[ignore_heap_size_of = "Defined in std"]
    scheduler_chan: Sender<TimerEventRequest>,
    next_timer_handle: Cell<TimerHandle>,
    timers: DOMRefCell<BinaryHeap<Timer>>,
    suspended_since: Cell<Option<MsDuration>>,
    /// Initially 0, increased whenever the associated document is reactivated
    /// by the amount of ms the document was inactive. The current time can be
    /// offset back by this amount for a coherent time across document
    /// activations.
    suspension_offset: Cell<MsDuration>,
    /// Calls to `fire_timer` with a different argument than this get ignored.
    /// They were previously scheduled and got invalidated when
    ///  - timers were suspended,
    ///  - the timer it was scheduled for got canceled or
    ///  - a timer was added with an earlier callback time. In this case the
    ///    original timer is rescheduled when it is the next one to get called.
    expected_event_id: Cell<TimerEventId>,
}

// Holder for the various JS values associated with setTimeout
// (ie. function value to invoke and all arguments to pass
//      to the function when calling it)
// TODO: Handle rooting during fire_timer when movable GC is turned on
#[derive(JSTraceable, HeapSizeOf)]
#[privatize]
struct Timer {
    handle: TimerHandle,
    source: TimerSource,
    callback: TimerCallback,
    arguments: Vec<Heap<JSVal>>,
    is_interval: IsInterval,
    duration: MsDuration,
    next_call: MsDuration,
}

// Enum allowing more descriptive values for the is_interval field
#[derive(JSTraceable, PartialEq, Copy, Clone, HeapSizeOf)]
pub enum IsInterval {
    Interval,
    NonInterval,
}

impl Ord for Timer {
    fn cmp(&self, other: &Timer) -> Ordering {
        // TimerEntries are stored in a max heap. => earlier is greater
        match self.next_call.cmp(&other.next_call).reverse() {
            Ordering::Equal => self.handle.cmp(&other.handle).reverse(),
            res @ _ => res
        }
    }
}

impl PartialOrd for Timer {
    fn partial_cmp(&self, other: &Timer) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Eq for Timer {}
impl PartialEq for Timer {
    fn eq(&self, other: &Timer) -> bool {
        self as *const Timer == other as *const Timer
    }
}

#[derive(JSTraceable, Clone)]
pub enum TimerCallback {
    StringTimerCallback(DOMString),
    FunctionTimerCallback(Rc<Function>)
}

impl HeapSizeOf for TimerCallback {
    fn heap_size_of_children(&self) -> usize {
        // FIXME: Rc<T> isn't HeapSizeOf and we can't ignore it due to #6870 and #6871
        0
    }
}

impl ActiveTimers {
    pub fn new(timer_event_chan: Box<TimerEventChan + Send>,
               scheduler_chan: Sender<TimerEventRequest>)
               -> ActiveTimers {
        ActiveTimers {
            timer_event_chan: timer_event_chan,
            scheduler_chan: scheduler_chan,
            next_timer_handle: Cell::new(TimerHandle(1)),
            timers: DOMRefCell::new(BinaryHeap::new()),
            suspended_since: Cell::new(None),
            suspension_offset: Cell::new(Length::new(0)),
            expected_event_id: Cell::new(TimerEventId(0)),
        }
    }

    pub fn set_timeout_or_interval(&self,
                               callback: TimerCallback,
                               arguments: Vec<HandleValue>,
                               timeout: i32,
                               is_interval: IsInterval,
                               source: TimerSource)
                               -> i32 {
        assert!(self.suspended_since.get().is_none());

        let TimerHandle(new_handle) = self.next_timer_handle.get();
        self.next_timer_handle.set(TimerHandle(new_handle + 1));

        let min_duration = match is_interval {
            IsInterval::NonInterval => 0,
            IsInterval::Interval => 1,
        };
        let duration = cmp::max(min_duration, timeout as u32);

        let next_call = self.base_time() + duration;

        let mut timer = Timer {
            handle: TimerHandle(new_handle),
            source: source,
            callback: callback,
            arguments: Vec::with_capacity(arguments.len()),
            is_interval: is_interval,
            duration: duration,
            next_call: next_call,
        };

        // This is a bit complicated, but this ensures that the vector's
        // buffer isn't reallocated (and moved) after setting the Heap values
        for _ in 0..arguments.len() {
            timer.arguments.push(Heap::default());
        }
        for (i, item) in arguments.iter().enumerate() {
            timer.arguments.get_mut(i).unwrap().set(item.get());
        }

        self.timers.borrow_mut().push(timer);
        let TimerHandle(max_handle) = self.timers.borrow().peek().unwrap().handle;

        if max_handle == new_handle {
            self.schedule_timer_call();
        }

        new_handle
    }

    pub fn clear_timeout_or_interval(&self, handle: i32) {
        let handle = TimerHandle(handle);
        let was_first = self.is_next_timer(handle);

        {
            let mut timers = self.timers.borrow_mut();
            let new_timers = timers.drain().filter(|t| t.handle != handle).collect();
            *timers = new_timers;
        }

        if was_first {
            self.invalidate_expected_event_id();
            self.schedule_timer_call();
        }
    }

    pub fn fire_timer<T: Reflectable>(&self, id: TimerEventId, this: &T) {
        let expected_id = self.expected_event_id.get();
        if expected_id != id {
            debug!("ignoring timer fire event {:?} (expected {:?}", id, expected_id);
            return;
        }

        assert!(self.suspended_since.get().is_none());

        let base_time = self.base_time();

        // Since the event id was the expected one, at least one timer should be due.
        assert!(base_time >= self.timers.borrow().peek().unwrap().next_call);

        loop {
            let timer = {
                let mut timers = self.timers.borrow_mut();

                if timers.is_empty() || timers.peek().unwrap().next_call > base_time {
                    break;
                }

                timers.pop().unwrap()
            };

            match timer.callback.clone() {
                TimerCallback::FunctionTimerCallback(function) => {
                    let arguments: Vec<JSVal> = timer.arguments.iter().map(|arg| arg.get()).collect();
                    let arguments = arguments.iter().map(|arg| HandleValue { ptr: arg }).collect();

                    let _ = function.Call_(this, arguments, Report);
                }
                TimerCallback::StringTimerCallback(code_str) => {
                    let proxy = this.reflector().get_jsobject();
                    let cx = global_object_for_js_object(proxy.get()).r().get_cx();
                    let mut rval = RootedValue::new(cx, UndefinedValue());
                    this.evaluate_js_on_global_with_result(&code_str, rval.handle_mut());
                }
            }

            if timer.is_interval == IsInterval::Interval {
                let mut timer = timer;
                timer.next_call = timer.next_call + timer.duration;
                self.timers.borrow_mut().push(timer);
            }
        }

        self.schedule_timer_call();
    }

    fn is_next_timer(&self, handle: TimerHandle) -> bool {
        match self.timers.borrow().peek() {
            None => false,
            Some(ref max_timer) => max_timer.handle == handle
        }
    }

    fn schedule_timer_call(&self) {
        assert!(self.suspended_since.get().is_none());

        let timers = self.timers.borrow();

        if let Some(timer) = timers.peek() {
            let expected_event_id = self.invalidate_expected_event_id();

            let delay = Length::new(timer.next_call.get().saturating_sub(precise_time_ms().get()));
            let request = TimerEventRequest(self.timer_event_chan.clone(), timer.source,
                                            expected_event_id, delay);
            self.scheduler_chan.send(request).unwrap();
        }
    }

    pub fn suspend(&self) {
        assert!(self.suspended_since.get().is_none());

        self.suspended_since.set(Some(precise_time_ms()));
        self.invalidate_expected_event_id();
    }

    pub fn resume(&self) {
        assert!(self.suspended_since.get().is_some());

        let additional_offset = match self.suspended_since.get() {
            Some(suspended_since) => precise_time_ms() - suspended_since,
            None => panic!("Timers are not suspended.")
        };

        self.suspension_offset.set(self.suspension_offset.get() + additional_offset);

        self.schedule_timer_call();
    }

    fn base_time(&self) -> MsDuration {
        precise_time_ms() - self.suspension_offset.get()
    }

    fn invalidate_expected_event_id(&self) -> TimerEventId {
        let TimerEventId(currently_expected) = self.expected_event_id.get();
        let next_id = TimerEventId(currently_expected + 1);
        debug!("invalidating expected timer (was {:?}, now {:?}", currently_expected, next_id);
        self.expected_event_id.set(next_id);
        next_id
    }
}

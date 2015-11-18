/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::callback::ExceptionHandling::Report;
use dom::bindings::cell::DOMRefCell;
use dom::bindings::codegen::Bindings::FunctionBinding::Function;
use dom::bindings::global::global_root_from_reflector;
use dom::bindings::reflector::Reflectable;
use dom::bindings::trace::JSTraceable;
use dom::window::ScriptHelpers;
use euclid::length::Length;
use ipc_channel::ipc::IpcSender;
use js::jsapi::{HandleValue, Heap, RootedValue};
use js::jsval::{JSVal, UndefinedValue};
use num::traits::Saturating;
use script_traits::{MsDuration, precise_time_ms};
use script_traits::{TimerEvent, TimerEventId, TimerEventRequest, TimerSource};
use std::cell::Cell;
use std::cmp::{self, Ord, Ordering};
use std::default::Default;
use std::rc::Rc;
use util::mem::HeapSizeOf;
use util::str::DOMString;

#[derive(JSTraceable, PartialEq, Eq, Copy, Clone, HeapSizeOf, Hash, PartialOrd, Ord)]
pub struct TimerHandle(i32);

#[derive(JSTraceable, HeapSizeOf)]
#[privatize]
pub struct ActiveTimers {
    #[ignore_heap_size_of = "Defined in std"]
    timer_event_chan: IpcSender<TimerEvent>,
    #[ignore_heap_size_of = "Defined in std"]
    scheduler_chan: IpcSender<TimerEventRequest>,
    next_timer_handle: Cell<TimerHandle>,
    timers: DOMRefCell<Vec<Timer>>,
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
    /// The nesting level of the currently executing timer task or 0.
    nesting_level: Cell<u32>,
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
    callback: InternalTimerCallback,
    is_interval: IsInterval,
    nesting_level: u32,
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
        match self.next_call.cmp(&other.next_call).reverse() {
            Ordering::Equal => self.handle.cmp(&other.handle).reverse(),
            res => res
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

#[derive(Clone)]
pub enum TimerCallback {
    StringTimerCallback(DOMString),
    FunctionTimerCallback(Rc<Function>),
}

#[derive(JSTraceable, Clone)]
enum InternalTimerCallback {
    StringTimerCallback(DOMString),
    FunctionTimerCallback(Rc<Function>, Rc<Vec<Heap<JSVal>>>),
    InternalCallback(Box<ScheduledCallback>),
}

impl HeapSizeOf for InternalTimerCallback {
    fn heap_size_of_children(&self) -> usize {
        // FIXME: Rc<T> isn't HeapSizeOf and we can't ignore it due to #6870 and #6871
        0
    }
}

pub trait ScheduledCallback: JSTraceable + HeapSizeOf {
    fn invoke(self: Box<Self>);

    fn box_clone(&self) -> Box<ScheduledCallback>;
}

impl Clone for Box<ScheduledCallback> {
    fn clone(&self) -> Box<ScheduledCallback> {
        self.box_clone()
    }
}

impl ActiveTimers {
    pub fn new(timer_event_chan: IpcSender<TimerEvent>,
               scheduler_chan: IpcSender<TimerEventRequest>)
               -> ActiveTimers {
        ActiveTimers {
            timer_event_chan: timer_event_chan,
            scheduler_chan: scheduler_chan,
            next_timer_handle: Cell::new(TimerHandle(1)),
            timers: DOMRefCell::new(Vec::new()),
            suspended_since: Cell::new(None),
            suspension_offset: Cell::new(Length::new(0)),
            expected_event_id: Cell::new(TimerEventId(0)),
            nesting_level: Cell::new(0),
        }
    }

    // see https://html.spec.whatwg.org/multipage/#timer-initialisation-steps
    pub fn set_timeout_or_interval(&self,
                               callback: TimerCallback,
                               arguments: Vec<HandleValue>,
                               timeout: i32,
                               is_interval: IsInterval,
                               source: TimerSource)
                               -> i32 {
        let callback = match callback {
            TimerCallback::StringTimerCallback(code_str) =>
                InternalTimerCallback::StringTimerCallback(code_str),
            TimerCallback::FunctionTimerCallback(function) => {
                // This is a bit complicated, but this ensures that the vector's
                // buffer isn't reallocated (and moved) after setting the Heap values
                let mut args = Vec::with_capacity(arguments.len());
                for _ in 0..arguments.len() {
                    args.push(Heap::default());
                }
                for (i, item) in arguments.iter().enumerate() {
                    args.get_mut(i).unwrap().set(item.get());
                }
                InternalTimerCallback::FunctionTimerCallback(function, Rc::new(args))
            }
        };

        let timeout = cmp::max(0, timeout);
        // step 7
        let duration = self.clamp_duration(Length::new(timeout as u64));

        let TimerHandle(handle) = self.schedule_internal_callback(callback, duration, is_interval, source);
        handle
    }

    pub fn schedule_callback(&self,
                             callback: Box<ScheduledCallback>,
                             duration: MsDuration,
                             source: TimerSource) -> TimerHandle {
        self.schedule_internal_callback(InternalTimerCallback::InternalCallback(callback),
                                        duration,
                                        IsInterval::NonInterval,
                                        source)
    }

    // see https://html.spec.whatwg.org/multipage/#timer-initialisation-steps
    fn schedule_internal_callback(&self,
                                  callback: InternalTimerCallback,
                                  duration: MsDuration,
                                  is_interval: IsInterval,
                                  source: TimerSource) -> TimerHandle {
        // step 3
        let TimerHandle(new_handle) = self.next_timer_handle.get();
        self.next_timer_handle.set(TimerHandle(new_handle + 1));

        let next_call = self.base_time() + duration;

        let timer = Timer {
            handle: TimerHandle(new_handle),
            source: source,
            callback: callback,
            is_interval: is_interval,
            duration: duration,
            // step 6
            nesting_level: self.nesting_level.get() + 1,
            next_call: next_call,
        };

        self.insert_timer(timer);

        let TimerHandle(max_handle) = self.timers.borrow().last().unwrap().handle;
        if max_handle == new_handle {
            self.schedule_timer_call();
        }

        // step 10
        TimerHandle(new_handle)
    }

    pub fn clear_timeout_or_interval(&self, handle: i32) {
        self.unschedule_callback(TimerHandle(handle));
    }

    pub fn unschedule_callback(&self, handle: TimerHandle) {
        let was_next = self.is_next_timer(handle);

        self.timers.borrow_mut().retain(|t| t.handle != handle);

        if was_next {
            self.invalidate_expected_event_id();
            self.schedule_timer_call();
        }
    }

    // see https://html.spec.whatwg.org/multipage/#timer-initialisation-steps
    #[allow(unsafe_code)]
    pub fn fire_timer<T: Reflectable>(&self, id: TimerEventId, this: &T) {
        let expected_id = self.expected_event_id.get();
        if expected_id != id {
            debug!("ignoring timer fire event {:?} (expected {:?}", id, expected_id);
            return;
        }

        assert!(self.suspended_since.get().is_none());

        let base_time = self.base_time();

        // Since the event id was the expected one, at least one timer should be due.
        assert!(base_time >= self.timers.borrow().last().unwrap().next_call);

        loop {
            let timer = {
                let mut timers = self.timers.borrow_mut();

                if timers.is_empty() || timers.last().unwrap().next_call > base_time {
                    break;
                }

                timers.pop().unwrap()
            };
            let callback = timer.callback.clone();

            // prep for step 6 in nested set_timeout_or_interval calls
            self.nesting_level.set(timer.nesting_level);

            // step 4.3
            if timer.is_interval == IsInterval::Interval {
                let mut timer = timer;

                // step 7
                timer.duration = self.clamp_duration(timer.duration);
                // step 8, 9
                timer.nesting_level += 1;
                timer.next_call = base_time + timer.duration;
                self.insert_timer(timer);
            }

            // step 14
            match callback {
                InternalTimerCallback::StringTimerCallback(code_str) => {
                    let cx = global_root_from_reflector(this).r().get_cx();
                    let mut rval = RootedValue::new(cx, UndefinedValue());

                    this.evaluate_js_on_global_with_result(&code_str, rval.handle_mut());
                },
                InternalTimerCallback::FunctionTimerCallback(function, arguments) => {
                    let arguments: Vec<JSVal> = arguments.iter().map(|arg| arg.get()).collect();
                    let arguments = arguments.iter().by_ref().map(|arg| unsafe {
                        HandleValue::from_marked_location(arg)
                    }).collect();

                    let _ = function.Call_(this, arguments, Report);
                },
                InternalTimerCallback::InternalCallback(callback) => {
                    callback.invoke();
                },
            };

            self.nesting_level.set(0);
        }

        self.schedule_timer_call();
    }

    fn insert_timer(&self, timer: Timer) {
        let mut timers = self.timers.borrow_mut();
        let insertion_index = timers.binary_search(&timer).err().unwrap();
        timers.insert(insertion_index, timer);
    }

    fn is_next_timer(&self, handle: TimerHandle) -> bool {
        match self.timers.borrow().last() {
            None => false,
            Some(ref max_timer) => max_timer.handle == handle
        }
    }

    fn schedule_timer_call(&self) {
        if self.suspended_since.get().is_some() {
            // The timer will be scheduled when the pipeline is thawed.
            return;
        }

        let timers = self.timers.borrow();

        if let Some(timer) = timers.last() {
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
        let offset = self.suspension_offset.get();

        match self.suspended_since.get() {
            Some(time) => time - offset,
            None => precise_time_ms() - offset,
        }
    }

    // see step 7 of https://html.spec.whatwg.org/multipage/#timer-initialisation-steps
    fn clamp_duration(&self, unclamped: MsDuration) -> MsDuration {
        let ms = if self.nesting_level.get() > 5 {
            4
        } else {
            0
        };

        cmp::max(Length::new(ms), unclamped)
    }

    fn invalidate_expected_event_id(&self) -> TimerEventId {
        let TimerEventId(currently_expected) = self.expected_event_id.get();
        let next_id = TimerEventId(currently_expected + 1);
        debug!("invalidating expected timer (was {:?}, now {:?}", currently_expected, next_id);
        self.expected_event_id.set(next_id);
        next_id
    }
}

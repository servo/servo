/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::Cell;
use std::cmp::{Ord, Ordering};
use std::collections::{HashMap, VecDeque};
use std::default::Default;
use std::rc::Rc;
use std::time::{Duration, Instant};

use base::id::PipelineId;
use deny_public_fields::DenyPublicFields;
use js::jsapi::Heap;
use js::jsval::{JSVal, UndefinedValue};
use js::rust::HandleValue;
use serde::{Deserialize, Serialize};
use servo_config::pref;
use timers::{BoxedTimerCallback, TimerEventRequest};

use crate::dom::bindings::callback::ExceptionHandling::Report;
use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::FunctionBinding::Function;
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::refcounted::Trusted;
use crate::dom::bindings::reflector::{DomGlobal, DomObject};
use crate::dom::bindings::root::Dom;
use crate::dom::bindings::str::DOMString;
use crate::dom::csp::is_js_evaluation_allowed;
use crate::dom::document::{ImageAnimationUpdateCallback, RefreshRedirectDue};
use crate::dom::eventsource::EventSourceTimeoutCallback;
use crate::dom::globalscope::GlobalScope;
#[cfg(feature = "testbinding")]
use crate::dom::testbinding::TestBindingCallback;
use crate::dom::types::{Window, WorkerGlobalScope};
use crate::dom::xmlhttprequest::XHRTimeoutCallback;
use crate::script_module::ScriptFetchOptions;
use crate::script_runtime::CanGc;
use crate::script_thread::ScriptThread;
use crate::task_source::SendableTaskSource;

#[derive(Clone, Copy, Debug, Eq, Hash, JSTraceable, MallocSizeOf, Ord, PartialEq, PartialOrd)]
pub(crate) struct OneshotTimerHandle(i32);

#[derive(DenyPublicFields, JSTraceable, MallocSizeOf)]
#[cfg_attr(crown, crown::unrooted_must_root_lint::must_root)]
pub(crate) struct OneshotTimers {
    global_scope: Dom<GlobalScope>,
    js_timers: JsTimers,
    next_timer_handle: Cell<OneshotTimerHandle>,
    timers: DomRefCell<VecDeque<OneshotTimer>>,
    suspended_since: Cell<Option<Instant>>,
    /// Initially 0, increased whenever the associated document is reactivated
    /// by the amount of ms the document was inactive. The current time can be
    /// offset back by this amount for a coherent time across document
    /// activations.
    suspension_offset: Cell<Duration>,
    /// Calls to `fire_timer` with a different argument than this get ignored.
    /// They were previously scheduled and got invalidated when
    ///  - timers were suspended,
    ///  - the timer it was scheduled for got canceled or
    ///  - a timer was added with an earlier callback time. In this case the
    ///    original timer is rescheduled when it is the next one to get called.
    #[no_trace]
    expected_event_id: Cell<TimerEventId>,
}

#[derive(DenyPublicFields, JSTraceable, MallocSizeOf)]
struct OneshotTimer {
    handle: OneshotTimerHandle,
    #[no_trace]
    source: TimerSource,
    callback: OneshotTimerCallback,
    scheduled_for: Instant,
}

// This enum is required to work around the fact that trait objects do not support generic methods.
// A replacement trait would have a method such as
//     `invoke<T: DomObject>(self: Box<Self>, this: &T, js_timers: &JsTimers);`.
#[derive(JSTraceable, MallocSizeOf)]
pub(crate) enum OneshotTimerCallback {
    XhrTimeout(XHRTimeoutCallback),
    EventSourceTimeout(EventSourceTimeoutCallback),
    JsTimer(JsTimerTask),
    #[cfg(feature = "testbinding")]
    TestBindingCallback(TestBindingCallback),
    RefreshRedirectDue(RefreshRedirectDue),
    ImageAnimationUpdate(ImageAnimationUpdateCallback),
}

impl OneshotTimerCallback {
    fn invoke<T: DomObject>(self, this: &T, js_timers: &JsTimers, can_gc: CanGc) {
        match self {
            OneshotTimerCallback::XhrTimeout(callback) => callback.invoke(can_gc),
            OneshotTimerCallback::EventSourceTimeout(callback) => callback.invoke(),
            OneshotTimerCallback::JsTimer(task) => task.invoke(this, js_timers, can_gc),
            #[cfg(feature = "testbinding")]
            OneshotTimerCallback::TestBindingCallback(callback) => callback.invoke(),
            OneshotTimerCallback::RefreshRedirectDue(callback) => callback.invoke(can_gc),
            OneshotTimerCallback::ImageAnimationUpdate(callback) => callback.invoke(can_gc),
        }
    }
}

impl Ord for OneshotTimer {
    fn cmp(&self, other: &OneshotTimer) -> Ordering {
        match self.scheduled_for.cmp(&other.scheduled_for).reverse() {
            Ordering::Equal => self.handle.cmp(&other.handle).reverse(),
            res => res,
        }
    }
}

impl PartialOrd for OneshotTimer {
    fn partial_cmp(&self, other: &OneshotTimer) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Eq for OneshotTimer {}
impl PartialEq for OneshotTimer {
    fn eq(&self, other: &OneshotTimer) -> bool {
        std::ptr::eq(self, other)
    }
}

impl OneshotTimers {
    pub(crate) fn new(global_scope: &GlobalScope) -> OneshotTimers {
        OneshotTimers {
            global_scope: Dom::from_ref(global_scope),
            js_timers: JsTimers::default(),
            next_timer_handle: Cell::new(OneshotTimerHandle(1)),
            timers: DomRefCell::new(VecDeque::new()),
            suspended_since: Cell::new(None),
            suspension_offset: Cell::new(Duration::ZERO),
            expected_event_id: Cell::new(TimerEventId(0)),
        }
    }

    pub(crate) fn schedule_callback(
        &self,
        callback: OneshotTimerCallback,
        duration: Duration,
        source: TimerSource,
    ) -> OneshotTimerHandle {
        let new_handle = self.next_timer_handle.get();
        self.next_timer_handle
            .set(OneshotTimerHandle(new_handle.0 + 1));

        let timer = OneshotTimer {
            handle: new_handle,
            source,
            callback,
            scheduled_for: self.base_time() + duration,
        };

        {
            let mut timers = self.timers.borrow_mut();
            let insertion_index = timers.binary_search(&timer).err().unwrap();
            timers.insert(insertion_index, timer);
        }

        if self.is_next_timer(new_handle) {
            self.schedule_timer_call();
        }

        new_handle
    }

    pub(crate) fn unschedule_callback(&self, handle: OneshotTimerHandle) {
        let was_next = self.is_next_timer(handle);

        self.timers.borrow_mut().retain(|t| t.handle != handle);

        if was_next {
            self.invalidate_expected_event_id();
            self.schedule_timer_call();
        }
    }

    fn is_next_timer(&self, handle: OneshotTimerHandle) -> bool {
        match self.timers.borrow().back() {
            None => false,
            Some(max_timer) => max_timer.handle == handle,
        }
    }

    pub(crate) fn fire_timer(&self, id: TimerEventId, global: &GlobalScope, can_gc: CanGc) {
        let expected_id = self.expected_event_id.get();
        if expected_id != id {
            debug!(
                "ignoring timer fire event {:?} (expected {:?})",
                id, expected_id
            );
            return;
        }

        assert!(self.suspended_since.get().is_none());

        let base_time = self.base_time();

        // Since the event id was the expected one, at least one timer should be due.
        if base_time < self.timers.borrow().back().unwrap().scheduled_for {
            warn!("Unexpected timing!");
            return;
        }

        // select timers to run to prevent firing timers
        // that were installed during fire of another timer
        let mut timers_to_run = Vec::new();

        loop {
            let mut timers = self.timers.borrow_mut();

            if timers.is_empty() || timers.back().unwrap().scheduled_for > base_time {
                break;
            }

            timers_to_run.push(timers.pop_back().unwrap());
        }

        for timer in timers_to_run {
            // Since timers can be coalesced together inside a task,
            // this loop can keep running, including after an interrupt of the JS,
            // and prevent a clean-shutdown of a JS-running thread.
            // This check prevents such a situation.
            if !global.can_continue_running() {
                return;
            }
            let callback = timer.callback;
            callback.invoke(global, &self.js_timers, can_gc);
        }

        self.schedule_timer_call();
    }

    fn base_time(&self) -> Instant {
        let offset = self.suspension_offset.get();
        match self.suspended_since.get() {
            Some(suspend_time) => suspend_time - offset,
            None => Instant::now() - offset,
        }
    }

    pub(crate) fn slow_down(&self) {
        let min_duration_ms = pref!(js_timers_minimum_duration) as u64;
        self.js_timers
            .set_min_duration(Duration::from_millis(min_duration_ms));
    }

    pub(crate) fn speed_up(&self) {
        self.js_timers.remove_min_duration();
    }

    pub(crate) fn suspend(&self) {
        // Suspend is idempotent: do nothing if the timers are already suspended.
        if self.suspended_since.get().is_some() {
            return warn!("Suspending an already suspended timer.");
        }

        debug!("Suspending timers.");
        self.suspended_since.set(Some(Instant::now()));
        self.invalidate_expected_event_id();
    }

    pub(crate) fn resume(&self) {
        // Resume is idempotent: do nothing if the timers are already resumed.
        let additional_offset = match self.suspended_since.get() {
            Some(suspended_since) => Instant::now() - suspended_since,
            None => return warn!("Resuming an already resumed timer."),
        };

        debug!("Resuming timers.");
        self.suspension_offset
            .set(self.suspension_offset.get() + additional_offset);
        self.suspended_since.set(None);

        self.schedule_timer_call();
    }

    fn schedule_timer_call(&self) {
        if self.suspended_since.get().is_some() {
            // The timer will be scheduled when the pipeline is fully activated.
            return;
        }

        let timers = self.timers.borrow();
        let Some(timer) = timers.back() else {
            return;
        };

        let expected_event_id = self.invalidate_expected_event_id();
        let callback = TimerListener {
            context: Trusted::new(&*self.global_scope),
            task_source: self
                .global_scope
                .task_manager()
                .timer_task_source()
                .to_sendable(),
            source: timer.source,
            id: expected_event_id,
        }
        .into_callback();

        let event_request = TimerEventRequest {
            callback,
            duration: timer.scheduled_for - Instant::now(),
        };

        self.global_scope.schedule_timer(event_request);
    }

    fn invalidate_expected_event_id(&self) -> TimerEventId {
        let TimerEventId(currently_expected) = self.expected_event_id.get();
        let next_id = TimerEventId(currently_expected + 1);
        debug!(
            "invalidating expected timer (was {:?}, now {:?}",
            currently_expected, next_id
        );
        self.expected_event_id.set(next_id);
        next_id
    }

    pub(crate) fn set_timeout_or_interval(
        &self,
        global: &GlobalScope,
        callback: TimerCallback,
        arguments: Vec<HandleValue>,
        timeout: Duration,
        is_interval: IsInterval,
        source: TimerSource,
    ) -> i32 {
        self.js_timers.set_timeout_or_interval(
            global,
            callback,
            arguments,
            timeout,
            is_interval,
            source,
        )
    }

    pub(crate) fn clear_timeout_or_interval(&self, global: &GlobalScope, handle: i32) {
        self.js_timers.clear_timeout_or_interval(global, handle)
    }
}

#[derive(Clone, Copy, Eq, Hash, JSTraceable, MallocSizeOf, Ord, PartialEq, PartialOrd)]
pub(crate) struct JsTimerHandle(i32);

#[derive(DenyPublicFields, JSTraceable, MallocSizeOf)]
pub(crate) struct JsTimers {
    next_timer_handle: Cell<JsTimerHandle>,
    /// <https://html.spec.whatwg.org/multipage/#list-of-active-timers>
    active_timers: DomRefCell<HashMap<JsTimerHandle, JsTimerEntry>>,
    /// The nesting level of the currently executing timer task or 0.
    nesting_level: Cell<u32>,
    /// Used to introduce a minimum delay in event intervals
    min_duration: Cell<Option<Duration>>,
}

#[derive(JSTraceable, MallocSizeOf)]
struct JsTimerEntry {
    oneshot_handle: OneshotTimerHandle,
}

// Holder for the various JS values associated with setTimeout
// (ie. function value to invoke and all arguments to pass
//      to the function when calling it)
// TODO: Handle rooting during invocation when movable GC is turned on
#[derive(JSTraceable, MallocSizeOf)]
pub(crate) struct JsTimerTask {
    #[ignore_malloc_size_of = "Because it is non-owning"]
    handle: JsTimerHandle,
    #[no_trace]
    source: TimerSource,
    callback: InternalTimerCallback,
    is_interval: IsInterval,
    nesting_level: u32,
    duration: Duration,
    is_user_interacting: bool,
}

// Enum allowing more descriptive values for the is_interval field
#[derive(Clone, Copy, JSTraceable, MallocSizeOf, PartialEq)]
pub(crate) enum IsInterval {
    Interval,
    NonInterval,
}

#[derive(Clone)]
pub(crate) enum TimerCallback {
    StringTimerCallback(DOMString),
    FunctionTimerCallback(Rc<Function>),
}

#[derive(Clone, JSTraceable, MallocSizeOf)]
enum InternalTimerCallback {
    StringTimerCallback(DOMString),
    FunctionTimerCallback(
        #[ignore_malloc_size_of = "Rc"] Rc<Function>,
        #[ignore_malloc_size_of = "Rc"] Rc<Box<[Heap<JSVal>]>>,
    ),
}

impl Default for JsTimers {
    fn default() -> Self {
        JsTimers {
            next_timer_handle: Cell::new(JsTimerHandle(1)),
            active_timers: DomRefCell::new(HashMap::new()),
            nesting_level: Cell::new(0),
            min_duration: Cell::new(None),
        }
    }
}

impl JsTimers {
    // see https://html.spec.whatwg.org/multipage/#timer-initialisation-steps
    pub(crate) fn set_timeout_or_interval(
        &self,
        global: &GlobalScope,
        callback: TimerCallback,
        arguments: Vec<HandleValue>,
        timeout: Duration,
        is_interval: IsInterval,
        source: TimerSource,
    ) -> i32 {
        let callback = match callback {
            TimerCallback::StringTimerCallback(code_str) => {
                if is_js_evaluation_allowed(global, code_str.as_ref()) {
                    InternalTimerCallback::StringTimerCallback(code_str)
                } else {
                    return 0;
                }
            },
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
                InternalTimerCallback::FunctionTimerCallback(
                    function,
                    Rc::new(args.into_boxed_slice()),
                )
            },
        };

        // step 2
        let JsTimerHandle(new_handle) = self.next_timer_handle.get();
        self.next_timer_handle.set(JsTimerHandle(new_handle + 1));

        // step 3 as part of initialize_and_schedule below

        // step 4
        let mut task = JsTimerTask {
            handle: JsTimerHandle(new_handle),
            source,
            callback,
            is_interval,
            is_user_interacting: ScriptThread::is_user_interacting(),
            nesting_level: 0,
            duration: Duration::ZERO,
        };

        // step 5
        task.duration = timeout.max(Duration::ZERO);

        // step 3, 6-9, 11-14
        self.initialize_and_schedule(global, task);

        // step 10
        new_handle
    }

    pub(crate) fn clear_timeout_or_interval(&self, global: &GlobalScope, handle: i32) {
        let mut active_timers = self.active_timers.borrow_mut();

        if let Some(entry) = active_timers.remove(&JsTimerHandle(handle)) {
            global.unschedule_callback(entry.oneshot_handle);
        }
    }

    pub(crate) fn set_min_duration(&self, duration: Duration) {
        self.min_duration.set(Some(duration));
    }

    pub(crate) fn remove_min_duration(&self) {
        self.min_duration.set(None);
    }

    // see step 13 of https://html.spec.whatwg.org/multipage/#timer-initialisation-steps
    fn user_agent_pad(&self, current_duration: Duration) -> Duration {
        match self.min_duration.get() {
            Some(min_duration) => min_duration.max(current_duration),
            None => current_duration,
        }
    }

    // see https://html.spec.whatwg.org/multipage/#timer-initialisation-steps
    fn initialize_and_schedule(&self, global: &GlobalScope, mut task: JsTimerTask) {
        let handle = task.handle;
        let mut active_timers = self.active_timers.borrow_mut();

        // step 6
        let nesting_level = self.nesting_level.get();

        // step 7, 13
        let duration = self.user_agent_pad(clamp_duration(nesting_level, task.duration));
        // step 8, 9
        task.nesting_level = nesting_level + 1;

        // essentially step 11, 12, and 14
        let callback = OneshotTimerCallback::JsTimer(task);
        let oneshot_handle = global.schedule_callback(callback, duration);

        // step 3
        let entry = active_timers
            .entry(handle)
            .or_insert(JsTimerEntry { oneshot_handle });
        entry.oneshot_handle = oneshot_handle;
    }
}

// see step 7 of https://html.spec.whatwg.org/multipage/#timer-initialisation-steps
fn clamp_duration(nesting_level: u32, unclamped: Duration) -> Duration {
    let lower_bound_ms = if nesting_level > 5 { 4 } else { 0 };
    let lower_bound = Duration::from_millis(lower_bound_ms);
    lower_bound.max(unclamped)
}

impl JsTimerTask {
    // see https://html.spec.whatwg.org/multipage/#timer-initialisation-steps
    pub(crate) fn invoke<T: DomObject>(self, this: &T, timers: &JsTimers, can_gc: CanGc) {
        // step 4.1 can be ignored, because we proactively prevent execution
        // of this task when its scheduled execution is canceled.

        // prep for step 6 in nested set_timeout_or_interval calls
        timers.nesting_level.set(self.nesting_level);

        // step 4.2
        let was_user_interacting = ScriptThread::is_user_interacting();
        ScriptThread::set_user_interacting(self.is_user_interacting);
        match self.callback {
            InternalTimerCallback::StringTimerCallback(ref code_str) => {
                let global = this.global();
                let cx = GlobalScope::get_cx();
                rooted!(in(*cx) let mut rval = UndefinedValue());
                // FIXME(cybai): Use base url properly by saving private reference for timers (#27260)
                global.evaluate_js_on_global_with_result(
                    code_str,
                    rval.handle_mut(),
                    ScriptFetchOptions::default_classic_script(&global),
                    global.api_base_url(),
                    can_gc,
                );
            },
            InternalTimerCallback::FunctionTimerCallback(ref function, ref arguments) => {
                let arguments = self.collect_heap_args(arguments);
                rooted!(in(*GlobalScope::get_cx()) let mut value: JSVal);
                let _ = function.Call_(this, arguments, value.handle_mut(), Report, can_gc);
            },
        };
        ScriptThread::set_user_interacting(was_user_interacting);

        // reset nesting level (see above)
        timers.nesting_level.set(0);

        // step 4.3
        // Since we choose proactively prevent execution (see 4.1 above), we must only
        // reschedule repeating timers when they were not canceled as part of step 4.2.
        if self.is_interval == IsInterval::Interval &&
            timers.active_timers.borrow().contains_key(&self.handle)
        {
            timers.initialize_and_schedule(&this.global(), self);
        }
    }

    // Returning Handles directly from Heap values is inherently unsafe, but here it's
    // always done via rooted JsTimers, which is safe.
    #[allow(unsafe_code)]
    fn collect_heap_args<'b>(&self, args: &'b [Heap<JSVal>]) -> Vec<HandleValue<'b>> {
        args.iter()
            .map(|arg| unsafe { HandleValue::from_raw(arg.handle()) })
            .collect()
    }
}

/// Describes the source that requested the [`TimerEvent`].
#[derive(Clone, Copy, Debug, Deserialize, MallocSizeOf, Serialize)]
pub enum TimerSource {
    /// The event was requested from a window (`ScriptThread`).
    FromWindow(PipelineId),
    /// The event was requested from a worker (`DedicatedGlobalWorkerScope`).
    FromWorker,
}

/// The id to be used for a [`TimerEvent`] is defined by the corresponding [`TimerEventRequest`].
#[derive(Clone, Copy, Debug, Deserialize, Eq, MallocSizeOf, PartialEq, Serialize)]
pub struct TimerEventId(pub u32);

/// A notification that a timer has fired. [`TimerSource`] must be `FromWindow` when
/// dispatched to `ScriptThread` and must be `FromWorker` when dispatched to a
/// `DedicatedGlobalWorkerScope`
#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub struct TimerEvent(pub TimerSource, pub TimerEventId);

/// A wrapper between timer events coming in over IPC, and the event-loop.
#[derive(Clone)]
struct TimerListener {
    task_source: SendableTaskSource,
    context: Trusted<GlobalScope>,
    source: TimerSource,
    id: TimerEventId,
}

impl TimerListener {
    /// Handle a timer-event coming from the [`timers::TimerScheduler`]
    /// by queuing the appropriate task on the relevant event-loop.
    fn handle(&self, event: TimerEvent) {
        let context = self.context.clone();
        // Step 18, queue a task,
        // https://html.spec.whatwg.org/multipage/#timer-initialisation-steps
        self.task_source.queue(task!(timer_event: move || {
                let global = context.root();
                let TimerEvent(source, id) = event;
                match source {
                    TimerSource::FromWorker => {
                        global.downcast::<WorkerGlobalScope>().expect("Window timer delivered to worker");
                    },
                    TimerSource::FromWindow(pipeline) => {
                        assert_eq!(pipeline, global.pipeline_id());
                        global.downcast::<Window>().expect("Worker timer delivered to window");
                    },
                };
                // Step 7, substeps run in a task.
                global.fire_timer(id, CanGc::note());
            })
        );
    }

    fn into_callback(self) -> BoxedTimerCallback {
        let timer_event = TimerEvent(self.source, self.id);
        Box::new(move || self.handle(timer_event))
    }
}

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
use crate::dom::bindings::codegen::UnionTypes::TrustedScriptOrString;
use crate::dom::bindings::error::Fallible;
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::refcounted::Trusted;
use crate::dom::bindings::reflector::{DomGlobal, DomObject};
use crate::dom::bindings::root::{AsHandleValue, Dom};
use crate::dom::bindings::str::DOMString;
use crate::dom::csp::CspReporting;
use crate::dom::document::RefreshRedirectDue;
use crate::dom::eventsource::EventSourceTimeoutCallback;
use crate::dom::globalscope::GlobalScope;
#[cfg(feature = "testbinding")]
use crate::dom::testbinding::TestBindingCallback;
use crate::dom::trustedscript::TrustedScript;
use crate::dom::types::{Window, WorkerGlobalScope};
use crate::dom::xmlhttprequest::XHRTimeoutCallback;
use crate::script_module::ScriptFetchOptions;
use crate::script_runtime::{CanGc, IntroductionType};
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

    /// <https://html.spec.whatwg.org/multipage/#timer-initialisation-steps>
    pub(crate) fn fire_timer(&self, id: TimerEventId, global: &GlobalScope, can_gc: CanGc) {
        // Step 9.2. If id does not exist in global's map of setTimeout and setInterval IDs, then abort these steps.
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

    /// <https://html.spec.whatwg.org/multipage/#timer-initialisation-steps>
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
        // Step 12. Let completionStep be an algorithm step which queues a global
        // task on the timer task source given global to run task.
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

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn set_timeout_or_interval(
        &self,
        global: &GlobalScope,
        callback: TimerCallback,
        arguments: Vec<HandleValue>,
        timeout: Duration,
        is_interval: IsInterval,
        source: TimerSource,
        can_gc: CanGc,
    ) -> Fallible<i32> {
        self.js_timers.set_timeout_or_interval(
            global,
            callback,
            arguments,
            timeout,
            is_interval,
            source,
            can_gc,
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

pub(crate) enum TimerCallback {
    StringTimerCallback(TrustedScriptOrString),
    FunctionTimerCallback(Rc<Function>),
}

#[derive(Clone, JSTraceable, MallocSizeOf)]
#[cfg_attr(crown, allow(crown::unrooted_must_root))]
enum InternalTimerCallback {
    StringTimerCallback(DOMString),
    FunctionTimerCallback(
        #[conditional_malloc_size_of] Rc<Function>,
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
    /// <https://html.spec.whatwg.org/multipage/#timer-initialisation-steps>
    #[allow(clippy::too_many_arguments)]
    #[cfg_attr(crown, allow(crown::unrooted_must_root))]
    pub(crate) fn set_timeout_or_interval(
        &self,
        global: &GlobalScope,
        callback: TimerCallback,
        arguments: Vec<HandleValue>,
        timeout: Duration,
        is_interval: IsInterval,
        source: TimerSource,
        can_gc: CanGc,
    ) -> Fallible<i32> {
        let callback = match callback {
            TimerCallback::StringTimerCallback(trusted_script_or_string) => {
                // Step 9.6.1.1. Let globalName be "Window" if global is a Window object; "WorkerGlobalScope" otherwise.
                let global_name = if global.is::<Window>() {
                    "Window"
                } else {
                    "WorkerGlobalScope"
                };
                // Step 9.6.1.2. Let methodName be "setInterval" if repeat is true; "setTimeout" otherwise.
                let method_name = if is_interval == IsInterval::Interval {
                    "setInterval"
                } else {
                    "setTimeout"
                };
                // Step 9.6.1.3. Let sink be a concatenation of globalName, U+0020 SPACE, and methodName.
                let sink = format!("{} {}", global_name, method_name);
                // Step 9.6.1.4. Set handler to the result of invoking the
                // Get Trusted Type compliant string algorithm with TrustedScript, global, handler, sink, and "script".
                let code_str = TrustedScript::get_trusted_script_compliant_string(
                    global,
                    trusted_script_or_string,
                    &sink,
                    can_gc,
                )?;
                // Step 9.6.3. Perform EnsureCSPDoesNotBlockStringCompilation(realm, « », handler, handler, timer, « », handler).
                // If this throws an exception, catch it, report it for global, and abort these steps.
                if global
                    .get_csp_list()
                    .is_js_evaluation_allowed(global, code_str.as_ref())
                {
                    // Step 9.6.2. Assert: handler is a string.
                    InternalTimerCallback::StringTimerCallback(code_str)
                } else {
                    return Ok(0);
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
                // Step 9.5. If handler is a Function, then invoke handler given arguments and "report",
                // and with callback this value set to thisArg.
                InternalTimerCallback::FunctionTimerCallback(
                    function,
                    Rc::new(args.into_boxed_slice()),
                )
            },
        };

        // Step 2. If previousId was given, let id be previousId; otherwise,
        // let id be an implementation-defined integer that is greater than zero
        // and does not already exist in global's map of setTimeout and setInterval IDs.
        let JsTimerHandle(new_handle) = self.next_timer_handle.get();
        self.next_timer_handle.set(JsTimerHandle(new_handle + 1));

        // Step 3. If the surrounding agent's event loop's currently running task
        // is a task that was created by this algorithm, then let nesting level
        // be the task's timer nesting level. Otherwise, let nesting level be 0.
        let mut task = JsTimerTask {
            handle: JsTimerHandle(new_handle),
            source,
            callback,
            is_interval,
            is_user_interacting: ScriptThread::is_user_interacting(),
            nesting_level: 0,
            duration: Duration::ZERO,
        };

        // Step 4. If timeout is less than 0, then set timeout to 0.
        task.duration = timeout.max(Duration::ZERO);

        self.initialize_and_schedule(global, task);

        // Step 15. Return id.
        Ok(new_handle)
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

    /// <https://html.spec.whatwg.org/multipage/#timer-initialisation-steps>
    fn initialize_and_schedule(&self, global: &GlobalScope, mut task: JsTimerTask) {
        let handle = task.handle;
        let mut active_timers = self.active_timers.borrow_mut();

        // Step 3. If the surrounding agent's event loop's currently running task
        // is a task that was created by this algorithm, then let nesting level be
        // the task's timer nesting level. Otherwise, let nesting level be 0.
        let nesting_level = self.nesting_level.get();

        let duration = self.user_agent_pad(clamp_duration(nesting_level, task.duration));
        // Step 10. Increment nesting level by one.
        // Step 11. Set task's timer nesting level to nesting level.
        task.nesting_level = nesting_level + 1;

        // Step 13. Set uniqueHandle to the result of running steps after a timeout given global,
        // "setTimeout/setInterval", timeout, and completionStep.
        let callback = OneshotTimerCallback::JsTimer(task);
        let oneshot_handle = global.schedule_callback(callback, duration);

        // Step 14. Set global's map of setTimeout and setInterval IDs[id] to uniqueHandle.
        let entry = active_timers
            .entry(handle)
            .or_insert(JsTimerEntry { oneshot_handle });
        entry.oneshot_handle = oneshot_handle;
    }
}

/// Step 5 of <https://html.spec.whatwg.org/multipage/#timer-initialisation-steps>
fn clamp_duration(nesting_level: u32, unclamped: Duration) -> Duration {
    // Step 5. If nesting level is greater than 5, and timeout is less than 4, then set timeout to 4.
    let lower_bound_ms = if nesting_level > 5 { 4 } else { 0 };
    let lower_bound = Duration::from_millis(lower_bound_ms);
    lower_bound.max(unclamped)
}

impl JsTimerTask {
    // see https://html.spec.whatwg.org/multipage/#timer-initialisation-steps
    pub(crate) fn invoke<T: DomObject>(self, this: &T, timers: &JsTimers, can_gc: CanGc) {
        // step 9.2 can be ignored, because we proactively prevent execution
        // of this task when its scheduled execution is canceled.

        // prep for step ? in nested set_timeout_or_interval calls
        timers.nesting_level.set(self.nesting_level);

        let _ = ScriptThread::user_iteracting_guard();
        match self.callback {
            InternalTimerCallback::StringTimerCallback(ref code_str) => {
                // Step 6.4. Let settings object be global's relevant settings object.
                // Step 6. Let realm be global's relevant realm.
                let global = this.global();
                // Step 7. Let initiating script be the active script.
                let cx = GlobalScope::get_cx();
                // Step 9.6.7. If initiating script is not null, then:
                rooted!(in(*cx) let mut rval = UndefinedValue());
                // Step 9.6.7.1. Set fetch options to a script fetch options whose cryptographic nonce
                // is initiating script's fetch options's cryptographic nonce,
                // integrity metadata is the empty string, parser metadata is "not-parser-inserted",
                // credentials mode is initiating script's fetch options's credentials mode,
                // referrer policy is initiating script's fetch options's referrer policy,
                // and fetch priority is "auto".
                // Step 9.6.8. Let script be the result of creating a classic script given handler,
                // settings object, base URL, and fetch options.
                // Step 9.6.9. Run the classic script script.
                //
                // FIXME(cybai): Use base url properly by saving private reference for timers (#27260)
                _ = global.evaluate_js_on_global_with_result(
                    code_str,
                    rval.handle_mut(),
                    ScriptFetchOptions::default_classic_script(&global),
                    // Step 9.6. Let base URL be settings object's API base URL.
                    // Step 9.7.2. Set base URL to initiating script's base URL.
                    global.api_base_url(),
                    can_gc,
                    Some(IntroductionType::DOM_TIMER),
                );
            },
            // Step 9.5. If handler is a Function, then invoke handler given arguments and
            // "report", and with callback this value set to thisArg.
            InternalTimerCallback::FunctionTimerCallback(ref function, ref arguments) => {
                let arguments = self.collect_heap_args(arguments);
                rooted!(in(*GlobalScope::get_cx()) let mut value: JSVal);
                let _ = function.Call_(this, arguments, value.handle_mut(), Report, can_gc);
            },
        };

        // reset nesting level (see above)
        timers.nesting_level.set(0);

        // Step 9.9. If repeat is true, then perform the timer initialization steps again,
        // given global, handler, timeout, arguments, true, and id.
        //
        // Since we choose proactively prevent execution (see 4.1 above), we must only
        // reschedule repeating timers when they were not canceled as part of step 4.2.
        if self.is_interval == IsInterval::Interval &&
            timers.active_timers.borrow().contains_key(&self.handle)
        {
            timers.initialize_and_schedule(&this.global(), self);
        }
    }

    fn collect_heap_args<'b>(&self, args: &'b [Heap<JSVal>]) -> Vec<HandleValue<'b>> {
        args.iter().map(|arg| arg.as_handle_value()).collect()
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
    /// <https://html.spec.whatwg.org/multipage/#timer-initialisation-steps>
    fn handle(&self, event: TimerEvent) {
        let context = self.context.clone();
        // Step 9. Let task be a task that runs the following substeps:
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
                global.fire_timer(id, CanGc::note());
            })
        );
    }

    fn into_callback(self) -> BoxedTimerCallback {
        let timer_event = TimerEvent(self.source, self.id);
        Box::new(move || self.handle(timer_event))
    }
}

/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use devtools_traits::{ScriptToDevtoolsControlMsg, WorkerId};
use dom::bindings::cell::DomRefCell;
use dom::bindings::codegen::Bindings::EventSourceBinding::EventSourceBinding::EventSourceMethods;
use dom::bindings::codegen::Bindings::WindowBinding::WindowMethods;
use dom::bindings::codegen::Bindings::WorkerGlobalScopeBinding::WorkerGlobalScopeMethods;
use dom::bindings::conversions::root_from_object;
use dom::bindings::error::{ErrorInfo, report_pending_exception};
use dom::bindings::inheritance::Castable;
use dom::bindings::reflector::DomObject;
use dom::bindings::root::{DomRoot, MutNullableDom};
use dom::bindings::settings_stack::{AutoEntryScript, entry_global, incumbent_global};
use dom::bindings::str::DOMString;
use dom::bindings::weakref::DOMTracker;
use dom::crypto::Crypto;
use dom::dedicatedworkerglobalscope::DedicatedWorkerGlobalScope;
use dom::errorevent::ErrorEvent;
use dom::event::{Event, EventBubbles, EventCancelable, EventStatus};
use dom::eventsource::EventSource;
use dom::eventtarget::EventTarget;
use dom::performance::Performance;
use dom::window::Window;
use dom::workerglobalscope::WorkerGlobalScope;
use dom::workletglobalscope::WorkletGlobalScope;
use dom_struct::dom_struct;
use ipc_channel::ipc::IpcSender;
use js::{JSCLASS_IS_DOMJSCLASS, JSCLASS_IS_GLOBAL};
use js::glue::{IsWrapper, UnwrapObject};
use js::jsapi::{CurrentGlobalOrNull, GetGlobalForObjectCrossCompartment};
use js::jsapi::{JSAutoCompartment, JSContext};
use js::jsapi::JSObject;
use js::panic::maybe_resume_unwind;
use js::rust::{CompileOptionsWrapper, Runtime, get_object_class};
use js::rust::{HandleValue, MutableHandleValue};
use js::rust::wrappers::Evaluate2;
use libc;
use microtask::{Microtask, MicrotaskQueue};
use msg::constellation_msg::PipelineId;
use net_traits::{CoreResourceThread, ResourceThreads, IpcSend};
use profile_traits::{mem, time};
use script_runtime::{CommonScriptMsg, ScriptChan, ScriptPort};
use script_thread::{MainThreadScriptChan, ScriptThread};
use script_traits::{MsDuration, ScriptToConstellationChan, TimerEvent};
use script_traits::{TimerEventId, TimerSchedulerMsg, TimerSource};
use servo_url::{MutableOrigin, ServoUrl};
use std::cell::Cell;
use std::collections::HashMap;
use std::collections::hash_map::Entry;
use std::ffi::CString;
use std::rc::Rc;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use task::TaskCanceller;
use task_source::TaskSourceName;
use task_source::file_reading::FileReadingTaskSource;
use task_source::networking::NetworkingTaskSource;
use task_source::performance_timeline::PerformanceTimelineTaskSource;
use task_source::remote_event::RemoteEventTaskSource;
use time::{Timespec, get_time};
use timers::{IsInterval, OneshotTimerCallback, OneshotTimerHandle};
use timers::{OneshotTimers, TimerCallback};

#[derive(JSTraceable)]
pub struct AutoCloseWorker(
    Arc<AtomicBool>,
);

impl Drop for AutoCloseWorker {
    fn drop(&mut self) {
        self.0.store(true, Ordering::SeqCst);
    }
}

#[dom_struct]
pub struct GlobalScope {
    eventtarget: EventTarget,
    crypto: MutNullableDom<Crypto>,
    next_worker_id: Cell<WorkerId>,

    /// Pipeline id associated with this global.
    pipeline_id: PipelineId,

    /// A flag to indicate whether the developer tools has requested
    /// live updates from the worker.
    devtools_wants_updates: Cell<bool>,

    /// Timers used by the Console API.
    console_timers: DomRefCell<HashMap<DOMString, u64>>,

    /// For providing instructions to an optional devtools server.
    #[ignore_malloc_size_of = "channels are hard"]
    devtools_chan: Option<IpcSender<ScriptToDevtoolsControlMsg>>,

    /// For sending messages to the memory profiler.
    #[ignore_malloc_size_of = "channels are hard"]
    mem_profiler_chan: mem::ProfilerChan,

    /// For sending messages to the time profiler.
    #[ignore_malloc_size_of = "channels are hard"]
    time_profiler_chan: time::ProfilerChan,

    /// A handle for communicating messages to the constellation thread.
    #[ignore_malloc_size_of = "channels are hard"]
    script_to_constellation_chan: ScriptToConstellationChan,

    #[ignore_malloc_size_of = "channels are hard"]
    scheduler_chan: IpcSender<TimerSchedulerMsg>,

    /// <https://html.spec.whatwg.org/multipage/#in-error-reporting-mode>
    in_error_reporting_mode: Cell<bool>,

    /// Associated resource threads for use by DOM objects like XMLHttpRequest,
    /// including resource_thread, filemanager_thread and storage_thread
    resource_threads: ResourceThreads,

    timers: OneshotTimers,

    /// The origin of the globalscope
    origin: MutableOrigin,

    /// The microtask queue associated with this global.
    ///
    /// It is refcounted because windows in the same script thread share the
    /// same microtask queue.
    ///
    /// <https://html.spec.whatwg.org/multipage/#microtask-queue>
    #[ignore_malloc_size_of = "Rc<T> is hard"]
    microtask_queue: Rc<MicrotaskQueue>,

    /// Vector storing closing references of all workers
    #[ignore_malloc_size_of = "Arc"]
    list_auto_close_worker: DomRefCell<Vec<AutoCloseWorker>>,

    /// Vector storing references of all eventsources.
    event_source_tracker: DOMTracker<EventSource>,
}

impl GlobalScope {
    pub fn new_inherited(
        pipeline_id: PipelineId,
        devtools_chan: Option<IpcSender<ScriptToDevtoolsControlMsg>>,
        mem_profiler_chan: mem::ProfilerChan,
        time_profiler_chan: time::ProfilerChan,
        script_to_constellation_chan: ScriptToConstellationChan,
        scheduler_chan: IpcSender<TimerSchedulerMsg>,
        resource_threads: ResourceThreads,
        timer_event_chan: IpcSender<TimerEvent>,
        origin: MutableOrigin,
        microtask_queue: Rc<MicrotaskQueue>,
    ) -> Self {
        Self {
            eventtarget: EventTarget::new_inherited(),
            crypto: Default::default(),
            next_worker_id: Cell::new(WorkerId(0)),
            pipeline_id,
            devtools_wants_updates: Default::default(),
            console_timers: DomRefCell::new(Default::default()),
            devtools_chan,
            mem_profiler_chan,
            time_profiler_chan,
            script_to_constellation_chan,
            scheduler_chan: scheduler_chan.clone(),
            in_error_reporting_mode: Default::default(),
            resource_threads,
            timers: OneshotTimers::new(timer_event_chan, scheduler_chan),
            origin,
            microtask_queue,
            list_auto_close_worker: Default::default(),
            event_source_tracker: DOMTracker::new(),
        }
    }

    pub fn track_worker(&self, closing_worker: Arc<AtomicBool>) {
       self.list_auto_close_worker.borrow_mut().push(AutoCloseWorker(closing_worker));
    }

    pub fn track_event_source(&self, event_source: &EventSource) {
        self.event_source_tracker.track(event_source);
    }

    pub fn close_event_sources(&self) -> bool {
        let mut canceled_any_fetch = false;
        self.event_source_tracker.for_each(|event_source: DomRoot<EventSource>| {
            match event_source.ReadyState() {
                2 => {},
                _ => {
                    event_source.cancel();
                    canceled_any_fetch = true;
                }
            }
        });
        canceled_any_fetch
    }

    /// Returns the global scope of the realm that the given DOM object's reflector
    /// was created in.
    #[allow(unsafe_code)]
    pub fn from_reflector<T: DomObject>(reflector: &T) -> DomRoot<Self> {
        unsafe { GlobalScope::from_object(*reflector.reflector().get_jsobject()) }
    }

    /// Returns the global scope of the realm that the given JS object was created in.
    #[allow(unsafe_code)]
    pub unsafe fn from_object(obj: *mut JSObject) -> DomRoot<Self> {
        assert!(!obj.is_null());
        let global = GetGlobalForObjectCrossCompartment(obj);
        global_scope_from_global(global)
    }

    /// Returns the global scope for the given JSContext
    #[allow(unsafe_code)]
    pub unsafe fn from_context(cx: *mut JSContext) -> DomRoot<Self> {
        let global = CurrentGlobalOrNull(cx);
        global_scope_from_global(global)
    }

    /// Returns the global object of the realm that the given JS object
    /// was created in, after unwrapping any wrappers.
    #[allow(unsafe_code)]
    pub unsafe fn from_object_maybe_wrapped(mut obj: *mut JSObject) -> DomRoot<Self> {
        if IsWrapper(obj) {
            obj = UnwrapObject(obj, /* stopAtWindowProxy = */ 0);
            assert!(!obj.is_null());
        }
        GlobalScope::from_object(obj)
    }

    pub fn get_cx(&self) -> *mut JSContext {
        Runtime::get()
    }

    pub fn crypto(&self) -> DomRoot<Crypto> {
        self.crypto.or_init(|| Crypto::new(self))
    }

    /// Get next worker id.
    pub fn get_next_worker_id(&self) -> WorkerId {
        let worker_id = self.next_worker_id.get();
        let WorkerId(id_num) = worker_id;
        self.next_worker_id.set(WorkerId(id_num + 1));
        worker_id
    }

    pub fn live_devtools_updates(&self) -> bool {
        self.devtools_wants_updates.get()
    }

    pub fn set_devtools_wants_updates(&self, value: bool) {
        self.devtools_wants_updates.set(value);
    }

    pub fn time(&self, label: DOMString) -> Result<(), ()> {
        let mut timers = self.console_timers.borrow_mut();
        if timers.len() >= 10000 {
            return Err(());
        }
        match timers.entry(label) {
            Entry::Vacant(entry) => {
                entry.insert(timestamp_in_ms(get_time()));
                Ok(())
            },
            Entry::Occupied(_) => Err(()),
        }
    }

    pub fn time_end(&self, label: &str) -> Result<u64, ()> {
        self.console_timers.borrow_mut().remove(label).ok_or(()).map(|start| {
            timestamp_in_ms(get_time()) - start
        })
    }

    /// Get an `&IpcSender<ScriptToDevtoolsControlMsg>` to send messages
    /// to the devtools thread when available.
    pub fn devtools_chan(&self) -> Option<&IpcSender<ScriptToDevtoolsControlMsg>> {
        self.devtools_chan.as_ref()
    }

    /// Get a sender to the memory profiler thread.
    pub fn mem_profiler_chan(&self) -> &mem::ProfilerChan {
        &self.mem_profiler_chan
    }

    /// Get a sender to the time profiler thread.
    pub fn time_profiler_chan(&self) -> &time::ProfilerChan {
        &self.time_profiler_chan
    }

    /// Get a sender to the constellation thread.
    pub fn script_to_constellation_chan(&self) -> &ScriptToConstellationChan {
        &self.script_to_constellation_chan
    }

    pub fn scheduler_chan(&self) -> &IpcSender<TimerSchedulerMsg> {
        &self.scheduler_chan
    }

    /// Get the `PipelineId` for this global scope.
    pub fn pipeline_id(&self) -> PipelineId {
        self.pipeline_id
    }

    /// Get the origin for this global scope
    pub fn origin(&self) -> &MutableOrigin {
        &self.origin
    }

    /// Get the [base url](https://html.spec.whatwg.org/multipage/#api-base-url)
    /// for this global scope.
    pub fn api_base_url(&self) -> ServoUrl {
        if let Some(window) = self.downcast::<Window>() {
            // https://html.spec.whatwg.org/multipage/#script-settings-for-browsing-contexts:api-base-url
            return window.Document().base_url();
        }
        if let Some(worker) = self.downcast::<WorkerGlobalScope>() {
            // https://html.spec.whatwg.org/multipage/#script-settings-for-workers:api-base-url
            return worker.get_url().clone();
        }
        if let Some(worklet) = self.downcast::<WorkletGlobalScope>() {
            // https://drafts.css-houdini.org/worklets/#script-settings-for-worklets
            return worklet.base_url();
        }
        unreachable!();
    }

    /// Get the URL for this global scope.
    pub fn get_url(&self) -> ServoUrl {
        if let Some(window) = self.downcast::<Window>() {
            return window.get_url();
        }
        if let Some(worker) = self.downcast::<WorkerGlobalScope>() {
            return worker.get_url().clone();
        }
        if let Some(worklet) = self.downcast::<WorkletGlobalScope>() {
            // TODO: is this the right URL to return?
            return worklet.base_url();
        }
        unreachable!();
    }

    /// Extract a `Window`, panic if the global object is not a `Window`.
    pub fn as_window(&self) -> &Window {
        self.downcast::<Window>().expect("expected a Window scope")
    }

    /// <https://html.spec.whatwg.org/multipage/#report-the-error>
    pub fn report_an_error(&self, error_info: ErrorInfo, value: HandleValue) {
        // Step 1.
        if self.in_error_reporting_mode.get() {
            return;
        }

        // Step 2.
        self.in_error_reporting_mode.set(true);

        // Steps 3-6.
        // FIXME(#13195): muted errors.
        let event = ErrorEvent::new(
            self,
            atom!("error"),
            EventBubbles::DoesNotBubble,
            EventCancelable::Cancelable,
            error_info.message.as_str().into(),
            error_info.filename.as_str().into(),
            error_info.lineno,
            error_info.column,
            value,
        );

        // Step 7.
        let event_status = event.upcast::<Event>().fire(self.upcast::<EventTarget>());

        // Step 8.
        self.in_error_reporting_mode.set(false);

        // Step 9.
        if event_status == EventStatus::NotCanceled {
            // https://html.spec.whatwg.org/multipage/#runtime-script-errors-2
            if let Some(dedicated) = self.downcast::<DedicatedWorkerGlobalScope>() {
                dedicated.forward_error_to_worker_object(error_info);
            }
        }

    }

    /// Get the `&ResourceThreads` for this global scope.
    pub fn resource_threads(&self) -> &ResourceThreads {
        &self.resource_threads
    }

    /// Get the `CoreResourceThread` for this global scope.
    pub fn core_resource_thread(&self) -> CoreResourceThread {
        self.resource_threads().sender()
    }

    /// `ScriptChan` to send messages to the event loop of this global scope.
    pub fn script_chan(&self) -> Box<ScriptChan + Send> {
        if let Some(window) = self.downcast::<Window>() {
            return MainThreadScriptChan(window.main_thread_script_chan().clone()).clone();
        }
        if let Some(worker) = self.downcast::<WorkerGlobalScope>() {
            return worker.script_chan();
        }
        unreachable!();
    }

    /// `ScriptChan` to send messages to the networking task source of
    /// this global scope.
    pub fn networking_task_source(&self) -> NetworkingTaskSource {
        if let Some(window) = self.downcast::<Window>() {
            return window.networking_task_source();
        }
        if let Some(worker) = self.downcast::<WorkerGlobalScope>() {
            return worker.networking_task_source();
        }
        unreachable!();
    }

    /// `ScriptChan` to send messages to the remote-event task source of
    /// this global scope.
    pub fn remote_event_task_source(&self) -> RemoteEventTaskSource {
        if let Some(window) = self.downcast::<Window>() {
            return window.remote_event_task_source();
        }
        if let Some(worker) = self.downcast::<WorkerGlobalScope>() {
            return worker.remote_event_task_source();
        }
        unreachable!();
    }

    /// Evaluate JS code on this global scope.
    pub fn evaluate_js_on_global_with_result(
            &self, code: &str, rval: MutableHandleValue) -> bool {
        self.evaluate_script_on_global_with_result(code, "", rval, 1)
    }

    /// Evaluate a JS script on this global scope.
    #[allow(unsafe_code)]
    pub fn evaluate_script_on_global_with_result(
            &self, code: &str, filename: &str, rval: MutableHandleValue, line_number: u32) -> bool {
        let metadata = time::TimerMetadata {
            url: if filename.is_empty() {
                self.get_url().as_str().into()
            } else {
                filename.into()
            },
            iframe: time::TimerMetadataFrameType::RootWindow,
            incremental: time::TimerMetadataReflowType::FirstReflow,
        };
        time::profile(
            time::ProfilerCategory::ScriptEvaluate,
            Some(metadata),
            self.time_profiler_chan().clone(),
            || {
                let cx = self.get_cx();
                let globalhandle = self.reflector().get_jsobject();
                let code: Vec<u16> = code.encode_utf16().collect();
                let filename = CString::new(filename).unwrap();

                let _ac = JSAutoCompartment::new(cx, globalhandle.get());
                let _aes = AutoEntryScript::new(self);
                let options = CompileOptionsWrapper::new(cx, filename.as_ptr(), line_number);

                debug!("evaluating Dom string");
                let result = unsafe {
                    Evaluate2(cx, options.ptr, code.as_ptr(),
                              code.len() as libc::size_t,
                              rval)
                };

                if !result {
                    debug!("error evaluating Dom string");
                    unsafe { report_pending_exception(cx, true) };
                }

                maybe_resume_unwind();
                result
            }
        )
    }

    pub fn schedule_callback(
            &self, callback: OneshotTimerCallback, duration: MsDuration)
            -> OneshotTimerHandle {
        self.timers.schedule_callback(callback, duration, self.timer_source())
    }

    pub fn unschedule_callback(&self, handle: OneshotTimerHandle) {
        self.timers.unschedule_callback(handle);
    }

    pub fn set_timeout_or_interval(
            &self,
            callback: TimerCallback,
            arguments: Vec<HandleValue>,
            timeout: i32,
            is_interval: IsInterval)
            -> i32 {
        self.timers.set_timeout_or_interval(
            self, callback, arguments, timeout, is_interval, self.timer_source())
    }

    pub fn clear_timeout_or_interval(&self, handle: i32) {
        self.timers.clear_timeout_or_interval(self, handle)
    }

    pub fn fire_timer(&self, handle: TimerEventId) {
        self.timers.fire_timer(handle, self)
    }

    pub fn resume(&self) {
        self.timers.resume()
    }

    pub fn suspend(&self) {
        self.timers.suspend()
    }

    pub fn slow_down_timers(&self) {
        self.timers.slow_down()
    }

    pub fn speed_up_timers(&self) {
        self.timers.speed_up()
    }

    fn timer_source(&self) -> TimerSource {
        if self.is::<Window>() {
            return TimerSource::FromWindow(self.pipeline_id());
        }
        if self.is::<WorkerGlobalScope>() {
            return TimerSource::FromWorker;
        }
        unreachable!();
    }

    /// Returns the task canceller of this global to ensure that everything is
    /// properly cancelled when the global scope is destroyed.
    pub fn task_canceller(&self, name: TaskSourceName) -> TaskCanceller {
        if let Some(window) = self.downcast::<Window>() {
            return window.task_canceller(name);
        }
        if let Some(worker) = self.downcast::<WorkerGlobalScope>() {
            // Note: the "name" is not passed to the worker,
            // because 'closing' it only requires one task canceller for all task sources.
            // https://html.spec.whatwg.org/multipage/#dom-workerglobalscope-closing
            return worker.task_canceller();
        }
        unreachable!();
    }

    /// Perform a microtask checkpoint.
    pub fn perform_a_microtask_checkpoint(&self) {
        self.microtask_queue.checkpoint(|_| Some(DomRoot::from_ref(self)));
    }

    /// Enqueue a microtask for subsequent execution.
    pub fn enqueue_microtask(&self, job: Microtask) {
        self.microtask_queue.enqueue(job);
    }

    /// Create a new sender/receiver pair that can be used to implement an on-demand
    /// event loop. Used for implementing web APIs that require blocking semantics
    /// without resorting to nested event loops.
    pub fn new_script_pair(&self) -> (Box<ScriptChan + Send>, Box<ScriptPort + Send>) {
        if let Some(window) = self.downcast::<Window>() {
            return window.new_script_pair();
        }
        if let Some(worker) = self.downcast::<WorkerGlobalScope>() {
            return worker.new_script_pair();
        }
        unreachable!();
    }

    /// Returns the microtask queue of this global.
    pub fn microtask_queue(&self) -> &Rc<MicrotaskQueue> {
        &self.microtask_queue
    }

    /// Process a single event as if it were the next event
    /// in the thread queue for this global scope.
    pub fn process_event(&self, msg: CommonScriptMsg) {
        if self.is::<Window>() {
            return ScriptThread::process_event(msg);
        }
        if let Some(worker) = self.downcast::<WorkerGlobalScope>() {
            return worker.process_event(msg);
        }
        unreachable!();
    }

    /// Channel to send messages to the file reading task source of
    /// this of this global scope.
    pub fn file_reading_task_source(&self) -> FileReadingTaskSource {
        if let Some(window) = self.downcast::<Window>() {
            return window.file_reading_task_source();
        }
        if let Some(worker) = self.downcast::<WorkerGlobalScope>() {
            return worker.file_reading_task_source();
        }
        unreachable!();
    }

    /// Returns the ["current"] global object.
    ///
    /// ["current"]: https://html.spec.whatwg.org/multipage/#current
    #[allow(unsafe_code)]
    pub fn current() -> Option<DomRoot<Self>> {
        unsafe {
            let cx = Runtime::get();
            assert!(!cx.is_null());
            let global = CurrentGlobalOrNull(cx);
            if global.is_null() {
                None
            } else {
                Some(global_scope_from_global(global))
            }
        }
    }

    /// Returns the ["entry"] global object.
    ///
    /// ["entry"]: https://html.spec.whatwg.org/multipage/#entry
    pub fn entry() -> DomRoot<Self> {
        entry_global()
    }

    /// Returns the ["incumbent"] global object.
    ///
    /// ["incumbent"]: https://html.spec.whatwg.org/multipage/#incumbent
    pub fn incumbent() -> Option<DomRoot<Self>> {
        incumbent_global()
    }

    pub fn performance(&self) -> DomRoot<Performance> {
        if let Some(window) = self.downcast::<Window>() {
            return window.Performance();
        }
        if let Some(worker) = self.downcast::<WorkerGlobalScope>() {
            return worker.Performance();
        }
        unreachable!();
    }

    /// Channel to send messages to the performance timeline task source
    /// of this global scope.
    pub fn performance_timeline_task_source(&self) -> PerformanceTimelineTaskSource {
        if let Some(window) = self.downcast::<Window>() {
            return window.performance_timeline_task_source();
        }
        if let Some(worker) = self.downcast::<WorkerGlobalScope>() {
            return worker.performance_timeline_task_source();
        }
        unreachable!();
    }

}

fn timestamp_in_ms(time: Timespec) -> u64 {
    (time.sec * 1000 + (time.nsec / 1000000) as i64) as u64
}

/// Returns the Rust global scope from a JS global object.
#[allow(unsafe_code)]
unsafe fn global_scope_from_global(global: *mut JSObject) -> DomRoot<GlobalScope> {
    assert!(!global.is_null());
    let clasp = get_object_class(global);
    assert_ne!(((*clasp).flags & (JSCLASS_IS_DOMJSCLASS | JSCLASS_IS_GLOBAL)), 0);
    root_from_object(global).unwrap()
}

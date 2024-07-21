/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! The script runtime contains common traits and structs commonly used by the
//! script thread, the dom, and the worker threads.

#![allow(dead_code)]

use core::ffi::c_char;
use std::cell::Cell;
use std::ffi::CString;
use std::io::{stdout, Write};
use std::ops::Deref;
use std::os::raw::c_void;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use std::{fmt, os, ptr, thread};

use base::id::PipelineId;
use js::glue::{
    CollectServoSizes, CreateJobQueue, DeleteJobQueue, DispatchableRun, JobQueueTraps,
    RUST_js_GetErrorMessage, SetBuildId, StreamConsumerConsumeChunk,
    StreamConsumerNoteResponseURLs, StreamConsumerStreamEnd, StreamConsumerStreamError,
};
use js::jsapi::{
    BuildIdCharVector, ContextOptionsRef, DisableIncrementalGC, Dispatchable as JSRunnable,
    Dispatchable_MaybeShuttingDown, GCDescription, GCOptions, GCProgress, GCReason,
    GetPromiseUserInputEventHandlingState, HandleObject, Heap, InitConsumeStreamCallback,
    InitDispatchToEventLoop, JSContext as RawJSContext, JSGCParamKey, JSGCStatus,
    JSJitCompilerOption, JSObject, JSSecurityCallbacks, JSTracer, JS_AddExtraGCRootsTracer,
    JS_InitDestroyPrincipalsCallback, JS_InitReadPrincipalsCallback, JS_RequestInterruptCallback,
    JS_SetGCCallback, JS_SetGCParameter, JS_SetGlobalJitCompilerOption,
    JS_SetOffthreadIonCompilationEnabled, JS_SetParallelParsingEnabled, JS_SetSecurityCallbacks,
    JobQueue, MimeType, PromiseRejectionHandlingState, PromiseUserInputEventHandlingState,
    SetDOMCallbacks, SetGCSliceCallback, SetJobQueue, SetPreserveWrapperCallbacks,
    SetProcessBuildIdOp, SetPromiseRejectionTrackerCallback, StreamConsumer as JSStreamConsumer,
};
use js::jsval::UndefinedValue;
use js::panic::wrap_panic;
use js::rust::wrappers::{GetPromiseIsHandled, JS_GetPromiseResult};
use js::rust::{
    Handle, HandleObject as RustHandleObject, IntoHandle, JSEngine, JSEngineHandle, ParentRuntime,
    Runtime as RustRuntime,
};
use lazy_static::lazy_static;
use malloc_size_of::MallocSizeOfOps;
use profile_traits::mem::{Report, ReportKind, ReportsChan};
use profile_traits::path;
use servo_config::{opts, pref};
use style::thread_state::{self, ThreadState};

use crate::body::BodyMixin;
use crate::dom::bindings::codegen::Bindings::PromiseBinding::PromiseJobCallback;
use crate::dom::bindings::codegen::Bindings::ResponseBinding::ResponseType as DOMResponseType;
use crate::dom::bindings::codegen::Bindings::ResponseBinding::Response_Binding::ResponseMethods;
use crate::dom::bindings::conversions::{
    get_dom_class, private_from_object, root_from_handleobject,
};
use crate::dom::bindings::error::{throw_dom_exception, Error};
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::refcounted::{
    trace_refcounted_objects, LiveDOMReferences, Trusted, TrustedPromise,
};
use crate::dom::bindings::reflector::DomObject;
use crate::dom::bindings::root::trace_roots;
use crate::dom::bindings::trace::JSTraceable;
use crate::dom::bindings::utils::DOM_CALLBACKS;
use crate::dom::bindings::{principals, settings_stack};
use crate::dom::event::{Event, EventBubbles, EventCancelable, EventStatus};
use crate::dom::eventtarget::EventTarget;
use crate::dom::globalscope::GlobalScope;
use crate::dom::promise::Promise;
use crate::dom::promiserejectionevent::PromiseRejectionEvent;
use crate::dom::response::Response;
use crate::microtask::{EnqueuedPromiseCallback, Microtask, MicrotaskQueue};
use crate::realms::{AlreadyInRealm, InRealm};
use crate::script_module::EnsureModuleHooksInitialized;
use crate::script_thread::trace_thread;
use crate::task::TaskBox;
use crate::task_source::networking::NetworkingTaskSource;
use crate::task_source::{TaskSource, TaskSourceName};

static JOB_QUEUE_TRAPS: JobQueueTraps = JobQueueTraps {
    getIncumbentGlobal: Some(get_incumbent_global),
    enqueuePromiseJob: Some(enqueue_promise_job),
    empty: Some(empty),
};

static SECURITY_CALLBACKS: JSSecurityCallbacks = JSSecurityCallbacks {
    // TODO: Content Security Policy <https://developer.mozilla.org/en-US/docs/Web/HTTP/CSP>
    contentSecurityPolicyAllows: None,
    subsumes: Some(principals::subsumes),
};

/// Common messages used to control the event loops in both the script and the worker
pub enum CommonScriptMsg {
    /// Requests that the script thread measure its memory usage. The results are sent back via the
    /// supplied channel.
    CollectReports(ReportsChan),
    /// Generic message that encapsulates event handling.
    Task(
        ScriptThreadEventCategory,
        Box<dyn TaskBox>,
        Option<PipelineId>,
        TaskSourceName,
    ),
}

impl fmt::Debug for CommonScriptMsg {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            CommonScriptMsg::CollectReports(_) => write!(f, "CollectReports(...)"),
            CommonScriptMsg::Task(ref category, ref task, _, _) => {
                f.debug_tuple("Task").field(category).field(task).finish()
            },
        }
    }
}

/// A cloneable interface for communicating with an event loop.
pub trait ScriptChan: JSTraceable {
    /// Send a message to the associated event loop.
    fn send(&self, msg: CommonScriptMsg) -> Result<(), ()>;
    /// Clone this handle.
    fn clone(&self) -> Box<dyn ScriptChan + Send>;
}

#[derive(Clone, Copy, Debug, Eq, Hash, JSTraceable, PartialEq)]
pub enum ScriptThreadEventCategory {
    AttachLayout,
    ConstellationMsg,
    DevtoolsMsg,
    DocumentEvent,
    DomEvent,
    FileRead,
    FormPlannedNavigation,
    HistoryEvent,
    ImageCacheMsg,
    InputEvent,
    NetworkEvent,
    PortMessage,
    Resize,
    ScriptEvent,
    SetScrollState,
    SetViewport,
    StylesheetLoad,
    TimerEvent,
    UpdateReplacedElement,
    WebSocketEvent,
    WorkerEvent,
    WorkletEvent,
    ServiceWorkerEvent,
    EnterFullscreen,
    ExitFullscreen,
    PerformanceTimelineTask,
    WebGPUMsg,
}

/// An interface for receiving ScriptMsg values in an event loop. Used for synchronous DOM
/// APIs that need to abstract over multiple kinds of event loops (worker/main thread) with
/// different Receiver interfaces.
pub trait ScriptPort {
    fn recv(&self) -> Result<CommonScriptMsg, ()>;
}

#[allow(unsafe_code)]
unsafe extern "C" fn get_incumbent_global(_: *const c_void, _: *mut RawJSContext) -> *mut JSObject {
    let mut result = ptr::null_mut();
    wrap_panic(&mut || {
        let incumbent_global = GlobalScope::incumbent();

        assert!(incumbent_global.is_some());

        result = incumbent_global
            .map(|g| g.reflector().get_jsobject().get())
            .unwrap_or(ptr::null_mut())
    });
    result
}

#[allow(unsafe_code)]
unsafe extern "C" fn empty(extra: *const c_void) -> bool {
    let mut result = false;
    wrap_panic(&mut || {
        let microtask_queue = &*(extra as *const MicrotaskQueue);
        result = microtask_queue.empty()
    });
    result
}

/// SM callback for promise job resolution. Adds a promise callback to the current
/// global's microtask queue.
#[allow(unsafe_code)]
unsafe extern "C" fn enqueue_promise_job(
    extra: *const c_void,
    cx: *mut RawJSContext,
    promise: HandleObject,
    job: HandleObject,
    _allocation_site: HandleObject,
    incumbent_global: HandleObject,
) -> bool {
    let cx = JSContext::from_ptr(cx);
    let mut result = false;
    wrap_panic(&mut || {
        let microtask_queue = &*(extra as *const MicrotaskQueue);
        let global = if !incumbent_global.is_null() {
            GlobalScope::from_object(incumbent_global.get())
        } else {
            let realm = AlreadyInRealm::assert_for_cx(cx);
            GlobalScope::from_context(*cx, InRealm::in_realm(&realm))
        };
        let pipeline = global.pipeline_id();
        let interaction = if promise.get().is_null() {
            PromiseUserInputEventHandlingState::DontCare
        } else {
            GetPromiseUserInputEventHandlingState(promise)
        };
        let is_user_interacting =
            interaction == PromiseUserInputEventHandlingState::HadUserInteractionAtCreation;
        microtask_queue.enqueue(
            Microtask::Promise(EnqueuedPromiseCallback {
                callback: PromiseJobCallback::new(cx, job.get()),
                pipeline,
                is_user_interacting,
            }),
            cx,
        );
        result = true
    });
    result
}

#[allow(unsafe_code, crown::unrooted_must_root)]
/// <https://html.spec.whatwg.org/multipage/#the-hostpromiserejectiontracker-implementation>
unsafe extern "C" fn promise_rejection_tracker(
    cx: *mut RawJSContext,
    _muted_errors: bool,
    promise: HandleObject,
    state: PromiseRejectionHandlingState,
    _data: *mut c_void,
) {
    // TODO: Step 2 - If script's muted errors is true, terminate these steps.

    // Step 3.
    let cx = JSContext::from_ptr(cx);
    let in_realm_proof = AlreadyInRealm::assert_for_cx(cx);
    let global = GlobalScope::from_context(*cx, InRealm::Already(&in_realm_proof));

    wrap_panic(&mut || {
        match state {
            // Step 4.
            PromiseRejectionHandlingState::Unhandled => {
                global.add_uncaught_rejection(promise);
            },
            // Step 5.
            PromiseRejectionHandlingState::Handled => {
                // Step 5-1.
                if global
                    .get_uncaught_rejections()
                    .borrow()
                    .contains(&Heap::boxed(promise.get()))
                {
                    global.remove_uncaught_rejection(promise);
                    return;
                }

                // Step 5-2.
                if !global
                    .get_consumed_rejections()
                    .borrow()
                    .contains(&Heap::boxed(promise.get()))
                {
                    return;
                }

                // Step 5-3.
                global.remove_consumed_rejection(promise);

                let target = Trusted::new(global.upcast::<EventTarget>());
                let promise = Promise::new_with_js_promise(Handle::from_raw(promise), cx);
                let trusted_promise = TrustedPromise::new(promise.clone());

                // Step 5-4.
                global.dom_manipulation_task_source().queue(
                task!(rejection_handled_event: move || {
                    let target = target.root();
                    let cx = GlobalScope::get_cx();
                    let root_promise = trusted_promise.root();

                    rooted!(in(*cx) let mut reason = UndefinedValue());
                    JS_GetPromiseResult(root_promise.reflector().get_jsobject(), reason.handle_mut());

                    let event = PromiseRejectionEvent::new(
                        &target.global(),
                        atom!("rejectionhandled"),
                        EventBubbles::DoesNotBubble,
                        EventCancelable::Cancelable,
                        root_promise,
                        reason.handle()
                    );

                    event.upcast::<Event>().fire(&target);
                }),
                global.upcast(),
            ).unwrap();
            },
        };
    })
}

#[allow(unsafe_code, crown::unrooted_must_root)]
/// <https://html.spec.whatwg.org/multipage/#notify-about-rejected-promises>
pub fn notify_about_rejected_promises(global: &GlobalScope) {
    let cx = GlobalScope::get_cx();
    unsafe {
        // Step 2.
        if global.get_uncaught_rejections().borrow().len() > 0 {
            // Step 1.
            let uncaught_rejections: Vec<TrustedPromise> = global
                .get_uncaught_rejections()
                .borrow()
                .iter()
                .map(|promise| {
                    let promise =
                        Promise::new_with_js_promise(Handle::from_raw(promise.handle()), cx);

                    TrustedPromise::new(promise)
                })
                .collect();

            // Step 3.
            global.get_uncaught_rejections().borrow_mut().clear();

            let target = Trusted::new(global.upcast::<EventTarget>());

            // Step 4.
            global.dom_manipulation_task_source().queue(
                task!(unhandled_rejection_event: move || {
                    let target = target.root();
                    let cx = GlobalScope::get_cx();

                    for promise in uncaught_rejections {
                        let promise = promise.root();

                        // Step 4-1.
                        let promise_is_handled = GetPromiseIsHandled(promise.reflector().get_jsobject());
                        if promise_is_handled {
                            continue;
                        }

                        // Step 4-2.
                        rooted!(in(*cx) let mut reason = UndefinedValue());
                        JS_GetPromiseResult(promise.reflector().get_jsobject(), reason.handle_mut());

                        let event = PromiseRejectionEvent::new(
                            &target.global(),
                            atom!("unhandledrejection"),
                            EventBubbles::DoesNotBubble,
                            EventCancelable::Cancelable,
                            promise.clone(),
                            reason.handle()
                        );

                        let event_status = event.upcast::<Event>().fire(&target);

                        // Step 4-3.
                        if event_status == EventStatus::Canceled {
                            // TODO: The promise rejection is not handled; we need to add it back to the list.
                        }

                        // Step 4-4.
                        if !promise_is_handled {
                            target.global().add_consumed_rejection(promise.reflector().get_jsobject().into_handle());
                        }
                    }
                }),
                global.upcast(),
            ).unwrap();
        }
    }
}

#[derive(JSTraceable)]
pub struct Runtime {
    rt: RustRuntime,
    pub microtask_queue: Rc<MicrotaskQueue>,
    job_queue: *mut JobQueue,
}

impl Drop for Runtime {
    #[allow(unsafe_code)]
    fn drop(&mut self) {
        unsafe {
            DeleteJobQueue(self.job_queue);
        }
        THREAD_ACTIVE.with(|t| {
            LiveDOMReferences::destruct();
            t.set(false);
        });
    }
}

impl Deref for Runtime {
    type Target = RustRuntime;
    fn deref(&self) -> &RustRuntime {
        &self.rt
    }
}

pub struct JSEngineSetup(JSEngine);

impl Default for JSEngineSetup {
    fn default() -> Self {
        let engine = JSEngine::init().unwrap();
        *JS_ENGINE.lock().unwrap() = Some(engine.handle());
        Self(engine)
    }
}

impl Drop for JSEngineSetup {
    fn drop(&mut self) {
        *JS_ENGINE.lock().unwrap() = None;

        while !self.0.can_shutdown() {
            thread::sleep(Duration::from_millis(50));
        }
    }
}

lazy_static! {
    static ref JS_ENGINE: Mutex<Option<JSEngineHandle>> = Mutex::new(None);
}

#[allow(unsafe_code)]
pub unsafe fn new_child_runtime(
    parent: ParentRuntime,
    networking_task_source: Option<NetworkingTaskSource>,
) -> Runtime {
    new_rt_and_cx_with_parent(Some(parent), networking_task_source)
}

#[allow(unsafe_code)]
pub fn new_rt_and_cx(networking_task_source: Option<NetworkingTaskSource>) -> Runtime {
    unsafe { new_rt_and_cx_with_parent(None, networking_task_source) }
}

#[allow(unsafe_code)]
unsafe fn new_rt_and_cx_with_parent(
    parent: Option<ParentRuntime>,
    networking_task_source: Option<NetworkingTaskSource>,
) -> Runtime {
    LiveDOMReferences::initialize();
    let (cx, runtime) = if let Some(parent) = parent {
        let runtime = RustRuntime::create_with_parent(parent);
        let cx = runtime.cx();
        (cx, runtime)
    } else {
        let runtime = RustRuntime::new(JS_ENGINE.lock().unwrap().as_ref().unwrap().clone());
        (runtime.cx(), runtime)
    };

    JS_AddExtraGCRootsTracer(cx, Some(trace_rust_roots), ptr::null_mut());

    JS_SetSecurityCallbacks(cx, &SECURITY_CALLBACKS);

    JS_InitDestroyPrincipalsCallback(cx, Some(principals::destroy_servo_jsprincipal));
    JS_InitReadPrincipalsCallback(cx, Some(principals::read_jsprincipal));

    // Needed for debug assertions about whether GC is running.
    if cfg!(debug_assertions) {
        JS_SetGCCallback(cx, Some(debug_gc_callback), ptr::null_mut());
    }

    if opts::get().debug.gc_profile {
        SetGCSliceCallback(cx, Some(gc_slice_callback));
    }

    unsafe extern "C" fn empty_wrapper_callback(_: *mut RawJSContext, _: HandleObject) -> bool {
        true
    }
    unsafe extern "C" fn empty_has_released_callback(_: HandleObject) -> bool {
        // fixme: return true when the Drop impl for a DOM object has been invoked
        false
    }
    SetDOMCallbacks(cx, &DOM_CALLBACKS);
    SetPreserveWrapperCallbacks(
        cx,
        Some(empty_wrapper_callback),
        Some(empty_has_released_callback),
    );
    // Pre barriers aren't working correctly at the moment
    DisableIncrementalGC(cx);

    unsafe extern "C" fn dispatch_to_event_loop(
        closure: *mut c_void,
        dispatchable: *mut JSRunnable,
    ) -> bool {
        let networking_task_src: &NetworkingTaskSource = &*(closure as *mut NetworkingTaskSource);
        let runnable = Runnable(dispatchable);
        let task = task!(dispatch_to_event_loop_message: move || {
            runnable.run(RustRuntime::get(), Dispatchable_MaybeShuttingDown::NotShuttingDown);
        });

        networking_task_src.queue_unconditionally(task).is_ok()
    }

    if let Some(source) = networking_task_source {
        let networking_task_src = Box::new(source);
        InitDispatchToEventLoop(
            cx,
            Some(dispatch_to_event_loop),
            Box::into_raw(networking_task_src) as *mut c_void,
        );
    }

    InitConsumeStreamCallback(cx, Some(consume_stream), Some(report_stream_error));

    let microtask_queue = Rc::new(MicrotaskQueue::default());
    let job_queue = CreateJobQueue(
        &JOB_QUEUE_TRAPS,
        &*microtask_queue as *const _ as *const c_void,
    );
    SetJobQueue(cx, job_queue);
    SetPromiseRejectionTrackerCallback(cx, Some(promise_rejection_tracker), ptr::null_mut());

    EnsureModuleHooksInitialized(runtime.rt());

    set_gc_zeal_options(cx);

    // Enable or disable the JITs.
    let cx_opts = &mut *ContextOptionsRef(cx);
    JS_SetGlobalJitCompilerOption(
        cx,
        JSJitCompilerOption::JSJITCOMPILER_BASELINE_INTERPRETER_ENABLE,
        pref!(js.baseline_interpreter.enabled) as u32,
    );
    JS_SetGlobalJitCompilerOption(
        cx,
        JSJitCompilerOption::JSJITCOMPILER_BASELINE_ENABLE,
        pref!(js.baseline_jit.enabled) as u32,
    );
    JS_SetGlobalJitCompilerOption(
        cx,
        JSJitCompilerOption::JSJITCOMPILER_ION_ENABLE,
        pref!(js.ion.enabled) as u32,
    );
    cx_opts.set_asmJS_(pref!(js.asmjs.enabled));
    let wasm_enabled = pref!(js.wasm.enabled);
    cx_opts.set_wasm_(wasm_enabled);
    if wasm_enabled {
        // If WASM is enabled without setting the buildIdOp,
        // initializing a module will report an out of memory error.
        // https://dxr.mozilla.org/mozilla-central/source/js/src/wasm/WasmTypes.cpp#458
        SetProcessBuildIdOp(Some(servo_build_id));
    }
    cx_opts.set_wasmBaseline_(pref!(js.wasm.baseline.enabled));
    cx_opts.set_wasmIon_(pref!(js.wasm.ion.enabled));
    cx_opts.set_strictMode_(pref!(js.strict.enabled));
    // TODO: handle js.strict.debug.enabled
    // TODO: handle js.throw_on_asmjs_validation_failure (needs new Spidermonkey)
    JS_SetGlobalJitCompilerOption(
        cx,
        JSJitCompilerOption::JSJITCOMPILER_NATIVE_REGEXP_ENABLE,
        pref!(js.native_regex.enabled) as u32,
    );
    JS_SetParallelParsingEnabled(cx, pref!(js.parallel_parsing.enabled));
    JS_SetOffthreadIonCompilationEnabled(cx, pref!(js.offthread_compilation.enabled));
    JS_SetGlobalJitCompilerOption(
        cx,
        JSJitCompilerOption::JSJITCOMPILER_BASELINE_WARMUP_TRIGGER,
        if pref!(js.baseline_jit.unsafe_eager_compilation.enabled) {
            0
        } else {
            u32::max_value()
        },
    );
    JS_SetGlobalJitCompilerOption(
        cx,
        JSJitCompilerOption::JSJITCOMPILER_ION_NORMAL_WARMUP_TRIGGER,
        if pref!(js.ion.unsafe_eager_compilation.enabled) {
            0
        } else {
            u32::max_value()
        },
    );
    // TODO: handle js.discard_system_source.enabled
    // TODO: handle js.asyncstack.enabled (needs new Spidermonkey)
    // TODO: handle js.throw_on_debugee_would_run (needs new Spidermonkey)
    // TODO: handle js.dump_stack_on_debugee_would_run (needs new Spidermonkey)
    // TODO: handle js.shared_memory.enabled
    JS_SetGCParameter(
        cx,
        JSGCParamKey::JSGC_MAX_BYTES,
        in_range(pref!(js.mem.max), 1, 0x100)
            .map(|val| (val * 1024 * 1024) as u32)
            .unwrap_or(u32::max_value()),
    );
    // NOTE: This is disabled above, so enabling it here will do nothing for now.
    JS_SetGCParameter(
        cx,
        JSGCParamKey::JSGC_INCREMENTAL_GC_ENABLED,
        pref!(js.mem.gc.incremental.enabled) as u32,
    );
    JS_SetGCParameter(
        cx,
        JSGCParamKey::JSGC_PER_ZONE_GC_ENABLED,
        pref!(js.mem.gc.per_zone.enabled) as u32,
    );
    if let Some(val) = in_range(pref!(js.mem.gc.incremental.slice_ms), 0, 100_000) {
        JS_SetGCParameter(cx, JSGCParamKey::JSGC_SLICE_TIME_BUDGET_MS, val as u32);
    }
    JS_SetGCParameter(
        cx,
        JSGCParamKey::JSGC_COMPACTING_ENABLED,
        pref!(js.mem.gc.compacting.enabled) as u32,
    );

    if let Some(val) = in_range(pref!(js.mem.gc.high_frequency_time_limit_ms), 0, 10_000) {
        JS_SetGCParameter(cx, JSGCParamKey::JSGC_HIGH_FREQUENCY_TIME_LIMIT, val as u32);
    }
    if let Some(val) = in_range(pref!(js.mem.gc.low_frequency_heap_growth), 0, 10_000) {
        JS_SetGCParameter(cx, JSGCParamKey::JSGC_LOW_FREQUENCY_HEAP_GROWTH, val as u32);
    }
    if let Some(val) = in_range(pref!(js.mem.gc.high_frequency_heap_growth_min), 0, 10_000) {
        JS_SetGCParameter(
            cx,
            JSGCParamKey::JSGC_HIGH_FREQUENCY_LARGE_HEAP_GROWTH,
            val as u32,
        );
    }
    if let Some(val) = in_range(pref!(js.mem.gc.high_frequency_heap_growth_max), 0, 10_000) {
        JS_SetGCParameter(
            cx,
            JSGCParamKey::JSGC_HIGH_FREQUENCY_SMALL_HEAP_GROWTH,
            val as u32,
        );
    }
    if let Some(val) = in_range(pref!(js.mem.gc.high_frequency_low_limit_mb), 0, 10_000) {
        JS_SetGCParameter(cx, JSGCParamKey::JSGC_SMALL_HEAP_SIZE_MAX, val as u32);
    }
    if let Some(val) = in_range(pref!(js.mem.gc.high_frequency_high_limit_mb), 0, 10_000) {
        JS_SetGCParameter(cx, JSGCParamKey::JSGC_LARGE_HEAP_SIZE_MIN, val as u32);
    }
    /*if let Some(val) = in_range(pref!(js.mem.gc.allocation_threshold_factor), 0, 10_000) {
        JS_SetGCParameter(cx, JSGCParamKey::JSGC_NON_INCREMENTAL_FACTOR, val as u32);
    }*/
    /*
        // JSGC_SMALL_HEAP_INCREMENTAL_LIMIT
        pref("javascript.options.mem.gc_small_heap_incremental_limit", 140);

        // JSGC_LARGE_HEAP_INCREMENTAL_LIMIT
        pref("javascript.options.mem.gc_large_heap_incremental_limit", 110);
    */
    if let Some(val) = in_range(pref!(js.mem.gc.empty_chunk_count_min), 0, 10_000) {
        JS_SetGCParameter(cx, JSGCParamKey::JSGC_MIN_EMPTY_CHUNK_COUNT, val as u32);
    }
    if let Some(val) = in_range(pref!(js.mem.gc.empty_chunk_count_max), 0, 10_000) {
        JS_SetGCParameter(cx, JSGCParamKey::JSGC_MAX_EMPTY_CHUNK_COUNT, val as u32);
    }

    Runtime {
        rt: runtime,
        microtask_queue,
        job_queue,
    }
}

fn in_range<T: PartialOrd + Copy>(val: T, min: T, max: T) -> Option<T> {
    if val < min || val >= max {
        None
    } else {
        Some(val)
    }
}

#[allow(unsafe_code)]
unsafe extern "C" fn get_size(obj: *mut JSObject) -> usize {
    match get_dom_class(obj) {
        Ok(v) => {
            let dom_object = private_from_object(obj) as *const c_void;

            if dom_object.is_null() {
                return 0;
            }
            let mut ops = MallocSizeOfOps::new(servo_allocator::usable_size, None, None);
            (v.malloc_size_of)(&mut ops, dom_object)
        },
        Err(_e) => 0,
    }
}

#[allow(unsafe_code)]
pub unsafe fn get_reports(cx: *mut RawJSContext, path_seg: String) -> Vec<Report> {
    let mut reports = vec![];

    let mut stats = ::std::mem::zeroed();
    if CollectServoSizes(cx, &mut stats, Some(get_size)) {
        let mut report = |mut path_suffix, kind, size| {
            let mut path = path![path_seg, "js"];
            path.append(&mut path_suffix);
            reports.push(Report { path, kind, size })
        };

        // A note about possibly confusing terminology: the JS GC "heap" is allocated via
        // mmap/VirtualAlloc, which means it's not on the malloc "heap", so we use
        // `ExplicitNonHeapSize` as its kind.

        report(
            path!["gc-heap", "used"],
            ReportKind::ExplicitNonHeapSize,
            stats.gcHeapUsed,
        );

        report(
            path!["gc-heap", "unused"],
            ReportKind::ExplicitNonHeapSize,
            stats.gcHeapUnused,
        );

        report(
            path!["gc-heap", "admin"],
            ReportKind::ExplicitNonHeapSize,
            stats.gcHeapAdmin,
        );

        report(
            path!["gc-heap", "decommitted"],
            ReportKind::ExplicitNonHeapSize,
            stats.gcHeapDecommitted,
        );

        // SpiderMonkey uses the system heap, not jemalloc.
        report(
            path!["malloc-heap"],
            ReportKind::ExplicitSystemHeapSize,
            stats.mallocHeap,
        );

        report(
            path!["non-heap"],
            ReportKind::ExplicitNonHeapSize,
            stats.nonHeap,
        );
    }

    reports
}

thread_local!(static GC_CYCLE_START: Cell<Option<Instant>> = const { Cell::new(None) });
thread_local!(static GC_SLICE_START: Cell<Option<Instant>> = const { Cell::new(None) });

#[allow(unsafe_code)]
unsafe extern "C" fn gc_slice_callback(
    _cx: *mut RawJSContext,
    progress: GCProgress,
    desc: *const GCDescription,
) {
    match progress {
        GCProgress::GC_CYCLE_BEGIN => GC_CYCLE_START.with(|start| {
            start.set(Some(Instant::now()));
            println!("GC cycle began");
        }),
        GCProgress::GC_SLICE_BEGIN => GC_SLICE_START.with(|start| {
            start.set(Some(Instant::now()));
            println!("GC slice began");
        }),
        GCProgress::GC_SLICE_END => GC_SLICE_START.with(|start| {
            let duration = start.get().unwrap().elapsed();
            start.set(None);
            println!("GC slice ended: duration={:?}", duration);
        }),
        GCProgress::GC_CYCLE_END => GC_CYCLE_START.with(|start| {
            let duration = start.get().unwrap().elapsed();
            start.set(None);
            println!("GC cycle ended: duration={:?}", duration);
        }),
    };
    if !desc.is_null() {
        let desc: &GCDescription = &*desc;
        let options = match desc.options_ {
            GCOptions::Normal => "Normal",
            GCOptions::Shrink => "Shrink",
            GCOptions::Shutdown => "Shutdown",
        };
        println!("  isZone={}, options={}", desc.isZone_, options);
    }
    let _ = stdout().flush();
}

#[allow(unsafe_code)]
unsafe extern "C" fn debug_gc_callback(
    _cx: *mut RawJSContext,
    status: JSGCStatus,
    _reason: GCReason,
    _data: *mut os::raw::c_void,
) {
    match status {
        JSGCStatus::JSGC_BEGIN => thread_state::enter(ThreadState::IN_GC),
        JSGCStatus::JSGC_END => thread_state::exit(ThreadState::IN_GC),
    }
}

thread_local!(
    static THREAD_ACTIVE: Cell<bool> = const { Cell::new(true) };
);

pub(crate) fn runtime_is_alive() -> bool {
    THREAD_ACTIVE.with(|t| t.get())
}

#[allow(unsafe_code)]
unsafe extern "C" fn trace_rust_roots(tr: *mut JSTracer, _data: *mut os::raw::c_void) {
    if !THREAD_ACTIVE.with(|t| t.get()) {
        return;
    }
    debug!("starting custom root handler");
    trace_thread(tr);
    trace_roots(tr);
    trace_refcounted_objects(tr);
    settings_stack::trace(tr);
    debug!("done custom root handler");
}

#[allow(unsafe_code)]
unsafe extern "C" fn servo_build_id(build_id: *mut BuildIdCharVector) -> bool {
    let servo_id = b"Servo\0";
    SetBuildId(build_id, servo_id[0] as *const c_char, servo_id.len())
}

#[allow(unsafe_code)]
#[cfg(feature = "debugmozjs")]
unsafe fn set_gc_zeal_options(cx: *mut RawJSContext) {
    use js::jsapi::{JS_SetGCZeal, JS_DEFAULT_ZEAL_FREQ};

    let level = match pref!(js.mem.gc.zeal.level) {
        level @ 0..=14 => level as u8,
        _ => return,
    };
    let frequency = match pref!(js.mem.gc.zeal.frequency) {
        frequency if frequency >= 0 => frequency as u32,
        _ => JS_DEFAULT_ZEAL_FREQ,
    };
    JS_SetGCZeal(cx, level, frequency);
}

#[allow(unsafe_code)]
#[cfg(not(feature = "debugmozjs"))]
unsafe fn set_gc_zeal_options(_: *mut RawJSContext) {}

/// A wrapper around a JSContext that is Send,
/// enabling an interrupt to be requested
/// from a thread other than the one running JS using that context.
#[derive(Clone)]
pub struct ContextForRequestInterrupt(Arc<Mutex<Option<*mut RawJSContext>>>);

impl ContextForRequestInterrupt {
    pub fn new(context: *mut RawJSContext) -> ContextForRequestInterrupt {
        ContextForRequestInterrupt(Arc::new(Mutex::new(Some(context))))
    }

    pub fn revoke(&self) {
        self.0.lock().unwrap().take();
    }

    #[allow(unsafe_code)]
    /// Can be called from any thread, to request the callback set by
    /// JS_AddInterruptCallback to be called on the thread
    /// where that context is running.
    /// The lock is held when calling JS_RequestInterruptCallback
    /// because it is possible for the JSContext to be destroyed
    /// on the other thread in the case of Worker shutdown
    pub fn request_interrupt(&self) {
        let maybe_cx = self.0.lock().unwrap();
        if let Some(cx) = *maybe_cx {
            unsafe {
                JS_RequestInterruptCallback(cx);
            }
        }
    }
}

#[allow(unsafe_code)]
/// It is safe to call `JS_RequestInterruptCallback(cx)` from any thread.
/// See the docs for the corresponding `requestInterrupt` method,
/// at `mozjs/js/src/vm/JSContext.h`.
unsafe impl Send for ContextForRequestInterrupt {}

#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct JSContext(*mut RawJSContext);

#[allow(unsafe_code)]
impl JSContext {
    pub unsafe fn from_ptr(raw_js_context: *mut RawJSContext) -> Self {
        JSContext(raw_js_context)
    }
}

#[allow(unsafe_code)]
impl Deref for JSContext {
    type Target = *mut RawJSContext;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub struct StreamConsumer(*mut JSStreamConsumer);

#[allow(unsafe_code)]
impl StreamConsumer {
    pub fn consume_chunk(&self, stream: &[u8]) -> bool {
        unsafe {
            let stream_ptr = stream.as_ptr();
            StreamConsumerConsumeChunk(self.0, stream_ptr, stream.len())
        }
    }

    pub fn stream_end(&self) {
        unsafe {
            StreamConsumerStreamEnd(self.0);
        }
    }

    pub fn stream_error(&self, error_code: usize) {
        unsafe {
            StreamConsumerStreamError(self.0, error_code);
        }
    }

    pub fn note_response_urls(
        &self,
        maybe_url: Option<String>,
        maybe_source_map_url: Option<String>,
    ) {
        unsafe {
            let maybe_url = maybe_url.map(|url| CString::new(url).unwrap());
            let maybe_source_map_url = maybe_source_map_url.map(|url| CString::new(url).unwrap());

            let maybe_url_param = match maybe_url.as_ref() {
                Some(url) => url.as_ptr(),
                None => ptr::null(),
            };
            let maybe_source_map_url_param = match maybe_source_map_url.as_ref() {
                Some(url) => url.as_ptr(),
                None => ptr::null(),
            };

            StreamConsumerNoteResponseURLs(self.0, maybe_url_param, maybe_source_map_url_param);
        }
    }
}

/// Implements the steps to compile webassembly response mentioned here
/// <https://webassembly.github.io/spec/web-api/#compile-a-potential-webassembly-response>
#[allow(unsafe_code)]
unsafe extern "C" fn consume_stream(
    _cx: *mut RawJSContext,
    obj: HandleObject,
    _mime_type: MimeType,
    _consumer: *mut JSStreamConsumer,
) -> bool {
    let cx = JSContext::from_ptr(_cx);
    let in_realm_proof = AlreadyInRealm::assert_for_cx(cx);
    let global = GlobalScope::from_context(*cx, InRealm::Already(&in_realm_proof));

    //Step 2.1 Upon fulfillment of source, store the Response with value unwrappedSource.
    if let Ok(unwrapped_source) =
        root_from_handleobject::<Response>(RustHandleObject::from_raw(obj), *cx)
    {
        //Step 2.2 Let mimeType be the result of extracting a MIME type from response’s header list.
        let mimetype = unwrapped_source.Headers().extract_mime_type();

        //Step 2.3 If mimeType is not `application/wasm`, return with a TypeError and abort these substeps.
        if !&mimetype[..].eq_ignore_ascii_case(b"application/wasm") {
            throw_dom_exception(
                cx,
                &global,
                Error::Type("Response has unsupported MIME type".to_string()),
            );
            return false;
        }

        //Step 2.4 If response is not CORS-same-origin, return with a TypeError and abort these substeps.
        match unwrapped_source.Type() {
            DOMResponseType::Basic | DOMResponseType::Cors | DOMResponseType::Default => {},
            _ => {
                throw_dom_exception(
                    cx,
                    &global,
                    Error::Type("Response.type must be 'basic', 'cors' or 'default'".to_string()),
                );
                return false;
            },
        }

        //Step 2.5 If response’s status is not an ok status, return with a TypeError and abort these substeps.
        if !unwrapped_source.Ok() {
            throw_dom_exception(
                cx,
                &global,
                Error::Type("Response does not have ok status".to_string()),
            );
            return false;
        }

        // Step 2.6.1 If response body is locked, return with a TypeError and abort these substeps.
        if unwrapped_source.is_locked() {
            throw_dom_exception(
                cx,
                &global,
                Error::Type("There was an error consuming the Response".to_string()),
            );
            return false;
        }

        // Step 2.6.2 If response body is alreaady consumed, return with a TypeError and abort these substeps.
        if unwrapped_source.is_disturbed() {
            throw_dom_exception(
                cx,
                &global,
                Error::Type("Response already consumed".to_string()),
            );
            return false;
        }
        unwrapped_source.set_stream_consumer(Some(StreamConsumer(_consumer)));
    } else {
        //Step 3 Upon rejection of source, return with reason.
        throw_dom_exception(
            cx,
            &global,
            Error::Type("expected Response or Promise resolving to Response".to_string()),
        );
        return false;
    }
    true
}

#[allow(unsafe_code)]
unsafe extern "C" fn report_stream_error(_cx: *mut RawJSContext, error_code: usize) {
    error!(
        "Error initializing StreamConsumer: {:?}",
        RUST_js_GetErrorMessage(ptr::null_mut(), error_code as u32)
    );
}

pub struct Runnable(*mut JSRunnable);

#[allow(unsafe_code)]
unsafe impl Sync for Runnable {}
#[allow(unsafe_code)]
unsafe impl Send for Runnable {}

#[allow(unsafe_code)]
impl Runnable {
    fn run(&self, cx: *mut RawJSContext, maybe_shutting_down: Dispatchable_MaybeShuttingDown) {
        unsafe {
            DispatchableRun(cx, self.0, maybe_shutting_down);
        }
    }
}

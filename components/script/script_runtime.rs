/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! The script runtime contains common traits and structs commonly used by the
//! script thread, the dom, and the worker threads.

use crate::dom::bindings::codegen::Bindings::PromiseBinding::PromiseJobCallback;
use crate::dom::bindings::conversions::get_dom_class;
use crate::dom::bindings::conversions::private_from_object;
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::refcounted::{trace_refcounted_objects, LiveDOMReferences};
use crate::dom::bindings::refcounted::{Trusted, TrustedPromise};
use crate::dom::bindings::reflector::DomObject;
use crate::dom::bindings::root::trace_roots;
use crate::dom::bindings::settings_stack;
use crate::dom::bindings::trace::{trace_traceables, JSTraceable};
use crate::dom::bindings::utils::DOM_CALLBACKS;
use crate::dom::event::{Event, EventBubbles, EventCancelable, EventStatus};
use crate::dom::eventtarget::EventTarget;
use crate::dom::globalscope::GlobalScope;
use crate::dom::promise::Promise;
use crate::dom::promiserejectionevent::PromiseRejectionEvent;
use crate::microtask::{EnqueuedPromiseCallback, Microtask};
use crate::script_thread::trace_thread;
use crate::task::TaskBox;
use crate::task_source::{TaskSource, TaskSourceName};
use js::glue::CollectServoSizes;
use js::glue::SetBuildId;
use js::jsapi::ContextOptionsRef;
use js::jsapi::{BuildIdCharVector, DisableIncrementalGC, GCDescription, GCProgress};
use js::jsapi::{HandleObject, Heap};
use js::jsapi::{JSContext, JSTracer, SetDOMCallbacks, SetGCSliceCallback};
use js::jsapi::{JSGCInvocationKind, JSGCStatus, JS_AddExtraGCRootsTracer, JS_SetGCCallback};
use js::jsapi::{JSGCMode, JSGCParamKey, JS_SetGCParameter, JS_SetGlobalJitCompilerOption};
use js::jsapi::{
    JSJitCompilerOption, JS_SetOffthreadIonCompilationEnabled, JS_SetParallelParsingEnabled,
};
use js::jsapi::{JSObject, PromiseRejectionHandlingState, SetPreserveWrapperCallback};
use js::jsapi::{
    SetEnqueuePromiseJobCallback, SetProcessBuildIdOp, SetPromiseRejectionTrackerCallback,
};
use js::panic::wrap_panic;
use js::rust::wrappers::{GetPromiseIsHandled, GetPromiseResult};
use js::rust::Handle;
use js::rust::IntoHandle;
use js::rust::JSEngine;
use js::rust::ParentRuntime;
use js::rust::Runtime as RustRuntime;
use malloc_size_of::MallocSizeOfOps;
use msg::constellation_msg::PipelineId;
use profile_traits::mem::{Report, ReportKind, ReportsChan};
use servo_config::opts;
use servo_config::pref;
use std::cell::Cell;
use std::fmt;
use std::io::{stdout, Write};
use std::ops::Deref;
use std::os;
use std::os::raw::c_void;
use std::panic::AssertUnwindSafe;
use std::ptr;
use std::sync::Arc;
use style::thread_state::{self, ThreadState};
use time::{now, Tm};

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

/// A cloneable interface for communicating with an event loop, from the event-loop itself.
pub trait LocalScriptChan: JSTraceable {
    /// Send a message to the associated event loop.
    fn send(&self, msg: CommonScriptMsg) -> Result<(), ()>;
    /// Clone this handle.
    fn clone(&self) -> Box<dyn LocalScriptChan>;
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
    WebVREvent,
    PerformanceTimelineTask,
}

/// An interface for receiving ScriptMsg values in an event loop. Used for synchronous DOM
/// APIs that need to abstract over multiple kinds of event loops (worker/main thread) with
/// different Receiver interfaces.
pub trait ScriptPort {
    fn recv(&self) -> Result<CommonScriptMsg, ()>;
}

/// SM callback for promise job resolution. Adds a promise callback to the current
/// global's microtask queue.
#[allow(unsafe_code)]
unsafe extern "C" fn enqueue_job(
    cx: *mut JSContext,
    _promise: HandleObject,
    job: HandleObject,
    _allocation_site: HandleObject,
    incumbent_global: HandleObject,
    _data: *mut c_void,
) -> bool {
    wrap_panic(
        AssertUnwindSafe(|| {
            let global = GlobalScope::from_object(incumbent_global.get());
            let pipeline = global.pipeline_id();
            global.enqueue_microtask(Microtask::Promise(EnqueuedPromiseCallback {
                callback: PromiseJobCallback::new(cx, job.get()),
                pipeline: pipeline,
            }));
            true
        }),
        false,
    )
}

#[allow(unsafe_code, unrooted_must_root)]
/// https://html.spec.whatwg.org/multipage/#the-hostpromiserejectiontracker-implementation
unsafe extern "C" fn promise_rejection_tracker(
    cx: *mut JSContext,
    promise: HandleObject,
    state: PromiseRejectionHandlingState,
    _data: *mut c_void,
) {
    // TODO: Step 2 - If script's muted errors is true, terminate these steps.

    // Step 3.
    let global = GlobalScope::from_context(cx);

    wrap_panic(
        AssertUnwindSafe(|| {
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
                        let cx = target.global().get_cx();
                        let root_promise = trusted_promise.root();

                        rooted!(in(cx) let reason = GetPromiseResult(root_promise.reflector().get_jsobject()));

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
        }),
        (),
    );
}

#[allow(unsafe_code, unrooted_must_root)]
/// https://html.spec.whatwg.org/multipage/#notify-about-rejected-promises
pub fn notify_about_rejected_promises(global: &GlobalScope) {
    unsafe {
        let cx = global.get_cx();

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
                    let cx = target.global().get_cx();

                    for promise in uncaught_rejections {
                        let promise = promise.root();

                        // Step 4-1.
                        let promise_is_handled = GetPromiseIsHandled(promise.reflector().get_jsobject());
                        if promise_is_handled {
                            continue;
                        }

                        // Step 4-2.
                        rooted!(in(cx) let reason = GetPromiseResult(promise.reflector().get_jsobject()));

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
pub struct Runtime(RustRuntime);

impl Drop for Runtime {
    fn drop(&mut self) {
        THREAD_ACTIVE.with(|t| {
            LiveDOMReferences::destruct();
            t.set(false);
        });
    }
}

impl Deref for Runtime {
    type Target = RustRuntime;
    fn deref(&self) -> &RustRuntime {
        &self.0
    }
}

lazy_static! {
    static ref JS_ENGINE: Arc<JSEngine> = JSEngine::init().unwrap();
}

#[allow(unsafe_code)]
pub unsafe fn new_child_runtime(parent: ParentRuntime) -> Runtime {
    new_rt_and_cx_with_parent(Some(parent))
}

#[allow(unsafe_code)]
pub fn new_rt_and_cx() -> Runtime {
    unsafe { new_rt_and_cx_with_parent(None) }
}

#[allow(unsafe_code)]
unsafe fn new_rt_and_cx_with_parent(parent: Option<ParentRuntime>) -> Runtime {
    LiveDOMReferences::initialize();
    let runtime = if let Some(parent) = parent {
        RustRuntime::create_with_parent(parent)
    } else {
        RustRuntime::new(JS_ENGINE.clone())
    };
    let cx = runtime.cx();

    JS_AddExtraGCRootsTracer(cx, Some(trace_rust_roots), ptr::null_mut());

    // Needed for debug assertions about whether GC is running.
    if cfg!(debug_assertions) {
        JS_SetGCCallback(cx, Some(debug_gc_callback), ptr::null_mut());
    }

    if opts::get().gc_profile {
        SetGCSliceCallback(cx, Some(gc_slice_callback));
    }

    unsafe extern "C" fn empty_wrapper_callback(_: *mut JSContext, _: HandleObject) -> bool {
        true
    }
    SetDOMCallbacks(cx, &DOM_CALLBACKS);
    SetPreserveWrapperCallback(cx, Some(empty_wrapper_callback));
    // Pre barriers aren't working correctly at the moment
    DisableIncrementalGC(cx);

    SetEnqueuePromiseJobCallback(cx, Some(enqueue_job), ptr::null_mut());
    SetPromiseRejectionTrackerCallback(cx, Some(promise_rejection_tracker), ptr::null_mut());

    set_gc_zeal_options(cx);

    // Enable or disable the JITs.
    let cx_opts = &mut *ContextOptionsRef(cx);
    cx_opts.set_baseline_(pref!(js.baseline.enabled));
    cx_opts.set_ion_(pref!(js.ion.enabled));
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
    cx_opts.set_extraWarnings_(pref!(js.strict.enabled));
    // TODO: handle js.strict.debug.enabled
    // TODO: handle js.throw_on_asmjs_validation_failure (needs new Spidermonkey)
    cx_opts.set_nativeRegExp_(pref!(js.native_regex.enabled));
    JS_SetParallelParsingEnabled(cx, pref!(js.parallel_parsing.enabled));
    JS_SetOffthreadIonCompilationEnabled(cx, pref!(js.offthread_compilation.enabled));
    JS_SetGlobalJitCompilerOption(
        cx,
        JSJitCompilerOption::JSJITCOMPILER_BASELINE_WARMUP_TRIGGER,
        if pref!(js.baseline.unsafe_eager_compilation.enabled) {
            0
        } else {
            u32::max_value()
        },
    );
    JS_SetGlobalJitCompilerOption(
        cx,
        JSJitCompilerOption::JSJITCOMPILER_ION_WARMUP_TRIGGER,
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
    cx_opts.set_werror_(pref!(js.werror.enabled));
    // TODO: handle js.shared_memory.enabled
    JS_SetGCParameter(
        cx,
        JSGCParamKey::JSGC_MAX_MALLOC_BYTES,
        (pref!(js.mem.high_water_mark) * 1024 * 1024) as u32,
    );
    JS_SetGCParameter(
        cx,
        JSGCParamKey::JSGC_MAX_BYTES,
        in_range(pref!(js.mem.max), 1, 0x100)
            .map(|val| (val * 1024 * 1024) as u32)
            .unwrap_or(u32::max_value()),
    );
    // NOTE: This is disabled above, so enabling it here will do nothing for now.
    let js_gc_mode = if pref!(js.mem.gc.incremental.enabled) {
        JSGCMode::JSGC_MODE_INCREMENTAL
    } else if pref!(js.mem.gc.per_zone.enabled) {
        JSGCMode::JSGC_MODE_ZONE
    } else {
        JSGCMode::JSGC_MODE_GLOBAL
    };
    JS_SetGCParameter(cx, JSGCParamKey::JSGC_MODE, js_gc_mode as u32);
    if let Some(val) = in_range(pref!(js.mem.gc.incremental.slice_ms), 0, 100_000) {
        JS_SetGCParameter(cx, JSGCParamKey::JSGC_SLICE_TIME_BUDGET, val as u32);
    }
    JS_SetGCParameter(
        cx,
        JSGCParamKey::JSGC_COMPACTING_ENABLED,
        pref!(js.mem.gc.compacting.enabled) as u32,
    );

    if let Some(val) = in_range(pref!(js.mem.gc.high_frequency_time_limit_ms), 0, 10_000) {
        JS_SetGCParameter(cx, JSGCParamKey::JSGC_HIGH_FREQUENCY_TIME_LIMIT, val as u32);
    }
    JS_SetGCParameter(
        cx,
        JSGCParamKey::JSGC_DYNAMIC_MARK_SLICE,
        pref!(js.mem.gc.dynamic_mark_slice.enabled) as u32,
    );
    JS_SetGCParameter(
        cx,
        JSGCParamKey::JSGC_DYNAMIC_HEAP_GROWTH,
        pref!(js.mem.gc.dynamic_heap_growth.enabled) as u32,
    );
    if let Some(val) = in_range(pref!(js.mem.gc.low_frequency_heap_growth), 0, 10_000) {
        JS_SetGCParameter(cx, JSGCParamKey::JSGC_LOW_FREQUENCY_HEAP_GROWTH, val as u32);
    }
    if let Some(val) = in_range(pref!(js.mem.gc.high_frequency_heap_growth_min), 0, 10_000) {
        JS_SetGCParameter(
            cx,
            JSGCParamKey::JSGC_HIGH_FREQUENCY_HEAP_GROWTH_MIN,
            val as u32,
        );
    }
    if let Some(val) = in_range(pref!(js.mem.gc.high_frequency_heap_growth_max), 0, 10_000) {
        JS_SetGCParameter(
            cx,
            JSGCParamKey::JSGC_HIGH_FREQUENCY_HEAP_GROWTH_MAX,
            val as u32,
        );
    }
    if let Some(val) = in_range(pref!(js.mem.gc.high_frequency_low_limit_mb), 0, 10_000) {
        JS_SetGCParameter(cx, JSGCParamKey::JSGC_HIGH_FREQUENCY_LOW_LIMIT, val as u32);
    }
    if let Some(val) = in_range(pref!(js.mem.gc.high_frequency_high_limit_mb), 0, 10_000) {
        JS_SetGCParameter(cx, JSGCParamKey::JSGC_HIGH_FREQUENCY_HIGH_LIMIT, val as u32);
    }
    if let Some(val) = in_range(pref!(js.mem.gc.allocation_threshold_factor), 0, 10_000) {
        JS_SetGCParameter(
            cx,
            JSGCParamKey::JSGC_ALLOCATION_THRESHOLD_FACTOR,
            val as u32,
        );
    }
    if let Some(val) = in_range(
        pref!(js.mem.gc.allocation_threshold_avoid_interrupt_factor),
        0,
        10_000,
    ) {
        JS_SetGCParameter(
            cx,
            JSGCParamKey::JSGC_ALLOCATION_THRESHOLD_FACTOR_AVOID_INTERRUPT,
            val as u32,
        );
    }
    if let Some(val) = in_range(pref!(js.mem.gc.empty_chunk_count_min), 0, 10_000) {
        JS_SetGCParameter(cx, JSGCParamKey::JSGC_MIN_EMPTY_CHUNK_COUNT, val as u32);
    }
    if let Some(val) = in_range(pref!(js.mem.gc.empty_chunk_count_max), 0, 10_000) {
        JS_SetGCParameter(cx, JSGCParamKey::JSGC_MAX_EMPTY_CHUNK_COUNT, val as u32);
    }

    Runtime(runtime)
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
        Err(_e) => {
            return 0;
        },
    }
}

#[allow(unsafe_code)]
pub fn get_reports(cx: *mut JSContext, path_seg: String) -> Vec<Report> {
    let mut reports = vec![];

    unsafe {
        let mut stats = ::std::mem::zeroed();
        if CollectServoSizes(cx, &mut stats, Some(get_size)) {
            let mut report = |mut path_suffix, kind, size| {
                let mut path = path![path_seg, "js"];
                path.append(&mut path_suffix);
                reports.push(Report {
                    path: path,
                    kind: kind,
                    size: size as usize,
                })
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
    }
    reports
}

thread_local!(static GC_CYCLE_START: Cell<Option<Tm>> = Cell::new(None));
thread_local!(static GC_SLICE_START: Cell<Option<Tm>> = Cell::new(None));

#[allow(unsafe_code)]
unsafe extern "C" fn gc_slice_callback(
    _cx: *mut JSContext,
    progress: GCProgress,
    desc: *const GCDescription,
) {
    match progress {
        GCProgress::GC_CYCLE_BEGIN => GC_CYCLE_START.with(|start| {
            start.set(Some(now()));
            println!("GC cycle began");
        }),
        GCProgress::GC_SLICE_BEGIN => GC_SLICE_START.with(|start| {
            start.set(Some(now()));
            println!("GC slice began");
        }),
        GCProgress::GC_SLICE_END => GC_SLICE_START.with(|start| {
            let dur = now() - start.get().unwrap();
            start.set(None);
            println!("GC slice ended: duration={}", dur);
        }),
        GCProgress::GC_CYCLE_END => GC_CYCLE_START.with(|start| {
            let dur = now() - start.get().unwrap();
            start.set(None);
            println!("GC cycle ended: duration={}", dur);
        }),
    };
    if !desc.is_null() {
        let desc: &GCDescription = &*desc;
        let invocation_kind = match desc.invocationKind_ {
            JSGCInvocationKind::GC_NORMAL => "GC_NORMAL",
            JSGCInvocationKind::GC_SHRINK => "GC_SHRINK",
        };
        println!(
            "  isZone={}, invocation_kind={}",
            desc.isZone_, invocation_kind
        );
    }
    let _ = stdout().flush();
}

#[allow(unsafe_code)]
unsafe extern "C" fn debug_gc_callback(
    _cx: *mut JSContext,
    status: JSGCStatus,
    _data: *mut os::raw::c_void,
) {
    match status {
        JSGCStatus::JSGC_BEGIN => thread_state::enter(ThreadState::IN_GC),
        JSGCStatus::JSGC_END => thread_state::exit(ThreadState::IN_GC),
    }
}

thread_local!(
    static THREAD_ACTIVE: Cell<bool> = Cell::new(true);
);

#[allow(unsafe_code)]
unsafe extern "C" fn trace_rust_roots(tr: *mut JSTracer, _data: *mut os::raw::c_void) {
    if !THREAD_ACTIVE.with(|t| t.get()) {
        return;
    }
    debug!("starting custom root handler");
    trace_thread(tr);
    trace_traceables(tr);
    trace_roots(tr);
    trace_refcounted_objects(tr);
    settings_stack::trace(tr);
    debug!("done custom root handler");
}

#[allow(unsafe_code)]
unsafe extern "C" fn servo_build_id(build_id: *mut BuildIdCharVector) -> bool {
    let servo_id = b"Servo\0";
    SetBuildId(build_id, &servo_id[0], servo_id.len())
}

#[allow(unsafe_code)]
#[cfg(feature = "debugmozjs")]
unsafe fn set_gc_zeal_options(cx: *mut JSContext) {
    use js::jsapi::{JS_SetGCZeal, JS_DEFAULT_ZEAL_FREQ};

    let level = match pref!(js.mem.gc.zeal.level) {
        level @ 0...14 => level as u8,
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
unsafe fn set_gc_zeal_options(_: *mut JSContext) {}

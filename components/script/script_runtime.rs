/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! The script runtime contains common traits and structs commonly used by the
//! script thread, the dom, and the worker threads.

use dom::bindings::callback::ExceptionHandling;
use dom::bindings::cell::DOMRefCell;
use dom::bindings::codegen::Bindings::PromiseBinding::PromiseJobCallback;
use dom::bindings::js::{Root, RootCollection, RootCollectionPtr, trace_roots};
use dom::bindings::refcounted::{LiveDOMReferences, trace_refcounted_objects};
use dom::bindings::trace::trace_traceables;
use dom::bindings::utils::DOM_CALLBACKS;
use dom::globalscope::GlobalScope;
use js::glue::CollectServoSizes;
use js::jsapi::{DisableIncrementalGC, GCDescription, GCProgress, HandleObject};
use js::jsapi::{JSContext, JS_GetRuntime, JSRuntime, JSTracer, SetDOMCallbacks, SetGCSliceCallback};
use js::jsapi::{JSGCInvocationKind, JSGCStatus, JS_AddExtraGCRootsTracer, JS_SetGCCallback};
use js::jsapi::{JSGCMode, JSGCParamKey, JS_SetGCParameter, JS_SetGlobalJitCompilerOption};
use js::jsapi::{JSJitCompilerOption, JS_SetOffthreadIonCompilationEnabled, JS_SetParallelParsingEnabled};
use js::jsapi::{JSObject, RuntimeOptionsRef, SetPreserveWrapperCallback, SetEnqueuePromiseJobCallback};
use js::panic::wrap_panic;
use js::rust::Runtime;
use msg::constellation_msg::PipelineId;
use profile_traits::mem::{Report, ReportKind, ReportsChan};
use script_thread::{Runnable, STACK_ROOTS, trace_thread};
use std::cell::Cell;
use std::io::{Write, stdout};
use std::marker::PhantomData;
use std::os;
use std::os::raw::c_void;
use std::panic::AssertUnwindSafe;
use std::ptr;
use std::rc::Rc;
use style::thread_state;
use time::{Tm, now};
use util::opts;
use util::prefs::PREFS;

/// Common messages used to control the event loops in both the script and the worker
pub enum CommonScriptMsg {
    /// Requests that the script thread measure its memory usage. The results are sent back via the
    /// supplied channel.
    CollectReports(ReportsChan),
    /// Generic message that encapsulates event handling.
    RunnableMsg(ScriptThreadEventCategory, Box<Runnable + Send>),
}

/// A cloneable interface for communicating with an event loop.
pub trait ScriptChan {
    /// Send a message to the associated event loop.
    fn send(&self, msg: CommonScriptMsg) -> Result<(), ()>;
    /// Clone this handle.
    fn clone(&self) -> Box<ScriptChan + Send>;
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
    ServiceWorkerEvent
}

/// An interface for receiving ScriptMsg values in an event loop. Used for synchronous DOM
/// APIs that need to abstract over multiple kinds of event loops (worker/main thread) with
/// different Receiver interfaces.
pub trait ScriptPort {
    fn recv(&self) -> Result<CommonScriptMsg, ()>;
}

pub struct StackRootTLS<'a>(PhantomData<&'a u32>);

impl<'a> StackRootTLS<'a> {
    pub fn new(roots: &'a RootCollection) -> StackRootTLS<'a> {
        STACK_ROOTS.with(|ref r| {
            r.set(Some(RootCollectionPtr(roots as *const _)))
        });
        StackRootTLS(PhantomData)
    }
}

impl<'a> Drop for StackRootTLS<'a> {
    fn drop(&mut self) {
        STACK_ROOTS.with(|ref r| r.set(None));
    }
}

/// A promise callback scheduled to run during the next microtask checkpoint (#4283).
#[derive(JSTraceable, HeapSizeOf)]
pub struct EnqueuedPromiseCallback {
    #[ignore_heap_size_of = "Rc has unclear ownership"]
    callback: Rc<PromiseJobCallback>,
    pipeline: PipelineId,
}

/// A collection of promise callbacks in FIFO order.
#[derive(JSTraceable, HeapSizeOf)]
pub struct PromiseJobQueue {
    /// A snapshot of `promise_job_queue` that was taken at the start of the microtask checkpoint.
    /// Used to work around mutability errors when appending new promise jobs while performing
    /// a microtask checkpoint.
    flushing_job_queue: DOMRefCell<Vec<EnqueuedPromiseCallback>>,
    /// The list of enqueued promise callbacks that will be invoked at the next microtask checkpoint.
    promise_job_queue: DOMRefCell<Vec<EnqueuedPromiseCallback>>,
    /// True if there is an outstanding runnable responsible for evaluating the promise job queue.
    /// This prevents runnables flooding the event queue needlessly, since the first one will
    /// execute all pending runnables.
    pending_promise_job_runnable: Cell<bool>,
}

impl PromiseJobQueue {
    /// Create a new PromiseJobQueue instance.
    pub fn new() -> PromiseJobQueue {
        PromiseJobQueue {
            promise_job_queue: DOMRefCell::new(vec![]),
            flushing_job_queue: DOMRefCell::new(vec![]),
            pending_promise_job_runnable: Cell::new(false),
        }
    }

    /// Add a new promise job callback to this queue. It will be invoked as part of the next
    /// microtask checkpoint.
    pub fn enqueue(&self, job: EnqueuedPromiseCallback, global: &GlobalScope) {
        self.promise_job_queue.borrow_mut().push(job);
        if !self.pending_promise_job_runnable.get() {
            self.pending_promise_job_runnable.set(true);
            global.flush_promise_jobs();
        }
    }

    /// Perform a microtask checkpoint, by invoking all of the pending promise job callbacks in
    /// FIFO order (#4283).
    pub fn flush_promise_jobs<F>(&self, target_provider: F)
        where F: Fn(PipelineId) -> Option<Root<GlobalScope>>
    {
        self.pending_promise_job_runnable.set(false);
        {
            let mut pending_queue = self.promise_job_queue.borrow_mut();
            *self.flushing_job_queue.borrow_mut() = pending_queue.drain(..).collect();
        }
        // N.B. borrowing this vector is safe w.r.t. mutability, since any promise job that
        // is enqueued while invoking these callbacks will be placed in `pending_queue`;
        // `flushing_queue` is a static snapshot during this checkpoint.
        for job in &*self.flushing_job_queue.borrow() {
            if let Some(target) = target_provider(job.pipeline) {
                let _ = job.callback.Call_(&*target, ExceptionHandling::Report);
            }
        }
        self.flushing_job_queue.borrow_mut().clear();
    }
}

/// SM callback for promise job resolution. Adds a promise callback to the current global's
/// promise job queue, and enqueues a runnable to perform a microtask checkpoint if one
/// is not already pending.
#[allow(unsafe_code)]
unsafe extern "C" fn enqueue_job(_cx: *mut JSContext,
                                 job: HandleObject,
                                 _allocation_site: HandleObject,
                                 _data: *mut c_void) -> bool {
    wrap_panic(AssertUnwindSafe(|| {
        let global = GlobalScope::from_object(job.get());
        let pipeline = global.pipeline_id();
        global.enqueue_promise_job(EnqueuedPromiseCallback {
            callback: PromiseJobCallback::new(job.get()),
            pipeline: pipeline,
        });
        true
    }), false)
}

#[allow(unsafe_code)]
pub unsafe fn new_rt_and_cx() -> Runtime {
    LiveDOMReferences::initialize();
    let runtime = Runtime::new();

    JS_AddExtraGCRootsTracer(runtime.rt(), Some(trace_rust_roots), ptr::null_mut());
    JS_AddExtraGCRootsTracer(runtime.rt(), Some(trace_refcounted_objects), ptr::null_mut());

    // Needed for debug assertions about whether GC is running.
    if cfg!(debug_assertions) {
        JS_SetGCCallback(runtime.rt(), Some(debug_gc_callback), ptr::null_mut());
    }

    if opts::get().gc_profile {
        SetGCSliceCallback(runtime.rt(), Some(gc_slice_callback));
    }

    unsafe extern "C" fn empty_wrapper_callback(_: *mut JSContext, _: *mut JSObject) -> bool { true }
    SetDOMCallbacks(runtime.rt(), &DOM_CALLBACKS);
    SetPreserveWrapperCallback(runtime.rt(), Some(empty_wrapper_callback));
    // Pre barriers aren't working correctly at the moment
    DisableIncrementalGC(runtime.rt());

    SetEnqueuePromiseJobCallback(runtime.rt(), Some(enqueue_job), ptr::null_mut());

    set_gc_zeal_options(runtime.rt());

    // Enable or disable the JITs.
    let rt_opts = &mut *RuntimeOptionsRef(runtime.rt());
    if let Some(val) = PREFS.get("js.baseline.enabled").as_boolean() {
        rt_opts.set_baseline_(val);
    }
    if let Some(val) = PREFS.get("js.ion.enabled").as_boolean() {
        rt_opts.set_ion_(val);
    }
    if let Some(val) = PREFS.get("js.asmjs.enabled").as_boolean() {
        rt_opts.set_asmJS_(val);
    }
    if let Some(val) = PREFS.get("js.strict.enabled").as_boolean() {
        rt_opts.set_extraWarnings_(val);
    }
    // TODO: handle js.strict.debug.enabled
    // TODO: handle js.throw_on_asmjs_validation_failure (needs new Spidermonkey)
    if let Some(val) = PREFS.get("js.native_regexp.enabled").as_boolean() {
        rt_opts.set_nativeRegExp_(val);
    }
    if let Some(val) = PREFS.get("js.parallel_parsing.enabled").as_boolean() {
        JS_SetParallelParsingEnabled(runtime.rt(), val);
    }
    if let Some(val) = PREFS.get("js.offthread_compilation_enabled").as_boolean() {
        JS_SetOffthreadIonCompilationEnabled(runtime.rt(), val);
    }
    if let Some(val) = PREFS.get("js.baseline.unsafe_eager_compilation.enabled").as_boolean() {
        let trigger: i32 = if val {
            0
        } else {
            -1
        };
        JS_SetGlobalJitCompilerOption(runtime.rt(),
                                      JSJitCompilerOption::JSJITCOMPILER_BASELINE_WARMUP_TRIGGER,
                                      trigger as u32);
    }
    if let Some(val) = PREFS.get("js.ion.unsafe_eager_compilation.enabled").as_boolean() {
        let trigger: i64 = if val {
            0
        } else {
            -1
        };
        JS_SetGlobalJitCompilerOption(runtime.rt(),
                                      JSJitCompilerOption::JSJITCOMPILER_ION_WARMUP_TRIGGER,
                                      trigger as u32);
    }
    // TODO: handle js.discard_system_source.enabled
    // TODO: handle js.asyncstack.enabled (needs new Spidermonkey)
    // TODO: handle js.throw_on_debugee_would_run (needs new Spidermonkey)
    // TODO: handle js.dump_stack_on_debugee_would_run (needs new Spidermonkey)
    if let Some(val) = PREFS.get("js.werror.enabled").as_boolean() {
        rt_opts.set_werror_(val);
    }
    // TODO: handle js.shared_memory.enabled
    if let Some(val) = PREFS.get("js.mem.high_water_mark").as_i64() {
        JS_SetGCParameter(runtime.rt(), JSGCParamKey::JSGC_MAX_MALLOC_BYTES, val as u32 * 1024 * 1024);
    }
    if let Some(val) = PREFS.get("js.mem.max").as_i64() {
        let max = if val <= 0 || val >= 0x1000 {
            -1
        } else {
            val * 1024 * 1024
        };
        JS_SetGCParameter(runtime.rt(), JSGCParamKey::JSGC_MAX_BYTES, max as u32);
    }
    // NOTE: This is disabled above, so enabling it here will do nothing for now.
    if let Some(val) = PREFS.get("js.mem.gc.incremental.enabled").as_boolean() {
        let compartment = if let Some(val) = PREFS.get("js.mem.gc.per_compartment.enabled").as_boolean() {
            val
        } else {
            false
        };
        let mode = if val {
            JSGCMode::JSGC_MODE_INCREMENTAL
        } else if compartment {
            JSGCMode::JSGC_MODE_COMPARTMENT
        } else {
            JSGCMode::JSGC_MODE_GLOBAL
        };
        JS_SetGCParameter(runtime.rt(), JSGCParamKey::JSGC_MODE, mode as u32);
    }
    if let Some(val) = PREFS.get("js.mem.gc.incremental.slice_ms").as_i64() {
        if val >= 0 && val < 100000 {
            JS_SetGCParameter(runtime.rt(), JSGCParamKey::JSGC_SLICE_TIME_BUDGET, val as u32);
        }
    }
    if let Some(val) = PREFS.get("js.mem.gc.compacting.enabled").as_boolean() {
        JS_SetGCParameter(runtime.rt(), JSGCParamKey::JSGC_COMPACTING_ENABLED, val as u32);
    }
    if let Some(val) = PREFS.get("js.mem.gc.high_frequency_time_limit_ms").as_i64() {
        if val >= 0 && val < 10000 {
            JS_SetGCParameter(runtime.rt(), JSGCParamKey::JSGC_HIGH_FREQUENCY_TIME_LIMIT, val as u32);
        }
    }
    if let Some(val) = PREFS.get("js.mem.gc.dynamic_mark_slice.enabled").as_boolean() {
        JS_SetGCParameter(runtime.rt(), JSGCParamKey::JSGC_DYNAMIC_MARK_SLICE, val as u32);
    }
    // TODO: handle js.mem.gc.refresh_frame_slices.enabled
    if let Some(val) = PREFS.get("js.mem.gc.dynamic_heap_growth.enabled").as_boolean() {
        JS_SetGCParameter(runtime.rt(), JSGCParamKey::JSGC_DYNAMIC_HEAP_GROWTH, val as u32);
    }
    if let Some(val) = PREFS.get("js.mem.gc.low_frequency_heap_growth").as_i64() {
        if val >= 0 && val < 10000 {
            JS_SetGCParameter(runtime.rt(), JSGCParamKey::JSGC_LOW_FREQUENCY_HEAP_GROWTH, val as u32);
        }
    }
    if let Some(val) = PREFS.get("js.mem.gc.high_frequency_heap_growth_min").as_i64() {
        if val >= 0 && val < 10000 {
            JS_SetGCParameter(runtime.rt(), JSGCParamKey::JSGC_HIGH_FREQUENCY_HEAP_GROWTH_MIN, val as u32);
        }
    }
    if let Some(val) = PREFS.get("js.mem.gc.high_frequency_heap_growth_max").as_i64() {
        if val >= 0 && val < 10000 {
            JS_SetGCParameter(runtime.rt(), JSGCParamKey::JSGC_HIGH_FREQUENCY_HEAP_GROWTH_MAX, val as u32);
        }
    }
    if let Some(val) = PREFS.get("js.mem.gc.high_frequency_low_limit_mb").as_i64() {
        if val >= 0 && val < 10000 {
            JS_SetGCParameter(runtime.rt(), JSGCParamKey::JSGC_HIGH_FREQUENCY_LOW_LIMIT, val as u32);
        }
    }
    if let Some(val) = PREFS.get("js.mem.gc.high_frequency_high_limit_mb").as_i64() {
        if val >= 0 && val < 10000 {
            JS_SetGCParameter(runtime.rt(), JSGCParamKey::JSGC_HIGH_FREQUENCY_HIGH_LIMIT, val as u32);
        }
    }
    if let Some(val) = PREFS.get("js.mem.gc.allocation_threshold_mb").as_i64() {
        if val >= 0 && val < 10000 {
            JS_SetGCParameter(runtime.rt(), JSGCParamKey::JSGC_ALLOCATION_THRESHOLD, val as u32);
        }
    }
    if let Some(val) = PREFS.get("js.mem.gc.decommit_threshold_mb").as_i64() {
        if val >= 0 && val < 10000 {
            JS_SetGCParameter(runtime.rt(), JSGCParamKey::JSGC_DECOMMIT_THRESHOLD, val as u32);
        }
    }
    if let Some(val) = PREFS.get("js.mem.gc.empty_chunk_count_min").as_i64() {
        if val >= 0 && val < 10000 {
            JS_SetGCParameter(runtime.rt(), JSGCParamKey::JSGC_MIN_EMPTY_CHUNK_COUNT, val as u32);
        }
    }
    if let Some(val) = PREFS.get("js.mem.gc.empty_chunk_count_max").as_i64() {
        if val >= 0 && val < 10000 {
            JS_SetGCParameter(runtime.rt(), JSGCParamKey::JSGC_MAX_EMPTY_CHUNK_COUNT, val as u32);
        }
    }

    runtime
}

#[allow(unsafe_code)]
pub fn get_reports(cx: *mut JSContext, path_seg: String) -> Vec<Report> {
    let mut reports = vec![];

    unsafe {
        let rt = JS_GetRuntime(cx);
        let mut stats = ::std::mem::zeroed();
        if CollectServoSizes(rt, &mut stats) {
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

            report(path!["gc-heap", "used"],
                   ReportKind::ExplicitNonHeapSize,
                   stats.gcHeapUsed);

            report(path!["gc-heap", "unused"],
                   ReportKind::ExplicitNonHeapSize,
                   stats.gcHeapUnused);

            report(path!["gc-heap", "admin"],
                   ReportKind::ExplicitNonHeapSize,
                   stats.gcHeapAdmin);

            report(path!["gc-heap", "decommitted"],
                   ReportKind::ExplicitNonHeapSize,
                   stats.gcHeapDecommitted);

            // SpiderMonkey uses the system heap, not jemalloc.
            report(path!["malloc-heap"],
                   ReportKind::ExplicitSystemHeapSize,
                   stats.mallocHeap);

            report(path!["non-heap"],
                   ReportKind::ExplicitNonHeapSize,
                   stats.nonHeap);
        }
    }
    reports
}

thread_local!(static GC_CYCLE_START: Cell<Option<Tm>> = Cell::new(None));
thread_local!(static GC_SLICE_START: Cell<Option<Tm>> = Cell::new(None));

#[allow(unsafe_code)]
unsafe extern "C" fn gc_slice_callback(_rt: *mut JSRuntime, progress: GCProgress, desc: *const GCDescription) {
    match progress {
        GCProgress::GC_CYCLE_BEGIN => {
            GC_CYCLE_START.with(|start| {
                start.set(Some(now()));
                println!("GC cycle began");
            })
        },
        GCProgress::GC_SLICE_BEGIN => {
            GC_SLICE_START.with(|start| {
                start.set(Some(now()));
                println!("GC slice began");
            })
        },
        GCProgress::GC_SLICE_END => {
            GC_SLICE_START.with(|start| {
                let dur = now() - start.get().unwrap();
                start.set(None);
                println!("GC slice ended: duration={}", dur);
            })
        },
        GCProgress::GC_CYCLE_END => {
            GC_CYCLE_START.with(|start| {
                let dur = now() - start.get().unwrap();
                start.set(None);
                println!("GC cycle ended: duration={}", dur);
            })
        },
    };
    if !desc.is_null() {
        let desc: &GCDescription = &*desc;
        let invocation_kind = match desc.invocationKind_ {
            JSGCInvocationKind::GC_NORMAL => "GC_NORMAL",
            JSGCInvocationKind::GC_SHRINK => "GC_SHRINK",
        };
        println!("  isCompartment={}, invocation_kind={}", desc.isCompartment_, invocation_kind);
    }
    let _ = stdout().flush();
}

#[allow(unsafe_code)]
unsafe extern "C" fn debug_gc_callback(_rt: *mut JSRuntime, status: JSGCStatus, _data: *mut os::raw::c_void) {
    match status {
        JSGCStatus::JSGC_BEGIN => thread_state::enter(thread_state::IN_GC),
        JSGCStatus::JSGC_END   => thread_state::exit(thread_state::IN_GC),
    }
}

#[allow(unsafe_code)]
unsafe extern fn trace_rust_roots(tr: *mut JSTracer, _data: *mut os::raw::c_void) {
    debug!("starting custom root handler");
    trace_thread(tr);
    trace_traceables(tr);
    trace_roots(tr);
    debug!("done custom root handler");
}

#[allow(unsafe_code)]
#[cfg(feature = "debugmozjs")]
unsafe fn set_gc_zeal_options(rt: *mut JSRuntime) {
    use js::jsapi::{JS_DEFAULT_ZEAL_FREQ, JS_SetGCZeal};

    let level = match PREFS.get("js.mem.gc.zeal.level").as_i64() {
        Some(level @ 0...14) => level as u8,
        _ => return,
    };
    let frequency = match PREFS.get("js.mem.gc.zeal.frequency").as_i64() {
        Some(frequency) if frequency >= 0 => frequency as u32,
        _ => JS_DEFAULT_ZEAL_FREQ,
    };
    JS_SetGCZeal(rt, level, frequency);
}

#[allow(unsafe_code)]
#[cfg(not(feature = "debugmozjs"))]
unsafe fn set_gc_zeal_options(_: *mut JSRuntime) {}

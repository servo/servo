/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! The script runtime contains common traits and structs commonly used by the
//! script thread, the dom, and the worker threads.

#![allow(dead_code)]

use core::ffi::c_char;
use std::cell::Cell;
use std::ffi::CString;
use std::io::{Write, stdout};
use std::ops::Deref;
use std::os::raw::c_void;
use std::rc::Rc;
use std::sync::Mutex;
use std::time::{Duration, Instant};
use std::{os, ptr, thread};

use background_hang_monitor_api::ScriptHangAnnotation;
use content_security_policy::CheckResult;
use js::conversions::jsstr_to_string;
use js::glue::{
    CollectServoSizes, CreateJobQueue, DeleteJobQueue, DispatchableRun, JobQueueTraps,
    RUST_js_GetErrorMessage, SetBuildId, StreamConsumerConsumeChunk,
    StreamConsumerNoteResponseURLs, StreamConsumerStreamEnd, StreamConsumerStreamError,
};
use js::jsapi::{
    AsmJSOption, BuildIdCharVector, ContextOptionsRef, DisableIncrementalGC,
    Dispatchable as JSRunnable, Dispatchable_MaybeShuttingDown, GCDescription, GCOptions,
    GCProgress, GCReason, GetPromiseUserInputEventHandlingState, HandleObject, HandleString, Heap,
    InitConsumeStreamCallback, InitDispatchToEventLoop, JS_AddExtraGCRootsTracer,
    JS_InitDestroyPrincipalsCallback, JS_InitReadPrincipalsCallback, JS_SetGCCallback,
    JS_SetGCParameter, JS_SetGlobalJitCompilerOption, JS_SetOffthreadIonCompilationEnabled,
    JS_SetParallelParsingEnabled, JS_SetSecurityCallbacks, JSContext as RawJSContext, JSGCParamKey,
    JSGCStatus, JSJitCompilerOption, JSObject, JSSecurityCallbacks, JSTracer, JobQueue, MimeType,
    PromiseRejectionHandlingState, PromiseUserInputEventHandlingState, RuntimeCode,
    SetDOMCallbacks, SetGCSliceCallback, SetJobQueue, SetPreserveWrapperCallbacks,
    SetProcessBuildIdOp, SetPromiseRejectionTrackerCallback, StreamConsumer as JSStreamConsumer,
};
use js::jsval::UndefinedValue;
use js::panic::wrap_panic;
pub(crate) use js::rust::ThreadSafeJSContext;
use js::rust::wrappers::{GetPromiseIsHandled, JS_GetPromiseResult};
use js::rust::{
    Handle, HandleObject as RustHandleObject, IntoHandle, JSEngine, JSEngineHandle, ParentRuntime,
    Runtime as RustRuntime,
};
use malloc_size_of::MallocSizeOfOps;
use malloc_size_of_derive::MallocSizeOf;
use profile_traits::mem::{Report, ReportKind};
use profile_traits::path;
use profile_traits::time::ProfilerCategory;
use script_bindings::script_runtime::{mark_runtime_dead, runtime_is_alive};
use servo_config::{opts, pref};
use style::thread_state::{self, ThreadState};

use crate::body::BodyMixin;
use crate::dom::bindings::codegen::Bindings::PromiseBinding::PromiseJobCallback;
use crate::dom::bindings::codegen::Bindings::ResponseBinding::Response_Binding::ResponseMethods;
use crate::dom::bindings::codegen::Bindings::ResponseBinding::ResponseType as DOMResponseType;
use crate::dom::bindings::conversions::{
    get_dom_class, private_from_object, root_from_handleobject,
};
use crate::dom::bindings::error::{Error, throw_dom_exception};
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::refcounted::{
    LiveDOMReferences, Trusted, TrustedPromise, trace_refcounted_objects,
};
use crate::dom::bindings::reflector::{DomGlobal, DomObject};
use crate::dom::bindings::root::trace_roots;
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
use crate::task_source::SendableTaskSource;

static JOB_QUEUE_TRAPS: JobQueueTraps = JobQueueTraps {
    getIncumbentGlobal: Some(get_incumbent_global),
    enqueuePromiseJob: Some(enqueue_promise_job),
    empty: Some(empty),
};

static SECURITY_CALLBACKS: JSSecurityCallbacks = JSSecurityCallbacks {
    contentSecurityPolicyAllows: Some(content_security_policy_allows),
    subsumes: Some(principals::subsumes),
};

#[derive(Clone, Copy, Debug, Eq, Hash, JSTraceable, MallocSizeOf, PartialEq)]
pub(crate) enum ScriptThreadEventCategory {
    AttachLayout,
    ConstellationMsg,
    DatabaseAccessEvent,
    DevtoolsMsg,
    DocumentEvent,
    FileRead,
    FontLoading,
    FormPlannedNavigation,
    HistoryEvent,
    ImageCacheMsg,
    InputEvent,
    NetworkEvent,
    PortMessage,
    Rendering,
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
    #[cfg(feature = "webgpu")]
    WebGPUMsg,
}

impl From<ScriptThreadEventCategory> for ProfilerCategory {
    fn from(category: ScriptThreadEventCategory) -> Self {
        match category {
            ScriptThreadEventCategory::AttachLayout => ProfilerCategory::ScriptAttachLayout,
            ScriptThreadEventCategory::ConstellationMsg => ProfilerCategory::ScriptConstellationMsg,
            ScriptThreadEventCategory::DatabaseAccessEvent => {
                ProfilerCategory::ScriptDatabaseAccessEvent
            },
            ScriptThreadEventCategory::DevtoolsMsg => ProfilerCategory::ScriptDevtoolsMsg,
            ScriptThreadEventCategory::DocumentEvent => ProfilerCategory::ScriptDocumentEvent,
            ScriptThreadEventCategory::EnterFullscreen => ProfilerCategory::ScriptEnterFullscreen,
            ScriptThreadEventCategory::ExitFullscreen => ProfilerCategory::ScriptExitFullscreen,
            ScriptThreadEventCategory::FileRead => ProfilerCategory::ScriptFileRead,
            ScriptThreadEventCategory::FontLoading => ProfilerCategory::ScriptFontLoading,
            ScriptThreadEventCategory::FormPlannedNavigation => {
                ProfilerCategory::ScriptPlannedNavigation
            },
            ScriptThreadEventCategory::HistoryEvent => ProfilerCategory::ScriptHistoryEvent,
            ScriptThreadEventCategory::ImageCacheMsg => ProfilerCategory::ScriptImageCacheMsg,
            ScriptThreadEventCategory::InputEvent => ProfilerCategory::ScriptInputEvent,
            ScriptThreadEventCategory::NetworkEvent => ProfilerCategory::ScriptNetworkEvent,
            ScriptThreadEventCategory::PerformanceTimelineTask => {
                ProfilerCategory::ScriptPerformanceEvent
            },
            ScriptThreadEventCategory::PortMessage => ProfilerCategory::ScriptPortMessage,
            ScriptThreadEventCategory::Resize => ProfilerCategory::ScriptResize,
            ScriptThreadEventCategory::Rendering => ProfilerCategory::ScriptRendering,
            ScriptThreadEventCategory::ScriptEvent => ProfilerCategory::ScriptEvent,
            ScriptThreadEventCategory::ServiceWorkerEvent => {
                ProfilerCategory::ScriptServiceWorkerEvent
            },
            ScriptThreadEventCategory::SetScrollState => ProfilerCategory::ScriptSetScrollState,
            ScriptThreadEventCategory::SetViewport => ProfilerCategory::ScriptSetViewport,
            ScriptThreadEventCategory::StylesheetLoad => ProfilerCategory::ScriptStylesheetLoad,
            ScriptThreadEventCategory::TimerEvent => ProfilerCategory::ScriptTimerEvent,
            ScriptThreadEventCategory::UpdateReplacedElement => {
                ProfilerCategory::ScriptUpdateReplacedElement
            },
            ScriptThreadEventCategory::WebSocketEvent => ProfilerCategory::ScriptWebSocketEvent,
            ScriptThreadEventCategory::WorkerEvent => ProfilerCategory::ScriptWorkerEvent,
            ScriptThreadEventCategory::WorkletEvent => ProfilerCategory::ScriptWorkletEvent,
            #[cfg(feature = "webgpu")]
            ScriptThreadEventCategory::WebGPUMsg => ProfilerCategory::ScriptWebGPUMsg,
        }
    }
}

impl From<ScriptThreadEventCategory> for ScriptHangAnnotation {
    fn from(category: ScriptThreadEventCategory) -> Self {
        match category {
            ScriptThreadEventCategory::AttachLayout => ScriptHangAnnotation::AttachLayout,
            ScriptThreadEventCategory::ConstellationMsg => ScriptHangAnnotation::ConstellationMsg,
            ScriptThreadEventCategory::DatabaseAccessEvent => {
                ScriptHangAnnotation::DatabaseAccessEvent
            },
            ScriptThreadEventCategory::DevtoolsMsg => ScriptHangAnnotation::DevtoolsMsg,
            ScriptThreadEventCategory::DocumentEvent => ScriptHangAnnotation::DocumentEvent,
            ScriptThreadEventCategory::InputEvent => ScriptHangAnnotation::InputEvent,
            ScriptThreadEventCategory::FileRead => ScriptHangAnnotation::FileRead,
            ScriptThreadEventCategory::FontLoading => ScriptHangAnnotation::FontLoading,
            ScriptThreadEventCategory::FormPlannedNavigation => {
                ScriptHangAnnotation::FormPlannedNavigation
            },
            ScriptThreadEventCategory::HistoryEvent => ScriptHangAnnotation::HistoryEvent,
            ScriptThreadEventCategory::ImageCacheMsg => ScriptHangAnnotation::ImageCacheMsg,
            ScriptThreadEventCategory::NetworkEvent => ScriptHangAnnotation::NetworkEvent,
            ScriptThreadEventCategory::Rendering => ScriptHangAnnotation::Rendering,
            ScriptThreadEventCategory::Resize => ScriptHangAnnotation::Resize,
            ScriptThreadEventCategory::ScriptEvent => ScriptHangAnnotation::ScriptEvent,
            ScriptThreadEventCategory::SetScrollState => ScriptHangAnnotation::SetScrollState,
            ScriptThreadEventCategory::SetViewport => ScriptHangAnnotation::SetViewport,
            ScriptThreadEventCategory::StylesheetLoad => ScriptHangAnnotation::StylesheetLoad,
            ScriptThreadEventCategory::TimerEvent => ScriptHangAnnotation::TimerEvent,
            ScriptThreadEventCategory::UpdateReplacedElement => {
                ScriptHangAnnotation::UpdateReplacedElement
            },
            ScriptThreadEventCategory::WebSocketEvent => ScriptHangAnnotation::WebSocketEvent,
            ScriptThreadEventCategory::WorkerEvent => ScriptHangAnnotation::WorkerEvent,
            ScriptThreadEventCategory::WorkletEvent => ScriptHangAnnotation::WorkletEvent,
            ScriptThreadEventCategory::ServiceWorkerEvent => {
                ScriptHangAnnotation::ServiceWorkerEvent
            },
            ScriptThreadEventCategory::EnterFullscreen => ScriptHangAnnotation::EnterFullscreen,
            ScriptThreadEventCategory::ExitFullscreen => ScriptHangAnnotation::ExitFullscreen,
            ScriptThreadEventCategory::PerformanceTimelineTask => {
                ScriptHangAnnotation::PerformanceTimelineTask
            },
            ScriptThreadEventCategory::PortMessage => ScriptHangAnnotation::PortMessage,
            #[cfg(feature = "webgpu")]
            ScriptThreadEventCategory::WebGPUMsg => ScriptHangAnnotation::WebGPUMsg,
        }
    }
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
            GlobalScope::from_context(*cx, InRealm::already(&realm))
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

#[allow(unsafe_code)]
#[cfg_attr(crown, allow(crown::unrooted_must_root))]
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
                global.task_manager().dom_manipulation_task_source().queue(
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
                        reason.handle(),
                        CanGc::note()
                    );

                    event.upcast::<Event>().fire(&target, CanGc::note());
                })
            );
            },
        };
    })
}

#[allow(unsafe_code)]
unsafe extern "C" fn content_security_policy_allows(
    cx: *mut RawJSContext,
    runtime_code: RuntimeCode,
    sample: HandleString,
) -> bool {
    let mut allowed = false;
    let cx = JSContext::from_ptr(cx);
    wrap_panic(&mut || {
        // SpiderMonkey provides null pointer when executing webassembly.
        let in_realm_proof = AlreadyInRealm::assert_for_cx(cx);
        let global = GlobalScope::from_context(*cx, InRealm::Already(&in_realm_proof));
        let Some(csp_list) = global.get_csp_list() else {
            allowed = true;
            return;
        };

        let (is_evaluation_allowed, violations) = match runtime_code {
            RuntimeCode::JS => {
                let source = match sample {
                    sample if !sample.is_null() => &jsstr_to_string(*cx, *sample),
                    _ => "",
                };
                csp_list.is_js_evaluation_allowed(source)
            },
            RuntimeCode::WASM => csp_list.is_wasm_evaluation_allowed(),
        };

        global.report_csp_violations(violations, None);
        allowed = is_evaluation_allowed == CheckResult::Allowed;
    });
    allowed
}

#[allow(unsafe_code)]
#[cfg_attr(crown, allow(crown::unrooted_must_root))]
/// <https://html.spec.whatwg.org/multipage/#notify-about-rejected-promises>
pub(crate) fn notify_about_rejected_promises(global: &GlobalScope) {
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
            global.task_manager().dom_manipulation_task_source().queue(
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
                            reason.handle(),
                            CanGc::note()
                        );

                        let event_status = event.upcast::<Event>().fire(&target, CanGc::note());

                        // Step 4-3.
                        if event_status == EventStatus::Canceled {
                            // TODO: The promise rejection is not handled; we need to add it back to the list.
                        }

                        // Step 4-4.
                        if !promise_is_handled {
                            target.global().add_consumed_rejection(promise.reflector().get_jsobject().into_handle());
                        }
                    }
                })
            );
        }
    }
}

#[derive(JSTraceable)]
pub(crate) struct Runtime {
    rt: RustRuntime,
    pub(crate) microtask_queue: Rc<MicrotaskQueue>,
    job_queue: *mut JobQueue,
    networking_task_src: Option<Box<SendableTaskSource>>,
}

impl Runtime {
    /// Create a new runtime, optionally with the given [`SendableTaskSource`] for networking.
    ///
    /// # Safety
    ///
    /// If panicking does not abort the program, any threads with child runtimes will continue
    /// executing after the thread with the parent runtime panics, but they will be in an
    /// invalid and undefined state.
    ///
    /// This, like many calls to SpiderMoney API, is unsafe.
    #[allow(unsafe_code)]
    pub(crate) fn new(networking_task_source: Option<SendableTaskSource>) -> Runtime {
        unsafe { Self::new_with_parent(None, networking_task_source) }
    }

    /// Create a new runtime, optionally with the given [`ParentRuntime`] and [`SendableTaskSource`]
    /// for networking.
    ///
    /// # Safety
    ///
    /// If panicking does not abort the program, any threads with child runtimes will continue
    /// executing after the thread with the parent runtime panics, but they will be in an
    /// invalid and undefined state.
    ///
    /// The `parent` pointer in the [`ParentRuntime`] argument must point to a valid object in memory.
    ///
    /// This, like many calls to the SpiderMoney API, is unsafe.
    #[allow(unsafe_code)]
    pub(crate) unsafe fn new_with_parent(
        parent: Option<ParentRuntime>,
        networking_task_source: Option<SendableTaskSource>,
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
            let networking_task_src: &SendableTaskSource = &*(closure as *mut SendableTaskSource);
            let runnable = Runnable(dispatchable);
            let task = task!(dispatch_to_event_loop_message: move || {
                if let Some(cx) = RustRuntime::get() {
                    runnable.run(cx.as_ptr(), Dispatchable_MaybeShuttingDown::NotShuttingDown);
                }
            });

            networking_task_src.queue_unconditionally(task);
            true
        }

        let mut networking_task_src_ptr = std::ptr::null_mut();
        if let Some(source) = networking_task_source {
            networking_task_src_ptr = Box::into_raw(Box::new(source));
            InitDispatchToEventLoop(
                cx,
                Some(dispatch_to_event_loop),
                networking_task_src_ptr as *mut c_void,
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
            pref!(js_baseline_interpreter_enabled) as u32,
        );
        JS_SetGlobalJitCompilerOption(
            cx,
            JSJitCompilerOption::JSJITCOMPILER_BASELINE_ENABLE,
            pref!(js_baseline_jit_enabled) as u32,
        );
        JS_SetGlobalJitCompilerOption(
            cx,
            JSJitCompilerOption::JSJITCOMPILER_ION_ENABLE,
            pref!(js_ion_enabled) as u32,
        );
        cx_opts.compileOptions_.asmJSOption_ = if pref!(js_asmjs_enabled) {
            AsmJSOption::Enabled
        } else {
            AsmJSOption::DisabledByAsmJSPref
        };
        let wasm_enabled = pref!(js_wasm_enabled);
        cx_opts.set_wasm_(wasm_enabled);
        if wasm_enabled {
            // If WASM is enabled without setting the buildIdOp,
            // initializing a module will report an out of memory error.
            // https://dxr.mozilla.org/mozilla-central/source/js/src/wasm/WasmTypes.cpp#458
            SetProcessBuildIdOp(Some(servo_build_id));
        }
        cx_opts.set_wasmBaseline_(pref!(js_wasm_baseline_enabled));
        cx_opts.set_wasmIon_(pref!(js_wasm_ion_enabled));
        // TODO: handle js.throw_on_asmjs_validation_failure (needs new Spidermonkey)
        JS_SetGlobalJitCompilerOption(
            cx,
            JSJitCompilerOption::JSJITCOMPILER_NATIVE_REGEXP_ENABLE,
            pref!(js_native_regex_enabled) as u32,
        );
        JS_SetParallelParsingEnabled(cx, pref!(js_parallel_parsing_enabled));
        JS_SetOffthreadIonCompilationEnabled(cx, pref!(js_offthread_compilation_enabled));
        JS_SetGlobalJitCompilerOption(
            cx,
            JSJitCompilerOption::JSJITCOMPILER_BASELINE_WARMUP_TRIGGER,
            if pref!(js_baseline_jit_unsafe_eager_compilation_enabled) {
                0
            } else {
                u32::MAX
            },
        );
        JS_SetGlobalJitCompilerOption(
            cx,
            JSJitCompilerOption::JSJITCOMPILER_ION_NORMAL_WARMUP_TRIGGER,
            if pref!(js_ion_unsafe_eager_compilation_enabled) {
                0
            } else {
                u32::MAX
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
            in_range(pref!(js_mem_max), 1, 0x100)
                .map(|val| (val * 1024 * 1024) as u32)
                .unwrap_or(u32::MAX),
        );
        // NOTE: This is disabled above, so enabling it here will do nothing for now.
        JS_SetGCParameter(
            cx,
            JSGCParamKey::JSGC_INCREMENTAL_GC_ENABLED,
            pref!(js_mem_gc_incremental_enabled) as u32,
        );
        JS_SetGCParameter(
            cx,
            JSGCParamKey::JSGC_PER_ZONE_GC_ENABLED,
            pref!(js_mem_gc_per_zone_enabled) as u32,
        );
        if let Some(val) = in_range(pref!(js_mem_gc_incremental_slice_ms), 0, 100_000) {
            JS_SetGCParameter(cx, JSGCParamKey::JSGC_SLICE_TIME_BUDGET_MS, val as u32);
        }
        JS_SetGCParameter(
            cx,
            JSGCParamKey::JSGC_COMPACTING_ENABLED,
            pref!(js_mem_gc_compacting_enabled) as u32,
        );

        if let Some(val) = in_range(pref!(js_mem_gc_high_frequency_time_limit_ms), 0, 10_000) {
            JS_SetGCParameter(cx, JSGCParamKey::JSGC_HIGH_FREQUENCY_TIME_LIMIT, val as u32);
        }
        if let Some(val) = in_range(pref!(js_mem_gc_low_frequency_heap_growth), 0, 10_000) {
            JS_SetGCParameter(cx, JSGCParamKey::JSGC_LOW_FREQUENCY_HEAP_GROWTH, val as u32);
        }
        if let Some(val) = in_range(pref!(js_mem_gc_high_frequency_heap_growth_min), 0, 10_000) {
            JS_SetGCParameter(
                cx,
                JSGCParamKey::JSGC_HIGH_FREQUENCY_LARGE_HEAP_GROWTH,
                val as u32,
            );
        }
        if let Some(val) = in_range(pref!(js_mem_gc_high_frequency_heap_growth_max), 0, 10_000) {
            JS_SetGCParameter(
                cx,
                JSGCParamKey::JSGC_HIGH_FREQUENCY_SMALL_HEAP_GROWTH,
                val as u32,
            );
        }
        if let Some(val) = in_range(pref!(js_mem_gc_high_frequency_low_limit_mb), 0, 10_000) {
            JS_SetGCParameter(cx, JSGCParamKey::JSGC_SMALL_HEAP_SIZE_MAX, val as u32);
        }
        if let Some(val) = in_range(pref!(js_mem_gc_high_frequency_high_limit_mb), 0, 10_000) {
            JS_SetGCParameter(cx, JSGCParamKey::JSGC_LARGE_HEAP_SIZE_MIN, val as u32);
        }
        /*if let Some(val) = in_range(pref!(js_mem_gc_allocation_threshold_factor), 0, 10_000) {
            JS_SetGCParameter(cx, JSGCParamKey::JSGC_NON_INCREMENTAL_FACTOR, val as u32);
        }*/
        /*
            // JSGC_SMALL_HEAP_INCREMENTAL_LIMIT
            pref("javascript.options.mem.gc_small_heap_incremental_limit", 140);

            // JSGC_LARGE_HEAP_INCREMENTAL_LIMIT
            pref("javascript.options.mem.gc_large_heap_incremental_limit", 110);
        */
        if let Some(val) = in_range(pref!(js_mem_gc_empty_chunk_count_min), 0, 10_000) {
            JS_SetGCParameter(cx, JSGCParamKey::JSGC_MIN_EMPTY_CHUNK_COUNT, val as u32);
        }
        if let Some(val) = in_range(pref!(js_mem_gc_empty_chunk_count_max), 0, 10_000) {
            JS_SetGCParameter(cx, JSGCParamKey::JSGC_MAX_EMPTY_CHUNK_COUNT, val as u32);
        }

        Runtime {
            rt: runtime,
            microtask_queue,
            job_queue,
            networking_task_src: (!networking_task_src_ptr.is_null())
                .then(|| Box::from_raw(networking_task_src_ptr)),
        }
    }

    pub(crate) fn thread_safe_js_context(&self) -> ThreadSafeJSContext {
        self.rt.thread_safe_js_context()
    }
}

impl Drop for Runtime {
    #[allow(unsafe_code)]
    fn drop(&mut self) {
        self.microtask_queue.clear();

        unsafe {
            DeleteJobQueue(self.job_queue);
        }
        LiveDOMReferences::destruct();
        mark_runtime_dead();
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

static JS_ENGINE: Mutex<Option<JSEngineHandle>> = Mutex::new(None);

fn in_range<T: PartialOrd + Copy>(val: T, min: T, max: T) -> Option<T> {
    if val < min || val >= max {
        None
    } else {
        Some(val)
    }
}

thread_local!(static MALLOC_SIZE_OF_OPS: Cell<*mut MallocSizeOfOps> = const { Cell::new(ptr::null_mut()) });

#[allow(unsafe_code)]
unsafe extern "C" fn get_size(obj: *mut JSObject) -> usize {
    match get_dom_class(obj) {
        Ok(v) => {
            let dom_object = private_from_object(obj) as *const c_void;

            if dom_object.is_null() {
                return 0;
            }
            let ops = MALLOC_SIZE_OF_OPS.get();
            (v.malloc_size_of)(&mut *ops, dom_object)
        },
        Err(_e) => 0,
    }
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

#[allow(unsafe_code)]
unsafe extern "C" fn trace_rust_roots(tr: *mut JSTracer, _data: *mut os::raw::c_void) {
    if !runtime_is_alive() {
        return;
    }
    trace!("starting custom root handler");
    trace_thread(tr);
    trace_roots(tr);
    trace_refcounted_objects(tr);
    settings_stack::trace(tr);
    trace!("done custom root handler");
}

#[allow(unsafe_code)]
unsafe extern "C" fn servo_build_id(build_id: *mut BuildIdCharVector) -> bool {
    let servo_id = b"Servo\0";
    SetBuildId(build_id, servo_id[0] as *const c_char, servo_id.len())
}

#[allow(unsafe_code)]
#[cfg(feature = "debugmozjs")]
unsafe fn set_gc_zeal_options(cx: *mut RawJSContext) {
    use js::jsapi::SetGCZeal;

    let level = match pref!(js_mem_gc_zeal_level) {
        level @ 0..=14 => level as u8,
        _ => return,
    };
    let frequency = match pref!(js_mem_gc_zeal_frequency) {
        frequency if frequency >= 0 => frequency as u32,
        // https://searchfox.org/mozilla-esr128/source/js/public/GCAPI.h#1392
        _ => 5000,
    };
    SetGCZeal(cx, level, frequency);
}

#[allow(unsafe_code)]
#[cfg(not(feature = "debugmozjs"))]
unsafe fn set_gc_zeal_options(_: *mut RawJSContext) {}

pub(crate) use script_bindings::script_runtime::JSContext;

/// Extra methods for the JSContext type defined in script_bindings, when
/// the methods are only called by code in the script crate.
pub(crate) trait JSContextHelper {
    fn get_reports(&self, path_seg: String, ops: &mut MallocSizeOfOps) -> Vec<Report>;
}

impl JSContextHelper for JSContext {
    #[allow(unsafe_code)]
    fn get_reports(&self, path_seg: String, ops: &mut MallocSizeOfOps) -> Vec<Report> {
        MALLOC_SIZE_OF_OPS.with(|ops_tls| ops_tls.set(ops));
        let stats = unsafe {
            let mut stats = ::std::mem::zeroed();
            if !CollectServoSizes(**self, &mut stats, Some(get_size)) {
                return vec![];
            }
            stats
        };
        MALLOC_SIZE_OF_OPS.with(|ops| ops.set(ptr::null_mut()));

        let mut reports = vec![];
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
        reports
    }
}

pub(crate) struct StreamConsumer(*mut JSStreamConsumer);

#[allow(unsafe_code)]
impl StreamConsumer {
    pub(crate) fn consume_chunk(&self, stream: &[u8]) -> bool {
        unsafe {
            let stream_ptr = stream.as_ptr();
            StreamConsumerConsumeChunk(self.0, stream_ptr, stream.len())
        }
    }

    pub(crate) fn stream_end(&self) {
        unsafe {
            StreamConsumerStreamEnd(self.0);
        }
    }

    pub(crate) fn stream_error(&self, error_code: usize) {
        unsafe {
            StreamConsumerStreamError(self.0, error_code);
        }
    }

    pub(crate) fn note_response_urls(
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
        let mimetype = unwrapped_source.Headers(CanGc::note()).extract_mime_type();

        //Step 2.3 If mimeType is not `application/wasm`, return with a TypeError and abort these substeps.
        if !&mimetype[..].eq_ignore_ascii_case(b"application/wasm") {
            throw_dom_exception(
                cx,
                &global,
                Error::Type("Response has unsupported MIME type".to_string()),
                CanGc::note(),
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
                    CanGc::note(),
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
                CanGc::note(),
            );
            return false;
        }

        // Step 2.6.1 If response body is locked, return with a TypeError and abort these substeps.
        if unwrapped_source.is_locked() {
            throw_dom_exception(
                cx,
                &global,
                Error::Type("There was an error consuming the Response".to_string()),
                CanGc::note(),
            );
            return false;
        }

        // Step 2.6.2 If response body is alreaady consumed, return with a TypeError and abort these substeps.
        if unwrapped_source.is_disturbed() {
            throw_dom_exception(
                cx,
                &global,
                Error::Type("Response already consumed".to_string()),
                CanGc::note(),
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
            CanGc::note(),
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

pub(crate) struct Runnable(*mut JSRunnable);

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

pub(crate) use script_bindings::script_runtime::CanGc;

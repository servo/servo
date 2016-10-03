/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Abstractions for global scopes.
//!
//! This module contains smart pointers to global scopes, to simplify writing
//! code that works in workers as well as window scopes.

use devtools_traits::{ScriptToDevtoolsControlMsg, WorkerId};
use dom::bindings::codegen::Bindings::WindowBinding::WindowMethods;
use dom::bindings::conversions::root_from_object;
use dom::bindings::error::{ErrorInfo, report_pending_exception};
use dom::bindings::js::Root;
use dom::bindings::reflector::{Reflectable, Reflector};
use dom::console::TimerSet;
use dom::window;
use dom::workerglobalscope::WorkerGlobalScope;
use ipc_channel::ipc::IpcSender;
use js::{JSCLASS_IS_DOMJSCLASS, JSCLASS_IS_GLOBAL};
use js::glue::{IsWrapper, UnwrapObject};
use js::jsapi::{CurrentGlobalOrNull, Evaluate2, GetGlobalForObjectCrossCompartment};
use js::jsapi::{HandleValue, JS_GetClass, JSAutoCompartment, JSContext};
use js::jsapi::{JSObject, MutableHandleValue};
use js::rust::CompileOptionsWrapper;
use libc;
use msg::constellation_msg::PipelineId;
use net_traits::{CoreResourceThread, IpcSend, ResourceThreads};
use profile_traits::{mem, time};
use script_runtime::{CommonScriptMsg, EnqueuedPromiseCallback, ScriptChan};
use script_runtime::{ScriptPort, maybe_take_panic_result};
use script_thread::{MainThreadScriptChan, RunnableWrapper, ScriptThread};
use script_traits::{MsDuration, ScriptMsg as ConstellationMsg, TimerEventRequest};
use std::ffi::CString;
use std::panic;
use task_source::file_reading::FileReadingTaskSource;
use timers::{OneshotTimerCallback, OneshotTimerHandle};
use url::Url;

/// A freely-copyable reference to a rooted global object.
#[derive(Copy, Clone)]
pub enum GlobalRef<'a> {
    /// A reference to a `Window` object.
    Window(&'a window::Window),
    /// A reference to a `WorkerGlobalScope` object.
    Worker(&'a WorkerGlobalScope),
}

/// A stack-based rooted reference to a global object.
pub enum GlobalRoot {
    /// A root for a `Window` object.
    Window(Root<window::Window>),
    /// A root for a `WorkerGlobalScope` object.
    Worker(Root<WorkerGlobalScope>),
}

impl<'a> GlobalRef<'a> {
    /// Get the `JSContext` for the `JSRuntime` associated with the thread
    /// this global object is on.
    pub fn get_cx(&self) -> *mut JSContext {
        match *self {
            GlobalRef::Window(ref window) => window.get_cx(),
            GlobalRef::Worker(ref worker) => worker.get_cx(),
        }
    }

    /// Extract a `Window`, causing thread failure if the global object is not
    /// a `Window`.
    pub fn as_window(&self) -> &window::Window {
        match *self {
            GlobalRef::Window(window) => window,
            GlobalRef::Worker(_) => panic!("expected a Window scope"),
        }
    }

    /// Get the `PipelineId` for this global scope.
    pub fn pipeline_id(&self) -> PipelineId {
        match *self {
            GlobalRef::Window(window) => window.pipeline_id(),
            GlobalRef::Worker(worker) => worker.pipeline_id(),
        }
    }

    /// Get a `mem::ProfilerChan` to send messages to the memory profiler thread.
    pub fn mem_profiler_chan(&self) -> &mem::ProfilerChan {
        match *self {
            GlobalRef::Window(window) => window.mem_profiler_chan(),
            GlobalRef::Worker(worker) => worker.mem_profiler_chan(),
        }
    }

    /// Get a `time::ProfilerChan` to send messages to the time profiler thread.
    pub fn time_profiler_chan(&self) -> &time::ProfilerChan {
        match *self {
            GlobalRef::Window(window) => window.time_profiler_chan(),
            GlobalRef::Worker(worker) => worker.time_profiler_chan(),
        }
    }

    /// Get a `IpcSender` to send messages to the constellation when available.
    pub fn constellation_chan(&self) -> &IpcSender<ConstellationMsg> {
        match *self {
            GlobalRef::Window(window) => window.constellation_chan(),
            GlobalRef::Worker(worker) => worker.constellation_chan(),
        }
    }

    /// Get the scheduler channel to request timer events.
    pub fn scheduler_chan(&self) -> &IpcSender<TimerEventRequest> {
        match *self {
            GlobalRef::Window(window) => window.scheduler_chan(),
            GlobalRef::Worker(worker) => worker.scheduler_chan(),
        }
    }

    /// Get an `IpcSender<ScriptToDevtoolsControlMsg>` to send messages to Devtools
    /// thread when available.
    pub fn devtools_chan(&self) -> Option<IpcSender<ScriptToDevtoolsControlMsg>> {
        match *self {
            GlobalRef::Window(window) => window.devtools_chan(),
            GlobalRef::Worker(worker) => worker.devtools_chan(),
        }
    }

    /// Get the `ResourceThreads` for this global scope.
    pub fn resource_threads(&self) -> ResourceThreads {
        match *self {
            GlobalRef::Window(ref window) => {
                let doc = window.Document();
                let doc = doc.r();
                let loader = doc.loader();
                loader.resource_threads().clone()
            }
            GlobalRef::Worker(ref worker) => worker.resource_threads().clone(),
        }
    }

    /// Get the `CoreResourceThread` for this global scope
    pub fn core_resource_thread(&self) -> CoreResourceThread {
        self.resource_threads().sender()
    }

    /// Get the worker's id.
    pub fn get_worker_id(&self) -> Option<WorkerId> {
        match *self {
            GlobalRef::Window(_) => None,
            GlobalRef::Worker(ref worker) => Some(worker.get_worker_id()),
        }
    }

    /// Get next worker id.
    pub fn get_next_worker_id(&self) -> WorkerId {
        match *self {
            GlobalRef::Window(ref window) => window.get_next_worker_id(),
            GlobalRef::Worker(ref worker) => worker.get_next_worker_id(),
        }
    }

    /// Get the URL for this global scope.
    pub fn get_url(&self) -> Url {
        match *self {
            GlobalRef::Window(ref window) => window.get_url(),
            GlobalRef::Worker(ref worker) => worker.get_url().clone(),
        }
    }

    /// Get the [base url](https://html.spec.whatwg.org/multipage/#api-base-url)
    /// for this global scope.
    pub fn api_base_url(&self) -> Url {
        match *self {
            // https://html.spec.whatwg.org/multipage/#script-settings-for-browsing-contexts:api-base-url
            GlobalRef::Window(ref window) => window.Document().base_url(),
            // https://html.spec.whatwg.org/multipage/#script-settings-for-workers:api-base-url
            GlobalRef::Worker(ref worker) => worker.get_url().clone(),
        }
    }

    /// `ScriptChan` used to send messages to the event loop of this global's
    /// thread.
    pub fn script_chan(&self) -> Box<ScriptChan + Send> {
        match *self {
            GlobalRef::Window(ref window) =>
                MainThreadScriptChan(window.main_thread_script_chan().clone()).clone(),
            GlobalRef::Worker(ref worker) => worker.script_chan(),
        }
    }

    /// `ScriptChan` used to send messages to the event loop of this global's
    /// thread.
    pub fn networking_task_source(&self) -> Box<ScriptChan + Send> {
        match *self {
            GlobalRef::Window(ref window) => window.networking_task_source(),
            GlobalRef::Worker(ref worker) => worker.script_chan(),
        }
    }

    /// `ScriptChan` used to send messages to the event loop of this global's
    /// thread.
    pub fn file_reading_task_source(&self) -> FileReadingTaskSource {
        match *self {
            GlobalRef::Window(ref window) => window.file_reading_task_source(),
            GlobalRef::Worker(ref worker) => worker.file_reading_task_source(),
        }
    }

    /// Create a new sender/receiver pair that can be used to implement an on-demand
    /// event loop. Used for implementing web APIs that require blocking semantics
    /// without resorting to nested event loops.
    pub fn new_script_pair(&self) -> (Box<ScriptChan + Send>, Box<ScriptPort + Send>) {
        match *self {
            GlobalRef::Window(ref window) => window.new_script_pair(),
            GlobalRef::Worker(ref worker) => worker.new_script_pair(),
        }
    }

    /// Process a single event as if it were the next event in the thread queue for
    /// this global.
    pub fn process_event(&self, msg: CommonScriptMsg) {
        match *self {
            GlobalRef::Window(_) => ScriptThread::process_event(msg),
            GlobalRef::Worker(ref worker) => worker.process_event(msg),
        }
    }

    /// Evaluate JS code on this global.
    pub fn evaluate_js_on_global_with_result(
            &self, code: &str, rval: MutableHandleValue) {
        self.evaluate_script_on_global_with_result(code, "", rval)
    }

    /// Evaluate a JS script on this global.
    #[allow(unsafe_code)]
    pub fn evaluate_script_on_global_with_result(
            &self, code: &str, filename: &str, rval: MutableHandleValue) {
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
                let options = CompileOptionsWrapper::new(cx, filename.as_ptr(), 1);
                unsafe {
                    if !Evaluate2(cx, options.ptr, code.as_ptr(),
                                  code.len() as libc::size_t,
                                  rval) {
                        debug!("error evaluating JS string");
                        report_pending_exception(cx, true);
                    }
                }

                if let Some(error) = maybe_take_panic_result() {
                    panic::resume_unwind(error);
                }
            }
        )
    }

    /// Set the `bool` value to indicate whether developer tools has requested
    /// updates from the global
    pub fn set_devtools_wants_updates(&self, send_updates: bool) {
        match *self {
            GlobalRef::Window(window) => window.set_devtools_wants_updates(send_updates),
            GlobalRef::Worker(worker) => worker.set_devtools_wants_updates(send_updates),
        }
    }

    /// Schedule the given `callback` to be invoked after at least `duration` milliseconds have
    /// passed.
    pub fn schedule_callback(&self,
                             callback: OneshotTimerCallback,
                             duration: MsDuration)
                             -> OneshotTimerHandle {
        match *self {
            GlobalRef::Window(window) => window.schedule_callback(callback, duration),
            GlobalRef::Worker(worker) => worker.schedule_callback(callback, duration),
        }
    }

    /// Unschedule a previously-scheduled callback.
    pub fn unschedule_callback(&self, handle: OneshotTimerHandle) {
        match *self {
            GlobalRef::Window(window) => window.unschedule_callback(handle),
            GlobalRef::Worker(worker) => worker.unschedule_callback(handle),
        }
    }

    /// Returns the global's timers for the Console API.
    pub fn console_timers(&self) -> &TimerSet {
        match *self {
            GlobalRef::Window(ref window) => window.console_timers(),
            GlobalRef::Worker(ref worker) => worker.console_timers(),
        }
    }

    /// Returns a wrapper for runnables to ensure they are cancelled if the global
    /// is being destroyed.
    pub fn get_runnable_wrapper(&self) -> RunnableWrapper {
        match *self {
            GlobalRef::Window(ref window) => window.get_runnable_wrapper(),
            GlobalRef::Worker(ref worker) => worker.get_runnable_wrapper(),
        }
    }

    /// Enqueue a promise callback for subsequent execution.
    pub fn enqueue_promise_job(&self, job: EnqueuedPromiseCallback) {
        match *self {
            GlobalRef::Window(_) => ScriptThread::enqueue_promise_job(job, *self),
            GlobalRef::Worker(ref worker) => worker.enqueue_promise_job(job),
        }
    }

    /// Start the process of executing the pending promise callbacks. They will be invoked
    /// in FIFO order, synchronously, at some point in the future.
    pub fn flush_promise_jobs(&self) {
        match *self {
            GlobalRef::Window(_) => ScriptThread::flush_promise_jobs(*self),
            GlobalRef::Worker(ref worker) => worker.flush_promise_jobs(),
        }
    }

    /// https://html.spec.whatwg.org/multipage/#report-the-error
    pub fn report_an_error(&self, error_info: ErrorInfo, value: HandleValue) {
        match *self {
            GlobalRef::Window(ref window) => window.report_an_error(error_info, value),
            GlobalRef::Worker(ref worker) => worker.report_an_error(error_info, value),
        }
    }
}

impl<'a> Reflectable for GlobalRef<'a> {
    fn reflector(&self) -> &Reflector {
        match *self {
            GlobalRef::Window(ref window) => window.reflector(),
            GlobalRef::Worker(ref worker) => worker.reflector(),
        }
    }
}

impl GlobalRoot {
    /// Obtain a safe reference to the global object that cannot outlive the
    /// lifetime of this root.
    pub fn r(&self) -> GlobalRef {
        match *self {
            GlobalRoot::Window(ref window) => GlobalRef::Window(window.r()),
            GlobalRoot::Worker(ref worker) => GlobalRef::Worker(worker.r()),
        }
    }
}

/// Returns the global object of the realm that the given DOM object's reflector was created in.
pub fn global_root_from_reflector<T: Reflectable>(reflector: &T) -> GlobalRoot {
    unsafe { global_root_from_object(*reflector.reflector().get_jsobject()) }
}

/// Returns the Rust global object from a JS global object.
#[allow(unrooted_must_root)]
unsafe fn global_root_from_global(global: *mut JSObject) -> GlobalRoot {
    assert!(!global.is_null());
    let clasp = JS_GetClass(global);
    assert!(((*clasp).flags & (JSCLASS_IS_DOMJSCLASS | JSCLASS_IS_GLOBAL)) != 0);
    match root_from_object(global) {
        Ok(window) => return GlobalRoot::Window(window),
        Err(_) => (),
    }

    match root_from_object(global) {
        Ok(worker) => return GlobalRoot::Worker(worker),
        Err(_) => (),
    }

    panic!("found DOM global that doesn't unwrap to Window or WorkerGlobalScope")
}

/// Returns the global object of the realm that the given JS object was created in.
#[allow(unrooted_must_root)]
pub unsafe fn global_root_from_object(obj: *mut JSObject) -> GlobalRoot {
    assert!(!obj.is_null());
    let global = GetGlobalForObjectCrossCompartment(obj);
    global_root_from_global(global)
}

/// Returns the global object for the given JSContext
#[allow(unrooted_must_root)]
pub unsafe fn global_root_from_context(cx: *mut JSContext) -> GlobalRoot {
    let global = CurrentGlobalOrNull(cx);
    global_root_from_global(global)
}

/// Returns the global object of the realm that the given JS object was created in,
/// after unwrapping any wrappers.
pub unsafe fn global_root_from_object_maybe_wrapped(mut obj: *mut JSObject) -> GlobalRoot {
    if IsWrapper(obj) {
        obj = UnwrapObject(obj, /* stopAtWindowProxy = */ 0);
        assert!(!obj.is_null());
    }
    global_root_from_object(obj)
}

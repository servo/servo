/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use devtools_traits::{DevtoolScriptControlMsg, ScriptToDevtoolsControlMsg, WorkerId};
use dom::bindings::codegen::Bindings::EventHandlerBinding::OnErrorEventHandlerNonNull;
use dom::bindings::codegen::Bindings::FunctionBinding::Function;
use dom::bindings::codegen::Bindings::RequestBinding::RequestInit;
use dom::bindings::codegen::Bindings::WorkerGlobalScopeBinding::WorkerGlobalScopeMethods;
use dom::bindings::codegen::UnionTypes::RequestOrUSVString;
use dom::bindings::error::{Error, ErrorInfo, ErrorResult, Fallible, report_pending_exception};
use dom::bindings::global::{GlobalRef, GlobalRoot};
use dom::bindings::inheritance::Castable;
use dom::bindings::js::{JS, MutNullableHeap, Root};
use dom::bindings::refcounted::Trusted;
use dom::bindings::reflector::Reflectable;
use dom::bindings::str::DOMString;
use dom::console::TimerSet;
use dom::crypto::Crypto;
use dom::dedicatedworkerglobalscope::DedicatedWorkerGlobalScope;
use dom::errorevent::ErrorEvent;
use dom::event::{Event, EventBubbles, EventCancelable};
use dom::eventdispatcher::EventStatus;
use dom::eventtarget::EventTarget;
use dom::promise::Promise;
use dom::serviceworkerglobalscope::ServiceWorkerGlobalScope;
use dom::window::{base64_atob, base64_btoa};
use dom::workerlocation::WorkerLocation;
use dom::workernavigator::WorkerNavigator;
use fetch;
use ipc_channel::ipc::IpcSender;
use js::jsapi::{HandleValue, JSAutoCompartment, JSContext, JSRuntime};
use js::jsval::UndefinedValue;
use js::rust::Runtime;
use msg::constellation_msg::{PipelineId, ReferrerPolicy};
use net_traits::{IpcSend, LoadOrigin};
use net_traits::{LoadContext, ResourceThreads, load_whole_resource};
use profile_traits::{mem, time};
use script_runtime::{CommonScriptMsg, ScriptChan, ScriptPort, maybe_take_panic_result};
use script_runtime::{ScriptThreadEventCategory, PromiseJobQueue, EnqueuedPromiseCallback};
use script_thread::{Runnable, RunnableWrapper};
use script_traits::{MsDuration, TimerEvent, TimerEventId, TimerEventRequest, TimerSource};
use script_traits::ScriptMsg as ConstellationMsg;
use script_traits::WorkerGlobalScopeInit;
use std::cell::Cell;
use std::default::Default;
use std::panic;
use std::rc::Rc;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::Receiver;
use task_source::file_reading::FileReadingTaskSource;
use timers::{IsInterval, OneshotTimerCallback, OneshotTimerHandle, OneshotTimers, TimerCallback};
use url::Url;

#[derive(Copy, Clone, PartialEq)]
pub enum WorkerGlobalScopeTypeId {
    DedicatedWorkerGlobalScope,
}

pub fn prepare_workerscope_init(global: GlobalRef,
                                devtools_sender: Option<IpcSender<DevtoolScriptControlMsg>>) -> WorkerGlobalScopeInit {
    let worker_id = global.get_next_worker_id();
    let init = WorkerGlobalScopeInit {
            resource_threads: global.resource_threads(),
            mem_profiler_chan: global.mem_profiler_chan().clone(),
            to_devtools_sender: global.devtools_chan(),
            time_profiler_chan: global.time_profiler_chan().clone(),
            from_devtools_sender: devtools_sender,
            constellation_chan: global.constellation_chan().clone(),
            scheduler_chan: global.scheduler_chan().clone(),
            worker_id: worker_id,
            pipeline_id: global.pipeline_id(),
        };

    init
}

// https://html.spec.whatwg.org/multipage/#the-workerglobalscope-common-interface
#[dom_struct]
pub struct WorkerGlobalScope {
    eventtarget: EventTarget,
    worker_id: WorkerId,
    pipeline_id: PipelineId,
    worker_url: Url,
    #[ignore_heap_size_of = "Arc"]
    closing: Option<Arc<AtomicBool>>,
    #[ignore_heap_size_of = "Defined in js"]
    runtime: Runtime,
    next_worker_id: Cell<WorkerId>,
    #[ignore_heap_size_of = "Defined in std"]
    resource_threads: ResourceThreads,
    location: MutNullableHeap<JS<WorkerLocation>>,
    navigator: MutNullableHeap<JS<WorkerNavigator>>,
    crypto: MutNullableHeap<JS<Crypto>>,
    timers: OneshotTimers,

    #[ignore_heap_size_of = "Defined in std"]
    mem_profiler_chan: mem::ProfilerChan,
    #[ignore_heap_size_of = "Defined in std"]
    time_profiler_chan: time::ProfilerChan,
    #[ignore_heap_size_of = "Defined in ipc-channel"]
    to_devtools_sender: Option<IpcSender<ScriptToDevtoolsControlMsg>>,

    #[ignore_heap_size_of = "Defined in ipc-channel"]
    /// Optional `IpcSender` for sending the `DevtoolScriptControlMsg`
    /// to the server from within the worker
    from_devtools_sender: Option<IpcSender<DevtoolScriptControlMsg>>,

    #[ignore_heap_size_of = "Defined in std"]
    /// This `Receiver` will be ignored later if the corresponding
    /// `IpcSender` doesn't exist
    from_devtools_receiver: Receiver<DevtoolScriptControlMsg>,

    /// A flag to indicate whether the developer tools has requested live updates
    /// from the worker
    devtools_wants_updates: Cell<bool>,

    #[ignore_heap_size_of = "Defined in std"]
    constellation_chan: IpcSender<ConstellationMsg>,

    #[ignore_heap_size_of = "Defined in std"]
    scheduler_chan: IpcSender<TimerEventRequest>,

    /// Timers used by the Console API.
    console_timers: TimerSet,

    promise_job_queue: PromiseJobQueue,

    /// https://html.spec.whatwg.org/multipage/#in-error-reporting-mode
    in_error_reporting_mode: Cell<bool>
}

impl WorkerGlobalScope {
    pub fn new_inherited(init: WorkerGlobalScopeInit,
                         worker_url: Url,
                         runtime: Runtime,
                         from_devtools_receiver: Receiver<DevtoolScriptControlMsg>,
                         timer_event_chan: IpcSender<TimerEvent>,
                         closing: Option<Arc<AtomicBool>>)
                         -> WorkerGlobalScope {
        WorkerGlobalScope {
            eventtarget: EventTarget::new_inherited(),
            next_worker_id: Cell::new(WorkerId(0)),
            worker_id: init.worker_id,
            pipeline_id: init.pipeline_id,
            worker_url: worker_url,
            closing: closing,
            runtime: runtime,
            resource_threads: init.resource_threads,
            location: Default::default(),
            navigator: Default::default(),
            crypto: Default::default(),
            timers: OneshotTimers::new(timer_event_chan, init.scheduler_chan.clone()),
            mem_profiler_chan: init.mem_profiler_chan,
            time_profiler_chan: init.time_profiler_chan,
            to_devtools_sender: init.to_devtools_sender,
            from_devtools_sender: init.from_devtools_sender,
            from_devtools_receiver: from_devtools_receiver,
            devtools_wants_updates: Cell::new(false),
            constellation_chan: init.constellation_chan,
            scheduler_chan: init.scheduler_chan,
            console_timers: TimerSet::new(),
            promise_job_queue: PromiseJobQueue::new(),
            in_error_reporting_mode: Default::default(),
        }
    }

    pub fn console_timers(&self) -> &TimerSet {
        &self.console_timers
    }

    pub fn mem_profiler_chan(&self) -> &mem::ProfilerChan {
        &self.mem_profiler_chan
    }

    pub fn time_profiler_chan(&self) -> &time::ProfilerChan {
        &self.time_profiler_chan
    }

    pub fn devtools_chan(&self) -> Option<IpcSender<ScriptToDevtoolsControlMsg>> {
        self.to_devtools_sender.clone()
    }

    pub fn from_devtools_sender(&self) -> Option<IpcSender<DevtoolScriptControlMsg>> {
        self.from_devtools_sender.clone()
    }

    pub fn from_devtools_receiver(&self) -> &Receiver<DevtoolScriptControlMsg> {
        &self.from_devtools_receiver
    }

    pub fn constellation_chan(&self) -> &IpcSender<ConstellationMsg> {
        &self.constellation_chan
    }

    pub fn scheduler_chan(&self) -> &IpcSender<TimerEventRequest> {
        &self.scheduler_chan
    }

    pub fn schedule_callback(&self, callback: OneshotTimerCallback, duration: MsDuration) -> OneshotTimerHandle {
        self.timers.schedule_callback(callback,
                                      duration,
                                      TimerSource::FromWorker)
    }

    pub fn unschedule_callback(&self, handle: OneshotTimerHandle) {
        self.timers.unschedule_callback(handle);
    }

    pub fn runtime(&self) -> *mut JSRuntime {
        self.runtime.rt()
    }

    pub fn get_cx(&self) -> *mut JSContext {
        self.runtime.cx()
    }

    pub fn is_closing(&self) -> bool {
        if let Some(ref closing) = self.closing {
            closing.load(Ordering::SeqCst)
        } else {
            false
        }
    }

    pub fn resource_threads(&self) -> &ResourceThreads {
        &self.resource_threads
    }

    pub fn get_url(&self) -> &Url {
        &self.worker_url
    }

    pub fn get_worker_id(&self) -> WorkerId {
        self.worker_id.clone()
    }

    pub fn get_next_worker_id(&self) -> WorkerId {
        let worker_id = self.next_worker_id.get();
        let WorkerId(id_num) = worker_id;
        self.next_worker_id.set(WorkerId(id_num + 1));
        worker_id
    }

    pub fn get_runnable_wrapper(&self) -> RunnableWrapper {
        RunnableWrapper {
            cancelled: self.closing.clone().unwrap(),
        }
    }

    pub fn enqueue_promise_job(&self, job: EnqueuedPromiseCallback) {
        self.promise_job_queue.enqueue(job, GlobalRef::Worker(self));
    }

    pub fn flush_promise_jobs(&self) {
        self.script_chan().send(CommonScriptMsg::RunnableMsg(
            ScriptThreadEventCategory::WorkerEvent,
            box FlushPromiseJobs {
                global: Trusted::new(self),
            })).unwrap();
    }

    fn do_flush_promise_jobs(&self) {
        self.promise_job_queue.flush_promise_jobs(|id| {
            assert_eq!(self.pipeline_id(), id);
            Some(GlobalRoot::Worker(Root::from_ref(self)))
        });
    }
}

impl LoadOrigin for WorkerGlobalScope {
    fn referrer_url(&self) -> Option<Url> {
        None
    }
    fn referrer_policy(&self) -> Option<ReferrerPolicy> {
        None
    }
    fn pipeline_id(&self) -> Option<PipelineId> {
        Some(self.pipeline_id())
    }
}

impl WorkerGlobalScopeMethods for WorkerGlobalScope {
    // https://html.spec.whatwg.org/multipage/#dom-workerglobalscope-self
    fn Self_(&self) -> Root<WorkerGlobalScope> {
        Root::from_ref(self)
    }

    // https://html.spec.whatwg.org/multipage/#dom-workerglobalscope-location
    fn Location(&self) -> Root<WorkerLocation> {
        self.location.or_init(|| {
            WorkerLocation::new(self, self.worker_url.clone())
        })
    }

    // https://html.spec.whatwg.org/multipage/#handler-workerglobalscope-onerror
    error_event_handler!(error, GetOnerror, SetOnerror);

    // https://html.spec.whatwg.org/multipage/#dom-workerglobalscope-importscripts
    fn ImportScripts(&self, url_strings: Vec<DOMString>) -> ErrorResult {
        let mut urls = Vec::with_capacity(url_strings.len());
        for url in url_strings {
            let url = self.worker_url.join(&url);
            match url {
                Ok(url) => urls.push(url),
                Err(_) => return Err(Error::Syntax),
            };
        }

        rooted!(in(self.runtime.cx()) let mut rval = UndefinedValue());
        for url in urls {
            let (url, source) = match load_whole_resource(LoadContext::Script,
                                                          &self.resource_threads.sender(),
                                                          url,
                                                          self) {
                Err(_) => return Err(Error::Network),
                Ok((metadata, bytes)) => {
                    (metadata.final_url, String::from_utf8(bytes).unwrap())
                }
            };

            let result = self.runtime.evaluate_script(
                self.reflector().get_jsobject(), &source, url.as_str(), 1, rval.handle_mut());

            if let Some(error) = maybe_take_panic_result() {
                panic::resume_unwind(error);
            }

            match result {
                Ok(_) => (),
                Err(_) => {
                    println!("evaluate_script failed");
                    return Err(Error::JSFailed);
                }
            }
        }

        Ok(())
    }

    // https://html.spec.whatwg.org/multipage/#dom-worker-navigator
    fn Navigator(&self) -> Root<WorkerNavigator> {
        self.navigator.or_init(|| WorkerNavigator::new(self))
    }

    // https://html.spec.whatwg.org/multipage/#dfn-Crypto
    fn Crypto(&self) -> Root<Crypto> {
        self.crypto.or_init(|| Crypto::new(GlobalRef::Worker(self)))
    }

    // https://html.spec.whatwg.org/multipage/#dom-windowbase64-btoa
    fn Btoa(&self, btoa: DOMString) -> Fallible<DOMString> {
        base64_btoa(btoa)
    }

    // https://html.spec.whatwg.org/multipage/#dom-windowbase64-atob
    fn Atob(&self, atob: DOMString) -> Fallible<DOMString> {
        base64_atob(atob)
    }

    // https://html.spec.whatwg.org/multipage/#dom-windowtimers-setinterval
    fn SetTimeout(&self, _cx: *mut JSContext, callback: Rc<Function>, timeout: i32, args: Vec<HandleValue>) -> i32 {
        self.timers.set_timeout_or_interval(GlobalRef::Worker(self),
                                            TimerCallback::FunctionTimerCallback(callback),
                                            args,
                                            timeout,
                                            IsInterval::NonInterval,
                                            TimerSource::FromWorker)
    }

    // https://html.spec.whatwg.org/multipage/#dom-windowtimers-setinterval
    fn SetTimeout_(&self, _cx: *mut JSContext, callback: DOMString, timeout: i32, args: Vec<HandleValue>) -> i32 {
        self.timers.set_timeout_or_interval(GlobalRef::Worker(self),
                                            TimerCallback::StringTimerCallback(callback),
                                            args,
                                            timeout,
                                            IsInterval::NonInterval,
                                            TimerSource::FromWorker)
    }

    // https://html.spec.whatwg.org/multipage/#dom-windowtimers-clearinterval
    fn ClearTimeout(&self, handle: i32) {
        self.timers.clear_timeout_or_interval(GlobalRef::Worker(self), handle);
    }

    // https://html.spec.whatwg.org/multipage/#dom-windowtimers-setinterval
    fn SetInterval(&self, _cx: *mut JSContext, callback: Rc<Function>, timeout: i32, args: Vec<HandleValue>) -> i32 {
        self.timers.set_timeout_or_interval(GlobalRef::Worker(self),
                                            TimerCallback::FunctionTimerCallback(callback),
                                            args,
                                            timeout,
                                            IsInterval::Interval,
                                            TimerSource::FromWorker)
    }

    // https://html.spec.whatwg.org/multipage/#dom-windowtimers-setinterval
    fn SetInterval_(&self, _cx: *mut JSContext, callback: DOMString, timeout: i32, args: Vec<HandleValue>) -> i32 {
        self.timers.set_timeout_or_interval(GlobalRef::Worker(self),
                                            TimerCallback::StringTimerCallback(callback),
                                            args,
                                            timeout,
                                            IsInterval::Interval,
                                            TimerSource::FromWorker)
    }

    // https://html.spec.whatwg.org/multipage/#dom-windowtimers-clearinterval
    fn ClearInterval(&self, handle: i32) {
        self.ClearTimeout(handle);
    }

    #[allow(unrooted_must_root)]
    // https://fetch.spec.whatwg.org/#fetch-method
    fn Fetch(&self, input: RequestOrUSVString, init: &RequestInit) -> Rc<Promise> {
        fetch::Fetch(self.global().r(), input, init)
    }
}


impl WorkerGlobalScope {
    #[allow(unsafe_code)]
    pub fn execute_script(&self, source: DOMString) {
        rooted!(in(self.runtime.cx()) let mut rval = UndefinedValue());
        match self.runtime.evaluate_script(
            self.reflector().get_jsobject(), &source, self.worker_url.as_str(), 1, rval.handle_mut()) {
            Ok(_) => (),
            Err(_) => {
                if self.is_closing() {
                    println!("evaluate_script failed (terminated)");
                } else {
                    // TODO: An error needs to be dispatched to the parent.
                    // https://github.com/servo/servo/issues/6422
                    println!("evaluate_script failed");
                    unsafe {
                        let _ac = JSAutoCompartment::new(self.runtime.cx(),
                                                         self.reflector().get_jsobject().get());
                        report_pending_exception(self.runtime.cx(), true);
                    }
                }
            }
        }
    }

    pub fn script_chan(&self) -> Box<ScriptChan + Send> {
        let dedicated = self.downcast::<DedicatedWorkerGlobalScope>();
        let service_worker = self.downcast::<ServiceWorkerGlobalScope>();
        if let Some(dedicated) = dedicated {
            return dedicated.script_chan();
        } else if let Some(service_worker) = service_worker {
            return service_worker.script_chan();
        } else {
            panic!("need to implement a sender for SharedWorker")
        }
    }

    pub fn file_reading_task_source(&self) -> FileReadingTaskSource {
        FileReadingTaskSource(self.script_chan())
    }

    pub fn pipeline_id(&self) -> PipelineId {
        self.pipeline_id
    }

    pub fn new_script_pair(&self) -> (Box<ScriptChan + Send>, Box<ScriptPort + Send>) {
        let dedicated = self.downcast::<DedicatedWorkerGlobalScope>();
        if let Some(dedicated) = dedicated {
            return dedicated.new_script_pair();
        } else {
            panic!("need to implement a sender for SharedWorker/ServiceWorker")
        }
    }

    pub fn process_event(&self, msg: CommonScriptMsg) {
        let dedicated = self.downcast::<DedicatedWorkerGlobalScope>();
        let service_worker = self.downcast::<ServiceWorkerGlobalScope>();
        if let Some(dedicated) = dedicated {
            return dedicated.process_event(msg);
        } else if let Some(service_worker) = service_worker {
            return service_worker.process_event(msg);
        } else {
            panic!("need to implement a sender for SharedWorker")
        }
    }

    pub fn handle_fire_timer(&self, timer_id: TimerEventId) {
        self.timers.fire_timer(timer_id, self);
    }

    pub fn set_devtools_wants_updates(&self, value: bool) {
        self.devtools_wants_updates.set(value);
    }

    pub fn close(&self) {
        if let Some(ref closing) = self.closing {
            closing.store(true, Ordering::SeqCst);
        }
    }

    /// https://html.spec.whatwg.org/multipage/#report-the-error
    pub fn report_an_error(&self, error_info: ErrorInfo, value: HandleValue) {
        // Step 1.
        if self.in_error_reporting_mode.get() {
            return;
        }

        // Step 2.
        self.in_error_reporting_mode.set(true);

        // Steps 3-12.
        // FIXME(#13195): muted errors.
        let event = ErrorEvent::new(GlobalRef::Worker(self),
                                    atom!("error"),
                                    EventBubbles::DoesNotBubble,
                                    EventCancelable::Cancelable,
                                    error_info.message.as_str().into(),
                                    error_info.filename.as_str().into(),
                                    error_info.lineno,
                                    error_info.column,
                                    value);

        // Step 13.
        let event_status = event.upcast::<Event>().fire(self.upcast::<EventTarget>());

        // Step 15
        if event_status == EventStatus::NotCanceled {
            if let Some(dedicated) = self.downcast::<DedicatedWorkerGlobalScope>() {
                dedicated.forward_error_to_worker_object(error_info);
            }
        }

        // Step 14
        self.in_error_reporting_mode.set(false);
    }
}

struct FlushPromiseJobs {
    global: Trusted<WorkerGlobalScope>,
}

impl Runnable for FlushPromiseJobs {
    fn handler(self: Box<FlushPromiseJobs>) {
        let global = self.global.root();
        global.do_flush_promise_jobs();
    }
}

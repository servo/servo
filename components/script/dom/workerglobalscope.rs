/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use devtools_traits::{DevtoolScriptControlMsg, WorkerId};
use dom::bindings::codegen::Bindings::EventHandlerBinding::OnErrorEventHandlerNonNull;
use dom::bindings::codegen::Bindings::FunctionBinding::Function;
use dom::bindings::codegen::Bindings::RequestBinding::RequestInit;
use dom::bindings::codegen::Bindings::WorkerGlobalScopeBinding::WorkerGlobalScopeMethods;
use dom::bindings::codegen::UnionTypes::RequestOrUSVString;
use dom::bindings::error::{Error, ErrorResult, Fallible, report_pending_exception};
use dom::bindings::inheritance::Castable;
use dom::bindings::js::{JS, MutNullableHeap, Root};
use dom::bindings::refcounted::Trusted;
use dom::bindings::reflector::Reflectable;
use dom::bindings::str::DOMString;
use dom::crypto::Crypto;
use dom::dedicatedworkerglobalscope::DedicatedWorkerGlobalScope;
use dom::globalscope::GlobalScope;
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
use net_traits::{IpcSend, load_whole_resource};
use net_traits::request::{CredentialsMode, Destination, RequestInit as NetRequestInit, Type as RequestType};
use script_runtime::{CommonScriptMsg, ScriptChan, ScriptPort, maybe_take_panic_result};
use script_runtime::{ScriptThreadEventCategory, PromiseJobQueue, EnqueuedPromiseCallback};
use script_thread::{Runnable, RunnableWrapper};
use script_traits::{TimerEvent, TimerEventId};
use script_traits::WorkerGlobalScopeInit;
use std::default::Default;
use std::panic;
use std::rc::Rc;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::Receiver;
use task_source::file_reading::FileReadingTaskSource;
use task_source::networking::NetworkingTaskSource;
use timers::{IsInterval, TimerCallback};
use url::Url;

pub fn prepare_workerscope_init(global: &GlobalScope,
                                devtools_sender: Option<IpcSender<DevtoolScriptControlMsg>>) -> WorkerGlobalScopeInit {
    let init = WorkerGlobalScopeInit {
            resource_threads: global.resource_threads().clone(),
            mem_profiler_chan: global.mem_profiler_chan().clone(),
            to_devtools_sender: global.devtools_chan().cloned(),
            time_profiler_chan: global.time_profiler_chan().clone(),
            from_devtools_sender: devtools_sender,
            constellation_chan: global.constellation_chan().clone(),
            scheduler_chan: global.scheduler_chan().clone(),
            worker_id: global.get_next_worker_id(),
            pipeline_id: global.pipeline_id(),
        };

    init
}

// https://html.spec.whatwg.org/multipage/#the-workerglobalscope-common-interface
#[dom_struct]
pub struct WorkerGlobalScope {
    globalscope: GlobalScope,

    worker_id: WorkerId,
    worker_url: Url,
    #[ignore_heap_size_of = "Arc"]
    closing: Option<Arc<AtomicBool>>,
    #[ignore_heap_size_of = "Defined in js"]
    runtime: Runtime,
    location: MutNullableHeap<JS<WorkerLocation>>,
    navigator: MutNullableHeap<JS<WorkerNavigator>>,

    #[ignore_heap_size_of = "Defined in ipc-channel"]
    /// Optional `IpcSender` for sending the `DevtoolScriptControlMsg`
    /// to the server from within the worker
    from_devtools_sender: Option<IpcSender<DevtoolScriptControlMsg>>,

    #[ignore_heap_size_of = "Defined in std"]
    /// This `Receiver` will be ignored later if the corresponding
    /// `IpcSender` doesn't exist
    from_devtools_receiver: Receiver<DevtoolScriptControlMsg>,

    promise_job_queue: PromiseJobQueue,
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
            globalscope:
                GlobalScope::new_inherited(
                    init.pipeline_id,
                    init.to_devtools_sender,
                    init.mem_profiler_chan,
                    init.time_profiler_chan,
                    init.constellation_chan,
                    init.scheduler_chan,
                    init.resource_threads,
                    timer_event_chan),
            worker_id: init.worker_id,
            worker_url: worker_url,
            closing: closing,
            runtime: runtime,
            location: Default::default(),
            navigator: Default::default(),
            from_devtools_sender: init.from_devtools_sender,
            from_devtools_receiver: from_devtools_receiver,
            promise_job_queue: PromiseJobQueue::new(),
        }
    }

    pub fn from_devtools_sender(&self) -> Option<IpcSender<DevtoolScriptControlMsg>> {
        self.from_devtools_sender.clone()
    }

    pub fn from_devtools_receiver(&self) -> &Receiver<DevtoolScriptControlMsg> {
        &self.from_devtools_receiver
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

    pub fn get_url(&self) -> &Url {
        &self.worker_url
    }

    pub fn get_worker_id(&self) -> WorkerId {
        self.worker_id.clone()
    }

    pub fn get_runnable_wrapper(&self) -> RunnableWrapper {
        RunnableWrapper {
            cancelled: self.closing.clone(),
        }
    }

    pub fn enqueue_promise_job(&self, job: EnqueuedPromiseCallback) {
        self.promise_job_queue.enqueue(job, self.upcast());
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
            let global = self.upcast::<GlobalScope>();
            assert_eq!(global.pipeline_id(), id);
            Some(Root::from_ref(global))
        });
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
            let global_scope = self.upcast::<GlobalScope>();
            let request = NetRequestInit {
                url: url.clone(),
                type_: RequestType::Script,
                destination: Destination::Script,
                credentials_mode: CredentialsMode::Include,
                use_url_credentials: true,
                origin: self.worker_url.clone(),
                pipeline_id: Some(self.upcast::<GlobalScope>().pipeline_id()),
                referrer_url: None,
                referrer_policy: None,
                .. NetRequestInit::default()
            };
            let (url, source) = match load_whole_resource(request,
                                                          &global_scope.resource_threads().sender()) {
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
        self.upcast::<GlobalScope>().crypto()
    }

    // https://html.spec.whatwg.org/multipage/#dom-windowbase64-btoa
    fn Btoa(&self, btoa: DOMString) -> Fallible<DOMString> {
        base64_btoa(btoa)
    }

    // https://html.spec.whatwg.org/multipage/#dom-windowbase64-atob
    fn Atob(&self, atob: DOMString) -> Fallible<DOMString> {
        base64_atob(atob)
    }

    #[allow(unsafe_code)]
    // https://html.spec.whatwg.org/multipage/#dom-windowtimers-settimeout
    unsafe fn SetTimeout(&self, _cx: *mut JSContext, callback: Rc<Function>,
                         timeout: i32, args: Vec<HandleValue>) -> i32 {
        self.upcast::<GlobalScope>().set_timeout_or_interval(
            TimerCallback::FunctionTimerCallback(callback),
            args,
            timeout,
            IsInterval::NonInterval)
    }

    #[allow(unsafe_code)]
    // https://html.spec.whatwg.org/multipage/#dom-windowtimers-settimeout
    unsafe fn SetTimeout_(&self, _cx: *mut JSContext, callback: DOMString,
                          timeout: i32, args: Vec<HandleValue>) -> i32 {
        self.upcast::<GlobalScope>().set_timeout_or_interval(
            TimerCallback::StringTimerCallback(callback),
            args,
            timeout,
            IsInterval::NonInterval)
    }

    // https://html.spec.whatwg.org/multipage/#dom-windowtimers-cleartimeout
    fn ClearTimeout(&self, handle: i32) {
        self.upcast::<GlobalScope>().clear_timeout_or_interval(handle);
    }

    #[allow(unsafe_code)]
    // https://html.spec.whatwg.org/multipage/#dom-windowtimers-setinterval
    unsafe fn SetInterval(&self, _cx: *mut JSContext, callback: Rc<Function>,
                          timeout: i32, args: Vec<HandleValue>) -> i32 {
        self.upcast::<GlobalScope>().set_timeout_or_interval(
            TimerCallback::FunctionTimerCallback(callback),
            args,
            timeout,
            IsInterval::Interval)
    }

    #[allow(unsafe_code)]
    // https://html.spec.whatwg.org/multipage/#dom-windowtimers-setinterval
    unsafe fn SetInterval_(&self, _cx: *mut JSContext, callback: DOMString,
                           timeout: i32, args: Vec<HandleValue>) -> i32 {
        self.upcast::<GlobalScope>().set_timeout_or_interval(
            TimerCallback::StringTimerCallback(callback),
            args,
            timeout,
            IsInterval::Interval)
    }

    // https://html.spec.whatwg.org/multipage/#dom-windowtimers-clearinterval
    fn ClearInterval(&self, handle: i32) {
        self.ClearTimeout(handle);
    }

    #[allow(unrooted_must_root)]
    // https://fetch.spec.whatwg.org/#fetch-method
    fn Fetch(&self, input: RequestOrUSVString, init: &RequestInit) -> Rc<Promise> {
        fetch::Fetch(self.upcast(), input, init)
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

    pub fn networking_task_source(&self) -> NetworkingTaskSource {
        NetworkingTaskSource(self.script_chan())
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
        self.upcast::<GlobalScope>().fire_timer(timer_id);
    }

    pub fn close(&self) {
        if let Some(ref closing) = self.closing {
            closing.store(true, Ordering::SeqCst);
        }
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

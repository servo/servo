/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use devtools_traits::{DevtoolScriptControlMsg, ScriptToDevtoolsControlMsg};
use dom::bindings::codegen::Bindings::FunctionBinding::Function;
use dom::bindings::codegen::Bindings::WorkerGlobalScopeBinding::WorkerGlobalScopeMethods;
use dom::bindings::conversions::Castable;
use dom::bindings::error::{Error, ErrorResult, Fallible, report_pending_exception};
use dom::bindings::global::GlobalRef;
use dom::bindings::js::{JS, MutNullableHeap, Root};
use dom::bindings::utils::Reflectable;
use dom::console::Console;
use dom::crypto::Crypto;
use dom::dedicatedworkerglobalscope::DedicatedWorkerGlobalScope;
use dom::eventtarget::EventTarget;
use dom::window::{base64_atob, base64_btoa};
use dom::workerlocation::WorkerLocation;
use dom::workernavigator::WorkerNavigator;
use ipc_channel::ipc::IpcSender;
use js::jsapi::{HandleValue, JSAutoRequest, JSContext};
use js::rust::Runtime;
use msg::constellation_msg::{ConstellationChan, PipelineId, WorkerId};
use net_traits::{ResourceTask, load_whole_resource};
use profile_traits::mem;
use script_task::{CommonScriptMsg, ScriptChan, ScriptPort};
use script_traits::{TimerEvent, TimerEventId, TimerEventRequest, TimerSource};
use std::cell::Cell;
use std::default::Default;
use std::rc::Rc;
use std::sync::mpsc::Receiver;
use timers::{ActiveTimers, IsInterval, TimerCallback};
use url::{Url, UrlParser};
use util::str::DOMString;

#[derive(Copy, Clone, PartialEq)]
pub enum WorkerGlobalScopeTypeId {
    DedicatedWorkerGlobalScope,
}

pub struct WorkerGlobalScopeInit {
    pub resource_task: ResourceTask,
    pub mem_profiler_chan: mem::ProfilerChan,
    pub to_devtools_sender: Option<IpcSender<ScriptToDevtoolsControlMsg>>,
    pub from_devtools_sender: Option<IpcSender<DevtoolScriptControlMsg>>,
    pub constellation_chan: ConstellationChan,
    pub scheduler_chan: IpcSender<TimerEventRequest>,
    pub worker_id: WorkerId,
}

// https://html.spec.whatwg.org/multipage/#the-workerglobalscope-common-interface
#[dom_struct]
pub struct WorkerGlobalScope {
    eventtarget: EventTarget,
    worker_id: WorkerId,
    worker_url: Url,
    #[ignore_heap_size_of = "Defined in std"]
    runtime: Rc<Runtime>,
    next_worker_id: Cell<WorkerId>,
    #[ignore_heap_size_of = "Defined in std"]
    resource_task: ResourceTask,
    location: MutNullableHeap<JS<WorkerLocation>>,
    navigator: MutNullableHeap<JS<WorkerNavigator>>,
    console: MutNullableHeap<JS<Console>>,
    crypto: MutNullableHeap<JS<Crypto>>,
    timers: ActiveTimers,
    #[ignore_heap_size_of = "Defined in std"]
    mem_profiler_chan: mem::ProfilerChan,
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
    constellation_chan: ConstellationChan,

    #[ignore_heap_size_of = "Defined in std"]
    scheduler_chan: IpcSender<TimerEventRequest>,
}

impl WorkerGlobalScope {
    pub fn new_inherited(init: WorkerGlobalScopeInit,
                         worker_url: Url,
                         runtime: Rc<Runtime>,
                         from_devtools_receiver: Receiver<DevtoolScriptControlMsg>,
                         timer_event_chan: IpcSender<TimerEvent>)
                         -> WorkerGlobalScope {
        WorkerGlobalScope {
            eventtarget: EventTarget::new_inherited(),
            next_worker_id: Cell::new(WorkerId(0)),
            worker_id: init.worker_id,
            worker_url: worker_url,
            runtime: runtime,
            resource_task: init.resource_task,
            location: Default::default(),
            navigator: Default::default(),
            console: Default::default(),
            crypto: Default::default(),
            timers: ActiveTimers::new(timer_event_chan, init.scheduler_chan.clone()),
            mem_profiler_chan: init.mem_profiler_chan,
            to_devtools_sender: init.to_devtools_sender,
            from_devtools_sender: init.from_devtools_sender,
            from_devtools_receiver: from_devtools_receiver,
            devtools_wants_updates: Cell::new(false),
            constellation_chan: init.constellation_chan,
            scheduler_chan: init.scheduler_chan,
        }
    }

    pub fn mem_profiler_chan(&self) -> mem::ProfilerChan {
        self.mem_profiler_chan.clone()
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

    pub fn constellation_chan(&self) -> ConstellationChan {
        self.constellation_chan.clone()
    }

    pub fn scheduler_chan(&self) -> IpcSender<TimerEventRequest> {
        self.scheduler_chan.clone()
    }

    pub fn get_cx(&self) -> *mut JSContext {
        self.runtime.cx()
    }

    pub fn resource_task(&self) -> &ResourceTask {
        &self.resource_task
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

    // https://html.spec.whatwg.org/multipage/#dom-workerglobalscope-importscripts
    fn ImportScripts(&self, url_strings: Vec<DOMString>) -> ErrorResult {
        let mut urls = Vec::with_capacity(url_strings.len());
        for url in url_strings {
            let url = UrlParser::new().base_url(&self.worker_url)
                                      .parse(&url);
            match url {
                Ok(url) => urls.push(url),
                Err(_) => return Err(Error::Syntax),
            };
        }

        for url in urls {
            let (url, source) = match load_whole_resource(&self.resource_task, url) {
                Err(_) => return Err(Error::Network),
                Ok((metadata, bytes)) => {
                    (metadata.final_url, String::from_utf8(bytes).unwrap())
                }
            };

            match self.runtime.evaluate_script(
                self.reflector().get_jsobject(), source, url.serialize(), 1) {
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

    // https://developer.mozilla.org/en-US/docs/Web/API/WorkerGlobalScope/console
    fn Console(&self) -> Root<Console> {
        self.console.or_init(|| Console::new(GlobalRef::Worker(self)))
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
        self.timers.set_timeout_or_interval(TimerCallback::FunctionTimerCallback(callback),
                                            args,
                                            timeout,
                                            IsInterval::NonInterval,
                                            TimerSource::FromWorker)
    }

    // https://html.spec.whatwg.org/multipage/#dom-windowtimers-setinterval
    fn SetTimeout_(&self, _cx: *mut JSContext, callback: DOMString, timeout: i32, args: Vec<HandleValue>) -> i32 {
        self.timers.set_timeout_or_interval(TimerCallback::StringTimerCallback(callback),
                                            args,
                                            timeout,
                                            IsInterval::NonInterval,
                                            TimerSource::FromWorker)
    }

    // https://html.spec.whatwg.org/multipage/#dom-windowtimers-clearinterval
    fn ClearTimeout(&self, handle: i32) {
        self.timers.clear_timeout_or_interval(handle);
    }

    // https://html.spec.whatwg.org/multipage/#dom-windowtimers-setinterval
    fn SetInterval(&self, _cx: *mut JSContext, callback: Rc<Function>, timeout: i32, args: Vec<HandleValue>) -> i32 {
        self.timers.set_timeout_or_interval(TimerCallback::FunctionTimerCallback(callback),
                                            args,
                                            timeout,
                                            IsInterval::Interval,
                                            TimerSource::FromWorker)
    }

    // https://html.spec.whatwg.org/multipage/#dom-windowtimers-setinterval
    fn SetInterval_(&self, _cx: *mut JSContext, callback: DOMString, timeout: i32, args: Vec<HandleValue>) -> i32 {
        self.timers.set_timeout_or_interval(TimerCallback::StringTimerCallback(callback),
                                            args,
                                            timeout,
                                            IsInterval::Interval,
                                            TimerSource::FromWorker)
    }

    // https://html.spec.whatwg.org/multipage/#dom-windowtimers-clearinterval
    fn ClearInterval(&self, handle: i32) {
        self.ClearTimeout(handle);
    }
}


impl WorkerGlobalScope {
    pub fn execute_script(&self, source: DOMString) {
        match self.runtime.evaluate_script(
            self.reflector().get_jsobject(), source, self.worker_url.serialize(), 1) {
            Ok(_) => (),
            Err(_) => {
                // TODO: An error needs to be dispatched to the parent.
                // https://github.com/servo/servo/issues/6422
                println!("evaluate_script failed");
                let _ar = JSAutoRequest::new(self.runtime.cx());
                report_pending_exception(self.runtime.cx(), self.reflector().get_jsobject().get());
            }
        }
    }

    pub fn script_chan(&self) -> Box<ScriptChan + Send> {
        let dedicated =
            self.downcast::<DedicatedWorkerGlobalScope>();
        match dedicated {
            Some(dedicated) => dedicated.script_chan(),
            None => panic!("need to implement a sender for SharedWorker"),
        }
    }

    pub fn pipeline(&self) -> PipelineId {
        let dedicated =
            self.downcast::<DedicatedWorkerGlobalScope>();
        match dedicated {
            Some(dedicated) => dedicated.pipeline(),
            None => panic!("need to add a pipeline for SharedWorker"),
        }
    }

    pub fn new_script_pair(&self) -> (Box<ScriptChan + Send>, Box<ScriptPort + Send>) {
        let dedicated =
            self.downcast::<DedicatedWorkerGlobalScope>();
        match dedicated {
            Some(dedicated) => dedicated.new_script_pair(),
            None => panic!("need to implement creating isolated event loops for SharedWorker"),
        }
    }

    pub fn process_event(&self, msg: CommonScriptMsg) {
        let dedicated =
            self.downcast::<DedicatedWorkerGlobalScope>();
        match dedicated {
            Some(dedicated) => dedicated.process_event(msg),
            None => panic!("need to implement processing single events for SharedWorker"),
        }
    }

    pub fn handle_fire_timer(&self, timer_id: TimerEventId) {
        self.timers.fire_timer(timer_id, self);
    }

    pub fn set_devtools_wants_updates(&self, value: bool) {
        self.devtools_wants_updates.set(value);
    }
}

/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::FunctionBinding::Function;
use dom::bindings::codegen::Bindings::WorkerGlobalScopeBinding::WorkerGlobalScopeMethods;
use dom::bindings::codegen::InheritTypes::DedicatedWorkerGlobalScopeCast;
use dom::bindings::error::{ErrorResult, Fallible};
use dom::bindings::error::Error::{Syntax, Network, JSFailed};
use dom::bindings::global::GlobalRef;
use dom::bindings::js::{JS, Root, MutNullableHeap};
use dom::bindings::utils::Reflectable;
use dom::console::Console;
use dom::dedicatedworkerglobalscope::DedicatedWorkerGlobalScopeHelpers;
use dom::eventtarget::{EventTarget, EventTargetTypeId};
use dom::workerlocation::WorkerLocation;
use dom::workernavigator::WorkerNavigator;
use dom::window::{base64_atob, base64_btoa};
use script_task::{ScriptChan, TimerSource, ScriptPort, ScriptMsg};
use timers::{IsInterval, TimerId, TimerManager, TimerCallback};

use devtools_traits::DevtoolsControlChan;

use msg::constellation_msg::{PipelineId, WorkerId};
use net_traits::{load_whole_resource, ResourceTask};
use util::str::DOMString;

use js::jsapi::{JSContext, HandleValue};
use js::rust::Runtime;
use url::{Url, UrlParser};

use std::default::Default;
use std::cell::Cell;
use std::rc::Rc;

#[derive(JSTraceable, Copy, Clone, PartialEq)]
pub enum WorkerGlobalScopeTypeId {
    DedicatedGlobalScope,
}

// https://html.spec.whatwg.org/multipage/#the-workerglobalscope-common-interface
#[dom_struct]
pub struct WorkerGlobalScope {
    eventtarget: EventTarget,
    worker_url: Url,
    runtime: Rc<Runtime>,
    next_worker_id: Cell<WorkerId>,
    resource_task: ResourceTask,
    location: MutNullableHeap<JS<WorkerLocation>>,
    navigator: MutNullableHeap<JS<WorkerNavigator>>,
    console: MutNullableHeap<JS<Console>>,
    timers: TimerManager,
    devtools_chan: Option<DevtoolsControlChan>,
}

impl WorkerGlobalScope {
    pub fn new_inherited(type_id: WorkerGlobalScopeTypeId,
                         worker_url: Url,
                         runtime: Rc<Runtime>,
                         resource_task: ResourceTask,
                         devtools_chan: Option<DevtoolsControlChan>) -> WorkerGlobalScope {
        WorkerGlobalScope {
            eventtarget: EventTarget::new_inherited(EventTargetTypeId::WorkerGlobalScope(type_id)),
            next_worker_id: Cell::new(WorkerId(0)),
            worker_url: worker_url,
            runtime: runtime,
            resource_task: resource_task,
            location: Default::default(),
            navigator: Default::default(),
            console: Default::default(),
            timers: TimerManager::new(),
            devtools_chan: devtools_chan,
        }
    }

    pub fn devtools_chan(&self) -> Option<DevtoolsControlChan> {
        self.devtools_chan.clone()
    }

    #[inline]
    pub fn eventtarget<'a>(&'a self) -> &'a EventTarget {
        &self.eventtarget
    }

    pub fn get_cx(&self) -> *mut JSContext {
        self.runtime.cx()
    }

    pub fn resource_task<'a>(&'a self) -> &'a ResourceTask {
        &   self.resource_task
    }

    pub fn get_url<'a>(&'a self) -> &'a Url {
        &self.worker_url
    }

    pub fn get_next_worker_id(&self) -> WorkerId {
        let worker_id = self.next_worker_id.get();
        let WorkerId(id_num) = worker_id;
        self.next_worker_id.set(WorkerId(id_num + 1));
        worker_id
    }
}

impl<'a> WorkerGlobalScopeMethods for &'a WorkerGlobalScope {
    // https://html.spec.whatwg.org/multipage/#dom-workerglobalscope-self
    fn Self_(self) -> Root<WorkerGlobalScope> {
        Root::from_ref(self)
    }

    // https://html.spec.whatwg.org/multipage/#dom-workerglobalscope-location
    fn Location(self) -> Root<WorkerLocation> {
        self.location.or_init(|| {
            WorkerLocation::new(self, self.worker_url.clone())
        })
    }

    // https://html.spec.whatwg.org/multipage/#dom-workerglobalscope-importscripts
    fn ImportScripts(self, url_strings: Vec<DOMString>) -> ErrorResult {
        let mut urls = Vec::with_capacity(url_strings.len());
        for url in url_strings.into_iter() {
            let url = UrlParser::new().base_url(&self.worker_url)
                                      .parse(&url);
            match url {
                Ok(url) => urls.push(url),
                Err(_) => return Err(Syntax),
            };
        }

        for url in urls.into_iter() {
            let (url, source) = match load_whole_resource(&self.resource_task, url) {
                Err(_) => return Err(Network),
                Ok((metadata, bytes)) => {
                    (metadata.final_url, String::from_utf8(bytes).unwrap())
                }
            };

            match self.runtime.evaluate_script(
                self.reflector().get_jsobject(), source, url.serialize(), 1) {
                Ok(_) => (),
                Err(_) => {
                    println!("evaluate_script failed");
                    return Err(JSFailed);
                }
            }
        }

        Ok(())
    }

    // https://html.spec.whatwg.org/multipage/#dom-worker-navigator
    fn Navigator(self) -> Root<WorkerNavigator> {
        self.navigator.or_init(|| WorkerNavigator::new(self))
    }

    fn Console(self) -> Root<Console> {
        self.console.or_init(|| Console::new(GlobalRef::Worker(self)))
    }

    fn Btoa(self, btoa: DOMString) -> Fallible<DOMString> {
        base64_btoa(btoa)
    }

    fn Atob(self, atob: DOMString) -> Fallible<DOMString> {
        base64_atob(atob)
    }

    fn SetTimeout(self, _cx: *mut JSContext, callback: Rc<Function>, timeout: i32, args: Vec<HandleValue>) -> i32 {
        self.timers.set_timeout_or_interval(TimerCallback::FunctionTimerCallback(callback),
                                            args,
                                            timeout,
                                            IsInterval::NonInterval,
                                            TimerSource::FromWorker,
                                            self.script_chan())
    }

    fn SetTimeout_(self, _cx: *mut JSContext, callback: DOMString, timeout: i32, args: Vec<HandleValue>) -> i32 {
        self.timers.set_timeout_or_interval(TimerCallback::StringTimerCallback(callback),
                                            args,
                                            timeout,
                                            IsInterval::NonInterval,
                                            TimerSource::FromWorker,
                                            self.script_chan())
    }

    fn ClearTimeout(self, handle: i32) {
        self.timers.clear_timeout_or_interval(handle);
    }

    fn SetInterval(self, _cx: *mut JSContext, callback: Rc<Function>, timeout: i32, args: Vec<HandleValue>) -> i32 {
        self.timers.set_timeout_or_interval(TimerCallback::FunctionTimerCallback(callback),
                                            args,
                                            timeout,
                                            IsInterval::Interval,
                                            TimerSource::FromWorker,
                                            self.script_chan())
    }

    fn SetInterval_(self, _cx: *mut JSContext, callback: DOMString, timeout: i32, args: Vec<HandleValue>) -> i32 {
        self.timers.set_timeout_or_interval(TimerCallback::StringTimerCallback(callback),
                                            args,
                                            timeout,
                                            IsInterval::Interval,
                                            TimerSource::FromWorker,
                                            self.script_chan())
    }

    fn ClearInterval(self, handle: i32) {
        self.ClearTimeout(handle);
    }
}

pub trait WorkerGlobalScopeHelpers {
    fn handle_fire_timer(self, timer_id: TimerId);
    fn script_chan(self) -> Box<ScriptChan+Send>;
    fn pipeline(self) -> PipelineId;
    fn new_script_pair(self) -> (Box<ScriptChan+Send>, Box<ScriptPort+Send>);
    fn process_event(self, msg: ScriptMsg);
    fn get_cx(self) -> *mut JSContext;
}

impl<'a> WorkerGlobalScopeHelpers for &'a WorkerGlobalScope {
    fn script_chan(self) -> Box<ScriptChan+Send> {
        let dedicated =
            DedicatedWorkerGlobalScopeCast::to_ref(self);
        match dedicated {
            Some(dedicated) => dedicated.script_chan(),
            None => panic!("need to implement a sender for SharedWorker"),
        }
    }

    fn pipeline(self) -> PipelineId {
        let dedicated =
            DedicatedWorkerGlobalScopeCast::to_ref(self);
        match dedicated {
            Some(dedicated) => dedicated.pipeline(),
            None => panic!("need to add a pipeline for SharedWorker"),
        }
    }

    fn new_script_pair(self) -> (Box<ScriptChan+Send>, Box<ScriptPort+Send>) {
        let dedicated =
            DedicatedWorkerGlobalScopeCast::to_ref(self);
        match dedicated {
            Some(dedicated) => dedicated.new_script_pair(),
            None => panic!("need to implement creating isolated event loops for SharedWorker"),
        }
    }

    fn process_event(self, msg: ScriptMsg) {
        let dedicated =
            DedicatedWorkerGlobalScopeCast::to_ref(self);
        match dedicated {
            Some(dedicated) => dedicated.process_event(msg),
            None => panic!("need to implement processing single events for SharedWorker"),
        }
    }

    fn handle_fire_timer(self, timer_id: TimerId) {
        self.timers.fire_timer(timer_id, self);
    }

    fn get_cx(self) -> *mut JSContext {
        self.runtime.cx()
    }
}


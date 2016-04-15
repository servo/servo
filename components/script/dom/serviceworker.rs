/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use devtools_traits::{DevtoolsPageInfo, ScriptToDevtoolsControlMsg};
use dom::bindings::cell::DOMRefCell;
use dom::bindings::codegen::Bindings::EventHandlerBinding::EventHandlerNonNull;
use dom::bindings::codegen::Bindings::ServiceWorkerBinding::{ServiceWorkerMethods, ServiceWorkerState, Wrap};
use dom::bindings::error::{Fallible, ErrorResult, Error};
use dom::bindings::global::GlobalRef;
use dom::bindings::inheritance::Castable;
use dom::bindings::js::Root;
use dom::bindings::refcounted::Trusted;
use dom::bindings::reflector::{Reflectable, reflect_dom_object};
use dom::bindings::str::USVString;
use dom::bindings::structuredclone::StructuredCloneData;
use dom::client::Client;
use dom::errorevent::ErrorEvent;
use dom::event::{Event, EventBubbles, EventCancelable};
use dom::eventtarget::EventTarget;
use dom::messageevent::MessageEvent;
use dom::serviceworkerglobalscope::{ServiceWorkerGlobalScope, ServiceWorkerScriptMsg};
use dom::workerglobalscope::WorkerGlobalScopeInit;
use ipc_channel::ipc;
use js::jsapi::{HandleValue, JSContext, JSRuntime, RootedValue};
use js::jsapi::{JSAutoCompartment, JS_RequestInterruptCallback};
use js::jsval::UndefinedValue;
use js::rust::Runtime;
use msg::constellation_msg::{PipelineId, ReferrerPolicy};
use net_traits::{RequestSource, LoadOrigin};
use script_thread::Runnable;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{Sender, channel};
use std::sync::{Arc, Mutex};
use url::Url;
use util::str::DOMString;

pub type TrustedServiceWorkerAddress = Trusted<ServiceWorker>;

#[dom_struct]
pub struct ServiceWorker {
    eventtarget: EventTarget,
    script_url: DOMRefCell<String>,
    state: DOMRefCell<ServiceWorkerState>,
    closing: Arc<AtomicBool>,
    #[ignore_heap_size_of = "Defined in std"]
    sender: Sender<(TrustedServiceWorkerAddress, ServiceWorkerScriptMsg)>,
    #[ignore_heap_size_of = "Defined in rust-mozjs"]
    runtime: Arc<Mutex<Option<SharedRt>>>
}

#[derive(Clone)]
pub struct WorkerScriptLoadOrigin {
    referrer_url: Option<Url>,
    referrer_policy: Option<ReferrerPolicy>,
    request_source: RequestSource,
    pipeline_id: Option<PipelineId>
}

impl LoadOrigin for WorkerScriptLoadOrigin {
    fn referrer_url(&self) -> Option<Url> {
        self.referrer_url.clone()
    }
    fn referrer_policy(&self) -> Option<ReferrerPolicy> {
        self.referrer_policy.clone()
    }
    fn request_source(&self) -> RequestSource {
        self.request_source.clone()
    }
    fn pipeline_id(&self) -> Option<PipelineId> {
        self.pipeline_id.clone()
    }
}

impl ServiceWorker {
    fn new_inherited(sender: Sender<(TrustedServiceWorkerAddress, ServiceWorkerScriptMsg)>,
                     closing: Arc<AtomicBool>,
                     script_url: &str) -> ServiceWorker {
        ServiceWorker {
            eventtarget: EventTarget::new_inherited(),
            closing: closing,
            sender: sender,
            script_url: DOMRefCell::new(String::from(script_url)),
            state: DOMRefCell::new(ServiceWorkerState::Installing),
            runtime: Arc::new(Mutex::new(None)),
        }
    }

    pub fn new(global: GlobalRef,
                closing: Arc<AtomicBool>,
                sender: Sender<(TrustedServiceWorkerAddress, ServiceWorkerScriptMsg)>,
                script_url: &str) -> Root<ServiceWorker> {
        reflect_dom_object(box ServiceWorker::new_inherited(sender, closing, script_url), global, Wrap)
    }

    pub fn handle_message(address: TrustedServiceWorkerAddress,
                          data: StructuredCloneData) {
        let service_worker = address.root();

        if service_worker.is_closing() {
            return;
        }

        let global = service_worker.r().global();
        let target = service_worker.upcast();
        let _ac = JSAutoCompartment::new(global.r().get_cx(), target.reflector().get_jsobject().get());
        let mut message = RootedValue::new(global.r().get_cx(), UndefinedValue());
        data.read(global.r(), message.handle_mut());
        MessageEvent::dispatch_jsval(target, global.r(), message.handle());
    }

    pub fn dispatch_simple_error(address: TrustedServiceWorkerAddress) {
        let service_worker = address.root();
        service_worker.upcast().fire_simple_event("error");
    }

    pub fn is_closing(&self) -> bool {
        self.closing.load(Ordering::SeqCst)
    }

    pub fn transition_state(&self, state: ServiceWorkerState) {
        *self.state.borrow_mut() = state;
    }

    pub fn handle_error_message(address: TrustedServiceWorkerAddress, message: DOMString,
                                filename: DOMString, lineno: u32, colno: u32) {
        let worker = address.root();

        if worker.is_closing() {
            return;
        }

        let global = worker.r().global();
        let error = RootedValue::new(global.r().get_cx(), UndefinedValue());
        let errorevent = ErrorEvent::new(global.r(), atom!("error"),
                                         EventBubbles::Bubbles, EventCancelable::Cancelable,
                                         message, filename, lineno, colno, error.handle());
        errorevent.upcast::<Event>().fire(worker.upcast());
    }

    pub fn init_service_worker(global: GlobalRef, service_worker_url: Url) -> Fallible<Root<ServiceWorker>> {
        let (sender, receiver) = channel();
        let closing = Arc::new(AtomicBool::new(false));
        let worker = ServiceWorker::new(global, closing.clone(), sender.clone(), service_worker_url.as_str());
        let worker_ref = Trusted::new(worker.r());
        let worker_id = global.get_next_worker_id();
        let (devtools_sender, devtools_receiver) = ipc::channel().unwrap();

        let worker_load_origin = WorkerScriptLoadOrigin {
            referrer_url: None,
            referrer_policy: None,
            request_source: global.request_source(),
            pipeline_id: Some(global.pipeline())
        };

        let optional_sender = match global.devtools_chan() {
            Some(ref chan) => {
                let pipeline_id = global.pipeline();
                let title = format!("Service Worker for {}", service_worker_url);
                let page_info = DevtoolsPageInfo {
                    title: title,
                    url: service_worker_url.clone(),
                };
                chan.send(ScriptToDevtoolsControlMsg::NewGlobal((pipeline_id, Some(worker_id)),
                                                                devtools_sender.clone(),
                                                                page_info)).unwrap();
                Some(devtools_sender)
            },
            None => None,
        };

        let init = WorkerGlobalScopeInit {
            core_resource_thread: global.core_resource_thread(),
            mem_profiler_chan: global.mem_profiler_chan().clone(),
            to_devtools_sender: global.devtools_chan(),
            time_profiler_chan: global.time_profiler_chan().clone(),
            from_devtools_sender: optional_sender,
            constellation_chan: global.constellation_chan().clone(),
            panic_chan: global.panic_chan().clone(),
            scheduler_chan: global.scheduler_chan().clone(),
            worker_id: worker_id,
            closing: closing,
        };

        // represents a service worker client
        let sw_client = Client::new(global.as_window());
        let trusted_client = Trusted::new(sw_client.r());

        ServiceWorkerGlobalScope::run_serviceworker_scope(
            init, service_worker_url, global.pipeline(), devtools_receiver, worker.runtime.clone(), worker_ref,
            global.script_chan(), sender, receiver, trusted_client, worker_load_origin);

        Ok(worker)
    }
}

impl ServiceWorkerMethods for ServiceWorker {

    // https://slightlyoff.github.io/ServiceWorker/spec/service_worker/#service-worker-postmessage-method
    fn PostMessage(&self, cx: *mut JSContext, message: HandleValue) -> ErrorResult {
        if let ServiceWorkerState::Redundant = *self.state.borrow() {
            return Err(Error::InvalidState);
        }

        let data = try!(StructuredCloneData::write(cx, message));
        let address = Trusted::new(self);
        self.sender.send((address, ServiceWorkerScriptMsg::DOMMessage(data))).unwrap();
        Ok(())
    }

    // https://slightlyoff.github.io/ServiceWorker/spec/service_worker/#service-worker-state-attribute
    fn State(&self) -> ServiceWorkerState {
        *self.state.borrow()
    }

    // https://slightlyoff.github.io/ServiceWorker/spec/service_worker/#service-worker-url-attribute
    fn ScriptURL(&self) -> USVString {
        USVString(self.script_url.borrow().clone())
    }
    // https://slightlyoff.github.io/ServiceWorker/spec/service_worker/#service-worker-container-onerror-attribute
    event_handler!(error, GetOnerror, SetOnerror);
}

pub struct ServiceWorkerMessageHandler {
    addr: TrustedServiceWorkerAddress,
    data: StructuredCloneData,
}

impl ServiceWorkerMessageHandler {
    pub fn new(addr: TrustedServiceWorkerAddress, data: StructuredCloneData) -> ServiceWorkerMessageHandler {
        ServiceWorkerMessageHandler {
            addr: addr,
            data: data,
        }
    }
}

impl Runnable for ServiceWorkerMessageHandler {
    fn handler(self: Box<ServiceWorkerMessageHandler>) {
        let this = *self;
        ServiceWorker::handle_message(this.addr, this.data);
    }
}

pub struct SimpleServiceWorkerErrorHandler {
    addr: TrustedServiceWorkerAddress,
}

impl SimpleServiceWorkerErrorHandler {
    pub fn new(addr: TrustedServiceWorkerAddress) -> SimpleServiceWorkerErrorHandler {
        SimpleServiceWorkerErrorHandler {
            addr: addr
        }
    }
}

impl Runnable for SimpleServiceWorkerErrorHandler {
    fn handler(self: Box<SimpleServiceWorkerErrorHandler>) {
        let this = *self;
        ServiceWorker::dispatch_simple_error(this.addr);
    }
}

pub struct ServiceWorkerErrorHandler {
    addr: TrustedServiceWorkerAddress,
    msg: DOMString,
    file_name: DOMString,
    line_num: u32,
    col_num: u32,
}

impl ServiceWorkerErrorHandler {
    pub fn new(addr: TrustedServiceWorkerAddress, msg: DOMString, file_name: DOMString, line_num: u32, col_num: u32)
            -> ServiceWorkerErrorHandler {
        ServiceWorkerErrorHandler {
            addr: addr,
            msg: msg,
            file_name: file_name,
            line_num: line_num,
            col_num: col_num,
        }
    }
}

impl Runnable for ServiceWorkerErrorHandler {
    fn handler(self: Box<ServiceWorkerErrorHandler>) {
        let this = *self;
        ServiceWorker::handle_error_message(this.addr, this.msg, this.file_name, this.line_num, this.col_num);
    }
}

#[derive(Copy, Clone)]
pub struct SharedRt {
    rt: *mut JSRuntime
}

impl SharedRt {
    pub fn new(rt: &Runtime) -> SharedRt {
        SharedRt {
            rt: rt.rt()
        }
    }

    #[allow(unsafe_code)]
    pub fn request_interrupt(&self) {
        unsafe {
            JS_RequestInterruptCallback(self.rt);
        }
    }
}

#[allow(unsafe_code)]
unsafe impl Send for SharedRt {}

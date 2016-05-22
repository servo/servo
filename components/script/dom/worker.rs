/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use devtools_traits::{DevtoolsPageInfo, ScriptToDevtoolsControlMsg};
use dom::bindings::codegen::Bindings::EventHandlerBinding::EventHandlerNonNull;
use dom::bindings::codegen::Bindings::WorkerBinding;
use dom::bindings::codegen::Bindings::WorkerBinding::WorkerMethods;
use dom::bindings::error::{Error, ErrorResult, Fallible};
use dom::bindings::global::GlobalRef;
use dom::bindings::inheritance::Castable;
use dom::bindings::js::Root;
use dom::bindings::refcounted::Trusted;
use dom::bindings::reflector::{Reflectable, reflect_dom_object};
use dom::bindings::str::DOMString;
use dom::bindings::structuredclone::StructuredCloneData;
use dom::dedicatedworkerglobalscope::{DedicatedWorkerGlobalScope, WorkerScriptMsg};
use dom::errorevent::ErrorEvent;
use dom::event::{Event, EventBubbles, EventCancelable};
use dom::eventtarget::EventTarget;
use dom::messageevent::MessageEvent;
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

pub type TrustedWorkerAddress = Trusted<Worker>;

// https://html.spec.whatwg.org/multipage/#worker
#[dom_struct]
pub struct Worker {
    eventtarget: EventTarget,
    #[ignore_heap_size_of = "Defined in std"]
    /// Sender to the Receiver associated with the DedicatedWorkerGlobalScope
    /// this Worker created.
    sender: Sender<(TrustedWorkerAddress, WorkerScriptMsg)>,
    closing: Arc<AtomicBool>,
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

impl Worker {
    fn new_inherited(sender: Sender<(TrustedWorkerAddress, WorkerScriptMsg)>,
                     closing: Arc<AtomicBool>) -> Worker {
        Worker {
            eventtarget: EventTarget::new_inherited(),
            sender: sender,
            closing: closing,
            runtime: Arc::new(Mutex::new(None))
        }
    }

    pub fn new(global: GlobalRef,
               sender: Sender<(TrustedWorkerAddress, WorkerScriptMsg)>,
               closing: Arc<AtomicBool>) -> Root<Worker> {
        reflect_dom_object(box Worker::new_inherited(sender, closing),
                           global,
                           WorkerBinding::Wrap)
    }

    // https://html.spec.whatwg.org/multipage/#dom-worker
    pub fn Constructor(global: GlobalRef, script_url: DOMString) -> Fallible<Root<Worker>> {
        // Step 2-4.
        let worker_url = match global.api_base_url().join(&script_url) {
            Ok(url) => url,
            Err(_) => return Err(Error::Syntax),
        };

        let core_resource_thread = global.core_resource_thread();
        let constellation_chan = global.constellation_chan().clone();
        let scheduler_chan = global.scheduler_chan().clone();

        let (sender, receiver) = channel();
        let closing = Arc::new(AtomicBool::new(false));
        let worker = Worker::new(global, sender.clone(), closing.clone());
        let worker_ref = Trusted::new(worker.r());
        let worker_id = global.get_next_worker_id();

        let worker_load_origin = WorkerScriptLoadOrigin {
            referrer_url: None,
            referrer_policy: None,
            request_source: global.request_source(),
            pipeline_id: Some(global.pipeline())
        };

        let (devtools_sender, devtools_receiver) = ipc::channel().unwrap();
        let optional_sender = match global.devtools_chan() {
            Some(ref chan) => {
                let pipeline_id = global.pipeline();
                let title = format!("Worker for {}", worker_url);
                let page_info = DevtoolsPageInfo {
                    title: title,
                    url: worker_url.clone(),
                };
                chan.send(ScriptToDevtoolsControlMsg::NewGlobal((pipeline_id, Some(worker_id)),
                                                                devtools_sender.clone(),
                                                                page_info)).unwrap();
                Some(devtools_sender)
            },
            None => None,
        };

        let init = WorkerGlobalScopeInit {
            core_resource_thread: core_resource_thread,
            mem_profiler_chan: global.mem_profiler_chan().clone(),
            time_profiler_chan: global.time_profiler_chan().clone(),
            to_devtools_sender: global.devtools_chan(),
            from_devtools_sender: optional_sender,
            constellation_chan: constellation_chan,
            scheduler_chan: scheduler_chan,
            panic_chan: global.panic_chan().clone(),
            worker_id: worker_id,
            closing: closing,
        };

        DedicatedWorkerGlobalScope::run_worker_scope(
            init, worker_url, global.pipeline(), devtools_receiver, worker.runtime.clone(), worker_ref,
            global.script_chan(), sender, receiver, worker_load_origin);

        Ok(worker)
    }

    pub fn is_closing(&self) -> bool {
        self.closing.load(Ordering::SeqCst)
    }

    pub fn handle_message(address: TrustedWorkerAddress,
                          data: StructuredCloneData) {
        let worker = address.root();

        if worker.is_closing() {
            return;
        }

        let global = worker.r().global();
        let target = worker.upcast();
        let _ac = JSAutoCompartment::new(global.r().get_cx(), target.reflector().get_jsobject().get());
        let mut message = RootedValue::new(global.r().get_cx(), UndefinedValue());
        data.read(global.r(), message.handle_mut());
        MessageEvent::dispatch_jsval(target, global.r(), message.handle());
    }

    pub fn dispatch_simple_error(address: TrustedWorkerAddress) {
        let worker = address.root();
        worker.upcast().fire_simple_event("error");
    }

    pub fn handle_error_message(address: TrustedWorkerAddress, message: DOMString,
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
}

impl WorkerMethods for Worker {
    // https://html.spec.whatwg.org/multipage/#dom-worker-postmessage
    fn PostMessage(&self, cx: *mut JSContext, message: HandleValue) -> ErrorResult {
        let data = try!(StructuredCloneData::write(cx, message));
        let address = Trusted::new(self);
        self.sender.send((address, WorkerScriptMsg::DOMMessage(data))).unwrap();
        Ok(())
    }

    // https://html.spec.whatwg.org/multipage/#terminate-a-worker
    fn Terminate(&self) {
        // Step 1
        if self.closing.swap(true, Ordering::SeqCst) {
            return;
        }

        // Step 4
        if let Some(runtime) = *self.runtime.lock().unwrap() {
            runtime.request_interrupt();
        }
    }

    // https://html.spec.whatwg.org/multipage/#handler-worker-onmessage
    event_handler!(message, GetOnmessage, SetOnmessage);

    // https://html.spec.whatwg.org/multipage/#handler-workerglobalscope-onerror
    event_handler!(error, GetOnerror, SetOnerror);
}

pub struct WorkerMessageHandler {
    addr: TrustedWorkerAddress,
    data: StructuredCloneData,
}

impl WorkerMessageHandler {
    pub fn new(addr: TrustedWorkerAddress, data: StructuredCloneData) -> WorkerMessageHandler {
        WorkerMessageHandler {
            addr: addr,
            data: data,
        }
    }
}

impl Runnable for WorkerMessageHandler {
    fn handler(self: Box<WorkerMessageHandler>) {
        let this = *self;
        Worker::handle_message(this.addr, this.data);
    }
}

pub struct SimpleWorkerErrorHandler {
    addr: TrustedWorkerAddress,
}

impl SimpleWorkerErrorHandler {
    pub fn new(addr: TrustedWorkerAddress) -> SimpleWorkerErrorHandler {
        SimpleWorkerErrorHandler {
            addr: addr
        }
    }
}

impl Runnable for SimpleWorkerErrorHandler {
    fn handler(self: Box<SimpleWorkerErrorHandler>) {
        let this = *self;
        Worker::dispatch_simple_error(this.addr);
    }
}

pub struct WorkerErrorHandler {
    addr: TrustedWorkerAddress,
    msg: DOMString,
    file_name: DOMString,
    line_num: u32,
    col_num: u32,
}

impl WorkerErrorHandler {
    pub fn new(addr: TrustedWorkerAddress, msg: DOMString, file_name: DOMString, line_num: u32, col_num: u32)
            -> WorkerErrorHandler {
        WorkerErrorHandler {
            addr: addr,
            msg: msg,
            file_name: file_name,
            line_num: line_num,
            col_num: col_num,
        }
    }
}

impl Runnable for WorkerErrorHandler {
    fn handler(self: Box<WorkerErrorHandler>) {
        let this = *self;
        Worker::handle_error_message(this.addr, this.msg, this.file_name, this.line_num, this.col_num);
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

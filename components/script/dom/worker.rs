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
use dom::bindings::structuredclone::StructuredCloneData;
use dom::bindings::trace::JSTraceable;
use dom::dedicatedworkerglobalscope::{DedicatedWorkerGlobalScope, WorkerScriptMsg};
use dom::errorevent::ErrorEvent;
use dom::event::{Event, EventBubbles, EventCancelable};
use dom::eventtarget::EventTarget;
use dom::messageevent::MessageEvent;
use dom::workerglobalscope::WorkerGlobalScopeInit;
use ipc_channel::ipc;
use js::jsapi::{HandleValue, JSContext, RootedValue};
use js::jsapi::{JSAutoCompartment, JSAutoRequest};
use js::jsval::UndefinedValue;
use script_thread::{Runnable, ScriptChan};
use std::sync::mpsc::{Sender, channel};
use util::str::DOMString;

pub type TrustedWorkerAddress = Trusted<Worker>;

// https://html.spec.whatwg.org/multipage/#worker
#[dom_struct]
pub struct Worker {
    eventtarget: EventTarget,
    #[ignore_heap_size_of = "Defined in std"]
    /// Sender to the Receiver associated with the DedicatedWorkerGlobalScope
    /// this Worker created.
    sender: Sender<(TrustedWorkerAddress, WorkerScriptMsg)>,
}

impl Worker {
    fn new_inherited(sender: Sender<(TrustedWorkerAddress, WorkerScriptMsg)>)
                     -> Worker {
        Worker {
            eventtarget: EventTarget::new_inherited(),
            sender: sender,
        }
    }

    pub fn new(global: GlobalRef,
               sender: Sender<(TrustedWorkerAddress, WorkerScriptMsg)>)
               -> Root<Worker> {
        reflect_dom_object(box Worker::new_inherited(sender),
                           global,
                           WorkerBinding::Wrap)
    }

    // https://html.spec.whatwg.org/multipage/#dom-worker
    pub fn Constructor(global: GlobalRef, script_url: DOMString) -> Fallible<Root<Worker>> {
        // Step 2-4.
        let worker_url = match global.get_url().join(&script_url) {
            Ok(url) => url,
            Err(_) => return Err(Error::Syntax),
        };

        let resource_thread = global.resource_thread();
        let constellation_chan = global.constellation_chan();
        let scheduler_chan = global.scheduler_chan();

        let (sender, receiver) = channel();
        let worker = Worker::new(global, sender.clone());
        let worker_ref = Trusted::new(worker.r(), global.script_chan());
        let worker_id = global.get_next_worker_id();

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
            resource_thread: resource_thread,
            mem_profiler_chan: global.mem_profiler_chan(),
            to_devtools_sender: global.devtools_chan(),
            from_devtools_sender: optional_sender,
            constellation_chan: constellation_chan,
            scheduler_chan: scheduler_chan,
            worker_id: worker_id,
        };
        DedicatedWorkerGlobalScope::run_worker_scope(
            init, worker_url, global.pipeline(), devtools_receiver, worker_ref,
            global.script_chan(), sender, receiver);

        Ok(worker)
    }

    pub fn handle_message(address: TrustedWorkerAddress,
                          data: StructuredCloneData) {
        let worker = address.root();

        let global = worker.r().global();
        let target = worker.upcast();
        let _ar = JSAutoRequest::new(global.r().get_cx());
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
        let address = Trusted::new(self, self.global().r().script_chan());
        self.sender.send((address, WorkerScriptMsg::DOMMessage(data))).unwrap();
        Ok(())
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

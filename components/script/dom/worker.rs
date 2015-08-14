/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::WorkerBinding;
use dom::bindings::codegen::Bindings::WorkerBinding::WorkerMethods;
use dom::bindings::codegen::Bindings::EventHandlerBinding::EventHandlerNonNull;
use dom::bindings::codegen::InheritTypes::{EventCast, EventTargetCast};
use dom::bindings::error::{Fallible, ErrorResult};
use dom::bindings::error::Error::Syntax;
use dom::bindings::global::{GlobalRef, GlobalField};
use dom::bindings::refcounted::Trusted;
use dom::bindings::structuredclone::StructuredCloneData;
use dom::bindings::trace::JSTraceable;
use dom::bindings::utils::{Reflectable, reflect_dom_object};
use dom::bindings::js::Root;
use dom::window::WindowHelpers;
use dom::dedicatedworkerglobalscope::DedicatedWorkerGlobalScope;
use dom::errorevent::ErrorEvent;
use dom::event::{Event, EventBubbles, EventCancelable, EventHelpers};
use dom::eventtarget::{EventTarget, EventTargetHelpers, EventTargetTypeId};
use dom::messageevent::MessageEvent;
use dom::workerglobalscope::WorkerGlobalScopeInit;
use script_task::{ScriptChan, ScriptMsg, Runnable};

use devtools_traits::{DevtoolsPageInfo, ScriptToDevtoolsControlMsg};

use util::str::DOMString;

use ipc_channel::ipc;
use js::jsapi::{JSContext, HandleValue, RootedValue};
use js::jsapi::{JSAutoRequest, JSAutoCompartment};
use js::jsval::UndefinedValue;
use url::UrlParser;

use std::borrow::ToOwned;
use std::sync::mpsc::{channel, Sender};

pub type TrustedWorkerAddress = Trusted<Worker>;

// https://html.spec.whatwg.org/multipage/#worker
#[dom_struct]
#[derive(HeapSizeOf)]
pub struct Worker {
    eventtarget: EventTarget,
    global: GlobalField,
    #[ignore_heap_size_of = "Defined in std"]
    /// Sender to the Receiver associated with the DedicatedWorkerGlobalScope
    /// this Worker created.
    sender: Sender<(TrustedWorkerAddress, ScriptMsg)>,
}

impl Worker {
    fn new_inherited(global: GlobalRef, sender: Sender<(TrustedWorkerAddress, ScriptMsg)>) -> Worker {
        Worker {
            eventtarget: EventTarget::new_inherited(EventTargetTypeId::Worker),
            global: GlobalField::from_rooted(&global),
            sender: sender,
        }
    }

    pub fn new(global: GlobalRef, sender: Sender<(TrustedWorkerAddress, ScriptMsg)>) -> Root<Worker> {
        reflect_dom_object(box Worker::new_inherited(global, sender),
                           global,
                           WorkerBinding::Wrap)
    }

    // https://www.whatwg.org/html/#dom-worker
    pub fn Constructor(global: GlobalRef, script_url: DOMString) -> Fallible<Root<Worker>> {
        // Step 2-4.
        let worker_url = match UrlParser::new().base_url(&global.get_url()).parse(&script_url) {
            Ok(url) => url,
            Err(_) => return Err(Syntax),
        };

        let resource_task = global.resource_task();
        let constellation_chan = global.constellation_chan();

        let (sender, receiver) = channel();
        let worker = Worker::new(global, sender.clone());
        let worker_ref = Trusted::new(global.get_cx(), worker.r(), global.script_chan());
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
            resource_task: resource_task,
            mem_profiler_chan: global.mem_profiler_chan(),
            devtools_chan: global.devtools_chan(),
            devtools_sender: optional_sender,
            constellation_chan: constellation_chan,
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

        let global = worker.r().global.root();
        let target = EventTargetCast::from_ref(worker.r());
        let _ar = JSAutoRequest::new(global.r().get_cx());
        let _ac = JSAutoCompartment::new(global.r().get_cx(), target.reflector().get_jsobject().get());
        let mut message = RootedValue::new(global.r().get_cx(), UndefinedValue());
        data.read(global.r(), message.handle_mut());
        MessageEvent::dispatch_jsval(target, global.r(), message.handle());
    }

    pub fn dispatch_simple_error(address: TrustedWorkerAddress) {
        let worker = address.root();
        let global = worker.r().global.root();
        let target = EventTargetCast::from_ref(worker.r());

        let event = Event::new(global.r(),
                               "error".to_owned(),
                               EventBubbles::DoesNotBubble,
                               EventCancelable::NotCancelable);
        event.r().fire(target);
    }

    pub fn handle_error_message(address: TrustedWorkerAddress, message: DOMString,
                                filename: DOMString, lineno: u32, colno: u32) {
        let worker = address.root();
        let global = worker.r().global.root();
        let error = RootedValue::new(global.r().get_cx(), UndefinedValue());
        let target = EventTargetCast::from_ref(worker.r());
        let errorevent = ErrorEvent::new(global.r(), "error".to_owned(),
                                         EventBubbles::Bubbles, EventCancelable::Cancelable,
                                         message, filename, lineno, colno, error.handle());
        let event = EventCast::from_ref(errorevent.r());
        event.fire(target);
    }
}

impl<'a> WorkerMethods for &'a Worker {
    // https://html.spec.whatwg.org/multipage/#dom-dedicatedworkerglobalscope-postmessage
    fn PostMessage(self, cx: *mut JSContext, message: HandleValue) -> ErrorResult {
        let data = try!(StructuredCloneData::write(cx, message));
        let address = Trusted::new(cx, self, self.global.root().r().script_chan().clone());
        self.sender.send((address, ScriptMsg::DOMMessage(data))).unwrap();
        Ok(())
    }

    event_handler!(message, GetOnmessage, SetOnmessage);
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

pub struct WorkerEventHandler {
    addr: TrustedWorkerAddress,
}

impl WorkerEventHandler {
    pub fn new(addr: TrustedWorkerAddress) -> WorkerEventHandler {
        WorkerEventHandler {
            addr: addr
        }
    }
}

impl Runnable for WorkerEventHandler {
    fn handler(self: Box<WorkerEventHandler>) {
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

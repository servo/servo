/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::WorkerBinding;
use dom::bindings::codegen::Bindings::WorkerBinding::WorkerMethods;
use dom::bindings::codegen::Bindings::EventHandlerBinding::EventHandlerNonNull;
use dom::bindings::codegen::InheritTypes::EventTargetCast;
use dom::bindings::error::{Fallible, ErrorResult};
use dom::bindings::error::Error::Syntax;
use dom::bindings::global::{GlobalRef, GlobalField};
use dom::bindings::js::{JSRef, Temporary};
use dom::bindings::refcounted::Trusted;
use dom::bindings::structuredclone::StructuredCloneData;
use dom::bindings::trace::JSTraceable;
use dom::bindings::utils::{Reflectable, reflect_dom_object};
use dom::dedicatedworkerglobalscope::DedicatedWorkerGlobalScope;
use dom::eventtarget::{EventTarget, EventTargetHelpers, EventTargetTypeId};
use dom::messageevent::MessageEvent;
use script_task::{ScriptChan, ScriptMsg, Runnable};

use servo_util::str::DOMString;

use js::jsapi::JSContext;
use js::jsval::JSVal;
use url::UrlParser;

use std::cell::Cell;

pub type TrustedWorkerAddress = Trusted<Worker>;

#[dom_struct]
pub struct Worker {
    eventtarget: EventTarget,
    refcount: Cell<uint>,
    global: GlobalField,
    /// Sender to the Receiver associated with the DedicatedWorkerGlobalScope
    /// this Worker created.
    sender: Sender<(TrustedWorkerAddress, ScriptMsg)>,
}

impl Worker {
    fn new_inherited(global: GlobalRef, sender: Sender<(TrustedWorkerAddress, ScriptMsg)>) -> Worker {
        Worker {
            eventtarget: EventTarget::new_inherited(EventTargetTypeId::Worker),
            refcount: Cell::new(0),
            global: GlobalField::from_rooted(&global),
            sender: sender,
        }
    }

    pub fn new(global: GlobalRef, sender: Sender<(TrustedWorkerAddress, ScriptMsg)>) -> Temporary<Worker> {
        reflect_dom_object(box Worker::new_inherited(global, sender),
                           global,
                           WorkerBinding::Wrap)
    }

    // http://www.whatwg.org/html/#dom-worker
    pub fn Constructor(global: GlobalRef, scriptURL: DOMString) -> Fallible<Temporary<Worker>> {
        // Step 2-4.
        let worker_url = match UrlParser::new().base_url(&global.get_url())
                .parse(scriptURL.as_slice()) {
            Ok(url) => url,
            Err(_) => return Err(Syntax),
        };

        let resource_task = global.resource_task();

        let (sender, receiver) = channel();
        let worker = Worker::new(global, sender.clone()).root();
        let worker_ref = Trusted::new(global.get_cx(), worker.r(), global.script_chan());

        DedicatedWorkerGlobalScope::run_worker_scope(
            worker_url, worker_ref, resource_task, global.script_chan(),
            sender, receiver);

        Ok(Temporary::from_rooted(worker.r()))
    }

    pub fn handle_message(address: TrustedWorkerAddress,
                          data: StructuredCloneData) {
        let worker = address.to_temporary().root();

        let global = worker.r().global.root();
        let target: JSRef<EventTarget> = EventTargetCast::from_ref(worker.r());

        let message = data.read(global.r());
        MessageEvent::dispatch_jsval(target, global.r(), message);
    }
}

impl<'a> WorkerMethods for JSRef<'a, Worker> {
    fn PostMessage(self, cx: *mut JSContext, message: JSVal) -> ErrorResult {
        let data = try!(StructuredCloneData::write(cx, message));
        let address = Trusted::new(cx, self, self.global.root().r().script_chan().clone());
        self.sender.send((address, ScriptMsg::DOMMessage(data)));
        Ok(())
    }

    event_handler!(message, GetOnmessage, SetOnmessage)
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
    fn handler(&self){
        Worker::handle_message(self.addr.clone(), self.data);
    }
}

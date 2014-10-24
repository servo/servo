/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::WorkerBinding;
use dom::bindings::codegen::Bindings::WorkerBinding::WorkerMethods;
use dom::bindings::codegen::Bindings::EventHandlerBinding::EventHandlerNonNull;
use dom::bindings::codegen::InheritTypes::EventTargetCast;
use dom::bindings::error::{Fallible, Syntax, ErrorResult, DataClone};
use dom::bindings::global::{GlobalRef, GlobalField};
use dom::bindings::js::{JS, JSRef, Temporary};
use dom::bindings::trace::JSTraceable;
use dom::bindings::utils::{Reflectable, Reflector, reflect_dom_object};
use dom::dedicatedworkerglobalscope::DedicatedWorkerGlobalScope;
use dom::eventtarget::{EventTarget, EventTargetHelpers, WorkerTypeId};
use dom::messageevent::MessageEvent;
use script_task::{ScriptChan, DOMMessage};

use servo_util::str::DOMString;

use js::glue::JS_STRUCTURED_CLONE_VERSION;
use js::jsapi::{JSContext, JS_AddObjectRoot, JS_RemoveObjectRoot, JSTracer};
use js::jsapi::{JS_ReadStructuredClone, JS_WriteStructuredClone, JS_ClearPendingException};
use js::jsval::{JSVal, UndefinedValue};
use url::UrlParser;

use libc::{c_void, size_t};
use std::cell::Cell;
use std::ptr;

pub struct TrustedWorkerAddress(pub *const c_void);
no_jsmanaged_fields!(TrustedWorkerAddress)

#[dom_struct]
pub struct Worker {
    eventtarget: EventTarget,
    refcount: Cell<uint>,
    global: GlobalField,
    /// Sender to the Receiver associated with the DedicatedWorkerGlobalScope
    /// this Worker created.
    sender: ScriptChan,
}

impl Worker {
    fn new_inherited(global: &GlobalRef, sender: ScriptChan) -> Worker {
        Worker {
            eventtarget: EventTarget::new_inherited(WorkerTypeId),
            refcount: Cell::new(0),
            global: GlobalField::from_rooted(global),
            sender: sender,
        }
    }

    pub fn new(global: &GlobalRef, sender: ScriptChan) -> Temporary<Worker> {
        reflect_dom_object(box Worker::new_inherited(global, sender),
                           global,
                           WorkerBinding::Wrap)
    }

    // http://www.whatwg.org/html/#dom-worker
    pub fn Constructor(global: &GlobalRef, scriptURL: DOMString) -> Fallible<Temporary<Worker>> {
        // Step 2-4.
        let worker_url = match UrlParser::new().base_url(&global.get_url())
                .parse(scriptURL.as_slice()) {
            Ok(url) => url,
            Err(_) => return Err(Syntax),
        };

        let resource_task = global.resource_task();
        let (receiver, sender) = ScriptChan::new();

        let worker = Worker::new(global, sender.clone()).root();
        let worker_ref = worker.addref();

        DedicatedWorkerGlobalScope::run_worker_scope(
            worker_url, worker_ref, resource_task, global.script_chan().clone(),
            sender, receiver);

        Ok(Temporary::from_rooted(*worker))
    }

    pub fn handle_message(address: TrustedWorkerAddress,
                          data: *mut u64, nbytes: size_t) {
        let worker = unsafe { JS::from_trusted_worker_address(address).root() };

        let global = worker.global.root();

        let mut message = UndefinedValue();
        unsafe {
            assert!(JS_ReadStructuredClone(
                global.root_ref().get_cx(), data as *const u64, nbytes,
                JS_STRUCTURED_CLONE_VERSION, &mut message,
                ptr::null(), ptr::null_mut()) != 0);
        }

        let target: JSRef<EventTarget> = EventTargetCast::from_ref(*worker);
        MessageEvent::dispatch_jsval(target, &global.root_ref(), message);
    }
}

impl Worker {
    // Creates a trusted address to the object, and roots it. Always pair this with a release()
    pub fn addref(&self) -> TrustedWorkerAddress {
        let refcount = self.refcount.get();
        if refcount == 0 {
            let cx = self.global.root().root_ref().get_cx();
            unsafe {
                JS_AddObjectRoot(cx, self.reflector().rootable());
            }
        }
        self.refcount.set(refcount + 1);
        TrustedWorkerAddress(self as *const Worker as *const c_void)
    }

    pub fn release(&self) {
        let refcount = self.refcount.get();
        assert!(refcount > 0)
        self.refcount.set(refcount - 1);
        if refcount == 1 {
            let cx = self.global.root().root_ref().get_cx();
            unsafe {
                JS_RemoveObjectRoot(cx, self.reflector().rootable());
            }
        }
    }

    pub fn handle_release(address: TrustedWorkerAddress) {
        let worker = unsafe { JS::from_trusted_worker_address(address).root() };
        worker.release();
    }
}

impl<'a> WorkerMethods for JSRef<'a, Worker> {
    fn PostMessage(self, cx: *mut JSContext, message: JSVal) -> ErrorResult {
        let mut data = ptr::null_mut();
        let mut nbytes = 0;
        let result = unsafe {
            JS_WriteStructuredClone(cx, message, &mut data, &mut nbytes,
                                    ptr::null(), ptr::null_mut())
        };
        if result == 0 {
            unsafe { JS_ClearPendingException(cx); }
            return Err(DataClone);
        }

        self.addref();
        let ScriptChan(ref sender) = self.sender;
        sender.send(DOMMessage(data, nbytes));
        Ok(())
    }

    event_handler!(message, GetOnmessage, SetOnmessage)
}

impl Reflectable for Worker {
    fn reflector<'a>(&'a self) -> &'a Reflector {
        self.eventtarget.reflector()
    }
}

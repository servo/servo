/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::DedicatedWorkerGlobalScopeBinding;
use dom::bindings::codegen::Bindings::DedicatedWorkerGlobalScopeBinding::DedicatedWorkerGlobalScopeMethods;
use dom::bindings::codegen::Bindings::EventHandlerBinding::EventHandlerNonNull;
use dom::bindings::codegen::InheritTypes::DedicatedWorkerGlobalScopeDerived;
use dom::bindings::codegen::InheritTypes::{EventTargetCast, WorkerGlobalScopeCast};
use dom::bindings::global::Worker;
use dom::bindings::js::{JSRef, Temporary, RootCollection};
use dom::bindings::trace::Untraceable;
use dom::bindings::utils::{Reflectable, Reflector};
use dom::eventtarget::{EventTarget, EventTargetHelpers};
use dom::eventtarget::WorkerGlobalScopeTypeId;
use dom::messageevent::MessageEvent;
use dom::worker::{Worker, TrustedWorkerAddress};
use dom::workerglobalscope::DedicatedGlobalScope;
use dom::workerglobalscope::WorkerGlobalScope;
use dom::xmlhttprequest::XMLHttpRequest;
use script_task::{ScriptTask, ScriptChan};
use script_task::{ScriptMsg, DOMMessage, XHRProgressMsg, WorkerRelease};
use script_task::WorkerPostMessage;
use script_task::StackRootTLS;

use servo_net::resource_task::{ResourceTask, load_whole_resource};

use js::glue::JS_STRUCTURED_CLONE_VERSION;
use js::jsapi::{JSContext, JS_ReadStructuredClone, JS_WriteStructuredClone};
use js::jsval::{JSVal, UndefinedValue};
use js::rust::Cx;

use std::rc::Rc;
use std::ptr;
use std::task::TaskBuilder;
use native::task::NativeTaskBuilder;
use url::Url;

#[deriving(Encodable)]
#[must_root]
pub struct DedicatedWorkerGlobalScope {
    workerglobalscope: WorkerGlobalScope,
    receiver: Untraceable<Receiver<ScriptMsg>>,
    /// Sender to the parent thread.
    parent_sender: ScriptChan,
    worker: Untraceable<TrustedWorkerAddress>,
}

impl DedicatedWorkerGlobalScope {
    pub fn new_inherited(worker_url: Url,
                         worker: TrustedWorkerAddress,
                         cx: Rc<Cx>,
                         resource_task: ResourceTask,
                         parent_sender: ScriptChan,
                         own_sender: ScriptChan,
                         receiver: Receiver<ScriptMsg>)
                         -> DedicatedWorkerGlobalScope {
        DedicatedWorkerGlobalScope {
            workerglobalscope: WorkerGlobalScope::new_inherited(
                DedicatedGlobalScope, worker_url, cx, resource_task,
                own_sender),
            receiver: Untraceable::new(receiver),
            parent_sender: parent_sender,
            worker: Untraceable::new(worker),
        }
    }

    pub fn new(worker_url: Url,
               worker: TrustedWorkerAddress,
               cx: Rc<Cx>,
               resource_task: ResourceTask,
               parent_sender: ScriptChan,
               own_sender: ScriptChan,
               receiver: Receiver<ScriptMsg>)
               -> Temporary<DedicatedWorkerGlobalScope> {
        let scope = box DedicatedWorkerGlobalScope::new_inherited(
            worker_url, worker, cx.clone(), resource_task, parent_sender,
            own_sender, receiver);
        DedicatedWorkerGlobalScopeBinding::Wrap(cx.ptr, scope)
    }
}

impl DedicatedWorkerGlobalScope {
    pub fn run_worker_scope(worker_url: Url,
                            worker: TrustedWorkerAddress,
                            resource_task: ResourceTask,
                            parent_sender: ScriptChan,
                            own_sender: ScriptChan,
                            receiver: Receiver<ScriptMsg>) {
        TaskBuilder::new()
            .native()
            .named(format!("Web Worker at {}", worker_url.serialize()))
            .spawn(proc() {
            let roots = RootCollection::new();
            let _stack_roots_tls = StackRootTLS::new(&roots);

            let (url, source) = match load_whole_resource(&resource_task, worker_url.clone()) {
                Err(_) => {
                    println!("error loading script {}", worker_url.serialize());
                    return;
                }
                Ok((metadata, bytes)) => {
                    (metadata.final_url, String::from_utf8(bytes).unwrap())
                }
            };

            let (_js_runtime, js_context) = ScriptTask::new_rt_and_cx();
            let global = DedicatedWorkerGlobalScope::new(
                worker_url, worker, js_context.clone(), resource_task,
                parent_sender, own_sender, receiver).root();
            match js_context.evaluate_script(
                global.reflector().get_jsobject(), source, url.serialize(), 1) {
                Ok(_) => (),
                Err(_) => println!("evaluate_script failed")
            }
            global.delayed_release_worker();

            let scope: JSRef<WorkerGlobalScope> =
                WorkerGlobalScopeCast::from_ref(*global);
            let target: JSRef<EventTarget> =
                EventTargetCast::from_ref(*global);
            loop {
                match global.receiver.deref().recv_opt() {
                    Ok(DOMMessage(data, nbytes)) => {
                        let mut message = UndefinedValue();
                        unsafe {
                            assert!(JS_ReadStructuredClone(
                                js_context.ptr, data as *const u64, nbytes,
                                JS_STRUCTURED_CLONE_VERSION, &mut message,
                                ptr::null(), ptr::mut_null()) != 0);
                        }

                        MessageEvent::dispatch_jsval(target, &Worker(scope), message);
                        global.delayed_release_worker();
                    },
                    Ok(XHRProgressMsg(addr, progress)) => {
                        XMLHttpRequest::handle_xhr_progress(addr, progress)
                    },
                    Ok(WorkerPostMessage(addr, data, nbytes)) => {
                        Worker::handle_message(addr, data, nbytes);
                    },
                    Ok(WorkerRelease(addr)) => {
                        Worker::handle_release(addr)
                    },
                    Ok(_) => fail!("Unexpected message"),
                    Err(_) => break,
                }
            }
        });
    }
}

impl<'a> DedicatedWorkerGlobalScopeMethods for JSRef<'a, DedicatedWorkerGlobalScope> {
    fn PostMessage(&self, cx: *mut JSContext, message: JSVal) {
        let mut data = ptr::mut_null();
        let mut nbytes = 0;
        unsafe {
            assert!(JS_WriteStructuredClone(cx, message, &mut data, &mut nbytes,
                                            ptr::null(), ptr::mut_null()) != 0);
        }

        let ScriptChan(ref sender) = self.parent_sender;
        sender.send(WorkerPostMessage(*self.worker, data, nbytes));
    }

    fn GetOnmessage(&self) -> Option<EventHandlerNonNull> {
        let eventtarget: JSRef<EventTarget> = EventTargetCast::from_ref(*self);
        eventtarget.get_event_handler_common("message")
    }

    fn SetOnmessage(&self, listener: Option<EventHandlerNonNull>) {
        let eventtarget: JSRef<EventTarget> = EventTargetCast::from_ref(*self);
        eventtarget.set_event_handler_common("message", listener)
    }
}

trait PrivateDedicatedWorkerGlobalScopeHelpers {
    fn delayed_release_worker(&self);
}

impl<'a> PrivateDedicatedWorkerGlobalScopeHelpers for JSRef<'a, DedicatedWorkerGlobalScope> {
    fn delayed_release_worker(&self) {
        let ScriptChan(ref sender) = self.parent_sender;
        sender.send(WorkerRelease(*self.worker));
    }
}

impl Reflectable for DedicatedWorkerGlobalScope {
    fn reflector<'a>(&'a self) -> &'a Reflector {
        self.workerglobalscope.reflector()
    }
}

impl DedicatedWorkerGlobalScopeDerived for EventTarget {
    fn is_dedicatedworkerglobalscope(&self) -> bool {
        match self.type_id {
            WorkerGlobalScopeTypeId(DedicatedGlobalScope) => true,
            _ => false
        }
    }
}

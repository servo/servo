/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::DedicatedWorkerGlobalScopeBinding;
use dom::bindings::codegen::Bindings::DedicatedWorkerGlobalScopeBinding::DedicatedWorkerGlobalScopeMethods;
use dom::bindings::codegen::Bindings::EventHandlerBinding::EventHandlerNonNull;
use dom::bindings::codegen::InheritTypes::DedicatedWorkerGlobalScopeDerived;
use dom::bindings::codegen::InheritTypes::{EventTargetCast, WorkerGlobalScopeCast};
use dom::bindings::error::ErrorResult;
use dom::bindings::error::Error::DataClone;
use dom::bindings::global;
use dom::bindings::js::{JSRef, Temporary, RootCollection};
use dom::bindings::utils::{Reflectable, Reflector};
use dom::eventtarget::{EventTarget, EventTargetHelpers, EventTargetTypeId};
use dom::messageevent::MessageEvent;
use dom::worker::{Worker, TrustedWorkerAddress};
use dom::workerglobalscope::{WorkerGlobalScope, WorkerGlobalScopeHelpers};
use dom::workerglobalscope::WorkerGlobalScopeTypeId;
use dom::xmlhttprequest::XMLHttpRequest;
use script_task::{ScriptTask, ScriptChan, ScriptMsg, TimerSource};
use script_task::ScriptMsg::{DOMMessage, FireTimerMsg, XHRProgressMsg};
use script_task::ScriptMsg::{XHRReleaseMsg, WorkerRelease, WorkerPostMessage};
use script_task::StackRootTLS;

use servo_net::resource_task::{ResourceTask, load_whole_resource};
use servo_util::task::spawn_named_native;
use servo_util::task_state;
use servo_util::task_state::{SCRIPT, IN_WORKER};

use js::glue::JS_STRUCTURED_CLONE_VERSION;
use js::jsapi::{JSContext, JS_ReadStructuredClone, JS_WriteStructuredClone, JS_ClearPendingException};
use js::jsval::{JSVal, UndefinedValue};
use js::rust::Cx;

use std::rc::Rc;
use std::ptr;
use url::Url;

#[dom_struct]
pub struct DedicatedWorkerGlobalScope {
    workerglobalscope: WorkerGlobalScope,
    receiver: Receiver<ScriptMsg>,
    /// Sender to the parent thread.
    parent_sender: ScriptChan,
    worker: TrustedWorkerAddress,
}

impl DedicatedWorkerGlobalScope {
    fn new_inherited(worker_url: Url,
                         worker: TrustedWorkerAddress,
                         cx: Rc<Cx>,
                         resource_task: ResourceTask,
                         parent_sender: ScriptChan,
                         own_sender: ScriptChan,
                         receiver: Receiver<ScriptMsg>)
                         -> DedicatedWorkerGlobalScope {
        DedicatedWorkerGlobalScope {
            workerglobalscope: WorkerGlobalScope::new_inherited(
                WorkerGlobalScopeTypeId::DedicatedGlobalScope, worker_url, cx,
                resource_task, own_sender),
            receiver: receiver,
            parent_sender: parent_sender,
            worker: worker,
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
        spawn_named_native(format!("WebWorker for {}", worker_url.serialize()), proc() {

            task_state::initialize(SCRIPT | IN_WORKER);

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
                match global.receiver.recv_opt() {
                    Ok(DOMMessage(data, nbytes)) => {
                        let mut message = UndefinedValue();
                        unsafe {
                            assert!(JS_ReadStructuredClone(
                                js_context.ptr, data as *const u64, nbytes,
                                JS_STRUCTURED_CLONE_VERSION, &mut message,
                                ptr::null(), ptr::null_mut()) != 0);
                        }

                        MessageEvent::dispatch_jsval(target, global::Worker(scope), message);
                        global.delayed_release_worker();
                    },
                    Ok(XHRProgressMsg(addr, progress)) => {
                        XMLHttpRequest::handle_progress(addr, progress)
                    },
                    Ok(XHRReleaseMsg(addr)) => {
                        XMLHttpRequest::handle_release(addr)
                    },
                    Ok(WorkerPostMessage(addr, data, nbytes)) => {
                        Worker::handle_message(addr, data, nbytes);
                    },
                    Ok(WorkerRelease(addr)) => {
                        Worker::handle_release(addr)
                    },
                    Ok(FireTimerMsg(TimerSource::FromWorker, timer_id)) => {
                        scope.handle_fire_timer(timer_id);
                    }
                    Ok(_) => panic!("Unexpected message"),
                    Err(_) => break,
                }
            }
        });
    }
}

impl<'a> DedicatedWorkerGlobalScopeMethods for JSRef<'a, DedicatedWorkerGlobalScope> {
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

        let ScriptChan(ref sender) = self.parent_sender;
        sender.send(WorkerPostMessage(self.worker, data, nbytes));
        Ok(())
    }

    event_handler!(message, GetOnmessage, SetOnmessage)
}

trait PrivateDedicatedWorkerGlobalScopeHelpers {
    fn delayed_release_worker(self);
}

impl<'a> PrivateDedicatedWorkerGlobalScopeHelpers for JSRef<'a, DedicatedWorkerGlobalScope> {
    fn delayed_release_worker(self) {
        let ScriptChan(ref sender) = self.parent_sender;
        sender.send(WorkerRelease(self.worker));
    }
}

impl Reflectable for DedicatedWorkerGlobalScope {
    fn reflector<'a>(&'a self) -> &'a Reflector {
        self.workerglobalscope.reflector()
    }
}

impl DedicatedWorkerGlobalScopeDerived for EventTarget {
    fn is_dedicatedworkerglobalscope(&self) -> bool {
        match *self.type_id() {
            EventTargetTypeId::WorkerGlobalScope(WorkerGlobalScopeTypeId::DedicatedGlobalScope) => true,
            _ => false
        }
    }
}

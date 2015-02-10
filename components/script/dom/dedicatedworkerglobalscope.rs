/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::cell::DOMRefCell;
use dom::bindings::codegen::Bindings::DedicatedWorkerGlobalScopeBinding;
use dom::bindings::codegen::Bindings::DedicatedWorkerGlobalScopeBinding::DedicatedWorkerGlobalScopeMethods;
use dom::bindings::codegen::Bindings::ErrorEventBinding::ErrorEventMethods;
use dom::bindings::codegen::Bindings::EventHandlerBinding::EventHandlerNonNull;
use dom::bindings::codegen::InheritTypes::DedicatedWorkerGlobalScopeDerived;
use dom::bindings::codegen::InheritTypes::{EventTargetCast, WorkerGlobalScopeCast};
use dom::bindings::error::ErrorResult;
use dom::bindings::global::GlobalRef;
use dom::bindings::js::{JSRef, Temporary, RootCollection};
use dom::bindings::refcounted::LiveDOMReferences;
use dom::bindings::structuredclone::StructuredCloneData;
use dom::bindings::utils::Reflectable;
use dom::errorevent::ErrorEvent;
use dom::eventtarget::{EventTarget, EventTargetHelpers, EventTargetTypeId};
use dom::messageevent::MessageEvent;
use dom::worker::{TrustedWorkerAddress, WorkerMessageHandler, Worker};
use dom::workerglobalscope::{WorkerGlobalScope, WorkerGlobalScopeHelpers};
use dom::workerglobalscope::WorkerGlobalScopeTypeId;
use script_task::{ScriptTask, ScriptChan, ScriptMsg, TimerSource};
use script_task::ScriptMsg::WorkerDispatchErrorEvent;
use script_task::StackRootTLS;

use net::resource_task::{ResourceTask, load_whole_resource};
use util::task::spawn_named;
use util::task_state;
use util::task_state::{SCRIPT, IN_WORKER};

use js::jsapi::JSContext;
use js::jsval::JSVal;
use js::rust::Cx;

use std::rc::Rc;
use std::sync::mpsc::{Sender, Receiver};
use url::Url;

/// A ScriptChan that can be cloned freely and will silently send a TrustedWorkerAddress with
/// every message. While this SendableWorkerScriptChan is alive, the associated Worker object
/// will remain alive.
#[derive(Clone)]
#[jstraceable]
pub struct SendableWorkerScriptChan {
    sender: Sender<(TrustedWorkerAddress, ScriptMsg)>,
    worker: TrustedWorkerAddress,
}

impl ScriptChan for SendableWorkerScriptChan {
    fn send(&self, msg: ScriptMsg) {
        self.sender.send((self.worker.clone(), msg)).unwrap();
    }

    fn clone(&self) -> Box<ScriptChan + Send> {
        box SendableWorkerScriptChan {
            sender: self.sender.clone(),
            worker: self.worker.clone(),
        }
    }
}

/// Set the `worker` field of a related DedicatedWorkerGlobalScope object to a particular
/// value for the duration of this object's lifetime. This ensures that the related Worker
/// object only lives as long as necessary (ie. while events are being executed), while
/// providing a reference that can be cloned freely.
struct AutoWorkerReset<'a> {
    workerscope: JSRef<'a, DedicatedWorkerGlobalScope>,
    old_worker: Option<TrustedWorkerAddress>,
}

impl<'a> AutoWorkerReset<'a> {
    fn new(workerscope: JSRef<'a, DedicatedWorkerGlobalScope>, worker: TrustedWorkerAddress) -> AutoWorkerReset<'a> {
        let reset = AutoWorkerReset {
            workerscope: workerscope,
            old_worker: workerscope.worker.borrow().clone()
        };
        *workerscope.worker.borrow_mut() = Some(worker);
        reset
    }
}

#[unsafe_destructor]
impl<'a> Drop for AutoWorkerReset<'a> {
    fn drop(&mut self) {
        *self.workerscope.worker.borrow_mut() = self.old_worker.clone();
    }
}

#[dom_struct]
pub struct DedicatedWorkerGlobalScope {
    workerglobalscope: WorkerGlobalScope,
    receiver: Receiver<(TrustedWorkerAddress, ScriptMsg)>,
    own_sender: Sender<(TrustedWorkerAddress, ScriptMsg)>,
    worker: DOMRefCell<Option<TrustedWorkerAddress>>,
    /// Sender to the parent thread.
    parent_sender: Box<ScriptChan+Send>,
}

impl DedicatedWorkerGlobalScope {
    fn new_inherited(worker_url: Url,
                         cx: Rc<Cx>,
                         resource_task: ResourceTask,
                         parent_sender: Box<ScriptChan+Send>,
                         own_sender: Sender<(TrustedWorkerAddress, ScriptMsg)>,
                         receiver: Receiver<(TrustedWorkerAddress, ScriptMsg)>)
                         -> DedicatedWorkerGlobalScope {
        DedicatedWorkerGlobalScope {
            workerglobalscope: WorkerGlobalScope::new_inherited(
                WorkerGlobalScopeTypeId::DedicatedGlobalScope, worker_url, cx, resource_task),
            receiver: receiver,
            own_sender: own_sender,
            parent_sender: parent_sender,
            worker: DOMRefCell::new(None),
        }
    }

    pub fn new(worker_url: Url,
               cx: Rc<Cx>,
               resource_task: ResourceTask,
               parent_sender: Box<ScriptChan+Send>,
               own_sender: Sender<(TrustedWorkerAddress, ScriptMsg)>,
               receiver: Receiver<(TrustedWorkerAddress, ScriptMsg)>)
               -> Temporary<DedicatedWorkerGlobalScope> {
        let scope = box DedicatedWorkerGlobalScope::new_inherited(
            worker_url, cx.clone(), resource_task, parent_sender,
            own_sender, receiver);
        DedicatedWorkerGlobalScopeBinding::Wrap(cx.ptr, scope)
    }
}

impl DedicatedWorkerGlobalScope {
    pub fn run_worker_scope(worker_url: Url,
                            worker: TrustedWorkerAddress,
                            resource_task: ResourceTask,
                            parent_sender: Box<ScriptChan+Send>,
                            own_sender: Sender<(TrustedWorkerAddress, ScriptMsg)>,
                            receiver: Receiver<(TrustedWorkerAddress, ScriptMsg)>) {
        spawn_named(format!("WebWorker for {}", worker_url.serialize()), move || {
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
                worker_url, js_context.clone(), resource_task,
                parent_sender, own_sender, receiver).root();

            {
                let _ar = AutoWorkerReset::new(global.r(), worker);

                match js_context.evaluate_script(
                    global.r().reflector().get_jsobject(), source, url.serialize(), 1) {
                    Ok(_) => (),
                    Err(_) => println!("evaluate_script failed")
                }
            }

            loop {
                match global.r().receiver.recv() {
                    Ok((linked_worker, msg)) => {
                        let _ar = AutoWorkerReset::new(global.r(), linked_worker);
                        global.r().handle_event(msg);
                    }
                    Err(_) => break,
                }
            }
        });
    }
}

pub trait DedicatedWorkerGlobalScopeHelpers {
    fn script_chan(self) -> Box<ScriptChan+Send>;
}

impl<'a> DedicatedWorkerGlobalScopeHelpers for JSRef<'a, DedicatedWorkerGlobalScope> {
    fn script_chan(self) -> Box<ScriptChan+Send> {
        box SendableWorkerScriptChan {
            sender: self.own_sender.clone(),
            worker: self.worker.borrow().as_ref().unwrap().clone(),
        }
    }
}

trait PrivateDedicatedWorkerGlobalScopeHelpers {
    fn handle_event(self, msg: ScriptMsg);
    fn dispatch_error_to_worker(self, JSRef<ErrorEvent>);
}

impl<'a> PrivateDedicatedWorkerGlobalScopeHelpers for JSRef<'a, DedicatedWorkerGlobalScope> {
    fn handle_event(self, msg: ScriptMsg) {
        match msg {
            ScriptMsg::DOMMessage(data) => {
                let scope: JSRef<WorkerGlobalScope> = WorkerGlobalScopeCast::from_ref(self);
                let target: JSRef<EventTarget> = EventTargetCast::from_ref(self);
                let message = data.read(GlobalRef::Worker(scope));
                MessageEvent::dispatch_jsval(target, GlobalRef::Worker(scope), message);
            },
            ScriptMsg::RunnableMsg(runnable) => {
                runnable.handler()
            },
            ScriptMsg::RefcountCleanup(addr) => {
                let scope: JSRef<WorkerGlobalScope> = WorkerGlobalScopeCast::from_ref(self);
                LiveDOMReferences::cleanup(scope.get_cx(), addr);
            }
            ScriptMsg::WorkerDispatchErrorEvent(addr, msg, file_name, line_num, col_num) => {
                Worker::handle_error_message(addr, msg, file_name, line_num, col_num);
            },
            ScriptMsg::FireTimer(TimerSource::FromWorker, timer_id) => {
                let scope: JSRef<WorkerGlobalScope> = WorkerGlobalScopeCast::from_ref(self);
                scope.handle_fire_timer(timer_id);
            }
            _ => panic!("Unexpected message"),
        }
    }

    fn dispatch_error_to_worker(self, errorevent: JSRef<ErrorEvent>) {
        let msg = errorevent.Message();
        let file_name = errorevent.Filename();
        let line_num = errorevent.Lineno();
        let col_num = errorevent.Colno();
        let worker = self.worker.borrow().as_ref().unwrap().clone();
        self.parent_sender.send(ScriptMsg::WorkerDispatchErrorEvent(worker, msg, file_name,
                                                                    line_num, col_num));
 }
}

impl<'a> DedicatedWorkerGlobalScopeMethods for JSRef<'a, DedicatedWorkerGlobalScope> {
    fn PostMessage(self, cx: *mut JSContext, message: JSVal) -> ErrorResult {
        let data = try!(StructuredCloneData::write(cx, message));
        let worker = self.worker.borrow().as_ref().unwrap().clone();
        self.parent_sender.send(ScriptMsg::RunnableMsg(
            box WorkerMessageHandler::new(worker, data)));
        Ok(())
    }

    event_handler!(message, GetOnmessage, SetOnmessage);
}

impl DedicatedWorkerGlobalScopeDerived for EventTarget {
    fn is_dedicatedworkerglobalscope(&self) -> bool {
        match *self.type_id() {
            EventTargetTypeId::WorkerGlobalScope(WorkerGlobalScopeTypeId::DedicatedGlobalScope) => true,
            _ => false
        }
    }
}

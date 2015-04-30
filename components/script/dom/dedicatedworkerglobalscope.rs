/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
#![allow(unsafe_code)]

use dom::bindings::cell::DOMRefCell;
use dom::bindings::codegen::Bindings::DedicatedWorkerGlobalScopeBinding;
use dom::bindings::codegen::Bindings::DedicatedWorkerGlobalScopeBinding::DedicatedWorkerGlobalScopeMethods;
use dom::bindings::codegen::Bindings::ErrorEventBinding::ErrorEventMethods;
use dom::bindings::codegen::Bindings::EventHandlerBinding::EventHandlerNonNull;
use dom::bindings::codegen::InheritTypes::{DedicatedWorkerGlobalScopeDerived, DedicatedWorkerGlobalScopeCast};
use dom::bindings::codegen::InheritTypes::{EventTargetCast, WorkerGlobalScopeCast};
use dom::bindings::error::ErrorResult;
use dom::bindings::global::{GlobalRef, global_object_for_js_context};
use dom::bindings::js::{JSRef, Temporary, RootCollection};
use dom::bindings::refcounted::LiveDOMReferences;
use dom::bindings::structuredclone::StructuredCloneData;
use dom::bindings::utils::Reflectable;
use dom::bindings::trace::JSTraceable;
use dom::errorevent::ErrorEvent;
use dom::eventtarget::{EventTarget, EventTargetHelpers, EventTargetTypeId};
use dom::messageevent::MessageEvent;
use dom::worker::{TrustedWorkerAddress, WorkerMessageHandler, WorkerEventHandler, WorkerErrorHandler, SharedRt};
use dom::workerglobalscope::{WorkerGlobalScope, WorkerGlobalScopeHelpers};
use dom::workerglobalscope::WorkerGlobalScopeTypeId;
use script_task::{ScriptTask, ScriptChan, ScriptMsg, TimerSource, ScriptPort};
use script_task::StackRootTLS;

use msg::constellation_msg::PipelineId;

use devtools_traits::DevtoolsControlChan;

use net_traits::{load_whole_resource, ResourceTask};
use util::task::spawn_named;
use util::task_state;
use util::task_state::{SCRIPT, IN_WORKER};

use js::jsapi::{JSContext, JS_SetOperationCallback, JSBool};
use js::jsval::JSVal;
use js::rust::Cx;

use std::cell::RefCell;
use std::collections::LinkedList;
use std::rc::Rc;
use std::sync::mpsc::{Sender, Receiver, channel};
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
    fn send(&self, msg: ScriptMsg) -> Result<(), ()> {
        return self.sender.send((self.worker.clone(), msg)).map_err(|_| ());
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

// https://html.spec.whatwg.org/multipage/#dedicatedworkerglobalscope
#[dom_struct]
pub struct DedicatedWorkerGlobalScope {
    workerglobalscope: WorkerGlobalScope,
    id: PipelineId,
    receiver: Receiver<(TrustedWorkerAddress, ScriptMsg)>,
    own_sender: Sender<(TrustedWorkerAddress, ScriptMsg)>,
    worker: DOMRefCell<Option<TrustedWorkerAddress>>,
    /// Sender to the parent thread.
    parent_sender: Box<ScriptChan+Send>,
    msg_queue: RefCell<LinkedList<(TrustedWorkerAddress, ScriptMsg)>>
}

impl DedicatedWorkerGlobalScope {
    fn new_inherited(worker_url: Url,
                         id: PipelineId,
                         devtools_chan: Option<DevtoolsControlChan>,
                         cx: Rc<Cx>,
                         resource_task: ResourceTask,
                         parent_sender: Box<ScriptChan+Send>,
                         own_sender: Sender<(TrustedWorkerAddress, ScriptMsg)>,
                         receiver: Receiver<(TrustedWorkerAddress, ScriptMsg)>)
                         -> DedicatedWorkerGlobalScope {
        DedicatedWorkerGlobalScope {
            workerglobalscope: WorkerGlobalScope::new_inherited(
                WorkerGlobalScopeTypeId::DedicatedGlobalScope, worker_url, cx, resource_task, devtools_chan),
            id: id,
            receiver: receiver,
            own_sender: own_sender,
            parent_sender: parent_sender,
            worker: DOMRefCell::new(None),
            msg_queue: RefCell::new(LinkedList::new())
        }
    }

    pub fn new(worker_url: Url,
               id: PipelineId,
               devtools_chan: Option<DevtoolsControlChan>,
               cx: Rc<Cx>,
               resource_task: ResourceTask,
               parent_sender: Box<ScriptChan+Send>,
               own_sender: Sender<(TrustedWorkerAddress, ScriptMsg)>,
               receiver: Receiver<(TrustedWorkerAddress, ScriptMsg)>)
               -> Temporary<DedicatedWorkerGlobalScope> {
        let scope = box DedicatedWorkerGlobalScope::new_inherited(
            worker_url, id, devtools_chan, cx.clone(), resource_task, parent_sender,
            own_sender, receiver);
        DedicatedWorkerGlobalScopeBinding::Wrap(cx.ptr, scope)
    }
}

impl DedicatedWorkerGlobalScope {
    pub fn run_worker_scope(worker_url: Url,
                            id: PipelineId,
                            devtools_chan: Option<DevtoolsControlChan>,
                            worker: TrustedWorkerAddress,
                            resource_task: ResourceTask,
                            parent_sender: Box<ScriptChan+Send>,
                            own_sender: Sender<(TrustedWorkerAddress, ScriptMsg)>,
                            receiver: Receiver<(TrustedWorkerAddress, ScriptMsg)>,
                            rt_sender: Sender<SharedRt>) {
        spawn_named(format!("WebWorker for {}", worker_url.serialize()), move || {
            task_state::initialize(SCRIPT | IN_WORKER);

            let roots = RootCollection::new();
            let _stack_roots_tls = StackRootTLS::new(&roots);

            let (url, source) = match load_whole_resource(&resource_task, worker_url.clone()) {
                Err(_) => {
                    println!("error loading script {}", worker_url.serialize());
                    parent_sender.send(ScriptMsg::RunnableMsg(
                        box WorkerEventHandler::new(worker))).unwrap();
                    return;
                }
                Ok((metadata, bytes)) => {
                    (metadata.final_url, String::from_utf8(bytes).unwrap())
                }
            };

            let runtime = ScriptTask::new_rt_and_cx();

            // Send JSRuntime ref to main thread for interrupt scheduling
            rt_sender.send(SharedRt::new(runtime.rt())).unwrap();

            // Handle interrupt requests
            unsafe {
                JS_SetOperationCallback(runtime.cx.ptr,
                    Some(interrupt_callback as unsafe extern "C" fn(*mut JSContext) -> JSBool));
            }

            let global = DedicatedWorkerGlobalScope::new(
                worker_url, id, devtools_chan, runtime.cx.clone(), resource_task,
                parent_sender, own_sender, receiver).root();

            {
                let _ar = AutoWorkerReset::new(global.r(), worker);

                match runtime.cx.evaluate_script(
                    global.r().reflector().get_jsobject(), source, url.serialize(), 1) {
                    Ok(_) => (),
                    Err(_) => {
                        if global.r().is_closing() {
                            println!("evaluate_script failed (terminated)");
                        } else {
                            println!("evaluate_script failed");
                            return
                        }
                    }
                }
            }

            loop {
                // Process any pending events
                while let Some((linked_worker, msg)) = global.r().take_event() {
                    let _ar = AutoWorkerReset::new(global.r(), linked_worker);
                    global.r().handle_event(msg);

                    if global.r().is_closing() { break }
                }

                // Wait for new events
                match global.r().receiver.recv() {
                    Ok((linked_worker, msg)) => {
                        let _ar = AutoWorkerReset::new(global.r(), linked_worker);
                        global.r().handle_event(msg);

                        if global.r().is_closing() { break }
                    }
                    Err(_) => break,
                }
            }

            WorkerGlobalScopeCast::from_ref(global.r()).clear_timers();
            global.r().clear_events();
        });
    }
}

unsafe extern "C" fn interrupt_callback(cx: *mut JSContext) -> JSBool {
    // get global for context
    let global = global_object_for_js_context(cx);
    let global_root = global.root();
    let scope = match global_root.r() {
        GlobalRef::Worker(w) => DedicatedWorkerGlobalScopeCast::to_ref(w).unwrap(),
        _ => panic!("global for worker is not a DedicatedWorkerGlobalScope")
    };

    // Process any critical control messages. It might be nice to have two message types, each on
    // their own channel, so that we don't need to buffer events outside of the channel.
    while let Ok(msg) = scope.receiver.try_recv() {
        match msg {
            (linked_worker, ScriptMsg::Terminate) => {
                let _ar = AutoWorkerReset::new(scope, linked_worker);
                scope.handle_event(ScriptMsg::Terminate)
            },
            _ => scope.queue_event(msg)
        }
    }

    // A false response causes the script to terminate
    !scope.is_closing() as JSBool
}

pub trait DedicatedWorkerGlobalScopeHelpers {
    fn script_chan(self) -> Box<ScriptChan+Send>;
    fn pipeline(self) -> PipelineId;
    fn new_script_pair(self) -> (Box<ScriptChan+Send>, Box<ScriptPort+Send>);
    fn process_event(self, msg: ScriptMsg);
    fn queue_event(self, msg: (TrustedWorkerAddress, ScriptMsg));
    fn take_event(self) -> Option<(TrustedWorkerAddress, ScriptMsg)>;
    fn clear_events(self);
    fn is_closing(self) -> bool;
}

impl<'a> DedicatedWorkerGlobalScopeHelpers for JSRef<'a, DedicatedWorkerGlobalScope> {
    fn script_chan(self) -> Box<ScriptChan+Send> {
        // FIXME(https://github.com/rust-lang/rust/issues/23338)
        let worker = self.worker.borrow();
        box SendableWorkerScriptChan {
            sender: self.own_sender.clone(),
            worker: worker.as_ref().unwrap().clone(),
        }
    }

    fn pipeline(self) -> PipelineId {
        self.id
    }

    fn new_script_pair(self) -> (Box<ScriptChan+Send>, Box<ScriptPort+Send>) {
        let (tx, rx) = channel();
        let chan = box SendableWorkerScriptChan {
            sender: tx,
            worker: self.worker.borrow().as_ref().unwrap().clone(),
        };
        (chan, box rx)
    }

    fn process_event(self, msg: ScriptMsg) {
        self.handle_event(msg);
    }

    fn queue_event(self, msg: (TrustedWorkerAddress, ScriptMsg)) {
        let mut events = self.msg_queue.borrow_mut();
        events.push_back(msg);
    }

    fn take_event(self) -> Option<(TrustedWorkerAddress, ScriptMsg)> {
        let mut events = self.msg_queue.borrow_mut();
        events.pop_front()
    }

    fn clear_events(self) {
        let mut events = self.msg_queue.borrow_mut();
        events.clear()
    }

    fn is_closing(self) -> bool {
        WorkerGlobalScopeCast::from_ref(self).get_closing()
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
            },
            ScriptMsg::FireTimer(TimerSource::FromWorker, timer_id) => {
                let scope: JSRef<WorkerGlobalScope> = WorkerGlobalScopeCast::from_ref(self);
                scope.handle_fire_timer(timer_id);
            },
            ScriptMsg::Terminate => {
                WorkerGlobalScopeCast::from_ref(self).set_closing(true);
            },
            _ => panic!("Unexpected message"),
        }
    }

    fn dispatch_error_to_worker(self, errorevent: JSRef<ErrorEvent>) {
        let msg = errorevent.Message();
        let file_name = errorevent.Filename();
        let line_num = errorevent.Lineno();
        let col_num = errorevent.Colno();
        let worker = self.worker.borrow().as_ref().unwrap().clone();
        self.parent_sender.send(ScriptMsg::RunnableMsg(
            box WorkerErrorHandler::new(worker, msg, file_name, line_num, col_num))).unwrap();
 }
}

impl<'a> DedicatedWorkerGlobalScopeMethods for JSRef<'a, DedicatedWorkerGlobalScope> {
    // https://html.spec.whatwg.org/multipage/#dom-dedicatedworkerglobalscope-postmessage
    fn PostMessage(self, cx: *mut JSContext, message: JSVal) -> ErrorResult {
        let data = try!(StructuredCloneData::write(cx, message));
        let worker = self.worker.borrow().as_ref().unwrap().clone();
        self.parent_sender.send(ScriptMsg::RunnableMsg(
            box WorkerMessageHandler::new(worker, data))).unwrap();
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

impl JSTraceable for RefCell<LinkedList<(TrustedWorkerAddress, ScriptMsg)>> {
    fn trace(&self, _: *mut ::js::jsapi::JSTracer) {
        // TODO? 
    }
}

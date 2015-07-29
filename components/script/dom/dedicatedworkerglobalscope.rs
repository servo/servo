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
use dom::bindings::error::{ErrorResult, report_pending_exception};
use dom::bindings::global::GlobalRef;
use dom::bindings::js::{RootCollection, Root};
use dom::bindings::refcounted::LiveDOMReferences;
use dom::bindings::structuredclone::StructuredCloneData;
use dom::bindings::utils::Reflectable;
use dom::errorevent::ErrorEvent;
use dom::eventtarget::{EventTarget, EventTargetHelpers, EventTargetTypeId};
use dom::messageevent::MessageEvent;
use dom::worker::{TrustedWorkerAddress, WorkerMessageHandler, WorkerEventHandler, WorkerErrorHandler};
use dom::workerglobalscope::{WorkerGlobalScope, WorkerGlobalScopeHelpers};
use dom::workerglobalscope::WorkerGlobalScopeTypeId;
use script_task::{ScriptTask, ScriptChan, ScriptMsg, TimerSource, ScriptPort};
use script_task::StackRootTLS;

use msg::constellation_msg::{ConstellationChan, PipelineId};

use devtools_traits::ScriptToDevtoolsControlMsg;

use net_traits::{load_whole_resource, ResourceTask};
use profile_traits::mem::{self, Reporter, ReporterRequest};
use util::task::spawn_named;
use util::task_state;
use util::task_state::{SCRIPT, IN_WORKER};

use ipc_channel::ipc::{self, IpcSender};
use ipc_channel::router::ROUTER;
use js::jsapi::{JSContext, RootedValue, HandleValue};
use js::jsapi::{JSAutoRequest, JSAutoCompartment};
use js::jsval::UndefinedValue;
use js::rust::Runtime;
use url::Url;

use rand::random;
use std::mem::replace;
use std::rc::Rc;
use std::sync::mpsc::{Sender, Receiver, channel};

/// A ScriptChan that can be cloned freely and will silently send a TrustedWorkerAddress with
/// every message. While this SendableWorkerScriptChan is alive, the associated Worker object
/// will remain alive.
#[derive(JSTraceable, Clone)]
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
    workerscope: &'a DedicatedWorkerGlobalScope,
    old_worker: Option<TrustedWorkerAddress>,
}

impl<'a> AutoWorkerReset<'a> {
    fn new(workerscope: &'a DedicatedWorkerGlobalScope, worker: TrustedWorkerAddress) -> AutoWorkerReset<'a> {
        AutoWorkerReset {
            workerscope: workerscope,
            old_worker: replace(&mut *workerscope.worker.borrow_mut(), Some(worker)),
        }
    }
}

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
}

impl DedicatedWorkerGlobalScope {
    fn new_inherited(worker_url: Url,
                     id: PipelineId,
                     mem_profiler_chan: mem::ProfilerChan,
                     devtools_chan: Option<IpcSender<ScriptToDevtoolsControlMsg>>,
                     runtime: Rc<Runtime>,
                     resource_task: ResourceTask,
                     constellation_chan: ConstellationChan,
                     parent_sender: Box<ScriptChan+Send>,
                     own_sender: Sender<(TrustedWorkerAddress, ScriptMsg)>,
                     receiver: Receiver<(TrustedWorkerAddress, ScriptMsg)>)
                     -> DedicatedWorkerGlobalScope {
        DedicatedWorkerGlobalScope {
            workerglobalscope: WorkerGlobalScope::new_inherited(
                WorkerGlobalScopeTypeId::DedicatedGlobalScope, worker_url,
                runtime, resource_task, mem_profiler_chan, devtools_chan, constellation_chan),
            id: id,
            receiver: receiver,
            own_sender: own_sender,
            parent_sender: parent_sender,
            worker: DOMRefCell::new(None),
        }
    }

    pub fn new(worker_url: Url,
               id: PipelineId,
               mem_profiler_chan: mem::ProfilerChan,
               devtools_chan: Option<IpcSender<ScriptToDevtoolsControlMsg>>,
               runtime: Rc<Runtime>,
               resource_task: ResourceTask,
               constellation_chan: ConstellationChan,
               parent_sender: Box<ScriptChan+Send>,
               own_sender: Sender<(TrustedWorkerAddress, ScriptMsg)>,
               receiver: Receiver<(TrustedWorkerAddress, ScriptMsg)>)
               -> Root<DedicatedWorkerGlobalScope> {
        let scope = box DedicatedWorkerGlobalScope::new_inherited(
            worker_url, id, mem_profiler_chan, devtools_chan, runtime.clone(), resource_task,
            constellation_chan, parent_sender, own_sender, receiver);
        DedicatedWorkerGlobalScopeBinding::Wrap(runtime.cx(), scope)
    }
}

impl DedicatedWorkerGlobalScope {
    pub fn run_worker_scope(worker_url: Url,
                            id: PipelineId,
                            mem_profiler_chan: mem::ProfilerChan,
                            devtools_chan: Option<IpcSender<ScriptToDevtoolsControlMsg>>,
                            worker: TrustedWorkerAddress,
                            resource_task: ResourceTask,
                            constellation_chan: ConstellationChan,
                            parent_sender: Box<ScriptChan+Send>,
                            own_sender: Sender<(TrustedWorkerAddress, ScriptMsg)>,
                            receiver: Receiver<(TrustedWorkerAddress, ScriptMsg)>) {
        let serialized_worker_url = worker_url.serialize();
        spawn_named(format!("WebWorker for {}", serialized_worker_url), move || {
            task_state::initialize(SCRIPT | IN_WORKER);

            let roots = RootCollection::new();
            let _stack_roots_tls = StackRootTLS::new(&roots);

            let (url, source) = match load_whole_resource(&resource_task, worker_url) {
                Err(_) => {
                    println!("error loading script {}", serialized_worker_url);
                    parent_sender.send(ScriptMsg::RunnableMsg(
                        box WorkerEventHandler::new(worker))).unwrap();
                    return;
                }
                Ok((metadata, bytes)) => {
                    (metadata.final_url, String::from_utf8(bytes).unwrap())
                }
            };

            let runtime = Rc::new(ScriptTask::new_rt_and_cx());
            let serialized_url = url.serialize();
            let parent_sender_for_reporter = parent_sender.clone();
            let global = DedicatedWorkerGlobalScope::new(
                url, id, mem_profiler_chan.clone(), devtools_chan, runtime.clone(), resource_task,
                constellation_chan, parent_sender, own_sender, receiver);
            // FIXME(njn): workers currently don't have a unique ID suitable for using in reporter
            // registration (#6631), so we instead use a random number and cross our fingers.
            let reporter_name = format!("worker-reporter-{}", random::<u64>());

            {
                let _ar = AutoWorkerReset::new(global.r(), worker);

                match runtime.evaluate_script(
                    global.r().reflector().get_jsobject(), source, serialized_url, 1) {
                    Ok(_) => (),
                    Err(_) => {
                        // TODO: An error needs to be dispatched to the parent.
                        // https://github.com/servo/servo/issues/6422
                        println!("evaluate_script failed");
                        let _ar = JSAutoRequest::new(runtime.cx());
                        report_pending_exception(runtime.cx(), global.r().reflector().get_jsobject().get());
                    }
                }

                // Register this task as a memory reporter. This needs to be done within the
                // scope of `_ar` otherwise script_chan_as_reporter() will panic.
                let (reporter_sender, reporter_receiver) = ipc::channel().unwrap();
                ROUTER.add_route(reporter_receiver.to_opaque(), box move |reporter_request| {
                    // Just injects an appropriate event into the worker task's queue.
                    let reporter_request: ReporterRequest = reporter_request.to().unwrap();
                    parent_sender_for_reporter.send(ScriptMsg::CollectReports(
                            reporter_request.reports_channel)).unwrap()
                });
                mem_profiler_chan.send(mem::ProfilerMsg::RegisterReporter(
                        reporter_name.clone(),
                        Reporter(reporter_sender)));
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

            // Unregister this task as a memory reporter.
            let msg = mem::ProfilerMsg::UnregisterReporter(reporter_name);
            mem_profiler_chan.send(msg);
        });
    }
}

pub trait DedicatedWorkerGlobalScopeHelpers {
    fn script_chan(self) -> Box<ScriptChan+Send>;
    fn pipeline(self) -> PipelineId;
    fn new_script_pair(self) -> (Box<ScriptChan+Send>, Box<ScriptPort+Send>);
    fn process_event(self, msg: ScriptMsg);
}

impl<'a> DedicatedWorkerGlobalScopeHelpers for &'a DedicatedWorkerGlobalScope {
    fn script_chan(self) -> Box<ScriptChan+Send> {
        box SendableWorkerScriptChan {
            sender: self.own_sender.clone(),
            worker: self.worker.borrow().as_ref().unwrap().clone(),
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
}

trait PrivateDedicatedWorkerGlobalScopeHelpers {
    fn handle_event(self, msg: ScriptMsg);
    fn dispatch_error_to_worker(self, &ErrorEvent);
}

impl<'a> PrivateDedicatedWorkerGlobalScopeHelpers for &'a DedicatedWorkerGlobalScope {
    fn handle_event(self, msg: ScriptMsg) {
        match msg {
            ScriptMsg::DOMMessage(data) => {
                let scope = WorkerGlobalScopeCast::from_ref(self);
                let target = EventTargetCast::from_ref(self);
                let _ar = JSAutoRequest::new(scope.get_cx());
                let _ac = JSAutoCompartment::new(scope.get_cx(), scope.reflector().get_jsobject().get());
                let mut message = RootedValue::new(scope.get_cx(), UndefinedValue());
                data.read(GlobalRef::Worker(scope), message.handle_mut());
                MessageEvent::dispatch_jsval(target, GlobalRef::Worker(scope), message.handle());
            },
            ScriptMsg::RunnableMsg(runnable) => {
                runnable.handler()
            },
            ScriptMsg::RefcountCleanup(addr) => {
                LiveDOMReferences::cleanup(addr);
            }
            ScriptMsg::FireTimer(TimerSource::FromWorker, timer_id) => {
                let scope = WorkerGlobalScopeCast::from_ref(self);
                scope.handle_fire_timer(timer_id);
            }
            ScriptMsg::CollectReports(reports_chan) => {
                let scope = WorkerGlobalScopeCast::from_ref(self);
                let cx = scope.get_cx();
                let path_seg = format!("url({})", scope.get_url());
                let reports = ScriptTask::get_reports(cx, path_seg);
                reports_chan.send(reports);
            }
            _ => panic!("Unexpected message"),
        }
    }

    fn dispatch_error_to_worker(self, errorevent: &ErrorEvent) {
        let msg = errorevent.Message();
        let file_name = errorevent.Filename();
        let line_num = errorevent.Lineno();
        let col_num = errorevent.Colno();
        let worker = self.worker.borrow().as_ref().unwrap().clone();
        self.parent_sender.send(ScriptMsg::RunnableMsg(
            box WorkerErrorHandler::new(worker, msg, file_name, line_num, col_num))).unwrap();
 }
}

impl<'a> DedicatedWorkerGlobalScopeMethods for &'a DedicatedWorkerGlobalScope {
    // https://html.spec.whatwg.org/multipage/#dom-dedicatedworkerglobalscope-postmessage
    fn PostMessage(self, cx: *mut JSContext, message: HandleValue) -> ErrorResult {
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

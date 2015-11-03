/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use devtools;
use devtools_traits::DevtoolScriptControlMsg;
use dom::bindings::cell::DOMRefCell;
use dom::bindings::codegen::Bindings::DedicatedWorkerGlobalScopeBinding;
use dom::bindings::codegen::Bindings::DedicatedWorkerGlobalScopeBinding::DedicatedWorkerGlobalScopeMethods;
use dom::bindings::codegen::Bindings::EventHandlerBinding::EventHandlerNonNull;
use dom::bindings::conversions::Castable;
use dom::bindings::error::ErrorResult;
use dom::bindings::global::GlobalRef;
use dom::bindings::js::{Root, RootCollection};
use dom::bindings::refcounted::LiveDOMReferences;
use dom::bindings::structuredclone::StructuredCloneData;
use dom::bindings::utils::Reflectable;
use dom::messageevent::MessageEvent;
use dom::worker::{SimpleWorkerErrorHandler, TrustedWorkerAddress, WorkerMessageHandler};
use dom::workerglobalscope::WorkerGlobalScope;
use dom::workerglobalscope::WorkerGlobalScopeInit;
use ipc_channel::ipc::{self, IpcReceiver, IpcSender};
use ipc_channel::router::ROUTER;
use js::jsapi::{HandleValue, JSContext, RootedValue};
use js::jsapi::{JSAutoCompartment, JSAutoRequest};
use js::jsval::UndefinedValue;
use js::rust::Runtime;
use msg::constellation_msg::PipelineId;
use net_traits::load_whole_resource;
use rand::random;
use script_task::ScriptTaskEventCategory::WorkerEvent;
use script_task::{ScriptTask, ScriptChan, ScriptPort, StackRootTLS, CommonScriptMsg};
use script_traits::{TimerEvent, TimerSource};
use std::mem::replace;
use std::rc::Rc;
use std::sync::mpsc::{Receiver, RecvError, Select, Sender, channel};
use url::Url;
use util::task::spawn_named;
use util::task_state;
use util::task_state::{IN_WORKER, SCRIPT};

/// Messages used to control the worker event loops
pub enum WorkerScriptMsg {
    /// Common variants associated with the script messages
    Common(CommonScriptMsg),
    /// Message sent through Worker.postMessage
    DOMMessage(StructuredCloneData),
}

/// A ScriptChan that can be cloned freely and will silently send a TrustedWorkerAddress with
/// common event loop messages. While this SendableWorkerScriptChan is alive, the associated
/// Worker object will remain alive.
#[derive(JSTraceable, Clone)]
pub struct SendableWorkerScriptChan {
    sender: Sender<(TrustedWorkerAddress, CommonScriptMsg)>,
    worker: TrustedWorkerAddress,
}

impl ScriptChan for SendableWorkerScriptChan {
    fn send(&self, msg: CommonScriptMsg) -> Result<(), ()> {
        self.sender.send((self.worker.clone(), msg)).map_err(|_| ())
    }

    fn clone(&self) -> Box<ScriptChan + Send> {
        box SendableWorkerScriptChan {
            sender: self.sender.clone(),
            worker: self.worker.clone(),
        }
    }
}

/// A ScriptChan that can be cloned freely and will silently send a TrustedWorkerAddress with
/// worker event loop messages. While this SendableWorkerScriptChan is alive, the associated
/// Worker object will remain alive.
#[derive(JSTraceable, Clone)]
pub struct WorkerThreadWorkerChan {
    sender: Sender<(TrustedWorkerAddress, WorkerScriptMsg)>,
    worker: TrustedWorkerAddress,
}

impl ScriptChan for WorkerThreadWorkerChan {
    fn send(&self, msg: CommonScriptMsg) -> Result<(), ()> {
        self.sender
            .send((self.worker.clone(), WorkerScriptMsg::Common(msg)))
            .map_err(|_| ())
    }

    fn clone(&self) -> Box<ScriptChan + Send> {
        box WorkerThreadWorkerChan {
            sender: self.sender.clone(),
            worker: self.worker.clone(),
        }
    }
}

impl ScriptPort for Receiver<(TrustedWorkerAddress, WorkerScriptMsg)> {
    fn recv(&self) -> CommonScriptMsg {
        match self.recv().unwrap().1 {
            WorkerScriptMsg::Common(script_msg) => script_msg,
            WorkerScriptMsg::DOMMessage(_) => panic!("unexpected worker event message!"),
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

enum MixedMessage {
    FromWorker((TrustedWorkerAddress, WorkerScriptMsg)),
    FromScheduler((TrustedWorkerAddress, TimerEvent)),
    FromDevtools(DevtoolScriptControlMsg),
}

// https://html.spec.whatwg.org/multipage/#dedicatedworkerglobalscope
#[dom_struct]
pub struct DedicatedWorkerGlobalScope {
    workerglobalscope: WorkerGlobalScope,
    id: PipelineId,
    #[ignore_heap_size_of = "Defined in std"]
    receiver: Receiver<(TrustedWorkerAddress, WorkerScriptMsg)>,
    #[ignore_heap_size_of = "Defined in std"]
    own_sender: Sender<(TrustedWorkerAddress, WorkerScriptMsg)>,
    #[ignore_heap_size_of = "Defined in std"]
    timer_event_port: Receiver<(TrustedWorkerAddress, TimerEvent)>,
    #[ignore_heap_size_of = "Trusted<T> has unclear ownership like JS<T>"]
    worker: DOMRefCell<Option<TrustedWorkerAddress>>,
    #[ignore_heap_size_of = "Can't measure trait objects"]
    /// Sender to the parent thread.
    parent_sender: Box<ScriptChan + Send>,
}

impl DedicatedWorkerGlobalScope {
    fn new_inherited(init: WorkerGlobalScopeInit,
                     worker_url: Url,
                     id: PipelineId,
                     from_devtools_receiver: Receiver<DevtoolScriptControlMsg>,
                     runtime: Rc<Runtime>,
                     parent_sender: Box<ScriptChan + Send>,
                     own_sender: Sender<(TrustedWorkerAddress, WorkerScriptMsg)>,
                     receiver: Receiver<(TrustedWorkerAddress, WorkerScriptMsg)>,
                     timer_event_chan: IpcSender<TimerEvent>,
                     timer_event_port: Receiver<(TrustedWorkerAddress, TimerEvent)>)
                     -> DedicatedWorkerGlobalScope {

        DedicatedWorkerGlobalScope {
            workerglobalscope: WorkerGlobalScope::new_inherited(
                init, worker_url, runtime, from_devtools_receiver, timer_event_chan),
            id: id,
            receiver: receiver,
            own_sender: own_sender,
            timer_event_port: timer_event_port,
            parent_sender: parent_sender,
            worker: DOMRefCell::new(None),
        }
    }

    pub fn new(init: WorkerGlobalScopeInit,
               worker_url: Url,
               id: PipelineId,
               from_devtools_receiver: Receiver<DevtoolScriptControlMsg>,
               runtime: Rc<Runtime>,
               parent_sender: Box<ScriptChan + Send>,
               own_sender: Sender<(TrustedWorkerAddress, WorkerScriptMsg)>,
               receiver: Receiver<(TrustedWorkerAddress, WorkerScriptMsg)>,
               timer_event_chan: IpcSender<TimerEvent>,
               timer_event_port: Receiver<(TrustedWorkerAddress, TimerEvent)>)
               -> Root<DedicatedWorkerGlobalScope> {
        let scope = box DedicatedWorkerGlobalScope::new_inherited(
            init, worker_url, id, from_devtools_receiver, runtime.clone(), parent_sender,
            own_sender, receiver, timer_event_chan, timer_event_port);
        DedicatedWorkerGlobalScopeBinding::Wrap(runtime.cx(), scope)
    }

    pub fn run_worker_scope(init: WorkerGlobalScopeInit,
                            worker_url: Url,
                            id: PipelineId,
                            from_devtools_receiver: IpcReceiver<DevtoolScriptControlMsg>,
                            worker: TrustedWorkerAddress,
                            parent_sender: Box<ScriptChan + Send>,
                            own_sender: Sender<(TrustedWorkerAddress, WorkerScriptMsg)>,
                            receiver: Receiver<(TrustedWorkerAddress, WorkerScriptMsg)>) {
        let serialized_worker_url = worker_url.serialize();
        spawn_named(format!("WebWorker for {}", serialized_worker_url), move || {
            task_state::initialize(SCRIPT | IN_WORKER);

            let roots = RootCollection::new();
            let _stack_roots_tls = StackRootTLS::new(&roots);

            let (url, source) = match load_whole_resource(&init.resource_task, worker_url) {
                Err(_) => {
                    println!("error loading script {}", serialized_worker_url);
                    parent_sender.send(CommonScriptMsg::RunnableMsg(WorkerEvent,
                        box SimpleWorkerErrorHandler::new(worker))).unwrap();
                    return;
                }
                Ok((metadata, bytes)) => {
                    (metadata.final_url, String::from_utf8(bytes).unwrap())
                }
            };

            let runtime = Rc::new(ScriptTask::new_rt_and_cx());

            let (devtools_mpsc_chan, devtools_mpsc_port) = channel();
            ROUTER.route_ipc_receiver_to_mpsc_sender(from_devtools_receiver, devtools_mpsc_chan);

            let (timer_tx, timer_rx) = channel();
            let (timer_ipc_chan, timer_ipc_port) = ipc::channel().unwrap();
            let worker_for_route = worker.clone();
            ROUTER.add_route(timer_ipc_port.to_opaque(), box move |message| {
                let event = message.to().unwrap();
                timer_tx.send((worker_for_route.clone(), event)).unwrap();
            });

            let global = DedicatedWorkerGlobalScope::new(
                init, url, id, devtools_mpsc_port, runtime.clone(),
                parent_sender.clone(), own_sender, receiver,
                timer_ipc_chan, timer_rx);
            // FIXME(njn): workers currently don't have a unique ID suitable for using in reporter
            // registration (#6631), so we instead use a random number and cross our fingers.
            let scope = global.upcast::<WorkerGlobalScope>();

            {
                let _ar = AutoWorkerReset::new(global.r(), worker);
                scope.execute_script(source);
            }

            let reporter_name = format!("worker-reporter-{}", random::<u64>());
            scope.mem_profiler_chan().run_with_memory_reporting(|| {
                while let Ok(event) = global.receive_event() {
                    global.handle_event(event);
                }
            }, reporter_name, parent_sender, CommonScriptMsg::CollectReports);
        });
    }

    pub fn script_chan(&self) -> Box<ScriptChan + Send> {
        box WorkerThreadWorkerChan {
            sender: self.own_sender.clone(),
            worker: self.worker.borrow().as_ref().unwrap().clone(),
        }
    }

    pub fn pipeline(&self) -> PipelineId {
        self.id
    }

    pub fn new_script_pair(&self) -> (Box<ScriptChan + Send>, Box<ScriptPort + Send>) {
        let (tx, rx) = channel();
        let chan = box SendableWorkerScriptChan {
            sender: tx,
            worker: self.worker.borrow().as_ref().unwrap().clone(),
        };
        (chan, box rx)
    }

    pub fn process_event(&self, msg: CommonScriptMsg) {
        self.handle_script_event(WorkerScriptMsg::Common(msg));
    }

    #[allow(unsafe_code)]
    fn receive_event(&self) -> Result<MixedMessage, RecvError> {
        let scope = self.upcast::<WorkerGlobalScope>();
        let worker_port = &self.receiver;
        let timer_event_port = &self.timer_event_port;
        let devtools_port = scope.from_devtools_receiver();

        let sel = Select::new();
        let mut worker_handle = sel.handle(worker_port);
        let mut timer_event_handle = sel.handle(timer_event_port);
        let mut devtools_handle = sel.handle(devtools_port);
        unsafe {
            worker_handle.add();
            timer_event_handle.add();
            if scope.from_devtools_sender().is_some() {
                devtools_handle.add();
            }
        }
        let ret = sel.wait();
        if ret == worker_handle.id() {
            Ok(MixedMessage::FromWorker(try!(worker_port.recv())))
        } else if ret == timer_event_handle.id() {
            Ok(MixedMessage::FromScheduler(try!(timer_event_port.recv())))
        } else if ret == devtools_handle.id() {
            Ok(MixedMessage::FromDevtools(try!(devtools_port.recv())))
        } else {
            panic!("unexpected select result!")
        }
    }

    fn handle_script_event(&self, msg: WorkerScriptMsg) {
        match msg {
            WorkerScriptMsg::DOMMessage(data) => {
                let scope = self.upcast::<WorkerGlobalScope>();
                let target = self.upcast();
                let _ar = JSAutoRequest::new(scope.get_cx());
                let _ac = JSAutoCompartment::new(scope.get_cx(), scope.reflector().get_jsobject().get());
                let mut message = RootedValue::new(scope.get_cx(), UndefinedValue());
                data.read(GlobalRef::Worker(scope), message.handle_mut());
                MessageEvent::dispatch_jsval(target, GlobalRef::Worker(scope), message.handle());
            },
            WorkerScriptMsg::Common(CommonScriptMsg::RunnableMsg(_, runnable)) => {
                runnable.handler()
            },
            WorkerScriptMsg::Common(CommonScriptMsg::RefcountCleanup(addr)) => {
                LiveDOMReferences::cleanup(addr);
            },
            WorkerScriptMsg::Common(CommonScriptMsg::CollectReports(reports_chan)) => {
                let scope = self.upcast::<WorkerGlobalScope>();
                let cx = scope.get_cx();
                let path_seg = format!("url({})", scope.get_url());
                let reports = ScriptTask::get_reports(cx, path_seg);
                reports_chan.send(reports);
            },
        }
    }

    fn handle_event(&self, event: MixedMessage) {
        match event {
            MixedMessage::FromDevtools(msg) => {
                let global_ref = GlobalRef::Worker(self.upcast());
                match msg {
                    DevtoolScriptControlMsg::EvaluateJS(_pipe_id, string, sender) =>
                        devtools::handle_evaluate_js(&global_ref, string, sender),
                    DevtoolScriptControlMsg::GetCachedMessages(pipe_id, message_types, sender) =>
                        devtools::handle_get_cached_messages(pipe_id, message_types, sender),
                    DevtoolScriptControlMsg::WantsLiveNotifications(_pipe_id, bool_val) =>
                        devtools::handle_wants_live_notifications(&global_ref, bool_val),
                    _ => debug!("got an unusable devtools control message inside the worker!"),
                }
            },
            MixedMessage::FromScheduler((linked_worker, timer_event)) => {
                match timer_event {
                    TimerEvent(TimerSource::FromWorker, id) => {
                        let _ar = AutoWorkerReset::new(self, linked_worker);
                        let scope = self.upcast::<WorkerGlobalScope>();
                        scope.handle_fire_timer(id);
                    },
                    TimerEvent(_, _) => {
                        panic!("A worker received a TimerEvent from a window.")
                    }
                }
            }
            MixedMessage::FromWorker((linked_worker, msg)) => {
                let _ar = AutoWorkerReset::new(self, linked_worker);
                self.handle_script_event(msg);
            },
        }
    }
}

impl DedicatedWorkerGlobalScopeMethods for DedicatedWorkerGlobalScope {
    // https://html.spec.whatwg.org/multipage/#dom-dedicatedworkerglobalscope-postmessage
    fn PostMessage(&self, cx: *mut JSContext, message: HandleValue) -> ErrorResult {
        let data = try!(StructuredCloneData::write(cx, message));
        let worker = self.worker.borrow().as_ref().unwrap().clone();
        self.parent_sender.send(CommonScriptMsg::RunnableMsg(WorkerEvent,
            box WorkerMessageHandler::new(worker, data))).unwrap();
        Ok(())
    }

    // https://html.spec.whatwg.org/multipage/#handler-dedicatedworkerglobalscope-onmessage
    event_handler!(message, GetOnmessage, SetOnmessage);
}

/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use devtools;
use devtools_traits::DevtoolScriptControlMsg;
use dom::abstractworker::{WorkerScriptLoadOrigin, WorkerScriptMsg, SharedRt, SimpleWorkerErrorHandler};
use dom::abstractworkerglobalscope::{SendableWorkerScriptChan, WorkerThreadWorkerChan};
use dom::bindings::cell::DOMRefCell;
use dom::bindings::codegen::Bindings::EventHandlerBinding::EventHandlerNonNull;
use dom::bindings::codegen::Bindings::ServiceWorkerGlobalScopeBinding;
use dom::bindings::codegen::Bindings::ServiceWorkerGlobalScopeBinding::ServiceWorkerGlobalScopeMethods;
use dom::bindings::global::{GlobalRef, global_root_from_context};
use dom::bindings::inheritance::Castable;
use dom::bindings::js::{Root, RootCollection};
use dom::bindings::refcounted::{Trusted, LiveDOMReferences};
use dom::bindings::reflector::Reflectable;
use dom::bindings::str::DOMString;
use dom::client::Client;
use dom::messageevent::MessageEvent;
use dom::serviceworker::TrustedServiceWorkerAddress;
use dom::workerglobalscope::WorkerGlobalScope;
use dom::workerglobalscope::WorkerGlobalScopeInit;
use ipc_channel::ipc::{self, IpcReceiver, IpcSender};
use ipc_channel::router::ROUTER;
use js::jsapi::{JS_SetInterruptCallback, JSAutoCompartment, JSContext};
use js::jsval::UndefinedValue;
use js::rust::Runtime;
use msg::constellation_msg::PipelineId;
use net_traits::{LoadContext, load_whole_resource, CustomResponse, IpcSend};
use rand::random;
use script_runtime::ScriptThreadEventCategory::ServiceWorkerEvent;
use script_runtime::{CommonScriptMsg, ScriptChan, ScriptPort, StackRootTLS, get_reports, new_rt_and_cx};
use script_traits::{TimerEvent, TimerSource};
use std::mem::replace;
use std::sync::mpsc::{Receiver, RecvError, Select, Sender, channel};
use std::sync::{Arc, Mutex};
use std::time::Instant;
use url::Url;
use util::prefs;
use util::thread::spawn_named;
use util::thread_state;
use util::thread_state::{IN_WORKER, SCRIPT};

/// Set the `worker` field of a related ServiceWorkerGlobalScope object to a particular
/// value for the duration of this object's lifetime. This ensures that the related Worker
/// object only lives as long as necessary (ie. while events are being executed), while
/// providing a reference that can be cloned freely.
struct AutoWorkerReset<'a> {
    workerscope: &'a ServiceWorkerGlobalScope,
    old_worker: Option<TrustedServiceWorkerAddress>,
}

impl<'a> AutoWorkerReset<'a> {
    fn new(workerscope: &'a ServiceWorkerGlobalScope,
           worker: TrustedServiceWorkerAddress)
           -> AutoWorkerReset<'a> {
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
    FromServiceWorker((TrustedServiceWorkerAddress, WorkerScriptMsg)),
    FromScheduler((TrustedServiceWorkerAddress, TimerEvent)),
    FromDevtools(DevtoolScriptControlMsg),
    FromNetwork(IpcSender<Option<CustomResponse>>),
}

#[dom_struct]
pub struct ServiceWorkerGlobalScope {
    workerglobalscope: WorkerGlobalScope,
    id: PipelineId,
    #[ignore_heap_size_of = "Defined in std"]
    receiver: Receiver<(TrustedServiceWorkerAddress, WorkerScriptMsg)>,
    #[ignore_heap_size_of = "Defined in std"]
    own_sender: Sender<(TrustedServiceWorkerAddress, WorkerScriptMsg)>,
    #[ignore_heap_size_of = "Defined in std"]
    timer_event_port: Receiver<(TrustedServiceWorkerAddress, TimerEvent)>,
    #[ignore_heap_size_of = "Trusted<T> has unclear ownership like JS<T>"]
    worker: DOMRefCell<Option<TrustedServiceWorkerAddress>>,
    #[ignore_heap_size_of = "Can't measure trait objects"]
    /// Sender to the parent thread.
    parent_sender: Box<ScriptChan + Send>,
    #[ignore_heap_size_of = "Defined in std"]
    service_worker_client: Trusted<Client>
}

impl ServiceWorkerGlobalScope {
    fn new_inherited(init: WorkerGlobalScopeInit,
                     worker_url: Url,
                     id: PipelineId,
                     from_devtools_receiver: Receiver<DevtoolScriptControlMsg>,
                     runtime: Runtime,
                     parent_sender: Box<ScriptChan + Send>,
                     own_sender: Sender<(TrustedServiceWorkerAddress, WorkerScriptMsg)>,
                     receiver: Receiver<(TrustedServiceWorkerAddress, WorkerScriptMsg)>,
                     timer_event_chan: IpcSender<TimerEvent>,
                     timer_event_port: Receiver<(TrustedServiceWorkerAddress, TimerEvent)>,
                     client: Trusted<Client>)
                     -> ServiceWorkerGlobalScope {
        ServiceWorkerGlobalScope {
            workerglobalscope: WorkerGlobalScope::new_inherited(init,
                                                                worker_url,
                                                                runtime,
                                                                from_devtools_receiver,
                                                                timer_event_chan),
            id: id,
            receiver: receiver,
            own_sender: own_sender,
            timer_event_port: timer_event_port,
            parent_sender: parent_sender,
            worker: DOMRefCell::new(None),
            service_worker_client: client
        }
    }

    pub fn new(init: WorkerGlobalScopeInit,
               worker_url: Url,
               id: PipelineId,
               from_devtools_receiver: Receiver<DevtoolScriptControlMsg>,
               runtime: Runtime,
               parent_sender: Box<ScriptChan + Send>,
               own_sender: Sender<(TrustedServiceWorkerAddress, WorkerScriptMsg)>,
               receiver: Receiver<(TrustedServiceWorkerAddress, WorkerScriptMsg)>,
               timer_event_chan: IpcSender<TimerEvent>,
               timer_event_port: Receiver<(TrustedServiceWorkerAddress, TimerEvent)>,
               client: Trusted<Client>)
               -> Root<ServiceWorkerGlobalScope> {
        let cx = runtime.cx();
        let scope = box ServiceWorkerGlobalScope::new_inherited(init,
                                                                  worker_url,
                                                                  id,
                                                                  from_devtools_receiver,
                                                                  runtime,
                                                                  parent_sender,
                                                                  own_sender,
                                                                  receiver,
                                                                  timer_event_chan,
                                                                  timer_event_port,
                                                                  client);
        ServiceWorkerGlobalScopeBinding::Wrap(cx, scope)
    }

    #[allow(unsafe_code)]
    pub fn run_serviceworker_scope(init: WorkerGlobalScopeInit,
                            worker_url: Url,
                            id: PipelineId,
                            from_devtools_receiver: IpcReceiver<DevtoolScriptControlMsg>,
                            main_thread_rt: Arc<Mutex<Option<SharedRt>>>,
                            worker: TrustedServiceWorkerAddress,
                            parent_sender: Box<ScriptChan + Send>,
                            own_sender: Sender<(TrustedServiceWorkerAddress, WorkerScriptMsg)>,
                            receiver: Receiver<(TrustedServiceWorkerAddress, WorkerScriptMsg)>,
                            client: Trusted<Client>,
                            worker_load_origin: WorkerScriptLoadOrigin) {
        let serialized_worker_url = worker_url.to_string();
        spawn_named(format!("ServiceWorker for {}", serialized_worker_url), move || {
            thread_state::initialize(SCRIPT | IN_WORKER);

            let roots = RootCollection::new();
            let _stack_roots_tls = StackRootTLS::new(&roots);

            let (url, source) = match load_whole_resource(LoadContext::Script,
                                                          &init.resource_threads.sender(),
                                                          worker_url,
                                                          &worker_load_origin) {
                Err(_) => {
                    println!("error loading script {}", serialized_worker_url);
                    parent_sender.send(CommonScriptMsg::RunnableMsg(ServiceWorkerEvent,
                        box SimpleWorkerErrorHandler::new(worker))).unwrap();
                    return;
                }
                Ok((metadata, bytes)) => {
                    (metadata.final_url, String::from_utf8(bytes).unwrap())
                }
            };

            let runtime = unsafe { new_rt_and_cx() };
            *main_thread_rt.lock().unwrap() = Some(SharedRt::new(&runtime));

            let (devtools_mpsc_chan, devtools_mpsc_port) = channel();
            ROUTER.route_ipc_receiver_to_mpsc_sender(from_devtools_receiver, devtools_mpsc_chan);

            let (timer_tx, timer_rx) = channel();
            let (timer_ipc_chan, timer_ipc_port) = ipc::channel().unwrap();
            let worker_for_route = worker.clone();
            ROUTER.add_route(timer_ipc_port.to_opaque(), box move |message| {
                let event = message.to().unwrap();
                timer_tx.send((worker_for_route.clone(), event)).unwrap();
            });

            let global = ServiceWorkerGlobalScope::new(
                init, url, id, devtools_mpsc_port, runtime,
                parent_sender.clone(), own_sender, receiver,
                timer_ipc_chan, timer_rx, client);
            // FIXME(njn): workers currently don't have a unique ID suitable for using in reporter
            // registration (#6631), so we instead use a random number and cross our fingers.
            let scope = global.upcast::<WorkerGlobalScope>();

            unsafe {
                // Handle interrupt requests
                JS_SetInterruptCallback(scope.runtime(), Some(interrupt_callback));
            }

            if scope.is_closing() {
                return;
            }

            {
                let _ar = AutoWorkerReset::new(global.r(), worker);
                scope.execute_script(DOMString::from(source));
            }


            let reporter_name = format!("service-worker-reporter-{}", random::<u64>());
            scope.mem_profiler_chan().run_with_memory_reporting(|| {
                // Service workers are time limited
                let sw_lifetime = Instant::now();
                let sw_lifetime_timeout = prefs::get_pref("dom.serviceworker.timeout_seconds").as_u64().unwrap();
                while let Ok(event) = global.receive_event() {
                    if scope.is_closing() {
                        break;
                    }
                    global.handle_event(event);
                    if sw_lifetime.elapsed().as_secs() == sw_lifetime_timeout {
                        break;
                    }
                }
            }, reporter_name, parent_sender, CommonScriptMsg::CollectReports);
        });
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
                        panic!("The service worker received a TimerEvent from a window.")
                    }
                }
            }
            MixedMessage::FromServiceWorker((linked_worker, msg)) => {
                let _ar = AutoWorkerReset::new(self, linked_worker);
                self.handle_script_event(msg);
            },
            MixedMessage::FromNetwork(network_sender) => {
                // We send None as of now
                let _ = network_sender.send(None);
            }
        }
    }

    fn handle_script_event(&self, msg: WorkerScriptMsg) {
        match msg {
            WorkerScriptMsg::DOMMessage(data) => {
                let scope = self.upcast::<WorkerGlobalScope>();
                let target = self.upcast();
                let _ac = JSAutoCompartment::new(scope.get_cx(),
                                                 scope.reflector().get_jsobject().get());
                rooted!(in(scope.get_cx()) let mut message = UndefinedValue());
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
                let reports = get_reports(cx, path_seg);
                reports_chan.send(reports);
            },
        }
    }

    #[allow(unsafe_code)]
    fn receive_event(&self) -> Result<MixedMessage, RecvError> {
        let scope = self.upcast::<WorkerGlobalScope>();
        let worker_port = &self.receiver;
        let timer_event_port = &self.timer_event_port;
        let devtools_port = scope.from_devtools_receiver();
        let msg_port = scope.custom_message_port();

        let sel = Select::new();
        let mut worker_handle = sel.handle(worker_port);
        let mut timer_event_handle = sel.handle(timer_event_port);
        let mut devtools_handle = sel.handle(devtools_port);
        let mut msg_port_handle = sel.handle(msg_port);
        unsafe {
            worker_handle.add();
            timer_event_handle.add();
            if scope.from_devtools_sender().is_some() {
                devtools_handle.add();
            }
            msg_port_handle.add();
        }
        let ret = sel.wait();
        if ret == worker_handle.id() {
            Ok(MixedMessage::FromServiceWorker(try!(worker_port.recv())))
        } else if ret == timer_event_handle.id() {
            Ok(MixedMessage::FromScheduler(try!(timer_event_port.recv())))
        } else if ret == devtools_handle.id() {
            Ok(MixedMessage::FromDevtools(try!(devtools_port.recv())))
        } else if ret == msg_port_handle.id() {
            Ok(MixedMessage::FromNetwork(try!(msg_port.recv())))
        } else {
            panic!("unexpected select result!")
        }
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

    pub fn process_event(&self, msg: CommonScriptMsg) {
        self.handle_script_event(WorkerScriptMsg::Common(msg));
    }

    pub fn new_script_pair(&self) -> (Box<ScriptChan + Send>, Box<ScriptPort + Send>) {
        let (tx, rx) = channel();
        let chan = box SendableWorkerScriptChan {
            sender: tx,
            worker: self.worker.borrow().as_ref().unwrap().clone(),
        };
        (chan, box rx)
    }
}

#[allow(unsafe_code)]
unsafe extern "C" fn interrupt_callback(cx: *mut JSContext) -> bool {
    let global = global_root_from_context(cx);
    let worker = match global.r() {
        GlobalRef::Worker(w) => w,
        _ => panic!("global for worker is not a worker scope")
    };
    assert!(worker.is::<ServiceWorkerGlobalScope>());

    // A false response causes the script to terminate
    !worker.is_closing()
}

impl ServiceWorkerGlobalScopeMethods for ServiceWorkerGlobalScope {
    // https://slightlyoff.github.io/ServiceWorker/spec/service_worker/#service-worker-global-scope-onmessage-attribute
    event_handler!(message, GetOnmessage, SetOnmessage);
}

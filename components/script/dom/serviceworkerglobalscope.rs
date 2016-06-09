/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use devtools;
use devtools_traits::DevtoolScriptControlMsg;
use dom::abstractworker::WorkerScriptMsg;
use dom::bindings::cell::DOMRefCell;
use dom::bindings::codegen::Bindings::EventHandlerBinding::EventHandlerNonNull;
use dom::bindings::codegen::Bindings::ServiceWorkerGlobalScopeBinding;
use dom::bindings::codegen::Bindings::ServiceWorkerGlobalScopeBinding::ServiceWorkerGlobalScopeMethods;
use dom::bindings::global::{GlobalRef, global_root_from_context};
use dom::bindings::inheritance::Castable;
use dom::bindings::js::{Root, RootCollection};
use dom::bindings::refcounted::LiveDOMReferences;
use dom::bindings::reflector::Reflectable;
use dom::bindings::str::DOMString;
use dom::messageevent::MessageEvent;
use dom::serviceworker::TrustedServiceWorkerAddress;
use dom::workerglobalscope::WorkerGlobalScope;
use ipc_channel::ipc::{self, IpcSender};
use ipc_channel::router::ROUTER;
use js::jsapi::{JS_SetInterruptCallback, JSAutoCompartment, JSContext, RootedValue};
use js::jsval::UndefinedValue;
use js::rust::Runtime;
use msg::constellation_msg::PipelineId;
use net_traits::{LoadContext, load_whole_resource, CustomResponse, IpcSend};
use script_runtime::{CommonScriptMsg, StackRootTLS, get_reports, new_rt_and_cx};
use script_traits::{TimerEvent, WorkerGlobalScopeInit, ScopeThings, ServiceWorkerMsg};
use std::sync::atomic::Ordering;
use std::sync::mpsc::{Receiver, RecvError, Select, Sender, channel};
use std::time::Instant;
use url::Url;
use util::prefs;
use util::thread::spawn_named;
use util::thread_state;
use util::thread_state::{IN_WORKER, SCRIPT};

pub enum MixedMessage {
    FromServiceWorker((TrustedServiceWorkerAddress, WorkerScriptMsg)),
    FromDevtools(DevtoolScriptControlMsg),
    FromNetwork(IpcSender<Option<CustomResponse>>),
    FromTimeoutThread
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
    timer_event_port: Receiver<MixedMessage>,
    #[ignore_heap_size_of = "Trusted<T> has unclear ownership like JS<T>"]
    worker: DOMRefCell<Option<TrustedServiceWorkerAddress>>
}

impl ServiceWorkerGlobalScope {
    fn new_inherited(init: WorkerGlobalScopeInit,
                     worker_url: Url,
                     id: PipelineId,
                     from_devtools_receiver: Receiver<DevtoolScriptControlMsg>,
                     runtime: Runtime,
                     own_sender: Sender<(TrustedServiceWorkerAddress, WorkerScriptMsg)>,
                     receiver: Receiver<(TrustedServiceWorkerAddress, WorkerScriptMsg)>,
                     timer_event_chan: IpcSender<TimerEvent>,
                     timer_event_port: Receiver<MixedMessage>)
                     -> ServiceWorkerGlobalScope {
        ServiceWorkerGlobalScope {
            workerglobalscope: WorkerGlobalScope::new_inherited(init,
                                                                worker_url,
                                                                runtime,
                                                                from_devtools_receiver,
                                                                timer_event_chan),
            id: id,
            receiver: receiver,
            timer_event_port: timer_event_port,
            own_sender: own_sender,
            worker: DOMRefCell::new(None)
        }
    }

    pub fn new(init: WorkerGlobalScopeInit,
               worker_url: Url,
               id: PipelineId,
               from_devtools_receiver: Receiver<DevtoolScriptControlMsg>,
               runtime: Runtime,
               own_sender: Sender<(TrustedServiceWorkerAddress, WorkerScriptMsg)>,
               receiver: Receiver<(TrustedServiceWorkerAddress, WorkerScriptMsg)>,
               timer_event_chan: IpcSender<TimerEvent>,
               timer_event_port: Receiver<MixedMessage>)
               -> Root<ServiceWorkerGlobalScope> {
        let cx = runtime.cx();
        let scope = box ServiceWorkerGlobalScope::new_inherited(init,
                                                                  worker_url,
                                                                  id,
                                                                  from_devtools_receiver,
                                                                  runtime,
                                                                  own_sender,
                                                                  receiver,
                                                                  timer_event_chan,
                                                                  timer_event_port);
        ServiceWorkerGlobalScopeBinding::Wrap(cx, scope)
    }

    #[allow(unsafe_code)]
    pub fn run_serviceworker_scope(scope_things: ScopeThings,
                            own_sender: Sender<(TrustedServiceWorkerAddress, WorkerScriptMsg)>,
                            receiver: Receiver<(TrustedServiceWorkerAddress, WorkerScriptMsg)>,
                            manager_sender: IpcSender<ServiceWorkerMsg>,
                            scope_url: Url) {
        let ScopeThings { script_url, pipeline_id,
                          init,
                          worker_load_origin,
                          devtools_receiver } = scope_things;
        let serialized_worker_url = script_url.to_string();

        spawn_named(format!("ServiceWorker for {}", serialized_worker_url), move || {
            thread_state::initialize(SCRIPT | IN_WORKER);

            let roots = RootCollection::new();
            let _stack_roots_tls = StackRootTLS::new(&roots);

            let (url, source) = match load_whole_resource(LoadContext::Script,
                                                          &init.resource_threads.sender(),
                                                          script_url,
                                                          &worker_load_origin) {
                Err(_) => {
                    println!("error loading script {}", serialized_worker_url);
                    return;
                }
                Ok((metadata, bytes)) => {
                    (metadata.final_url, String::from_utf8(bytes).unwrap())
                }
            };

            let runtime = unsafe { new_rt_and_cx() };

            let (devtools_mpsc_chan, devtools_mpsc_port) = channel();
            ROUTER.route_ipc_receiver_to_mpsc_sender(devtools_receiver, devtools_mpsc_chan);

            let (timer_ipc_chan, timer_ipc_port) = ipc::channel().unwrap();
            let (timer_chan, timer_port) = channel();

            let global = ServiceWorkerGlobalScope::new(
                init, url, pipeline_id, devtools_mpsc_port, runtime,
                own_sender, receiver,
                timer_ipc_chan, timer_port);
            let scope = global.upcast::<WorkerGlobalScope>();

            unsafe {
                // Handle interrupt requests
                JS_SetInterruptCallback(scope.runtime(), Some(interrupt_callback));
            }

            if scope.is_closing() {
                return;
            }

            scope.execute_script(DOMString::from(source));

            // Service workers are time limited
            spawn_named("TimeoutThread".to_owned(), move || {
                let sw_lifetime = Instant::now();
                let sw_lifetime_timeout = prefs::get_pref("dom.serviceworker.timeout_seconds").as_u64().unwrap();
                loop {
                if sw_lifetime.elapsed().as_secs() == sw_lifetime_timeout {
                        timer_chan.send(MixedMessage::FromTimeoutThread).unwrap();
                        manager_sender.send(ServiceWorkerMsg::Timeout(scope_url.clone())).unwrap();
                        break;
                    }
                }
            });

            while let Ok(event) = global.receive_event() {
                if scope.is_closing() {
                    break;
                }
                global.handle_event(event);
            }
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
            }
            MixedMessage::FromServiceWorker((_, msg)) => {
                self.handle_script_event(msg);
            }
            MixedMessage::FromTimeoutThread => {
                let scope = self.upcast::<WorkerGlobalScope>();
                scope.get_closing().swap(true, Ordering::SeqCst);
            }
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
                let reports = get_reports(cx, path_seg);
                reports_chan.send(reports);
            },
        }
    }

    #[allow(unsafe_code)]
    fn receive_event(&self) -> Result<MixedMessage, RecvError> {
        let scope = self.upcast::<WorkerGlobalScope>();
        let worker_port = &self.receiver;
        let devtools_port = scope.from_devtools_receiver();
        let msg_port = scope.custom_message_port();
        let timer_event_port = &self.timer_event_port;

        let sel = Select::new();
        let mut worker_handle = sel.handle(worker_port);
        let mut devtools_handle = sel.handle(devtools_port);
        let mut msg_port_handle = sel.handle(msg_port);
        let mut timer_port_handle = sel.handle(timer_event_port);
        unsafe {
            worker_handle.add();
            if scope.from_devtools_sender().is_some() {
                devtools_handle.add();
            }
            msg_port_handle.add();
            timer_port_handle.add();
        }

        let ret = sel.wait();
        if ret == worker_handle.id() {
            Ok(MixedMessage::FromServiceWorker(try!(worker_port.recv())))
        }else if ret == devtools_handle.id() {
            Ok(MixedMessage::FromDevtools(try!(devtools_port.recv())))
        } else if ret == msg_port_handle.id() {
            Ok(MixedMessage::FromNetwork(try!(msg_port.recv())))
        } else if ret == timer_port_handle.id() {
            Ok(MixedMessage::FromTimeoutThread)
        } else {
            panic!("unexpected select result!")
        }
    }

    pub fn pipeline(&self) -> PipelineId {
        self.id
    }

    pub fn process_event(&self, msg: CommonScriptMsg) {
        self.handle_script_event(WorkerScriptMsg::Common(msg));
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

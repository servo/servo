/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use devtools;
use devtools_traits::DevtoolScriptControlMsg;
use dom::abstractworker::{SharedRt, SimpleWorkerErrorHandler, WorkerScriptMsg};
use dom::abstractworkerglobalscope::{SendableWorkerScriptChan, WorkerThreadWorkerChan};
use dom::bindings::cell::DOMRefCell;
use dom::bindings::codegen::Bindings::DedicatedWorkerGlobalScopeBinding;
use dom::bindings::codegen::Bindings::DedicatedWorkerGlobalScopeBinding::DedicatedWorkerGlobalScopeMethods;
use dom::bindings::codegen::Bindings::EventHandlerBinding::EventHandlerNonNull;
use dom::bindings::error::{ErrorInfo, ErrorResult};
use dom::bindings::inheritance::Castable;
use dom::bindings::js::{Root, RootCollection};
use dom::bindings::reflector::DomObject;
use dom::bindings::str::DOMString;
use dom::bindings::structuredclone::StructuredCloneData;
use dom::globalscope::GlobalScope;
use dom::messageevent::MessageEvent;
use dom::worker::{TrustedWorkerAddress, WorkerErrorHandler, WorkerMessageHandler};
use dom::workerglobalscope::WorkerGlobalScope;
use dom_struct::dom_struct;
use ipc_channel::ipc::{self, IpcReceiver, IpcSender};
use ipc_channel::router::ROUTER;
use js::jsapi::{HandleValue, JS_SetInterruptCallback};
use js::jsapi::{JSAutoCompartment, JSContext};
use js::jsval::UndefinedValue;
use js::rust::Runtime;
use msg::constellation_msg::TopLevelBrowsingContextId;
use net_traits::{IpcSend, load_whole_resource};
use net_traits::request::{CredentialsMode, Destination, RequestInit, Type as RequestType};
use script_runtime::{CommonScriptMsg, ScriptChan, ScriptPort, StackRootTLS, get_reports, new_rt_and_cx};
use script_runtime::ScriptThreadEventCategory::WorkerEvent;
use script_traits::{TimerEvent, TimerSource, WorkerGlobalScopeInit, WorkerScriptLoadOrigin};
use servo_rand::random;
use servo_url::ServoUrl;
use std::mem::replace;
use std::sync::{Arc, Mutex};
use std::sync::atomic::AtomicBool;
use std::sync::mpsc::{Receiver, RecvError, Select, Sender, channel};
use std::thread;
use style::thread_state;

/// Set the `worker` field of a related DedicatedWorkerGlobalScope object to a particular
/// value for the duration of this object's lifetime. This ensures that the related Worker
/// object only lives as long as necessary (ie. while events are being executed), while
/// providing a reference that can be cloned freely.
struct AutoWorkerReset<'a> {
    workerscope: &'a DedicatedWorkerGlobalScope,
    old_worker: Option<TrustedWorkerAddress>,
}

impl<'a> AutoWorkerReset<'a> {
    fn new(workerscope: &'a DedicatedWorkerGlobalScope,
           worker: TrustedWorkerAddress)
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
    FromWorker((TrustedWorkerAddress, WorkerScriptMsg)),
    FromScheduler((TrustedWorkerAddress, TimerEvent)),
    FromDevtools(DevtoolScriptControlMsg)
}

// https://html.spec.whatwg.org/multipage/#dedicatedworkerglobalscope
#[dom_struct]
pub struct DedicatedWorkerGlobalScope {
    workerglobalscope: WorkerGlobalScope,
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
                     worker_url: ServoUrl,
                     from_devtools_receiver: Receiver<DevtoolScriptControlMsg>,
                     runtime: Runtime,
                     parent_sender: Box<ScriptChan + Send>,
                     own_sender: Sender<(TrustedWorkerAddress, WorkerScriptMsg)>,
                     receiver: Receiver<(TrustedWorkerAddress, WorkerScriptMsg)>,
                     timer_event_chan: IpcSender<TimerEvent>,
                     timer_event_port: Receiver<(TrustedWorkerAddress, TimerEvent)>,
                     closing: Arc<AtomicBool>)
                     -> DedicatedWorkerGlobalScope {
        DedicatedWorkerGlobalScope {
            workerglobalscope: WorkerGlobalScope::new_inherited(init,
                                                                worker_url,
                                                                runtime,
                                                                from_devtools_receiver,
                                                                timer_event_chan,
                                                                Some(closing)),
            receiver: receiver,
            own_sender: own_sender,
            timer_event_port: timer_event_port,
            parent_sender: parent_sender,
            worker: DOMRefCell::new(None),
        }
    }

    #[allow(unsafe_code)]
    pub fn new(init: WorkerGlobalScopeInit,
               worker_url: ServoUrl,
               from_devtools_receiver: Receiver<DevtoolScriptControlMsg>,
               runtime: Runtime,
               parent_sender: Box<ScriptChan + Send>,
               own_sender: Sender<(TrustedWorkerAddress, WorkerScriptMsg)>,
               receiver: Receiver<(TrustedWorkerAddress, WorkerScriptMsg)>,
               timer_event_chan: IpcSender<TimerEvent>,
               timer_event_port: Receiver<(TrustedWorkerAddress, TimerEvent)>,
               closing: Arc<AtomicBool>)
               -> Root<DedicatedWorkerGlobalScope> {
        let cx = runtime.cx();
        let scope = box DedicatedWorkerGlobalScope::new_inherited(init,
                                                                  worker_url,
                                                                  from_devtools_receiver,
                                                                  runtime,
                                                                  parent_sender,
                                                                  own_sender,
                                                                  receiver,
                                                                  timer_event_chan,
                                                                  timer_event_port,
                                                                  closing);
        unsafe {
            DedicatedWorkerGlobalScopeBinding::Wrap(cx, scope)
        }
    }

    #[allow(unsafe_code)]
    pub fn run_worker_scope(init: WorkerGlobalScopeInit,
                            worker_url: ServoUrl,
                            from_devtools_receiver: IpcReceiver<DevtoolScriptControlMsg>,
                            worker_rt_for_mainthread: Arc<Mutex<Option<SharedRt>>>,
                            worker: TrustedWorkerAddress,
                            parent_sender: Box<ScriptChan + Send>,
                            own_sender: Sender<(TrustedWorkerAddress, WorkerScriptMsg)>,
                            receiver: Receiver<(TrustedWorkerAddress, WorkerScriptMsg)>,
                            worker_load_origin: WorkerScriptLoadOrigin,
                            closing: Arc<AtomicBool>) {
        let serialized_worker_url = worker_url.to_string();
        let name = format!("WebWorker for {}", serialized_worker_url);
        let top_level_browsing_context_id = TopLevelBrowsingContextId::installed();

        thread::Builder::new().name(name).spawn(move || {
            thread_state::initialize(thread_state::SCRIPT | thread_state::IN_WORKER);

            if let Some(top_level_browsing_context_id) = top_level_browsing_context_id {
                TopLevelBrowsingContextId::install(top_level_browsing_context_id);
            }

            let roots = RootCollection::new();
            let _stack_roots_tls = StackRootTLS::new(&roots);

            let WorkerScriptLoadOrigin { referrer_url, referrer_policy, pipeline_id } = worker_load_origin;

            let request = RequestInit {
                url: worker_url.clone(),
                type_: RequestType::Script,
                destination: Destination::Worker,
                credentials_mode: CredentialsMode::Include,
                use_url_credentials: true,
                origin: worker_url,
                pipeline_id: pipeline_id,
                referrer_url: referrer_url,
                referrer_policy: referrer_policy,
                .. RequestInit::default()
            };

            let (metadata, bytes) = match load_whole_resource(request,
                                                              &init.resource_threads.sender()) {
                Err(_) => {
                    println!("error loading script {}", serialized_worker_url);
                    parent_sender.send(CommonScriptMsg::RunnableMsg(WorkerEvent,
                        box SimpleWorkerErrorHandler::new(worker))).unwrap();
                    return;
                }
                Ok((metadata, bytes)) => (metadata, bytes)
            };
            let url = metadata.final_url;
            let source = String::from_utf8_lossy(&bytes);

            let runtime = unsafe { new_rt_and_cx() };
            *worker_rt_for_mainthread.lock().unwrap() = Some(SharedRt::new(&runtime));

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
                init, url, devtools_mpsc_port, runtime,
                parent_sender.clone(), own_sender, receiver,
                timer_ipc_chan, timer_rx, closing);
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
                let _ar = AutoWorkerReset::new(&global, worker.clone());
                scope.execute_script(DOMString::from(source));
            }

            let reporter_name = format!("dedicated-worker-reporter-{}", random::<u64>());
            scope.upcast::<GlobalScope>().mem_profiler_chan().run_with_memory_reporting(|| {
                // https://html.spec.whatwg.org/multipage/#event-loop-processing-model
                // Step 1
                while let Ok(event) = global.receive_event() {
                    if scope.is_closing() {
                        break;
                    }
                    // Step 3
                    global.handle_event(event);
                    // Step 6
                    let _ar = AutoWorkerReset::new(&global, worker.clone());
                    global.upcast::<WorkerGlobalScope>().perform_a_microtask_checkpoint();
                }
            }, reporter_name, parent_sender, CommonScriptMsg::CollectReports);
        }).expect("Thread spawning failed");
    }

    pub fn script_chan(&self) -> Box<ScriptChan + Send> {
        box WorkerThreadWorkerChan {
            sender: self.own_sender.clone(),
            worker: self.worker.borrow().as_ref().unwrap().clone(),
        }
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
            Ok(MixedMessage::FromWorker(worker_port.recv()?))
        } else if ret == timer_event_handle.id() {
            Ok(MixedMessage::FromScheduler(timer_event_port.recv()?))
        } else if ret == devtools_handle.id() {
            Ok(MixedMessage::FromDevtools(devtools_port.recv()?))
        } else {
            panic!("unexpected select result!")
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
                data.read(scope.upcast(), message.handle_mut());
                MessageEvent::dispatch_jsval(target, scope.upcast(), message.handle());
            },
            WorkerScriptMsg::Common(CommonScriptMsg::RunnableMsg(_, runnable)) => {
                runnable.handler()
            },
            WorkerScriptMsg::Common(CommonScriptMsg::CollectReports(reports_chan)) => {
                let scope = self.upcast::<WorkerGlobalScope>();
                let cx = scope.get_cx();
                let path_seg = format!("url({})", scope.get_url());
                let reports = get_reports(cx, path_seg);
                reports_chan.send(reports);
            }
        }
    }

    fn handle_event(&self, event: MixedMessage) {
        match event {
            MixedMessage::FromDevtools(msg) => {
                match msg {
                    DevtoolScriptControlMsg::EvaluateJS(_pipe_id, string, sender) =>
                        devtools::handle_evaluate_js(self.upcast(), string, sender),
                    DevtoolScriptControlMsg::GetCachedMessages(pipe_id, message_types, sender) =>
                        devtools::handle_get_cached_messages(pipe_id, message_types, sender),
                    DevtoolScriptControlMsg::WantsLiveNotifications(_pipe_id, bool_val) =>
                        devtools::handle_wants_live_notifications(self.upcast(), bool_val),
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
            }
        }
    }

    pub fn forward_error_to_worker_object(&self, error_info: ErrorInfo) {
        let worker = self.worker.borrow().as_ref().unwrap().clone();
        // TODO: Should use the DOM manipulation task source.
        self.parent_sender
            .send(CommonScriptMsg::RunnableMsg(WorkerEvent,
                                               box WorkerErrorHandler::new(worker, error_info)))
            .unwrap();
    }
}

#[allow(unsafe_code)]
unsafe extern "C" fn interrupt_callback(cx: *mut JSContext) -> bool {
    let worker =
        Root::downcast::<WorkerGlobalScope>(GlobalScope::from_context(cx))
            .expect("global is not a worker scope");
    assert!(worker.is::<DedicatedWorkerGlobalScope>());

    // A false response causes the script to terminate
    !worker.is_closing()
}

impl DedicatedWorkerGlobalScopeMethods for DedicatedWorkerGlobalScope {
    #[allow(unsafe_code)]
    // https://html.spec.whatwg.org/multipage/#dom-dedicatedworkerglobalscope-postmessage
    unsafe fn PostMessage(&self, cx: *mut JSContext, message: HandleValue) -> ErrorResult {
        let data = StructuredCloneData::write(cx, message)?;
        let worker = self.worker.borrow().as_ref().unwrap().clone();
        self.parent_sender
            .send(CommonScriptMsg::RunnableMsg(WorkerEvent,
                                               box WorkerMessageHandler::new(worker, data)))
            .unwrap();
        Ok(())
    }

    // https://html.spec.whatwg.org/multipage/#dom-dedicatedworkerglobalscope-close
    fn Close(&self) {
        // Step 2
        self.upcast::<WorkerGlobalScope>().close();
    }

    // https://html.spec.whatwg.org/multipage/#handler-dedicatedworkerglobalscope-onmessage
    event_handler!(message, GetOnmessage, SetOnmessage);
}

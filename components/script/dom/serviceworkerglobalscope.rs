/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use devtools;
use devtools_traits::DevtoolScriptControlMsg;
use dom::abstractworker::WorkerScriptMsg;
use dom::abstractworkerglobalscope::{WorkerEventLoopMethods, run_worker_event_loop};
use dom::bindings::codegen::Bindings::ServiceWorkerGlobalScopeBinding;
use dom::bindings::codegen::Bindings::ServiceWorkerGlobalScopeBinding::ServiceWorkerGlobalScopeMethods;
use dom::bindings::inheritance::Castable;
use dom::bindings::reflector::DomObject;
use dom::bindings::root::{DomRoot, RootCollection, ThreadLocalStackRoots};
use dom::bindings::str::DOMString;
use dom::dedicatedworkerglobalscope::AutoWorkerReset;
use dom::event::Event;
use dom::eventtarget::EventTarget;
use dom::extendableevent::ExtendableEvent;
use dom::extendablemessageevent::ExtendableMessageEvent;
use dom::globalscope::GlobalScope;
use dom::worker::TrustedWorkerAddress;
use dom::workerglobalscope::WorkerGlobalScope;
use dom_struct::dom_struct;
use ipc_channel::ipc::{self, IpcSender, IpcReceiver};
use js::jsapi::{JSAutoCompartment, JSContext, JS_AddInterruptCallback};
use js::jsval::UndefinedValue;
use net_traits::{load_whole_resource, IpcSend, CustomResponseMediator};
use net_traits::request::{CredentialsMode, Destination, RequestInit};
use script_runtime::{CommonScriptMsg, ScriptChan, new_rt_and_cx, Runtime};
use script_traits::{TimerEvent, WorkerGlobalScopeInit, ScopeThings, ServiceWorkerMsg, WorkerScriptLoadOrigin};
use servo_channel::{channel, route_ipc_receiver_to_new_servo_sender, Receiver, Sender};
use servo_config::prefs::PREFS;
use servo_rand::random;
use servo_url::ServoUrl;
use std::thread;
use std::time::Duration;
use style::thread_state::{self, ThreadState};
use task_queue::{QueuedTask, QueuedTaskConversion, TaskQueue};
use task_source::TaskSourceName;

/// Messages used to control service worker event loop
pub enum ServiceWorkerScriptMsg {
    /// Message common to all workers
    CommonWorker(WorkerScriptMsg),
    /// Message to request a custom response by the service worker
    Response(CustomResponseMediator),
    /// Wake-up call from the task queue.
    WakeUp,
}

impl QueuedTaskConversion for ServiceWorkerScriptMsg {
    fn task_source_name(&self) -> Option<&TaskSourceName> {
        let script_msg = match self {
            ServiceWorkerScriptMsg::CommonWorker(WorkerScriptMsg::Common(script_msg)) => script_msg,
            _ => return None,
        };
        match script_msg {
            CommonScriptMsg::Task(_category, _boxed, _pipeline_id, task_source) => {
                Some(&task_source)
            },
            _ => return None,
        }
    }

    fn into_queued_task(self) -> Option<QueuedTask> {
        let script_msg = match self {
            ServiceWorkerScriptMsg::CommonWorker(WorkerScriptMsg::Common(script_msg)) => script_msg,
            _ => return None,
        };
        let (category, boxed, pipeline_id, task_source) = match script_msg {
            CommonScriptMsg::Task(category, boxed, pipeline_id, task_source) => {
                (category, boxed, pipeline_id, task_source)
            },
            _ => return None,
        };
        Some((None, category, boxed, pipeline_id, task_source))
    }

    fn from_queued_task(queued_task: QueuedTask) -> Self {
        let (_worker, category, boxed, pipeline_id, task_source) = queued_task;
        let script_msg = CommonScriptMsg::Task(category, boxed, pipeline_id, task_source);
        ServiceWorkerScriptMsg::CommonWorker(WorkerScriptMsg::Common(script_msg))
    }

    fn wake_up_msg() -> Self {
        ServiceWorkerScriptMsg::WakeUp
    }

    fn is_wake_up(&self) -> bool {
        match self {
            ServiceWorkerScriptMsg::WakeUp => true,
            _ => false,
        }
    }
}

pub enum MixedMessage {
    FromServiceWorker(ServiceWorkerScriptMsg),
    FromDevtools(DevtoolScriptControlMsg),
    FromTimeoutThread(()),
}

#[derive(Clone, JSTraceable)]
pub struct ServiceWorkerChan {
    pub sender: Sender<ServiceWorkerScriptMsg>,
}

impl ScriptChan for ServiceWorkerChan {
    fn send(&self, msg: CommonScriptMsg) -> Result<(), ()> {
        self.sender
            .send(ServiceWorkerScriptMsg::CommonWorker(
                WorkerScriptMsg::Common(msg),
            ))
            .map_err(|_| ())
    }

    fn clone(&self) -> Box<ScriptChan + Send> {
        Box::new(ServiceWorkerChan {
            sender: self.sender.clone(),
        })
    }
}

unsafe_no_jsmanaged_fields!(TaskQueue<ServiceWorkerScriptMsg>);

#[dom_struct]
pub struct ServiceWorkerGlobalScope {
    workerglobalscope: WorkerGlobalScope,
    #[ignore_malloc_size_of = "Defined in std"]
    task_queue: TaskQueue<ServiceWorkerScriptMsg>,
    #[ignore_malloc_size_of = "Defined in std"]
    own_sender: Sender<ServiceWorkerScriptMsg>,
    #[ignore_malloc_size_of = "Defined in std"]
    timer_event_port: Receiver<()>,
    #[ignore_malloc_size_of = "Defined in std"]
    swmanager_sender: IpcSender<ServiceWorkerMsg>,
    scope_url: ServoUrl,
}

impl WorkerEventLoopMethods for ServiceWorkerGlobalScope {
    type TimerMsg = ();
    type WorkerMsg = ServiceWorkerScriptMsg;
    type Event = MixedMessage;

    fn timer_event_port(&self) -> &Receiver<()> {
        &self.timer_event_port
    }

    fn task_queue(&self) -> &TaskQueue<ServiceWorkerScriptMsg> {
        &self.task_queue
    }

    fn handle_event(&self, event: MixedMessage) {
        self.handle_mixed_message(event);
    }

    fn handle_worker_post_event(&self, _worker: &TrustedWorkerAddress) -> Option<AutoWorkerReset> {
        None
    }

    fn from_worker_msg(&self, msg: ServiceWorkerScriptMsg) -> MixedMessage {
        MixedMessage::FromServiceWorker(msg)
    }

    fn from_timer_msg(&self, msg: ()) -> MixedMessage {
        MixedMessage::FromTimeoutThread(msg)
    }

    fn from_devtools_msg(&self, msg: DevtoolScriptControlMsg) -> MixedMessage {
        MixedMessage::FromDevtools(msg)
    }
}

impl ServiceWorkerGlobalScope {
    fn new_inherited(
        init: WorkerGlobalScopeInit,
        worker_url: ServoUrl,
        from_devtools_receiver: Receiver<DevtoolScriptControlMsg>,
        runtime: Runtime,
        own_sender: Sender<ServiceWorkerScriptMsg>,
        receiver: Receiver<ServiceWorkerScriptMsg>,
        timer_event_chan: IpcSender<TimerEvent>,
        timer_event_port: Receiver<()>,
        swmanager_sender: IpcSender<ServiceWorkerMsg>,
        scope_url: ServoUrl,
    ) -> ServiceWorkerGlobalScope {
        ServiceWorkerGlobalScope {
            workerglobalscope: WorkerGlobalScope::new_inherited(
                init,
                worker_url,
                runtime,
                from_devtools_receiver,
                timer_event_chan,
                None,
            ),
            task_queue: TaskQueue::new(receiver, own_sender.clone()),
            timer_event_port: timer_event_port,
            own_sender: own_sender,
            swmanager_sender: swmanager_sender,
            scope_url: scope_url,
        }
    }

    #[allow(unsafe_code)]
    pub fn new(
        init: WorkerGlobalScopeInit,
        worker_url: ServoUrl,
        from_devtools_receiver: Receiver<DevtoolScriptControlMsg>,
        runtime: Runtime,
        own_sender: Sender<ServiceWorkerScriptMsg>,
        receiver: Receiver<ServiceWorkerScriptMsg>,
        timer_event_chan: IpcSender<TimerEvent>,
        timer_event_port: Receiver<()>,
        swmanager_sender: IpcSender<ServiceWorkerMsg>,
        scope_url: ServoUrl,
    ) -> DomRoot<ServiceWorkerGlobalScope> {
        let cx = runtime.cx();
        let scope = Box::new(ServiceWorkerGlobalScope::new_inherited(
            init,
            worker_url,
            from_devtools_receiver,
            runtime,
            own_sender,
            receiver,
            timer_event_chan,
            timer_event_port,
            swmanager_sender,
            scope_url,
        ));
        unsafe { ServiceWorkerGlobalScopeBinding::Wrap(cx, scope) }
    }

    #[allow(unsafe_code)]
    // https://html.spec.whatwg.org/multipage/#run-a-worker
    pub fn run_serviceworker_scope(
        scope_things: ScopeThings,
        own_sender: Sender<ServiceWorkerScriptMsg>,
        receiver: Receiver<ServiceWorkerScriptMsg>,
        devtools_receiver: IpcReceiver<DevtoolScriptControlMsg>,
        swmanager_sender: IpcSender<ServiceWorkerMsg>,
        scope_url: ServoUrl,
    ) {
        let ScopeThings {
            script_url,
            init,
            worker_load_origin,
            ..
        } = scope_things;

        let serialized_worker_url = script_url.to_string();
        let origin = GlobalScope::current()
            .expect("No current global object")
            .origin()
            .immutable()
            .clone();
        thread::Builder::new()
            .name(format!("ServiceWorker for {}", serialized_worker_url))
            .spawn(move || {
                thread_state::initialize(ThreadState::SCRIPT | ThreadState::IN_WORKER);
                let roots = RootCollection::new();
                let _stack_roots = ThreadLocalStackRoots::new(&roots);

                let WorkerScriptLoadOrigin {
                    referrer_url,
                    referrer_policy,
                    pipeline_id,
                } = worker_load_origin;

                let request = RequestInit {
                    url: script_url.clone(),
                    destination: Destination::ServiceWorker,
                    credentials_mode: CredentialsMode::Include,
                    use_url_credentials: true,
                    pipeline_id: pipeline_id,
                    referrer_url: referrer_url,
                    referrer_policy: referrer_policy,
                    origin,
                    ..RequestInit::default()
                };

                let (url, source) =
                    match load_whole_resource(request, &init.resource_threads.sender()) {
                        Err(_) => {
                            println!("error loading script {}", serialized_worker_url);
                            return;
                        },
                        Ok((metadata, bytes)) => {
                            (metadata.final_url, String::from_utf8(bytes).unwrap())
                        },
                    };

                let runtime = unsafe { new_rt_and_cx() };

                let (devtools_mpsc_chan, devtools_mpsc_port) = channel();
                route_ipc_receiver_to_new_servo_sender(devtools_receiver, devtools_mpsc_chan);
                // TODO XXXcreativcoder use this timer_ipc_port, when we have a service worker instance here
                let (timer_ipc_chan, _timer_ipc_port) = ipc::channel().unwrap();
                let (timer_chan, timer_port) = channel();
                let global = ServiceWorkerGlobalScope::new(
                    init,
                    url,
                    devtools_mpsc_port,
                    runtime,
                    own_sender,
                    receiver,
                    timer_ipc_chan,
                    timer_port,
                    swmanager_sender,
                    scope_url,
                );
                let scope = global.upcast::<WorkerGlobalScope>();

                unsafe {
                    // Handle interrupt requests
                    JS_AddInterruptCallback(scope.get_cx(), Some(interrupt_callback));
                }

                scope.execute_script(DOMString::from(source));
                // Service workers are time limited
                thread::Builder::new()
                    .name("SWTimeoutThread".to_owned())
                    .spawn(move || {
                        let sw_lifetime_timeout = PREFS
                            .get("dom.serviceworker.timeout_seconds")
                            .as_u64()
                            .unwrap();
                        thread::sleep(Duration::new(sw_lifetime_timeout, 0));
                        let _ = timer_chan.send(());
                    })
                    .expect("Thread spawning failed");

                global.dispatch_activate();
                let reporter_name = format!("service-worker-reporter-{}", random::<u64>());
                scope
                    .upcast::<GlobalScope>()
                    .mem_profiler_chan()
                    .run_with_memory_reporting(
                        || {
                            // Step 29, Run the responsible event loop specified
                            // by inside settings until it is destroyed.
                            // The worker processing model remains on this step
                            // until the event loop is destroyed,
                            // which happens after the closing flag is set to true.
                            while !scope.is_closing() {
                                run_worker_event_loop(&*global, None);
                            }
                        },
                        reporter_name,
                        scope.script_chan(),
                        CommonScriptMsg::CollectReports,
                    );
            })
            .expect("Thread spawning failed");
    }

    fn handle_mixed_message(&self, msg: MixedMessage) -> bool {
        match msg {
            MixedMessage::FromDevtools(msg) => {
                match msg {
                    DevtoolScriptControlMsg::EvaluateJS(_pipe_id, string, sender) => {
                        devtools::handle_evaluate_js(self.upcast(), string, sender)
                    },
                    DevtoolScriptControlMsg::GetCachedMessages(pipe_id, message_types, sender) => {
                        devtools::handle_get_cached_messages(pipe_id, message_types, sender)
                    },
                    DevtoolScriptControlMsg::WantsLiveNotifications(_pipe_id, bool_val) => {
                        devtools::handle_wants_live_notifications(self.upcast(), bool_val)
                    },
                    _ => debug!("got an unusable devtools control message inside the worker!"),
                }
                true
            },
            MixedMessage::FromServiceWorker(msg) => {
                self.handle_script_event(msg);
                true
            },
            MixedMessage::FromTimeoutThread(_) => {
                let _ = self
                    .swmanager_sender
                    .send(ServiceWorkerMsg::Timeout(self.scope_url.clone()));
                false
            },
        }
    }

    fn handle_script_event(&self, msg: ServiceWorkerScriptMsg) {
        use self::ServiceWorkerScriptMsg::*;

        match msg {
            CommonWorker(WorkerScriptMsg::DOMMessage(data)) => {
                let scope = self.upcast::<WorkerGlobalScope>();
                let target = self.upcast();
                let _ac =
                    JSAutoCompartment::new(scope.get_cx(), scope.reflector().get_jsobject().get());
                rooted!(in(scope.get_cx()) let mut message = UndefinedValue());
                data.read(scope.upcast(), message.handle_mut());
                ExtendableMessageEvent::dispatch_jsval(target, scope.upcast(), message.handle());
            },
            CommonWorker(WorkerScriptMsg::Common(msg)) => {
                self.upcast::<WorkerGlobalScope>().process_event(msg);
            },
            Response(mediator) => {
                // TODO XXXcreativcoder This will eventually use a FetchEvent interface to fire event
                // when we have the Request and Response dom api's implemented
                // https://w3c.github.io/ServiceWorker/#fetchevent-interface
                self.upcast::<EventTarget>().fire_event(atom!("fetch"));
                let _ = mediator.response_chan.send(None);
            },
            WakeUp => {},
        }
    }

    pub fn script_chan(&self) -> Box<ScriptChan + Send> {
        Box::new(ServiceWorkerChan {
            sender: self.own_sender.clone(),
        })
    }

    fn dispatch_activate(&self) {
        let event = ExtendableEvent::new(self, atom!("activate"), false, false);
        let event = (&*event).upcast::<Event>();
        self.upcast::<EventTarget>().dispatch_event(event);
    }
}

#[allow(unsafe_code)]
unsafe extern "C" fn interrupt_callback(cx: *mut JSContext) -> bool {
    let worker = DomRoot::downcast::<WorkerGlobalScope>(GlobalScope::from_context(cx))
        .expect("global is not a worker scope");
    assert!(worker.is::<ServiceWorkerGlobalScope>());

    // A false response causes the script to terminate
    !worker.is_closing()
}

impl ServiceWorkerGlobalScopeMethods for ServiceWorkerGlobalScope {
    // https://w3c.github.io/ServiceWorker/#service-worker-global-scope-onmessage-attribute
    event_handler!(message, GetOnmessage, SetOnmessage);
}

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
use typeholder::TypeHolderTrait;
use task_queue::{QueuedTask, QueuedTaskConversion, TaskQueue};
use task_source::TaskSourceName;

/// Messages used to control service worker event loop
pub enum ServiceWorkerScriptMsg<TH: TypeHolderTrait> {
    /// Message common to all workers
    CommonWorker(WorkerScriptMsg<TH>),
    /// Message to request a custom response by the service worker
    Response(CustomResponseMediator),
    /// Wake-up call from the task queue.
    WakeUp,
}

impl<TH: TypeHolderTrait> QueuedTaskConversion<TH> for ServiceWorkerScriptMsg<TH> {
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

    fn into_queued_task(self) -> Option<QueuedTask<TH>> {
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

    fn from_queued_task(queued_task: QueuedTask<TH>) -> Self {
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

pub enum MixedMessage<TH: TypeHolderTrait> {
    FromServiceWorker(ServiceWorkerScriptMsg<TH>),
    FromDevtools(DevtoolScriptControlMsg),
    FromTimeoutThread(()),
}

#[derive(Clone, JSTraceable)]
pub struct ServiceWorkerChan<TH: TypeHolderTrait> {
    pub sender: Sender<ServiceWorkerScriptMsg<TH>>,
}

impl<TH: TypeHolderTrait> ScriptChan for ServiceWorkerChan<TH> {
    fn send(&self, msg: CommonScriptMsg) -> Result<(), ()> {
        self.sender
            .send(ServiceWorkerScriptMsg::CommonWorker(
                WorkerScriptMsg::Common(msg),
            )).map_err(|_| ())
    }

    fn clone(&self) -> Box<ScriptChan + Send> {
        Box::new(ServiceWorkerChan {
            sender: self.sender.clone(),
        })
    }
}

unsafe_no_jsmanaged_fields_generic!(TaskQueue<ServiceWorkerScriptMsg<TH>, TH>);

#[dom_struct]
pub struct ServiceWorkerGlobalScope<TH: TypeHolderTrait> {
    workerglobalscope: WorkerGlobalScope<TH>,
    #[ignore_malloc_size_of = "Defined in std"]
    task_queue: TaskQueue<ServiceWorkerScriptMsg<TH>, TH>,
    #[ignore_malloc_size_of = "Defined in std"]
    own_sender: Sender<ServiceWorkerScriptMsg<TH>>,
    #[ignore_malloc_size_of = "Defined in std"]
    timer_event_port: Receiver<()>,
    #[ignore_malloc_size_of = "Defined in std"]
    swmanager_sender: IpcSender<ServiceWorkerMsg>,
    scope_url: ServoUrl,
}

impl<TH: TypeHolderTrait> WorkerEventLoopMethods<TH> for ServiceWorkerGlobalScope<TH> {
    type TimerMsg = ();
    type WorkerMsg = ServiceWorkerScriptMsg<TH>;
    type Event = MixedMessage<TH>;

    fn timer_event_port(&self) -> &Receiver<()> {
        &self.timer_event_port
    }

    fn task_queue(&self) -> &TaskQueue<ServiceWorkerScriptMsg<TH>, TH> {
        &self.task_queue
    }

    fn handle_event(&self, event: MixedMessage<TH>) {
        self.handle_mixed_message(event);
    }

    fn handle_worker_post_event(&self, _worker: &TrustedWorkerAddress<TH>) -> Option<AutoWorkerReset<TH>> {
        None
    }

    fn from_worker_msg(&self, msg: ServiceWorkerScriptMsg<TH>) -> MixedMessage<TH> {
        MixedMessage::FromServiceWorker(msg)
    }

    fn from_timer_msg(&self, msg: ()) -> MixedMessage<TH> {
        MixedMessage::FromTimeoutThread(msg)
    }

    fn from_devtools_msg(&self, msg: DevtoolScriptControlMsg) -> MixedMessage<TH> {
        MixedMessage::FromDevtools(msg)
    }
}

impl<TH: TypeHolderTrait> ServiceWorkerGlobalScope<TH> {
    fn new_inherited(
        init: WorkerGlobalScopeInit,
        worker_url: ServoUrl,
        from_devtools_receiver: Receiver<DevtoolScriptControlMsg>,
        runtime: Runtime,
        own_sender: Sender<ServiceWorkerScriptMsg<TH>>,
        receiver: Receiver<ServiceWorkerScriptMsg<TH>>,
        timer_event_chan: IpcSender<TimerEvent>,
        timer_event_port: Receiver<()>,
        swmanager_sender: IpcSender<ServiceWorkerMsg>,
        scope_url: ServoUrl,
    ) -> ServiceWorkerGlobalScope<TH> {
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
        own_sender: Sender<ServiceWorkerScriptMsg<TH>>,
        receiver: Receiver<ServiceWorkerScriptMsg<TH>>,
        timer_event_chan: IpcSender<TimerEvent>,
        timer_event_port: Receiver<()>,
        swmanager_sender: IpcSender<ServiceWorkerMsg>,
        scope_url: ServoUrl,
    ) -> DomRoot<ServiceWorkerGlobalScope<TH>> {
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
    pub fn run_serviceworker_scope(
        scope_things: ScopeThings,
        own_sender: Sender<ServiceWorkerScriptMsg<TH>>,
        receiver: Receiver<ServiceWorkerScriptMsg<TH>>,
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
        let origin = GlobalScope::<TH>::current()
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

                let runtime = unsafe { new_rt_and_cx::<TH>() };

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
                let scope = global.upcast::<WorkerGlobalScope<TH>>();

                unsafe {
                    // Handle interrupt requests
                    JS_AddInterruptCallback(scope.get_cx(), Some(interrupt_callback::<TH>));
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
                    .upcast::<GlobalScope<TH>>()
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

    fn handle_mixed_message(&self, msg: MixedMessage<TH>) -> bool {
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

    fn handle_script_event(&self, msg: ServiceWorkerScriptMsg<TH>) {
        use self::ServiceWorkerScriptMsg::*;

        match msg {
            CommonWorker(WorkerScriptMsg::DOMMessage(data)) => {
                let scope = self.upcast::<WorkerGlobalScope<TH>>();
                let target = self.upcast();
                let _ac =
                    JSAutoCompartment::new(scope.get_cx(), scope.reflector().get_jsobject().get());
                rooted!(in(scope.get_cx()) let mut message = UndefinedValue());
                data.read(scope.upcast(), message.handle_mut());
                ExtendableMessageEvent::dispatch_jsval(target, scope.upcast(), message.handle());
            },
            CommonWorker(WorkerScriptMsg::Common(msg)) => {
                self.upcast::<WorkerGlobalScope<TH>>().process_event(msg);
            },
            Response(mediator) => {
                // TODO XXXcreativcoder This will eventually use a FetchEvent interface to fire event
                // when we have the Request and Response dom api's implemented
                // https://slightlyoff.github.io/ServiceWorker/spec/service_worker_1/index.html#fetch-event-section
                self.upcast::<EventTarget<TH>>().fire_event(atom!("fetch"));
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
        let event = (&*event).upcast::<Event<TH>>();
        self.upcast::<EventTarget<TH>>().dispatch_event(event);
    }
}

#[allow(unsafe_code)]
unsafe extern "C" fn interrupt_callback<TH: TypeHolderTrait>(cx: *mut JSContext) -> bool {
    let worker = DomRoot::downcast::<WorkerGlobalScope<TH>>(GlobalScope::<TH>::from_context(cx))
        .expect("global is not a worker scope");
    assert!(worker.is::<ServiceWorkerGlobalScope<TH>>());

    // A false response causes the script to terminate
    !worker.is_closing()
}

impl<TH: TypeHolderTrait> ServiceWorkerGlobalScopeMethods<TH> for ServiceWorkerGlobalScope<TH> {
    // https://w3c.github.io/ServiceWorker/#service-worker-global-scope-onmessage-attribute
    event_handler!(message, GetOnmessage, SetOnmessage);
}

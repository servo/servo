/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::devtools;
use crate::dom::abstractworker::{SimpleWorkerErrorHandler, WorkerScriptMsg};
use crate::dom::abstractworkerglobalscope::{run_worker_event_loop, WorkerEventLoopMethods};
use crate::dom::abstractworkerglobalscope::{SendableWorkerScriptChan, WorkerThreadWorkerChan};
use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::DedicatedWorkerGlobalScopeBinding;
use crate::dom::bindings::codegen::Bindings::DedicatedWorkerGlobalScopeBinding::DedicatedWorkerGlobalScopeMethods;
use crate::dom::bindings::error::{ErrorInfo, ErrorResult};
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::reflector::DomObject;
use crate::dom::bindings::root::{DomRoot, RootCollection, ThreadLocalStackRoots};
use crate::dom::bindings::str::DOMString;
use crate::dom::bindings::structuredclone::StructuredCloneData;
use crate::dom::errorevent::ErrorEvent;
use crate::dom::event::{Event, EventBubbles, EventCancelable, EventStatus};
use crate::dom::eventtarget::EventTarget;
use crate::dom::globalscope::GlobalScope;
use crate::dom::messageevent::MessageEvent;
use crate::dom::worker::{TrustedWorkerAddress, Worker};
use crate::dom::workerglobalscope::WorkerGlobalScope;
use crate::script_runtime::ScriptThreadEventCategory::WorkerEvent;
use crate::script_runtime::{new_child_runtime, CommonScriptMsg, Runtime, ScriptChan, ScriptPort};
use crate::task_queue::{QueuedTask, QueuedTaskConversion, TaskQueue};
use crate::task_source::TaskSourceName;
use crossbeam_channel::{unbounded, Receiver, Sender};
use devtools_traits::DevtoolScriptControlMsg;
use dom_struct::dom_struct;
use ipc_channel::ipc::{self, IpcReceiver, IpcSender};
use ipc_channel::router::ROUTER;
use js::jsapi::JS_AddInterruptCallback;
use js::jsapi::{JSAutoCompartment, JSContext};
use js::jsval::UndefinedValue;
use js::rust::HandleValue;
use msg::constellation_msg::{PipelineId, TopLevelBrowsingContextId};
use net_traits::request::{CredentialsMode, Destination, Referrer, RequestBuilder};
use net_traits::{load_whole_resource, IpcSend};
use script_traits::{TimerEvent, TimerSource, WorkerGlobalScopeInit, WorkerScriptLoadOrigin};
use servo_rand::random;
use servo_url::ServoUrl;
use std::mem::replace;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use std::thread;
use style::thread_state::{self, ThreadState};

/// Set the `worker` field of a related DedicatedWorkerGlobalScope object to a particular
/// value for the duration of this object's lifetime. This ensures that the related Worker
/// object only lives as long as necessary (ie. while events are being executed), while
/// providing a reference that can be cloned freely.
pub struct AutoWorkerReset<'a> {
    workerscope: &'a DedicatedWorkerGlobalScope,
    old_worker: Option<TrustedWorkerAddress>,
}

impl<'a> AutoWorkerReset<'a> {
    fn new(
        workerscope: &'a DedicatedWorkerGlobalScope,
        worker: TrustedWorkerAddress,
    ) -> AutoWorkerReset<'a> {
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

pub enum DedicatedWorkerScriptMsg {
    /// Standard message from a worker.
    CommonWorker(TrustedWorkerAddress, WorkerScriptMsg),
    /// Wake-up call from the task queue.
    WakeUp,
}

pub enum MixedMessage {
    FromWorker(DedicatedWorkerScriptMsg),
    FromScheduler((TrustedWorkerAddress, TimerEvent)),
    FromDevtools(DevtoolScriptControlMsg),
}

impl QueuedTaskConversion for DedicatedWorkerScriptMsg {
    fn task_source_name(&self) -> Option<&TaskSourceName> {
        let common_worker_msg = match self {
            DedicatedWorkerScriptMsg::CommonWorker(_, common_worker_msg) => common_worker_msg,
            _ => return None,
        };
        let script_msg = match common_worker_msg {
            WorkerScriptMsg::Common(ref script_msg) => script_msg,
            _ => return None,
        };
        match script_msg {
            CommonScriptMsg::Task(_category, _boxed, _pipeline_id, source_name) => {
                Some(&source_name)
            },
            _ => None,
        }
    }

    fn pipeline_id(&self) -> Option<PipelineId> {
        // Workers always return None, since the pipeline_id is only used to check for document activity,
        // and this check does not apply to worker event-loops.
        None
    }

    fn into_queued_task(self) -> Option<QueuedTask> {
        let (worker, common_worker_msg) = match self {
            DedicatedWorkerScriptMsg::CommonWorker(worker, common_worker_msg) => {
                (worker, common_worker_msg)
            },
            _ => return None,
        };
        let script_msg = match common_worker_msg {
            WorkerScriptMsg::Common(script_msg) => script_msg,
            _ => return None,
        };
        let (category, boxed, pipeline_id, task_source) = match script_msg {
            CommonScriptMsg::Task(category, boxed, pipeline_id, task_source) => {
                (category, boxed, pipeline_id, task_source)
            },
            _ => return None,
        };
        Some((Some(worker), category, boxed, pipeline_id, task_source))
    }

    fn from_queued_task(queued_task: QueuedTask) -> Self {
        let (worker, category, boxed, pipeline_id, task_source) = queued_task;
        let script_msg = CommonScriptMsg::Task(category, boxed, pipeline_id, task_source);
        DedicatedWorkerScriptMsg::CommonWorker(worker.unwrap(), WorkerScriptMsg::Common(script_msg))
    }

    fn inactive_msg() -> Self {
        // Inactive is only relevant in the context of a browsing-context event-loop.
        panic!("Workers should never receive messages marked as inactive");
    }

    fn wake_up_msg() -> Self {
        DedicatedWorkerScriptMsg::WakeUp
    }

    fn is_wake_up(&self) -> bool {
        match self {
            DedicatedWorkerScriptMsg::WakeUp => true,
            _ => false,
        }
    }
}

unsafe_no_jsmanaged_fields!(TaskQueue<DedicatedWorkerScriptMsg>);

// https://html.spec.whatwg.org/multipage/#dedicatedworkerglobalscope
#[dom_struct]
pub struct DedicatedWorkerGlobalScope {
    workerglobalscope: WorkerGlobalScope,
    #[ignore_malloc_size_of = "Defined in std"]
    task_queue: TaskQueue<DedicatedWorkerScriptMsg>,
    #[ignore_malloc_size_of = "Defined in std"]
    own_sender: Sender<DedicatedWorkerScriptMsg>,
    #[ignore_malloc_size_of = "Defined in std"]
    timer_event_port: Receiver<(TrustedWorkerAddress, TimerEvent)>,
    #[ignore_malloc_size_of = "Trusted<T> has unclear ownership like Dom<T>"]
    worker: DomRefCell<Option<TrustedWorkerAddress>>,
    #[ignore_malloc_size_of = "Can't measure trait objects"]
    /// Sender to the parent thread.
    parent_sender: Box<ScriptChan + Send>,
}

impl WorkerEventLoopMethods for DedicatedWorkerGlobalScope {
    type TimerMsg = (TrustedWorkerAddress, TimerEvent);
    type WorkerMsg = DedicatedWorkerScriptMsg;
    type Event = MixedMessage;

    fn timer_event_port(&self) -> &Receiver<(TrustedWorkerAddress, TimerEvent)> {
        &self.timer_event_port
    }

    fn task_queue(&self) -> &TaskQueue<DedicatedWorkerScriptMsg> {
        &self.task_queue
    }

    fn handle_event(&self, event: MixedMessage) {
        self.handle_mixed_message(event);
    }

    fn handle_worker_post_event(&self, worker: &TrustedWorkerAddress) -> Option<AutoWorkerReset> {
        let ar = AutoWorkerReset::new(&self, worker.clone());
        Some(ar)
    }

    fn from_worker_msg(&self, msg: DedicatedWorkerScriptMsg) -> MixedMessage {
        MixedMessage::FromWorker(msg)
    }

    fn from_timer_msg(&self, msg: (TrustedWorkerAddress, TimerEvent)) -> MixedMessage {
        MixedMessage::FromScheduler(msg)
    }

    fn from_devtools_msg(&self, msg: DevtoolScriptControlMsg) -> MixedMessage {
        MixedMessage::FromDevtools(msg)
    }
}

impl DedicatedWorkerGlobalScope {
    fn new_inherited(
        init: WorkerGlobalScopeInit,
        worker_url: ServoUrl,
        from_devtools_receiver: Receiver<DevtoolScriptControlMsg>,
        runtime: Runtime,
        parent_sender: Box<dyn ScriptChan + Send>,
        own_sender: Sender<DedicatedWorkerScriptMsg>,
        receiver: Receiver<DedicatedWorkerScriptMsg>,
        timer_event_chan: IpcSender<TimerEvent>,
        timer_event_port: Receiver<(TrustedWorkerAddress, TimerEvent)>,
        closing: Arc<AtomicBool>,
    ) -> DedicatedWorkerGlobalScope {
        DedicatedWorkerGlobalScope {
            workerglobalscope: WorkerGlobalScope::new_inherited(
                init,
                worker_url,
                runtime,
                from_devtools_receiver,
                timer_event_chan,
                Some(closing),
            ),
            task_queue: TaskQueue::new(receiver, own_sender.clone()),
            own_sender: own_sender,
            timer_event_port: timer_event_port,
            parent_sender: parent_sender,
            worker: DomRefCell::new(None),
        }
    }

    #[allow(unsafe_code)]
    pub fn new(
        init: WorkerGlobalScopeInit,
        worker_url: ServoUrl,
        from_devtools_receiver: Receiver<DevtoolScriptControlMsg>,
        runtime: Runtime,
        parent_sender: Box<dyn ScriptChan + Send>,
        own_sender: Sender<DedicatedWorkerScriptMsg>,
        receiver: Receiver<DedicatedWorkerScriptMsg>,
        timer_event_chan: IpcSender<TimerEvent>,
        timer_event_port: Receiver<(TrustedWorkerAddress, TimerEvent)>,
        closing: Arc<AtomicBool>,
    ) -> DomRoot<DedicatedWorkerGlobalScope> {
        let cx = runtime.cx();
        let scope = Box::new(DedicatedWorkerGlobalScope::new_inherited(
            init,
            worker_url,
            from_devtools_receiver,
            runtime,
            parent_sender,
            own_sender,
            receiver,
            timer_event_chan,
            timer_event_port,
            closing,
        ));
        unsafe { DedicatedWorkerGlobalScopeBinding::Wrap(cx, scope) }
    }

    #[allow(unsafe_code)]
    // https://html.spec.whatwg.org/multipage/#run-a-worker
    pub fn run_worker_scope(
        init: WorkerGlobalScopeInit,
        worker_url: ServoUrl,
        from_devtools_receiver: IpcReceiver<DevtoolScriptControlMsg>,
        worker: TrustedWorkerAddress,
        parent_sender: Box<dyn ScriptChan + Send>,
        own_sender: Sender<DedicatedWorkerScriptMsg>,
        receiver: Receiver<DedicatedWorkerScriptMsg>,
        worker_load_origin: WorkerScriptLoadOrigin,
        closing: Arc<AtomicBool>,
    ) {
        let serialized_worker_url = worker_url.to_string();
        let name = format!("WebWorker for {}", serialized_worker_url);
        let top_level_browsing_context_id = TopLevelBrowsingContextId::installed();
        let current_global = GlobalScope::current().expect("No current global object");
        let origin = current_global.origin().immutable().clone();
        let parent = current_global.runtime_handle();

        thread::Builder::new()
            .name(name)
            .spawn(move || {
                thread_state::initialize(ThreadState::SCRIPT | ThreadState::IN_WORKER);

                if let Some(top_level_browsing_context_id) = top_level_browsing_context_id {
                    TopLevelBrowsingContextId::install(top_level_browsing_context_id);
                }

                let roots = RootCollection::new();
                let _stack_roots = ThreadLocalStackRoots::new(&roots);

                let WorkerScriptLoadOrigin {
                    referrer_url,
                    referrer_policy,
                    pipeline_id,
                } = worker_load_origin;

                let referrer = referrer_url.map(|referrer_url| Referrer::ReferrerUrl(referrer_url));

                let request = RequestBuilder::new(worker_url.clone())
                    .destination(Destination::Worker)
                    .credentials_mode(CredentialsMode::Include)
                    .use_url_credentials(true)
                    .pipeline_id(pipeline_id)
                    .referrer(referrer)
                    .referrer_policy(referrer_policy)
                    .origin(origin);

                let (metadata, bytes) =
                    match load_whole_resource(request, &init.resource_threads.sender()) {
                        Err(_) => {
                            println!("error loading script {}", serialized_worker_url);
                            parent_sender
                                .send(CommonScriptMsg::Task(
                                    WorkerEvent,
                                    Box::new(SimpleWorkerErrorHandler::new(worker)),
                                    pipeline_id,
                                    TaskSourceName::DOMManipulation,
                                ))
                                .unwrap();
                            return;
                        },
                        Ok((metadata, bytes)) => (metadata, bytes),
                    };
                let url = metadata.final_url;
                let source = String::from_utf8_lossy(&bytes);

                let runtime = unsafe { new_child_runtime(parent) };

                let (devtools_mpsc_chan, devtools_mpsc_port) = unbounded();
                ROUTER.route_ipc_receiver_to_crossbeam_sender(
                    from_devtools_receiver,
                    devtools_mpsc_chan,
                );

                let (timer_tx, timer_rx) = unbounded();
                let (timer_ipc_chan, timer_ipc_port) = ipc::channel().unwrap();
                let worker_for_route = worker.clone();
                ROUTER.add_route(
                    timer_ipc_port.to_opaque(),
                    Box::new(move |message| {
                        let event = message.to().unwrap();
                        timer_tx.send((worker_for_route.clone(), event)).unwrap();
                    }),
                );

                let global = DedicatedWorkerGlobalScope::new(
                    init,
                    url,
                    devtools_mpsc_port,
                    runtime,
                    parent_sender.clone(),
                    own_sender,
                    receiver,
                    timer_ipc_chan,
                    timer_rx,
                    closing,
                );
                // FIXME(njn): workers currently don't have a unique ID suitable for using in reporter
                // registration (#6631), so we instead use a random number and cross our fingers.
                let scope = global.upcast::<WorkerGlobalScope>();

                unsafe {
                    // Handle interrupt requests
                    JS_AddInterruptCallback(scope.get_cx(), Some(interrupt_callback));
                }

                if scope.is_closing() {
                    return;
                }

                {
                    let _ar = AutoWorkerReset::new(&global, worker.clone());
                    scope.execute_script(DOMString::from(source));
                }

                let reporter_name = format!("dedicated-worker-reporter-{}", random::<u64>());
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
                                run_worker_event_loop(&*global, Some(&worker));
                            }
                        },
                        reporter_name,
                        parent_sender,
                        CommonScriptMsg::CollectReports,
                    );
            })
            .expect("Thread spawning failed");
    }

    pub fn script_chan(&self) -> Box<dyn ScriptChan + Send> {
        Box::new(WorkerThreadWorkerChan {
            sender: self.own_sender.clone(),
            worker: self.worker.borrow().as_ref().unwrap().clone(),
        })
    }

    pub fn new_script_pair(&self) -> (Box<dyn ScriptChan + Send>, Box<dyn ScriptPort + Send>) {
        let (tx, rx) = unbounded();
        let chan = Box::new(SendableWorkerScriptChan {
            sender: tx,
            worker: self.worker.borrow().as_ref().unwrap().clone(),
        });
        (chan, Box::new(rx))
    }

    fn handle_script_event(&self, msg: WorkerScriptMsg) {
        match msg {
            WorkerScriptMsg::DOMMessage(data) => {
                let scope = self.upcast::<WorkerGlobalScope>();
                let target = self.upcast();
                let _ac =
                    JSAutoCompartment::new(scope.get_cx(), scope.reflector().get_jsobject().get());
                rooted!(in(scope.get_cx()) let mut message = UndefinedValue());
                data.read(scope.upcast(), message.handle_mut());
                MessageEvent::dispatch_jsval(target, scope.upcast(), message.handle(), None, None);
            },
            WorkerScriptMsg::Common(msg) => {
                self.upcast::<WorkerGlobalScope>().process_event(msg);
            },
        }
    }

    fn handle_mixed_message(&self, msg: MixedMessage) {
        match msg {
            MixedMessage::FromDevtools(msg) => match msg {
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
            },
            MixedMessage::FromScheduler((linked_worker, timer_event)) => match timer_event {
                TimerEvent(TimerSource::FromWorker, id) => {
                    let _ar = AutoWorkerReset::new(self, linked_worker);
                    let scope = self.upcast::<WorkerGlobalScope>();
                    scope.handle_fire_timer(id);
                },
                TimerEvent(_, _) => panic!("A worker received a TimerEvent from a window."),
            },
            MixedMessage::FromWorker(DedicatedWorkerScriptMsg::CommonWorker(
                linked_worker,
                msg,
            )) => {
                let _ar = AutoWorkerReset::new(self, linked_worker);
                self.handle_script_event(msg);
            },
            MixedMessage::FromWorker(DedicatedWorkerScriptMsg::WakeUp) => {},
        }
    }

    // https://html.spec.whatwg.org/multipage/#runtime-script-errors-2
    #[allow(unsafe_code)]
    pub fn forward_error_to_worker_object(&self, error_info: ErrorInfo) {
        let worker = self.worker.borrow().as_ref().unwrap().clone();
        let pipeline_id = self.upcast::<GlobalScope>().pipeline_id();
        let task = Box::new(task!(forward_error_to_worker_object: move || {
            let worker = worker.root();
            let global = worker.global();

            // Step 1.
            let event = ErrorEvent::new(
                &global,
                atom!("error"),
                EventBubbles::DoesNotBubble,
                EventCancelable::Cancelable,
                error_info.message.as_str().into(),
                error_info.filename.as_str().into(),
                error_info.lineno,
                error_info.column,
                HandleValue::null(),
            );
            let event_status =
                event.upcast::<Event>().fire(worker.upcast::<EventTarget>());

            // Step 2.
            if event_status == EventStatus::NotCanceled {
                global.report_an_error(error_info, HandleValue::null());
            }
        }));
        self.parent_sender
            .send(CommonScriptMsg::Task(
                WorkerEvent,
                task,
                Some(pipeline_id),
                TaskSourceName::DOMManipulation,
            ))
            .unwrap();
    }
}

#[allow(unsafe_code)]
unsafe extern "C" fn interrupt_callback(cx: *mut JSContext) -> bool {
    let worker = DomRoot::downcast::<WorkerGlobalScope>(GlobalScope::from_context(cx))
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
        let pipeline_id = self.upcast::<GlobalScope>().pipeline_id();
        let task = Box::new(task!(post_worker_message: move || {
            Worker::handle_message(worker, data);
        }));
        // TODO: Change this task source to a new `unshipped-port-message-queue` task source
        self.parent_sender
            .send(CommonScriptMsg::Task(
                WorkerEvent,
                task,
                Some(pipeline_id),
                TaskSourceName::DOMManipulation,
            ))
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

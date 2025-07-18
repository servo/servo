/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use core::fmt;
#[cfg(feature = "webgpu")]
use std::cell::RefCell;
use std::option::Option;
use std::result::Result;

use base::id::PipelineId;
#[cfg(feature = "bluetooth")]
use bluetooth_traits::BluetoothRequest;
use constellation_traits::ScriptToConstellationMessage;
use crossbeam_channel::{Receiver, SendError, Sender, select};
use devtools_traits::{DevtoolScriptControlMsg, ScriptToDevtoolsControlMsg};
use ipc_channel::ipc::IpcSender;
use net_traits::FetchResponseMsg;
use net_traits::image_cache::ImageCacheResponseMessage;
use profile_traits::mem::{self as profile_mem, OpaqueSender, ReportsChan};
use profile_traits::time::{self as profile_time};
use script_traits::{Painter, ScriptThreadMessage};
use stylo_atoms::Atom;
use timers::TimerScheduler;
#[cfg(feature = "webgpu")]
use webgpu_traits::WebGPUMsg;

use crate::dom::abstractworker::WorkerScriptMsg;
use crate::dom::bindings::trace::CustomTraceable;
use crate::dom::dedicatedworkerglobalscope::DedicatedWorkerScriptMsg;
use crate::dom::serviceworkerglobalscope::ServiceWorkerScriptMsg;
use crate::dom::worker::TrustedWorkerAddress;
use crate::script_runtime::ScriptThreadEventCategory;
use crate::task::TaskBox;
use crate::task_queue::{QueuedTask, QueuedTaskConversion, TaskQueue};
use crate::task_source::TaskSourceName;

#[derive(Debug)]
pub(crate) enum MixedMessage {
    FromConstellation(ScriptThreadMessage),
    FromScript(MainThreadScriptMsg),
    FromDevtools(DevtoolScriptControlMsg),
    FromImageCache(ImageCacheResponseMessage),
    #[cfg(feature = "webgpu")]
    FromWebGPUServer(WebGPUMsg),
    TimerFired,
}

impl MixedMessage {
    pub(crate) fn pipeline_id(&self) -> Option<PipelineId> {
        match self {
            MixedMessage::FromConstellation(inner_msg) => match inner_msg {
                ScriptThreadMessage::StopDelayingLoadEventsMode(id) => Some(*id),
                ScriptThreadMessage::AttachLayout(new_layout_info) => new_layout_info
                    .parent_info
                    .or(Some(new_layout_info.new_pipeline_id)),
                ScriptThreadMessage::Resize(id, ..) => Some(*id),
                ScriptThreadMessage::ThemeChange(id, ..) => Some(*id),
                ScriptThreadMessage::ResizeInactive(id, ..) => Some(*id),
                ScriptThreadMessage::UnloadDocument(id) => Some(*id),
                ScriptThreadMessage::ExitPipeline(_webview_id, id, ..) => Some(*id),
                ScriptThreadMessage::ExitScriptThread => None,
                ScriptThreadMessage::SendInputEvent(id, ..) => Some(*id),
                ScriptThreadMessage::Viewport(id, ..) => Some(*id),
                ScriptThreadMessage::GetTitle(id) => Some(*id),
                ScriptThreadMessage::SetDocumentActivity(id, ..) => Some(*id),
                ScriptThreadMessage::SetThrottled(id, ..) => Some(*id),
                ScriptThreadMessage::SetThrottledInContainingIframe(id, ..) => Some(*id),
                ScriptThreadMessage::NavigateIframe(id, ..) => Some(*id),
                ScriptThreadMessage::PostMessage { target: id, .. } => Some(*id),
                ScriptThreadMessage::UpdatePipelineId(_, _, _, id, _) => Some(*id),
                ScriptThreadMessage::UpdateHistoryState(id, ..) => Some(*id),
                ScriptThreadMessage::RemoveHistoryStates(id, ..) => Some(*id),
                ScriptThreadMessage::FocusIFrame(id, ..) => Some(*id),
                ScriptThreadMessage::FocusDocument(id, ..) => Some(*id),
                ScriptThreadMessage::Unfocus(id, ..) => Some(*id),
                ScriptThreadMessage::WebDriverScriptCommand(id, ..) => Some(*id),
                ScriptThreadMessage::TickAllAnimations(..) => None,
                ScriptThreadMessage::WebFontLoaded(id, ..) => Some(*id),
                ScriptThreadMessage::DispatchIFrameLoadEvent {
                    target: _,
                    parent: id,
                    child: _,
                } => Some(*id),
                ScriptThreadMessage::DispatchStorageEvent(id, ..) => Some(*id),
                ScriptThreadMessage::ReportCSSError(id, ..) => Some(*id),
                ScriptThreadMessage::Reload(id, ..) => Some(*id),
                ScriptThreadMessage::PaintMetric(id, ..) => Some(*id),
                ScriptThreadMessage::ExitFullScreen(id, ..) => Some(*id),
                ScriptThreadMessage::MediaSessionAction(..) => None,
                #[cfg(feature = "webgpu")]
                ScriptThreadMessage::SetWebGPUPort(..) => None,
                ScriptThreadMessage::SetScrollStates(id, ..) => Some(*id),
                ScriptThreadMessage::EvaluateJavaScript(id, _, _) => Some(*id),
                ScriptThreadMessage::SendImageKeysBatch(..) => None,
            },
            MixedMessage::FromScript(inner_msg) => match inner_msg {
                MainThreadScriptMsg::Common(CommonScriptMsg::Task(_, _, pipeline_id, _)) => {
                    *pipeline_id
                },
                MainThreadScriptMsg::Common(CommonScriptMsg::CollectReports(_)) => None,
                MainThreadScriptMsg::NavigationResponse { pipeline_id, .. } => Some(*pipeline_id),
                MainThreadScriptMsg::WorkletLoaded(pipeline_id) => Some(*pipeline_id),
                MainThreadScriptMsg::RegisterPaintWorklet { pipeline_id, .. } => Some(*pipeline_id),
                MainThreadScriptMsg::Inactive => None,
                MainThreadScriptMsg::WakeUp => None,
            },
            MixedMessage::FromImageCache(response) => match response {
                ImageCacheResponseMessage::NotifyPendingImageLoadStatus(response) => {
                    Some(response.pipeline_id)
                },
                ImageCacheResponseMessage::VectorImageRasterizationComplete(response) => {
                    Some(response.pipeline_id)
                },
            },
            MixedMessage::FromDevtools(_) | MixedMessage::TimerFired => None,
            #[cfg(feature = "webgpu")]
            MixedMessage::FromWebGPUServer(..) => None,
        }
    }
}

/// Messages used to control the script event loop.
#[derive(Debug)]
pub(crate) enum MainThreadScriptMsg {
    /// Common variants associated with the script messages
    Common(CommonScriptMsg),
    /// Notifies the script thread that a new worklet has been loaded, and thus the page should be
    /// reflowed.
    WorkletLoaded(PipelineId),
    NavigationResponse {
        pipeline_id: PipelineId,
        message: Box<FetchResponseMsg>,
    },
    /// Notifies the script thread that a new paint worklet has been registered.
    RegisterPaintWorklet {
        pipeline_id: PipelineId,
        name: Atom,
        properties: Vec<Atom>,
        painter: Box<dyn Painter>,
    },
    /// A task related to a not fully-active document has been throttled.
    Inactive,
    /// Wake-up call from the task queue.
    WakeUp,
}

/// Common messages used to control the event loops in both the script and the worker
pub(crate) enum CommonScriptMsg {
    /// Requests that the script thread measure its memory usage. The results are sent back via the
    /// supplied channel.
    CollectReports(ReportsChan),
    /// Generic message that encapsulates event handling.
    Task(
        ScriptThreadEventCategory,
        Box<dyn TaskBox>,
        Option<PipelineId>,
        TaskSourceName,
    ),
}

impl fmt::Debug for CommonScriptMsg {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            CommonScriptMsg::CollectReports(_) => write!(f, "CollectReports(...)"),
            CommonScriptMsg::Task(ref category, ref task, _, _) => {
                f.debug_tuple("Task").field(category).field(task).finish()
            },
        }
    }
}

/// A wrapper around various types of `Sender`s that send messages back to the event loop
/// of a script context event loop. This will either target the main `ScriptThread` event
/// loop or that of a worker.
#[derive(Clone, JSTraceable, MallocSizeOf)]
pub(crate) enum ScriptEventLoopSender {
    /// A sender that sends to the main `ScriptThread` event loop.
    MainThread(Sender<MainThreadScriptMsg>),
    /// A sender that sends to a `ServiceWorker` event loop.
    ServiceWorker(Sender<ServiceWorkerScriptMsg>),
    /// A sender that sends to a dedicated worker (such as a generic Web Worker) event loop.
    /// Note that this sender keeps the main thread Worker DOM object alive as long as it or
    /// or any message it sends is not dropped.
    DedicatedWorker {
        sender: Sender<DedicatedWorkerScriptMsg>,
        main_thread_worker: TrustedWorkerAddress,
    },
}

impl ScriptEventLoopSender {
    /// Send a message to the event loop, which might be a main thread event loop or a worker event loop.
    pub(crate) fn send(&self, message: CommonScriptMsg) -> Result<(), SendError<()>> {
        match self {
            Self::MainThread(sender) => sender
                .send(MainThreadScriptMsg::Common(message))
                .map_err(|_| SendError(())),
            Self::ServiceWorker(sender) => sender
                .send(ServiceWorkerScriptMsg::CommonWorker(
                    WorkerScriptMsg::Common(message),
                ))
                .map_err(|_| SendError(())),
            Self::DedicatedWorker {
                sender,
                main_thread_worker,
            } => {
                let common_message = WorkerScriptMsg::Common(message);
                sender
                    .send(DedicatedWorkerScriptMsg::CommonWorker(
                        main_thread_worker.clone(),
                        common_message,
                    ))
                    .map_err(|_| SendError(()))
            },
        }
    }
}

/// A wrapper around various types of `Receiver`s that receive event loop messages. Used for
/// synchronous DOM APIs that need to abstract over multiple kinds of event loops (worker/main
/// thread) with different Receiver interfaces.
pub(crate) enum ScriptEventLoopReceiver {
    /// A receiver that receives messages to the main `ScriptThread` event loop.
    MainThread(Receiver<MainThreadScriptMsg>),
    /// A receiver that receives messages to dedicated workers (such as a generic Web Worker) event loop.
    DedicatedWorker(Receiver<DedicatedWorkerScriptMsg>),
}

impl ScriptEventLoopReceiver {
    pub(crate) fn recv(&self) -> Result<CommonScriptMsg, ()> {
        match self {
            Self::MainThread(receiver) => match receiver.recv() {
                Ok(MainThreadScriptMsg::Common(script_msg)) => Ok(script_msg),
                Ok(_) => panic!("unexpected main thread event message!"),
                Err(_) => Err(()),
            },
            Self::DedicatedWorker(receiver) => match receiver.recv() {
                Ok(DedicatedWorkerScriptMsg::CommonWorker(_, WorkerScriptMsg::Common(message))) => {
                    Ok(message)
                },
                Ok(_) => panic!("unexpected worker event message!"),
                Err(_) => Err(()),
            },
        }
    }
}

impl QueuedTaskConversion for MainThreadScriptMsg {
    fn task_source_name(&self) -> Option<&TaskSourceName> {
        let script_msg = match self {
            MainThreadScriptMsg::Common(script_msg) => script_msg,
            _ => return None,
        };
        match script_msg {
            CommonScriptMsg::Task(_category, _boxed, _pipeline_id, task_source) => {
                Some(task_source)
            },
            _ => None,
        }
    }

    fn pipeline_id(&self) -> Option<PipelineId> {
        let script_msg = match self {
            MainThreadScriptMsg::Common(script_msg) => script_msg,
            _ => return None,
        };
        match script_msg {
            CommonScriptMsg::Task(_category, _boxed, pipeline_id, _task_source) => *pipeline_id,
            _ => None,
        }
    }

    fn into_queued_task(self) -> Option<QueuedTask> {
        let script_msg = match self {
            MainThreadScriptMsg::Common(script_msg) => script_msg,
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
        MainThreadScriptMsg::Common(script_msg)
    }

    fn inactive_msg() -> Self {
        MainThreadScriptMsg::Inactive
    }

    fn wake_up_msg() -> Self {
        MainThreadScriptMsg::WakeUp
    }

    fn is_wake_up(&self) -> bool {
        matches!(self, MainThreadScriptMsg::WakeUp)
    }
}

impl OpaqueSender<CommonScriptMsg> for ScriptEventLoopSender {
    fn send(&self, message: CommonScriptMsg) {
        self.send(message).unwrap()
    }
}

#[derive(Clone, JSTraceable)]
pub(crate) struct ScriptThreadSenders {
    /// A channel to hand out to script thread-based entities that need to be able to enqueue
    /// events in the event queue.
    pub(crate) self_sender: Sender<MainThreadScriptMsg>,

    /// A handle to the bluetooth thread.
    #[no_trace]
    #[cfg(feature = "bluetooth")]
    pub(crate) bluetooth_sender: IpcSender<BluetoothRequest>,

    /// A [`Sender`] that sends messages to the `Constellation`.
    #[no_trace]
    pub(crate) constellation_sender: IpcSender<ScriptThreadMessage>,

    /// A [`Sender`] that sends messages to the `Constellation` associated with
    /// particular pipelines.
    #[no_trace]
    pub(crate) pipeline_to_constellation_sender:
        IpcSender<(PipelineId, ScriptToConstellationMessage)>,

    /// The shared [`IpcSender`] which is sent to the `ImageCache` when requesting an image. The
    /// messages on this channel are routed to crossbeam [`Sender`] on the router thread, which
    /// in turn sends messages to [`ScriptThreadReceivers::image_cache_receiver`].
    #[no_trace]
    pub(crate) image_cache_sender: IpcSender<ImageCacheResponseMessage>,

    /// For providing contact with the time profiler.
    #[no_trace]
    pub(crate) time_profiler_sender: profile_time::ProfilerChan,

    /// For providing contact with the memory profiler.
    #[no_trace]
    pub(crate) memory_profiler_sender: profile_mem::ProfilerChan,

    /// For providing instructions to an optional devtools server.
    #[no_trace]
    pub(crate) devtools_server_sender: Option<IpcSender<ScriptToDevtoolsControlMsg>>,

    #[no_trace]
    pub(crate) devtools_client_to_script_thread_sender: IpcSender<DevtoolScriptControlMsg>,

    #[no_trace]
    pub(crate) content_process_shutdown_sender: Sender<()>,
}

#[derive(JSTraceable)]
pub(crate) struct ScriptThreadReceivers {
    /// A [`Receiver`] that receives messages from the constellation.
    #[no_trace]
    pub(crate) constellation_receiver: Receiver<ScriptThreadMessage>,

    /// The [`Receiver`] which receives incoming messages from the `ImageCache`.
    #[no_trace]
    pub(crate) image_cache_receiver: Receiver<ImageCacheResponseMessage>,

    /// For receiving commands from an optional devtools server. Will be ignored if no such server
    /// exists. When devtools are not active this will be [`crossbeam_channel::never()`].
    #[no_trace]
    pub(crate) devtools_server_receiver: Receiver<DevtoolScriptControlMsg>,

    /// Receiver to receive commands from optional WebGPU server. When there is no active
    /// WebGPU context, this will be [`crossbeam_channel::never()`].
    #[no_trace]
    #[cfg(feature = "webgpu")]
    pub(crate) webgpu_receiver: RefCell<Receiver<WebGPUMsg>>,
}

impl ScriptThreadReceivers {
    /// Block until a message is received by any of the receivers of this [`ScriptThreadReceivers`]
    /// or the given [`TaskQueue`] or [`TimerScheduler`]. Return the first message received.
    pub(crate) fn recv(
        &self,
        task_queue: &TaskQueue<MainThreadScriptMsg>,
        timer_scheduler: &TimerScheduler,
    ) -> MixedMessage {
        select! {
            recv(task_queue.select()) -> msg => {
                task_queue.take_tasks(msg.unwrap());
                let event = task_queue
                    .recv()
                    .expect("Spurious wake-up of the event-loop, task-queue has no tasks available");
                MixedMessage::FromScript(event)
            },
            recv(self.constellation_receiver) -> msg => MixedMessage::FromConstellation(msg.unwrap()),
            recv(self.devtools_server_receiver) -> msg => MixedMessage::FromDevtools(msg.unwrap()),
            recv(self.image_cache_receiver) -> msg => MixedMessage::FromImageCache(msg.unwrap()),
            recv(timer_scheduler.wait_channel()) -> _ => MixedMessage::TimerFired,
            recv({
                #[cfg(feature = "webgpu")]
                {
                    self.webgpu_receiver.borrow()
                }
                #[cfg(not(feature = "webgpu"))]
                {
                    crossbeam_channel::never::<()>()
                }
            }) -> msg => {
                #[cfg(feature = "webgpu")]
                {
                    MixedMessage::FromWebGPUServer(msg.unwrap())
                }
                #[cfg(not(feature = "webgpu"))]
                {
                    unreachable!("This should never be hit when webgpu is disabled ({msg:?})");
                }
            }
        }
    }

    /// Try to receive a from any of the receivers of this [`ScriptThreadReceivers`] or the given
    /// [`TaskQueue`]. Return `None` if no messages are ready to be received.
    pub(crate) fn try_recv(
        &self,
        task_queue: &TaskQueue<MainThreadScriptMsg>,
    ) -> Option<MixedMessage> {
        if let Ok(message) = self.constellation_receiver.try_recv() {
            return MixedMessage::FromConstellation(message).into();
        }
        if let Ok(message) = task_queue.take_tasks_and_recv() {
            return MixedMessage::FromScript(message).into();
        }
        if let Ok(message) = self.devtools_server_receiver.try_recv() {
            return MixedMessage::FromDevtools(message).into();
        }
        if let Ok(message) = self.image_cache_receiver.try_recv() {
            return MixedMessage::FromImageCache(message).into();
        }
        #[cfg(feature = "webgpu")]
        if let Ok(message) = self.webgpu_receiver.borrow().try_recv() {
            return MixedMessage::FromWebGPUServer(message).into();
        }
        None
    }
}

/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::AnimationState;
use crate::AuxiliaryBrowsingContextLoadInfo;
use crate::BroadcastMsg;
use crate::DocumentState;
use crate::FocusSequenceNumber;
use crate::IFrameLoadInfoWithData;
use crate::LayoutControlMsg;
use crate::LoadData;
use crate::MessagePortMsg;
use crate::PortMessageTask;
use crate::StructuredSerializedData;
use crate::WindowSizeType;
use crate::WorkerGlobalScopeInit;
use crate::WorkerScriptLoadOrigin;
use canvas_traits::canvas::{CanvasId, CanvasMsg};
use devtools_traits::{ScriptToDevtoolsControlMsg, WorkerId};
use embedder_traits::{EmbedderMsg, MediaSessionEvent};
use euclid::default::Size2D as UntypedSize2D;
use euclid::Size2D;
use gfx_traits::Epoch;
use ipc_channel::ipc::{IpcReceiver, IpcSender};
use msg::constellation_msg::{
    BroadcastChannelRouterId, BrowsingContextId, MessagePortId, MessagePortRouterId, PipelineId,
    TopLevelBrowsingContextId,
};
use msg::constellation_msg::{HistoryStateId, TraversalDirection};
use msg::constellation_msg::{ServiceWorkerId, ServiceWorkerRegistrationId};
use net_traits::request::RequestBuilder;
use net_traits::storage_thread::StorageType;
use net_traits::CoreResourceMsg;
use servo_url::ImmutableOrigin;
use servo_url::ServoUrl;
use smallvec::SmallVec;
use std::collections::{HashMap, VecDeque};
use std::fmt;
use style_traits::viewport::ViewportConstraints;
use style_traits::CSSPixel;
use webgpu::{wgpu, WebGPU, WebGPUResponseResult};
use webrender_api::units::{DeviceIntPoint, DeviceIntSize};

/// A particular iframe's size, associated with a browsing context.
#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub struct IFrameSize {
    /// The child browsing context for this iframe.
    pub id: BrowsingContextId,
    /// The size of the iframe.
    pub size: Size2D<f32, CSSPixel>,
}

/// An iframe sizing operation.
#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub struct IFrameSizeMsg {
    /// The iframe sizing data.
    pub data: IFrameSize,
    /// The kind of sizing operation.
    pub type_: WindowSizeType,
}

/// Messages from the layout to the constellation.
#[derive(Deserialize, Serialize)]
pub enum LayoutMsg {
    /// Inform the constellation of the size of the iframe's viewport.
    IFrameSizes(Vec<IFrameSizeMsg>),
    /// Requests that the constellation inform the compositor that it needs to record
    /// the time when the frame with the given ID (epoch) is painted.
    PendingPaintMetric(PipelineId, Epoch),
    /// Notifies the constellation that the viewport has been constrained in some manner
    ViewportConstrained(PipelineId, ViewportConstraints),
}

impl fmt::Debug for LayoutMsg {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        use self::LayoutMsg::*;
        let variant = match *self {
            IFrameSizes(..) => "IFrameSizes",
            PendingPaintMetric(..) => "PendingPaintMetric",
            ViewportConstrained(..) => "ViewportConstrained",
        };
        write!(formatter, "LayoutMsg::{}", variant)
    }
}

/// Whether a DOM event was prevented by web content
#[derive(Debug, Deserialize, Serialize)]
pub enum EventResult {
    /// Allowed by web content
    DefaultAllowed,
    /// Prevented by web content
    DefaultPrevented,
}

/// A log entry reported to the constellation
/// We don't report all log entries, just serious ones.
/// We need a separate type for this because `LogLevel` isn't serializable.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum LogEntry {
    /// Panic, with a reason and backtrace
    Panic(String, String),
    /// Error, with a reason
    Error(String),
    /// warning, with a reason
    Warn(String),
}

/// https://html.spec.whatwg.org/multipage/#replacement-enabled
#[derive(Debug, Deserialize, Serialize)]
pub enum HistoryEntryReplacement {
    /// Traverse the history with replacement enabled.
    Enabled,
    /// Traverse the history with replacement disabled.
    Disabled,
}

/// Messages from the script to the constellation.
#[derive(Deserialize, Serialize)]
pub enum ScriptMsg {
    /// Request to complete the transfer of a set of ports to a router.
    CompleteMessagePortTransfer(MessagePortRouterId, Vec<MessagePortId>),
    /// The results of attempting to complete the transfer of a batch of ports.
    MessagePortTransferResult(
        /* The router whose transfer of ports succeeded, if any */
        Option<MessagePortRouterId>,
        /* The ids of ports transferred successfully */
        Vec<MessagePortId>,
        /* The ids, and buffers, of ports whose transfer failed */
        HashMap<MessagePortId, VecDeque<PortMessageTask>>,
    ),
    /// A new message-port was created or transferred, with corresponding control-sender.
    NewMessagePort(MessagePortRouterId, MessagePortId),
    /// A global has started managing message-ports
    NewMessagePortRouter(MessagePortRouterId, IpcSender<MessagePortMsg>),
    /// A global has stopped managing message-ports
    RemoveMessagePortRouter(MessagePortRouterId),
    /// A task requires re-routing to an already shipped message-port.
    RerouteMessagePort(MessagePortId, PortMessageTask),
    /// A message-port was shipped, let the entangled port know.
    MessagePortShipped(MessagePortId),
    /// A message-port has been discarded by script.
    RemoveMessagePort(MessagePortId),
    /// Entangle two message-ports.
    EntanglePorts(MessagePortId, MessagePortId),
    /// A global has started managing broadcast-channels.
    NewBroadcastChannelRouter(
        BroadcastChannelRouterId,
        IpcSender<BroadcastMsg>,
        ImmutableOrigin,
    ),
    /// A global has stopped managing broadcast-channels.
    RemoveBroadcastChannelRouter(BroadcastChannelRouterId, ImmutableOrigin),
    /// A global started managing broadcast channels for a given channel-name.
    NewBroadcastChannelNameInRouter(BroadcastChannelRouterId, String, ImmutableOrigin),
    /// A global stopped managing broadcast channels for a given channel-name.
    RemoveBroadcastChannelNameInRouter(BroadcastChannelRouterId, String, ImmutableOrigin),
    /// Broadcast a message to all same-origin broadcast channels,
    /// excluding the source of the broadcast.
    ScheduleBroadcast(BroadcastChannelRouterId, BroadcastMsg),
    /// Forward a message to the embedder.
    ForwardToEmbedder(EmbedderMsg),
    /// Requests are sent to constellation and fetches are checked manually
    /// for cross-origin loads
    InitiateNavigateRequest(RequestBuilder, /* cancellation_chan */ IpcReceiver<()>),
    /// Broadcast a storage event to every same-origin pipeline.
    /// The strings are key, old value and new value.
    BroadcastStorageEvent(
        StorageType,
        ServoUrl,
        Option<String>,
        Option<String>,
        Option<String>,
    ),
    /// Indicates whether this pipeline is currently running animations.
    ChangeRunningAnimationsState(AnimationState),
    /// Requests that a new 2D canvas thread be created. (This is done in the constellation because
    /// 2D canvases may use the GPU and we don't want to give untrusted content access to the GPU.)
    CreateCanvasPaintThread(
        UntypedSize2D<u64>,
        IpcSender<(IpcSender<CanvasMsg>, CanvasId)>,
    ),
    /// Notifies the constellation that this pipeline is requesting focus.
    ///
    /// When this message is sent, the sender pipeline has already its local
    /// focus state updated. The constellation, after receiving this message,
    /// will broadcast messages to other pipelines that are affected by this
    /// focus operation.
    ///
    /// The first field contains the browsing context ID of the container
    /// element if one was focused.
    ///
    /// The second field is a sequence number that the constellation should use
    /// when sending a focus-related message to the sender pipeline next time.
    Focus(Option<BrowsingContextId>, FocusSequenceNumber),
    /// Get the top-level browsing context info for a given browsing context.
    GetTopForBrowsingContext(
        BrowsingContextId,
        IpcSender<Option<TopLevelBrowsingContextId>>,
    ),
    /// Get the browsing context id of the browsing context in which pipeline is
    /// embedded and the parent pipeline id of that browsing context.
    GetBrowsingContextInfo(
        PipelineId,
        IpcSender<Option<(BrowsingContextId, Option<PipelineId>)>>,
    ),
    /// Get the nth child browsing context ID for a given browsing context, sorted in tree order.
    GetChildBrowsingContextId(
        BrowsingContextId,
        usize,
        IpcSender<Option<BrowsingContextId>>,
    ),
    /// All pending loads are complete, and the `load` event for this pipeline
    /// has been dispatched.
    LoadComplete,
    /// A new load has been requested, with an option to replace the current entry once loaded
    /// instead of adding a new entry.
    LoadUrl(LoadData, HistoryEntryReplacement),
    /// Abort loading after sending a LoadUrl message.
    AbortLoadUrl,
    /// Post a message to the currently active window of a given browsing context.
    PostMessage {
        /// The target of the posted message.
        target: BrowsingContextId,
        /// The source of the posted message.
        source: PipelineId,
        /// The expected origin of the target.
        target_origin: Option<ImmutableOrigin>,
        /// The source origin of the message.
        /// https://html.spec.whatwg.org/multipage/#dom-messageevent-origin
        source_origin: ImmutableOrigin,
        /// The data to be posted.
        data: StructuredSerializedData,
    },
    /// Inform the constellation that a fragment was navigated to and whether or not it was a replacement navigation.
    NavigatedToFragment(ServoUrl, HistoryEntryReplacement),
    /// HTMLIFrameElement Forward or Back traversal.
    TraverseHistory(TraversalDirection),
    /// Inform the constellation of a pushed history state.
    PushHistoryState(HistoryStateId, ServoUrl),
    /// Inform the constellation of a replaced history state.
    ReplaceHistoryState(HistoryStateId, ServoUrl),
    /// Gets the length of the joint session history from the constellation.
    JointSessionHistoryLength(IpcSender<u32>),
    /// Notification that this iframe should be removed.
    /// Returns a list of pipelines which were closed.
    RemoveIFrame(BrowsingContextId, IpcSender<Vec<PipelineId>>),
    /// Notifies constellation that an iframe's visibility has been changed.
    VisibilityChangeComplete(bool),
    /// A load has been requested in an IFrame.
    ScriptLoadedURLInIFrame(IFrameLoadInfoWithData),
    /// A load of the initial `about:blank` has been completed in an IFrame.
    ScriptNewIFrame(IFrameLoadInfoWithData, IpcSender<LayoutControlMsg>),
    /// Script has opened a new auxiliary browsing context.
    ScriptNewAuxiliary(
        AuxiliaryBrowsingContextLoadInfo,
        IpcSender<LayoutControlMsg>,
    ),
    /// Mark a new document as active
    ActivateDocument,
    /// Set the document state for a pipeline (used by screenshot / reftests)
    SetDocumentState(DocumentState),
    /// Update the pipeline Url, which can change after redirections.
    SetFinalUrl(ServoUrl),
    /// Script has handled a touch event, and either prevented or allowed default actions.
    TouchEventProcessed(EventResult),
    /// A log entry, with the top-level browsing context id and thread name
    LogEntry(Option<String>, LogEntry),
    /// Discard the document.
    DiscardDocument,
    /// Discard the browsing context.
    DiscardTopLevelBrowsingContext,
    /// Notifies the constellation that this pipeline has exited.
    PipelineExited,
    /// Send messages from postMessage calls from serviceworker
    /// to constellation for storing in service worker manager
    ForwardDOMMessage(DOMMessage, ServoUrl),
    /// https://w3c.github.io/ServiceWorker/#schedule-job-algorithm.
    ScheduleJob(Job),
    /// Get Window Informations size and position
    GetClientWindow(IpcSender<(DeviceIntSize, DeviceIntPoint)>),
    /// Get the screen size (pixel)
    GetScreenSize(IpcSender<DeviceIntSize>),
    /// Get the available screen size (pixel)
    GetScreenAvailSize(IpcSender<DeviceIntSize>),
    /// Notifies the constellation about media session events
    /// (i.e. when there is metadata for the active media session, playback state changes...).
    MediaSessionEvent(PipelineId, MediaSessionEvent),
    /// Create a WebGPU Adapter instance
    RequestAdapter(
        IpcSender<WebGPUResponseResult>,
        wgpu::instance::RequestAdapterOptions,
        SmallVec<[wgpu::id::AdapterId; 4]>,
    ),
    /// Get WebGPU channel
    GetWebGPUChan(IpcSender<WebGPU>),
    /// Notify the constellation of a pipeline's document's title.
    TitleChanged(PipelineId, String),
}

impl fmt::Debug for ScriptMsg {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        use self::ScriptMsg::*;
        let variant = match *self {
            CompleteMessagePortTransfer(..) => "CompleteMessagePortTransfer",
            MessagePortTransferResult(..) => "MessagePortTransferResult",
            NewMessagePortRouter(..) => "NewMessagePortRouter",
            RemoveMessagePortRouter(..) => "RemoveMessagePortRouter",
            NewMessagePort(..) => "NewMessagePort",
            RerouteMessagePort(..) => "RerouteMessagePort",
            RemoveMessagePort(..) => "RemoveMessagePort",
            MessagePortShipped(..) => "MessagePortShipped",
            EntanglePorts(..) => "EntanglePorts",
            NewBroadcastChannelRouter(..) => "NewBroadcastChannelRouter",
            RemoveBroadcastChannelRouter(..) => "RemoveBroadcastChannelRouter",
            RemoveBroadcastChannelNameInRouter(..) => "RemoveBroadcastChannelNameInRouter",
            NewBroadcastChannelNameInRouter(..) => "NewBroadcastChannelNameInRouter",
            ScheduleBroadcast(..) => "ScheduleBroadcast",
            ForwardToEmbedder(..) => "ForwardToEmbedder",
            InitiateNavigateRequest(..) => "InitiateNavigateRequest",
            BroadcastStorageEvent(..) => "BroadcastStorageEvent",
            ChangeRunningAnimationsState(..) => "ChangeRunningAnimationsState",
            CreateCanvasPaintThread(..) => "CreateCanvasPaintThread",
            Focus(..) => "Focus",
            GetBrowsingContextInfo(..) => "GetBrowsingContextInfo",
            GetTopForBrowsingContext(..) => "GetParentBrowsingContext",
            GetChildBrowsingContextId(..) => "GetChildBrowsingContextId",
            LoadComplete => "LoadComplete",
            LoadUrl(..) => "LoadUrl",
            AbortLoadUrl => "AbortLoadUrl",
            PostMessage { .. } => "PostMessage",
            NavigatedToFragment(..) => "NavigatedToFragment",
            TraverseHistory(..) => "TraverseHistory",
            PushHistoryState(..) => "PushHistoryState",
            ReplaceHistoryState(..) => "ReplaceHistoryState",
            JointSessionHistoryLength(..) => "JointSessionHistoryLength",
            RemoveIFrame(..) => "RemoveIFrame",
            VisibilityChangeComplete(..) => "VisibilityChangeComplete",
            ScriptLoadedURLInIFrame(..) => "ScriptLoadedURLInIFrame",
            ScriptNewIFrame(..) => "ScriptNewIFrame",
            ScriptNewAuxiliary(..) => "ScriptNewAuxiliary",
            ActivateDocument => "ActivateDocument",
            SetDocumentState(..) => "SetDocumentState",
            SetFinalUrl(..) => "SetFinalUrl",
            TouchEventProcessed(..) => "TouchEventProcessed",
            LogEntry(..) => "LogEntry",
            DiscardDocument => "DiscardDocument",
            DiscardTopLevelBrowsingContext => "DiscardTopLevelBrowsingContext",
            PipelineExited => "PipelineExited",
            ForwardDOMMessage(..) => "ForwardDOMMessage",
            ScheduleJob(..) => "ScheduleJob",
            GetClientWindow(..) => "GetClientWindow",
            GetScreenSize(..) => "GetScreenSize",
            GetScreenAvailSize(..) => "GetScreenAvailSize",
            MediaSessionEvent(..) => "MediaSessionEvent",
            RequestAdapter(..) => "RequestAdapter",
            GetWebGPUChan(..) => "GetWebGPUChan",
            TitleChanged(..) => "TitleChanged",
        };
        write!(formatter, "ScriptMsg::{}", variant)
    }
}

/// Entities required to spawn service workers
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ScopeThings {
    /// script resource url
    pub script_url: ServoUrl,
    /// network load origin of the resource
    pub worker_load_origin: WorkerScriptLoadOrigin,
    /// base resources required to create worker global scopes
    pub init: WorkerGlobalScopeInit,
    /// the port to receive devtools message from
    pub devtools_chan: Option<IpcSender<ScriptToDevtoolsControlMsg>>,
    /// service worker id
    pub worker_id: WorkerId,
}

/// Message that gets passed to service worker scope on postMessage
#[derive(Debug, Deserialize, Serialize)]
pub struct DOMMessage {
    /// The origin of the message
    pub origin: ImmutableOrigin,
    /// The payload of the message
    pub data: StructuredSerializedData,
}

/// Channels to allow service worker manager to communicate with constellation and resource thread
#[derive(Deserialize, Serialize)]
pub struct SWManagerSenders {
    /// Sender of messages to the constellation.
    pub swmanager_sender: IpcSender<SWManagerMsg>,
    /// Sender for communicating with resource thread.
    pub resource_sender: IpcSender<CoreResourceMsg>,
    /// Sender of messages to the manager.
    pub own_sender: IpcSender<ServiceWorkerMsg>,
    /// Receiver of messages from the constellation.
    pub receiver: IpcReceiver<ServiceWorkerMsg>,
}

/// Messages sent to Service Worker Manager thread
#[derive(Debug, Deserialize, Serialize)]
pub enum ServiceWorkerMsg {
    /// Timeout message sent by active service workers
    Timeout(ServoUrl),
    /// Message sent by constellation to forward to a running service worker
    ForwardDOMMessage(DOMMessage, ServoUrl),
    /// https://w3c.github.io/ServiceWorker/#schedule-job-algorithm
    ScheduleJob(Job),
    /// Exit the service worker manager
    Exit,
}

#[derive(Debug, Deserialize, PartialEq, Serialize)]
/// https://w3c.github.io/ServiceWorker/#dfn-job-type
pub enum JobType {
    /// <https://w3c.github.io/ServiceWorker/#register>
    Register,
    /// <https://w3c.github.io/ServiceWorker/#unregister-algorithm>
    Unregister,
    /// <https://w3c.github.io/ServiceWorker/#update-algorithm
    Update,
}

#[derive(Debug, Deserialize, Serialize)]
/// The kind of error the job promise should be rejected with.
pub enum JobError {
    /// https://w3c.github.io/ServiceWorker/#reject-job-promise
    TypeError,
    /// https://w3c.github.io/ServiceWorker/#reject-job-promise
    SecurityError,
}

#[derive(Debug, Deserialize, Serialize)]
/// Messages sent from Job algorithms steps running in the SW manager,
/// in order to resolve or reject the job promise.
pub enum JobResult {
    /// https://w3c.github.io/ServiceWorker/#reject-job-promise
    RejectPromise(JobError),
    /// https://w3c.github.io/ServiceWorker/#resolve-job-promise
    ResolvePromise(Job, JobResultValue),
}

#[derive(Debug, Deserialize, Serialize)]
/// Jobs are resolved with the help of various values.
pub enum JobResultValue {
    /// Data representing a serviceworker registration.
    Registration {
        /// The Id of the registration.
        id: ServiceWorkerRegistrationId,
        /// The installing worker, if any.
        installing_worker: Option<ServiceWorkerId>,
        /// The waiting worker, if any.
        waiting_worker: Option<ServiceWorkerId>,
        /// The active worker, if any.
        active_worker: Option<ServiceWorkerId>,
    },
}

#[derive(Debug, Deserialize, Serialize)]
/// https://w3c.github.io/ServiceWorker/#dfn-job
pub struct Job {
    /// <https://w3c.github.io/ServiceWorker/#dfn-job-type>
    pub job_type: JobType,
    /// <https://w3c.github.io/ServiceWorker/#dfn-job-scope-url>
    pub scope_url: ServoUrl,
    /// <https://w3c.github.io/ServiceWorker/#dfn-job-script-url>
    pub script_url: ServoUrl,
    /// <https://w3c.github.io/ServiceWorker/#dfn-job-client>
    pub client: IpcSender<JobResult>,
    /// <https://w3c.github.io/ServiceWorker/#job-referrer>
    pub referrer: ServoUrl,
    /// Various data needed to process job.
    pub scope_things: Option<ScopeThings>,
}

impl Job {
    /// https://w3c.github.io/ServiceWorker/#create-job-algorithm
    pub fn create_job(
        job_type: JobType,
        scope_url: ServoUrl,
        script_url: ServoUrl,
        client: IpcSender<JobResult>,
        referrer: ServoUrl,
        scope_things: Option<ScopeThings>,
    ) -> Job {
        Job {
            job_type,
            scope_url,
            script_url,
            client,
            referrer,
            scope_things,
        }
    }
}

impl PartialEq for Job {
    /// Equality criteria as described in https://w3c.github.io/ServiceWorker/#dfn-job-equivalent
    fn eq(&self, other: &Self) -> bool {
        // TODO: match on job type, take worker type and `update_via_cache_mode` into account.
        let same_job = self.job_type == other.job_type;
        if same_job {
            match self.job_type {
                JobType::Register | JobType::Update => {
                    self.scope_url == other.scope_url && self.script_url == other.script_url
                },
                JobType::Unregister => self.scope_url == other.scope_url,
            }
        } else {
            false
        }
    }
}

/// Messages outgoing from the Service Worker Manager thread to constellation
#[derive(Debug, Deserialize, Serialize)]
pub enum SWManagerMsg {
    /// Placeholder to keep the enum,
    /// as it will be needed when implementing
    /// https://github.com/servo/servo/issues/24660
    PostMessageToClient,
}

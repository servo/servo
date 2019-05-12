/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::AnimationState;
use crate::AuxiliaryBrowsingContextLoadInfo;
use crate::DocumentState;
use crate::IFrameLoadInfo;
use crate::IFrameLoadInfoWithData;
use crate::LayoutControlMsg;
use crate::LoadData;
use crate::WindowSizeType;
use crate::WorkerGlobalScopeInit;
use crate::WorkerScriptLoadOrigin;
use canvas_traits::canvas::{CanvasId, CanvasMsg};
use devtools_traits::{ScriptToDevtoolsControlMsg, WorkerId};
use embedder_traits::EmbedderMsg;
use euclid::{Size2D, TypedSize2D};
use gfx_traits::Epoch;
use ipc_channel::ipc::{IpcReceiver, IpcSender};
use msg::constellation_msg::{BrowsingContextId, PipelineId, TopLevelBrowsingContextId};
use msg::constellation_msg::{HistoryStateId, TraversalDirection};
use net_traits::request::RequestBuilder;
use net_traits::storage_thread::StorageType;
use net_traits::CoreResourceMsg;
use servo_url::ImmutableOrigin;
use servo_url::ServoUrl;
use std::fmt;
use style_traits::viewport::ViewportConstraints;
use style_traits::CSSPixel;
use webrender_api::{DeviceIntPoint, DeviceIntSize};

/// A particular iframe's size, associated with a browsing context.
#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub struct IFrameSize {
    /// The child browsing context for this iframe.
    pub id: BrowsingContextId,
    /// The size of the iframe.
    pub size: TypedSize2D<f32, CSSPixel>,
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
    /// Indicates whether this pipeline is currently running animations.
    ChangeRunningAnimationsState(PipelineId, AnimationState),
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
            ChangeRunningAnimationsState(..) => "ChangeRunningAnimationsState",
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
    CreateCanvasPaintThread(Size2D<u64>, IpcSender<(IpcSender<CanvasMsg>, CanvasId)>),
    /// Notifies the constellation that this frame has received focus.
    Focus,
    /// Requests that the constellation retrieve the current contents of the clipboard
    GetClipboardContents(IpcSender<String>),
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
        /// The data to be posted.
        data: Vec<u8>,
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
    ScriptNewIFrame(IFrameLoadInfo, IpcSender<LayoutControlMsg>),
    /// Script has opened a new auxiliary browsing context.
    ScriptNewAuxiliary(
        AuxiliaryBrowsingContextLoadInfo,
        IpcSender<LayoutControlMsg>,
    ),
    /// Requests that the constellation set the contents of the clipboard
    SetClipboardContents(String),
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
    /// Store the data required to activate a service worker for the given scope
    RegisterServiceWorker(ScopeThings, ServoUrl),
    /// Get Window Informations size and position
    GetClientWindow(IpcSender<(DeviceIntSize, DeviceIntPoint)>),
    /// Get the screen size (pixel)
    GetScreenSize(IpcSender<(DeviceIntSize)>),
    /// Get the available screen size (pixel)
    GetScreenAvailSize(IpcSender<(DeviceIntSize)>),
}

impl fmt::Debug for ScriptMsg {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        use self::ScriptMsg::*;
        let variant = match *self {
            ForwardToEmbedder(..) => "ForwardToEmbedder",
            InitiateNavigateRequest(..) => "InitiateNavigateRequest",
            BroadcastStorageEvent(..) => "BroadcastStorageEvent",
            ChangeRunningAnimationsState(..) => "ChangeRunningAnimationsState",
            CreateCanvasPaintThread(..) => "CreateCanvasPaintThread",
            Focus => "Focus",
            GetClipboardContents(..) => "GetClipboardContents",
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
            SetClipboardContents(..) => "SetClipboardContents",
            ActivateDocument => "ActivateDocument",
            SetDocumentState(..) => "SetDocumentState",
            SetFinalUrl(..) => "SetFinalUrl",
            TouchEventProcessed(..) => "TouchEventProcessed",
            LogEntry(..) => "LogEntry",
            DiscardDocument => "DiscardDocument",
            DiscardTopLevelBrowsingContext => "DiscardTopLevelBrowsingContext",
            PipelineExited => "PipelineExited",
            ForwardDOMMessage(..) => "ForwardDOMMessage",
            RegisterServiceWorker(..) => "RegisterServiceWorker",
            GetClientWindow(..) => "GetClientWindow",
            GetScreenSize(..) => "GetScreenSize",
            GetScreenAvailSize(..) => "GetScreenAvailSize",
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
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct DOMMessage(pub Vec<u8>);

/// Channels to allow service worker manager to communicate with constellation and resource thread
pub struct SWManagerSenders {
    /// sender for communicating with constellation
    pub swmanager_sender: IpcSender<SWManagerMsg>,
    /// sender for communicating with resource thread
    pub resource_sender: IpcSender<CoreResourceMsg>,
}

/// Messages sent to Service Worker Manager thread
#[derive(Debug, Deserialize, Serialize)]
pub enum ServiceWorkerMsg {
    /// Message to register the service worker
    RegisterServiceWorker(ScopeThings, ServoUrl),
    /// Timeout message sent by active service workers
    Timeout(ServoUrl),
    /// Message sent by constellation to forward to a running service worker
    ForwardDOMMessage(DOMMessage, ServoUrl),
    /// Exit the service worker manager
    Exit,
}

/// Messages outgoing from the Service Worker Manager thread to constellation
#[derive(Debug, Deserialize, Serialize)]
pub enum SWManagerMsg {
    /// Provide the constellation with a means of communicating with the Service Worker Manager
    OwnSender(IpcSender<ServiceWorkerMsg>),
}

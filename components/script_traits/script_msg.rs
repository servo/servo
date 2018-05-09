/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use AnimationState;
use DocumentState;
use IFrameLoadInfo;
use IFrameLoadInfoWithData;
use LayoutControlMsg;
use LoadData;
use WorkerGlobalScopeInit;
use WorkerScriptLoadOrigin;
use canvas_traits::canvas::{CanvasMsg, CanvasId};
use devtools_traits::{ScriptToDevtoolsControlMsg, WorkerId};
use euclid::{Size2D, TypedSize2D};
use gfx_traits::Epoch;
use ipc_channel::ipc::{IpcReceiver, IpcSender};
use msg::constellation_msg::{BrowsingContextId, HistoryStateId, PipelineId, TraversalDirection};
use msg::constellation_msg::{InputMethodType, Key, KeyModifiers, KeyState};
use net_traits::CoreResourceMsg;
use net_traits::request::RequestInit;
use net_traits::storage_thread::StorageType;
use servo_url::ImmutableOrigin;
use servo_url::ServoUrl;
use style_traits::CSSPixel;
use style_traits::cursor::CursorKind;
use style_traits::viewport::ViewportConstraints;
use webrender_api::{DeviceIntPoint, DeviceUintSize};

/// Messages from the layout to the constellation.
#[derive(Deserialize, Serialize)]
pub enum LayoutMsg {
    /// Indicates whether this pipeline is currently running animations.
    ChangeRunningAnimationsState(PipelineId, AnimationState),
    /// Inform the constellation of the size of the iframe's viewport.
    IFrameSizes(Vec<(BrowsingContextId, TypedSize2D<f32, CSSPixel>)>),
    /// Requests that the constellation inform the compositor that it needs to record
    /// the time when the frame with the given ID (epoch) is painted.
    PendingPaintMetric(PipelineId, Epoch),
    /// Requests that the constellation inform the compositor of the a cursor change.
    SetCursor(CursorKind),
    /// Notifies the constellation that the viewport has been constrained in some manner
    ViewportConstrained(PipelineId, ViewportConstraints),
}

/// Whether a DOM event was prevented by web content
#[derive(Deserialize, Serialize)]
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

/// Messages from the script to the constellation.
#[derive(Deserialize, Serialize)]
pub enum ScriptMsg {
    /// Requests are sent to constellation and fetches are checked manually
    /// for cross-origin loads
    InitiateNavigateRequest(RequestInit, /* cancellation_chan */ IpcReceiver<()>),
    /// Broadcast a storage event to every same-origin pipeline.
    /// The strings are key, old value and new value.
    BroadcastStorageEvent(StorageType, ServoUrl, Option<String>, Option<String>, Option<String>),
    /// Indicates whether this pipeline is currently running animations.
    ChangeRunningAnimationsState(AnimationState),
    /// Requests that a new 2D canvas thread be created. (This is done in the constellation because
    /// 2D canvases may use the GPU and we don't want to give untrusted content access to the GPU.)
    CreateCanvasPaintThread(Size2D<i32>, IpcSender<(IpcSender<CanvasMsg>, CanvasId)>),
    /// Notifies the constellation that this frame has received focus.
    Focus,
    /// Requests that the constellation retrieve the current contents of the clipboard
    GetClipboardContents(IpcSender<String>),
    /// Get the browsing context id for a given pipeline.
    GetBrowsingContextId(PipelineId, IpcSender<Option<BrowsingContextId>>),
    /// Get the parent info for a given pipeline.
    GetParentInfo(PipelineId, IpcSender<Option<PipelineId>>),
    /// <head> tag finished parsing
    HeadParsed,
    /// All pending loads are complete, and the `load` event for this pipeline
    /// has been dispatched.
    LoadComplete,
    /// A new load has been requested, with an option to replace the current entry once loaded
    /// instead of adding a new entry.
    LoadUrl(LoadData, bool),
    /// Abort loading after sending a LoadUrl message.
    AbortLoadUrl,
    /// Post a message to the currently active window of a given browsing context.
    PostMessage(BrowsingContextId, Option<ImmutableOrigin>, Vec<u8>),
    /// HTMLIFrameElement Forward or Back traversal.
    TraverseHistory(TraversalDirection),
    /// Inform the constellation of a pushed history state.
    PushHistoryState(HistoryStateId, ServoUrl),
    /// Inform the constellation of a replaced history state.
    ReplaceHistoryState(HistoryStateId, ServoUrl),
    /// Gets the length of the joint session history from the constellation.
    JointSessionHistoryLength(IpcSender<u32>),
    /// Favicon detected
    NewFavicon(ServoUrl),
    /// Status message to be displayed in the chrome, eg. a link URL on mouseover.
    NodeStatus(Option<String>),
    /// Notification that this iframe should be removed.
    /// Returns a list of pipelines which were closed.
    RemoveIFrame(BrowsingContextId, IpcSender<Vec<PipelineId>>),
    /// Change pipeline visibility
    SetVisible(bool),
    /// Notifies constellation that an iframe's visibility has been changed.
    VisibilityChangeComplete(bool),
    /// A load has been requested in an IFrame.
    ScriptLoadedURLInIFrame(IFrameLoadInfoWithData),
    /// A load of the initial `about:blank` has been completed in an IFrame.
    ScriptNewIFrame(IFrameLoadInfo, IpcSender<LayoutControlMsg>),
    /// Requests that the constellation set the contents of the clipboard
    SetClipboardContents(String),
    /// Mark a new document as active
    ActivateDocument,
    /// Set the document state for a pipeline (used by screenshot / reftests)
    SetDocumentState(DocumentState),
    /// Update the pipeline Url, which can change after redirections.
    SetFinalUrl(ServoUrl),
    /// Check if an alert dialog box should be presented
    Alert(String, IpcSender<bool>),
    /// Set title of current page
    /// <https://html.spec.whatwg.org/multipage/#document.title>
    SetTitle(Option<String>),
    /// Send a key event
    SendKeyEvent(Option<char>, Key, KeyState, KeyModifiers),
    /// Move the window to a point
    MoveTo(DeviceIntPoint),
    /// Resize the window to size
    ResizeTo(DeviceUintSize),
    /// Script has handled a touch event, and either prevented or allowed default actions.
    TouchEventProcessed(EventResult),
    /// A log entry, with the top-level browsing context id and thread name
    LogEntry(Option<String>, LogEntry),
    /// Discard the document.
    DiscardDocument,
    /// Notifies the constellation that this pipeline has exited.
    PipelineExited,
    /// Send messages from postMessage calls from serviceworker
    /// to constellation for storing in service worker manager
    ForwardDOMMessage(DOMMessage, ServoUrl),
    /// Store the data required to activate a service worker for the given scope
    RegisterServiceWorker(ScopeThings, ServoUrl),
    /// Enter or exit fullscreen
    SetFullscreenState(bool),
    /// Get Window Informations size and position
    GetClientWindow(IpcSender<(DeviceUintSize, DeviceIntPoint)>),
    /// Get the screen size (pixel)
    GetScreenSize(IpcSender<(DeviceUintSize)>),
    /// Get the available screen size (pixel)
    GetScreenAvailSize(IpcSender<(DeviceUintSize)>),
    /// Request to present an IME to the user when an editable element is focused.
    ShowIME(InputMethodType),
    /// Request to hide the IME when the editable element is blurred.
    HideIME,
    /// Requests that the compositor shut down.
    Exit,
}

/// Entities required to spawn service workers
#[derive(Clone, Deserialize, Serialize)]
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
#[derive(Deserialize, Serialize)]
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
#[derive(Deserialize, Serialize)]
pub enum SWManagerMsg {
    /// Provide the constellation with a means of communicating with the Service Worker Manager
    OwnSender(IpcSender<ServiceWorkerMsg>),
}

/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use AnimationState;
use CompositorEvent;
use DocumentState;
use IFrameLoadInfo;
use IFrameLoadInfoWithData;
use LayoutControlMsg;
use LoadData;
use MozBrowserEvent;
use WorkerGlobalScopeInit;
use WorkerScriptLoadOrigin;
use canvas_traits::CanvasMsg;
use devtools_traits::{ScriptToDevtoolsControlMsg, WorkerId};
use euclid::point::Point2D;
use euclid::size::{Size2D, TypedSize2D};
use ipc_channel::ipc::IpcSender;
use msg::constellation_msg::{FrameId, FrameType, PipelineId, TraversalDirection};
use msg::constellation_msg::{Key, KeyModifiers, KeyState};
use net_traits::CoreResourceMsg;
use net_traits::storage_thread::StorageType;
use offscreen_gl_context::{GLContextAttributes, GLLimits};
use servo_url::ImmutableOrigin;
use servo_url::ServoUrl;
use style_traits::CSSPixel;
use style_traits::cursor::Cursor;
use style_traits::viewport::ViewportConstraints;
use webrender_traits::ClipId;

/// Messages from the layout to the constellation.
#[derive(Deserialize, Serialize)]
pub enum LayoutMsg {
    /// Indicates whether this pipeline is currently running animations.
    ChangeRunningAnimationsState(PipelineId, AnimationState),
    /// Inform the constellation of the size of the pipeline's viewport.
    FrameSizes(Vec<(PipelineId, TypedSize2D<f32, CSSPixel>)>),
    /// Requests that the constellation inform the compositor of the a cursor change.
    SetCursor(Cursor),
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
    /// Broadcast a storage event to every same-origin pipeline.
    /// The strings are key, old value and new value.
    BroadcastStorageEvent(PipelineId, StorageType, ServoUrl, Option<String>, Option<String>, Option<String>),
    /// Indicates whether this pipeline is currently running animations.
    ChangeRunningAnimationsState(PipelineId, AnimationState),
    /// Requests that a new 2D canvas thread be created. (This is done in the constellation because
    /// 2D canvases may use the GPU and we don't want to give untrusted content access to the GPU.)
    CreateCanvasPaintThread(Size2D<i32>, IpcSender<IpcSender<CanvasMsg>>),
    /// Requests that a new WebGL thread be created. (This is done in the constellation because
    /// WebGL uses the GPU and we don't want to give untrusted content access to the GPU.)
    CreateWebGLPaintThread(Size2D<i32>,
                           GLContextAttributes,
                           IpcSender<Result<(IpcSender<CanvasMsg>, GLLimits), String>>),
    /// Notifies the constellation that this frame has received focus.
    Focus(PipelineId),
    /// Forward an event that was sent to the parent window.
    ForwardEvent(PipelineId, CompositorEvent),
    /// Requests that the constellation retrieve the current contents of the clipboard
    GetClipboardContents(IpcSender<String>),
    /// Get the frame id for a given pipeline.
    GetFrameId(PipelineId, IpcSender<Option<FrameId>>),
    /// Get the parent info for a given pipeline.
    GetParentInfo(PipelineId, IpcSender<Option<(PipelineId, FrameType)>>),
    /// <head> tag finished parsing
    HeadParsed,
    /// All pending loads are complete, and the `load` event for this pipeline
    /// has been dispatched.
    LoadComplete(PipelineId),
    /// A new load has been requested, with an option to replace the current entry once loaded
    /// instead of adding a new entry.
    LoadUrl(PipelineId, LoadData, bool),
    /// Post a message to the currently active window of a given browsing context.
    PostMessage(FrameId, Option<ImmutableOrigin>, Vec<u8>),
    /// Dispatch a mozbrowser event to the parent of this pipeline.
    /// The first PipelineId is for the parent, the second is for the originating pipeline.
    MozBrowserEvent(PipelineId, PipelineId, MozBrowserEvent),
    /// HTMLIFrameElement Forward or Back traversal.
    TraverseHistory(Option<PipelineId>, TraversalDirection),
    /// Gets the length of the joint session history from the constellation.
    JointSessionHistoryLength(PipelineId, IpcSender<u32>),
    /// Favicon detected
    NewFavicon(ServoUrl),
    /// Status message to be displayed in the chrome, eg. a link URL on mouseover.
    NodeStatus(Option<String>),
    /// Notification that this iframe should be removed.
    /// Returns a list of pipelines which were closed.
    RemoveIFrame(FrameId, IpcSender<Vec<PipelineId>>),
    /// Change pipeline visibility
    SetVisible(PipelineId, bool),
    /// Notifies constellation that an iframe's visibility has been changed.
    VisibilityChangeComplete(PipelineId, bool),
    /// A load has been requested in an IFrame.
    ScriptLoadedURLInIFrame(IFrameLoadInfoWithData),
    /// A load of `about:blank` has been completed in an IFrame.
    ScriptLoadedAboutBlankInIFrame(IFrameLoadInfo, IpcSender<LayoutControlMsg>),
    /// Requests that the constellation set the contents of the clipboard
    SetClipboardContents(String),
    /// Mark a new document as active
    ActivateDocument(PipelineId),
    /// Set the document state for a pipeline (used by screenshot / reftests)
    SetDocumentState(PipelineId, DocumentState),
    /// Update the pipeline Url, which can change after redirections.
    SetFinalUrl(PipelineId, ServoUrl),
    /// Check if an alert dialog box should be presented
    Alert(PipelineId, String, IpcSender<bool>),
    /// Scroll a page in a window
    ScrollFragmentPoint(ClipId, Point2D<f32>, bool),
    /// Set title of current page
    /// https://html.spec.whatwg.org/multipage/#document.title
    SetTitle(PipelineId, Option<String>),
    /// Send a key event
    SendKeyEvent(Option<char>, Key, KeyState, KeyModifiers),
    /// Get Window Informations size and position
    GetClientWindow(IpcSender<(Size2D<u32>, Point2D<i32>)>),
    /// Move the window to a point
    MoveTo(Point2D<i32>),
    /// Resize the window to size
    ResizeTo(Size2D<u32>),
    /// Script has handled a touch event, and either prevented or allowed default actions.
    TouchEventProcessed(EventResult),
    /// A log entry, with the top-level frame id and thread name
    LogEntry(Option<FrameId>, Option<String>, LogEntry),
    /// Notifies the constellation that this pipeline has exited.
    PipelineExited(PipelineId),
    /// Send messages from postMessage calls from serviceworker
    /// to constellation for storing in service worker manager
    ForwardDOMMessage(DOMMessage, ServoUrl),
    /// Store the data required to activate a service worker for the given scope
    RegisterServiceWorker(ScopeThings, ServoUrl),
    /// Enter or exit fullscreen
    SetFullscreenState(bool),
    /// Requests that the compositor shut down.
    Exit,
}

/// Entities required to spawn service workers
#[derive(Deserialize, Serialize, Clone)]
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
#[derive(Deserialize, Serialize, Debug, Clone)]
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

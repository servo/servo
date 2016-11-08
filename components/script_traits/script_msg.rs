/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use AnimationState;
use CompositorEvent;
use DocumentState;
use IFrameLoadInfo;
use LoadData;
use MozBrowserEvent;
use WorkerGlobalScopeInit;
use WorkerScriptLoadOrigin;
use canvas_traits::CanvasMsg;
use devtools_traits::{ScriptToDevtoolsControlMsg, WorkerId};
use euclid::point::Point2D;
use euclid::size::Size2D;
use ipc_channel::ipc::IpcSender;
use msg::constellation_msg::{Key, KeyModifiers, KeyState};
use msg::constellation_msg::{PipelineId, TraversalDirection};
use net_traits::CoreResourceMsg;
use offscreen_gl_context::{GLContextAttributes, GLLimits};
use style_traits::cursor::Cursor;
use style_traits::viewport::ViewportConstraints;
use url::Url;

/// Messages from the layout to the constellation.
#[derive(Deserialize, Serialize)]
pub enum LayoutMsg {
    /// Indicates whether this pipeline is currently running animations.
    ChangeRunningAnimationsState(PipelineId, AnimationState),
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
    Warn(String)
}

/// Messages from the script to the constellation.
#[derive(Deserialize, Serialize)]
pub enum ScriptMsg {
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
    /// <head> tag finished parsing
    HeadParsed,
    /// All pending loads are complete, and the `load` event for this pipeline
    /// has been dispatched.
    LoadComplete(PipelineId),
    /// A new load has been requested, with an option to replace the current entry once loaded
    /// instead of adding a new entry.
    LoadUrl(PipelineId, LoadData, bool),
    /// Dispatch a mozbrowser event to the parent of this pipeline.
    /// The first PipelineId is for the parent, the second is for the originating pipeline.
    MozBrowserEvent(PipelineId, PipelineId, MozBrowserEvent),
    /// HTMLIFrameElement Forward or Back traversal.
    TraverseHistory(Option<PipelineId>, TraversalDirection),
    /// Gets the length of the joint session history from the constellation.
    JointSessionHistoryLength(PipelineId, IpcSender<u32>),
    /// Favicon detected
    NewFavicon(Url),
    /// Status message to be displayed in the chrome, eg. a link URL on mouseover.
    NodeStatus(Option<String>),
    /// Notification that this iframe should be removed.
    RemoveIFrame(PipelineId, Option<IpcSender<()>>),
    /// Change pipeline visibility
    SetVisible(PipelineId, bool),
    /// Notifies constellation that an iframe's visibility has been changed.
    VisibilityChangeComplete(PipelineId, bool),
    /// A load has been requested in an IFrame.
    ScriptLoadedURLInIFrame(IFrameLoadInfo),
    /// Requests that the constellation set the contents of the clipboard
    SetClipboardContents(String),
    /// Mark a new document as active
    ActivateDocument(PipelineId),
    /// Set the document state for a pipeline (used by screenshot / reftests)
    SetDocumentState(PipelineId, DocumentState),
    /// Update the pipeline Url, which can change after redirections.
    SetFinalUrl(PipelineId, Url),
    /// Check if an alert dialog box should be presented
    Alert(PipelineId, String, IpcSender<bool>),
    /// Scroll a page in a window
    ScrollFragmentPoint(PipelineId, Point2D<f32>, bool),
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
    /// A log entry, with the pipeline id and thread name
    LogEntry(Option<PipelineId>, Option<String>, LogEntry),
    /// Notifies the constellation that this pipeline has exited.
    PipelineExited(PipelineId),
    /// Send messages from postMessage calls from serviceworker
    /// to constellation for storing in service worker manager
    ForwardDOMMessage(DOMMessage, Url),
    /// Store the data required to activate a service worker for the given scope
    RegisterServiceWorker(ScopeThings, Url),
    /// Requests that the compositor shut down.
    Exit
}

/// Entities required to spawn service workers
#[derive(Deserialize, Serialize, Clone)]
pub struct ScopeThings {
    /// script resource url
    pub script_url: Url,
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
    pub resource_sender: IpcSender<CoreResourceMsg>
}

/// Messages sent to Service Worker Manager thread
#[derive(Deserialize, Serialize)]
pub enum ServiceWorkerMsg {
    /// Message to register the service worker
    RegisterServiceWorker(ScopeThings, Url),
    /// Timeout message sent by active service workers
    Timeout(Url),
    /// Message sent by constellation to forward to a running service worker
    ForwardDOMMessage(DOMMessage, Url),
    /// Exit the service worker manager
    Exit,
}

/// Messages outgoing from the Service Worker Manager thread to constellation
#[derive(Deserialize, Serialize)]
pub enum SWManagerMsg {
    /// Provide the constellation with a means of communicating with the Service Worker Manager
    OwnSender(IpcSender<ServiceWorkerMsg>)

}

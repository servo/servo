/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use canvas_traits::CanvasMsg;
use euclid::point::Point2D;
use euclid::size::Size2D;
use ipc_channel::ipc::IpcSender;
use msg::constellation_msg::{AnimationState, DocumentState, IframeLoadInfo, NavigationDirection};
use msg::constellation_msg::{Failure, MozBrowserEvent, PipelineId};
use msg::constellation_msg::{LoadData, SubpageId};
use msg::constellation_msg::{MouseButton, MouseEventType};
use offscreen_gl_context::GLContextAttributes;
use style_traits::viewport::ViewportConstraints;
use url::Url;
use util::cursor::Cursor;

/// Messages from the layout to the constellation.
#[derive(Deserialize, Serialize)]
pub enum LayoutMsg {
    /// Indicates whether this pipeline is currently running animations.
    ChangeRunningAnimationsState(PipelineId, AnimationState),
    /// Layout task failure.
    Failure(Failure),
    /// Requests that the constellation inform the compositor of the a cursor change.
    SetCursor(Cursor),
    /// Notifies the constellation that the viewport has been constrained in some manner
    ViewportConstrained(PipelineId, ViewportConstraints),
}

/// Messages from the script to the constellation.
#[derive(Deserialize, Serialize)]
pub enum ScriptMsg {
    /// Indicates whether this pipeline is currently running animations.
    ChangeRunningAnimationsState(PipelineId, AnimationState),
    /// Requests that a new 2D canvas thread be created. (This is done in the constellation because
    /// 2D canvases may use the GPU and we don't want to give untrusted content access to the GPU.)
    CreateCanvasPaintTask(Size2D<i32>, IpcSender<(IpcSender<CanvasMsg>, usize)>),
    /// Requests that a new WebGL thread be created. (This is done in the constellation because
    /// WebGL uses the GPU and we don't want to give untrusted content access to the GPU.)
    CreateWebGLPaintTask(Size2D<i32>,
                         GLContextAttributes,
                         IpcSender<Result<(IpcSender<CanvasMsg>, usize), String>>),
    /// Dispatched after the DOM load event has fired on a document
    /// Causes a `load` event to be dispatched to any enclosing frame context element
    /// for the given pipeline.
    DOMLoad(PipelineId),
    /// Script task failure.
    Failure(Failure),
    /// Notifies the constellation that this frame has received focus.
    Focus(PipelineId),
    /// Re-send a mouse button event that was sent to the parent window.
    ForwardMouseButtonEvent(PipelineId, MouseEventType, MouseButton, Point2D<f32>),
    /// Re-send a mouse move event that was sent to the parent window.
    ForwardMouseMoveEvent(PipelineId, Point2D<f32>),
    /// Requests that the constellation retrieve the current contents of the clipboard
    GetClipboardContents(IpcSender<String>),
    /// <head> tag finished parsing
    HeadParsed,
    /// All pending loads are complete.
    LoadComplete(PipelineId),
    /// A new load has been requested.
    LoadUrl(PipelineId, LoadData),
    /// Dispatch a mozbrowser event to a given iframe. Only available in experimental mode.
    MozBrowserEvent(PipelineId, SubpageId, MozBrowserEvent),
    /// HTMLIFrameElement Forward or Back navigation.
    Navigate(Option<(PipelineId, SubpageId)>, NavigationDirection),
    /// Favicon detected
    NewFavicon(Url),
    /// Status message to be displayed in the chrome, eg. a link URL on mouseover.
    NodeStatus(Option<String>),
    /// Notification that this iframe should be removed.
    RemoveIFrame(PipelineId),
    /// A load has been requested in an IFrame.
    ScriptLoadedURLInIFrame(IframeLoadInfo),
    /// Requests that the constellation set the contents of the clipboard
    SetClipboardContents(String),
    /// Mark a new document as active
    ActivateDocument(PipelineId),
    /// Set the document state for a pipeline (used by screenshot / reftests)
    SetDocumentState(PipelineId, DocumentState),
    /// Update the pipeline Url, which can change after redirections.
    SetFinalUrl(PipelineId, Url),
}

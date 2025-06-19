/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! This module contains traits in script used generically in the rest of Servo.
//! The traits are here instead of in script so that these modules won't have
//! to depend on script.

#![deny(missing_docs)]
#![deny(unsafe_code)]

use std::collections::HashMap;
use std::fmt;
use std::sync::Arc;

use background_hang_monitor_api::BackgroundHangMonitorRegister;
use base::cross_process_instant::CrossProcessInstant;
use base::id::{BrowsingContextId, HistoryStateId, PipelineId, PipelineNamespaceId, WebViewId};
#[cfg(feature = "bluetooth")]
use bluetooth_traits::BluetoothRequest;
use canvas_traits::webgl::WebGLPipeline;
use compositing_traits::CrossProcessCompositorApi;
use constellation_traits::{
    LoadData, NavigationHistoryBehavior, ScriptToConstellationChan, StructuredSerializedData,
    WindowSizeType,
};
use crossbeam_channel::{RecvTimeoutError, Sender};
use devtools_traits::ScriptToDevtoolsControlMsg;
use embedder_traits::user_content_manager::UserContentManager;
use embedder_traits::{
    CompositorHitTestResult, FocusSequenceNumber, InputEvent, JavaScriptEvaluationId,
    MediaSessionActionType, Theme, ViewportDetails, WebDriverScriptCommand,
};
use euclid::{Rect, Scale, Size2D, UnknownUnit};
use ipc_channel::ipc::{IpcReceiver, IpcSender};
use keyboard_types::Modifiers;
use malloc_size_of_derive::MallocSizeOf;
use media::WindowGLContext;
use net_traits::ResourceThreads;
use net_traits::image_cache::ImageCache;
use net_traits::storage_thread::StorageType;
use pixels::PixelFormat;
use profile_traits::mem;
use serde::{Deserialize, Serialize};
use servo_url::{ImmutableOrigin, ServoUrl};
use strum_macros::IntoStaticStr;
use style_traits::{CSSPixel, SpeculativePainter};
use stylo_atoms::Atom;
#[cfg(feature = "webgpu")]
use webgpu_traits::WebGPUMsg;
use webrender_api::units::{DevicePixel, LayoutVector2D};
use webrender_api::{ExternalScrollId, ImageKey};

/// The initial data required to create a new layout attached to an existing script thread.
#[derive(Debug, Deserialize, Serialize)]
pub struct NewLayoutInfo {
    /// The ID of the parent pipeline and frame type, if any.
    /// If `None`, this is a root pipeline.
    pub parent_info: Option<PipelineId>,
    /// Id of the newly-created pipeline.
    pub new_pipeline_id: PipelineId,
    /// Id of the browsing context associated with this pipeline.
    pub browsing_context_id: BrowsingContextId,
    /// Id of the top-level browsing context associated with this pipeline.
    pub webview_id: WebViewId,
    /// Id of the opener, if any
    pub opener: Option<BrowsingContextId>,
    /// Network request data which will be initiated by the script thread.
    pub load_data: LoadData,
    /// Initial [`ViewportDetails`] for this layout.
    pub viewport_details: ViewportDetails,
    /// The [`Theme`] of the new layout.
    pub theme: Theme,
}

/// When a pipeline is closed, should its browsing context be discarded too?
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub enum DiscardBrowsingContext {
    /// Discard the browsing context
    Yes,
    /// Don't discard the browsing context
    No,
}

/// Is a document fully active, active or inactive?
/// A document is active if it is the current active document in its session history,
/// it is fuly active if it is active and all of its ancestors are active,
/// and it is inactive otherwise.
///
/// * <https://html.spec.whatwg.org/multipage/#active-document>
/// * <https://html.spec.whatwg.org/multipage/#fully-active>
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, MallocSizeOf, PartialEq, Serialize)]
pub enum DocumentActivity {
    /// An inactive document
    Inactive,
    /// An active but not fully active document
    Active,
    /// A fully active document
    FullyActive,
}

/// Type of recorded progressive web metric
#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub enum ProgressiveWebMetricType {
    /// Time to first Paint
    FirstPaint,
    /// Time to first contentful paint
    FirstContentfulPaint,
    /// Time to interactive
    TimeToInteractive,
}

/// The reason why the pipeline id of an iframe is being updated.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, MallocSizeOf, PartialEq, Serialize)]
pub enum UpdatePipelineIdReason {
    /// The pipeline id is being updated due to a navigation.
    Navigation,
    /// The pipeline id is being updated due to a history traversal.
    Traversal,
}

/// Messages sent to the `ScriptThread` event loop from the `Constellation`, `Compositor`, and (for
/// now) `Layout`.
#[derive(Deserialize, IntoStaticStr, Serialize)]
pub enum ScriptThreadMessage {
    /// Takes the associated window proxy out of "delaying-load-events-mode",
    /// used if a scheduled navigated was refused by the embedder.
    /// <https://html.spec.whatwg.org/multipage/#delaying-load-events-mode>
    StopDelayingLoadEventsMode(PipelineId),
    /// Gives a channel and ID to a layout, as well as the ID of that layout's parent
    AttachLayout(NewLayoutInfo),
    /// Window resized.  Sends a DOM event eventually, but first we combine events.
    Resize(PipelineId, ViewportDetails, WindowSizeType),
    /// Theme changed.
    ThemeChange(PipelineId, Theme),
    /// Notifies script that window has been resized but to not take immediate action.
    ResizeInactive(PipelineId, ViewportDetails),
    /// Window switched from fullscreen mode.
    ExitFullScreen(PipelineId),
    /// Notifies the script that the document associated with this pipeline should 'unload'.
    UnloadDocument(PipelineId),
    /// Notifies the script that a pipeline should be closed.
    ExitPipeline(WebViewId, PipelineId, DiscardBrowsingContext),
    /// Notifies the script that the whole thread should be closed.
    ExitScriptThread,
    /// Sends a DOM event.
    SendInputEvent(PipelineId, ConstellationInputEvent),
    /// Notifies script of the viewport.
    Viewport(PipelineId, Rect<f32, UnknownUnit>),
    /// Requests that the script thread immediately send the constellation the title of a pipeline.
    GetTitle(PipelineId),
    /// Notifies script thread of a change to one of its document's activity
    SetDocumentActivity(PipelineId, DocumentActivity),
    /// Set whether to use less resources by running timers at a heavily limited rate.
    SetThrottled(PipelineId, bool),
    /// Notify the containing iframe (in PipelineId) that the nested browsing context (BrowsingContextId) is throttled.
    SetThrottledInContainingIframe(PipelineId, BrowsingContextId, bool),
    /// Notifies script thread that a url should be loaded in this iframe.
    /// PipelineId is for the parent, BrowsingContextId is for the nested browsing context
    NavigateIframe(
        PipelineId,
        BrowsingContextId,
        LoadData,
        NavigationHistoryBehavior,
    ),
    /// Post a message to a given window.
    PostMessage {
        /// The target of the message.
        target: PipelineId,
        /// The source of the message.
        source: PipelineId,
        /// The top level browsing context associated with the source pipeline.
        source_browsing_context: WebViewId,
        /// The expected origin of the target.
        target_origin: Option<ImmutableOrigin>,
        /// The source origin of the message.
        /// <https://html.spec.whatwg.org/multipage/#dom-messageevent-origin>
        source_origin: ImmutableOrigin,
        /// The data to be posted.
        data: StructuredSerializedData,
    },
    /// Updates the current pipeline ID of a given iframe.
    /// First PipelineId is for the parent, second is the new PipelineId for the frame.
    UpdatePipelineId(
        PipelineId,
        BrowsingContextId,
        WebViewId,
        PipelineId,
        UpdatePipelineIdReason,
    ),
    /// Updates the history state and url of a given pipeline.
    UpdateHistoryState(PipelineId, Option<HistoryStateId>, ServoUrl),
    /// Removes inaccesible history states.
    RemoveHistoryStates(PipelineId, Vec<HistoryStateId>),
    /// Set an iframe to be focused. Used when an element in an iframe gains focus.
    /// PipelineId is for the parent, BrowsingContextId is for the nested browsing context
    FocusIFrame(PipelineId, BrowsingContextId, FocusSequenceNumber),
    /// Focus the document. Used when the container gains focus.
    FocusDocument(PipelineId, FocusSequenceNumber),
    /// Notifies that the document's container (e.g., an iframe) is not included
    /// in the top-level browsing context's focus chain (not considering system
    /// focus) anymore.
    ///
    /// Obviously, this message is invalid for a top-level document.
    Unfocus(PipelineId, FocusSequenceNumber),
    /// Passes a webdriver command to the script thread for execution
    WebDriverScriptCommand(PipelineId, WebDriverScriptCommand),
    /// Notifies script thread that all animations are done
    TickAllAnimations(Vec<WebViewId>),
    /// Notifies the script thread that a new Web font has been loaded, and thus the page should be
    /// reflowed.
    WebFontLoaded(PipelineId, bool /* success */),
    /// Cause a `load` event to be dispatched at the appropriate iframe element.
    DispatchIFrameLoadEvent {
        /// The frame that has been marked as loaded.
        target: BrowsingContextId,
        /// The pipeline that contains a frame loading the target pipeline.
        parent: PipelineId,
        /// The pipeline that has completed loading.
        child: PipelineId,
    },
    /// Cause a `storage` event to be dispatched at the appropriate window.
    /// The strings are key, old value and new value.
    DispatchStorageEvent(
        PipelineId,
        StorageType,
        ServoUrl,
        Option<String>,
        Option<String>,
        Option<String>,
    ),
    /// Report an error from a CSS parser for the given pipeline
    ReportCSSError(PipelineId, String, u32, u32, String),
    /// Reload the given page.
    Reload(PipelineId),
    /// Notifies the script thread about a new recorded paint metric.
    PaintMetric(
        PipelineId,
        ProgressiveWebMetricType,
        CrossProcessInstant,
        bool, /* first_reflow */
    ),
    /// Notifies the media session about a user requested media session action.
    MediaSessionAction(PipelineId, MediaSessionActionType),
    /// Notifies script thread that WebGPU server has started
    #[cfg(feature = "webgpu")]
    SetWebGPUPort(IpcReceiver<WebGPUMsg>),
    /// The compositor scrolled and is updating the scroll states of the nodes in the given
    /// pipeline via the Constellation.
    SetScrollStates(PipelineId, HashMap<ExternalScrollId, LayoutVector2D>),
    /// Evaluate the given JavaScript and return a result via a corresponding message
    /// to the Constellation.
    EvaluateJavaScript(PipelineId, JavaScriptEvaluationId, String),
}

impl fmt::Debug for ScriptThreadMessage {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        let variant_string: &'static str = self.into();
        write!(formatter, "ConstellationControlMsg::{variant_string}")
    }
}

/// Used to determine if a script has any pending asynchronous activity.
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
pub enum DocumentState {
    /// The document has been loaded and is idle.
    Idle,
    /// The document is either loading or waiting on an event.
    Pending,
}

/// Input events from the embedder that are sent via the `Constellation`` to the `ScriptThread`.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ConstellationInputEvent {
    /// The hit test result of this input event, if any.
    pub hit_test_result: Option<CompositorHitTestResult>,
    /// The pressed mouse button state of the constellation when this input
    /// event was triggered.
    pub pressed_mouse_buttons: u16,
    /// The currently active keyboard modifiers.
    pub active_keyboard_modifiers: Modifiers,
    /// The [`InputEvent`] itself.
    pub event: InputEvent,
}

/// Data needed to construct a script thread.
///
/// NB: *DO NOT* add any Senders or Receivers here! pcwalton will have to rewrite your code if you
/// do! Use IPC senders and receivers instead.
pub struct InitialScriptState {
    /// The ID of the pipeline with which this script thread is associated.
    pub id: PipelineId,
    /// The subpage ID of this pipeline to create in its pipeline parent.
    /// If `None`, this is the root.
    pub parent_info: Option<PipelineId>,
    /// The ID of the browsing context this script is part of.
    pub browsing_context_id: BrowsingContextId,
    /// The ID of the top-level browsing context this script is part of.
    pub webview_id: WebViewId,
    /// The ID of the opener, if any.
    pub opener: Option<BrowsingContextId>,
    /// Loading into a Secure Context
    pub inherited_secure_context: Option<bool>,
    /// A channel with which messages can be sent to us (the script thread).
    pub constellation_sender: IpcSender<ScriptThreadMessage>,
    /// A port on which messages sent by the constellation to script can be received.
    pub constellation_receiver: IpcReceiver<ScriptThreadMessage>,
    /// A channel on which messages can be sent to the constellation from script.
    pub pipeline_to_constellation_sender: ScriptToConstellationChan,
    /// A handle to register script-(and associated layout-)threads for hang monitoring.
    pub background_hang_monitor_register: Box<dyn BackgroundHangMonitorRegister>,
    /// A channel to the resource manager thread.
    pub resource_threads: ResourceThreads,
    /// A channel to the bluetooth thread.
    #[cfg(feature = "bluetooth")]
    pub bluetooth_sender: IpcSender<BluetoothRequest>,
    /// The image cache for this script thread.
    pub image_cache: Arc<dyn ImageCache>,
    /// A channel to the time profiler thread.
    pub time_profiler_sender: profile_traits::time::ProfilerChan,
    /// A channel to the memory profiler thread.
    pub memory_profiler_sender: mem::ProfilerChan,
    /// A channel to the developer tools, if applicable.
    pub devtools_server_sender: Option<IpcSender<ScriptToDevtoolsControlMsg>>,
    /// Initial [`ViewportDetails`] for the frame that is initiating this `ScriptThread`.
    pub viewport_details: ViewportDetails,
    /// Initial [`Theme`] for the frame that is initiating this `ScriptThread`.
    pub theme: Theme,
    /// The ID of the pipeline namespace for this script thread.
    pub pipeline_namespace_id: PipelineNamespaceId,
    /// A ping will be sent on this channel once the script thread shuts down.
    pub content_process_shutdown_sender: Sender<()>,
    /// A channel to the WebGL thread used in this pipeline.
    pub webgl_chan: Option<WebGLPipeline>,
    /// The XR device registry
    pub webxr_registry: Option<webxr_api::Registry>,
    /// Access to the compositor across a process boundary.
    pub compositor_api: CrossProcessCompositorApi,
    /// Application window's GL Context for Media player
    pub player_context: WindowGLContext,
    /// User content manager
    pub user_content_manager: UserContentManager,
}

/// Errors from executing a paint worklet
#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum PaintWorkletError {
    /// Execution timed out.
    Timeout,
    /// No such worklet.
    WorkletNotFound,
}

impl From<RecvTimeoutError> for PaintWorkletError {
    fn from(_: RecvTimeoutError) -> PaintWorkletError {
        PaintWorkletError::Timeout
    }
}

/// Execute paint code in the worklet thread pool.
pub trait Painter: SpeculativePainter {
    /// <https://drafts.css-houdini.org/css-paint-api/#draw-a-paint-image>
    fn draw_a_paint_image(
        &self,
        size: Size2D<f32, CSSPixel>,
        zoom: Scale<f32, CSSPixel, DevicePixel>,
        properties: Vec<(Atom, String)>,
        arguments: Vec<String>,
    ) -> Result<DrawAPaintImageResult, PaintWorkletError>;
}

impl fmt::Debug for dyn Painter {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.debug_tuple("Painter")
            .field(&format_args!(".."))
            .finish()
    }
}

/// The result of executing paint code: the image together with any image URLs that need to be loaded.
///
/// TODO: this should return a WR display list. <https://github.com/servo/servo/issues/17497>
#[derive(Clone, Debug, Deserialize, MallocSizeOf, Serialize)]
pub struct DrawAPaintImageResult {
    /// The image height
    pub width: u32,
    /// The image width
    pub height: u32,
    /// The image format
    pub format: PixelFormat,
    /// The image drawn, or None if an invalid paint image was drawn
    pub image_key: Option<ImageKey>,
    /// Drawing the image might have requested loading some image URLs.
    pub missing_image_urls: Vec<ServoUrl>,
}

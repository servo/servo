/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! This module contains traits in script used generically in the rest of Servo.
//! The traits are here instead of in script so that these modules won't have
//! to depend on script.

#![deny(missing_docs)]
#![deny(unsafe_code)]

mod script_msg;
pub mod serializable;
pub mod transferable;

use std::borrow::Cow;
use std::collections::{HashMap, VecDeque};
use std::fmt;
use std::sync::Arc;

use background_hang_monitor_api::BackgroundHangMonitorRegister;
use base::Epoch;
use base::cross_process_instant::CrossProcessInstant;
use base::id::{
    BlobId, BrowsingContextId, HistoryStateId, MessagePortId, PipelineId, PipelineNamespaceId,
    WebViewId,
};
use bitflags::bitflags;
#[cfg(feature = "bluetooth")]
use bluetooth_traits::BluetoothRequest;
use canvas_traits::webgl::WebGLPipeline;
use crossbeam_channel::{RecvTimeoutError, Sender};
use devtools_traits::{DevtoolScriptControlMsg, ScriptToDevtoolsControlMsg, WorkerId};
use embedder_traits::input_events::InputEvent;
use embedder_traits::{MediaSessionActionType, Theme, WebDriverScriptCommand};
use euclid::{Rect, Scale, Size2D, UnknownUnit, Vector2D};
use http::{HeaderMap, Method};
use ipc_channel::Error as IpcError;
use ipc_channel::ipc::{IpcReceiver, IpcSender};
use libc::c_void;
use log::warn;
use malloc_size_of::malloc_size_of_is_0;
use malloc_size_of_derive::MallocSizeOf;
use media::WindowGLContext;
use net_traits::image_cache::ImageCache;
use net_traits::request::{InsecureRequestsPolicy, Referrer, RequestBody};
use net_traits::storage_thread::StorageType;
use net_traits::{ReferrerPolicy, ResourceThreads};
use pixels::PixelFormat;
use profile_traits::{mem, time as profile_time};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use servo_url::{ImmutableOrigin, ServoUrl};
use strum_macros::IntoStaticStr;
use style_traits::{CSSPixel, SpeculativePainter};
use stylo_atoms::Atom;
#[cfg(feature = "webgpu")]
use webgpu::WebGPUMsg;
use webrender_api::units::{DevicePixel, LayoutPixel};
use webrender_api::{DocumentId, ExternalScrollId, ImageKey};
use webrender_traits::{
    CompositorHitTestResult, CrossProcessCompositorApi,
    UntrustedNodeAddress as WebRenderUntrustedNodeAddress,
};

pub use crate::script_msg::{
    DOMMessage, IFrameSizeMsg, Job, JobError, JobResult, JobResultValue, JobType, LayoutMsg,
    LogEntry, SWManagerMsg, SWManagerSenders, ScopeThings, ScriptMsg, ServiceWorkerMsg,
    TouchEventResult,
};
use crate::serializable::{BlobData, BlobImpl};
use crate::transferable::MessagePortImpl;

/// The address of a node. Layout sends these back. They must be validated via
/// `from_untrusted_node_address` before they can be used, because we do not trust layout.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct UntrustedNodeAddress(pub *const c_void);

malloc_size_of_is_0!(UntrustedNodeAddress);

#[allow(unsafe_code)]
unsafe impl Send for UntrustedNodeAddress {}

impl From<WebRenderUntrustedNodeAddress> for UntrustedNodeAddress {
    fn from(o: WebRenderUntrustedNodeAddress) -> Self {
        UntrustedNodeAddress(o.0)
    }
}

impl From<style_traits::dom::OpaqueNode> for UntrustedNodeAddress {
    fn from(o: style_traits::dom::OpaqueNode) -> Self {
        UntrustedNodeAddress(o.0 as *const c_void)
    }
}

impl Serialize for UntrustedNodeAddress {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        (self.0 as usize).serialize(s)
    }
}

impl<'de> Deserialize<'de> for UntrustedNodeAddress {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<UntrustedNodeAddress, D::Error> {
        let value: usize = Deserialize::deserialize(d)?;
        Ok(UntrustedNodeAddress::from_id(value))
    }
}

impl UntrustedNodeAddress {
    /// Creates an `UntrustedNodeAddress` from the given pointer address value.
    #[inline]
    pub fn from_id(id: usize) -> UntrustedNodeAddress {
        UntrustedNodeAddress(id as *const c_void)
    }
}

/// The origin where a given load was initiated.
/// Useful for origin checks, for example before evaluation a JS URL.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum LoadOrigin {
    /// A load originating in the constellation.
    Constellation,
    /// A load originating in webdriver.
    WebDriver,
    /// A load originating in script.
    Script(ImmutableOrigin),
}

/// can be passed to `LoadUrl` to load a page with GET/POST
/// parameters or headers
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct LoadData {
    /// The origin where the load started.
    pub load_origin: LoadOrigin,
    /// The URL.
    pub url: ServoUrl,
    /// The creator pipeline id if this is an about:blank load.
    pub creator_pipeline_id: Option<PipelineId>,
    /// The method.
    #[serde(
        deserialize_with = "::hyper_serde::deserialize",
        serialize_with = "::hyper_serde::serialize"
    )]
    pub method: Method,
    /// The headers.
    #[serde(
        deserialize_with = "::hyper_serde::deserialize",
        serialize_with = "::hyper_serde::serialize"
    )]
    pub headers: HeaderMap,
    /// The data that will be used as the body of the request.
    pub data: Option<RequestBody>,
    /// The result of evaluating a javascript scheme url.
    pub js_eval_result: Option<JsEvalResult>,
    /// The referrer.
    pub referrer: Referrer,
    /// The referrer policy.
    pub referrer_policy: ReferrerPolicy,

    /// The source to use instead of a network response for a srcdoc document.
    pub srcdoc: String,
    /// The inherited context is Secure, None if not inherited
    pub inherited_secure_context: Option<bool>,
    /// The inherited policy for upgrading insecure requests; None if not inherited.
    pub inherited_insecure_requests_policy: Option<InsecureRequestsPolicy>,

    /// Servo internal: if crash details are present, trigger a crash error page with these details.
    pub crash: Option<String>,
}

/// The result of evaluating a javascript scheme url.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum JsEvalResult {
    /// The js evaluation had a non-string result, 204 status code.
    /// <https://html.spec.whatwg.org/multipage/#navigate> 12.11
    NoContent,
    /// The js evaluation had a string result.
    Ok(Vec<u8>),
}

impl LoadData {
    /// Create a new `LoadData` object.
    pub fn new(
        load_origin: LoadOrigin,
        url: ServoUrl,
        creator_pipeline_id: Option<PipelineId>,
        referrer: Referrer,
        referrer_policy: ReferrerPolicy,
        inherited_secure_context: Option<bool>,
        inherited_insecure_requests_policy: Option<InsecureRequestsPolicy>,
    ) -> LoadData {
        LoadData {
            load_origin,
            url,
            creator_pipeline_id,
            method: Method::GET,
            headers: HeaderMap::new(),
            data: None,
            js_eval_result: None,
            referrer,
            referrer_policy,
            srcdoc: "".to_string(),
            inherited_secure_context,
            crash: None,
            inherited_insecure_requests_policy,
        }
    }
}

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
    /// Information about the initial window size.
    pub window_size: WindowSizeData,
}

/// When a pipeline is closed, should its browsing context be discarded too?
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub enum DiscardBrowsingContext {
    /// Discard the browsing context
    Yes,
    /// Don't discard the browsing context
    No,
}

/// <https://html.spec.whatwg.org/multipage/#navigation-supporting-concepts:navigationhistorybehavior>
#[derive(Debug, Default, Deserialize, PartialEq, Serialize)]
pub enum NavigationHistoryBehavior {
    /// The default value, which will be converted very early in the navigate algorithm into "push"
    /// or "replace". Usually it becomes "push", but under certain circumstances it becomes
    /// "replace" instead.
    #[default]
    Auto,
    /// A regular navigation which adds a new session history entry, and will clear the forward
    /// session history.
    Push,
    /// A navigation that will replace the active session history entry.
    Replace,
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
    Resize(PipelineId, WindowSizeData, WindowSizeType),
    /// Theme changed.
    ThemeChange(PipelineId, Theme),
    /// Notifies script that window has been resized but to not take immediate action.
    ResizeInactive(PipelineId, WindowSizeData),
    /// Window switched from fullscreen mode.
    ExitFullScreen(PipelineId),
    /// Notifies the script that the document associated with this pipeline should 'unload'.
    UnloadDocument(PipelineId),
    /// Notifies the script that a pipeline should be closed.
    ExitPipeline(PipelineId, DiscardBrowsingContext),
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
    FocusIFrame(PipelineId, BrowsingContextId),
    /// Passes a webdriver command to the script thread for execution
    WebDriverScriptCommand(PipelineId, WebDriverScriptCommand),
    /// Notifies script thread that all animations are done
    TickAllAnimations(PipelineId, AnimationTickType),
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
    PaintMetric(PipelineId, ProgressiveWebMetricType, CrossProcessInstant),
    /// Notifies the media session about a user requested media session action.
    MediaSessionAction(PipelineId, MediaSessionActionType),
    /// Notifies script thread that WebGPU server has started
    #[cfg(feature = "webgpu")]
    SetWebGPUPort(IpcReceiver<WebGPUMsg>),
    /// The compositor scrolled and is updating the scroll states of the nodes in the given
    /// pipeline via the Constellation.
    SetScrollStates(PipelineId, Vec<ScrollState>),
    /// Send the paint time for a specific epoch.
    SetEpochPaintTime(PipelineId, Epoch, CrossProcessInstant),
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

/// For a given pipeline, whether any animations are currently running
/// and any animation callbacks are queued
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum AnimationState {
    /// Animations are active but no callbacks are queued
    AnimationsPresent,
    /// Animations are active and callbacks are queued
    AnimationCallbacksPresent,
    /// No animations are active and no callbacks are queued
    NoAnimationsPresent,
    /// No animations are active but callbacks are queued
    NoAnimationCallbacksPresent,
}

/// Input events from the embedder that are sent via the `Constellation`` to the `ScriptThread`.
#[derive(Debug, Deserialize, Serialize)]
pub struct ConstellationInputEvent {
    /// The hit test result of this input event, if any.
    pub hit_test_result: Option<CompositorHitTestResult>,
    /// The pressed mouse button state of the constellation when this input
    /// event was triggered.
    pub pressed_mouse_buttons: u16,
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
    /// A sender layout to communicate to the constellation.
    pub layout_to_constellation_ipc_sender: IpcSender<LayoutMsg>,
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
    /// Information about the initial window size.
    pub window_size: WindowSizeData,
    /// The ID of the pipeline namespace for this script thread.
    pub pipeline_namespace_id: PipelineNamespaceId,
    /// A ping will be sent on this channel once the script thread shuts down.
    pub content_process_shutdown_sender: Sender<()>,
    /// A channel to the WebGL thread used in this pipeline.
    pub webgl_chan: Option<WebGLPipeline>,
    /// The XR device registry
    pub webxr_registry: Option<webxr_api::Registry>,
    /// The Webrender document ID associated with this thread.
    pub webrender_document: DocumentId,
    /// Access to the compositor across a process boundary.
    pub compositor_api: CrossProcessCompositorApi,
    /// Application window's GL Context for Media player
    pub player_context: WindowGLContext,
}

/// This trait allows creating a `ServiceWorkerManager` without depending on the `script`
/// crate.
pub trait ServiceWorkerManagerFactory {
    /// Create a `ServiceWorkerManager`.
    fn create(sw_senders: SWManagerSenders, origin: ImmutableOrigin);
}

/// Whether the sandbox attribute is present for an iframe element
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum IFrameSandboxState {
    /// Sandbox attribute is present
    IFrameSandboxed,
    /// Sandbox attribute is not present
    IFrameUnsandboxed,
}

/// Specifies the information required to load an auxiliary browsing context.
#[derive(Debug, Deserialize, Serialize)]
pub struct AuxiliaryWebViewCreationRequest {
    /// Load data containing the url to load
    pub load_data: LoadData,
    /// The webview that caused this request.
    pub opener_webview_id: WebViewId,
    /// The pipeline opener browsing context.
    pub opener_pipeline_id: PipelineId,
    /// Sender for the constellation’s response to our request.
    pub response_sender: IpcSender<Option<AuxiliaryWebViewCreationResponse>>,
}

/// Constellation’s response to auxiliary browsing context creation requests.
#[derive(Debug, Deserialize, Serialize)]
pub struct AuxiliaryWebViewCreationResponse {
    /// The new webview ID.
    pub new_webview_id: WebViewId,
    /// The new pipeline ID.
    pub new_pipeline_id: PipelineId,
}

/// Specifies the information required to load an iframe.
#[derive(Debug, Deserialize, Serialize)]
pub struct IFrameLoadInfo {
    /// Pipeline ID of the parent of this iframe
    pub parent_pipeline_id: PipelineId,
    /// The ID for this iframe's nested browsing context.
    pub browsing_context_id: BrowsingContextId,
    /// The ID for the top-level ancestor browsing context of this iframe's nested browsing context.
    pub webview_id: WebViewId,
    /// The new pipeline ID that the iframe has generated.
    pub new_pipeline_id: PipelineId,
    ///  Whether this iframe should be considered private
    pub is_private: bool,
    ///  Whether this iframe should be considered secure
    pub inherited_secure_context: Option<bool>,
    /// Whether this load should replace the current entry (reload). If true, the current
    /// entry will be replaced instead of a new entry being added.
    pub history_handling: NavigationHistoryBehavior,
}

/// Specifies the information required to load a URL in an iframe.
#[derive(Debug, Deserialize, Serialize)]
pub struct IFrameLoadInfoWithData {
    /// The information required to load an iframe.
    pub info: IFrameLoadInfo,
    /// Load data containing the url to load
    pub load_data: LoadData,
    /// The old pipeline ID for this iframe, if a page was previously loaded.
    pub old_pipeline_id: Option<PipelineId>,
    /// Sandbox type of this iframe
    pub sandbox: IFrameSandboxState,
    /// The initial viewport size for this iframe.
    pub window_size: WindowSizeData,
}

bitflags! {
    #[derive(Debug, Default, Deserialize, Serialize)]
    /// Specifies if rAF should be triggered and/or CSS Animations and Transitions.
    pub struct AnimationTickType: u8 {
        /// Trigger a call to requestAnimationFrame.
        const REQUEST_ANIMATION_FRAME = 0b001;
        /// Trigger restyles for CSS Animations and Transitions.
        const CSS_ANIMATIONS_AND_TRANSITIONS = 0b010;
    }
}

/// The scroll state of a stacking context.
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
pub struct ScrollState {
    /// The ID of the scroll root.
    pub scroll_id: ExternalScrollId,
    /// The scrolling offset of this stacking context.
    pub scroll_offset: Vector2D<f32, LayoutPixel>,
}

/// Data about the window size.
#[derive(Clone, Copy, Debug, Deserialize, MallocSizeOf, PartialEq, Serialize)]
pub struct WindowSizeData {
    /// The size of the initial layout viewport, before parsing an
    /// <http://www.w3.org/TR/css-device-adapt/#initial-viewport>
    pub initial_viewport: Size2D<f32, CSSPixel>,

    /// The resolution of the window in dppx, not including any "pinch zoom" factor.
    pub device_pixel_ratio: Scale<f32, CSSPixel, DevicePixel>,
}

/// The type of window size change.
#[derive(Clone, Copy, Debug, Deserialize, Eq, MallocSizeOf, PartialEq, Serialize)]
pub enum WindowSizeType {
    /// Initial load.
    Initial,
    /// Window resize.
    Resize,
}

/// Resources required by workerglobalscopes
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct WorkerGlobalScopeInit {
    /// Chan to a resource thread
    pub resource_threads: ResourceThreads,
    /// Chan to the memory profiler
    pub mem_profiler_chan: mem::ProfilerChan,
    /// Chan to the time profiler
    pub time_profiler_chan: profile_time::ProfilerChan,
    /// To devtools sender
    pub to_devtools_sender: Option<IpcSender<ScriptToDevtoolsControlMsg>>,
    /// From devtools sender
    pub from_devtools_sender: Option<IpcSender<DevtoolScriptControlMsg>>,
    /// Messages to send to constellation
    pub script_to_constellation_chan: ScriptToConstellationChan,
    /// The worker id
    pub worker_id: WorkerId,
    /// The pipeline id
    pub pipeline_id: PipelineId,
    /// The origin
    pub origin: ImmutableOrigin,
    /// The creation URL
    pub creation_url: Option<ServoUrl>,
    /// An optional string allowing the user agnet to be set for testing.
    pub user_agent: Cow<'static, str>,
    /// True if secure context
    pub inherited_secure_context: Option<bool>,
}

/// Common entities representing a network load origin
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct WorkerScriptLoadOrigin {
    /// referrer url
    pub referrer_url: Option<ServoUrl>,
    /// the referrer policy which is used
    pub referrer_policy: ReferrerPolicy,
    /// the pipeline id of the entity requesting the load
    pub pipeline_id: PipelineId,
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

/// A Script to Constellation channel.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ScriptToConstellationChan {
    /// Sender for communicating with constellation thread.
    pub sender: IpcSender<(PipelineId, ScriptMsg)>,
    /// Used to identify the origin of the message.
    pub pipeline_id: PipelineId,
}

impl ScriptToConstellationChan {
    /// Send ScriptMsg and attach the pipeline_id to the message.
    pub fn send(&self, msg: ScriptMsg) -> Result<(), IpcError> {
        self.sender.send((self.pipeline_id, msg))
    }
}

/// A data-holder for serialized data and transferred objects.
/// <https://html.spec.whatwg.org/multipage/#structuredserializewithtransfer>
#[derive(Debug, Deserialize, MallocSizeOf, Serialize)]
pub struct StructuredSerializedData {
    /// Data serialized by SpiderMonkey.
    pub serialized: Vec<u8>,
    /// Serialized in a structured callback,
    pub blobs: Option<HashMap<BlobId, BlobImpl>>,
    /// Transferred objects.
    pub ports: Option<HashMap<MessagePortId, MessagePortImpl>>,
}

impl StructuredSerializedData {
    /// Clone the serialized data for use with broadcast-channels.
    pub fn clone_for_broadcast(&self) -> StructuredSerializedData {
        let serialized = self.serialized.clone();

        let blobs = if let Some(blobs) = self.blobs.as_ref() {
            let mut blob_clones = HashMap::with_capacity(blobs.len());

            for (original_id, blob) in blobs.iter() {
                let type_string = blob.type_string();

                if let BlobData::Memory(bytes) = blob.blob_data() {
                    let blob_clone = BlobImpl::new_from_bytes(bytes.clone(), type_string);

                    // Note: we insert the blob at the original id,
                    // otherwise this will not match the storage key as serialized by SM in `serialized`.
                    // The clone has it's own new Id however.
                    blob_clones.insert(*original_id, blob_clone);
                } else {
                    // Not panicking only because this is called from the constellation.
                    warn!("Serialized blob not in memory format(should never happen).");
                }
            }
            Some(blob_clones)
        } else {
            None
        };

        if self.ports.is_some() {
            // Not panicking only because this is called from the constellation.
            warn!(
                "Attempt to broadcast structured serialized data including ports(should never happen)."
            );
        }

        StructuredSerializedData {
            serialized,
            blobs,
            // Ports cannot be broadcast.
            ports: None,
        }
    }
}

/// A task on the <https://html.spec.whatwg.org/multipage/#port-message-queue>
#[derive(Debug, Deserialize, MallocSizeOf, Serialize)]
pub struct PortMessageTask {
    /// The origin of this task.
    pub origin: ImmutableOrigin,
    /// A data-holder for serialized data and transferred objects.
    pub data: StructuredSerializedData,
}

/// Messages for communication between the constellation and a global managing ports.
#[derive(Debug, Deserialize, Serialize)]
pub enum MessagePortMsg {
    /// Complete the transfer for a batch of ports.
    CompleteTransfer(HashMap<MessagePortId, VecDeque<PortMessageTask>>),
    /// Complete the transfer of a single port,
    /// whose transfer was pending because it had been requested
    /// while a previous failed transfer was being rolled-back.
    CompletePendingTransfer(MessagePortId, VecDeque<PortMessageTask>),
    /// Remove a port, the entangled one doesn't exists anymore.
    RemoveMessagePort(MessagePortId),
    /// Handle a new port-message-task.
    NewTask(MessagePortId, PortMessageTask),
}

/// Message for communication between the constellation and a global managing broadcast channels.
#[derive(Debug, Deserialize, Serialize)]
pub struct BroadcastMsg {
    /// The origin of this message.
    pub origin: ImmutableOrigin,
    /// The name of the channel.
    pub channel_name: String,
    /// A data-holder for serialized data.
    pub data: StructuredSerializedData,
}

impl Clone for BroadcastMsg {
    fn clone(&self) -> BroadcastMsg {
        BroadcastMsg {
            data: self.data.clone_for_broadcast(),
            origin: self.origin.clone(),
            channel_name: self.channel_name.clone(),
        }
    }
}

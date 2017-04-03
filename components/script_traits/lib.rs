/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! This module contains traits in script used generically in the rest of Servo.
//! The traits are here instead of in script so that these modules won't have
//! to depend on script.

#![deny(missing_docs)]
#![deny(unsafe_code)]

extern crate app_units;
extern crate bluetooth_traits;
extern crate canvas_traits;
extern crate cookie as cookie_rs;
extern crate devtools_traits;
extern crate euclid;
extern crate gfx_traits;
extern crate heapsize;
#[macro_use]
extern crate heapsize_derive;
extern crate hyper;
extern crate hyper_serde;
extern crate ipc_channel;
extern crate libc;
extern crate msg;
extern crate net_traits;
extern crate offscreen_gl_context;
extern crate profile_traits;
extern crate rustc_serialize;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate servo_atoms;
extern crate servo_url;
extern crate style_traits;
extern crate time;
extern crate webrender_traits;
extern crate webvr_traits;

mod script_msg;
pub mod webdriver_msg;

use app_units::Au;
use bluetooth_traits::BluetoothRequest;
use devtools_traits::{DevtoolScriptControlMsg, ScriptToDevtoolsControlMsg, WorkerId};
use euclid::Size2D;
use euclid::length::Length;
use euclid::point::Point2D;
use euclid::rect::Rect;
use euclid::scale_factor::ScaleFactor;
use euclid::size::TypedSize2D;
use gfx_traits::Epoch;
use heapsize::HeapSizeOf;
use hyper::header::Headers;
use hyper::method::Method;
use ipc_channel::ipc::{IpcReceiver, IpcSender};
use libc::c_void;
use msg::constellation_msg::{BrowsingContextId, TopLevelBrowsingContextId, FrameType, Key, KeyModifiers, KeyState};
use msg::constellation_msg::{PipelineId, PipelineNamespaceId, TraversalDirection};
use net_traits::{FetchResponseMsg, ReferrerPolicy, ResourceThreads};
use net_traits::image::base::Image;
use net_traits::image_cache::ImageCache;
use net_traits::response::HttpsState;
use net_traits::storage_thread::StorageType;
use profile_traits::mem;
use profile_traits::time as profile_time;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use servo_atoms::Atom;
use servo_url::ImmutableOrigin;
use servo_url::ServoUrl;
use std::collections::HashMap;
use std::fmt;
use std::sync::Arc;
use std::sync::mpsc::{Receiver, Sender, RecvTimeoutError};
use style_traits::CSSPixel;
use webdriver_msg::{LoadStatus, WebDriverScriptCommand};
use webrender_traits::ClipId;
use webvr_traits::{WebVREvent, WebVRMsg};

pub use script_msg::{LayoutMsg, ScriptMsg, EventResult, LogEntry};
pub use script_msg::{ServiceWorkerMsg, ScopeThings, SWManagerMsg, SWManagerSenders, DOMMessage};

/// The address of a node. Layout sends these back. They must be validated via
/// `from_untrusted_node_address` before they can be used, because we do not trust layout.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct UntrustedNodeAddress(pub *const c_void);

impl HeapSizeOf for UntrustedNodeAddress {
    fn heap_size_of_children(&self) -> usize {
        0
    }
}

#[allow(unsafe_code)]
unsafe impl Send for UntrustedNodeAddress {}

impl Serialize for UntrustedNodeAddress {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        (self.0 as usize).serialize(s)
    }
}

impl Deserialize for UntrustedNodeAddress {
    fn deserialize<D: Deserializer>(d: D) -> Result<UntrustedNodeAddress, D::Error> {
        let value: usize = try!(Deserialize::deserialize(d));
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

/// Messages sent to the layout thread from the constellation and/or compositor.
#[derive(Deserialize, Serialize)]
pub enum LayoutControlMsg {
    /// Requests that this layout thread exit.
    ExitNow,
    /// Requests the current epoch (layout counter) from this layout.
    GetCurrentEpoch(IpcSender<Epoch>),
    /// Asks layout to run another step in its animation.
    TickAnimations,
    /// Tells layout about the new scrolling offsets of each scrollable stacking context.
    SetScrollStates(Vec<ScrollState>),
    /// Requests the current load state of Web fonts. `true` is returned if fonts are still loading
    /// and `false` is returned if all fonts have loaded.
    GetWebFontLoadState(IpcSender<bool>),
}

/// can be passed to `LoadUrl` to load a page with GET/POST
/// parameters or headers
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct LoadData {
    /// The URL.
    pub url: ServoUrl,
    /// The creator pipeline id if this is an about:blank load.
    pub creator_pipeline_id: Option<PipelineId>,
    /// The method.
    #[serde(deserialize_with = "::hyper_serde::deserialize",
            serialize_with = "::hyper_serde::serialize")]
    pub method: Method,
    /// The headers.
    #[serde(deserialize_with = "::hyper_serde::deserialize",
            serialize_with = "::hyper_serde::serialize")]
    pub headers: Headers,
    /// The data.
    pub data: Option<Vec<u8>>,
    /// The referrer policy.
    pub referrer_policy: Option<ReferrerPolicy>,
    /// The referrer URL.
    pub referrer_url: Option<ServoUrl>,
}

impl LoadData {
    /// Create a new `LoadData` object.
    pub fn new(url: ServoUrl,
               creator_pipeline_id: Option<PipelineId>,
               referrer_policy: Option<ReferrerPolicy>,
               referrer_url: Option<ServoUrl>)
               -> LoadData {
        LoadData {
            url: url,
            creator_pipeline_id: creator_pipeline_id,
            method: Method::Get,
            headers: Headers::new(),
            data: None,
            referrer_policy: referrer_policy,
            referrer_url: referrer_url,
        }
    }
}

/// The initial data required to create a new layout attached to an existing script thread.
#[derive(Deserialize, Serialize)]
pub struct NewLayoutInfo {
    /// The ID of the parent pipeline and frame type, if any.
    /// If `None`, this is a root pipeline.
    pub parent_info: Option<(PipelineId, FrameType)>,
    /// Id of the newly-created pipeline.
    pub new_pipeline_id: PipelineId,
    /// Id of the browsing context associated with this pipeline.
    pub browsing_context_id: BrowsingContextId,
    /// Id of the top-level browsing context associated with this pipeline.
    pub top_level_browsing_context_id: TopLevelBrowsingContextId,
    /// Network request data which will be initiated by the script thread.
    pub load_data: LoadData,
    /// Information about the initial window size.
    pub window_size: Option<WindowSizeData>,
    /// A port on which layout can receive messages from the pipeline.
    pub pipeline_port: IpcReceiver<LayoutControlMsg>,
    /// A shutdown channel so that layout can tell the content process to shut down when it's done.
    pub content_process_shutdown_chan: Option<IpcSender<()>>,
    /// Number of threads to use for layout.
    pub layout_threads: usize,
}

/// When a pipeline is closed, should its browsing context be discarded too?
#[derive(Copy, Clone, PartialEq, Eq, Hash, Deserialize, Serialize)]
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
/// https://html.spec.whatwg.org/multipage/#active-document
/// https://html.spec.whatwg.org/multipage/#fully-active
#[derive(Copy, Clone, PartialEq, Eq, Hash, HeapSizeOf, Debug, Deserialize, Serialize)]
pub enum DocumentActivity {
    /// An inactive document
    Inactive,
    /// An active but not fully active document
    Active,
    /// A fully active document
    FullyActive,
}

/// The reason why the pipeline id of an iframe is being updated.
#[derive(Copy, Clone, PartialEq, Eq, Hash, HeapSizeOf, Debug, Deserialize, Serialize)]
pub enum UpdatePipelineIdReason {
    /// The pipeline id is being updated due to a navigation.
    Navigation,
    /// The pipeline id is being updated due to a history traversal.
    Traversal,
}

/// Messages sent from the constellation or layout to the script thread.
#[derive(Deserialize, Serialize)]
pub enum ConstellationControlMsg {
    /// Sends the final response to script thread for fetching after all redirections
    /// have been resolved
    NavigationResponse(PipelineId, FetchResponseMsg),
    /// Gives a channel and ID to a layout thread, as well as the ID of that layout's parent
    AttachLayout(NewLayoutInfo),
    /// Window resized.  Sends a DOM event eventually, but first we combine events.
    Resize(PipelineId, WindowSizeData, WindowSizeType),
    /// Notifies script that window has been resized but to not take immediate action.
    ResizeInactive(PipelineId, WindowSizeData),
    /// Notifies the script that a pipeline should be closed.
    ExitPipeline(PipelineId, DiscardBrowsingContext),
    /// Notifies the script that the whole thread should be closed.
    ExitScriptThread,
    /// Sends a DOM event.
    SendEvent(PipelineId, CompositorEvent),
    /// Notifies script of the viewport.
    Viewport(PipelineId, Rect<f32>),
    /// Notifies script of a new set of scroll offsets.
    SetScrollState(PipelineId, Vec<(UntrustedNodeAddress, Point2D<f32>)>),
    /// Requests that the script thread immediately send the constellation the title of a pipeline.
    GetTitle(PipelineId),
    /// Notifies script thread of a change to one of its document's activity
    SetDocumentActivity(PipelineId, DocumentActivity),
    /// Notifies script thread whether frame is visible
    ChangeFrameVisibilityStatus(PipelineId, bool),
    /// Notifies script thread that frame visibility change is complete
    /// PipelineId is for the parent, BrowsingContextId is for the nested browsing context
    NotifyVisibilityChange(PipelineId, BrowsingContextId, bool),
    /// Notifies script thread that a url should be loaded in this iframe.
    /// PipelineId is for the parent, BrowsingContextId is for the nested browsing context
    Navigate(PipelineId, BrowsingContextId, LoadData, bool),
    /// Post a message to a given window.
    PostMessage(PipelineId, Option<ImmutableOrigin>, Vec<u8>),
    /// Requests the script thread forward a mozbrowser event to a mozbrowser iframe it owns,
    /// or to the window if no browsing context id is provided.
    MozBrowserEvent(PipelineId, Option<TopLevelBrowsingContextId>, MozBrowserEvent),
    /// Updates the current pipeline ID of a given iframe.
    /// First PipelineId is for the parent, second is the new PipelineId for the frame.
    UpdatePipelineId(PipelineId, BrowsingContextId, PipelineId, UpdatePipelineIdReason),
    /// Set an iframe to be focused. Used when an element in an iframe gains focus.
    /// PipelineId is for the parent, BrowsingContextId is for the nested browsing context
    FocusIFrame(PipelineId, BrowsingContextId),
    /// Passes a webdriver command to the script thread for execution
    WebDriverScriptCommand(PipelineId, WebDriverScriptCommand),
    /// Notifies script thread that all animations are done
    TickAllAnimations(PipelineId),
    /// Notifies the script thread of a transition end
    TransitionEnd(UntrustedNodeAddress, String, f64),
    /// Notifies the script thread that a new Web font has been loaded, and thus the page should be
    /// reflowed.
    WebFontLoaded(PipelineId),
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
    DispatchStorageEvent(PipelineId, StorageType, ServoUrl, Option<String>, Option<String>, Option<String>),
    /// Report an error from a CSS parser for the given pipeline
    ReportCSSError(PipelineId, String, usize, usize, String),
    /// Reload the given page.
    Reload(PipelineId),
    /// Notifies the script thread of WebVR events.
    WebVREvents(PipelineId, Vec<WebVREvent>)
}

impl fmt::Debug for ConstellationControlMsg {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        use self::ConstellationControlMsg::*;
        let variant = match *self {
            NavigationResponse(..) => "NavigationResponse",
            AttachLayout(..) => "AttachLayout",
            Resize(..) => "Resize",
            ResizeInactive(..) => "ResizeInactive",
            ExitPipeline(..) => "ExitPipeline",
            ExitScriptThread => "ExitScriptThread",
            SendEvent(..) => "SendEvent",
            Viewport(..) => "Viewport",
            SetScrollState(..) => "SetScrollState",
            GetTitle(..) => "GetTitle",
            SetDocumentActivity(..) => "SetDocumentActivity",
            ChangeFrameVisibilityStatus(..) => "ChangeFrameVisibilityStatus",
            NotifyVisibilityChange(..) => "NotifyVisibilityChange",
            Navigate(..) => "Navigate",
            PostMessage(..) => "PostMessage",
            MozBrowserEvent(..) => "MozBrowserEvent",
            UpdatePipelineId(..) => "UpdatePipelineId",
            FocusIFrame(..) => "FocusIFrame",
            WebDriverScriptCommand(..) => "WebDriverScriptCommand",
            TickAllAnimations(..) => "TickAllAnimations",
            TransitionEnd(..) => "TransitionEnd",
            WebFontLoaded(..) => "WebFontLoaded",
            DispatchIFrameLoadEvent { .. } => "DispatchIFrameLoadEvent",
            DispatchStorageEvent(..) => "DispatchStorageEvent",
            ReportCSSError(..) => "ReportCSSError",
            Reload(..) => "Reload",
            WebVREvents(..) => "WebVREvents",
        };
        write!(formatter, "ConstellationMsg::{}", variant)
    }
}

/// Used to determine if a script has any pending asynchronous activity.
#[derive(Copy, Clone, Debug, PartialEq, Deserialize, Serialize)]
pub enum DocumentState {
    /// The document has been loaded and is idle.
    Idle,
    /// The document is either loading or waiting on an event.
    Pending,
}

/// For a given pipeline, whether any animations are currently running
/// and any animation callbacks are queued
#[derive(Clone, Eq, PartialEq, Deserialize, Serialize, Debug)]
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

/// The type of input represented by a multi-touch event.
#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub enum TouchEventType {
    /// A new touch point came in contact with the screen.
    Down,
    /// An existing touch point changed location.
    Move,
    /// A touch point was removed from the screen.
    Up,
    /// The system stopped tracking a touch point.
    Cancel,
}

/// An opaque identifier for a touch point.
///
/// http://w3c.github.io/touch-events/#widl-Touch-identifier
#[derive(Clone, Copy, Debug, Eq, PartialEq, Deserialize, Serialize)]
pub struct TouchId(pub i32);

/// The mouse button involved in the event.
#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub enum MouseButton {
    /// The left mouse button.
    Left,
    /// The middle mouse button.
    Middle,
    /// The right mouse button.
    Right,
}

/// The types of mouse events
#[derive(Deserialize, HeapSizeOf, Serialize)]
pub enum MouseEventType {
    /// Mouse button clicked
    Click,
    /// Mouse button down
    MouseDown,
    /// Mouse button up
    MouseUp,
}

/// Events from the compositor that the script thread needs to know about
#[derive(Deserialize, Serialize)]
pub enum CompositorEvent {
    /// The window was resized.
    ResizeEvent(WindowSizeData, WindowSizeType),
    /// A mouse button state changed.
    MouseButtonEvent(MouseEventType, MouseButton, Point2D<f32>),
    /// The mouse was moved over a point (or was moved out of the recognizable region).
    MouseMoveEvent(Option<Point2D<f32>>),
    /// A touch event was generated with a touch ID and location.
    TouchEvent(TouchEventType, TouchId, Point2D<f32>),
    /// Touchpad pressure event
    TouchpadPressureEvent(Point2D<f32>, f32, TouchpadPressurePhase),
    /// A key was pressed.
    KeyEvent(Option<char>, Key, KeyState, KeyModifiers),
}

/// Touchpad pressure phase for `TouchpadPressureEvent`.
#[derive(Copy, Clone, HeapSizeOf, PartialEq, Deserialize, Serialize)]
pub enum TouchpadPressurePhase {
    /// Pressure before a regular click.
    BeforeClick,
    /// Pressure after a regular click.
    AfterFirstClick,
    /// Pressure after a "forceTouch" click
    AfterSecondClick,
}

/// Requests a TimerEvent-Message be sent after the given duration.
#[derive(Deserialize, Serialize)]
pub struct TimerEventRequest(pub IpcSender<TimerEvent>, pub TimerSource, pub TimerEventId, pub MsDuration);

/// Type of messages that can be sent to the timer scheduler.
#[derive(Deserialize, Serialize)]
pub enum TimerSchedulerMsg {
    /// Message to schedule a new timer event.
    Request(TimerEventRequest),
    /// Message to exit the timer scheduler.
    Exit,
}

/// Notifies the script thread to fire due timers.
/// `TimerSource` must be `FromWindow` when dispatched to `ScriptThread` and
/// must be `FromWorker` when dispatched to a `DedicatedGlobalWorkerScope`
#[derive(Deserialize, Serialize)]
pub struct TimerEvent(pub TimerSource, pub TimerEventId);

/// Describes the thread that requested the TimerEvent.
#[derive(Copy, Clone, HeapSizeOf, Deserialize, Serialize)]
pub enum TimerSource {
    /// The event was requested from a window (ScriptThread).
    FromWindow(PipelineId),
    /// The event was requested from a worker (DedicatedGlobalWorkerScope).
    FromWorker,
}

/// The id to be used for a `TimerEvent` is defined by the corresponding `TimerEventRequest`.
#[derive(PartialEq, Eq, Copy, Clone, Debug, HeapSizeOf, Deserialize, Serialize)]
pub struct TimerEventId(pub u32);

/// Unit of measurement.
#[derive(Clone, Copy, HeapSizeOf)]
pub enum Milliseconds {}
/// Unit of measurement.
#[derive(Clone, Copy, HeapSizeOf)]
pub enum Nanoseconds {}

/// Amount of milliseconds.
pub type MsDuration = Length<u64, Milliseconds>;
/// Amount of nanoseconds.
pub type NsDuration = Length<u64, Nanoseconds>;

/// Returns the duration since an unspecified epoch measured in ms.
pub fn precise_time_ms() -> MsDuration {
    Length::new(time::precise_time_ns() / (1000 * 1000))
}
/// Returns the duration since an unspecified epoch measured in ns.
pub fn precise_time_ns() -> NsDuration {
    Length::new(time::precise_time_ns())
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
    pub parent_info: Option<(PipelineId, FrameType)>,
    /// The ID of the browsing context this script is part of.
    pub browsing_context_id: BrowsingContextId,
    /// The ID of the top-level browsing context this script is part of.
    pub top_level_browsing_context_id: TopLevelBrowsingContextId,
    /// A channel with which messages can be sent to us (the script thread).
    pub control_chan: IpcSender<ConstellationControlMsg>,
    /// A port on which messages sent by the constellation to script can be received.
    pub control_port: IpcReceiver<ConstellationControlMsg>,
    /// A channel on which messages can be sent to the constellation from script.
    pub constellation_chan: IpcSender<ScriptMsg>,
    /// A sender for the layout thread to communicate to the constellation.
    pub layout_to_constellation_chan: IpcSender<LayoutMsg>,
    /// A channel to schedule timer events.
    pub scheduler_chan: IpcSender<TimerSchedulerMsg>,
    /// A channel to the resource manager thread.
    pub resource_threads: ResourceThreads,
    /// A channel to the bluetooth thread.
    pub bluetooth_thread: IpcSender<BluetoothRequest>,
    /// The image cache for this script thread.
    pub image_cache: Arc<ImageCache>,
    /// A channel to the time profiler thread.
    pub time_profiler_chan: profile_traits::time::ProfilerChan,
    /// A channel to the memory profiler thread.
    pub mem_profiler_chan: mem::ProfilerChan,
    /// A channel to the developer tools, if applicable.
    pub devtools_chan: Option<IpcSender<ScriptToDevtoolsControlMsg>>,
    /// Information about the initial window size.
    pub window_size: Option<WindowSizeData>,
    /// The ID of the pipeline namespace for this script thread.
    pub pipeline_namespace_id: PipelineNamespaceId,
    /// A ping will be sent on this channel once the script thread shuts down.
    pub content_process_shutdown_chan: IpcSender<()>,
    /// A channel to the webvr thread, if available.
    pub webvr_thread: Option<IpcSender<WebVRMsg>>
}

/// This trait allows creating a `ScriptThread` without depending on the `script`
/// crate.
pub trait ScriptThreadFactory {
    /// Type of message sent from script to layout.
    type Message;
    /// Create a `ScriptThread`.
    fn create(state: InitialScriptState, load_data: LoadData)
        -> (Sender<Self::Message>, Receiver<Self::Message>);
}

/// Whether the sandbox attribute is present for an iframe element
#[derive(PartialEq, Eq, Copy, Clone, Debug, Deserialize, Serialize)]
pub enum IFrameSandboxState {
    /// Sandbox attribute is present
    IFrameSandboxed,
    /// Sandbox attribute is not present
    IFrameUnsandboxed,
}

/// Specifies the information required to load an iframe.
#[derive(Deserialize, Serialize)]
pub struct IFrameLoadInfo {
    /// Pipeline ID of the parent of this iframe
    pub parent_pipeline_id: PipelineId,
    /// The ID for this iframe's nested browsing context.
    pub browsing_context_id: BrowsingContextId,
    /// The ID for the top-level ancestor browsing context of this iframe's nested browsing context.
    /// Note: this is the same as the browsing_context_id for mozbrowser iframes.
    pub top_level_browsing_context_id: TopLevelBrowsingContextId,
    /// The new pipeline ID that the iframe has generated.
    pub new_pipeline_id: PipelineId,
    ///  Whether this iframe should be considered private
    pub is_private: bool,
    /// Whether this iframe is a mozbrowser iframe
    pub frame_type: FrameType,
    /// Wether this load should replace the current entry (reload). If true, the current
    /// entry will be replaced instead of a new entry being added.
    pub replace: bool,
}

/// Specifies the information required to load a URL in an iframe.
#[derive(Deserialize, Serialize)]
pub struct IFrameLoadInfoWithData {
    /// The information required to load an iframe.
    pub info: IFrameLoadInfo,
    /// Load data containing the url to load
    pub load_data: Option<LoadData>,
    /// The old pipeline ID for this iframe, if a page was previously loaded.
    pub old_pipeline_id: Option<PipelineId>,
    /// Sandbox type of this iframe
    pub sandbox: IFrameSandboxState,
}

// https://developer.mozilla.org/en-US/docs/Web/API/Using_the_Browser_API#Events
/// The events fired in a Browser API context (`<iframe mozbrowser>`)
#[derive(Deserialize, Serialize)]
pub enum MozBrowserEvent {
    /// Sent when the scroll position within a browser `<iframe>` changes.
    AsyncScroll,
    /// Sent when window.close() is called within a browser `<iframe>`.
    Close,
    /// Sent when a browser `<iframe>` tries to open a context menu. This allows
    /// handling `<menuitem>` element available within the browser `<iframe>`'s content.
    ContextMenu,
    /// Sent when an error occurred while trying to load content within a browser `<iframe>`.
    /// Includes a human-readable description, and a machine-readable report.
    Error(MozBrowserErrorType, String, String),
    /// Sent when the favicon of a browser `<iframe>` changes.
    IconChange(String, String, String),
    /// Sent when the browser `<iframe>` has reached the server.
    Connected,
    /// Sent when the browser `<iframe>` has finished loading all its assets.
    LoadEnd,
    /// Sent when the browser `<iframe>` starts to load a new page.
    LoadStart,
    /// Sent when a browser `<iframe>`'s location changes.
    LocationChange(String, bool, bool),
    /// Sent when a new tab is opened within a browser `<iframe>` as a result of the user
    /// issuing a command to open a link target in a new tab (for example ctrl/cmd + click.)
    /// Includes the URL.
    OpenTab(String),
    /// Sent when a new window is opened within a browser `<iframe>`.
    /// Includes the URL, target browsing context name, and features.
    OpenWindow(String, Option<String>, Option<String>),
    /// Sent when the SSL state changes within a browser `<iframe>`.
    SecurityChange(HttpsState),
    /// Sent when alert(), confirm(), or prompt() is called within a browser `<iframe>`.
    ShowModalPrompt(String, String, String, String), // TODO(simartin): Handle unblock()
    /// Sent when the document.title changes within a browser `<iframe>`.
    TitleChange(String),
    /// Sent when an HTTP authentification is requested.
    UsernameAndPasswordRequired,
    /// Sent when a link to a search engine is found.
    OpenSearch,
    /// Sent when visibility state changes.
    VisibilityChange(bool),
}

impl MozBrowserEvent {
    /// Get the name of the event as a `& str`
    pub fn name(&self) -> &'static str {
        match *self {
            MozBrowserEvent::AsyncScroll => "mozbrowserasyncscroll",
            MozBrowserEvent::Close => "mozbrowserclose",
            MozBrowserEvent::Connected => "mozbrowserconnected",
            MozBrowserEvent::ContextMenu => "mozbrowsercontextmenu",
            MozBrowserEvent::Error(_, _, _) => "mozbrowsererror",
            MozBrowserEvent::IconChange(_, _, _) => "mozbrowsericonchange",
            MozBrowserEvent::LoadEnd => "mozbrowserloadend",
            MozBrowserEvent::LoadStart => "mozbrowserloadstart",
            MozBrowserEvent::LocationChange(_, _, _) => "mozbrowserlocationchange",
            MozBrowserEvent::OpenTab(_) => "mozbrowseropentab",
            MozBrowserEvent::OpenWindow(_, _, _) => "mozbrowseropenwindow",
            MozBrowserEvent::SecurityChange(_) => "mozbrowsersecuritychange",
            MozBrowserEvent::ShowModalPrompt(_, _, _, _) => "mozbrowsershowmodalprompt",
            MozBrowserEvent::TitleChange(_) => "mozbrowsertitlechange",
            MozBrowserEvent::UsernameAndPasswordRequired => "mozbrowserusernameandpasswordrequired",
            MozBrowserEvent::OpenSearch => "mozbrowseropensearch",
            MozBrowserEvent::VisibilityChange(_) => "mozbrowservisibilitychange",
        }
    }
}

// https://developer.mozilla.org/en-US/docs/Web/Events/mozbrowsererror
/// The different types of Browser error events
#[derive(Deserialize, Serialize)]
pub enum MozBrowserErrorType {
    // For the moment, we are just reporting panics, using the "fatal" type.
    /// A fatal error
    Fatal,
}

impl MozBrowserErrorType {
    /// Get the name of the error type as a `& str`
    pub fn name(&self) -> &'static str {
        match *self {
            MozBrowserErrorType::Fatal => "fatal",
        }
    }
}

/// Specifies whether the script or layout thread needs to be ticked for animation.
#[derive(Deserialize, Serialize)]
pub enum AnimationTickType {
    /// The script thread.
    Script,
    /// The layout thread.
    Layout,
}

/// The scroll state of a stacking context.
#[derive(Copy, Clone, Debug, Deserialize, Serialize)]
pub struct ScrollState {
    /// The ID of the scroll root.
    pub scroll_root_id: ClipId,
    /// The scrolling offset of this stacking context.
    pub scroll_offset: Point2D<f32>,
}

/// One hardware pixel.
///
/// This unit corresponds to the smallest addressable element of the display hardware.
#[derive(Copy, Clone, Debug)]
pub enum DevicePixel {}

/// Data about the window size.
#[derive(Copy, Clone, Deserialize, Serialize, HeapSizeOf)]
pub struct WindowSizeData {
    /// The size of the initial layout viewport, before parsing an
    /// http://www.w3.org/TR/css-device-adapt/#initial-viewport
    pub initial_viewport: TypedSize2D<f32, CSSPixel>,

    /// The resolution of the window in dppx, not including any "pinch zoom" factor.
    pub device_pixel_ratio: ScaleFactor<f32, CSSPixel, DevicePixel>,
}

/// The type of window size change.
#[derive(Deserialize, Eq, PartialEq, Serialize, Copy, Clone, HeapSizeOf)]
pub enum WindowSizeType {
    /// Initial load.
    Initial,
    /// Window resize.
    Resize,
}

/// Messages to the constellation originating from the WebDriver server.
#[derive(Deserialize, Serialize)]
pub enum WebDriverCommandMsg {
    /// Get the window size.
    GetWindowSize(TopLevelBrowsingContextId, IpcSender<WindowSizeData>),
    /// Load a URL in the top-level browsing context with the given ID.
    LoadUrl(TopLevelBrowsingContextId, LoadData, IpcSender<LoadStatus>),
    /// Refresh the top-level browsing context with the given ID.
    Refresh(TopLevelBrowsingContextId, IpcSender<LoadStatus>),
    /// Pass a webdriver command to the script thread of the current pipeline
    /// of a browsing context.
    ScriptCommand(BrowsingContextId, WebDriverScriptCommand),
    /// Act as if keys were pressed in the browsing context with the given ID.
    SendKeys(BrowsingContextId, Vec<(Key, KeyModifiers, KeyState)>),
    /// Set the window size.
    SetWindowSize(TopLevelBrowsingContextId, Size2D<u32>, IpcSender<WindowSizeData>),
    /// Take a screenshot of the window.
    TakeScreenshot(TopLevelBrowsingContextId, IpcSender<Option<Image>>),
}

/// Messages to the constellation.
#[derive(Deserialize, Serialize)]
pub enum ConstellationMsg {
    /// Exit the constellation.
    Exit,
    /// Request that the constellation send the BrowsingContextId corresponding to the document
    /// with the provided pipeline id
    GetBrowsingContext(PipelineId, IpcSender<Option<BrowsingContextId>>),
    /// Request that the constellation send the current pipeline id for the provided
    /// browsing context id, over a provided channel.
    GetPipeline(BrowsingContextId, IpcSender<Option<PipelineId>>),
    /// Request that the constellation send the current focused top-level browsing context id,
    /// over a provided channel.
    GetFocusTopLevelBrowsingContext(IpcSender<Option<TopLevelBrowsingContextId>>),
    /// Requests that the constellation inform the compositor of the title of the pipeline
    /// immediately.
    GetPipelineTitle(PipelineId),
    /// Request to load the initial page.
    InitLoadUrl(ServoUrl),
    /// Query the constellation to see if the current compositor output is stable
    IsReadyToSaveImage(HashMap<PipelineId, Epoch>),
    /// Inform the constellation of a key event.
    KeyEvent(Option<char>, Key, KeyState, KeyModifiers),
    /// Request to load a page.
    LoadUrl(PipelineId, LoadData),
    /// Request to traverse the joint session history of the provided browsing context.
    TraverseHistory(TopLevelBrowsingContextId, TraversalDirection),
    /// Inform the constellation of a window being resized.
    WindowSize(TopLevelBrowsingContextId, WindowSizeData, WindowSizeType),
    /// Requests that the constellation instruct layout to begin a new tick of the animation.
    TickAnimation(PipelineId, AnimationTickType),
    /// Dispatch a webdriver command
    WebDriverCommand(WebDriverCommandMsg),
    /// Reload a top-level browsing context.
    Reload(TopLevelBrowsingContextId),
    /// A log entry, with the top-level browsing context id and thread name
    LogEntry(Option<TopLevelBrowsingContextId>, Option<String>, LogEntry),
    /// Set the WebVR thread channel.
    SetWebVRThread(IpcSender<WebVRMsg>),
    /// Dispatch WebVR events to the subscribed script threads.
    WebVREvents(Vec<PipelineId>, Vec<WebVREvent>),
}

/// Resources required by workerglobalscopes
#[derive(Serialize, Deserialize, Clone)]
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
    pub constellation_chan: IpcSender<ScriptMsg>,
    /// Message to send to the scheduler
    pub scheduler_chan: IpcSender<TimerSchedulerMsg>,
    /// The worker id
    pub worker_id: WorkerId,
    /// The pipeline id
    pub pipeline_id: PipelineId,
    /// The origin
    pub origin: ImmutableOrigin,
}

/// Common entities representing a network load origin
#[derive(Deserialize, Serialize, Clone)]
pub struct WorkerScriptLoadOrigin {
    /// referrer url
    pub referrer_url: Option<ServoUrl>,
    /// the referrer policy which is used
    pub referrer_policy: Option<ReferrerPolicy>,
    /// the pipeline id of the entity requesting the load
    pub pipeline_id: Option<PipelineId>,
}

/// Errors from executing a paint worklet
#[derive(Debug, Deserialize, Serialize, Clone)]
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

/// Execute paint code in the worklet thread pool.<
pub trait PaintWorkletExecutor: Sync + Send {
    /// https://drafts.css-houdini.org/css-paint-api/#draw-a-paint-image
    fn draw_a_paint_image(&self,
                          name: Atom,
                          concrete_object_size: Size2D<Au>)
                          -> Result<Image, PaintWorkletError>;
}


/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! This module contains traits in script used generically in the rest of Servo.
//! The traits are here instead of in script so that these modules won't have
//! to depend on script.

#![feature(custom_derive, plugin)]
#![plugin(heapsize_plugin, plugins, serde_macros)]
#![deny(missing_docs)]
#![deny(unsafe_code)]

extern crate app_units;
extern crate canvas_traits;
extern crate cookie as cookie_rs;
extern crate devtools_traits;
extern crate euclid;
extern crate gfx_traits;
extern crate heapsize;
extern crate ipc_channel;
extern crate layers;
extern crate libc;
extern crate msg;
extern crate net_traits;
extern crate offscreen_gl_context;
extern crate profile_traits;
extern crate rustc_serialize;
extern crate serde;
extern crate style_traits;
extern crate time;
extern crate url;
extern crate util;

mod script_msg;
pub mod webdriver_msg;

use app_units::Au;
use devtools_traits::{DevtoolScriptControlMsg, ScriptToDevtoolsControlMsg, WorkerId};
use euclid::Size2D;
use euclid::length::Length;
use euclid::point::Point2D;
use euclid::rect::Rect;
use euclid::scale_factor::ScaleFactor;
use euclid::size::TypedSize2D;
use gfx_traits::Epoch;
use gfx_traits::LayerId;
use gfx_traits::StackingContextId;
use heapsize::HeapSizeOf;
use ipc_channel::ipc::{IpcReceiver, IpcSender};
use layers::geometry::DevicePixel;
use libc::c_void;
use msg::constellation_msg::{FrameId, FrameType, Image, Key, KeyModifiers, KeyState, LoadData};
use msg::constellation_msg::{NavigationDirection, PipelineId, ReferrerPolicy};
use msg::constellation_msg::{PipelineNamespaceId, SubpageId, WindowSizeType};
use net_traits::bluetooth_thread::BluetoothMethodMsg;
use net_traits::image_cache_thread::ImageCacheThread;
use net_traits::response::HttpsState;
use net_traits::{LoadOrigin, ResourceThreads};
use profile_traits::mem;
use profile_traits::time as profile_time;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::collections::HashMap;
use std::fmt;
use std::sync::mpsc::{Sender, Receiver};
use style_traits::{PagePx, ViewportPx};
use url::Url;
use util::ipc::OptionalOpaqueIpcSender;
use webdriver_msg::{LoadStatus, WebDriverScriptCommand};

pub use script_msg::{LayoutMsg, ScriptMsg, EventResult, LogEntry};
pub use script_msg::{ServiceWorkerMsg, ScopeThings, SWManagerMsg, SWManagerSenders};

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
    fn serialize<S: Serializer>(&self, s: &mut S) -> Result<(), S::Error> {
        (self.0 as usize).serialize(s)
    }
}

impl Deserialize for UntrustedNodeAddress {
    fn deserialize<D: Deserializer>(d: &mut D) -> Result<UntrustedNodeAddress, D::Error> {
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
    /// Informs layout as to which regions of the page are visible.
    SetVisibleRects(Vec<(LayerId, Rect<Au>)>),
    /// Tells layout about the new scrolling offsets of each scrollable stacking context.
    SetStackingContextScrollStates(Vec<StackingContextScrollState>),
    /// Requests the current load state of Web fonts. `true` is returned if fonts are still loading
    /// and `false` is returned if all fonts have loaded.
    GetWebFontLoadState(IpcSender<bool>),
}

/// The initial data associated with a newly-created framed pipeline.
#[derive(Deserialize, Serialize)]
pub struct NewLayoutInfo {
    /// Id of the parent of this new pipeline.
    pub containing_pipeline_id: PipelineId,
    /// Id of the newly-created pipeline.
    pub new_pipeline_id: PipelineId,
    /// Id of the new frame associated with this pipeline.
    pub subpage_id: SubpageId,
    /// Type of the new frame associated with this pipeline.
    pub frame_type: FrameType,
    /// Network request data which will be initiated by the script thread.
    pub load_data: LoadData,
    /// The paint channel, cast to `OptionalOpaqueIpcSender`. This is really an
    /// `Sender<LayoutToPaintMsg>`.
    pub paint_chan: OptionalOpaqueIpcSender,
    /// A port on which layout can receive messages from the pipeline.
    pub pipeline_port: IpcReceiver<LayoutControlMsg>,
    /// A sender for the layout thread to communicate to the constellation.
    pub layout_to_constellation_chan: IpcSender<LayoutMsg>,
    /// A shutdown channel so that layout can tell the content process to shut down when it's done.
    pub content_process_shutdown_chan: IpcSender<()>,
}

/// Messages sent from the constellation or layout to the script thread.
#[derive(Deserialize, Serialize)]
pub enum ConstellationControlMsg {
    /// Gives a channel and ID to a layout thread, as well as the ID of that layout's parent
    AttachLayout(NewLayoutInfo),
    /// Window resized.  Sends a DOM event eventually, but first we combine events.
    Resize(PipelineId, WindowSizeData, WindowSizeType),
    /// Notifies script that window has been resized but to not take immediate action.
    ResizeInactive(PipelineId, WindowSizeData),
    /// Notifies the script that a pipeline should be closed.
    ExitPipeline(PipelineId),
    /// Sends a DOM event.
    SendEvent(PipelineId, CompositorEvent),
    /// Notifies script of the viewport.
    Viewport(PipelineId, Rect<f32>),
    /// Notifies script of a new set of scroll offsets.
    SetScrollState(PipelineId, Vec<(UntrustedNodeAddress, Point2D<f32>)>),
    /// Requests that the script thread immediately send the constellation the title of a pipeline.
    GetTitle(PipelineId),
    /// Notifies script thread to suspend all its timers
    Freeze(PipelineId),
    /// Notifies script thread to resume all its timers
    Thaw(PipelineId),
    /// Notifies script thread whether frame is visible
    ChangeFrameVisibilityStatus(PipelineId, bool),
    /// Notifies script thread that frame visibility change is complete
    NotifyVisibilityChange(PipelineId, PipelineId, bool),
    /// Notifies script thread that a url should be loaded in this iframe.
    Navigate(PipelineId, SubpageId, LoadData),
    /// Requests the script thread forward a mozbrowser event to an iframe it owns,
    /// or to the window if no subpage id is provided.
    MozBrowserEvent(PipelineId, Option<SubpageId>, MozBrowserEvent),
    /// Updates the current subpage and pipeline IDs of a given iframe
    UpdateSubpageId(PipelineId, SubpageId, SubpageId, PipelineId),
    /// Set an iframe to be focused. Used when an element in an iframe gains focus.
    FocusIFrame(PipelineId, SubpageId),
    /// Passes a webdriver command to the script thread for execution
    WebDriverScriptCommand(PipelineId, WebDriverScriptCommand),
    /// Notifies script thread that all animations are done
    TickAllAnimations(PipelineId),
    /// Notifies the script thread that a new Web font has been loaded, and thus the page should be
    /// reflowed.
    WebFontLoaded(PipelineId),
    /// Cause a `load` event to be dispatched at the appropriate frame element.
    DispatchFrameLoadEvent {
        /// The pipeline that has been marked as loaded.
        target: PipelineId,
        /// The pipeline that contains a frame loading the target pipeline.
        parent: PipelineId,
    },
    /// Notifies a parent frame that one of its child frames is now active.
    FramedContentChanged(PipelineId, SubpageId),
    /// Report an error from a CSS parser for the given pipeline
    ReportCSSError(PipelineId, String, usize, usize, String),
    /// Reload the given page.
    Reload(PipelineId),
}

impl fmt::Debug for ConstellationControlMsg {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        use self::ConstellationControlMsg::*;
        write!(formatter, "ConstellationMsg::{}", match *self {
            AttachLayout(..) => "AttachLayout",
            Resize(..) => "Resize",
            ResizeInactive(..) => "ResizeInactive",
            ExitPipeline(..) => "ExitPipeline",
            SendEvent(..) => "SendEvent",
            Viewport(..) => "Viewport",
            SetScrollState(..) => "SetScrollState",
            GetTitle(..) => "GetTitle",
            Freeze(..) => "Freeze",
            Thaw(..) => "Thaw",
            ChangeFrameVisibilityStatus(..) => "ChangeFrameVisibilityStatus",
            NotifyVisibilityChange(..) => "NotifyVisibilityChange",
            Navigate(..) => "Navigate",
            MozBrowserEvent(..) => "MozBrowserEvent",
            UpdateSubpageId(..) => "UpdateSubpageId",
            FocusIFrame(..) => "FocusIFrame",
            WebDriverScriptCommand(..) => "WebDriverScriptCommand",
            TickAllAnimations(..) => "TickAllAnimations",
            WebFontLoaded(..) => "WebFontLoaded",
            DispatchFrameLoadEvent { .. } => "DispatchFrameLoadEvent",
            FramedContentChanged(..) => "FramedContentChanged",
            ReportCSSError(..) => "ReportCSSError",
            Reload(..) => "Reload",
        })
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

/// Touchpad pressure phase for TouchpadPressureEvent.
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
pub struct TimerEventRequest(pub IpcSender<TimerEvent>,
                             pub TimerSource,
                             pub TimerEventId,
                             pub MsDuration);

/// Notifies the script thread to fire due timers.
/// TimerSource must be FromWindow when dispatched to ScriptThread and
/// must be FromWorker when dispatched to a DedicatedGlobalWorkerScope
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

/// The id to be used for a TimerEvent is defined by the corresponding TimerEventRequest.
#[derive(PartialEq, Eq, Copy, Clone, Debug, HeapSizeOf, Deserialize, Serialize)]
pub struct TimerEventId(pub u32);

/// Unit of measurement.
#[derive(Clone, Copy, HeapSizeOf)]
pub enum Milliseconds {}
/// Unit of measurement.
#[derive(Clone, Copy, HeapSizeOf)]
pub enum Nanoseconds {}

/// Amount of milliseconds.
pub type MsDuration = Length<Milliseconds, u64>;
/// Amount of nanoseconds.
pub type NsDuration = Length<Nanoseconds, u64>;

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
    pub parent_info: Option<(PipelineId, SubpageId, FrameType)>,
    /// A channel with which messages can be sent to us (the script thread).
    pub control_chan: IpcSender<ConstellationControlMsg>,
    /// A port on which messages sent by the constellation to script can be received.
    pub control_port: IpcReceiver<ConstellationControlMsg>,
    /// A channel on which messages can be sent to the constellation from script.
    pub constellation_chan: IpcSender<ScriptMsg>,
    /// A channel to schedule timer events.
    pub scheduler_chan: IpcSender<TimerEventRequest>,
    /// A channel to the resource manager thread.
    pub resource_threads: ResourceThreads,
    /// A channel to the bluetooth thread.
    pub bluetooth_thread: IpcSender<BluetoothMethodMsg>,
    /// A channel to the image cache thread.
    pub image_cache_thread: ImageCacheThread,
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
}

/// This trait allows creating a `ScriptThread` without depending on the `script`
/// crate.
pub trait ScriptThreadFactory {
    /// Type of message sent from script to layout.
    type Message;
    /// Create a `ScriptThread`.
    fn create(state: InitialScriptState,
              load_data: LoadData)
              -> (Sender<Self::Message>, Receiver<Self::Message>);
}

/// Whether the sandbox attribute is present for an iframe element
#[derive(PartialEq, Eq, Copy, Clone, Debug, Deserialize, Serialize)]
pub enum IFrameSandboxState {
    /// Sandbox attribute is present
    IFrameSandboxed,
    /// Sandbox attribute is not present
    IFrameUnsandboxed
}

/// Specifies the information required to load a URL in an iframe.
#[derive(Deserialize, Serialize)]
pub struct IFrameLoadInfo {
    /// Load data containing the url to load
    pub load_data: Option<LoadData>,
    /// Pipeline ID of the parent of this iframe
    pub containing_pipeline_id: PipelineId,
    /// The new subpage ID for this load
    pub new_subpage_id: SubpageId,
    /// The old subpage ID for this iframe, if a page was previously loaded.
    pub old_subpage_id: Option<SubpageId>,
    /// The new pipeline ID that the iframe has generated.
    pub new_pipeline_id: PipelineId,
    /// Sandbox type of this iframe
    pub sandbox: IFrameSandboxState,
    ///  Whether this iframe should be considered private
    pub is_private: bool,
    /// Whether this iframe is a mozbrowser iframe
    pub frame_type: FrameType,
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
pub struct StackingContextScrollState {
    /// The ID of the stacking context.
    pub stacking_context_id: StackingContextId,
    /// The scrolling offset of this stacking context.
    pub scroll_offset: Point2D<f32>,
}

/// Data about the window size.
#[derive(Copy, Clone, Deserialize, Serialize, HeapSizeOf)]
pub struct WindowSizeData {
    /// The size of the initial layout viewport, before parsing an
    /// http://www.w3.org/TR/css-device-adapt/#initial-viewport
    pub initial_viewport: TypedSize2D<ViewportPx, f32>,

    /// The "viewing area" in page px. See `PagePx` documentation for details.
    pub visible_viewport: TypedSize2D<PagePx, f32>,

    /// The resolution of the window in dppx, not including any "pinch zoom" factor.
    pub device_pixel_ratio: ScaleFactor<ViewportPx, DevicePixel, f32>,
}

/// Messages to the constellation originating from the WebDriver server.
#[derive(Deserialize, Serialize)]
pub enum WebDriverCommandMsg {
    /// Get the window size.
    GetWindowSize(PipelineId, IpcSender<WindowSizeData>),
    /// Load a URL in the pipeline with the given ID.
    LoadUrl(PipelineId, LoadData, IpcSender<LoadStatus>),
    /// Refresh the pipeline with the given ID.
    Refresh(PipelineId, IpcSender<LoadStatus>),
    /// Pass a webdriver command to the script thread of the pipeline with the
    /// given ID for execution.
    ScriptCommand(PipelineId, WebDriverScriptCommand),
    /// Act as if keys were pressed in the pipeline with the given ID.
    SendKeys(PipelineId, Vec<(Key, KeyModifiers, KeyState)>),
    /// Set the window size.
    SetWindowSize(PipelineId, Size2D<u32>, IpcSender<WindowSizeData>),
    /// Take a screenshot of the window, if the pipeline with the given ID is
    /// the root pipeline.
    TakeScreenshot(PipelineId, IpcSender<Option<Image>>),
}

/// Messages to the constellation.
#[derive(Deserialize, Serialize)]
pub enum ConstellationMsg {
    /// Exit the constellation.
    Exit,
    /// Inform the constellation of the size of the viewport.
    FrameSize(PipelineId, Size2D<f32>),
    /// Request that the constellation send the FrameId corresponding to the document
    /// with the provided pipeline id
    GetFrame(PipelineId, IpcSender<Option<FrameId>>),
    /// Request that the constellation send the current pipeline id for the provided frame
    /// id, or for the root frame if this is None, over a provided channel.
    /// Also returns a boolean saying whether the document has finished loading or not.
    GetPipeline(Option<FrameId>, IpcSender<Option<(PipelineId, bool)>>),
    /// Requests that the constellation inform the compositor of the title of the pipeline
    /// immediately.
    GetPipelineTitle(PipelineId),
    /// Request to load the initial page.
    InitLoadUrl(Url),
    /// Query the constellation to see if the current compositor output is stable
    IsReadyToSaveImage(HashMap<PipelineId, Epoch>),
    /// Inform the constellation of a key event.
    KeyEvent(Option<char>, Key, KeyState, KeyModifiers),
    /// Request to load a page.
    LoadUrl(PipelineId, LoadData),
    /// Request to navigate a frame.
    Navigate(Option<(PipelineId, SubpageId)>, NavigationDirection),
    /// Inform the constellation of a window being resized.
    WindowSize(WindowSizeData, WindowSizeType),
    /// Requests that the constellation instruct layout to begin a new tick of the animation.
    TickAnimation(PipelineId, AnimationTickType),
    /// Dispatch a webdriver command
    WebDriverCommand(WebDriverCommandMsg),
    /// Reload the current page.
    Reload,
    /// A log entry, with the pipeline id and thread name
    LogEntry(Option<PipelineId>, Option<String>, LogEntry),
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
    pub scheduler_chan: IpcSender<TimerEventRequest>,
    /// The worker id
    pub worker_id: WorkerId,
}

/// Common entities representing a network load origin
#[derive(Deserialize, Serialize, Clone)]
pub struct WorkerScriptLoadOrigin {
    /// referrer url
    pub referrer_url: Option<Url>,
    /// the referrer policy which is used
    pub referrer_policy: Option<ReferrerPolicy>,
    /// the pipeline id of the entity requesting the load
    pub pipeline_id: Option<PipelineId>
}

impl LoadOrigin for WorkerScriptLoadOrigin {
    fn referrer_url(&self) -> Option<Url> {
        self.referrer_url.clone()
    }
    fn referrer_policy(&self) -> Option<ReferrerPolicy> {
        self.referrer_policy.clone()
    }
    fn pipeline_id(&self) -> Option<PipelineId> {
        self.pipeline_id.clone()
    }
}

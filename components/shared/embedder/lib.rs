/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

pub mod resources;

use std::fmt::{Debug, Error, Formatter};

use base::id::{PipelineId, TopLevelBrowsingContextId, WebViewId};
use crossbeam_channel::{Receiver, Sender};
use http::{HeaderMap, Method, StatusCode};
use ipc_channel::ipc::IpcSender;
use keyboard_types::KeyboardEvent;
use log::warn;
use malloc_size_of_derive::MallocSizeOf;
use num_derive::FromPrimitive;
use serde::{Deserialize, Serialize};
use servo_url::ServoUrl;
use webrender_api::units::{DeviceIntPoint, DeviceIntRect, DeviceIntSize};

/// A cursor for the window. This is different from a CSS cursor (see
/// `CursorKind`) in that it has no `Auto` value.
#[repr(u8)]
#[derive(Clone, Copy, Debug, Deserialize, Eq, FromPrimitive, PartialEq, Serialize)]
pub enum Cursor {
    None,
    Default,
    Pointer,
    ContextMenu,
    Help,
    Progress,
    Wait,
    Cell,
    Crosshair,
    Text,
    VerticalText,
    Alias,
    Copy,
    Move,
    NoDrop,
    NotAllowed,
    Grab,
    Grabbing,
    EResize,
    NResize,
    NeResize,
    NwResize,
    SResize,
    SeResize,
    SwResize,
    WResize,
    EwResize,
    NsResize,
    NeswResize,
    NwseResize,
    ColResize,
    RowResize,
    AllScroll,
    ZoomIn,
    ZoomOut,
}

#[cfg(feature = "webxr")]
pub use webxr_api::MainThreadWaker as EventLoopWaker;
#[cfg(not(feature = "webxr"))]
pub trait EventLoopWaker: 'static + Send {
    fn clone_box(&self) -> Box<dyn EventLoopWaker>;
    fn wake(&self);
}

#[cfg(not(feature = "webxr"))]
impl Clone for Box<dyn EventLoopWaker> {
    fn clone(&self) -> Self {
        EventLoopWaker::clone_box(self.as_ref())
    }
}

/// Sends messages to the embedder.
pub struct EmbedderProxy {
    pub sender: Sender<(Option<TopLevelBrowsingContextId>, EmbedderMsg)>,
    pub event_loop_waker: Box<dyn EventLoopWaker>,
}

impl EmbedderProxy {
    pub fn send(&self, msg: (Option<TopLevelBrowsingContextId>, EmbedderMsg)) {
        // Send a message and kick the OS event loop awake.
        if let Err(err) = self.sender.send(msg) {
            warn!("Failed to send response ({:?}).", err);
        }
        self.event_loop_waker.wake();
    }
}

impl Clone for EmbedderProxy {
    fn clone(&self) -> EmbedderProxy {
        EmbedderProxy {
            sender: self.sender.clone(),
            event_loop_waker: self.event_loop_waker.clone(),
        }
    }
}

/// The port that the embedder receives messages on.
pub struct EmbedderReceiver {
    pub receiver: Receiver<(Option<TopLevelBrowsingContextId>, EmbedderMsg)>,
}

impl EmbedderReceiver {
    pub fn try_recv_embedder_msg(
        &mut self,
    ) -> Option<(Option<TopLevelBrowsingContextId>, EmbedderMsg)> {
        self.receiver.try_recv().ok()
    }
    pub fn recv_embedder_msg(&mut self) -> (Option<TopLevelBrowsingContextId>, EmbedderMsg) {
        self.receiver.recv().unwrap()
    }
}

#[derive(Deserialize, Serialize)]
pub enum ContextMenuResult {
    Dismissed,
    Ignored,
    Selected(usize),
}

#[derive(Deserialize, Serialize)]
pub enum PromptDefinition {
    /// Show a message.
    Alert(String, IpcSender<()>),
    /// Ask a Ok/Cancel question.
    OkCancel(String, IpcSender<PromptResult>),
    /// Ask a Yes/No question.
    YesNo(String, IpcSender<PromptResult>),
    /// Ask the user to enter text.
    Input(String, String, IpcSender<Option<String>>),
    /// Ask user to enter their username and password
    Credentials(IpcSender<PromptCredentialsInput>),
}

#[derive(Debug, Deserialize, Serialize)]
pub struct PromptCredentialsInput {
    /// Username for http request authentication
    pub username: Option<String>,
    /// Password for http request authentication
    pub password: Option<String>,
}

#[derive(Deserialize, PartialEq, Serialize)]
pub enum PromptOrigin {
    /// Prompt is triggered from content (window.prompt/alert/confirm/…).
    /// Prompt message is unknown.
    Untrusted,
    /// Prompt is triggered from Servo (ask for permission, show error,…).
    Trusted,
}

#[derive(Deserialize, PartialEq, Serialize)]
pub enum PromptResult {
    /// Prompt was closed by clicking on the primary button (ok/yes)
    Primary,
    /// Prompt was closed by clicking on the secondary button (cancel/no)
    Secondary,
    /// Prompt was dismissed
    Dismissed,
}

#[derive(Deserialize, Serialize)]
pub enum EmbedderMsg {
    /// A status message to be displayed by the browser chrome.
    Status(Option<String>),
    /// Alerts the embedder that the current page has changed its title.
    ChangePageTitle(Option<String>),
    /// Move the window to a point
    MoveTo(DeviceIntPoint),
    /// Resize the window to size
    ResizeTo(DeviceIntSize),
    /// Show dialog to user
    Prompt(PromptDefinition, PromptOrigin),
    /// Show a context menu to the user
    ShowContextMenu(IpcSender<ContextMenuResult>, Option<String>, Vec<String>),
    /// Whether or not to allow a pipeline to load a url.
    AllowNavigationRequest(PipelineId, ServoUrl),
    /// Whether or not to allow script to open a new tab/browser
    AllowOpeningWebView(IpcSender<Option<WebViewId>>),
    /// A webview was created.
    WebViewOpened(TopLevelBrowsingContextId),
    /// A webview was destroyed.
    WebViewClosed(TopLevelBrowsingContextId),
    /// A webview gained focus for keyboard events.
    WebViewFocused(TopLevelBrowsingContextId),
    /// All webviews lost focus for keyboard events.
    WebViewBlurred,
    /// Wether or not to unload a document
    AllowUnload(IpcSender<bool>),
    /// Sends an unconsumed key event back to the embedder.
    Keyboard(KeyboardEvent),
    /// Inform embedder to clear the clipboard
    ClearClipboardContents,
    /// Gets system clipboard contents
    GetClipboardContents(IpcSender<String>),
    /// Sets system clipboard contents
    SetClipboardContents(String),
    /// Changes the cursor.
    SetCursor(Cursor),
    /// A favicon was detected
    NewFavicon(ServoUrl),
    /// `<head>` tag finished parsing
    HeadParsed,
    /// The history state has changed.
    HistoryChanged(Vec<ServoUrl>, usize),
    /// Enter or exit fullscreen
    SetFullscreenState(bool),
    /// The load of a page has begun
    LoadStart,
    /// The load of a page has completed
    LoadComplete,
    WebResourceRequested(WebResourceRequest, IpcSender<WebResourceResponseMsg>),
    /// A pipeline panicked. First string is the reason, second one is the backtrace.
    Panic(String, Option<String>),
    /// Open dialog to select bluetooth device.
    GetSelectedBluetoothDevice(Vec<String>, IpcSender<Option<String>>),
    /// Open file dialog to select files. Set boolean flag to true allows to select multiple files.
    SelectFiles(Vec<FilterPattern>, bool, IpcSender<Option<Vec<String>>>),
    /// Open interface to request permission specified by prompt.
    PromptPermission(PermissionPrompt, IpcSender<PermissionRequest>),
    /// Request to present an IME to the user when an editable element is focused.
    /// If the input is text, the second parameter defines the pre-existing string
    /// text content and the zero-based index into the string locating the insertion point.
    /// bool is true for multi-line and false otherwise.
    ShowIME(InputMethodType, Option<(String, i32)>, bool, DeviceIntRect),
    /// Request to hide the IME when the editable element is blurred.
    HideIME,
    /// Servo has shut down
    Shutdown,
    /// Report a complete sampled profile
    ReportProfile(Vec<u8>),
    /// Notifies the embedder about media session events
    /// (i.e. when there is metadata for the active media session, playback state changes...).
    MediaSessionEvent(MediaSessionEvent),
    /// Report the status of Devtools Server with a token that can be used to bypass the permission prompt.
    OnDevtoolsStarted(Result<u16, ()>, String),
    /// Notify the embedder that it needs to present a new frame.
    ReadyToPresent(Vec<WebViewId>),
    /// The given event was delivered to a pipeline in the given browser.
    EventDelivered(CompositorEventVariant),
    /// Request to play a haptic effect on a connected gamepad.
    PlayGamepadHapticEffect(usize, GamepadHapticEffectType, IpcSender<bool>),
    /// Request to stop a haptic effect on a connected gamepad.
    StopGamepadHapticEffect(usize, IpcSender<bool>),
}

/// The variant of CompositorEvent that was delivered to a pipeline.
#[derive(Debug, Deserialize, Serialize)]
pub enum CompositorEventVariant {
    ResizeEvent,
    MouseButtonEvent,
    MouseMoveEvent,
    TouchEvent,
    WheelEvent,
    KeyboardEvent,
    CompositionEvent,
    IMEDismissedEvent,
    GamepadEvent,
    ClipboardEvent,
}

impl Debug for EmbedderMsg {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        match *self {
            EmbedderMsg::Status(..) => write!(f, "Status"),
            EmbedderMsg::ChangePageTitle(..) => write!(f, "ChangePageTitle"),
            EmbedderMsg::MoveTo(..) => write!(f, "MoveTo"),
            EmbedderMsg::ResizeTo(..) => write!(f, "ResizeTo"),
            EmbedderMsg::Prompt(..) => write!(f, "Prompt"),
            EmbedderMsg::AllowUnload(..) => write!(f, "AllowUnload"),
            EmbedderMsg::AllowNavigationRequest(..) => write!(f, "AllowNavigationRequest"),
            EmbedderMsg::Keyboard(..) => write!(f, "Keyboard"),
            EmbedderMsg::ClearClipboardContents => write!(f, "ClearClipboardContents"),
            EmbedderMsg::GetClipboardContents(..) => write!(f, "GetClipboardContents"),
            EmbedderMsg::SetClipboardContents(..) => write!(f, "SetClipboardContents"),
            EmbedderMsg::SetCursor(..) => write!(f, "SetCursor"),
            EmbedderMsg::NewFavicon(..) => write!(f, "NewFavicon"),
            EmbedderMsg::HeadParsed => write!(f, "HeadParsed"),
            EmbedderMsg::HistoryChanged(..) => write!(f, "HistoryChanged"),
            EmbedderMsg::SetFullscreenState(..) => write!(f, "SetFullscreenState"),
            EmbedderMsg::LoadStart => write!(f, "LoadStart"),
            EmbedderMsg::LoadComplete => write!(f, "LoadComplete"),
            EmbedderMsg::WebResourceRequested(..) => write!(f, "WebResourceRequested"),
            EmbedderMsg::Panic(..) => write!(f, "Panic"),
            EmbedderMsg::GetSelectedBluetoothDevice(..) => write!(f, "GetSelectedBluetoothDevice"),
            EmbedderMsg::SelectFiles(..) => write!(f, "SelectFiles"),
            EmbedderMsg::PromptPermission(..) => write!(f, "PromptPermission"),
            EmbedderMsg::ShowIME(..) => write!(f, "ShowIME"),
            EmbedderMsg::HideIME => write!(f, "HideIME"),
            EmbedderMsg::Shutdown => write!(f, "Shutdown"),
            EmbedderMsg::AllowOpeningWebView(..) => write!(f, "AllowOpeningWebView"),
            EmbedderMsg::WebViewOpened(..) => write!(f, "WebViewOpened"),
            EmbedderMsg::WebViewClosed(..) => write!(f, "WebViewClosed"),
            EmbedderMsg::WebViewFocused(..) => write!(f, "WebViewFocused"),
            EmbedderMsg::WebViewBlurred => write!(f, "WebViewBlurred"),
            EmbedderMsg::ReportProfile(..) => write!(f, "ReportProfile"),
            EmbedderMsg::MediaSessionEvent(..) => write!(f, "MediaSessionEvent"),
            EmbedderMsg::OnDevtoolsStarted(..) => write!(f, "OnDevtoolsStarted"),
            EmbedderMsg::ShowContextMenu(..) => write!(f, "ShowContextMenu"),
            EmbedderMsg::ReadyToPresent(..) => write!(f, "ReadyToPresent"),
            EmbedderMsg::EventDelivered(..) => write!(f, "HitTestedEvent"),
            EmbedderMsg::PlayGamepadHapticEffect(..) => write!(f, "PlayGamepadHapticEffect"),
            EmbedderMsg::StopGamepadHapticEffect(..) => write!(f, "StopGamepadHapticEffect"),
        }
    }
}

/// Filter for file selection;
/// the `String` content is expected to be extension (e.g, "doc", without the prefixing ".")
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct FilterPattern(pub String);

/// <https://w3c.github.io/mediasession/#mediametadata>
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct MediaMetadata {
    /// Title
    pub title: String,
    /// Artist
    pub artist: String,
    /// Album
    pub album: String,
}

impl MediaMetadata {
    pub fn new(title: String) -> Self {
        Self {
            title,
            artist: "".to_owned(),
            album: "".to_owned(),
        }
    }
}

/// <https://w3c.github.io/mediasession/#enumdef-mediasessionplaybackstate>
#[repr(i32)]
#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum MediaSessionPlaybackState {
    /// The browsing context does not specify whether it’s playing or paused.
    None_ = 1,
    /// The browsing context is currently playing media and it can be paused.
    Playing,
    /// The browsing context has paused media and it can be resumed.
    Paused,
}

/// <https://w3c.github.io/mediasession/#dictdef-mediapositionstate>
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct MediaPositionState {
    pub duration: f64,
    pub playback_rate: f64,
    pub position: f64,
}

impl MediaPositionState {
    pub fn new(duration: f64, playback_rate: f64, position: f64) -> Self {
        Self {
            duration,
            playback_rate,
            position,
        }
    }
}

/// Type of events sent from script to the embedder about the media session.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum MediaSessionEvent {
    /// Indicates that the media metadata is available.
    SetMetadata(MediaMetadata),
    /// Indicates that the playback state has changed.
    PlaybackStateChange(MediaSessionPlaybackState),
    /// Indicates that the position state is set.
    SetPositionState(MediaPositionState),
}

/// Enum with variants that match the DOM PermissionName enum
#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum PermissionName {
    Geolocation,
    Notifications,
    Push,
    Midi,
    Camera,
    Microphone,
    Speaker,
    DeviceInfo,
    BackgroundSync,
    Bluetooth,
    PersistentStorage,
}

/// Information required to display a permission prompt
#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum PermissionPrompt {
    Insecure(PermissionName),
    Request(PermissionName),
}

/// Status for prompting user for permission.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum PermissionRequest {
    Granted,
    Denied,
}

/// Used to specify the kind of input method editor appropriate to edit a field.
/// This is a subset of htmlinputelement::InputType because some variants of InputType
/// don't make sense in this context.
#[derive(Debug, Deserialize, Serialize)]
pub enum InputMethodType {
    Color,
    Date,
    DatetimeLocal,
    Email,
    Month,
    Number,
    Password,
    Search,
    Tel,
    Text,
    Time,
    Url,
    Week,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
/// <https://w3.org/TR/gamepad/#dom-gamepadhapticeffecttype-dual-rumble>
pub struct DualRumbleEffectParams {
    pub duration: f64,
    pub start_delay: f64,
    pub strong_magnitude: f64,
    pub weak_magnitude: f64,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
/// <https://w3.org/TR/gamepad/#dom-gamepadhapticeffecttype>
pub enum GamepadHapticEffectType {
    DualRumble(DualRumbleEffectParams),
}

#[derive(Clone, Debug, Deserialize, MallocSizeOf, Serialize)]
pub struct WebResourceRequest {
    #[serde(
        deserialize_with = "::hyper_serde::deserialize",
        serialize_with = "::hyper_serde::serialize"
    )]
    #[ignore_malloc_size_of = "Defined in hyper"]
    pub method: Method,
    #[serde(
        deserialize_with = "::hyper_serde::deserialize",
        serialize_with = "::hyper_serde::serialize"
    )]
    #[ignore_malloc_size_of = "Defined in hyper"]
    pub headers: HeaderMap,
    pub url: ServoUrl,
    pub is_for_main_frame: bool,
    pub is_redirect: bool,
}

impl WebResourceRequest {
    pub fn new(
        method: Method,
        headers: HeaderMap,
        url: ServoUrl,
        is_for_main_frame: bool,
        is_redirect: bool,
    ) -> Self {
        WebResourceRequest {
            method,
            url,
            headers,
            is_for_main_frame,
            is_redirect,
        }
    }
}

#[derive(Clone, Deserialize, Serialize)]
pub enum WebResourceResponseMsg {
    // Response of WebResourceRequest, no body included.
    Start(WebResourceResponse),
    // send a body chunk. It is expected Response sent before body.
    Body(HttpBodyData),
    // not to override the response.
    None,
}

#[derive(Clone, Deserialize, Serialize)]
pub enum HttpBodyData {
    Chunk(Vec<u8>),
    Done,
    Cancelled,
}

#[derive(Clone, Debug, Deserialize, MallocSizeOf, Serialize)]
pub struct WebResourceResponse {
    pub url: ServoUrl,
    #[serde(
        deserialize_with = "::hyper_serde::deserialize",
        serialize_with = "::hyper_serde::serialize"
    )]
    #[ignore_malloc_size_of = "Defined in hyper"]
    pub headers: HeaderMap,
    #[serde(
        deserialize_with = "::hyper_serde::deserialize",
        serialize_with = "::hyper_serde::serialize"
    )]
    #[ignore_malloc_size_of = "Defined in hyper"]
    pub status_code: StatusCode,
    pub status_message: Vec<u8>,
}

impl WebResourceResponse {
    pub fn new(url: ServoUrl) -> WebResourceResponse {
        WebResourceResponse {
            url,
            headers: HeaderMap::new(),
            status_code: StatusCode::OK,
            status_message: b"OK".to_vec(),
        }
    }

    pub fn headers(mut self, headers: HeaderMap) -> WebResourceResponse {
        self.headers = headers;
        self
    }

    pub fn status_code(mut self, status_code: StatusCode) -> WebResourceResponse {
        self.status_code = status_code;
        self
    }

    pub fn status_message(mut self, status_message: Vec<u8>) -> WebResourceResponse {
        self.status_message = status_message;
        self
    }
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
/// <http://w3c.github.io/touch-events/#widl-Touch-identifier>
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct TouchId(pub i32);

#[derive(
    Clone, Copy, Debug, Deserialize, Eq, Hash, MallocSizeOf, Ord, PartialEq, PartialOrd, Serialize,
)]
/// Index of gamepad in list of system's connected gamepads
pub struct GamepadIndex(pub usize);

#[derive(Clone, Debug, Deserialize, Serialize)]
/// The minimum and maximum values that can be reported for axis or button input from this gamepad
pub struct GamepadInputBounds {
    /// Minimum and maximum axis values
    pub axis_bounds: (f64, f64),
    /// Minimum and maximum button values
    pub button_bounds: (f64, f64),
}

#[derive(Clone, Debug, Deserialize, Serialize)]
/// The haptic effects supported by this gamepad
pub struct GamepadSupportedHapticEffects {
    /// Gamepad support for dual rumble effects
    pub supports_dual_rumble: bool,
    /// Gamepad support for trigger rumble effects
    pub supports_trigger_rumble: bool,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
/// The type of Gamepad event
pub enum GamepadEvent {
    /// A new gamepad has been connected
    /// <https://www.w3.org/TR/gamepad/#event-gamepadconnected>
    Connected(
        GamepadIndex,
        String,
        GamepadInputBounds,
        GamepadSupportedHapticEffects,
    ),
    /// An existing gamepad has been disconnected
    /// <https://www.w3.org/TR/gamepad/#event-gamepaddisconnected>
    Disconnected(GamepadIndex),
    /// An existing gamepad has been updated
    /// <https://www.w3.org/TR/gamepad/#receiving-inputs>
    Updated(GamepadIndex, GamepadUpdateType),
}

#[derive(Clone, Debug, Deserialize, Serialize)]
/// The type of Gamepad input being updated
pub enum GamepadUpdateType {
    /// Axis index and input value
    /// <https://www.w3.org/TR/gamepad/#dfn-represents-a-standard-gamepad-axis>
    Axis(usize, f64),
    /// Button index and input value
    /// <https://www.w3.org/TR/gamepad/#dfn-represents-a-standard-gamepad-button>
    Button(usize, f64),
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub enum MouseButton {
    /// The left mouse button.
    Left = 1,
    /// The right mouse button.
    Right = 2,
    /// The middle mouse button.
    Middle = 4,
}

/// The types of mouse events
#[derive(Debug, Deserialize, MallocSizeOf, Serialize)]
pub enum MouseEventType {
    /// Mouse button clicked
    Click,
    /// Mouse button down
    MouseDown,
    /// Mouse button up
    MouseUp,
}

/// Mode to measure WheelDelta floats in
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
pub enum WheelMode {
    /// Delta values are specified in pixels
    DeltaPixel = 0x00,
    /// Delta values are specified in lines
    DeltaLine = 0x01,
    /// Delta values are specified in pages
    DeltaPage = 0x02,
}

/// The Wheel event deltas in every direction
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
pub struct WheelDelta {
    /// Delta in the left/right direction
    pub x: f64,
    /// Delta in the up/down direction
    pub y: f64,
    /// Delta in the direction going into/out of the screen
    pub z: f64,
    /// Mode to measure the floats in
    pub mode: WheelMode,
}

/// The mouse button involved in the event.
/// The types of clipboard events
#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum ClipboardEventType {
    /// Contents of the system clipboard are changed
    Change,
    /// Copy
    Copy,
    /// Cut
    Cut,
    /// Paste
    Paste(String),
}

impl ClipboardEventType {
    /// Convert to event name
    pub fn as_str(&self) -> &str {
        match *self {
            ClipboardEventType::Change => "clipboardchange",
            ClipboardEventType::Copy => "copy",
            ClipboardEventType::Cut => "cut",
            ClipboardEventType::Paste(..) => "paste",
        }
    }
}

/// The direction of a history traversal
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub enum TraversalDirection {
    /// Travel forward the given number of documents.
    Forward(usize),
    /// Travel backward the given number of documents.
    Back(usize),
}

/// The type of platform theme.
#[derive(Clone, Copy, Debug, Deserialize, Eq, MallocSizeOf, PartialEq, Serialize)]
pub enum Theme {
    /// Light theme.
    Light,
    /// Dark theme.
    Dark,
}
// The type of MediaSession action.
/// <https://w3c.github.io/mediasession/#enumdef-mediasessionaction>
#[derive(Clone, Debug, Deserialize, Eq, Hash, MallocSizeOf, PartialEq, Serialize)]
pub enum MediaSessionActionType {
    /// The action intent is to resume playback.
    Play,
    /// The action intent is to pause the currently active playback.
    Pause,
    /// The action intent is to move the playback time backward by a short period (i.e. a few
    /// seconds).
    SeekBackward,
    /// The action intent is to move the playback time forward by a short period (i.e. a few
    /// seconds).
    SeekForward,
    /// The action intent is to either start the current playback from the beginning if the
    /// playback has a notion, of beginning, or move to the previous item in the playlist if the
    /// playback has a notion of playlist.
    PreviousTrack,
    /// The action is to move to the playback to the next item in the playlist if the playback has
    /// a notion of playlist.
    NextTrack,
    /// The action intent is to skip the advertisement that is currently playing.
    SkipAd,
    /// The action intent is to stop the playback and clear the state if appropriate.
    Stop,
    /// The action intent is to move the playback time to a specific time.
    SeekTo,
}

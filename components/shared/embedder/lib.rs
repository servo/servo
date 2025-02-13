/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

pub mod input_events;
pub mod resources;

use std::fmt::{Debug, Error, Formatter};
use std::path::PathBuf;

use base::id::{PipelineId, WebViewId};
use crossbeam_channel::Sender;
use http::{HeaderMap, Method, StatusCode};
use ipc_channel::ipc::IpcSender;
pub use keyboard_types::{KeyboardEvent, Modifiers};
use log::warn;
use malloc_size_of_derive::MallocSizeOf;
use num_derive::FromPrimitive;
use serde::{Deserialize, Serialize};
use servo_url::ServoUrl;
use webrender_api::units::{DeviceIntPoint, DeviceIntRect, DeviceIntSize};

pub use crate::input_events::*;

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
    pub sender: Sender<EmbedderMsg>,
    pub event_loop_waker: Box<dyn EventLoopWaker>,
}

impl EmbedderProxy {
    pub fn send(&self, message: EmbedderMsg) {
        // Send a message and kick the OS event loop awake.
        if let Err(err) = self.sender.send(message) {
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
    /// Ask the user to enter text.
    Input(String, String, IpcSender<Option<String>>),
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct AuthenticationResponse {
    /// Username for http request authentication
    pub username: String,
    /// Password for http request authentication
    pub password: String,
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

/// A response to a request to allow or deny an action.
#[derive(Clone, Copy, Deserialize, PartialEq, Serialize)]
pub enum AllowOrDeny {
    Allow,
    Deny,
}

#[derive(Deserialize, Serialize)]
pub enum EmbedderMsg {
    /// A status message to be displayed by the browser chrome.
    Status(WebViewId, Option<String>),
    /// Alerts the embedder that the current page has changed its title.
    ChangePageTitle(WebViewId, Option<String>),
    /// Move the window to a point
    MoveTo(WebViewId, DeviceIntPoint),
    /// Resize the window to size
    ResizeTo(WebViewId, DeviceIntSize),
    /// Show dialog to user
    Prompt(WebViewId, PromptDefinition, PromptOrigin),
    /// Request authentication for a load or navigation from the embedder.
    RequestAuthentication(
        WebViewId,
        ServoUrl,
        bool, /* for proxy */
        IpcSender<Option<AuthenticationResponse>>,
    ),
    /// Show a context menu to the user
    ShowContextMenu(
        WebViewId,
        IpcSender<ContextMenuResult>,
        Option<String>,
        Vec<String>,
    ),
    /// Whether or not to allow a pipeline to load a url.
    AllowNavigationRequest(WebViewId, PipelineId, ServoUrl),
    /// Whether or not to allow script to open a new tab/browser
    AllowOpeningWebView(WebViewId, IpcSender<Option<WebViewId>>),
    /// A webview was created.
    WebViewOpened(WebViewId),
    /// A webview was destroyed.
    WebViewClosed(WebViewId),
    /// A webview gained focus for keyboard events.
    WebViewFocused(WebViewId),
    /// All webviews lost focus for keyboard events.
    WebViewBlurred,
    /// Wether or not to unload a document
    AllowUnload(WebViewId, IpcSender<AllowOrDeny>),
    /// Sends an unconsumed key event back to the embedder.
    Keyboard(WebViewId, KeyboardEvent),
    /// Inform embedder to clear the clipboard
    ClearClipboard(WebViewId),
    /// Gets system clipboard contents
    GetClipboardText(WebViewId, IpcSender<Result<String, String>>),
    /// Sets system clipboard contents
    SetClipboardText(WebViewId, String),
    /// Changes the cursor.
    SetCursor(WebViewId, Cursor),
    /// A favicon was detected
    NewFavicon(WebViewId, ServoUrl),
    /// The history state has changed.
    HistoryChanged(WebViewId, Vec<ServoUrl>, usize),
    /// Entered or exited fullscreen.
    NotifyFullscreenStateChanged(WebViewId, bool),
    /// The [`LoadStatus`] of the Given `WebView` has changed.
    NotifyLoadStatusChanged(WebViewId, LoadStatus),
    WebResourceRequested(
        Option<WebViewId>,
        WebResourceRequest,
        IpcSender<WebResourceResponseMsg>,
    ),
    /// A pipeline panicked. First string is the reason, second one is the backtrace.
    Panic(WebViewId, String, Option<String>),
    /// Open dialog to select bluetooth device.
    GetSelectedBluetoothDevice(WebViewId, Vec<String>, IpcSender<Option<String>>),
    /// Open file dialog to select files. Set boolean flag to true allows to select multiple files.
    SelectFiles(
        WebViewId,
        Vec<FilterPattern>,
        bool,
        IpcSender<Option<Vec<PathBuf>>>,
    ),
    /// Open interface to request permission specified by prompt.
    PromptPermission(WebViewId, PermissionFeature, IpcSender<AllowOrDeny>),
    /// Request to present an IME to the user when an editable element is focused.
    /// If the input is text, the second parameter defines the pre-existing string
    /// text content and the zero-based index into the string locating the insertion point.
    /// bool is true for multi-line and false otherwise.
    ShowIME(
        WebViewId,
        InputMethodType,
        Option<(String, i32)>,
        bool,
        DeviceIntRect,
    ),
    /// Request to hide the IME when the editable element is blurred.
    HideIME(WebViewId),
    /// Report a complete sampled profile
    ReportProfile(Vec<u8>),
    /// Notifies the embedder about media session events
    /// (i.e. when there is metadata for the active media session, playback state changes...).
    MediaSessionEvent(WebViewId, MediaSessionEvent),
    /// Report the status of Devtools Server with a token that can be used to bypass the permission prompt.
    OnDevtoolsStarted(Result<u16, ()>, String),
    /// Ask the user to allow a devtools client to connect.
    RequestDevtoolsConnection(IpcSender<AllowOrDeny>),
    /// Request to play a haptic effect on a connected gamepad.
    PlayGamepadHapticEffect(WebViewId, usize, GamepadHapticEffectType, IpcSender<bool>),
    /// Request to stop a haptic effect on a connected gamepad.
    StopGamepadHapticEffect(WebViewId, usize, IpcSender<bool>),
}

impl Debug for EmbedderMsg {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        match *self {
            EmbedderMsg::Status(..) => write!(f, "Status"),
            EmbedderMsg::ChangePageTitle(..) => write!(f, "ChangePageTitle"),
            EmbedderMsg::MoveTo(..) => write!(f, "MoveTo"),
            EmbedderMsg::ResizeTo(..) => write!(f, "ResizeTo"),
            EmbedderMsg::Prompt(..) => write!(f, "Prompt"),
            EmbedderMsg::RequestAuthentication(..) => write!(f, "RequestAuthentication"),
            EmbedderMsg::AllowUnload(..) => write!(f, "AllowUnload"),
            EmbedderMsg::AllowNavigationRequest(..) => write!(f, "AllowNavigationRequest"),
            EmbedderMsg::Keyboard(..) => write!(f, "Keyboard"),
            EmbedderMsg::ClearClipboard(..) => write!(f, "ClearClipboard"),
            EmbedderMsg::GetClipboardText(..) => write!(f, "GetClipboardText"),
            EmbedderMsg::SetClipboardText(..) => write!(f, "SetClipboardText"),
            EmbedderMsg::SetCursor(..) => write!(f, "SetCursor"),
            EmbedderMsg::NewFavicon(..) => write!(f, "NewFavicon"),
            EmbedderMsg::HistoryChanged(..) => write!(f, "HistoryChanged"),
            EmbedderMsg::NotifyFullscreenStateChanged(..) => {
                write!(f, "NotifyFullscreenStateChanged")
            },
            EmbedderMsg::NotifyLoadStatusChanged(_, status) => {
                write!(f, "NotifyLoadStatusChanged({status:?})")
            },
            EmbedderMsg::WebResourceRequested(..) => write!(f, "WebResourceRequested"),
            EmbedderMsg::Panic(..) => write!(f, "Panic"),
            EmbedderMsg::GetSelectedBluetoothDevice(..) => write!(f, "GetSelectedBluetoothDevice"),
            EmbedderMsg::SelectFiles(..) => write!(f, "SelectFiles"),
            EmbedderMsg::PromptPermission(..) => write!(f, "PromptPermission"),
            EmbedderMsg::ShowIME(..) => write!(f, "ShowIME"),
            EmbedderMsg::HideIME(..) => write!(f, "HideIME"),
            EmbedderMsg::AllowOpeningWebView(..) => write!(f, "AllowOpeningWebView"),
            EmbedderMsg::WebViewOpened(..) => write!(f, "WebViewOpened"),
            EmbedderMsg::WebViewClosed(..) => write!(f, "WebViewClosed"),
            EmbedderMsg::WebViewFocused(..) => write!(f, "WebViewFocused"),
            EmbedderMsg::WebViewBlurred => write!(f, "WebViewBlurred"),
            EmbedderMsg::ReportProfile(..) => write!(f, "ReportProfile"),
            EmbedderMsg::MediaSessionEvent(..) => write!(f, "MediaSessionEvent"),
            EmbedderMsg::OnDevtoolsStarted(..) => write!(f, "OnDevtoolsStarted"),
            EmbedderMsg::RequestDevtoolsConnection(..) => write!(f, "RequestDevtoolsConnection"),
            EmbedderMsg::ShowContextMenu(..) => write!(f, "ShowContextMenu"),
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
#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub enum PermissionFeature {
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

/// The status of the load in this `WebView`.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum LoadStatus {
    /// The load has started, but the headers have not yet been parsed.
    Started,
    /// The `<head>` tag has been parsed in the currently loading page. At this point the page's
    /// `HTMLBodyElement` is now available in the DOM.
    HeadParsed,
    /// The `Document` and all subresources have loaded. This is equivalent to
    /// `document.readyState` == `complete`.
    /// See <https://developer.mozilla.org/en-US/docs/Web/API/Document/readyState>
    Complete,
}

/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

pub mod input_events;
pub mod resources;
mod webdriver;

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
use url::Url;
use webrender_api::units::{DeviceIntPoint, DeviceIntRect, DeviceIntSize};

pub use crate::input_events::*;
pub use crate::webdriver::*;

/// Tracks whether Servo isn't shutting down, is in the process of shutting down,
/// or has finished shutting down.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ShutdownState {
    NotShuttingDown,
    ShuttingDown,
    FinishedShuttingDown,
}

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

/// [Simple dialogs](https://html.spec.whatwg.org/multipage/#simple-dialogs) are synchronous dialogs
/// that can be opened by web content. Since their messages are controlled by web content, they
/// should be presented to the user in a way that makes them impossible to mistake for browser UI.
#[derive(Deserialize, Serialize)]
pub enum SimpleDialog {
    /// [`alert()`](https://html.spec.whatwg.org/multipage/#dom-alert).
    /// TODO: Include details about the document origin.
    Alert {
        message: String,
        response_sender: IpcSender<AlertResponse>,
    },
    /// [`confirm()`](https://html.spec.whatwg.org/multipage/#dom-confirm).
    /// TODO: Include details about the document origin.
    Confirm {
        message: String,
        response_sender: IpcSender<ConfirmResponse>,
    },
    /// [`prompt()`](https://html.spec.whatwg.org/multipage/#dom-prompt).
    /// TODO: Include details about the document origin.
    Prompt {
        message: String,
        default: String,
        response_sender: IpcSender<PromptResponse>,
    },
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct AuthenticationResponse {
    /// Username for http request authentication
    pub username: String,
    /// Password for http request authentication
    pub password: String,
}

#[derive(Deserialize, PartialEq, Serialize)]
pub enum AlertResponse {
    /// The user chose Ok, or the dialog was otherwise dismissed or ignored.
    Ok,
}

impl Default for AlertResponse {
    fn default() -> Self {
        // Per <https://html.spec.whatwg.org/multipage/#dom-alert>,
        // if we **cannot show simple dialogs**, including cases where the user or user agent decides to ignore
        // all modal dialogs, we need to return (which represents Ok).
        Self::Ok
    }
}

#[derive(Deserialize, PartialEq, Serialize)]
pub enum ConfirmResponse {
    /// The user chose Ok.
    Ok,
    /// The user chose Cancel, or the dialog was otherwise dismissed or ignored.
    Cancel,
}

impl Default for ConfirmResponse {
    fn default() -> Self {
        // Per <https://html.spec.whatwg.org/multipage/#dom-confirm>,
        // if we **cannot show simple dialogs**, including cases where the user or user agent decides to ignore
        // all modal dialogs, we need to return false (which represents Cancel), not true (Ok).
        Self::Cancel
    }
}

#[derive(Deserialize, PartialEq, Serialize)]
pub enum PromptResponse {
    /// The user chose Ok, with the given input.
    Ok(String),
    /// The user chose Cancel, or the dialog was otherwise dismissed or ignored.
    Cancel,
}

impl Default for PromptResponse {
    fn default() -> Self {
        // Per <https://html.spec.whatwg.org/multipage/#dom-prompt>,
        // if we **cannot show simple dialogs**, including cases where the user or user agent decides to ignore
        // all modal dialogs, we need to return null (which represents Cancel), not the default input.
        Self::Cancel
    }
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
    /// Show the user a [simple dialog](https://html.spec.whatwg.org/multipage/#simple-dialogs) (`alert()`, `confirm()`,
    /// or `prompt()`). Since their messages are controlled by web content, they should be presented to the user in a
    /// way that makes them impossible to mistake for browser UI.
    ShowSimpleDialog(WebViewId, SimpleDialog),
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
    /// Informs the embedder that the constellation has completed shutdown.
    /// Required because the constellation can have pending calls to make
    /// (e.g. SetFrameTree) at the time that we send it an ExitMsg.
    ShutdownComplete,
}

impl Debug for EmbedderMsg {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        match *self {
            EmbedderMsg::Status(..) => write!(f, "Status"),
            EmbedderMsg::ChangePageTitle(..) => write!(f, "ChangePageTitle"),
            EmbedderMsg::MoveTo(..) => write!(f, "MoveTo"),
            EmbedderMsg::ResizeTo(..) => write!(f, "ResizeTo"),
            EmbedderMsg::ShowSimpleDialog(..) => write!(f, "ShowSimpleDialog"),
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
            EmbedderMsg::ShutdownComplete => write!(f, "ShutdownComplete"),
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
    pub url: Url,
    pub is_for_main_frame: bool,
    pub is_redirect: bool,
}

#[derive(Clone, Deserialize, Serialize)]
pub enum WebResourceResponseMsg {
    /// Start an interception of this web resource load. It's expected that the client subsequently
    /// send either a `CancelLoad` or `FinishLoad` message after optionally sending chunks of body
    /// data via `SendBodyData`.
    Start(WebResourceResponse),
    /// Send a chunk of body data.
    SendBodyData(Vec<u8>),
    /// Signal that this load has been finished by the interceptor.
    FinishLoad,
    /// Signal that this load has been cancelled by the interceptor.
    CancelLoad,
    /// Signal that this load will not be intercepted.
    DoNotIntercept,
}

#[derive(Clone, Debug, Deserialize, MallocSizeOf, Serialize)]
pub struct WebResourceResponse {
    pub url: Url,
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
    pub fn new(url: Url) -> WebResourceResponse {
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

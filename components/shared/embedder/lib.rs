/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

pub mod resources;

use std::fmt::{Debug, Error, Formatter};

use base::id::{PipelineId, TopLevelBrowsingContextId, WebViewId};
use crossbeam_channel::{Receiver, Sender};
use ipc_channel::ipc::IpcSender;
use keyboard_types::KeyboardEvent;
use log::warn;
use num_derive::FromPrimitive;
use serde::{Deserialize, Serialize};
use servo_url::ServoUrl;
use webrender_api::units::{DeviceIntPoint, DeviceIntRect, DeviceIntSize};
pub use webxr_api::MainThreadWaker as EventLoopWaker;

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
    AllowOpeningWebView(IpcSender<bool>),
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
            EmbedderMsg::GetClipboardContents(..) => write!(f, "GetClipboardContents"),
            EmbedderMsg::SetClipboardContents(..) => write!(f, "SetClipboardContents"),
            EmbedderMsg::SetCursor(..) => write!(f, "SetCursor"),
            EmbedderMsg::NewFavicon(..) => write!(f, "NewFavicon"),
            EmbedderMsg::HeadParsed => write!(f, "HeadParsed"),
            EmbedderMsg::HistoryChanged(..) => write!(f, "HistoryChanged"),
            EmbedderMsg::SetFullscreenState(..) => write!(f, "SetFullscreenState"),
            EmbedderMsg::LoadStart => write!(f, "LoadStart"),
            EmbedderMsg::LoadComplete => write!(f, "LoadComplete"),
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

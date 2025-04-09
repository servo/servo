/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Types used by the embedding layer and/or exposed to the API. This crate is responsible for
//! defining types that cross the process boundary from the embedding/rendering layer all the way
//! to script, thus it should have very minimal dependencies on other parts of Servo. If a type
//! is not exposed in the API or doesn't involve messages sent to the embedding/libservo layer, it
//! is probably a better fit for the `constellation_traits` crate.

pub mod input_events;
pub mod resources;
pub mod user_content_manager;
mod webdriver;

use std::ffi::c_void;
use std::fmt::{Debug, Error, Formatter};
use std::path::PathBuf;
use std::sync::Arc;

use base::id::{PipelineId, ScrollTreeNodeId, WebViewId};
use crossbeam_channel::Sender;
use euclid::{Scale, Size2D};
use http::{HeaderMap, Method, StatusCode};
use ipc_channel::ipc::IpcSender;
pub use keyboard_types::{KeyboardEvent, Modifiers};
use log::warn;
use malloc_size_of::malloc_size_of_is_0;
use malloc_size_of_derive::MallocSizeOf;
use num_derive::FromPrimitive;
use pixels::Image;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use servo_url::ServoUrl;
use strum_macros::IntoStaticStr;
use style_traits::CSSPixel;
use url::Url;
use webrender_api::units::{DeviceIntPoint, DeviceIntRect, DeviceIntSize, DevicePixel};

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

#[derive(Debug, Default, Deserialize, PartialEq, Serialize)]
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
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
pub enum AllowOrDeny {
    Allow,
    Deny,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SelectElementOption {
    /// A unique identifier for the option that can be used to select it.
    pub id: usize,
    /// The label that should be used to display the option to the user.
    pub label: String,
    /// Whether or not the option is selectable
    pub is_disabled: bool,
}

/// Represents the contents of either an `<option>` or an `<optgroup>` element
#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum SelectElementOptionOrOptgroup {
    Option(SelectElementOption),
    Optgroup {
        label: String,
        options: Vec<SelectElementOption>,
    },
}

/// Data about a `WebView` or `<iframe>` viewport: its size and also the
/// HiDPI scale factor to use when rendering the contents.
#[derive(Clone, Copy, Debug, Default, Deserialize, MallocSizeOf, PartialEq, Serialize)]
pub struct ViewportDetails {
    /// The size of the layout viewport.
    pub size: Size2D<f32, CSSPixel>,

    /// The scale factor to use to account for HiDPI scaling. This does not take into account
    /// any page or pinch zoom applied by the compositor to the contents.
    pub hidpi_scale_factor: Scale<f32, CSSPixel, DevicePixel>,
}

#[derive(Deserialize, IntoStaticStr, Serialize)]
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
    AllowOpeningWebView(WebViewId, IpcSender<Option<(WebViewId, ViewportDetails)>>),
    /// A webview was destroyed.
    WebViewClosed(WebViewId),
    /// A webview gained focus for keyboard events
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
    /// Request to display a notification.
    ShowNotification(Option<WebViewId>, Notification),
    /// Indicates that the user has activated a `<select>` element.
    ///
    /// The embedder should respond with the new state of the `<select>` element.
    ShowSelectElementMenu(
        WebViewId,
        Vec<SelectElementOptionOrOptgroup>,
        Option<usize>,
        DeviceIntRect,
        IpcSender<Option<usize>>,
    ),
}

impl Debug for EmbedderMsg {
    fn fmt(&self, formatter: &mut Formatter) -> Result<(), Error> {
        let string: &'static str = self.into();
        write!(formatter, "{string}")
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

/// Data that could be used to display a desktop notification to the end user
/// when the [Notification API](<https://notifications.spec.whatwg.org/#notifications>) is called.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Notification {
    /// Title of the notification.
    pub title: String,
    /// Body string of the notification.
    pub body: String,
    /// An identifier tag for the notification. Notification with the same tag
    /// can be replaced by another to avoid users' screen being filled up with similar notifications.
    pub tag: String,
    /// The tag for the language used in the notification's title, body, and the title of each its actions. [RFC 5646](https://datatracker.ietf.org/doc/html/rfc5646)
    pub language: String,
    /// A boolean value indicates the notification should remain readily available
    /// until the end user activates or dismisses the notification.
    pub require_interaction: bool,
    /// When `true`, indicates no sounds or vibrations should be made. When `None`,
    /// the device's default settings should be respected.
    pub silent: Option<bool>,
    /// The URL of an icon. The icon will be displayed as part of the notification.
    pub icon_url: Option<ServoUrl>,
    /// Icon's raw image data and metadata.
    pub icon_resource: Option<Arc<Image>>,
    /// The URL of a badge. The badge is used when there is no enough space to display the notification,
    /// such as on a mobile device's notification bar.
    pub badge_url: Option<ServoUrl>,
    /// Badge's raw image data and metadata.
    pub badge_resource: Option<Arc<Image>>,
    /// The URL of an image. The image will be displayed as part of the notification.
    pub image_url: Option<ServoUrl>,
    /// Image's raw image data and metadata.
    pub image_resource: Option<Arc<Image>>,
    /// Actions available for users to choose from for interacting with the notification.
    pub actions: Vec<NotificationAction>,
}

/// Actions available for users to choose from for interacting with the notification.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct NotificationAction {
    /// A string that identifies the action.
    pub name: String,
    /// The title string of the action to be shown to the user.
    pub title: String,
    /// The URL of an icon. The icon will be displayed with the action.
    pub icon_url: Option<ServoUrl>,
    /// Icon's raw image data and metadata.
    pub icon_resource: Option<Arc<Image>>,
}

/// Information about a `WebView`'s screen geometry and offset. This is used
/// for the [Screen](https://drafts.csswg.org/cssom-view/#the-screen-interface)
/// CSSOM APIs and `window.screenLeft` / `window.screenTop`.
#[derive(Clone, Copy, Debug, Default)]
pub struct ScreenGeometry {
    /// The size of the screen in device pixels. This will be converted to
    /// CSS pixels based on the pixel scaling of the `WebView`.
    pub size: DeviceIntSize,
    /// The available size of the screen in device pixels. This size is the size
    /// available for web content on the screen, and should be `size` minus any system
    /// toolbars, docks, and interface elements of the browser. This will be converted to
    /// CSS pixels based on the pixel scaling of the `WebView`.
    pub available_size: DeviceIntSize,
    /// The offset of the `WebView` in device pixels for the purposes of the `window.screenLeft`
    /// and `window.screenTop` APIs. This will be converted to CSS pixels based on the pixel scaling
    /// of the `WebView`.
    pub offset: DeviceIntPoint,
}

impl From<SelectElementOption> for SelectElementOptionOrOptgroup {
    fn from(value: SelectElementOption) -> Self {
        Self::Option(value)
    }
}

/// The address of a node. Layout sends these back. They must be validated via
/// `from_untrusted_node_address` before they can be used, because we do not trust layout.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct UntrustedNodeAddress(pub *const c_void);

malloc_size_of_is_0!(UntrustedNodeAddress);

#[allow(unsafe_code)]
unsafe impl Send for UntrustedNodeAddress {}

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

/// The result of a hit test in the compositor.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CompositorHitTestResult {
    /// The pipeline id of the resulting item.
    pub pipeline_id: PipelineId,

    /// The hit test point in the item's viewport.
    pub point_in_viewport: euclid::default::Point2D<f32>,

    /// The hit test point relative to the item itself.
    pub point_relative_to_item: euclid::default::Point2D<f32>,

    /// The node address of the hit test result.
    pub node: UntrustedNodeAddress,

    /// The cursor that should be used when hovering the item hit by the hit test.
    pub cursor: Option<Cursor>,

    /// The scroll tree node associated with this hit test item.
    pub scroll_tree_node: ScrollTreeNodeId,
}

/// Whether the default action for a touch event was prevented by web content
#[derive(Debug, Deserialize, Serialize)]
pub enum TouchEventResult {
    /// Allowed by web content
    DefaultAllowed(TouchSequenceId, TouchEventType),
    /// Prevented by web content
    DefaultPrevented(TouchSequenceId, TouchEventType),
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

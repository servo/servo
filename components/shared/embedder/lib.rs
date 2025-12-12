/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Types used by the embedding layer and/or exposed to the API. This crate is responsible for
//! defining types that cross the process boundary from the embedding/rendering layer all the way
//! to script, thus it should have very minimal dependencies on other parts of Servo. If a type
//! is not exposed in the API or doesn't involve messages sent to the embedding/libservo layer, it
//! is probably a better fit for the `constellation_traits` crate.

pub mod embedder_controls;
pub mod input_events;
pub mod resources;
pub mod user_content_manager;
pub mod webdriver;

use std::collections::HashMap;
use std::ffi::c_void;
use std::fmt::{Debug, Display, Error, Formatter};
use std::hash::Hash;
use std::ops::Range;
use std::path::PathBuf;
use std::sync::Arc;

use base::generic_channel::{GenericCallback, GenericSender, GenericSharedMemory, SendResult};
use base::id::{PipelineId, WebViewId};
use crossbeam_channel::Sender;
use euclid::{Box2D, Point2D, Scale, Size2D, Vector2D};
use http::{HeaderMap, Method, StatusCode};
use ipc_channel::ipc::IpcSender;
use log::warn;
use malloc_size_of::malloc_size_of_is_0;
use malloc_size_of_derive::MallocSizeOf;
use pixels::SharedRasterImage;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use servo_geometry::{DeviceIndependentIntRect, DeviceIndependentIntSize};
use servo_url::ServoUrl;
use strum::{EnumMessage, IntoStaticStr};
use style::queries::values::PrefersColorScheme;
use style_traits::CSSPixel;
use url::Url;
use uuid::Uuid;
use webrender_api::ExternalScrollId;
use webrender_api::units::{
    DeviceIntPoint, DeviceIntRect, DeviceIntSize, DevicePixel, DevicePoint, DeviceRect,
    DeviceVector2D, LayoutPoint, LayoutRect, LayoutSize, LayoutVector2D,
};

pub use crate::embedder_controls::*;
pub use crate::input_events::*;
pub use crate::webdriver::*;

/// A point in a `WebView`, either expressed in device pixels or page pixels.
/// Page pixels are CSS pixels, which take into account device pixel scale,
/// page zoom, and pinch zoom.
#[derive(Clone, Copy, Debug, Deserialize, MallocSizeOf, PartialEq, Serialize)]
pub enum WebViewPoint {
    Device(DevicePoint),
    Page(Point2D<f32, CSSPixel>),
}

impl WebViewPoint {
    pub fn as_device_point(&self, scale: Scale<f32, CSSPixel, DevicePixel>) -> DevicePoint {
        match self {
            Self::Device(point) => *point,
            Self::Page(point) => *point * scale,
        }
    }
}

impl From<DevicePoint> for WebViewPoint {
    fn from(point: DevicePoint) -> Self {
        Self::Device(point)
    }
}

impl From<LayoutPoint> for WebViewPoint {
    fn from(point: LayoutPoint) -> Self {
        Self::Page(Point2D::new(point.x, point.y))
    }
}

impl From<Point2D<f32, CSSPixel>> for WebViewPoint {
    fn from(point: Point2D<f32, CSSPixel>) -> Self {
        Self::Page(point)
    }
}

/// A rectangle in a `WebView`, either expressed in device pixels or page pixels.
/// Page pixels are CSS pixels, which take into account device pixel scale,
/// page zoom, and pinch zoom.
#[derive(Clone, Copy, Debug, Deserialize, MallocSizeOf, PartialEq, Serialize)]
pub enum WebViewRect {
    Device(DeviceRect),
    Page(Box2D<f32, CSSPixel>),
}

impl WebViewRect {
    pub fn as_device_rect(&self, scale: Scale<f32, CSSPixel, DevicePixel>) -> DeviceRect {
        match self {
            Self::Device(rect) => *rect,
            Self::Page(rect) => *rect * scale,
        }
    }
}

impl From<DeviceRect> for WebViewRect {
    fn from(rect: DeviceRect) -> Self {
        Self::Device(rect)
    }
}

impl From<LayoutRect> for WebViewRect {
    fn from(rect: LayoutRect) -> Self {
        Self::Page(Box2D::new(
            Point2D::new(rect.min.x, rect.min.y),
            Point2D::new(rect.max.x, rect.max.y),
        ))
    }
}

impl From<Box2D<f32, CSSPixel>> for WebViewRect {
    fn from(rect: Box2D<f32, CSSPixel>) -> Self {
        Self::Page(rect)
    }
}

#[derive(Clone, Copy, Debug, Deserialize, MallocSizeOf, PartialEq, Serialize)]
pub enum WebViewVector {
    Device(DeviceVector2D),
    Page(Vector2D<f32, CSSPixel>),
}

impl WebViewVector {
    pub fn as_device_vector(&self, scale: Scale<f32, CSSPixel, DevicePixel>) -> DeviceVector2D {
        match self {
            Self::Device(vector) => *vector,
            Self::Page(vector) => *vector * scale,
        }
    }
}

impl From<DeviceVector2D> for WebViewVector {
    fn from(vector: DeviceVector2D) -> Self {
        Self::Device(vector)
    }
}

impl From<LayoutVector2D> for WebViewVector {
    fn from(vector: LayoutVector2D) -> Self {
        Self::Page(Vector2D::new(vector.x, vector.y))
    }
}

impl From<Vector2D<f32, CSSPixel>> for WebViewVector {
    fn from(vector: Vector2D<f32, CSSPixel>) -> Self {
        Self::Page(vector)
    }
}

#[derive(Clone, Copy, Debug, Deserialize, MallocSizeOf, PartialEq, Serialize)]
pub enum Scroll {
    Delta(WebViewVector),
    Start,
    End,
}

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
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, MallocSizeOf, PartialEq, Serialize)]
pub enum Cursor {
    None,
    #[default]
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

pub trait EventLoopWaker: 'static + Send {
    fn clone_box(&self) -> Box<dyn EventLoopWaker>;
    fn wake(&self) {}
}

impl Clone for Box<dyn EventLoopWaker> {
    fn clone(&self) -> Self {
        self.clone_box()
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

/// A [`RefreshDriver`] is a trait that can be implemented by Servo embedders in
/// order to drive let Servo know when to start preparing the next frame. For example,
/// on systems that support Vsync notifications, an embedder may want to implement
/// this trait to drive Servo animations via those notifications.
pub trait RefreshDriver {
    /// Servo will call this method when it wants to be informed of the next frame start
    /// time. Implementors should call the callback when it is time to start preparing
    /// the new frame.
    ///
    /// Multiple callbacks may be registered for the same frame. It is up to the implementation
    /// to call *all* callbacks that have been registered since the last frame.
    fn observe_next_frame(&self, start_frame_callback: Box<dyn Fn() + Send + 'static>);
}

#[derive(Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct AuthenticationResponse {
    /// Username for http request authentication
    pub username: String,
    /// Password for http request authentication
    pub password: String,
}

/// A response to a request to allow or deny an action.
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
pub enum AllowOrDeny {
    Allow,
    Deny,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
/// Whether a protocol handler is requested to be registered or unregistered.
pub enum RegisterOrUnregister {
    Register,
    Unregister,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ProtocolHandlerUpdateRegistration {
    /// The scheme for the protocol handler
    pub scheme: String,
    /// The URL to navigate to when handling requests for scheme
    pub url: ServoUrl,
    /// Whether this update is to register or unregister the protocol handler
    pub register_or_unregister: RegisterOrUnregister,
}

/// Data about a `WebView` or `<iframe>` viewport: its size and also the
/// HiDPI scale factor to use when rendering the contents.
#[derive(Clone, Copy, Debug, Default, Deserialize, MallocSizeOf, PartialEq, Serialize)]
pub struct ViewportDetails {
    /// The size of the layout viewport.
    pub size: Size2D<f32, CSSPixel>,

    /// The scale factor to use to account for HiDPI scaling. This does not take into account
    /// any page or pinch zoom applied by `Paint` to the contents.
    pub hidpi_scale_factor: Scale<f32, CSSPixel, DevicePixel>,
}

impl ViewportDetails {
    /// Convert this [`ViewportDetails`] size to a [`LayoutSize`]. This is the same numerical
    /// value as [`Self::size`], because a `LayoutPixel` is the same as a `CSSPixel`.
    pub fn layout_size(&self) -> LayoutSize {
        Size2D::from_untyped(self.size.to_untyped())
    }
}

/// Unlike [`ScreenGeometry`], the data is in device-independent pixels
/// to be used by DOM APIs
#[derive(Default, Deserialize, Serialize)]
pub struct ScreenMetrics {
    pub screen_size: DeviceIndependentIntSize,
    pub available_size: DeviceIndependentIntSize,
}

/// An opaque identifier for a single history traversal operation.
#[derive(Clone, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct TraversalId(String);

impl TraversalId {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self(Uuid::new_v4().to_string())
    }
}

#[derive(Clone, Copy, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub enum PixelFormat {
    /// Luminance channel only
    K8,
    /// Luminance + alpha
    KA8,
    /// RGB, 8 bits per channel
    RGB8,
    /// RGB + alpha, 8 bits per channel
    RGBA8,
    /// BGR + alpha, 8 bits per channel
    BGRA8,
}

/// A raster image buffer.
#[derive(Clone, Deserialize, Serialize)]
pub struct Image {
    pub width: u32,
    pub height: u32,
    pub format: PixelFormat,
    /// A shared memory block containing the data of one or more image frames.
    data: Arc<GenericSharedMemory>,
    range: Range<usize>,
}

impl Image {
    pub fn new(
        width: u32,
        height: u32,
        data: Arc<GenericSharedMemory>,
        range: Range<usize>,
        format: PixelFormat,
    ) -> Self {
        Self {
            width,
            height,
            format,
            data,
            range,
        }
    }

    /// Return the bytes belonging to the first image frame.
    pub fn data(&self) -> &[u8] {
        &self.data[self.range.clone()]
    }
}

/// Messages towards the embedder.
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
    ShowSimpleDialog(WebViewId, SimpleDialogRequest),
    /// Request authentication for a load or navigation from the embedder.
    RequestAuthentication(
        WebViewId,
        ServoUrl,
        bool, /* for proxy */
        GenericSender<Option<AuthenticationResponse>>,
    ),
    /// Whether or not to allow a pipeline to load a url.
    AllowNavigationRequest(WebViewId, PipelineId, ServoUrl),
    /// Request to (un)register protocol handler by page content.
    AllowProtocolHandlerRequest(
        WebViewId,
        ProtocolHandlerUpdateRegistration,
        GenericSender<AllowOrDeny>,
    ),
    /// Whether or not to allow script to open a new tab/browser
    AllowOpeningWebView(
        WebViewId,
        GenericSender<Option<(WebViewId, ViewportDetails)>>,
    ),
    /// A webview was destroyed.
    WebViewClosed(WebViewId),
    /// A webview potentially gained focus for keyboard events.
    /// If the boolean value is false, the webiew could not be focused.
    WebViewFocused(WebViewId, bool),
    /// All webviews lost focus for keyboard events.
    WebViewBlurred,
    /// Wether or not to unload a document
    AllowUnload(WebViewId, GenericSender<AllowOrDeny>),
    /// Inform embedder to clear the clipboard
    ClearClipboard(WebViewId),
    /// Gets system clipboard contents
    GetClipboardText(WebViewId, IpcSender<Result<String, String>>),
    /// Sets system clipboard contents
    SetClipboardText(WebViewId, String),
    /// Changes the cursor.
    SetCursor(WebViewId, Cursor),
    /// A favicon was detected
    NewFavicon(WebViewId, Image),
    /// The history state has changed.
    HistoryChanged(WebViewId, Vec<ServoUrl>, usize),
    /// A history traversal operation completed.
    HistoryTraversalComplete(WebViewId, TraversalId),
    /// Get the device independent window rectangle.
    GetWindowRect(WebViewId, GenericSender<DeviceIndependentIntRect>),
    /// Get the device independent screen size and available size.
    GetScreenMetrics(WebViewId, GenericSender<ScreenMetrics>),
    /// Entered or exited fullscreen.
    NotifyFullscreenStateChanged(WebViewId, bool),
    /// The [`LoadStatus`] of the Given `WebView` has changed.
    NotifyLoadStatusChanged(WebViewId, LoadStatus),
    WebResourceRequested(
        Option<WebViewId>,
        WebResourceRequest,
        GenericSender<WebResourceResponseMsg>,
    ),
    /// A pipeline panicked. First string is the reason, second one is the backtrace.
    Panic(WebViewId, String, Option<String>),
    /// Open dialog to select bluetooth device.
    GetSelectedBluetoothDevice(WebViewId, Vec<String>, GenericSender<Option<String>>),
    /// Open file dialog to select files. Set boolean flag to true allows to select multiple files.
    SelectFiles(
        EmbedderControlId,
        FilePickerRequest,
        GenericSender<Option<Vec<PathBuf>>>,
    ),
    /// Open interface to request permission specified by prompt.
    PromptPermission(WebViewId, PermissionFeature, GenericSender<AllowOrDeny>),
    /// Report a complete sampled profile
    ReportProfile(Vec<u8>),
    /// Notifies the embedder about media session events
    /// (i.e. when there is metadata for the active media session, playback state changes...).
    MediaSessionEvent(WebViewId, MediaSessionEvent),
    /// Report the status of Devtools Server with a token that can be used to bypass the permission prompt.
    OnDevtoolsStarted(Result<u16, ()>, String),
    /// Ask the user to allow a devtools client to connect.
    RequestDevtoolsConnection(GenericSender<AllowOrDeny>),
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
    /// Request to display a form control to the embedder.
    ShowEmbedderControl(EmbedderControlId, DeviceIntRect, EmbedderControlRequest),
    /// Request to display a form control to the embedder.
    HideEmbedderControl(EmbedderControlId),
    /// Inform the embedding layer that a JavaScript evaluation has
    /// finished with the given result.
    FinishJavaScriptEvaluation(
        JavaScriptEvaluationId,
        Result<JSValue, JavaScriptEvaluationError>,
    ),
    /// Inform the embedding layer that a particular `InputEvent` was handled by Servo
    /// and the embedder can continue processing it, if necessary.
    InputEventHandled(WebViewId, InputEventId, InputEventResult),
}

impl Debug for EmbedderMsg {
    fn fmt(&self, formatter: &mut Formatter) -> Result<(), Error> {
        let string: &'static str = self.into();
        write!(formatter, "{string}")
    }
}

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
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
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

impl From<Theme> for PrefersColorScheme {
    fn from(value: Theme) -> Self {
        match value {
            Theme::Light => PrefersColorScheme::Light,
            Theme::Dark => PrefersColorScheme::Dark,
        }
    }
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
    pub icon_resource: Option<Arc<SharedRasterImage>>,
    /// The URL of a badge. The badge is used when there is no enough space to display the notification,
    /// such as on a mobile device's notification bar.
    pub badge_url: Option<ServoUrl>,
    /// Badge's raw image data and metadata.
    pub badge_resource: Option<Arc<SharedRasterImage>>,
    /// The URL of an image. The image will be displayed as part of the notification.
    pub image_url: Option<ServoUrl>,
    /// Image's raw image data and metadata.
    pub image_resource: Option<Arc<SharedRasterImage>>,
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
    pub icon_resource: Option<Arc<SharedRasterImage>>,
}

/// Information about a `WebView`'s screen geometry and offset. This is used
/// for the [Screen](https://drafts.csswg.org/cssom-view/#the-screen-interface) CSSOM APIs
/// and `window.screenLeft` / `window.screenX` / `window.screenTop` / `window.screenY` /
/// `window.moveBy`/ `window.resizeBy` / `window.outerWidth` / `window.outerHeight` /
/// `window.screen.availHeight` / `window.screen.availWidth`.
#[derive(Clone, Copy, Debug, Default)]
pub struct ScreenGeometry {
    /// The size of the screen in device pixels. This will be converted to
    /// CSS pixels based on the pixel scaling of the `WebView`.
    pub size: DeviceIntSize,
    /// The available size of the screen in device pixels for the purposes of
    /// the `window.screen.availHeight` / `window.screen.availWidth`. This is the size
    /// available for web content on the screen, and should be `size` minus any system
    /// toolbars, docks, and interface elements. This will be converted to
    /// CSS pixels based on the pixel scaling of the `WebView`.
    pub available_size: DeviceIntSize,
    /// The rectangle the `WebView`'s containing window (including OS decorations)
    /// in device pixels for the purposes of the
    /// `window.screenLeft`, `window.outerHeight` and similar APIs.
    /// This will be converted to CSS pixels based on the pixel scaling of the `WebView`.
    pub window_rect: DeviceIntRect,
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

#[expect(unsafe_code)]
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

/// The result of a hit test in `Paint`.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct PaintHitTestResult {
    /// The pipeline id of the resulting item.
    pub pipeline_id: PipelineId,

    /// The hit test point in the item's viewport.
    pub point_in_viewport: Point2D<f32, CSSPixel>,

    /// The [`ExternalScrollId`] of the scroll tree node associated with this hit test item.
    pub external_scroll_id: ExternalScrollId,
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

/// A sequence number generated by a script thread for its pipelines. The
/// constellation attaches the target pipeline's last seen `FocusSequenceNumber`
/// to every focus-related message it sends.
///
/// This is used to resolve the inconsistency that occurs due to bidirectional
/// focus state synchronization and provide eventual consistency. Example:
///
/// ```text
/// script                            constellation
/// -----------------------------------------------------------------------
/// send ActivateDocument ----------> receive ActivateDocument
///                             ,---- send FocusDocument
///                             |
/// focus an iframe             |
/// send Focus -----------------|---> receive Focus
///                             |     focus the iframe's content document
/// receive FocusDocument <-----'     send FocusDocument to the content pipeline --> ...
/// unfocus the iframe
/// focus the document
///
/// Final state:                      Final state:
///  the iframe is not focused         the iframe is focused
/// ```
///
/// When the above sequence completes, from the script thread's point of view,
/// the iframe is unfocused, but from the constellation's point of view, the
/// iframe is still focused.
///
/// This inconsistency can be resolved by associating a sequence number to each
/// message. Whenever a script thread initiates a focus operation, it generates
/// and sends a brand new sequence number. The constellation attaches the
/// last-received sequence number to each message it sends. This way, the script
/// thread can discard out-dated incoming focus messages, and eventually, all
/// actors converge to the consistent state which is determined based on the
/// last focus message received by the constellation.
///
/// ```text
/// script                            constellation
/// -----------------------------------------------------------------------
/// send ActivateDocument ----------> receive ActivateDocument
///                             ,---- send FocusDocument (0)
///                             |
/// seq_number += 1             |
/// focus an iframe             |
/// send Focus (1) -------------|---> receive Focus (1)
///                             |     focus the iframe's content document
/// receive FocusDocument (0) <-'     send FocusDocument to the content pipeline --> ...
/// ignore it because 0 < 1
///
/// Final state:                      Final state:
///  the iframe is focused             the iframe is focused
/// ```
#[derive(
    Clone,
    Copy,
    Debug,
    Default,
    Deserialize,
    Eq,
    Hash,
    MallocSizeOf,
    PartialEq,
    Serialize,
    PartialOrd,
)]
pub struct FocusSequenceNumber(pub u64);

impl Display for FocusSequenceNumber {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        Display::fmt(&self.0, f)
    }
}

/// An identifier for a particular JavaScript evaluation that is used to track the
/// evaluation from the embedding layer to the script layer and then back.
#[derive(Clone, Copy, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct JavaScriptEvaluationId(pub usize);

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub enum JSValue {
    Undefined,
    Null,
    Boolean(bool),
    Number(f64),
    String(String),
    Element(String),
    ShadowRoot(String),
    Frame(String),
    Window(String),
    Array(Vec<JSValue>),
    Object(HashMap<String, JSValue>),
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct JavaScriptErrorInfo {
    pub message: String,
    pub filename: String,
    pub stack: Option<String>,
    pub line_number: u64,
    pub column: u64,
}

/// Indicates the reason that JavaScript evaluation failed due serializing issues the
/// result of the evaluation.
#[derive(Clone, Debug, Deserialize, EnumMessage, PartialEq, Serialize)]
pub enum JavaScriptEvaluationResultSerializationError {
    /// Serialization could not complete because a JavaScript value contained a detached
    /// shadow root according to <https://w3c.github.io/webdriver/#dfn-internal-json-clone>.
    DetachedShadowRoot,
    /// Serialization could not complete because a JavaScript value contained a "stale"
    /// element reference according to <https://w3c.github.io/webdriver/#dfn-get-a-known-element>.
    StaleElementReference,
    /// Serialization could not complete because a JavaScript value of an unknown type
    /// was encountered.
    UnknownType,
    /// This is a catch all for other kinds of errors that can happen during JavaScript value
    /// serialization. For instances where this can happen, see:
    /// <https://w3c.github.io/webdriver/#dfn-clone-an-object>.
    OtherJavaScriptError,
}

/// An error that happens when trying to evaluate JavaScript on a `WebView`.
#[derive(Clone, Debug, Deserialize, EnumMessage, PartialEq, Serialize)]
pub enum JavaScriptEvaluationError {
    /// The `Document` of frame that the script was going to execute in no longer exists.
    DocumentNotFound,
    /// The script could not be compiled.
    CompilationFailure,
    /// The script could not be evaluated.
    EvaluationFailure(Option<JavaScriptErrorInfo>),
    /// An internal Servo error prevented the JavaSript evaluation from completing properly.
    /// This indicates a bug in Servo.
    InternalError,
    /// The `WebView` on which this evaluation request was triggered is not ready. This might
    /// happen if the `WebView`'s `Document` is changing due to ongoing load events, for instance.
    WebViewNotReady,
    /// The script executed successfully, but Servo could not serialize the JavaScript return
    /// value into a [`JSValue`].
    SerializationError(JavaScriptEvaluationResultSerializationError),
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub enum ScreenshotCaptureError {
    /// The screenshot request failed to read the screenshot image from the `WebView`'s
    /// `RenderingContext`.
    CouldNotReadImage,
    /// The WebView that this screenshot request was made for no longer exists.
    WebViewDoesNotExist,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub struct RgbColor {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
}

/// A Script to Embedder Channel
#[derive(Clone, Debug, Deserialize, MallocSizeOf, Serialize)]
pub struct ScriptToEmbedderChan(GenericCallback<EmbedderMsg>);

impl ScriptToEmbedderChan {
    /// Create a new Channel allowing script to send messages to the Embedder
    pub fn new(
        embedder_chan: Sender<EmbedderMsg>,
        waker: Box<dyn EventLoopWaker>,
    ) -> ScriptToEmbedderChan {
        let embedder_callback = GenericCallback::new(move |embedder_msg| {
            let msg = match embedder_msg {
                Ok(embedder_msg) => embedder_msg,
                Err(err) => {
                    log::warn!("Script to Embedder message error: {err}");
                    return;
                },
            };
            let _ = embedder_chan.send(msg);
            waker.wake();
        })
        .expect("Failed to create channel");
        ScriptToEmbedderChan(embedder_callback)
    }

    /// Send a message to and wake the Embedder
    pub fn send(&self, msg: EmbedderMsg) -> SendResult {
        self.0.send(msg)
    }
}

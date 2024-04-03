/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Abstract windowing methods. The concrete implementations of these can be found in `platform/`.

use std::fmt::{Debug, Error, Formatter};
use std::time::Duration;

use embedder_traits::{EmbedderProxy, EventLoopWaker};
use euclid::Scale;
use gfx::rendering_context::RenderingContext;
use keyboard_types::KeyboardEvent;
use libc::c_void;
use msg::constellation_msg::{PipelineId, TopLevelBrowsingContextId, TraversalDirection};
use script_traits::{
    GamepadEvent, MediaSessionActionType, MouseButton, TouchEventType, TouchId, WheelDelta,
};
use servo_geometry::DeviceIndependentPixel;
use servo_url::ServoUrl;
use style_traits::DevicePixel;
use webrender_api::units::{DeviceIntPoint, DeviceIntRect, DeviceIntSize, DevicePoint, DeviceRect};
use webrender_api::ScrollLocation;

#[derive(Clone)]
pub enum MouseWindowEvent {
    Click(MouseButton, DevicePoint),
    MouseDown(MouseButton, DevicePoint),
    MouseUp(MouseButton, DevicePoint),
}

/// Various debug and profiling flags that WebRender supports.
#[derive(Clone)]
pub enum WebRenderDebugOption {
    Profiler,
    TextureCacheDebug,
    RenderTargetDebug,
}

/// Events that the embedder sends to Servo, including events from the windowing system.
#[derive(Clone)]
pub enum EmbedderEvent {
    /// Sent when no message has arrived, but the event loop was kicked for some reason (perhaps
    /// by another Servo subsystem).
    ///
    /// FIXME(pcwalton): This is kind of ugly and may not work well with multiprocess Servo.
    /// It's possible that this should be something like
    /// `CompositorMessageWindowEvent(compositor_thread::Msg)` instead.
    Idle,
    /// Sent when part of the window is marked dirty and needs to be redrawn. Before sending this
    /// message, the window must make the same GL context as in `PrepareRenderingEvent` current.
    Refresh,
    /// Sent when the window is resized.
    WindowResize,
    /// Sent when a navigation request from script is allowed/refused.
    AllowNavigationResponse(PipelineId, bool),
    /// Sent when a new URL is to be loaded.
    LoadUrl(TopLevelBrowsingContextId, ServoUrl),
    /// Sent when a mouse hit test is to be performed.
    MouseWindowEventClass(MouseWindowEvent),
    /// Sent when a mouse move.
    MouseWindowMoveEventClass(DevicePoint),
    /// Touch event: type, identifier, point
    Touch(TouchEventType, TouchId, DevicePoint),
    /// Sent when user moves the mouse wheel.
    Wheel(WheelDelta, DevicePoint),
    /// Sent when the user scrolls. The first point is the delta and the second point is the
    /// origin.
    Scroll(ScrollLocation, DeviceIntPoint, TouchEventType),
    /// Sent when the user zooms.
    Zoom(f32),
    /// Simulated "pinch zoom" gesture for non-touch platforms (e.g. ctrl-scrollwheel).
    PinchZoom(f32),
    /// Sent when the user resets zoom to default.
    ResetZoom,
    /// Sent when the user uses chrome navigation (i.e. backspace or shift-backspace).
    Navigation(TopLevelBrowsingContextId, TraversalDirection),
    /// Sent when the user quits the application
    Quit,
    /// Sent when the user exits from fullscreen mode
    ExitFullScreen(TopLevelBrowsingContextId),
    /// Sent when a key input state changes
    Keyboard(KeyboardEvent),
    /// Sent when Ctr+R/Apple+R is called to reload the current page.
    Reload(TopLevelBrowsingContextId),
    /// Create a new top-level browsing context.
    NewWebView(ServoUrl, TopLevelBrowsingContextId),
    /// Close a top-level browsing context.
    CloseWebView(TopLevelBrowsingContextId),
    /// Panic a top-level browsing context.
    SendError(Option<TopLevelBrowsingContextId>, String),
    /// Move and/or resize a webview to the given rect.
    MoveResizeWebView(TopLevelBrowsingContextId, DeviceRect),
    /// Start painting a webview, and optionally stop painting all others.
    ShowWebView(TopLevelBrowsingContextId, bool),
    /// Stop painting a webview.
    HideWebView(TopLevelBrowsingContextId),
    /// Start painting a webview on top of all others, and optionally stop painting all others.
    RaiseWebViewToTop(TopLevelBrowsingContextId, bool),
    /// Make a webview focused.
    FocusWebView(TopLevelBrowsingContextId),
    /// Make none of the webviews focused.
    BlurWebView,
    /// Toggles a debug flag in WebRender
    ToggleWebRenderDebug(WebRenderDebugOption),
    /// Capture current WebRender
    CaptureWebRender,
    /// Clear the network cache.
    ClearCache,
    /// Toggle sampling profiler with the given sampling rate and max duration.
    ToggleSamplingProfiler(Duration, Duration),
    /// Sent when the user triggers a media action through the UA exposed media UI
    /// (play, pause, seek, etc.).
    MediaSessionAction(MediaSessionActionType),
    /// Set whether to use less resources, by stopping animations and running timers at a heavily limited rate.
    SetWebViewThrottled(TopLevelBrowsingContextId, bool),
    /// Virtual keyboard was dismissed
    IMEDismissed,
    /// Sent on platforms like Android where the native widget surface can be
    /// automatically destroyed by the system, for example when the app
    /// is sent to background.
    InvalidateNativeSurface,
    /// Sent on platforms like Android where system recreates a new surface for
    /// the native widget when it is brough back to foreground. This event
    /// carries the pointer to the native widget and its new size.
    ReplaceNativeSurface(*mut c_void, DeviceIntSize),
    /// Sent when new Gamepad information is available.
    Gamepad(GamepadEvent),
}

impl Debug for EmbedderEvent {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        match *self {
            EmbedderEvent::Idle => write!(f, "Idle"),
            EmbedderEvent::Refresh => write!(f, "Refresh"),
            EmbedderEvent::WindowResize => write!(f, "Resize"),
            EmbedderEvent::Keyboard(..) => write!(f, "Keyboard"),
            EmbedderEvent::AllowNavigationResponse(..) => write!(f, "AllowNavigationResponse"),
            EmbedderEvent::LoadUrl(..) => write!(f, "LoadUrl"),
            EmbedderEvent::MouseWindowEventClass(..) => write!(f, "Mouse"),
            EmbedderEvent::MouseWindowMoveEventClass(..) => write!(f, "MouseMove"),
            EmbedderEvent::Touch(..) => write!(f, "Touch"),
            EmbedderEvent::Wheel(..) => write!(f, "Wheel"),
            EmbedderEvent::Scroll(..) => write!(f, "Scroll"),
            EmbedderEvent::Zoom(..) => write!(f, "Zoom"),
            EmbedderEvent::PinchZoom(..) => write!(f, "PinchZoom"),
            EmbedderEvent::ResetZoom => write!(f, "ResetZoom"),
            EmbedderEvent::Navigation(..) => write!(f, "Navigation"),
            EmbedderEvent::Quit => write!(f, "Quit"),
            EmbedderEvent::Reload(..) => write!(f, "Reload"),
            EmbedderEvent::NewWebView(_, TopLevelBrowsingContextId(webview_id)) => {
                write!(f, "NewWebView({webview_id:?})")
            },
            EmbedderEvent::SendError(..) => write!(f, "SendError"),
            EmbedderEvent::CloseWebView(TopLevelBrowsingContextId(webview_id)) => {
                write!(f, "CloseWebView({webview_id:?})")
            },
            EmbedderEvent::MoveResizeWebView(webview_id, _) => {
                write!(f, "MoveResizeWebView({webview_id:?})")
            },
            EmbedderEvent::ShowWebView(TopLevelBrowsingContextId(webview_id), hide_others) => {
                write!(f, "ShowWebView({webview_id:?}, {hide_others})")
            },
            EmbedderEvent::HideWebView(TopLevelBrowsingContextId(webview_id)) => {
                write!(f, "HideWebView({webview_id:?})")
            },
            EmbedderEvent::RaiseWebViewToTop(
                TopLevelBrowsingContextId(webview_id),
                hide_others,
            ) => {
                write!(f, "RaiseWebViewToTop({webview_id:?}, {hide_others})")
            },
            EmbedderEvent::FocusWebView(TopLevelBrowsingContextId(webview_id)) => {
                write!(f, "FocusWebView({webview_id:?})")
            },
            EmbedderEvent::BlurWebView => write!(f, "BlurWebView"),
            EmbedderEvent::ToggleWebRenderDebug(..) => write!(f, "ToggleWebRenderDebug"),
            EmbedderEvent::CaptureWebRender => write!(f, "CaptureWebRender"),
            EmbedderEvent::ToggleSamplingProfiler(..) => write!(f, "ToggleSamplingProfiler"),
            EmbedderEvent::ExitFullScreen(..) => write!(f, "ExitFullScreen"),
            EmbedderEvent::MediaSessionAction(..) => write!(f, "MediaSessionAction"),
            EmbedderEvent::SetWebViewThrottled(..) => write!(f, "SetWebViewThrottled"),
            EmbedderEvent::IMEDismissed => write!(f, "IMEDismissed"),
            EmbedderEvent::ClearCache => write!(f, "ClearCache"),
            EmbedderEvent::InvalidateNativeSurface => write!(f, "InvalidateNativeSurface"),
            EmbedderEvent::ReplaceNativeSurface(..) => write!(f, "ReplaceNativeSurface"),
            EmbedderEvent::Gamepad(..) => write!(f, "Gamepad"),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum AnimationState {
    Idle,
    Animating,
}

// TODO: this trait assumes that the window is responsible
// for creating the GL context, making it current, buffer
// swapping, etc. Really that should all be done by surfman.
pub trait WindowMethods {
    /// Get the coordinates of the native window, the screen and the framebuffer.
    fn get_coordinates(&self) -> EmbedderCoordinates;
    /// Set whether the application is currently animating.
    /// Typically, when animations are active, the window
    /// will want to avoid blocking on UI events, and just
    /// run the event loop at the vsync interval.
    fn set_animation_state(&self, _state: AnimationState);
    /// Get the [`RenderingContext`] of this Window.
    fn rendering_context(&self) -> RenderingContext;
}

pub trait EmbedderMethods {
    /// Returns a thread-safe object to wake up the window's event loop.
    fn create_event_loop_waker(&mut self) -> Box<dyn EventLoopWaker>;

    /// Register services with a WebXR Registry.
    fn register_webxr(&mut self, _: &mut webxr::MainThreadRegistry, _: EmbedderProxy) {}

    /// Returns the user agent string to report in network requests.
    fn get_user_agent_string(&self) -> Option<String> {
        None
    }
}

#[derive(Clone, Copy, Debug)]
pub struct EmbedderCoordinates {
    /// The pixel density of the display.
    pub hidpi_factor: Scale<f32, DeviceIndependentPixel, DevicePixel>,
    /// Size of the screen.
    pub screen: DeviceIntSize,
    /// Size of the available screen space (screen without toolbars and docks).
    pub screen_avail: DeviceIntSize,
    /// Size of the native window.
    pub window: (DeviceIntSize, DeviceIntPoint),
    /// Size of the GL buffer in the window.
    pub framebuffer: DeviceIntSize,
    /// Coordinates of the document within the framebuffer.
    pub viewport: DeviceIntRect,
}

impl EmbedderCoordinates {
    /// Get the unflipped viewport rectangle for use with the WebRender API.
    pub fn get_viewport(&self) -> DeviceIntRect {
        self.viewport
    }

    /// Flip the given rect.
    /// This should be used when drawing directly to the framebuffer with OpenGL commands.
    pub fn flip_rect(&self, rect: &DeviceIntRect) -> DeviceIntRect {
        let mut result = *rect;
        result.min.y = self.framebuffer.height - result.min.y - result.size().height;
        result
    }

    /// Get the flipped viewport rectangle.
    /// This should be used when drawing directly to the framebuffer with OpenGL commands.
    pub fn get_flipped_viewport(&self) -> DeviceIntRect {
        self.flip_rect(&self.get_viewport())
    }
}

/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Abstract windowing methods. The concrete implementations of these can be found in `platform/`.

use embedder_traits::EventLoopWaker;
use euclid::TypedScale;
#[cfg(feature = "gleam")]
use gleam::gl;
use ipc_channel::ipc::IpcSender;
use msg::constellation_msg::{Key, KeyModifiers, KeyState, TopLevelBrowsingContextId, TraversalDirection};
use script_traits::{MouseButton, TouchEventType, TouchId};
use servo_geometry::{DeviceIndependentPixel, DeviceUintLength};
use servo_url::ServoUrl;
use std::fmt::{Debug, Error, Formatter};
#[cfg(feature = "gleam")]
use std::rc::Rc;
use style_traits::DevicePixel;
use webrender_api::{DeviceIntPoint, DevicePoint, DeviceUintSize, DeviceUintRect, ScrollLocation};

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

/// Events that the windowing system sends to Servo.
#[derive(Clone)]
pub enum WindowEvent {
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
    Resize,
    /// Sent when a new URL is to be loaded.
    LoadUrl(TopLevelBrowsingContextId, ServoUrl),
    /// Sent when a mouse hit test is to be performed.
    MouseWindowEventClass(MouseWindowEvent),
    /// Sent when a mouse move.
    MouseWindowMoveEventClass(DevicePoint),
    /// Touch event: type, identifier, point
    Touch(TouchEventType, TouchId, DevicePoint),
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
    /// Sent when a key input state changes
    KeyEvent(Option<char>, Key, KeyState, KeyModifiers),
    /// Sent when Ctr+R/Apple+R is called to reload the current page.
    Reload(TopLevelBrowsingContextId),
    /// Create a new top level browsing context
    NewBrowser(ServoUrl, IpcSender<TopLevelBrowsingContextId>),
    /// Close a top level browsing context
    CloseBrowser(TopLevelBrowsingContextId),
    /// Panic a top level browsing context.
    SendError(Option<TopLevelBrowsingContextId>, String),
    /// Make a top level browsing context visible, hiding the previous
    /// visible one.
    SelectBrowser(TopLevelBrowsingContextId),
    /// Toggles a debug flag in WebRender
    ToggleWebRenderDebug(WebRenderDebugOption),
    /// Capture current WebRender
    CaptureWebRender,
}

impl Debug for WindowEvent {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        match *self {
            WindowEvent::Idle => write!(f, "Idle"),
            WindowEvent::Refresh => write!(f, "Refresh"),
            WindowEvent::Resize => write!(f, "Resize"),
            WindowEvent::KeyEvent(..) => write!(f, "Key"),
            WindowEvent::LoadUrl(..) => write!(f, "LoadUrl"),
            WindowEvent::MouseWindowEventClass(..) => write!(f, "Mouse"),
            WindowEvent::MouseWindowMoveEventClass(..) => write!(f, "MouseMove"),
            WindowEvent::Touch(..) => write!(f, "Touch"),
            WindowEvent::Scroll(..) => write!(f, "Scroll"),
            WindowEvent::Zoom(..) => write!(f, "Zoom"),
            WindowEvent::PinchZoom(..) => write!(f, "PinchZoom"),
            WindowEvent::ResetZoom => write!(f, "ResetZoom"),
            WindowEvent::Navigation(..) => write!(f, "Navigation"),
            WindowEvent::Quit => write!(f, "Quit"),
            WindowEvent::Reload(..) => write!(f, "Reload"),
            WindowEvent::NewBrowser(..) => write!(f, "NewBrowser"),
            WindowEvent::SendError(..) => write!(f, "SendError"),
            WindowEvent::CloseBrowser(..) => write!(f, "CloseBrowser"),
            WindowEvent::SelectBrowser(..) => write!(f, "SelectBrowser"),
            WindowEvent::ToggleWebRenderDebug(..) => write!(f, "ToggleWebRenderDebug"),
            WindowEvent::CaptureWebRender => write!(f, "CaptureWebRender"),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum AnimationState {
    Idle,
    Animating,
}

pub trait WindowMethods {
    /// Presents the window to the screen (perhaps by page flipping).
    fn present(&self);
    /// Requests that the window system prepare a composite. Typically this will involve making
    /// some type of platform-specific graphics context current. Returns true if the composite may
    /// proceed and false if it should not.
    fn prepare_for_composite(&self, width: DeviceUintLength, height: DeviceUintLength) -> bool;
    /// Return the GL function pointer trait.
    #[cfg(feature = "gleam")]
    fn gl(&self) -> Rc<gl::Gl>;
    /// Returns a thread-safe object to wake up the window's event loop.
    fn create_event_loop_waker(&self) -> Box<EventLoopWaker>;
    /// Get the coordinates of the native window, the screen and the framebuffer.
    fn get_coordinates(&self) -> EmbedderCoordinates;
    /// Does this window support a clipboard
    fn supports_clipboard(&self) -> bool;
    /// Set whether the application is currently animating.
    /// Typically, when animations are active, the window
    /// will want to avoid blocking on UI events, and just
    /// run the event loop at the vsync interval.
    fn set_animation_state(&self, _state: AnimationState);
}

#[derive(Clone, Copy, Debug)]
pub struct EmbedderCoordinates {
    /// The pixel density of the display.
    pub hidpi_factor: TypedScale<f32, DeviceIndependentPixel, DevicePixel>,
    /// Size of the screen.
    pub screen: DeviceUintSize,
    /// Size of the available screen space (screen without toolbars and docks).
    pub screen_avail: DeviceUintSize,
    /// Size of the native window.
    pub window: (DeviceUintSize, DeviceIntPoint),
    /// Size of the GL buffer in the window.
    pub framebuffer: DeviceUintSize,
    /// Coordinates of the document within the framebuffer.
    pub viewport: DeviceUintRect,
}

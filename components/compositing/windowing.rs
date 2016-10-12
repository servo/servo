/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Abstract windowing methods. The concrete implementations of these can be found in `platform/`.

use compositor_thread::{CompositorProxy, CompositorReceiver};
use euclid::{Point2D, Size2D};
use euclid::point::TypedPoint2D;
use euclid::scale_factor::ScaleFactor;
use euclid::size::TypedSize2D;
use gfx_traits::DevicePixel;
use msg::constellation_msg::{Key, KeyModifiers, KeyState};
use net_traits::net_error_list::NetError;
use script_traits::{MouseButton, TouchEventType, TouchId, TouchpadPressurePhase};
use std::fmt::{Debug, Error, Formatter};
use style_traits::cursor::Cursor;
use url::Url;
use util::geometry::ScreenPx;

#[derive(Clone)]
pub enum MouseWindowEvent {
    Click(MouseButton, TypedPoint2D<f32, DevicePixel>),
    MouseDown(MouseButton, TypedPoint2D<f32, DevicePixel>),
    MouseUp(MouseButton, TypedPoint2D<f32, DevicePixel>),
}

#[derive(Clone)]
pub enum WindowNavigateMsg {
    Forward,
    Back,
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
    /// Sent to initialize the GL context. The windowing system must have a valid, current GL
    /// context when this message is sent.
    InitializeCompositing,
    /// Sent when the window is resized.
    Resize(TypedSize2D<u32, DevicePixel>),
    /// Touchpad Pressure
    TouchpadPressure(TypedPoint2D<f32, DevicePixel>, f32, TouchpadPressurePhase),
    /// Sent when you want to override the viewport.
    Viewport(TypedPoint2D<u32, DevicePixel>, TypedSize2D<u32, DevicePixel>),
    /// Sent when a new URL is to be loaded.
    LoadUrl(String),
    /// Sent when a mouse hit test is to be performed.
    MouseWindowEventClass(MouseWindowEvent),
    /// Sent when a mouse move.
    MouseWindowMoveEventClass(TypedPoint2D<f32, DevicePixel>),
    /// Touch event: type, identifier, point
    Touch(TouchEventType, TouchId, TypedPoint2D<f32, DevicePixel>),
    /// Sent when the user scrolls. The first point is the delta and the second point is the
    /// origin.
    Scroll(TypedPoint2D<f32, DevicePixel>, TypedPoint2D<i32, DevicePixel>, TouchEventType),
    /// Sent when the user zooms.
    Zoom(f32),
    /// Simulated "pinch zoom" gesture for non-touch platforms (e.g. ctrl-scrollwheel).
    PinchZoom(f32),
    /// Sent when the user resets zoom to default.
    ResetZoom,
    /// Sent when the user uses chrome navigation (i.e. backspace or shift-backspace).
    Navigation(WindowNavigateMsg),
    /// Sent when the user quits the application
    Quit,
    /// Sent when a key input state changes
    KeyEvent(Option<char>, Key, KeyState, KeyModifiers),
    /// Sent when Ctr+R/Apple+R is called to reload the current page.
    Reload,
}

impl Debug for WindowEvent {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        match *self {
            WindowEvent::Idle => write!(f, "Idle"),
            WindowEvent::Refresh => write!(f, "Refresh"),
            WindowEvent::InitializeCompositing => write!(f, "InitializeCompositing"),
            WindowEvent::Resize(..) => write!(f, "Resize"),
            WindowEvent::TouchpadPressure(..) => write!(f, "TouchpadPressure"),
            WindowEvent::Viewport(..) => write!(f, "Viewport"),
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
            WindowEvent::Reload => write!(f, "Reload"),
        }
    }
}

pub trait WindowMethods {
    /// Returns the size of the window in hardware pixels.
    fn framebuffer_size(&self) -> TypedSize2D<u32, DevicePixel>;
    /// Returns the size of the window in density-independent "px" units.
    fn size(&self) -> TypedSize2D<f32, ScreenPx>;
    /// Presents the window to the screen (perhaps by page flipping).
    fn present(&self);

    /// Return the size of the window with head and borders and position of the window values
    fn client_window(&self) -> (Size2D<u32>, Point2D<i32>);
    /// Set the size inside of borders and head
    fn set_inner_size(&self, size: Size2D<u32>);
    /// Set the window position
    fn set_position(&self, point: Point2D<i32>);

    /// Sets the page title for the current page.
    fn set_page_title(&self, title: Option<String>);
    /// Sets the load data for the current page.
    fn set_page_url(&self, url: Url);
    /// Called when the browser chrome should display a status message.
    fn status(&self, Option<String>);
    /// Called when the browser has started loading a frame.
    fn load_start(&self, back: bool, forward: bool);
    /// Called when the browser is done loading a frame.
    fn load_end(&self, back: bool, forward: bool, root: bool);
    /// Called when the browser encounters an error while loading a URL
    fn load_error(&self, code: NetError, url: String);
    /// Called when the <head> tag has finished parsing
    fn head_parsed(&self);

    /// Returns the scale factor of the system (device pixels / screen pixels).
    fn scale_factor(&self) -> ScaleFactor<f32, ScreenPx, DevicePixel>;

    /// Creates a channel to the compositor. The dummy parameter is needed because we don't have
    /// UFCS in Rust yet.
    ///
    /// This is part of the windowing system because its implementation often involves OS-specific
    /// magic to wake the up window's event loop.
    fn create_compositor_channel(&self) -> (Box<CompositorProxy + Send>, Box<CompositorReceiver>);

    /// Requests that the window system prepare a composite. Typically this will involve making
    /// some type of platform-specific graphics context current. Returns true if the composite may
    /// proceed and false if it should not.
    fn prepare_for_composite(&self, width: usize, height: usize) -> bool;

    /// Sets the cursor to be used in the window.
    fn set_cursor(&self, cursor: Cursor);

    /// Process a key event.
    fn handle_key(&self, ch: Option<char>, key: Key, mods: KeyModifiers);

    /// Does this window support a clipboard
    fn supports_clipboard(&self) -> bool;

    /// Add a favicon
    fn set_favicon(&self, url: Url);
}

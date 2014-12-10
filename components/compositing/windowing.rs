/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Abstract windowing methods. The concrete implementations of these can be found in `platform/`.

use main_thread::MainThreadProxy;

use geom::point::TypedPoint2D;
use geom::scale_factor::ScaleFactor;
use geom::size::TypedSize2D;
use layers::geometry::DevicePixel;
use layers::platform::surface::NativeGraphicsMetadata;
use servo_msg::constellation_msg::{Key, KeyState, KeyModifiers};
use servo_msg::compositor_msg::{ReadyState, PaintState};
use servo_util::geometry::ScreenPx;
use std::fmt::{FormatError, Formatter, Show};
use std::rc::Rc;

pub enum MouseWindowEvent {
    MouseWindowClickEvent(uint, TypedPoint2D<DevicePixel, f32>),
    MouseWindowMouseDownEvent(uint, TypedPoint2D<DevicePixel, f32>),
    MouseWindowMouseUpEvent(uint, TypedPoint2D<DevicePixel, f32>),
}

pub enum WindowNavigateMsg {
    Forward,
    Back,
}

/// Events that the windowing system sends to Servo.
pub enum WindowEvent {
    /// Sent when no message has arrived, but the event loop was kicked for some reason (perhaps
    /// by another Servo subsystem).
    ///
    /// FIXME(pcwalton): This is kind of ugly and may not work well with multiprocess Servo.
    /// It's possible that this should be something like
    /// `CompositorMessageWindowEvent(compositor_task::Msg)` instead.
    IdleWindowEvent,
    /// Sent when part of the window is marked dirty and needs to be redrawn.
    RefreshWindowEvent,
    /// Sent when the window is resized.
    ResizeWindowEvent(TypedSize2D<DevicePixel, uint>),
    /// Sent when a new URL is to be loaded.
    LoadUrlWindowEvent(String),
    /// Sent when a mouse hit test is to be performed.
    MouseWindowEventClass(MouseWindowEvent),
    /// Sent when a mouse move.
    MouseWindowMoveEventClass(TypedPoint2D<DevicePixel, f32>),
    /// Sent when the user scrolls. Includes the current cursor position.
    ScrollWindowEvent(TypedPoint2D<DevicePixel, f32>, TypedPoint2D<DevicePixel, i32>),
    /// Sent when the user zooms.
    ZoomWindowEvent(f32),
    /// Simulated "pinch zoom" gesture for non-touch platforms (e.g. ctrl-scrollwheel).
    PinchZoomWindowEvent(f32),
    /// Sent when the user uses chrome navigation (i.e. backspace or shift-backspace).
    NavigationWindowEvent(WindowNavigateMsg),
    /// Sent when the user quits the application.
    QuitWindowEvent,
    /// Sent when a key input state changes.
    KeyEvent(Key, KeyState, KeyModifiers),
    /// Informs the main thread that the ready state has changed.
    SetReadyStateWindowEvent(ReadyState),
    /// Informs the main thread that the paint state has changed.
    SetPaintStateWindowEvent(PaintState),
    /// Sent when a synchronous repaint is needed (often when a resize event occurs). The main
    /// thread will not return from processing until the repaint happens.
    SynchronousRepaintWindowEvent,
}

impl Show for WindowEvent {
    fn fmt(&self, f: &mut Formatter) -> Result<(),FormatError> {
        match *self {
            IdleWindowEvent => write!(f, "Idle"),
            RefreshWindowEvent => write!(f, "Refresh"),
            ResizeWindowEvent(..) => write!(f, "Resize"),
            KeyEvent(..) => write!(f, "Key"),
            LoadUrlWindowEvent(..) => write!(f, "LoadUrl"),
            MouseWindowEventClass(..) => write!(f, "Mouse"),
            MouseWindowMoveEventClass(..) => write!(f, "MouseMove"),
            ScrollWindowEvent(..) => write!(f, "Scroll"),
            ZoomWindowEvent(..) => write!(f, "Zoom"),
            PinchZoomWindowEvent(..) => write!(f, "PinchZoom"),
            NavigationWindowEvent(..) => write!(f, "Navigation"),
            SetReadyStateWindowEvent(..) => write!(f, "SetReadyState"),
            SetPaintStateWindowEvent(..) => write!(f, "SetPaintState"),
            SynchronousRepaintWindowEvent => write!(f, "SynchronousRepaintWindowEvent"),
            QuitWindowEvent => write!(f, "Quit"),
        }
    }
}

pub trait WindowMethods {
    /// Returns the size of the window in hardware pixels.
    fn framebuffer_size(&self) -> TypedSize2D<DevicePixel, uint>;
    /// Returns the size of the window in density-independent "px" units.
    fn size(&self) -> TypedSize2D<ScreenPx, f32>;
    /// Presents the window to the screen (perhaps by page flipping).
    fn present(&self);

    /// Sets the ready state of the current page.
    fn set_ready_state(&self, ready_state: ReadyState);
    /// Sets the paint state of the current page.
    fn set_paint_state(&self, paint_state: PaintState);

    /// Returns the hidpi factor of the monitor.
    fn hidpi_factor(&self) -> ScaleFactor<ScreenPx, DevicePixel, f32>;

    /// Gets the OS native graphics information for this window.
    fn native_metadata(&self) -> NativeGraphicsMetadata;

    /// Returns a *compositor support object*—a thread-safe object that provides the native
    /// graphics context to the compositing thread.
    fn create_compositor_support(&self) -> Box<CompositorSupport + Send>;

    /// Creates a channel on which events can be sent to the main thread. The dummy parameter is
    /// needed because we don't have UFCS in Rust yet.
    ///
    /// This is part of the windowing system because its implementation often involves OS-specific
    /// magic to wake the up window's event loop.
    fn create_main_thread_proxy(_: &Option<Rc<Self>>, sender: Sender<WindowEvent>)
                                -> Box<MainThreadProxy + Send>;
}

/// An thread-safe object that provides support for the compositor. Typically this wraps the native
/// OS graphics context—`CGLContextObj` on Mac or `EGLContext` on EGL-compliant platforms, for
/// example.
pub trait CompositorSupport {
    /// Initializes compositing. Typically this involves making some kind of OS graphics context
    /// current.
    fn initialize(&mut self);
    /// Presents the current rendered contents to the screen, perhaps by performing a page flip.
    fn present(&mut self);
}


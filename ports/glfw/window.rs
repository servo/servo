/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! A windowing implementation using GLFW.

use NestedEventLoopListener;

use compositing::compositor_task::{mod, CompositorProxy, CompositorReceiver};
use compositing::windowing::WindowNavigateMsg;
use compositing::windowing::{MouseWindowEvent, WindowEvent, WindowMethods};
use geom::point::{Point2D, TypedPoint2D};
use geom::scale_factor::ScaleFactor;
use geom::size::TypedSize2D;
use glfw::{mod, Context};
use gleam::gl;
use layers::geometry::DevicePixel;
use layers::platform::surface::NativeGraphicsMetadata;
use libc::c_int;
use msg::compositor_msg::{PaintState, ReadyState};
use msg::constellation_msg::{KeyState, LoadData};
use msg::constellation_msg::{Key, KeyModifiers};
use msg::constellation_msg::{SHIFT, ALT, CONTROL, SUPER};
use std::cell::{Cell, RefCell};
use std::comm::Receiver;
use std::num::Float;
use std::rc::Rc;
use time::{mod, Timespec};
use util::cursor::Cursor;
use util::geometry::ScreenPx;

/// The type of a window.
pub struct Window {
    glfw: glfw::Glfw,

    glfw_window: glfw::Window,
    events: Receiver<(f64, glfw::WindowEvent)>,

    event_queue: RefCell<Vec<WindowEvent>>,

    mouse_down_button: Cell<Option<glfw::MouseButton>>,
    mouse_down_point: Cell<Point2D<c_int>>,

    ready_state: Cell<ReadyState>,
    paint_state: Cell<PaintState>,

    last_title_set_time: Cell<Timespec>,
}

impl Window {
    /// Creates a new window.
    pub fn new(glfw: glfw::Glfw, is_foreground: bool, size: TypedSize2D<DevicePixel, uint>)
               -> Rc<Window> {
        // Create the GLFW window.
        let window_size = size.to_untyped();
        glfw.window_hint(glfw::WindowHint::Visible(is_foreground));
        let (glfw_window, events) = glfw.create_window(window_size.width as u32,
                                                       window_size.height as u32,
                                                       "Servo",
                                                       glfw::WindowMode::Windowed)
            .expect("Failed to create GLFW window");
        glfw_window.make_current();

        gl::load_with(|s| glfw_window.get_proc_address(s));

        // Create our window object.
        let window = Window {
            glfw: glfw,

            glfw_window: glfw_window,
            events: events,

            event_queue: RefCell::new(vec!()),

            mouse_down_button: Cell::new(None),
            mouse_down_point: Cell::new(Point2D(0 as c_int, 0)),

            ready_state: Cell::new(ReadyState::Blank),
            paint_state: Cell::new(PaintState::Idle),

            last_title_set_time: Cell::new(Timespec::new(0, 0)),
        };

        // Register event handlers.
        window.glfw_window.set_framebuffer_size_polling(true);
        window.glfw_window.set_refresh_polling(true);
        window.glfw_window.set_key_polling(true);
        window.glfw_window.set_mouse_button_polling(true);
        window.glfw_window.set_cursor_pos_polling(true);
        window.glfw_window.set_scroll_polling(true);

        window.glfw.set_swap_interval(1);

        Rc::new(window)
    }

    pub fn wait_events(&self) -> WindowEvent {
        self.wait_or_poll_events(|glfw| glfw.wait_events())
    }

    pub fn poll_events(&self) -> WindowEvent {
        self.wait_or_poll_events(|glfw| glfw.poll_events())
    }

    /// Helper method to factor out functionality from `poll_events` and `wait_events`.
    fn wait_or_poll_events(&self, callback: |glfw: &glfw::Glfw|) -> WindowEvent {
        {
            let mut event_queue = self.event_queue.borrow_mut();
            if !event_queue.is_empty() {
                return event_queue.remove(0).unwrap();
            }
        }

        callback(&self.glfw);
        for (_, event) in glfw::flush_messages(&self.events) {
            self.handle_window_event(&self.glfw_window, event);
        }

        if self.glfw_window.should_close() {
            WindowEvent::Quit
        } else {
            self.event_queue.borrow_mut().remove(0).unwrap_or(WindowEvent::Idle)
        }
    }

    pub unsafe fn set_nested_event_loop_listener(
            &self,
            listener: *mut (NestedEventLoopListener + 'static)) {
        self.glfw_window.set_refresh_polling(false);
        glfw::ffi::glfwSetWindowRefreshCallback(self.glfw_window.ptr, Some(on_refresh));
        glfw::ffi::glfwSetFramebufferSizeCallback(self.glfw_window.ptr, Some(on_framebuffer_size));
        g_nested_event_loop_listener = Some(listener)
    }

    pub unsafe fn remove_nested_event_loop_listener(&self) {
        glfw::ffi::glfwSetWindowRefreshCallback(self.glfw_window.ptr, None);
        glfw::ffi::glfwSetFramebufferSizeCallback(self.glfw_window.ptr, None);
        self.glfw_window.set_refresh_polling(true);
        g_nested_event_loop_listener = None
    }
}

static mut g_nested_event_loop_listener: Option<*mut (NestedEventLoopListener + 'static)> = None;

impl WindowMethods for Window {
    /// Returns the size of the window in hardware pixels.
    fn framebuffer_size(&self) -> TypedSize2D<DevicePixel, uint> {
        let (width, height) = self.glfw_window.get_framebuffer_size();
        TypedSize2D(width as uint, height as uint)
    }

    /// Returns the size of the window in density-independent "px" units.
    fn size(&self) -> TypedSize2D<ScreenPx, f32> {
        let (width, height) = self.glfw_window.get_size();
        TypedSize2D(width as f32, height as f32)
    }

    /// Presents the window to the screen (perhaps by page flipping).
    fn present(&self) {
        self.glfw_window.swap_buffers();
    }

    /// Sets the ready state.
    fn set_ready_state(&self, ready_state: ReadyState) {
        self.ready_state.set(ready_state);
        self.update_window_title()
    }

    /// Sets the paint state.
    fn set_paint_state(&self, paint_state: PaintState) {
        self.paint_state.set(paint_state);
        self.update_window_title()
    }

    fn hidpi_factor(&self) -> ScaleFactor<ScreenPx, DevicePixel, f32> {
        let backing_size = self.framebuffer_size().width.get();
        let window_size = self.size().width.get();
        ScaleFactor((backing_size as f32) / window_size)
    }

    #[cfg(target_os="linux")]
    fn native_metadata(&self) -> NativeGraphicsMetadata {
        NativeGraphicsMetadata {
            display: unsafe { glfw::ffi::glfwGetX11Display() },
        }
    }

    #[cfg(target_os="macos")]
    fn native_metadata(&self) -> NativeGraphicsMetadata {
        use cgl::{CGLGetCurrentContext, CGLGetPixelFormat};
        unsafe {
            NativeGraphicsMetadata {
                pixel_format: CGLGetPixelFormat(CGLGetCurrentContext()),
            }
        }
    }

    fn create_compositor_channel(_: &Option<Rc<Window>>)
                                 -> (Box<CompositorProxy+Send>, Box<CompositorReceiver>) {
        let (sender, receiver) = channel();
        (box GlfwCompositorProxy {
             sender: sender,
         } as Box<CompositorProxy+Send>,
         box receiver as Box<CompositorReceiver>)
    }


    /// Helper function to handle keyboard events.
    fn handle_key(&self, key: Key, mods: KeyModifiers) {
        match key {
            Key::Escape => self.glfw_window.set_should_close(true),
            Key::Equal if mods.contains(CONTROL) => { // Ctrl-+
                self.event_queue.borrow_mut().push(WindowEvent::Zoom(1.1));
            }
            Key::Minus if mods.contains(CONTROL) => { // Ctrl--
                self.event_queue.borrow_mut().push(WindowEvent::Zoom(1.0/1.1));
            }
            Key::Backspace if mods.contains(SHIFT) => { // Shift-Backspace
                self.event_queue.borrow_mut().push(WindowEvent::Navigation(WindowNavigateMsg::Forward));
            }
            Key::Backspace => { // Backspace
                self.event_queue.borrow_mut().push(WindowEvent::Navigation(WindowNavigateMsg::Back));
            }
            Key::PageDown => {
                let (_, height) = self.glfw_window.get_size();
                self.scroll_window(0.0, -height as f32);
            }
            Key::PageUp => {
                let (_, height) = self.glfw_window.get_size();
                self.scroll_window(0.0, height as f32);
            }
            _ => {}
        }
    }

    fn prepare_for_composite(&self) -> bool {
        true
    }

    fn set_cursor(&self, _: Cursor) {
        // No-op. We could take over mouse handling ourselves and draw the cursor as an extra
        // layer with our own custom bitmaps or something, but it doesn't seem worth the
        // trouble.
    }

    fn load_end(&self) {}

    fn set_page_title(&self, _: Option<String>) {}

    fn set_page_load_data(&self, _: LoadData) {}
}

impl Window {
    fn handle_window_event(&self, window: &glfw::Window, event: glfw::WindowEvent) {
        match event {
            glfw::WindowEvent::Key(key, _, action, mods) => {
                let key = glfw_key_to_script_key(key);
                let state = match action {
                    glfw::Action::Press => KeyState::Pressed,
                    glfw::Action::Release => KeyState::Released,
                    glfw::Action::Repeat => KeyState::Repeated,
                };
                let modifiers = glfw_mods_to_script_mods(mods);
                self.event_queue.borrow_mut().push(WindowEvent::KeyEvent(key, state, modifiers));
            },
            glfw::WindowEvent::FramebufferSize(width, height) => {
                self.event_queue.borrow_mut().push(
                    WindowEvent::Resize(TypedSize2D(width as uint, height as uint)));
            },
            glfw::WindowEvent::Refresh => {
                self.event_queue.borrow_mut().push(WindowEvent::Refresh);
            },
            glfw::WindowEvent::MouseButton(button, action, _mods) => {
                let cursor_position = self.cursor_position();
                match button {
                    glfw::MouseButton::Button5 => { // Back button (might be different per platform)
                        self.event_queue.borrow_mut().push(WindowEvent::Navigation(WindowNavigateMsg::Back));
                    },
                    glfw::MouseButton::Button6 => { // Forward
                        self.event_queue.borrow_mut().push(WindowEvent::Navigation(WindowNavigateMsg::Forward));
                    },
                    glfw::MouseButtonLeft | glfw::MouseButtonRight => {
                        self.handle_mouse(button,
                                          action,
                                          cursor_position.x.get() as i32,
                                          cursor_position.y.get() as i32);
                    }
                    _ => {}
                }
            },
            glfw::WindowEvent::CursorPos(..) => {
                self.event_queue
                    .borrow_mut()
                    .push(WindowEvent::MouseWindowMoveEventClass(self.cursor_position()));
            },
            glfw::WindowEvent::Scroll(xpos, ypos) => {
                match (window.get_key(glfw::Key::LeftControl),
                       window.get_key(glfw::Key::RightControl)) {
                    (glfw::Action::Press, _) | (_, glfw::Action::Press) => {
                        // Ctrl-Scrollwheel simulates a "pinch zoom" gesture.
                        if ypos < 0.0 {
                            self.event_queue.borrow_mut().push(WindowEvent::PinchZoom(1.0/1.1));
                        } else if ypos > 0.0 {
                            self.event_queue.borrow_mut().push(WindowEvent::PinchZoom(1.1));
                        }
                    },
                    _ => {
                        let dx = (xpos as f32) * 30.0;
                        let dy = (ypos as f32) * 30.0;
                        self.scroll_window(dx, dy);
                    }
                }
            },
            _ => {}
        }
    }

    /// Helper function to send a scroll event.
    fn scroll_window(&self, dx: f32, dy: f32) {
        let cursor_pos = self.cursor_position().cast().unwrap();
        self.event_queue.borrow_mut().push(WindowEvent::Scroll(TypedPoint2D(dx, dy), cursor_pos));
    }

    /// Helper function to set the window title in accordance with the ready state.
    fn update_window_title(&self) {
        let now = time::get_time();
        if now.sec == self.last_title_set_time.get().sec {
            return
        }
        self.last_title_set_time.set(now);

        match self.ready_state.get() {
            ReadyState::Blank => {
                self.glfw_window.set_title("blank — Servo [GLFW]")
            }
            ReadyState::Loading => {
                self.glfw_window.set_title("Loading — Servo [GLFW]")
            }
            ReadyState::PerformingLayout => {
                self.glfw_window.set_title("Performing Layout — Servo [GLFW]")
            }
            ReadyState::FinishedLoading => {
                match self.paint_state.get() {
                    PaintState::Painting => {
                        self.glfw_window.set_title("Rendering — Servo [GLFW]")
                    }
                    PaintState::Idle => {
                        self.glfw_window.set_title("Servo [GLFW]")
                    }
                }
            }
        }
    }

    /// Helper function to handle a click
    fn handle_mouse(&self, button: glfw::MouseButton, action: glfw::Action, x: c_int, y: c_int) {
        // FIXME(tkuehn): max pixel dist should be based on pixel density
        let max_pixel_dist = 10f64;
        let event = match action {
            glfw::Action::Press => {
                self.mouse_down_point.set(Point2D(x, y));
                self.mouse_down_button.set(Some(button));
                MouseWindowEvent::MouseDown(button as uint, TypedPoint2D(x as f32, y as f32))
            }
            glfw::Action::Release => {
                match self.mouse_down_button.get() {
                    None => (),
                    Some(but) if button == but => {
                        let pixel_dist = self.mouse_down_point.get() - Point2D(x, y);
                        let pixel_dist = ((pixel_dist.x * pixel_dist.x +
                                           pixel_dist.y * pixel_dist.y) as f64).sqrt();
                        if pixel_dist < max_pixel_dist {
                            let click_event = MouseWindowEvent::Click(
                                button as uint, TypedPoint2D(x as f32, y as f32));
                            self.event_queue.borrow_mut().push(WindowEvent::MouseWindowEventClass(click_event));
                        }
                    }
                    Some(_) => (),
                }
                MouseWindowEvent::MouseUp(button as uint, TypedPoint2D(x as f32, y as f32))
            }
            _ => panic!("I cannot recognize the type of mouse action that occured. :-(")
        };
        self.event_queue.borrow_mut().push(WindowEvent::MouseWindowEventClass(event));
    }

    /// Returns the cursor position, properly accounting for HiDPI.
    fn cursor_position(&self) -> TypedPoint2D<DevicePixel,f32> {
        // Handle hidpi displays, since GLFW returns non-hi-def coordinates.
        let (x, y) = self.glfw_window.get_cursor_pos();
        let hidpi_factor = self.hidpi_factor();
        Point2D::from_untyped(&Point2D(x as f32, y as f32)) * hidpi_factor
    }
}

struct GlfwCompositorProxy {
    sender: Sender<compositor_task::Msg>,
}

impl CompositorProxy for GlfwCompositorProxy {
    fn send(&mut self, msg: compositor_task::Msg) {
        // Send a message and kick the OS event loop awake.
        self.sender.send(msg);
        glfw::Glfw::post_empty_event()
    }
    fn clone_compositor_proxy(&self) -> Box<CompositorProxy+Send> {
        box GlfwCompositorProxy {
            sender: self.sender.clone(),
        } as Box<CompositorProxy+Send>
    }
}

extern "C" fn on_refresh(_glfw_window: *mut glfw::ffi::GLFWwindow) {
    unsafe {
        match g_nested_event_loop_listener {
            None => {}
            Some(listener) => {
                (*listener).handle_event_from_nested_event_loop(WindowEvent::Refresh);
            }
        }
    }
}

extern "C" fn on_framebuffer_size(_glfw_window: *mut glfw::ffi::GLFWwindow,
                                  width: c_int,
                                  height: c_int) {
    unsafe {
        match g_nested_event_loop_listener {
            None => {}
            Some(listener) => {
                let size = TypedSize2D(width as uint, height as uint);
                (*listener).handle_event_from_nested_event_loop(WindowEvent::Resize(size));
            }
        }
    }
}

fn glfw_mods_to_script_mods(mods: glfw::Modifiers) -> KeyModifiers {
    let mut result = KeyModifiers::from_bits(0).unwrap();
    if mods.contains(glfw::Shift) {
        result.insert(SHIFT);
    }
    if mods.contains(glfw::Alt) {
        result.insert(ALT);
    }
    if mods.contains(glfw::Control) {
        result.insert(CONTROL);
    }
    if mods.contains(glfw::Super) {
        result.insert(SUPER);
    }
    result
}

macro_rules! glfw_keys_to_script_keys(
    ($key:expr, $($name:ident),+) => (
        match $key {
            $(glfw::Key::$name => Key::$name,)+
        }
    );
)

fn glfw_key_to_script_key(key: glfw::Key) -> Key {
    glfw_keys_to_script_keys!(key,
                              Space,
                              Apostrophe,
                              Comma,
                              Minus,
                              Period,
                              Slash,
                              Num0,
                              Num1,
                              Num2,
                              Num3,
                              Num4,
                              Num5,
                              Num6,
                              Num7,
                              Num8,
                              Num9,
                              Semicolon,
                              Equal,
                              A,
                              B,
                              C,
                              D,
                              E,
                              F,
                              G,
                              H,
                              I,
                              J,
                              K,
                              L,
                              M,
                              N,
                              O,
                              P,
                              Q,
                              R,
                              S,
                              T,
                              U,
                              V,
                              W,
                              X,
                              Y,
                              Z,
                              LeftBracket,
                              Backslash,
                              RightBracket,
                              GraveAccent,
                              World1,
                              World2,

                              Escape,
                              Enter,
                              Tab,
                              Backspace,
                              Insert,
                              Delete,
                              Right,
                              Left,
                              Down,
                              Up,
                              PageUp,
                              PageDown,
                              Home,
                              End,
                              CapsLock,
                              ScrollLock,
                              NumLock,
                              PrintScreen,
                              Pause,
                              F1,
                              F2,
                              F3,
                              F4,
                              F5,
                              F6,
                              F7,
                              F8,
                              F9,
                              F10,
                              F11,
                              F12,
                              F13,
                              F14,
                              F15,
                              F16,
                              F17,
                              F18,
                              F19,
                              F20,
                              F21,
                              F22,
                              F23,
                              F24,
                              F25,
                              Kp0,
                              Kp1,
                              Kp2,
                              Kp3,
                              Kp4,
                              Kp5,
                              Kp6,
                              Kp7,
                              Kp8,
                              Kp9,
                              KpDecimal,
                              KpDivide,
                              KpMultiply,
                              KpSubtract,
                              KpAdd,
                              KpEnter,
                              KpEqual,
                              LeftShift,
                              LeftControl,
                              LeftAlt,
                              LeftSuper,
                              RightShift,
                              RightControl,
                              RightAlt,
                              RightSuper,
                              Menu)
}

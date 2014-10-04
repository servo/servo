/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! A windowing implementation using GLFW.

use NestedEventLoopListener;

use alert::{Alert, AlertMethods};
use compositing::compositor_task::{mod, CompositorProxy, CompositorReceiver};
use compositing::windowing::{Forward, Back};
use compositing::windowing::{IdleWindowEvent, ResizeWindowEvent, LoadUrlWindowEvent};
use compositing::windowing::{KeyEvent, MouseWindowClickEvent, MouseWindowMouseDownEvent};
use compositing::windowing::{MouseWindowEventClass,  MouseWindowMoveEventClass};
use compositing::windowing::{MouseWindowMouseUpEvent, RefreshWindowEvent};
use compositing::windowing::{NavigationWindowEvent, ScrollWindowEvent, ZoomWindowEvent};
use compositing::windowing::{PinchZoomWindowEvent, QuitWindowEvent};
use compositing::windowing::{WindowEvent, WindowMethods, FinishedWindowEvent};
use geom::point::{Point2D, TypedPoint2D};
use geom::scale_factor::ScaleFactor;
use geom::size::TypedSize2D;
use glfw::{mod, Context};
use layers::geometry::DevicePixel;
use layers::platform::surface::NativeGraphicsMetadata;
use libc::c_int;
use msg::compositor_msg::{FinishedLoading, Blank, Loading, PerformingLayout, ReadyState};
use msg::compositor_msg::{IdleRenderState, RenderState, RenderingRenderState};
use msg::constellation_msg;
use std::cell::{Cell, RefCell};
use std::comm::Receiver;
use std::rc::Rc;
use time::{mod, Timespec};
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
    render_state: Cell<RenderState>,

    last_title_set_time: Cell<Timespec>,
}

impl Window {
    /// Creates a new window.
    pub fn new(glfw: glfw::Glfw, is_foreground: bool, size: TypedSize2D<DevicePixel, uint>)
               -> Rc<Window> {
        // Create the GLFW window.
        let window_size = size.to_untyped();
        glfw.window_hint(glfw::Visible(is_foreground));
        let (glfw_window, events) = glfw.create_window(window_size.width as u32,
                                                       window_size.height as u32,
                                                       "Servo", glfw::Windowed)
            .expect("Failed to create GLFW window");
        glfw_window.make_current();

        // Create our window object.
        let window = Window {
            glfw: glfw,

            glfw_window: glfw_window,
            events: events,

            event_queue: RefCell::new(vec!()),

            mouse_down_button: Cell::new(None),
            mouse_down_point: Cell::new(Point2D(0 as c_int, 0)),

            ready_state: Cell::new(Blank),
            render_state: Cell::new(IdleRenderState),

            last_title_set_time: Cell::new(Timespec::new(0, 0)),
        };

        // Register event handlers.
        window.glfw_window.set_framebuffer_size_polling(true);
        window.glfw_window.set_refresh_polling(true);
        window.glfw_window.set_key_polling(true);
        window.glfw_window.set_mouse_button_polling(true);
        window.glfw_window.set_cursor_pos_polling(true);
        window.glfw_window.set_scroll_polling(true);

        glfw.set_swap_interval(1);

        Rc::new(window)
    }

    pub fn wait_events(&self) -> WindowEvent {
        {
            let mut event_queue = self.event_queue.borrow_mut();
            if !event_queue.is_empty() {
                return event_queue.remove(0).unwrap();
            }
        }

        self.glfw.wait_events();
        for (_, event) in glfw::flush_messages(&self.events) {
            self.handle_window_event(&self.glfw_window, event);
        }

        if self.glfw_window.should_close() {
            QuitWindowEvent
        } else {
            self.event_queue.borrow_mut().remove(0).unwrap_or(IdleWindowEvent)
        }
    }

    pub unsafe fn set_nested_event_loop_listener(
            &self,
            listener: *mut NestedEventLoopListener + 'static) {
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

static mut g_nested_event_loop_listener: Option<*mut NestedEventLoopListener + 'static> = None;

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

    /// Sets the render state.
    fn set_render_state(&self, render_state: RenderState) {
        if self.ready_state.get() == FinishedLoading &&
            self.render_state.get() == RenderingRenderState &&
            render_state == IdleRenderState {
            // page loaded
            self.event_queue.borrow_mut().push(FinishedWindowEvent);
        }

        self.render_state.set(render_state);
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
}

impl Window {
    fn handle_window_event(&self, window: &glfw::Window, event: glfw::WindowEvent) {
        match event {
            glfw::KeyEvent(key, _, action, mods) => {
                if action == glfw::Press {
                    self.handle_key(key, mods);
                }
                let key = glfw_key_to_script_key(key);
                let state = match action {
                    glfw::Press => constellation_msg::Pressed,
                    glfw::Release => constellation_msg::Released,
                    glfw::Repeat => constellation_msg::Repeated,
                };
                let modifiers = glfw_mods_to_script_mods(mods);
                self.event_queue.borrow_mut().push(KeyEvent(key, state, modifiers));
            },
            glfw::FramebufferSizeEvent(width, height) => {
                self.event_queue.borrow_mut().push(
                    ResizeWindowEvent(TypedSize2D(width as uint, height as uint)));
            },
            glfw::RefreshEvent => {
                self.event_queue.borrow_mut().push(RefreshWindowEvent);
            },
            glfw::MouseButtonEvent(button, action, _mods) => {
                let (x, y) = window.get_cursor_pos();
                //handle hidpi displays, since GLFW returns non-hi-def coordinates.
                let (backing_size, _) = window.get_framebuffer_size();
                let (window_size, _) = window.get_size();
                let hidpi = (backing_size as f32) / (window_size as f32);
                let x = x as f32 * hidpi;
                let y = y as f32 * hidpi;

                match button {
                    glfw::MouseButton5 => { // Back button (might be different per platform)
                        self.event_queue.borrow_mut().push(NavigationWindowEvent(Back));
                    },
                    glfw::MouseButton6 => { // Forward
                        self.event_queue.borrow_mut().push(NavigationWindowEvent(Forward));
                    },
                    glfw::MouseButtonLeft | glfw::MouseButtonRight => {
                        self.handle_mouse(button, action, x as i32, y as i32);
                    }
                    _ => {}
                }
            },
            glfw::CursorPosEvent(xpos, ypos) => {
                self.event_queue.borrow_mut().push(
                    MouseWindowMoveEventClass(TypedPoint2D(xpos as f32, ypos as f32)));
            },
            glfw::ScrollEvent(xpos, ypos) => {
                match (window.get_key(glfw::KeyLeftControl),
                       window.get_key(glfw::KeyRightControl)) {
                    (glfw::Press, _) | (_, glfw::Press) => {
                        // Ctrl-Scrollwheel simulates a "pinch zoom" gesture.
                        if ypos < 0.0 {
                            self.event_queue.borrow_mut().push(PinchZoomWindowEvent(1.0/1.1));
                        } else if ypos > 0.0 {
                            self.event_queue.borrow_mut().push(PinchZoomWindowEvent(1.1));
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
        let (x, y) = self.glfw_window.get_cursor_pos();
        //handle hidpi displays, since GLFW returns non-hi-def coordinates.
        let (backing_size, _) = self.glfw_window.get_framebuffer_size();
        let (window_size, _) = self.glfw_window.get_size();
        let hidpi = (backing_size as f32) / (window_size as f32);
        let x = x as f32 * hidpi;
        let y = y as f32 * hidpi;

        self.event_queue.borrow_mut().push(ScrollWindowEvent(TypedPoint2D(dx, dy),
        TypedPoint2D(x as i32, y as i32)));
    }

    /// Helper function to set the window title in accordance with the ready state.
    fn update_window_title(&self) {
        let now = time::get_time();
        if now.sec == self.last_title_set_time.get().sec {
            return
        }
        self.last_title_set_time.set(now);

        match self.ready_state.get() {
            Blank => {
                self.glfw_window.set_title("blank — Servo")
            }
            Loading => {
                self.glfw_window.set_title("Loading — Servo")
            }
            PerformingLayout => {
                self.glfw_window.set_title("Performing Layout — Servo")
            }
            FinishedLoading => {
                match self.render_state.get() {
                    RenderingRenderState => {
                        self.glfw_window.set_title("Rendering — Servo")
                    }
                    IdleRenderState => {
                        self.glfw_window.set_title("Servo")
                    }
                }
            }
        }
    }

    /// Helper function to handle keyboard events.
    fn handle_key(&self, key: glfw::Key, mods: glfw::Modifiers) {
        match key {
            glfw::KeyEscape => self.glfw_window.set_should_close(true),
            glfw::KeyL if mods.contains(glfw::Control) => self.load_url(), // Ctrl+L
            glfw::KeyEqual if mods.contains(glfw::Control) => { // Ctrl-+
                self.event_queue.borrow_mut().push(ZoomWindowEvent(1.1));
            }
            glfw::KeyMinus if mods.contains(glfw::Control) => { // Ctrl--
                self.event_queue.borrow_mut().push(ZoomWindowEvent(1.0/1.1));
            }
            glfw::KeyBackspace if mods.contains(glfw::Shift) => { // Shift-Backspace
                self.event_queue.borrow_mut().push(NavigationWindowEvent(Forward));
            }
            glfw::KeyBackspace => { // Backspace
                self.event_queue.borrow_mut().push(NavigationWindowEvent(Back));
            }
            glfw::KeyPageDown => {
                let (_, height) = self.glfw_window.get_size();
                self.scroll_window(0.0, -height as f32);
            }
            glfw::KeyPageUp => {
                let (_, height) = self.glfw_window.get_size();
                self.scroll_window(0.0, height as f32);
            }
            _ => {}
        }
    }

    /// Helper function to handle a click
    fn handle_mouse(&self, button: glfw::MouseButton, action: glfw::Action, x: c_int, y: c_int) {
        // FIXME(tkuehn): max pixel dist should be based on pixel density
        let max_pixel_dist = 10f64;
        let event = match action {
            glfw::Press => {
                self.mouse_down_point.set(Point2D(x, y));
                self.mouse_down_button.set(Some(button));
                MouseWindowMouseDownEvent(button as uint, TypedPoint2D(x as f32, y as f32))
            }
            glfw::Release => {
                match self.mouse_down_button.get() {
                    None => (),
                    Some(but) if button == but => {
                        let pixel_dist = self.mouse_down_point.get() - Point2D(x, y);
                        let pixel_dist = ((pixel_dist.x * pixel_dist.x +
                                           pixel_dist.y * pixel_dist.y) as f64).sqrt();
                        if pixel_dist < max_pixel_dist {
                            let click_event = MouseWindowClickEvent(button as uint,
                                                                    TypedPoint2D(x as f32,
                                                                                 y as f32));
                            self.event_queue.borrow_mut().push(MouseWindowEventClass(click_event));
                        }
                    }
                    Some(_) => (),
                }
                MouseWindowMouseUpEvent(button as uint, TypedPoint2D(x as f32, y as f32))
            }
            _ => panic!("I cannot recognize the type of mouse action that occured. :-(")
        };
        self.event_queue.borrow_mut().push(MouseWindowEventClass(event));
    }

    /// Helper function to pop up an alert box prompting the user to load a URL.
    fn load_url(&self) {
        let mut alert: Alert = AlertMethods::new("Navigate to:");
        alert.add_prompt();
        alert.run();
        let value = alert.prompt_value();
        if "" == value.as_slice() {    // To avoid crashing on Linux.
            self.event_queue.borrow_mut().push(LoadUrlWindowEvent("http://purple.com/".to_string()))
        } else {
            self.event_queue.borrow_mut().push(LoadUrlWindowEvent(value.clone()))
        }
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
                (*listener).handle_event_from_nested_event_loop(RefreshWindowEvent);
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
                (*listener).handle_event_from_nested_event_loop(ResizeWindowEvent(size));
            }
        }
    }
}

fn glfw_mods_to_script_mods(mods: glfw::Modifiers) -> constellation_msg::KeyModifiers {
    let mut result = constellation_msg::KeyModifiers::from_bits(0).unwrap();
    if mods.contains(glfw::Shift) {
        result.insert(constellation_msg::Shift);
    }
    if mods.contains(glfw::Alt) {
        result.insert(constellation_msg::Alt);
    }
    if mods.contains(glfw::Control) {
        result.insert(constellation_msg::Control);
    }
    if mods.contains(glfw::Super) {
        result.insert(constellation_msg::Super);
    }
    result
}

fn glfw_key_to_script_key(key: glfw::Key) -> constellation_msg::Key {
    match key {
        glfw::KeySpace => constellation_msg::KeySpace,
        glfw::KeyApostrophe => constellation_msg::KeyApostrophe,
        glfw::KeyComma => constellation_msg::KeyComma,
        glfw::KeyMinus => constellation_msg::KeyMinus,
        glfw::KeyPeriod => constellation_msg::KeyPeriod,
        glfw::KeySlash => constellation_msg::KeySlash,
        glfw::Key0 => constellation_msg::Key0,
        glfw::Key1 => constellation_msg::Key1,
        glfw::Key2 => constellation_msg::Key2,
        glfw::Key3 => constellation_msg::Key3,
        glfw::Key4 => constellation_msg::Key4,
        glfw::Key5 => constellation_msg::Key5,
        glfw::Key6 => constellation_msg::Key6,
        glfw::Key7 => constellation_msg::Key7,
        glfw::Key8 => constellation_msg::Key8,
        glfw::Key9 => constellation_msg::Key9,
        glfw::KeySemicolon => constellation_msg::KeySemicolon,
        glfw::KeyEqual => constellation_msg::KeyEqual,
        glfw::KeyA => constellation_msg::KeyA,
        glfw::KeyB => constellation_msg::KeyB,
        glfw::KeyC => constellation_msg::KeyC,
        glfw::KeyD => constellation_msg::KeyD,
        glfw::KeyE => constellation_msg::KeyE,
        glfw::KeyF => constellation_msg::KeyF,
        glfw::KeyG => constellation_msg::KeyG,
        glfw::KeyH => constellation_msg::KeyH,
        glfw::KeyI => constellation_msg::KeyI,
        glfw::KeyJ => constellation_msg::KeyJ,
        glfw::KeyK => constellation_msg::KeyK,
        glfw::KeyL => constellation_msg::KeyL,
        glfw::KeyM => constellation_msg::KeyM,
        glfw::KeyN => constellation_msg::KeyN,
        glfw::KeyO => constellation_msg::KeyO,
        glfw::KeyP => constellation_msg::KeyP,
        glfw::KeyQ => constellation_msg::KeyQ,
        glfw::KeyR => constellation_msg::KeyR,
        glfw::KeyS => constellation_msg::KeyS,
        glfw::KeyT => constellation_msg::KeyT,
        glfw::KeyU => constellation_msg::KeyU,
        glfw::KeyV => constellation_msg::KeyV,
        glfw::KeyW => constellation_msg::KeyW,
        glfw::KeyX => constellation_msg::KeyX,
        glfw::KeyY => constellation_msg::KeyY,
        glfw::KeyZ => constellation_msg::KeyZ,
        glfw::KeyLeftBracket => constellation_msg::KeyLeftBracket,
        glfw::KeyBackslash => constellation_msg::KeyBackslash,
        glfw::KeyRightBracket => constellation_msg::KeyRightBracket,
        glfw::KeyGraveAccent => constellation_msg::KeyGraveAccent,
        glfw::KeyWorld1 => constellation_msg::KeyWorld1,
        glfw::KeyWorld2 => constellation_msg::KeyWorld2,
        glfw::KeyEscape => constellation_msg::KeyEscape,
        glfw::KeyEnter => constellation_msg::KeyEnter,
        glfw::KeyTab => constellation_msg::KeyTab,
        glfw::KeyBackspace => constellation_msg::KeyBackspace,
        glfw::KeyInsert => constellation_msg::KeyInsert,
        glfw::KeyDelete => constellation_msg::KeyDelete,
        glfw::KeyRight => constellation_msg::KeyRight,
        glfw::KeyLeft => constellation_msg::KeyLeft,
        glfw::KeyDown => constellation_msg::KeyDown,
        glfw::KeyUp => constellation_msg::KeyUp,
        glfw::KeyPageUp => constellation_msg::KeyPageUp,
        glfw::KeyPageDown => constellation_msg::KeyPageDown,
        glfw::KeyHome => constellation_msg::KeyHome,
        glfw::KeyEnd => constellation_msg::KeyEnd,
        glfw::KeyCapsLock => constellation_msg::KeyCapsLock,
        glfw::KeyScrollLock => constellation_msg::KeyScrollLock,
        glfw::KeyNumLock => constellation_msg::KeyNumLock,
        glfw::KeyPrintScreen => constellation_msg::KeyPrintScreen,
        glfw::KeyPause => constellation_msg::KeyPause,
        glfw::KeyF1 => constellation_msg::KeyF1,
        glfw::KeyF2 => constellation_msg::KeyF2,
        glfw::KeyF3 => constellation_msg::KeyF3,
        glfw::KeyF4 => constellation_msg::KeyF4,
        glfw::KeyF5 => constellation_msg::KeyF5,
        glfw::KeyF6 => constellation_msg::KeyF6,
        glfw::KeyF7 => constellation_msg::KeyF7,
        glfw::KeyF8 => constellation_msg::KeyF8,
        glfw::KeyF9 => constellation_msg::KeyF9,
        glfw::KeyF10 => constellation_msg::KeyF10,
        glfw::KeyF11 => constellation_msg::KeyF11,
        glfw::KeyF12 => constellation_msg::KeyF12,
        glfw::KeyF13 => constellation_msg::KeyF13,
        glfw::KeyF14 => constellation_msg::KeyF14,
        glfw::KeyF15 => constellation_msg::KeyF15,
        glfw::KeyF16 => constellation_msg::KeyF16,
        glfw::KeyF17 => constellation_msg::KeyF17,
        glfw::KeyF18 => constellation_msg::KeyF18,
        glfw::KeyF19 => constellation_msg::KeyF19,
        glfw::KeyF20 => constellation_msg::KeyF20,
        glfw::KeyF21 => constellation_msg::KeyF21,
        glfw::KeyF22 => constellation_msg::KeyF22,
        glfw::KeyF23 => constellation_msg::KeyF23,
        glfw::KeyF24 => constellation_msg::KeyF24,
        glfw::KeyF25 => constellation_msg::KeyF25,
        glfw::KeyKp0 => constellation_msg::KeyKp0,
        glfw::KeyKp1 => constellation_msg::KeyKp1,
        glfw::KeyKp2 => constellation_msg::KeyKp2,
        glfw::KeyKp3 => constellation_msg::KeyKp3,
        glfw::KeyKp4 => constellation_msg::KeyKp4,
        glfw::KeyKp5 => constellation_msg::KeyKp5,
        glfw::KeyKp6 => constellation_msg::KeyKp6,
        glfw::KeyKp7 => constellation_msg::KeyKp7,
        glfw::KeyKp8 => constellation_msg::KeyKp8,
        glfw::KeyKp9 => constellation_msg::KeyKp9,
        glfw::KeyKpDecimal => constellation_msg::KeyKpDecimal,
        glfw::KeyKpDivide => constellation_msg::KeyKpDivide,
        glfw::KeyKpMultiply => constellation_msg::KeyKpMultiply,
        glfw::KeyKpSubtract => constellation_msg::KeyKpSubtract,
        glfw::KeyKpAdd => constellation_msg::KeyKpAdd,
        glfw::KeyKpEnter => constellation_msg::KeyKpEnter,
        glfw::KeyKpEqual => constellation_msg::KeyKpEqual,
        glfw::KeyLeftShift => constellation_msg::KeyLeftShift,
        glfw::KeyLeftControl => constellation_msg::KeyLeftControl,
        glfw::KeyLeftAlt => constellation_msg::KeyLeftAlt,
        glfw::KeyLeftSuper => constellation_msg::KeyLeftSuper,
        glfw::KeyRightShift => constellation_msg::KeyRightShift,
        glfw::KeyRightControl => constellation_msg::KeyRightControl,
        glfw::KeyRightAlt => constellation_msg::KeyRightAlt,
        glfw::KeyRightSuper => constellation_msg::KeyRightSuper,
        glfw::KeyMenu => constellation_msg::KeyMenu,
    }
}

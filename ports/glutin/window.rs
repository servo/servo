/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! A windowing implementation using glutin.

use compositing::compositor_task::{self, CompositorProxy, CompositorReceiver};
use compositing::windowing::{WindowEvent, WindowMethods};
use euclid::scale_factor::ScaleFactor;
use euclid::size::{Size2D, TypedSize2D};
use gleam::gl;
use glutin;
use layers::geometry::DevicePixel;
use layers::platform::surface::NativeDisplay;
use msg::constellation_msg::{self, Key};
use net_traits::net_error_list::NetError;
use std::rc::Rc;
use std::sync::mpsc::{channel, Sender};
use url::Url;
use util::cursor::Cursor;
use util::geometry::ScreenPx;

use NestedEventLoopListener;

#[cfg(feature = "window")]
use compositing::windowing::{MouseWindowEvent, WindowNavigateMsg};
#[cfg(feature = "window")]
use euclid::point::Point2D;
#[cfg(feature = "window")]
use glutin::{Api, ElementState, Event, GlRequest, MouseButton, VirtualKeyCode, MouseScrollDelta};
#[cfg(feature = "window")]
use msg::constellation_msg::{KeyState, NONE, CONTROL, SHIFT, ALT, SUPER};
#[cfg(feature = "window")]
use std::cell::{Cell, RefCell};
#[cfg(feature = "window")]
use util::opts;

#[cfg(all(feature = "headless", target_os="linux"))]
use std::ptr;

#[cfg(feature = "window")]
static mut g_nested_event_loop_listener: Option<*mut (NestedEventLoopListener + 'static)> = None;

#[cfg(feature = "window")]
bitflags! {
    flags KeyModifiers: u8 {
        const LEFT_CONTROL = 1,
        const RIGHT_CONTROL = 2,
        const LEFT_SHIFT = 4,
        const RIGHT_SHIFT = 8,
        const LEFT_ALT = 16,
        const RIGHT_ALT = 32,
        const LEFT_SUPER = 64,
        const RIGHT_SUPER = 128,
    }
}

// Some shortcuts use Cmd on Mac and Control on other systems.
#[cfg(all(feature = "window", target_os="macos"))]
const CMD_OR_CONTROL: constellation_msg::KeyModifiers = SUPER;
#[cfg(all(feature = "window", not(target_os="macos")))]
const CMD_OR_CONTROL: constellation_msg::KeyModifiers = CONTROL;

// Some shortcuts use Cmd on Mac and Alt on other systems.
#[cfg(all(feature = "window", target_os="macos"))]
const CMD_OR_ALT: constellation_msg::KeyModifiers = SUPER;
#[cfg(all(feature = "window", not(target_os="macos")))]
const CMD_OR_ALT: constellation_msg::KeyModifiers = ALT;

// This should vary by zoom level and maybe actual text size (focused or under cursor)
#[cfg(feature = "window")]
const LINE_HEIGHT: f32 = 38.0;

/// The type of a window.
#[cfg(feature = "window")]
pub struct Window {
    window: glutin::Window,

    mouse_down_button: Cell<Option<glutin::MouseButton>>,
    mouse_down_point: Cell<Point2D<i32>>,
    event_queue: RefCell<Vec<WindowEvent>>,

    mouse_pos: Cell<Point2D<i32>>,
    key_modifiers: Cell<KeyModifiers>,
}

#[cfg(feature = "window")]
impl Window {
    pub fn new(is_foreground: bool,
               window_size: TypedSize2D<DevicePixel, u32>,
               parent: glutin::WindowID) -> Rc<Window> {
        let mut glutin_window = glutin::WindowBuilder::new()
                            .with_title("Servo".to_string())
                            .with_dimensions(window_size.to_untyped().width, window_size.to_untyped().height)
                            .with_gl(Window::gl_version())
                            .with_visibility(is_foreground)
                            .with_parent(parent)
                            .build()
                            .unwrap();

        unsafe { glutin_window.make_current().expect("Failed to make context current!") }

        glutin_window.set_window_resize_callback(Some(Window::nested_window_resize as fn(u32, u32)));

        Window::load_gl_functions(&glutin_window);

        let window = Window {
            window: glutin_window,
            event_queue: RefCell::new(vec!()),
            mouse_down_button: Cell::new(None),
            mouse_down_point: Cell::new(Point2D::new(0, 0)),

            mouse_pos: Cell::new(Point2D::new(0, 0)),
            key_modifiers: Cell::new(KeyModifiers::empty()),
        };

        gl::clear_color(0.6, 0.6, 0.6, 1.0);
        gl::clear(gl::COLOR_BUFFER_BIT);
        gl::finish();
        window.present();

        Rc::new(window)
    }

    pub fn platform_window(&self) -> glutin::WindowID {
        unsafe { self.window.platform_window() }
    }

    fn nested_window_resize(width: u32, height: u32) {
        unsafe {
            match g_nested_event_loop_listener {
                None => {}
                Some(listener) => {
                    (*listener).handle_event_from_nested_event_loop(
                        WindowEvent::Resize(Size2D::typed(width, height)));
                }
            }
        }
    }

    #[cfg(not(target_os="android"))]
    fn gl_version() -> GlRequest {
        GlRequest::Specific(Api::OpenGl, (3, 0))
    }

    #[cfg(target_os="android")]
    fn gl_version() -> GlRequest {
        GlRequest::Specific(Api::OpenGlEs, (2, 0))
    }

    #[cfg(not(target_os="android"))]
    fn load_gl_functions(window: &glutin::Window) {
        gl::load_with(|s| window.get_proc_address(s));
    }

    #[cfg(target_os="android")]
    fn load_gl_functions(_: &glutin::Window) {
    }

    fn handle_window_event(&self, event: glutin::Event) -> bool {
        match event {
            Event::KeyboardInput(element_state, _scan_code, virtual_key_code) => {
                if virtual_key_code.is_some() {
                    let virtual_key_code = virtual_key_code.unwrap();

                    match (element_state, virtual_key_code) {
                        (_, VirtualKeyCode::LControl) => self.toggle_modifier(LEFT_CONTROL),
                        (_, VirtualKeyCode::RControl) => self.toggle_modifier(RIGHT_CONTROL),
                        (_, VirtualKeyCode::LShift) => self.toggle_modifier(LEFT_SHIFT),
                        (_, VirtualKeyCode::RShift) => self.toggle_modifier(RIGHT_SHIFT),
                        (_, VirtualKeyCode::LAlt) => self.toggle_modifier(LEFT_ALT),
                        (_, VirtualKeyCode::RAlt) => self.toggle_modifier(RIGHT_ALT),
                        (_, VirtualKeyCode::LWin) => self.toggle_modifier(LEFT_SUPER),
                        (_, VirtualKeyCode::RWin) => self.toggle_modifier(RIGHT_SUPER),
                        (ElementState::Pressed, VirtualKeyCode::Escape) => return true,
                        (_, key_code) => {
                            match Window::glutin_key_to_script_key(key_code) {
                                Ok(key) => {
                                    let state = match element_state {
                                        ElementState::Pressed => KeyState::Pressed,
                                        ElementState::Released => KeyState::Released,
                                    };
                                    let modifiers = Window::glutin_mods_to_script_mods(self.key_modifiers.get());
                                    self.event_queue.borrow_mut().push(WindowEvent::KeyEvent(key, state, modifiers));
                                }
                                _ => {}
                            }
                        }
                    }
                }
            }
            Event::Resized(width, height) => {
                self.event_queue.borrow_mut().push(WindowEvent::Resize(Size2D::typed(width, height)));
            }
            Event::MouseInput(element_state, mouse_button) => {
                if mouse_button == MouseButton::Left ||
                                    mouse_button == MouseButton::Right {
                        let mouse_pos = self.mouse_pos.get();
                        self.handle_mouse(mouse_button, element_state, mouse_pos.x, mouse_pos.y);
                   }
            }
            Event::MouseMoved((x, y)) => {
                self.mouse_pos.set(Point2D::new(x, y));
                self.event_queue.borrow_mut().push(
                    WindowEvent::MouseWindowMoveEventClass(Point2D::typed(x as f32, y as f32)));
            }
            Event::MouseWheel(delta) => {
                if self.ctrl_pressed() {
                    // Ctrl-Scrollwheel simulates a "pinch zoom" gesture.
                    let dy = match delta {
                        MouseScrollDelta::LineDelta(_, dy) => dy,
                        MouseScrollDelta::PixelDelta(_, dy) => dy
                    };
                    if dy < 0.0 {
                        self.event_queue.borrow_mut().push(WindowEvent::PinchZoom(1.0 / 1.1));
                    } else if dy > 0.0 {
                        self.event_queue.borrow_mut().push(WindowEvent::PinchZoom(1.1));
                    }
                } else {
                    match delta {
                        MouseScrollDelta::LineDelta(dx, dy) => {
                            self.scroll_window(dx, dy * LINE_HEIGHT);
                        }
                        MouseScrollDelta::PixelDelta(dx, dy) => self.scroll_window(dx, dy)
                    }
                }
            },
            Event::Refresh => {
                self.event_queue.borrow_mut().push(WindowEvent::Refresh);
            }
            _ => {}
        }

        false
    }

    #[inline]
    fn ctrl_pressed(&self) -> bool {
        self.key_modifiers.get().intersects(LEFT_CONTROL | RIGHT_CONTROL)
    }

    fn toggle_modifier(&self, modifier: KeyModifiers) {
        let mut modifiers = self.key_modifiers.get();
        modifiers.toggle(modifier);
        self.key_modifiers.set(modifiers);
    }

    /// Helper function to send a scroll event.
    fn scroll_window(&self, dx: f32, dy: f32) {
        let mouse_pos = self.mouse_pos.get();
        let event = WindowEvent::Scroll(Point2D::typed(dx as f32, dy as f32),
                                        Point2D::typed(mouse_pos.x as i32, mouse_pos.y as i32));
        self.event_queue.borrow_mut().push(event);
    }

    /// Helper function to handle a click
    fn handle_mouse(&self, button: glutin::MouseButton, action: glutin::ElementState, x: i32, y: i32) {
        use script_traits::MouseButton;

        // FIXME(tkuehn): max pixel dist should be based on pixel density
        let max_pixel_dist = 10f64;
        let event = match action {
            ElementState::Pressed => {
                self.mouse_down_point.set(Point2D::new(x, y));
                self.mouse_down_button.set(Some(button));
                MouseWindowEvent::MouseDown(MouseButton::Left, Point2D::typed(x as f32, y as f32))
            }
            ElementState::Released => {
                let mouse_up_event = MouseWindowEvent::MouseUp(MouseButton::Left, Point2D::typed(x as f32, y as f32));
                match self.mouse_down_button.get() {
                    None => mouse_up_event,
                    Some(but) if button == but => {
                        let pixel_dist = self.mouse_down_point.get() - Point2D::new(x, y);
                        let pixel_dist = ((pixel_dist.x * pixel_dist.x +
                                           pixel_dist.y * pixel_dist.y) as f64).sqrt();
                        if pixel_dist < max_pixel_dist {
                            self.event_queue.borrow_mut().push(WindowEvent::MouseWindowEventClass(mouse_up_event));
                            MouseWindowEvent::Click(MouseButton::Left, Point2D::typed(x as f32, y as f32))
                        } else {
                            mouse_up_event
                        }
                    },
                    Some(_) => mouse_up_event,
                }
            }
        };
        self.event_queue.borrow_mut().push(WindowEvent::MouseWindowEventClass(event));
    }

    #[cfg(target_os="macos")]
    fn handle_next_event(&self) -> bool {
        let event = self.window.wait_events().next().unwrap();
        let mut close = self.handle_window_event(event);
        if !close {
            while let Some(event) = self.window.poll_events().next() {
                if self.handle_window_event(event) {
                    close = true;
                    break
                }
            }
        }
        close
    }

    #[cfg(any(target_os="linux", target_os="android"))]
    fn handle_next_event(&self) -> bool {
        use std::thread::sleep_ms;

        // TODO(gw): This is an awful hack to work around the
        // broken way we currently call X11 from multiple threads.
        //
        // On some (most?) X11 implementations, blocking here
        // with XPeekEvent results in the paint task getting stuck
        // in XGetGeometry randomly. When this happens the result
        // is that until you trigger the XPeekEvent to return
        // (by moving the mouse over the window) the paint task
        // never completes and you don't see the most recent
        // results.
        //
        // For now, poll events and sleep for ~1 frame if there
        // are no events. This means we don't spin the CPU at
        // 100% usage, but is far from ideal!
        //
        // See https://github.com/servo/servo/issues/5780
        //
        let first_event = self.window.poll_events().next();

        match first_event {
            Some(event) => {
                self.handle_window_event(event)
            }
            None => {
                sleep_ms(16);
                false
            }
        }
    }

    pub fn wait_events(&self) -> Vec<WindowEvent> {
        use std::mem;

        let mut events = mem::replace(&mut *self.event_queue.borrow_mut(), Vec::new());
        let mut close_event = false;

        // When writing to a file then exiting, use event
        // polling so that we don't block on a GUI event
        // such as mouse click.
        if opts::get().output_file.is_some() || opts::get().exit_after_load {
            while let Some(event) = self.window.poll_events().next() {
                close_event = self.handle_window_event(event) || close_event;
            }
        } else {
            close_event = self.handle_next_event();
        }

        if close_event {
            events.push(WindowEvent::Quit)
        }

        events.extend(mem::replace(&mut *self.event_queue.borrow_mut(), Vec::new()).into_iter());
        events
    }

    pub unsafe fn set_nested_event_loop_listener(
            &self,
            listener: *mut (NestedEventLoopListener + 'static)) {
        g_nested_event_loop_listener = Some(listener)
    }

    pub unsafe fn remove_nested_event_loop_listener(&self) {
        g_nested_event_loop_listener = None
    }

    fn glutin_key_to_script_key(key: glutin::VirtualKeyCode) -> Result<constellation_msg::Key, ()> {
        // TODO(negge): add more key mappings
        match key {
            VirtualKeyCode::A => Ok(Key::A),
            VirtualKeyCode::B => Ok(Key::B),
            VirtualKeyCode::C => Ok(Key::C),
            VirtualKeyCode::D => Ok(Key::D),
            VirtualKeyCode::E => Ok(Key::E),
            VirtualKeyCode::F => Ok(Key::F),
            VirtualKeyCode::G => Ok(Key::G),
            VirtualKeyCode::H => Ok(Key::H),
            VirtualKeyCode::I => Ok(Key::I),
            VirtualKeyCode::J => Ok(Key::J),
            VirtualKeyCode::K => Ok(Key::K),
            VirtualKeyCode::L => Ok(Key::L),
            VirtualKeyCode::M => Ok(Key::M),
            VirtualKeyCode::N => Ok(Key::N),
            VirtualKeyCode::O => Ok(Key::O),
            VirtualKeyCode::P => Ok(Key::P),
            VirtualKeyCode::Q => Ok(Key::Q),
            VirtualKeyCode::R => Ok(Key::R),
            VirtualKeyCode::S => Ok(Key::S),
            VirtualKeyCode::T => Ok(Key::T),
            VirtualKeyCode::U => Ok(Key::U),
            VirtualKeyCode::V => Ok(Key::V),
            VirtualKeyCode::W => Ok(Key::W),
            VirtualKeyCode::X => Ok(Key::X),
            VirtualKeyCode::Y => Ok(Key::Y),
            VirtualKeyCode::Z => Ok(Key::Z),

            VirtualKeyCode::Numpad0 => Ok(Key::Kp0),
            VirtualKeyCode::Numpad1 => Ok(Key::Kp1),
            VirtualKeyCode::Numpad2 => Ok(Key::Kp2),
            VirtualKeyCode::Numpad3 => Ok(Key::Kp3),
            VirtualKeyCode::Numpad4 => Ok(Key::Kp4),
            VirtualKeyCode::Numpad5 => Ok(Key::Kp5),
            VirtualKeyCode::Numpad6 => Ok(Key::Kp6),
            VirtualKeyCode::Numpad7 => Ok(Key::Kp7),
            VirtualKeyCode::Numpad8 => Ok(Key::Kp8),
            VirtualKeyCode::Numpad9 => Ok(Key::Kp9),

            VirtualKeyCode::Key0 => Ok(Key::Num0),
            VirtualKeyCode::Key1 => Ok(Key::Num1),
            VirtualKeyCode::Key2 => Ok(Key::Num2),
            VirtualKeyCode::Key3 => Ok(Key::Num3),
            VirtualKeyCode::Key4 => Ok(Key::Num4),
            VirtualKeyCode::Key5 => Ok(Key::Num5),
            VirtualKeyCode::Key6 => Ok(Key::Num6),
            VirtualKeyCode::Key7 => Ok(Key::Num7),
            VirtualKeyCode::Key8 => Ok(Key::Num8),
            VirtualKeyCode::Key9 => Ok(Key::Num9),

            VirtualKeyCode::Return => Ok(Key::Enter),
            VirtualKeyCode::Space => Ok(Key::Space),
            VirtualKeyCode::Escape => Ok(Key::Escape),
            VirtualKeyCode::Equals => Ok(Key::Equal),
            VirtualKeyCode::Minus => Ok(Key::Minus),
            VirtualKeyCode::Back => Ok(Key::Backspace),
            VirtualKeyCode::PageDown => Ok(Key::PageDown),
            VirtualKeyCode::PageUp => Ok(Key::PageUp),

            VirtualKeyCode::Insert => Ok(Key::Insert),
            VirtualKeyCode::Home => Ok(Key::Home),
            VirtualKeyCode::Delete => Ok(Key::Delete),
            VirtualKeyCode::End => Ok(Key::End),

            VirtualKeyCode::Left => Ok(Key::Left),
            VirtualKeyCode::Up => Ok(Key::Up),
            VirtualKeyCode::Right => Ok(Key::Right),
            VirtualKeyCode::Down => Ok(Key::Down),

            VirtualKeyCode::Apostrophe => Ok(Key::Apostrophe),
            VirtualKeyCode::Backslash => Ok(Key::Backslash),
            VirtualKeyCode::Comma => Ok(Key::Comma),
            VirtualKeyCode::Grave => Ok(Key::GraveAccent),
            VirtualKeyCode::LBracket => Ok(Key::LeftBracket),
            VirtualKeyCode::Period => Ok(Key::Period),
            VirtualKeyCode::RBracket => Ok(Key::RightBracket),
            VirtualKeyCode::Semicolon => Ok(Key::Semicolon),
            VirtualKeyCode::Slash => Ok(Key::Slash),
            VirtualKeyCode::Tab => Ok(Key::Tab),
            VirtualKeyCode::Subtract => Ok(Key::Minus),

            _ => Err(()),
        }
    }

    fn glutin_mods_to_script_mods(modifiers: KeyModifiers) -> constellation_msg::KeyModifiers {
        let mut result = constellation_msg::KeyModifiers::from_bits(0).unwrap();
        if modifiers.intersects(LEFT_SHIFT | RIGHT_SHIFT) {
            result.insert(SHIFT);
        }
        if modifiers.intersects(LEFT_CONTROL | RIGHT_CONTROL) {
            result.insert(CONTROL);
        }
        if modifiers.intersects(LEFT_ALT | RIGHT_ALT) {
            result.insert(ALT);
        }
        if modifiers.intersects(LEFT_SUPER | RIGHT_SUPER) {
            result.insert(SUPER);
        }
        result
    }

    #[cfg(all(feature = "window", not(target_os="win")))]
    fn platform_handle_key(&self, key: Key, mods: constellation_msg::KeyModifiers) {
        match (mods, key) {
            (CMD_OR_CONTROL, Key::LeftBracket) => {
                self.event_queue.borrow_mut().push(WindowEvent::Navigation(WindowNavigateMsg::Back));
            }
            (CMD_OR_CONTROL, Key::RightBracket) => {
                self.event_queue.borrow_mut().push(WindowEvent::Navigation(WindowNavigateMsg::Forward));
            }
            _ => {}
        }
    }

    #[cfg(all(feature = "window", target_os="win"))]
    fn platform_handle_key(&self, key: Key, mods: constellation_msg::KeyModifiers) {
    }
}

// WindowProxy is not implemented for android yet

#[cfg(all(feature = "window", target_os="android"))]
fn create_window_proxy(_: &Rc<Window>) -> Option<glutin::WindowProxy> {
    None
}

#[cfg(all(feature = "window", not(target_os="android")))]
fn create_window_proxy(window: &Rc<Window>) -> Option<glutin::WindowProxy> {
    Some(window.window.create_window_proxy())
}

#[cfg(feature = "window")]
impl WindowMethods for Window {
    fn framebuffer_size(&self) -> TypedSize2D<DevicePixel, u32> {
        let scale_factor = self.window.hidpi_factor() as u32;
        let (width, height) = self.window.get_inner_size().unwrap();
        Size2D::typed(width * scale_factor, height * scale_factor)
    }

    fn size(&self) -> TypedSize2D<ScreenPx, f32> {
        let (width, height) = self.window.get_inner_size().unwrap();
        Size2D::typed(width as f32, height as f32)
    }

    fn present(&self) {
        self.window.swap_buffers().unwrap();
    }

    fn create_compositor_channel(window: &Option<Rc<Window>>)
                                 -> (Box<CompositorProxy + Send>, Box<CompositorReceiver>) {
        let (sender, receiver) = channel();

        let window_proxy = match window {
            &Some(ref window) => create_window_proxy(window),
            &None => None,
        };

        (box GlutinCompositorProxy {
             sender: sender,
             window_proxy: window_proxy,
         } as Box<CompositorProxy + Send>,
         box receiver as Box<CompositorReceiver>)
    }

    fn hidpi_factor(&self) -> ScaleFactor<ScreenPx, DevicePixel, f32> {
        ScaleFactor::new(self.window.hidpi_factor())
    }

    fn set_page_title(&self, title: Option<String>) {
        let title = match title {
            Some(ref title) if title.len() > 0 => &**title,
            _ => "untitled",
        };
        let title = format!("{} - Servo", title);
        self.window.set_title(&title);
    }

    fn set_page_url(&self, _: Url) {
    }

    fn status(&self, _: Option<String>) {
    }

    fn load_start(&self, _: bool, _: bool) {
    }

    fn load_end(&self, _: bool, _: bool) {
    }

    fn load_error(&self, _: NetError, _: String) {
    }

    fn head_parsed(&self) {
    }

    /// Has no effect on Android.
    fn set_cursor(&self, c: Cursor) {
        use glutin::MouseCursor;

        let glutin_cursor = match c {
            Cursor::NoCursor => MouseCursor::NoneCursor,
            Cursor::DefaultCursor => MouseCursor::Default,
            Cursor::PointerCursor => MouseCursor::Hand,
            Cursor::ContextMenuCursor => MouseCursor::ContextMenu,
            Cursor::HelpCursor => MouseCursor::Help,
            Cursor::ProgressCursor => MouseCursor::Progress,
            Cursor::WaitCursor => MouseCursor::Wait,
            Cursor::CellCursor => MouseCursor::Cell,
            Cursor::CrosshairCursor => MouseCursor::Crosshair,
            Cursor::TextCursor => MouseCursor::Text,
            Cursor::VerticalTextCursor => MouseCursor::VerticalText,
            Cursor::AliasCursor => MouseCursor::Alias,
            Cursor::CopyCursor => MouseCursor::Copy,
            Cursor::MoveCursor => MouseCursor::Move,
            Cursor::NoDropCursor => MouseCursor::NoDrop,
            Cursor::NotAllowedCursor => MouseCursor::NotAllowed,
            Cursor::GrabCursor => MouseCursor::Grab,
            Cursor::GrabbingCursor => MouseCursor::Grabbing,
            Cursor::EResizeCursor => MouseCursor::EResize,
            Cursor::NResizeCursor => MouseCursor::NResize,
            Cursor::NeResizeCursor => MouseCursor::NeResize,
            Cursor::NwResizeCursor => MouseCursor::NwResize,
            Cursor::SResizeCursor => MouseCursor::SResize,
            Cursor::SeResizeCursor => MouseCursor::SeResize,
            Cursor::SwResizeCursor => MouseCursor::SwResize,
            Cursor::WResizeCursor => MouseCursor::WResize,
            Cursor::EwResizeCursor => MouseCursor::EwResize,
            Cursor::NsResizeCursor => MouseCursor::NsResize,
            Cursor::NeswResizeCursor => MouseCursor::NeswResize,
            Cursor::NwseResizeCursor => MouseCursor::NwseResize,
            Cursor::ColResizeCursor => MouseCursor::ColResize,
            Cursor::RowResizeCursor => MouseCursor::RowResize,
            Cursor::AllScrollCursor => MouseCursor::AllScroll,
            Cursor::ZoomInCursor => MouseCursor::ZoomIn,
            Cursor::ZoomOutCursor => MouseCursor::ZoomOut,
        };
        self.window.set_cursor(glutin_cursor);
    }

    fn set_favicon(&self, _: Url) {
    }

    fn prepare_for_composite(&self, _width: usize, _height: usize) -> bool {
        true
    }

    #[cfg(target_os="linux")]
    fn native_display(&self) -> NativeDisplay {
        use x11::xlib;
        unsafe {
            NativeDisplay::new(self.window.platform_display() as *mut xlib::Display)
        }
    }

    #[cfg(not(target_os="linux"))]
    fn native_display(&self) -> NativeDisplay {
        NativeDisplay::new()
    }

    /// Helper function to handle keyboard events.
    fn handle_key(&self, key: Key, mods: constellation_msg::KeyModifiers) {

        match (mods, key) {
            (_, Key::Equal) if mods & !SHIFT == CMD_OR_CONTROL => {
                self.event_queue.borrow_mut().push(WindowEvent::Zoom(1.1));
            }
            (CMD_OR_CONTROL, Key::Minus) => {
                self.event_queue.borrow_mut().push(WindowEvent::Zoom(1.0 / 1.1));
            }
            (CMD_OR_CONTROL, Key::Num0) |
            (CMD_OR_CONTROL, Key::Kp0) => {
                self.event_queue.borrow_mut().push(WindowEvent::ResetZoom);
            }

            (SHIFT, Key::Backspace) => {
                self.event_queue.borrow_mut().push(WindowEvent::Navigation(WindowNavigateMsg::Forward));
            }
            (NONE, Key::Backspace) => {
                self.event_queue.borrow_mut().push(WindowEvent::Navigation(WindowNavigateMsg::Back));
            }

            (CMD_OR_ALT, Key::Right) => {
                self.event_queue.borrow_mut().push(WindowEvent::Navigation(WindowNavigateMsg::Forward));
            }
            (CMD_OR_ALT, Key::Left) => {
                self.event_queue.borrow_mut().push(WindowEvent::Navigation(WindowNavigateMsg::Back));
            }

            (NONE, Key::PageDown) |
            (NONE, Key::Space) => {
                self.scroll_window(0.0, -self.framebuffer_size().as_f32().to_untyped().height + 2.0 * LINE_HEIGHT);
            }
            (NONE, Key::PageUp) |
            (SHIFT, Key::Space) => {
                self.scroll_window(0.0, self.framebuffer_size().as_f32().to_untyped().height - 2.0 * LINE_HEIGHT);
            }
            (NONE, Key::Up) => {
                self.scroll_window(0.0, 3.0 * LINE_HEIGHT);
            }
            (NONE, Key::Down) => {
                self.scroll_window(0.0, -3.0 * LINE_HEIGHT);
            }

            _ => {
                self.platform_handle_key(key, mods);
            }
        }
    }

    fn supports_clipboard(&self) -> bool {
        true
    }
}

/// The type of a window.
#[cfg(feature = "headless")]
pub struct Window {
    #[allow(dead_code)]
    context: glutin::HeadlessContext,
    width: u32,
    height: u32,
}

#[cfg(feature = "headless")]
impl Window {
    pub fn new(_is_foreground: bool,
               window_size: TypedSize2D<DevicePixel, u32>,
               _parent: glutin::WindowID) -> Rc<Window> {
        let window_size = window_size.to_untyped();
        let headless_builder = glutin::HeadlessRendererBuilder::new(window_size.width,
                                                                    window_size.height);
        let headless_context = headless_builder.build().unwrap();
        unsafe { headless_context.make_current() };

        gl::load_with(|s| headless_context.get_proc_address(s));

        let window = Window {
            context: headless_context,
            width: window_size.width,
            height: window_size.height,
        };

        Rc::new(window)
    }

    pub fn wait_events(&self) -> Vec<WindowEvent> {
        vec![WindowEvent::Idle]
    }

    pub unsafe fn set_nested_event_loop_listener(
            &self,
            _listener: *mut (NestedEventLoopListener + 'static)) {
    }

    pub unsafe fn remove_nested_event_loop_listener(&self) {
    }
}

#[cfg(feature = "headless")]
impl WindowMethods for Window {
    fn framebuffer_size(&self) -> TypedSize2D<DevicePixel, u32> {
        Size2D::typed(self.width, self.height)
    }

    fn size(&self) -> TypedSize2D<ScreenPx, f32> {
        Size2D::typed(self.width as f32, self.height as f32)
    }

    fn present(&self) {
    }

    fn create_compositor_channel(_: &Option<Rc<Window>>)
                                 -> (Box<CompositorProxy + Send>, Box<CompositorReceiver>) {
        let (sender, receiver) = channel();

        (box GlutinCompositorProxy {
             sender: sender,
             window_proxy: None,
         } as Box<CompositorProxy + Send>,
         box receiver as Box<CompositorReceiver>)
    }

    fn hidpi_factor(&self) -> ScaleFactor<ScreenPx, DevicePixel, f32> {
        ScaleFactor::new(1.0)
    }

    fn set_page_title(&self, _: Option<String>) {
    }

    fn set_page_url(&self, _: Url) {
    }

    fn load_start(&self, _: bool, _: bool) {
    }
    fn load_end(&self, _: bool, _: bool) {
    }
    fn load_error(&self, _: NetError, _: String) {
    }
    fn head_parsed(&self) {
    }

    fn set_cursor(&self, _: Cursor) {
    }

    fn set_favicon(&self, _: Url) {
    }

    fn status(&self, _: Option<String>) {
    }

    fn prepare_for_composite(&self, _width: usize, _height: usize) -> bool {
        true
    }

    #[cfg(target_os="linux")]
    fn native_display(&self) -> NativeDisplay {
        NativeDisplay::new(ptr::null_mut())
    }

    /// Helper function to handle keyboard events.
    fn handle_key(&self, _: Key, _: constellation_msg::KeyModifiers) {
    }

    fn supports_clipboard(&self) -> bool {
        false
    }
}

struct GlutinCompositorProxy {
    sender: Sender<compositor_task::Msg>,
    window_proxy: Option<glutin::WindowProxy>,
}

// TODO: Should this be implemented here or upstream in glutin::WindowProxy?
unsafe impl Send for GlutinCompositorProxy {}

impl CompositorProxy for GlutinCompositorProxy {
    fn send(&self, msg: compositor_task::Msg) {
        // Send a message and kick the OS event loop awake.
        self.sender.send(msg).unwrap();
        if let Some(ref window_proxy) = self.window_proxy {
            window_proxy.wakeup_event_loop()
        }
    }
    fn clone_compositor_proxy(&self) -> Box<CompositorProxy + Send> {
        box GlutinCompositorProxy {
            sender: self.sender.clone(),
            window_proxy: self.window_proxy.clone(),
        } as Box<CompositorProxy + Send>
    }
}

// These functions aren't actually called. They are here as a link
// hack because Skia references them.

#[allow(non_snake_case)]
#[no_mangle]
pub extern "C" fn glBindVertexArrayOES(_array: usize)
{
    unimplemented!()
}

#[allow(non_snake_case)]
#[no_mangle]
pub extern "C" fn glDeleteVertexArraysOES(_n: isize, _arrays: *const ())
{
    unimplemented!()
}

#[allow(non_snake_case)]
#[no_mangle]
pub extern "C" fn glGenVertexArraysOES(_n: isize, _arrays: *const ())
{
    unimplemented!()
}

#[allow(non_snake_case)]
#[no_mangle]
pub extern "C" fn glRenderbufferStorageMultisampleIMG(_: isize, _: isize, _: isize, _: isize, _: isize)
{
    unimplemented!()
}

#[allow(non_snake_case)]
#[no_mangle]
pub extern "C" fn glFramebufferTexture2DMultisampleIMG(_: isize, _: isize, _: isize, _: isize, _: isize, _: isize)
{
    unimplemented!()
}

#[allow(non_snake_case)]
#[no_mangle]
pub extern "C" fn glDiscardFramebufferEXT(_: isize, _: isize, _: *const ())
{
    unimplemented!()
}

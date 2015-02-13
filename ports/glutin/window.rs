/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! A windowing implementation using glutin.

use compositing::compositor_task::{self, CompositorProxy, CompositorReceiver};
use compositing::windowing::{WindowEvent, WindowMethods};
use geom::scale_factor::ScaleFactor;
use geom::size::TypedSize2D;
use gleam::gl;
use glutin;
use layers::geometry::DevicePixel;
use layers::platform::surface::NativeGraphicsMetadata;
use msg::constellation_msg;
use msg::constellation_msg::Key;
use msg::compositor_msg::{PaintState, ReadyState};
use msg::constellation_msg::LoadData;
use NestedEventLoopListener;
use std::rc::Rc;
use std::sync::mpsc::{channel, Sender};
use util::cursor::Cursor;
use util::geometry::ScreenPx;

#[cfg(feature = "window")]
use compositing::windowing::{MouseWindowEvent, WindowNavigateMsg};
#[cfg(feature = "window")]
use geom::point::{Point2D, TypedPoint2D};
#[cfg(feature = "window")]
use glutin::{ElementState, Event, MouseButton, VirtualKeyCode};
#[cfg(feature = "window")]
use msg::constellation_msg::{KeyState, CONTROL, SHIFT, ALT};
#[cfg(feature = "window")]
use std::cell::{Cell, RefCell};
#[cfg(feature = "window")]
use std::num::Float;
#[cfg(feature = "window")]
use time::{self, Timespec};
#[cfg(feature = "window")]
use util::opts;

#[cfg(all(feature = "headless", target_os="linux"))]
use std::ptr;

#[cfg(feature = "window")]
static mut g_nested_event_loop_listener: Option<*mut (NestedEventLoopListener + 'static)> = None;

#[cfg(feature = "window")]
bitflags! {
    #[derive(Debug)]
    flags KeyModifiers: u8 {
        const LEFT_CONTROL = 1,
        const RIGHT_CONTROL = 2,
        const LEFT_SHIFT = 4,
        const RIGHT_SHIFT = 8,
        const LEFT_ALT = 16,
        const RIGHT_ALT = 32,
    }
}

/// The type of a window.
#[cfg(feature = "window")]
pub struct Window {
    window: glutin::Window,

    mouse_down_button: Cell<Option<glutin::MouseButton>>,
    mouse_down_point: Cell<Point2D<i32>>,
    event_queue: RefCell<Vec<WindowEvent>>,

    mouse_pos: Cell<Point2D<i32>>,
    ready_state: Cell<ReadyState>,
    paint_state: Cell<PaintState>,
    key_modifiers: Cell<KeyModifiers>,

    last_title_set_time: Cell<Timespec>,
}

#[cfg(feature = "window")]
impl Window {
    pub fn new(is_foreground: bool, window_size: TypedSize2D<DevicePixel, u32>) -> Rc<Window> {
        let mut glutin_window = glutin::WindowBuilder::new()
                            .with_title("Servo [glutin]".to_string())
                            .with_dimensions(window_size.to_untyped().width, window_size.to_untyped().height)
                            .with_gl_version(Window::gl_version())
                            .with_visibility(is_foreground)
                            .build()
                            .unwrap();
        unsafe { glutin_window.make_current() };

        glutin_window.set_window_resize_callback(Some(Window::nested_window_resize as fn(u32, u32)));

        Window::load_gl_functions(&glutin_window);

        let window = Window {
            window: glutin_window,
            event_queue: RefCell::new(vec!()),
            mouse_down_button: Cell::new(None),
            mouse_down_point: Cell::new(Point2D(0, 0)),

            mouse_pos: Cell::new(Point2D(0, 0)),
            ready_state: Cell::new(ReadyState::Blank),
            paint_state: Cell::new(PaintState::Idle),
            key_modifiers: Cell::new(KeyModifiers::empty()),

            last_title_set_time: Cell::new(Timespec::new(0, 0)),
        };

        gl::clear_color(0.6, 0.6, 0.6, 1.0);
        gl::clear(gl::COLOR_BUFFER_BIT);
        gl::finish();
        window.present();

        Rc::new(window)
    }

    fn nested_window_resize(width: u32, height: u32) {
        unsafe {
            match g_nested_event_loop_listener {
                None => {}
                Some(listener) => {
                    (*listener).handle_event_from_nested_event_loop(WindowEvent::Resize(TypedSize2D(width, height)));
                }
            }
        }
    }

    #[cfg(not(target_os="android"))]
    fn gl_version() -> (u32, u32) {
        (3, 0)
    }

    #[cfg(target_os="android")]
    fn gl_version() -> (u32, u32) {
        (2, 0)
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
                        (ElementState::Pressed, VirtualKeyCode::Escape) => return true,
                        (ElementState::Pressed, key_code) => {
                            match Window::glutin_key_to_script_key(key_code) {
                                Ok(key) => {
                                    let state = KeyState::Pressed;
                                    let modifiers = Window::glutin_mods_to_script_mods(self.key_modifiers.get());
                                    self.event_queue.borrow_mut().push(WindowEvent::KeyEvent(key, state, modifiers));
                                }
                                _ => {}
                            }
                        }
                        (_, _) => {}
                    }
                }
            }
            Event::Resized(width, height) => {
                self.event_queue.borrow_mut().push(WindowEvent::Resize(TypedSize2D(width, height)));
            }
            Event::MouseInput(element_state, mouse_button) => {
                if mouse_button == MouseButton::LeftMouseButton ||
                                    mouse_button == MouseButton::RightMouseButton {
                        let mouse_pos = self.mouse_pos.get();
                        self.handle_mouse(mouse_button, element_state, mouse_pos.x, mouse_pos.y);
                   }
            }
            Event::MouseMoved((x, y)) => {
                self.mouse_pos.set(Point2D(x, y));
                self.event_queue.borrow_mut().push(
                    WindowEvent::MouseWindowMoveEventClass(TypedPoint2D(x as f32, y as f32)));
            }
            Event::MouseWheel(delta) => {
                if self.ctrl_pressed() {
                    // Ctrl-Scrollwheel simulates a "pinch zoom" gesture.
                    if delta < 0 {
                        self.event_queue.borrow_mut().push(WindowEvent::PinchZoom(1.0/1.1));
                    } else if delta > 0 {
                        self.event_queue.borrow_mut().push(WindowEvent::PinchZoom(1.1));
                    }
                } else {
                    let dx = 0.0;
                    let dy = (delta as f32) * 30.0;
                    self.scroll_window(dx, dy);
                }
            },
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
        let event = WindowEvent::Scroll(TypedPoint2D(dx as f32, dy as f32),
                                 TypedPoint2D(mouse_pos.x as i32, mouse_pos.y as i32));
        self.event_queue.borrow_mut().push(event);
    }

    /// Helper function to handle a click
    fn handle_mouse(&self, button: glutin::MouseButton, action: glutin::ElementState, x: i32, y: i32) {
        // FIXME(tkuehn): max pixel dist should be based on pixel density
        let max_pixel_dist = 10f64;
        let event = match action {
            ElementState::Pressed => {
                self.mouse_down_point.set(Point2D(x, y));
                self.mouse_down_button.set(Some(button));
                MouseWindowEvent::MouseDown(0, TypedPoint2D(x as f32, y as f32))
            }
            ElementState::Released => {
                match self.mouse_down_button.get() {
                    None => (),
                    Some(but) if button == but => {
                        let pixel_dist = self.mouse_down_point.get() - Point2D(x, y);
                        let pixel_dist = ((pixel_dist.x * pixel_dist.x +
                                           pixel_dist.y * pixel_dist.y) as f64).sqrt();
                        if pixel_dist < max_pixel_dist {
                            let click_event = MouseWindowEvent::Click(
                                0, TypedPoint2D(x as f32, y as f32));
                            self.event_queue.borrow_mut().push(WindowEvent::MouseWindowEventClass(click_event));
                        }
                    }
                    Some(_) => (),
                }
                MouseWindowEvent::MouseUp(0, TypedPoint2D(x as f32, y as f32))
            }
        };
        self.event_queue.borrow_mut().push(WindowEvent::MouseWindowEventClass(event));
    }

    fn update_window_title(&self) {
        let now = time::get_time();
        if now.sec == self.last_title_set_time.get().sec {
            return
        }
        self.last_title_set_time.set(now);

        match self.ready_state.get() {
            ReadyState::Blank => {
                self.window.set_title("blank - Servo [glutin]")
            }
            ReadyState::Loading => {
                self.window.set_title("Loading - Servo [glutin]")
            }
            ReadyState::PerformingLayout => {
                self.window.set_title("Performing Layout - Servo [glutin]")
            }
            ReadyState::FinishedLoading => {
                match self.paint_state.get() {
                    PaintState::Painting => {
                        self.window.set_title("Rendering - Servo [glutin]")
                    }
                    PaintState::Idle => {
                        self.window.set_title("Servo [glutin]")
                    }
                }
            }
        }
    }

    pub fn wait_events(&self) -> WindowEvent {
        {
            let mut event_queue = self.event_queue.borrow_mut();
            if !event_queue.is_empty() {
                return event_queue.remove(0);
            }
        }

        let mut close_event = false;

        // When writing to a file then exiting, use event
        // polling so that we don't block on a GUI event
        // such as mouse click.
        if opts::get().output_file.is_some() {
            for event in self.window.poll_events() {
                close_event = self.handle_window_event(event);
                if close_event {
                    break;
                }
            }
        } else {
            // GWTODO: Something has changed in the wait_events
            // behaviour in glutin. Switching to poll events
            // for now, so that things display correctly,
            // need to fix glutin / handle the changed behaviour.
            for event in self.window.poll_events() {
                close_event = self.handle_window_event(event);
                if close_event {
                    break;
                }
            }
        }

        if close_event || self.window.is_closed() {
            WindowEvent::Quit
        } else {
            let mut event_queue = self.event_queue.borrow_mut();
            if event_queue.is_empty() {
                WindowEvent::Idle
            } else {
                event_queue.remove(0)
            }
        }
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

            VirtualKeyCode::Numpad0 => Ok(Key::Num0),
            VirtualKeyCode::Numpad1 => Ok(Key::Num1),
            VirtualKeyCode::Numpad2 => Ok(Key::Num2),
            VirtualKeyCode::Numpad3 => Ok(Key::Num3),
            VirtualKeyCode::Numpad4 => Ok(Key::Num4),
            VirtualKeyCode::Numpad5 => Ok(Key::Num5),
            VirtualKeyCode::Numpad6 => Ok(Key::Num6),
            VirtualKeyCode::Numpad7 => Ok(Key::Num7),
            VirtualKeyCode::Numpad8 => Ok(Key::Num8),
            VirtualKeyCode::Numpad9 => Ok(Key::Num9),

            VirtualKeyCode::Key0 => Ok(Key::Kp0),
            VirtualKeyCode::Key1 => Ok(Key::Kp1),
            VirtualKeyCode::Key2 => Ok(Key::Kp2),
            VirtualKeyCode::Key3 => Ok(Key::Kp3),
            VirtualKeyCode::Key4 => Ok(Key::Kp4),
            VirtualKeyCode::Key5 => Ok(Key::Kp5),
            VirtualKeyCode::Key6 => Ok(Key::Kp6),
            VirtualKeyCode::Key7 => Ok(Key::Kp7),
            VirtualKeyCode::Key8 => Ok(Key::Kp8),
            VirtualKeyCode::Key9 => Ok(Key::Kp9),

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
        result
    }
}

#[cfg(feature = "window")]
impl WindowMethods for Window {
    fn framebuffer_size(&self) -> TypedSize2D<DevicePixel, u32> {
        let scale_factor = self.window.hidpi_factor() as u32;
        let (width, height) = self.window.get_inner_size().unwrap();
        TypedSize2D(width * scale_factor, height * scale_factor)
    }

    fn size(&self) -> TypedSize2D<ScreenPx, f32> {
        let (width, height) = self.window.get_inner_size().unwrap();
        TypedSize2D(width as f32, height as f32)
    }

    fn present(&self) {
        self.window.swap_buffers()
    }

    fn create_compositor_channel(window: &Option<Rc<Window>>)
                                 -> (Box<CompositorProxy+Send>, Box<CompositorReceiver>) {
        let (sender, receiver) = channel();

        let window_proxy = match window {
            &Some(ref window) => Some(window.window.create_window_proxy()),
            &None => None,
        };

        (box GlutinCompositorProxy {
             sender: sender,
             window_proxy: window_proxy,
         } as Box<CompositorProxy+Send>,
         box receiver as Box<CompositorReceiver>)
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
        ScaleFactor(self.window.hidpi_factor())
    }

    fn set_page_title(&self, _: Option<String>) {
    }

    fn set_page_load_data(&self, _: LoadData) {
    }

    fn load_end(&self) {
    }

    fn set_cursor(&self, _: Cursor) {
    }

    fn prepare_for_composite(&self) -> bool {
        true
    }

    #[cfg(target_os="linux")]
    fn native_metadata(&self) -> NativeGraphicsMetadata {
        NativeGraphicsMetadata {
            display: unsafe { self.window.platform_display() }
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

    #[cfg(target_os="android")]
    fn native_metadata(&self) -> NativeGraphicsMetadata {
        use egl::egl::GetCurrentDisplay;
        NativeGraphicsMetadata {
            display: GetCurrentDisplay(),
        }
    }

    /// Helper function to handle keyboard events.
    fn handle_key(&self, key: Key, mods: constellation_msg::KeyModifiers) {
        match key {
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
                self.scroll_window(0.0, -self.framebuffer_size().as_f32().to_untyped().height);
            }
            Key::PageUp => {
                self.scroll_window(0.0, self.framebuffer_size().as_f32().to_untyped().height);
            }
            _ => {}
        }
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
    pub fn new(_is_foreground: bool, window_size: TypedSize2D<DevicePixel, u32>) -> Rc<Window> {
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

    pub fn wait_events(&self) -> WindowEvent {
        WindowEvent::Idle
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
        TypedSize2D(self.width, self.height)
    }

    fn size(&self) -> TypedSize2D<ScreenPx, f32> {
        TypedSize2D(self.width as f32, self.height as f32)
    }

    fn present(&self) {
    }

    fn create_compositor_channel(_: &Option<Rc<Window>>)
                                 -> (Box<CompositorProxy+Send>, Box<CompositorReceiver>) {
        let (sender, receiver) = channel();

        (box GlutinCompositorProxy {
             sender: sender,
             window_proxy: None,
         } as Box<CompositorProxy+Send>,
         box receiver as Box<CompositorReceiver>)
    }

    /// Sets the ready state.
    fn set_ready_state(&self, _: ReadyState) {
    }

    /// Sets the paint state.
    fn set_paint_state(&self, _: PaintState) {
    }

    fn hidpi_factor(&self) -> ScaleFactor<ScreenPx, DevicePixel, f32> {
        ScaleFactor(1.0)
    }

    fn set_page_title(&self, _: Option<String>) {
    }

    fn set_page_load_data(&self, _: LoadData) {
    }

    fn load_end(&self) {
    }

    fn set_cursor(&self, _: Cursor) {
    }

    fn prepare_for_composite(&self) -> bool {
        true
    }

    #[cfg(target_os="linux")]
    fn native_metadata(&self) -> NativeGraphicsMetadata {
        NativeGraphicsMetadata {
            display: ptr::null_mut()
        }
    }

    /// Helper function to handle keyboard events.
    fn handle_key(&self, _: Key, _: constellation_msg::KeyModifiers) {
    }
}

struct GlutinCompositorProxy {
    sender: Sender<compositor_task::Msg>,
    window_proxy: Option<glutin::WindowProxy>,
}

// TODO: Should this be implemented here or upstream in glutin::WindowProxy?
unsafe impl Send for GlutinCompositorProxy {}

impl CompositorProxy for GlutinCompositorProxy {
    fn send(&mut self, msg: compositor_task::Msg) {
        // Send a message and kick the OS event loop awake.
        self.sender.send(msg).unwrap();
        match self.window_proxy {
            Some(ref window_proxy) => window_proxy.wakeup_event_loop(),
            None => {}
        }
    }
    fn clone_compositor_proxy(&self) -> Box<CompositorProxy+Send> {
        box GlutinCompositorProxy {
            sender: self.sender.clone(),
            window_proxy: self.window_proxy.clone(),
        } as Box<CompositorProxy+Send>
    }
}

// These functions aren't actually called. They are here as a link
// hack because Skia references them.

#[allow(non_snake_case)]
#[no_mangle]
pub extern "C" fn glBindVertexArrayOES(_array: uint)
{
    unimplemented!()
}

#[allow(non_snake_case)]
#[no_mangle]
pub extern "C" fn glDeleteVertexArraysOES(_n: int, _arrays: *const ())
{
    unimplemented!()
}

#[allow(non_snake_case)]
#[no_mangle]
pub extern "C" fn glGenVertexArraysOES(_n: int, _arrays: *const ())
{
    unimplemented!()
}

#[allow(non_snake_case)]
#[no_mangle]
pub extern "C" fn glRenderbufferStorageMultisampleIMG(_: int, _: int, _: int, _: int, _: int)
{
    unimplemented!()
}

#[allow(non_snake_case)]
#[no_mangle]
pub extern "C" fn glFramebufferTexture2DMultisampleIMG(_: int, _: int, _: int, _: int, _: int, _: int)
{
    unimplemented!()
}

#[allow(non_snake_case)]
#[no_mangle]
pub extern "C" fn glDiscardFramebufferEXT(_: int, _: int, _: *const ())
{
    unimplemented!()
}

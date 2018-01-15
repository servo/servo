/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! A windowing implementation using glutin.

use NestedEventLoopListener;
use compositing::compositor_thread::EventLoopWaker;
use compositing::windowing::{AnimationState, MouseWindowEvent};
use compositing::windowing::{WebRenderDebugOption, WindowEvent, WindowMethods};
use euclid::{Point2D, Size2D, TypedPoint2D, TypedVector2D, TypedScale, TypedSize2D};
#[cfg(target_os = "windows")]
use gdi32;
use gleam::gl;
use glutin;
use glutin::{Api, ElementState, Event, GlRequest, MouseButton, MouseScrollDelta, VirtualKeyCode};
#[cfg(not(target_os = "windows"))]
use glutin::ScanCode;
use glutin::TouchPhase;
#[cfg(target_os = "macos")]
use glutin::os::macos::{ActivationPolicy, WindowBuilderExt};
use msg::constellation_msg::{self, Key, TopLevelBrowsingContextId as BrowserId};
use msg::constellation_msg::{KeyModifiers, KeyState, TraversalDirection};
use net_traits::net_error_list::NetError;
#[cfg(any(target_os = "linux", target_os = "macos"))]
use osmesa_sys;
use script_traits::{LoadData, TouchEventType, TouchpadPressurePhase};
use servo::ipc_channel::ipc::IpcSender;
use servo_config::opts;
use servo_config::prefs::PREFS;
use servo_config::resource_files;
use servo_geometry::DeviceIndependentPixel;
use servo_url::ServoUrl;
use std::cell::{Cell, RefCell};
#[cfg(any(target_os = "linux", target_os = "macos"))]
use std::ffi::CString;
#[cfg(any(target_os = "linux", target_os = "macos"))]
use std::mem;
use std::os::raw::c_void;
use std::ptr;
use std::rc::Rc;
use std::thread;
use std::time;
use style_traits::DevicePixel;
use style_traits::cursor::CursorKind;
#[cfg(target_os = "windows")]
use user32;
use webrender_api::{DeviceUintRect, DeviceUintSize, ScrollLocation};
#[cfg(target_os = "windows")]
use winapi;

static mut G_NESTED_EVENT_LOOP_LISTENER: Option<*mut (NestedEventLoopListener + 'static)> = None;

bitflags! {
    struct GlutinKeyModifiers: u8 {
        const LEFT_CONTROL = 1;
        const RIGHT_CONTROL = 2;
        const LEFT_SHIFT = 4;
        const RIGHT_SHIFT = 8;
        const LEFT_ALT = 16;
        const RIGHT_ALT = 32;
        const LEFT_SUPER = 64;
        const RIGHT_SUPER = 128;
    }
}

// Some shortcuts use Cmd on Mac and Control on other systems.
#[cfg(target_os = "macos")]
const CMD_OR_CONTROL: KeyModifiers = KeyModifiers::SUPER;
#[cfg(not(target_os = "macos"))]
const CMD_OR_CONTROL: KeyModifiers = KeyModifiers::CONTROL;

// Some shortcuts use Cmd on Mac and Alt on other systems.
#[cfg(target_os = "macos")]
const CMD_OR_ALT: KeyModifiers = KeyModifiers::SUPER;
#[cfg(not(target_os = "macos"))]
const CMD_OR_ALT: KeyModifiers = KeyModifiers::ALT;

// This should vary by zoom level and maybe actual text size (focused or under cursor)
const LINE_HEIGHT: f32 = 38.0;

const MULTISAMPLES: u16 = 16;

#[cfg(target_os = "macos")]
fn builder_with_platform_options(mut builder: glutin::WindowBuilder) -> glutin::WindowBuilder {
    if opts::get().headless || opts::get().output_file.is_some() {
        // Prevent the window from showing in Dock.app, stealing focus,
        // or appearing at all when running in headless mode or generating an
        // output file.
        builder = builder.with_activation_policy(ActivationPolicy::Prohibited)
    }
    builder.with_app_name(String::from("Servo"))
}

#[cfg(not(target_os = "macos"))]
fn builder_with_platform_options(builder: glutin::WindowBuilder) -> glutin::WindowBuilder {
    builder
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
struct HeadlessContext {
    width: u32,
    height: u32,
    _context: osmesa_sys::OSMesaContext,
    _buffer: Vec<u32>,
}

#[cfg(not(any(target_os = "linux", target_os = "macos")))]
struct HeadlessContext {
    width: u32,
    height: u32,
}

impl HeadlessContext {
    #[cfg(any(target_os = "linux", target_os = "macos"))]
    fn new(width: u32, height: u32) -> HeadlessContext {
        let mut attribs = Vec::new();

        attribs.push(osmesa_sys::OSMESA_PROFILE);
        attribs.push(osmesa_sys::OSMESA_CORE_PROFILE);
        attribs.push(osmesa_sys::OSMESA_CONTEXT_MAJOR_VERSION);
        attribs.push(3);
        attribs.push(osmesa_sys::OSMESA_CONTEXT_MINOR_VERSION);
        attribs.push(3);
        attribs.push(0);

        let context = unsafe {
            osmesa_sys::OSMesaCreateContextAttribs(attribs.as_ptr(), ptr::null_mut())
        };

        assert!(!context.is_null());

        let mut buffer = vec![0; (width * height) as usize];

        unsafe {
            let ret = osmesa_sys::OSMesaMakeCurrent(context,
                                                    buffer.as_mut_ptr() as *mut _,
                                                    gl::UNSIGNED_BYTE,
                                                    width as i32,
                                                    height as i32);
            assert!(ret != 0);
        };

        HeadlessContext {
            width: width,
            height: height,
            _context: context,
            _buffer: buffer,
        }
    }

    #[cfg(not(any(target_os = "linux", target_os = "macos")))]
    fn new(width: u32, height: u32) -> HeadlessContext {
        HeadlessContext {
            width: width,
            height: height,
        }
    }

    #[cfg(any(target_os = "linux", target_os = "macos"))]
    fn get_proc_address(s: &str) -> *const c_void {
        let c_str = CString::new(s).expect("Unable to create CString");
        unsafe {
            mem::transmute(osmesa_sys::OSMesaGetProcAddress(c_str.as_ptr()))
        }
    }

    #[cfg(not(any(target_os = "linux", target_os = "macos")))]
    fn get_proc_address(_: &str) -> *const c_void {
        ptr::null() as *const _
    }
}

enum WindowKind {
    Window(glutin::Window),
    Headless(HeadlessContext),
}

/// The type of a window.
pub struct Window {
    kind: WindowKind,

    mouse_down_button: Cell<Option<glutin::MouseButton>>,
    mouse_down_point: Cell<Point2D<i32>>,
    event_queue: RefCell<Vec<WindowEvent>>,

    /// id of the top level browsing context. It is unique as tabs
    /// are not supported yet. None until created.
    browser_id: Cell<Option<BrowserId>>,

    mouse_pos: Cell<Point2D<i32>>,
    key_modifiers: Cell<GlutinKeyModifiers>,
    current_url: RefCell<Option<ServoUrl>>,

    #[cfg(not(target_os = "windows"))]
    /// The contents of the last ReceivedCharacter event for use in a subsequent KeyEvent.
    pending_key_event_char: Cell<Option<char>>,

    #[cfg(target_os = "windows")]
    last_pressed_key: Cell<Option<constellation_msg::Key>>,

    /// The list of keys that have been pressed but not yet released, to allow providing
    /// the equivalent ReceivedCharacter data as was received for the press event.
    #[cfg(not(target_os = "windows"))]
    pressed_key_map: RefCell<Vec<(ScanCode, char)>>,

    animation_state: Cell<AnimationState>,

    gl: Rc<gl::Gl>,
}

#[cfg(not(target_os = "windows"))]
fn window_creation_scale_factor() -> TypedScale<f32, DeviceIndependentPixel, DevicePixel> {
    TypedScale::new(1.0)
}

#[cfg(target_os = "windows")]
fn window_creation_scale_factor() -> TypedScale<f32, DeviceIndependentPixel, DevicePixel> {
        let hdc = unsafe { user32::GetDC(::std::ptr::null_mut()) };
        let ppi = unsafe { gdi32::GetDeviceCaps(hdc, winapi::wingdi::LOGPIXELSY) };
        TypedScale::new(ppi as f32 / 96.0)
}


impl Window {
    pub fn set_browser_id(&self, browser_id: BrowserId) {
        self.browser_id.set(Some(browser_id));
    }

    pub fn new(is_foreground: bool,
               window_size: TypedSize2D<u32, DeviceIndependentPixel>,
               parent: Option<glutin::WindowID>) -> Rc<Window> {
        let win_size: TypedSize2D<u32, DevicePixel> =
            (window_size.to_f32() * window_creation_scale_factor())
                .to_usize().cast().expect("Window size should fit in u32");
        let width = win_size.to_untyped().width;
        let height = win_size.to_untyped().height;

        // If there's no chrome, start off with the window invisible. It will be set to visible in
        // `load_end()`. This avoids an ugly flash of unstyled content (especially important since
        // unstyled content is white and chrome often has a transparent background). See issue
        // #9996.
        let visible = is_foreground && !opts::get().no_native_titlebar;

        let window_kind = if opts::get().headless {
            WindowKind::Headless(HeadlessContext::new(width, height))
        } else {
            let mut builder =
                glutin::WindowBuilder::new().with_title("Servo".to_string())
                                            .with_decorations(!opts::get().no_native_titlebar)
                                            .with_transparency(opts::get().no_native_titlebar)
                                            .with_dimensions(width, height)
                                            .with_gl(Window::gl_version())
                                            .with_visibility(visible)
                                            .with_parent(parent)
                                            .with_multitouch();

            if let Ok(mut icon_path) = resource_files::resources_dir_path() {
                icon_path.push("servo.png");
                builder = builder.with_icon(icon_path);
            }

            if opts::get().enable_vsync {
                builder = builder.with_vsync();
            }

            if opts::get().use_msaa {
                builder = builder.with_multisampling(MULTISAMPLES)
            }

            builder = builder_with_platform_options(builder);

            let mut glutin_window = builder.build().expect("Failed to create window.");

            unsafe { glutin_window.make_current().expect("Failed to make context current!") }

            glutin_window.set_window_resize_callback(Some(Window::nested_window_resize as fn(u32, u32)));

            WindowKind::Window(glutin_window)
        };

        let gl = match window_kind {
            WindowKind::Window(ref window) => {
                match gl::GlType::default() {
                    gl::GlType::Gl => {
                        unsafe {
                            gl::GlFns::load_with(|s| window.get_proc_address(s) as *const _)
                        }
                    }
                    gl::GlType::Gles => {
                        unsafe {
                            gl::GlesFns::load_with(|s| window.get_proc_address(s) as *const _)
                        }
                    }
                }
            }
            WindowKind::Headless(..) => {
                unsafe {
                    gl::GlFns::load_with(|s| HeadlessContext::get_proc_address(s))
                }
            }
        };

        if opts::get().headless {
            // Print some information about the headless renderer that
            // can be useful in diagnosing CI failures on build machines.
            println!("{}", gl.get_string(gl::VENDOR));
            println!("{}", gl.get_string(gl::RENDERER));
            println!("{}", gl.get_string(gl::VERSION));
        }

        gl.clear_color(0.6, 0.6, 0.6, 1.0);
        gl.clear(gl::COLOR_BUFFER_BIT);
        gl.finish();

        let window = Window {
            kind: window_kind,
            event_queue: RefCell::new(vec!()),
            mouse_down_button: Cell::new(None),
            mouse_down_point: Cell::new(Point2D::new(0, 0)),

            browser_id: Cell::new(None),

            mouse_pos: Cell::new(Point2D::new(0, 0)),
            key_modifiers: Cell::new(GlutinKeyModifiers::empty()),
            current_url: RefCell::new(None),

            #[cfg(not(target_os = "windows"))]
            pending_key_event_char: Cell::new(None),
            #[cfg(not(target_os = "windows"))]
            pressed_key_map: RefCell::new(vec![]),
            #[cfg(target_os = "windows")]
            last_pressed_key: Cell::new(None),
            gl: gl.clone(),
            animation_state: Cell::new(AnimationState::Idle),
        };

        window.present();

        Rc::new(window)
    }

    pub fn platform_window(&self) -> glutin::WindowID {
        match self.kind {
            WindowKind::Window(ref window) => {
                unsafe { glutin::WindowID::new(window.platform_window()) }
            }
            WindowKind::Headless(..) => {
                unreachable!();
            }
        }
    }

    fn nested_window_resize(_width: u32, _height: u32) {
        unsafe {
            if let Some(listener) = G_NESTED_EVENT_LOOP_LISTENER {
                (*listener).handle_event_from_nested_event_loop(WindowEvent::Resize);
            }
        }
    }

    #[cfg(not(any(target_arch = "arm", target_arch = "aarch64")))]
    fn gl_version() -> GlRequest {
        return GlRequest::Specific(Api::OpenGl, (3, 2));
    }

    #[cfg(any(target_arch = "arm", target_arch = "aarch64"))]
    fn gl_version() -> GlRequest {
        GlRequest::Specific(Api::OpenGlEs, (3, 0))
    }

    #[cfg(not(target_os = "windows"))]
    fn handle_received_character(&self, ch: char) {
        if !ch.is_control() {
            self.pending_key_event_char.set(Some(ch));
        }
    }

    #[cfg(target_os = "windows")]
    fn handle_received_character(&self, ch: char) {
        let modifiers = Window::glutin_mods_to_script_mods(self.key_modifiers.get());
        if let Some(last_pressed_key) = self.last_pressed_key.get() {
            let event = WindowEvent::KeyEvent(Some(ch), last_pressed_key, KeyState::Pressed, modifiers);
            self.event_queue.borrow_mut().push(event);
        } else {
            // Only send the character if we can print it (by ignoring characters like backspace)
            if !ch.is_control() {
                match Window::char_to_script_key(ch) {
                    Some(key) => {
                        let event = WindowEvent::KeyEvent(Some(ch),
                                                          key,
                                                          KeyState::Pressed,
                                                          modifiers);
                        self.event_queue.borrow_mut().push(event);
                    }
                    None => {}
                }
            }
        }
        self.last_pressed_key.set(None);
    }

    fn toggle_keyboard_modifiers(&self, virtual_key_code: VirtualKeyCode) {
        match virtual_key_code {
            VirtualKeyCode::LControl => self.toggle_modifier(GlutinKeyModifiers::LEFT_CONTROL),
            VirtualKeyCode::RControl => self.toggle_modifier(GlutinKeyModifiers::RIGHT_CONTROL),
            VirtualKeyCode::LShift => self.toggle_modifier(GlutinKeyModifiers::LEFT_SHIFT),
            VirtualKeyCode::RShift => self.toggle_modifier(GlutinKeyModifiers::RIGHT_SHIFT),
            VirtualKeyCode::LAlt => self.toggle_modifier(GlutinKeyModifiers::LEFT_ALT),
            VirtualKeyCode::RAlt => self.toggle_modifier(GlutinKeyModifiers::RIGHT_ALT),
            VirtualKeyCode::LWin => self.toggle_modifier(GlutinKeyModifiers::LEFT_SUPER),
            VirtualKeyCode::RWin => self.toggle_modifier(GlutinKeyModifiers::RIGHT_SUPER),
            _ => {}
        }
    }

    #[cfg(not(target_os = "windows"))]
    fn handle_keyboard_input(&self, element_state: ElementState, _scan_code: u8, virtual_key_code: VirtualKeyCode) {
        self.toggle_keyboard_modifiers(virtual_key_code);

        let ch = match element_state {
            ElementState::Pressed => {
                // Retrieve any previously stored ReceivedCharacter value.
                // Store the association between the scan code and the actual
                // character value, if there is one.
                let ch = self.pending_key_event_char
                            .get()
                            .and_then(|ch| filter_nonprintable(ch, virtual_key_code));
                self.pending_key_event_char.set(None);
                if let Some(ch) = ch {
                    self.pressed_key_map.borrow_mut().push((_scan_code, ch));
                }
                ch
            }

            ElementState::Released => {
                // Retrieve the associated character value for this release key,
                // if one was previously stored.
                let idx = self.pressed_key_map
                            .borrow()
                            .iter()
                            .position(|&(code, _)| code == _scan_code);
                idx.map(|idx| self.pressed_key_map.borrow_mut().swap_remove(idx).1)
            }
        };

        if let Ok(key) = Window::glutin_key_to_script_key(virtual_key_code) {
            let state = match element_state {
                ElementState::Pressed => KeyState::Pressed,
                ElementState::Released => KeyState::Released,
            };
            let modifiers = Window::glutin_mods_to_script_mods(self.key_modifiers.get());
            self.event_queue.borrow_mut().push(WindowEvent::KeyEvent(ch, key, state, modifiers));
        }
    }

    #[cfg(target_os = "windows")]
    fn handle_keyboard_input(&self, element_state: ElementState, _scan_code: u8, virtual_key_code: VirtualKeyCode) {
        self.toggle_keyboard_modifiers(virtual_key_code);

        if let Ok(key) = Window::glutin_key_to_script_key(virtual_key_code) {
            let state = match element_state {
                ElementState::Pressed => KeyState::Pressed,
                ElementState::Released => KeyState::Released,
            };
            if element_state == ElementState::Pressed {
                if is_printable(virtual_key_code) {
                    self.last_pressed_key.set(Some(key));
                }
            }
            let modifiers = Window::glutin_mods_to_script_mods(self.key_modifiers.get());
            self.event_queue.borrow_mut().push(WindowEvent::KeyEvent(None, key, state, modifiers));
        }
    }

    fn handle_window_event(&self, event: glutin::Event) -> bool {
        match event {
            Event::ReceivedCharacter(ch) => {
                self.handle_received_character(ch)
            }
            Event::KeyboardInput(element_state, _scan_code, Some(virtual_key_code)) => {
                self.handle_keyboard_input(element_state, _scan_code, virtual_key_code);
            }
            Event::KeyboardInput(_, _, None) => {
                debug!("Keyboard input without virtual key.");
            }
            Event::Resized(..) => {
                self.event_queue.borrow_mut().push(WindowEvent::Resize);
            }
            Event::MouseInput(element_state, mouse_button, pos) => {
                if mouse_button == MouseButton::Left ||
                   mouse_button == MouseButton::Right {
                    match pos {
                        Some((x, y)) => {
                            self.mouse_pos.set(Point2D::new(x, y));
                            self.event_queue.borrow_mut().push(
                                WindowEvent::MouseWindowMoveEventClass(TypedPoint2D::new(x as f32, y as f32)));
                            self.handle_mouse(mouse_button, element_state, x, y);
                        }
                        None => {
                            let mouse_pos = self.mouse_pos.get();
                            self.handle_mouse(mouse_button, element_state, mouse_pos.x, mouse_pos.y);
                        }
                    }
                }
            }
            Event::MouseMoved(x, y) => {
                self.mouse_pos.set(Point2D::new(x, y));
                self.event_queue.borrow_mut().push(
                    WindowEvent::MouseWindowMoveEventClass(TypedPoint2D::new(x as f32, y as f32)));
            }
            Event::MouseWheel(delta, phase, pos) => {
                let (dx, dy) = match delta {
                    MouseScrollDelta::LineDelta(dx, dy) => (dx, dy * LINE_HEIGHT),
                    MouseScrollDelta::PixelDelta(dx, dy) => (dx, dy),
                };
                let scroll_location = ScrollLocation::Delta(TypedVector2D::new(dx, dy));
                if let Some((x, y)) = pos {
                    self.mouse_pos.set(Point2D::new(x, y));
                    self.event_queue.borrow_mut().push(
                        WindowEvent::MouseWindowMoveEventClass(TypedPoint2D::new(x as f32, y as f32)));
                };
                let phase = glutin_phase_to_touch_event_type(phase);
                self.scroll_window(scroll_location, phase);
            },
            Event::Touch(touch) => {
                use script_traits::TouchId;

                let phase = glutin_phase_to_touch_event_type(touch.phase);
                let id = TouchId(touch.id as i32);
                let point = TypedPoint2D::new(touch.location.0 as f32, touch.location.1 as f32);
                self.event_queue.borrow_mut().push(WindowEvent::Touch(phase, id, point));
            }
            Event::TouchpadPressure(pressure, stage) => {
                let m = self.mouse_pos.get();
                let point = TypedPoint2D::new(m.x as f32, m.y as f32);
                let phase = glutin_pressure_stage_to_touchpad_pressure_phase(stage);
                self.event_queue.borrow_mut().push(WindowEvent::TouchpadPressure(point, pressure, phase));
            }
            Event::Refresh => {
                self.event_queue.borrow_mut().push(WindowEvent::Refresh);
            }
            Event::Closed => {
                return true
            }
            _ => {}
        }

        false
    }

    fn toggle_modifier(&self, modifier: GlutinKeyModifiers) {
        let mut modifiers = self.key_modifiers.get();
        modifiers.toggle(modifier);
        self.key_modifiers.set(modifiers);
    }

    /// Helper function to send a scroll event.
    fn scroll_window(&self, mut scroll_location: ScrollLocation, phase: TouchEventType) {
        // Scroll events snap to the major axis of movement, with vertical
        // preferred over horizontal.
        if let ScrollLocation::Delta(ref mut delta) = scroll_location {
            if delta.y.abs() >= delta.x.abs() {
                delta.x = 0.0;
            } else {
                delta.y = 0.0;
            }
        }

        let mouse_pos = self.mouse_pos.get();
        let event = WindowEvent::Scroll(scroll_location,
                                        TypedPoint2D::new(mouse_pos.x as i32, mouse_pos.y as i32),
                                        phase);
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
                MouseWindowEvent::MouseDown(MouseButton::Left, TypedPoint2D::new(x as f32, y as f32))
            }
            ElementState::Released => {
                let mouse_up_event = MouseWindowEvent::MouseUp(MouseButton::Left,
                                                               TypedPoint2D::new(x as f32, y as f32));
                match self.mouse_down_button.get() {
                    None => mouse_up_event,
                    Some(but) if button == but => {
                        let pixel_dist = self.mouse_down_point.get() - Point2D::new(x, y);
                        let pixel_dist = ((pixel_dist.x * pixel_dist.x +
                                           pixel_dist.y * pixel_dist.y) as f64).sqrt();
                        if pixel_dist < max_pixel_dist {
                            self.event_queue.borrow_mut().push(WindowEvent::MouseWindowEventClass(mouse_up_event));
                            MouseWindowEvent::Click(MouseButton::Left, TypedPoint2D::new(x as f32, y as f32))
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

    #[cfg(any(target_os = "macos", target_os = "windows"))]
    fn handle_next_event(&self) -> bool {
        match self.kind {
            WindowKind::Window(ref window) => {
                let event = match window.wait_events().next() {
                    None => {
                        warn!("Window event stream closed.");
                        return true;
                    },
                    Some(event) => event,
                };
                let mut close = self.handle_window_event(event);
                if !close {
                    while let Some(event) = window.poll_events().next() {
                        if self.handle_window_event(event) {
                            close = true;
                            break
                        }
                    }
                }
                close
            }
            WindowKind::Headless(..) => {
                false
            }
        }
    }

    #[cfg(any(target_os = "linux", target_os = "android"))]
    fn handle_next_event(&self) -> bool {
        match self.kind {
            WindowKind::Window(ref window) => {
                let event = match window.wait_events().next() {
                    None => {
                        warn!("Window event stream closed.");
                        return true;
                    },
                    Some(event) => event,
                };
                let mut close = self.handle_window_event(event);
                if !close {
                    while let Some(event) = window.poll_events().next() {
                        if self.handle_window_event(event) {
                            close = true;
                            break
                        }
                    }
                }
                close
            }
            WindowKind::Headless(..) => {
                false
            }
        }
    }

    pub fn wait_events(&self) -> Vec<WindowEvent> {
        use std::mem;

        let mut events = mem::replace(&mut *self.event_queue.borrow_mut(), Vec::new());
        let mut close_event = false;

        let poll = self.animation_state.get() == AnimationState::Animating ||
                   opts::get().output_file.is_some() ||
                   opts::get().exit_after_load ||
                   opts::get().headless;
        // When writing to a file then exiting, use event
        // polling so that we don't block on a GUI event
        // such as mouse click.
        if poll {
            match self.kind {
                WindowKind::Window(ref window) => {
                    while let Some(event) = window.poll_events().next() {
                        close_event = self.handle_window_event(event) || close_event;
                    }
                }
                WindowKind::Headless(..) => {
                    // Sleep the main thread to avoid using 100% CPU
                    // This can be done better, see comments in #18777
                    if events.is_empty() {
                        thread::sleep(time::Duration::from_millis(5));
                    }
                }
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
        G_NESTED_EVENT_LOOP_LISTENER = Some(listener)
    }

    pub unsafe fn remove_nested_event_loop_listener(&self) {
        G_NESTED_EVENT_LOOP_LISTENER = None
    }

    #[cfg(target_os = "windows")]
    fn char_to_script_key(c: char) -> Option<constellation_msg::Key> {
        match c {
            ' ' => Some(Key::Space),
            '"' => Some(Key::Apostrophe),
            '\'' => Some(Key::Apostrophe),
            '<' => Some(Key::Comma),
            ',' => Some(Key::Comma),
            '_' => Some(Key::Minus),
            '-' => Some(Key::Minus),
            '>' => Some(Key::Period),
            '.' => Some(Key::Period),
            '?' => Some(Key::Slash),
            '/' => Some(Key::Slash),
            '~' => Some(Key::GraveAccent),
            '`' => Some(Key::GraveAccent),
            ')' => Some(Key::Num0),
            '0' => Some(Key::Num0),
            '!' => Some(Key::Num1),
            '1' => Some(Key::Num1),
            '@' => Some(Key::Num2),
            '2' => Some(Key::Num2),
            '#' => Some(Key::Num3),
            '3' => Some(Key::Num3),
            '$' => Some(Key::Num4),
            '4' => Some(Key::Num4),
            '%' => Some(Key::Num5),
            '5' => Some(Key::Num5),
            '^' => Some(Key::Num6),
            '6' => Some(Key::Num6),
            '&' => Some(Key::Num7),
            '7' => Some(Key::Num7),
            '*' => Some(Key::Num8),
            '8' => Some(Key::Num8),
            '(' => Some(Key::Num9),
            '9' => Some(Key::Num9),
            ':' => Some(Key::Semicolon),
            ';' => Some(Key::Semicolon),
            '+' => Some(Key::Equal),
            '=' => Some(Key::Equal),
            'A' => Some(Key::A),
            'a' => Some(Key::A),
            'B' => Some(Key::B),
            'b' => Some(Key::B),
            'C' => Some(Key::C),
            'c' => Some(Key::C),
            'D' => Some(Key::D),
            'd' => Some(Key::D),
            'E' => Some(Key::E),
            'e' => Some(Key::E),
            'F' => Some(Key::F),
            'f' => Some(Key::F),
            'G' => Some(Key::G),
            'g' => Some(Key::G),
            'H' => Some(Key::H),
            'h' => Some(Key::H),
            'I' => Some(Key::I),
            'i' => Some(Key::I),
            'J' => Some(Key::J),
            'j' => Some(Key::J),
            'K' => Some(Key::K),
            'k' => Some(Key::K),
            'L' => Some(Key::L),
            'l' => Some(Key::L),
            'M' => Some(Key::M),
            'm' => Some(Key::M),
            'N' => Some(Key::N),
            'n' => Some(Key::N),
            'O' => Some(Key::O),
            'o' => Some(Key::O),
            'P' => Some(Key::P),
            'p' => Some(Key::P),
            'Q' => Some(Key::Q),
            'q' => Some(Key::Q),
            'R' => Some(Key::R),
            'r' => Some(Key::R),
            'S' => Some(Key::S),
            's' => Some(Key::S),
            'T' => Some(Key::T),
            't' => Some(Key::T),
            'U' => Some(Key::U),
            'u' => Some(Key::U),
            'V' => Some(Key::V),
            'v' => Some(Key::V),
            'W' => Some(Key::W),
            'w' => Some(Key::W),
            'X' => Some(Key::X),
            'x' => Some(Key::X),
            'Y' => Some(Key::Y),
            'y' => Some(Key::Y),
            'Z' => Some(Key::Z),
            'z' => Some(Key::Z),
            '{' => Some(Key::LeftBracket),
            '[' => Some(Key::LeftBracket),
            '|' => Some(Key::Backslash),
            '\\' => Some(Key::Backslash),
            '}' => Some(Key::RightBracket),
            ']' => Some(Key::RightBracket),
            _ => None
        }
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

            VirtualKeyCode::LShift => Ok(Key::LeftShift),
            VirtualKeyCode::LControl => Ok(Key::LeftControl),
            VirtualKeyCode::LAlt => Ok(Key::LeftAlt),
            VirtualKeyCode::LWin => Ok(Key::LeftSuper),
            VirtualKeyCode::RShift => Ok(Key::RightShift),
            VirtualKeyCode::RControl => Ok(Key::RightControl),
            VirtualKeyCode::RAlt => Ok(Key::RightAlt),
            VirtualKeyCode::RWin => Ok(Key::RightSuper),

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

            VirtualKeyCode::F1 => Ok(Key::F1),
            VirtualKeyCode::F2 => Ok(Key::F2),
            VirtualKeyCode::F3 => Ok(Key::F3),
            VirtualKeyCode::F4 => Ok(Key::F4),
            VirtualKeyCode::F5 => Ok(Key::F5),
            VirtualKeyCode::F6 => Ok(Key::F6),
            VirtualKeyCode::F7 => Ok(Key::F7),
            VirtualKeyCode::F8 => Ok(Key::F8),
            VirtualKeyCode::F9 => Ok(Key::F9),
            VirtualKeyCode::F10 => Ok(Key::F10),
            VirtualKeyCode::F11 => Ok(Key::F11),
            VirtualKeyCode::F12 => Ok(Key::F12),

            VirtualKeyCode::NavigateBackward => Ok(Key::NavigateBackward),
            VirtualKeyCode::NavigateForward => Ok(Key::NavigateForward),
            _ => Err(()),
        }
    }

    fn glutin_mods_to_script_mods(modifiers: GlutinKeyModifiers) -> constellation_msg::KeyModifiers {
        let mut result = constellation_msg::KeyModifiers::empty();
        if modifiers.intersects(GlutinKeyModifiers::LEFT_SHIFT | GlutinKeyModifiers::RIGHT_SHIFT) {
            result.insert(KeyModifiers::SHIFT);
        }
        if modifiers.intersects(GlutinKeyModifiers::LEFT_CONTROL | GlutinKeyModifiers::RIGHT_CONTROL) {
            result.insert(KeyModifiers::CONTROL);
        }
        if modifiers.intersects(GlutinKeyModifiers::LEFT_ALT | GlutinKeyModifiers::RIGHT_ALT) {
            result.insert(KeyModifiers::ALT);
        }
        if modifiers.intersects(GlutinKeyModifiers::LEFT_SUPER | GlutinKeyModifiers::RIGHT_SUPER) {
            result.insert(KeyModifiers::SUPER);
        }
        result
    }

    #[cfg(not(target_os = "win"))]
    fn platform_handle_key(&self, key: Key, mods: constellation_msg::KeyModifiers, browser_id: BrowserId) {
        match (mods, key) {
            (CMD_OR_CONTROL, Key::LeftBracket) => {
                let event = WindowEvent::Navigation(browser_id, TraversalDirection::Back(1));
                self.event_queue.borrow_mut().push(event);
            }
            (CMD_OR_CONTROL, Key::RightBracket) => {
                let event = WindowEvent::Navigation(browser_id, TraversalDirection::Forward(1));
                self.event_queue.borrow_mut().push(event);
            }
            _ => {}
        }
    }

    #[cfg(target_os = "win")]
    fn platform_handle_key(&self, key: Key, mods: constellation_msg::KeyModifiers, browser_id: BrowserId) {
    }
}

fn create_window_proxy(window: &Window) -> Option<glutin::WindowProxy> {
    match window.kind {
        WindowKind::Window(ref window) => {
            Some(window.create_window_proxy())
        }
        WindowKind::Headless(..) => {
            None
        }
    }
}

impl WindowMethods for Window {
    fn gl(&self) -> Rc<gl::Gl> {
        self.gl.clone()
    }

    fn framebuffer_size(&self) -> DeviceUintSize {
        match self.kind {
            WindowKind::Window(ref window) => {
                let scale_factor = window.hidpi_factor() as u32;
                // TODO(ajeffrey): can this fail?
                let (width, height) = window.get_inner_size().expect("Failed to get window inner size.");
                DeviceUintSize::new(width, height) * scale_factor
            }
            WindowKind::Headless(ref context) => {
                DeviceUintSize::new(context.width, context.height)
            }
        }
    }

    fn window_rect(&self) -> DeviceUintRect {
        let size = self.framebuffer_size();
        let origin = TypedPoint2D::zero();
        DeviceUintRect::new(origin, size)
    }

    fn size(&self) -> TypedSize2D<f32, DeviceIndependentPixel> {
        match self.kind {
            WindowKind::Window(ref window) => {
                // TODO(ajeffrey): can this fail?
                let (width, height) = window.get_inner_size().expect("Failed to get window inner size.");
                TypedSize2D::new(width as f32, height as f32)
            }
            WindowKind::Headless(ref context) => {
                TypedSize2D::new(context.width as f32, context.height as f32)
            }
        }
    }

    fn client_window(&self, _: BrowserId) -> (Size2D<u32>, Point2D<i32>) {
        match self.kind {
            WindowKind::Window(ref window) => {
                // TODO(ajeffrey): can this fail?
                let (width, height) = window.get_outer_size().expect("Failed to get window outer size.");
                let size = Size2D::new(width, height);
                // TODO(ajeffrey): can this fail?
                let (x, y) = window.get_position().expect("Failed to get window position.");
                let origin = Point2D::new(x as i32, y as i32);
                (size, origin)
            }
            WindowKind::Headless(ref context) => {
                let size = TypedSize2D::new(context.width, context.height);
                (size, Point2D::zero())
            }
        }

    }

    fn screen_size(&self, _: BrowserId) -> Size2D<u32> {
        match self.kind {
            WindowKind::Window(_) => {
                let (width, height) = glutin::get_primary_monitor().get_dimensions();
                Size2D::new(width, height)
            }
            WindowKind::Headless(ref context) => {
                Size2D::new(context.width, context.height)
            }
        }
    }

    fn screen_avail_size(&self, _: BrowserId) -> Size2D<u32> {
        // FIXME: Glutin doesn't have API for available size. Fallback to screen size
        match self.kind {
            WindowKind::Window(_) => {
                let (width, height) = glutin::get_primary_monitor().get_dimensions();
                Size2D::new(width, height)
            }
            WindowKind::Headless(ref context) => {
                Size2D::new(context.width, context.height)
            }
        }
    }

    fn set_animation_state(&self, state: AnimationState) {
        self.animation_state.set(state);
    }

    fn set_inner_size(&self, _: BrowserId, size: Size2D<u32>) {
        match self.kind {
            WindowKind::Window(ref window) => {
                window.set_inner_size(size.width as u32, size.height as u32)
            }
            WindowKind::Headless(..) => {}
        }
    }

    fn set_position(&self, _: BrowserId, point: Point2D<i32>) {
        match self.kind {
            WindowKind::Window(ref window) => {
                window.set_position(point.x, point.y)
            }
            WindowKind::Headless(..) => {}
        }
    }

    fn set_fullscreen_state(&self, _: BrowserId, _state: bool) {
        match self.kind {
            WindowKind::Window(..) => {
                warn!("Fullscreen is not implemented!")
            },
            WindowKind::Headless(..) => {}
        }
    }

    fn present(&self) {
        match self.kind {
            WindowKind::Window(ref window) => {
                if let Err(err) = window.swap_buffers() {
                    warn!("Failed to swap window buffers ({}).", err);
                }
            }
            WindowKind::Headless(..) => {}
        }
    }

    fn create_event_loop_waker(&self) -> Box<EventLoopWaker> {
        struct GlutinEventLoopWaker {
            window_proxy: Option<glutin::WindowProxy>,
        }
        impl EventLoopWaker for GlutinEventLoopWaker {
            fn wake(&self) {
                // kick the OS event loop awake.
                if let Some(ref window_proxy) = self.window_proxy {
                    window_proxy.wakeup_event_loop()
                }
            }
            fn clone(&self) -> Box<EventLoopWaker + Send> {
                Box::new(GlutinEventLoopWaker {
                    window_proxy: self.window_proxy.clone(),
                })
            }
        }
        let window_proxy = create_window_proxy(self);
        Box::new(GlutinEventLoopWaker {
            window_proxy: window_proxy,
        })
    }

    #[cfg(not(target_os = "windows"))]
    fn hidpi_factor(&self) -> TypedScale<f32, DeviceIndependentPixel, DevicePixel> {
        match self.kind {
            WindowKind::Window(ref window) => {
                TypedScale::new(window.hidpi_factor())
            }
            WindowKind::Headless(..) => {
                TypedScale::new(1.0)
            }
        }
    }

    #[cfg(target_os = "windows")]
    fn hidpi_factor(&self) -> TypedScale<f32, DeviceIndependentPixel, DevicePixel> {
        let hdc = unsafe { user32::GetDC(::std::ptr::null_mut()) };
        let ppi = unsafe { gdi32::GetDeviceCaps(hdc, winapi::wingdi::LOGPIXELSY) };
        TypedScale::new(ppi as f32 / 96.0)
    }

    fn set_page_title(&self, _: BrowserId, title: Option<String>) {
        match self.kind {
            WindowKind::Window(ref window) => {
                let fallback_title: String = if let Some(ref current_url) = *self.current_url.borrow() {
                    current_url.to_string()
                } else {
                    String::from("Untitled")
                };

                let title = match title {
                    Some(ref title) if title.len() > 0 => &**title,
                    _ => &fallback_title,
                };
                let title = format!("{} - Servo", title);
                window.set_title(&title);
            }
            WindowKind::Headless(..) => {}
        }
    }

    fn status(&self, _: BrowserId, _: Option<String>) {
    }

    fn load_start(&self, _: BrowserId) {
    }

    fn load_end(&self, _: BrowserId) {
        if opts::get().no_native_titlebar {
            match self.kind {
                WindowKind::Window(ref window) => {
                    window.show();
                }
                WindowKind::Headless(..) => {}
            }
        }
    }

    fn history_changed(&self, _: BrowserId, history: Vec<LoadData>, current: usize) {
        *self.current_url.borrow_mut() = Some(history[current].url.clone());
    }

    fn load_error(&self, _: BrowserId, _: NetError, _: String) {
    }

    fn head_parsed(&self, _: BrowserId) {
    }

    /// Has no effect on Android.
    fn set_cursor(&self, cursor: CursorKind) {
        match self.kind {
            WindowKind::Window(ref window) => {
                use glutin::MouseCursor;

                let glutin_cursor = match cursor {
                    CursorKind::Auto => MouseCursor::Default,
                    CursorKind::None => MouseCursor::NoneCursor,
                    CursorKind::Default => MouseCursor::Default,
                    CursorKind::Pointer => MouseCursor::Hand,
                    CursorKind::ContextMenu => MouseCursor::ContextMenu,
                    CursorKind::Help => MouseCursor::Help,
                    CursorKind::Progress => MouseCursor::Progress,
                    CursorKind::Wait => MouseCursor::Wait,
                    CursorKind::Cell => MouseCursor::Cell,
                    CursorKind::Crosshair => MouseCursor::Crosshair,
                    CursorKind::Text => MouseCursor::Text,
                    CursorKind::VerticalText => MouseCursor::VerticalText,
                    CursorKind::Alias => MouseCursor::Alias,
                    CursorKind::Copy => MouseCursor::Copy,
                    CursorKind::Move => MouseCursor::Move,
                    CursorKind::NoDrop => MouseCursor::NoDrop,
                    CursorKind::NotAllowed => MouseCursor::NotAllowed,
                    CursorKind::Grab => MouseCursor::Grab,
                    CursorKind::Grabbing => MouseCursor::Grabbing,
                    CursorKind::EResize => MouseCursor::EResize,
                    CursorKind::NResize => MouseCursor::NResize,
                    CursorKind::NeResize => MouseCursor::NeResize,
                    CursorKind::NwResize => MouseCursor::NwResize,
                    CursorKind::SResize => MouseCursor::SResize,
                    CursorKind::SeResize => MouseCursor::SeResize,
                    CursorKind::SwResize => MouseCursor::SwResize,
                    CursorKind::WResize => MouseCursor::WResize,
                    CursorKind::EwResize => MouseCursor::EwResize,
                    CursorKind::NsResize => MouseCursor::NsResize,
                    CursorKind::NeswResize => MouseCursor::NeswResize,
                    CursorKind::NwseResize => MouseCursor::NwseResize,
                    CursorKind::ColResize => MouseCursor::ColResize,
                    CursorKind::RowResize => MouseCursor::RowResize,
                    CursorKind::AllScroll => MouseCursor::AllScroll,
                    CursorKind::ZoomIn => MouseCursor::ZoomIn,
                    CursorKind::ZoomOut => MouseCursor::ZoomOut,
                };
                window.set_cursor(glutin_cursor);
            }
            WindowKind::Headless(..) => {}
        }
    }

    fn set_favicon(&self, _: BrowserId, _: ServoUrl) {
    }

    fn prepare_for_composite(&self, _width: usize, _height: usize) -> bool {
        true
    }

    /// Helper function to handle keyboard events.
    fn handle_key(&self, _: Option<BrowserId>, ch: Option<char>, key: Key, mods: constellation_msg::KeyModifiers) {
        let browser_id = match self.browser_id.get() {
            Some(id) => id,
            None => { unreachable!("Can't get keys without a browser"); }
        };
        match (mods, ch, key) {
            (_, Some('+'), _) => {
                if mods & !KeyModifiers::SHIFT == CMD_OR_CONTROL {
                    self.event_queue.borrow_mut().push(WindowEvent::Zoom(1.1));
                } else if mods & !KeyModifiers::SHIFT == CMD_OR_CONTROL | KeyModifiers::ALT {
                    self.event_queue.borrow_mut().push(WindowEvent::PinchZoom(1.1));
                }
            }
            (CMD_OR_CONTROL, Some('-'), _) => {
                self.event_queue.borrow_mut().push(WindowEvent::Zoom(1.0 / 1.1));
            }
            (_, Some('-'), _) if mods == CMD_OR_CONTROL | KeyModifiers::ALT => {
                self.event_queue.borrow_mut().push(WindowEvent::PinchZoom(1.0 / 1.1));
            }
            (CMD_OR_CONTROL, Some('0'), _) => {
                self.event_queue.borrow_mut().push(WindowEvent::ResetZoom);
            }

            (KeyModifiers::NONE, None, Key::NavigateForward) => {
                let event = WindowEvent::Navigation(browser_id, TraversalDirection::Forward(1));
                self.event_queue.borrow_mut().push(event);
            }
            (KeyModifiers::NONE, None, Key::NavigateBackward) => {
                let event = WindowEvent::Navigation(browser_id, TraversalDirection::Back(1));
                self.event_queue.borrow_mut().push(event);
            }

            (KeyModifiers::NONE, None, Key::Escape) => {
                if let Some(true) = PREFS.get("shell.builtin-key-shortcuts.enabled").as_boolean() {
                    self.event_queue.borrow_mut().push(WindowEvent::Quit);
                }
            }

            (CMD_OR_ALT, None, Key::Right) => {
                let event = WindowEvent::Navigation(browser_id, TraversalDirection::Forward(1));
                self.event_queue.borrow_mut().push(event);
            }
            (CMD_OR_ALT, None, Key::Left) => {
                let event = WindowEvent::Navigation(browser_id, TraversalDirection::Back(1));
                self.event_queue.borrow_mut().push(event);
            }

            (KeyModifiers::NONE, None, Key::PageDown) => {
               let scroll_location = ScrollLocation::Delta(TypedVector2D::new(0.0,
                                   -self.framebuffer_size()
                                        .to_f32()
                                        .to_untyped()
                                        .height + 2.0 * LINE_HEIGHT));
                self.scroll_window(scroll_location,
                                   TouchEventType::Move);
            }
            (KeyModifiers::NONE, None, Key::PageUp) => {
                let scroll_location = ScrollLocation::Delta(TypedVector2D::new(0.0,
                                   self.framebuffer_size()
                                       .to_f32()
                                       .to_untyped()
                                       .height - 2.0 * LINE_HEIGHT));
                self.scroll_window(scroll_location,
                                   TouchEventType::Move);
            }

            (KeyModifiers::NONE, None, Key::Home) => {
                self.scroll_window(ScrollLocation::Start, TouchEventType::Move);
            }

            (KeyModifiers::NONE, None, Key::End) => {
                self.scroll_window(ScrollLocation::End, TouchEventType::Move);
            }

            (KeyModifiers::NONE, None, Key::Up) => {
                self.scroll_window(ScrollLocation::Delta(TypedVector2D::new(0.0, 3.0 * LINE_HEIGHT)),
                                   TouchEventType::Move);
            }
            (KeyModifiers::NONE, None, Key::Down) => {
                self.scroll_window(ScrollLocation::Delta(TypedVector2D::new(0.0, -3.0 * LINE_HEIGHT)),
                                   TouchEventType::Move);
            }
            (KeyModifiers::NONE, None, Key::Left) => {
                self.scroll_window(ScrollLocation::Delta(TypedVector2D::new(LINE_HEIGHT, 0.0)), TouchEventType::Move);
            }
            (KeyModifiers::NONE, None, Key::Right) => {
                self.scroll_window(ScrollLocation::Delta(TypedVector2D::new(-LINE_HEIGHT, 0.0)), TouchEventType::Move);
            }
            (CMD_OR_CONTROL, Some('r'), _) => {
                if let Some(true) = PREFS.get("shell.builtin-key-shortcuts.enabled").as_boolean() {
                    self.event_queue.borrow_mut().push(WindowEvent::Reload(browser_id));
                }
            }
            (CMD_OR_CONTROL, Some('q'), _) => {
                if let Some(true) = PREFS.get("shell.builtin-key-shortcuts.enabled").as_boolean() {
                    self.event_queue.borrow_mut().push(WindowEvent::Quit);
                }
            }
            (KeyModifiers::CONTROL, None, Key::F10) => {
                let event = WindowEvent::ToggleWebRenderDebug(WebRenderDebugOption::RenderTargetDebug);
                self.event_queue.borrow_mut().push(event);
            }
            (KeyModifiers::CONTROL, None, Key::F11) => {
                let event = WindowEvent::ToggleWebRenderDebug(WebRenderDebugOption::TextureCacheDebug);
                self.event_queue.borrow_mut().push(event);
            }
            (KeyModifiers::CONTROL, None, Key::F12) => {
                let event = WindowEvent::ToggleWebRenderDebug(WebRenderDebugOption::Profiler);
                self.event_queue.borrow_mut().push(event);
            }

            _ => {
                self.platform_handle_key(key, mods, browser_id);
            }
        }
    }

    fn allow_navigation(&self, _: BrowserId, _: ServoUrl, response_chan: IpcSender<bool>) {
        if let Err(e) = response_chan.send(true) {
            warn!("Failed to send allow_navigation() response: {}", e);
        };
    }

    fn supports_clipboard(&self) -> bool {
        true
    }
}

fn glutin_phase_to_touch_event_type(phase: TouchPhase) -> TouchEventType {
    match phase {
        TouchPhase::Started => TouchEventType::Down,
        TouchPhase::Moved => TouchEventType::Move,
        TouchPhase::Ended => TouchEventType::Up,
        TouchPhase::Cancelled => TouchEventType::Cancel,
    }
}

fn glutin_pressure_stage_to_touchpad_pressure_phase(stage: i64) -> TouchpadPressurePhase {
    if stage < 1 {
        TouchpadPressurePhase::BeforeClick
    } else if stage < 2 {
        TouchpadPressurePhase::AfterFirstClick
    } else {
        TouchpadPressurePhase::AfterSecondClick
    }
}

fn is_printable(key_code: VirtualKeyCode) -> bool {
    use glutin::VirtualKeyCode::*;
    match key_code {
        Escape |
        F1 |
        F2 |
        F3 |
        F4 |
        F5 |
        F6 |
        F7 |
        F8 |
        F9 |
        F10 |
        F11 |
        F12 |
        F13 |
        F14 |
        F15 |
        Snapshot |
        Scroll |
        Pause |
        Insert |
        Home |
        Delete |
        End |
        PageDown |
        PageUp |
        Left |
        Up |
        Right |
        Down |
        Back |
        LAlt |
        LControl |
        LMenu |
        LShift |
        LWin |
        Mail |
        MediaSelect |
        MediaStop |
        Mute |
        MyComputer |
        NavigateForward |
        NavigateBackward |
        NextTrack |
        NoConvert |
        PlayPause |
        Power |
        PrevTrack |
        RAlt |
        RControl |
        RMenu |
        RShift |
        RWin |
        Sleep |
        Stop |
        VolumeDown |
        VolumeUp |
        Wake |
        WebBack |
        WebFavorites |
        WebForward |
        WebHome |
        WebRefresh |
        WebSearch |
        WebStop => false,
        _ => true,
    }
}

#[cfg(not(target_os = "windows"))]
fn filter_nonprintable(ch: char, key_code: VirtualKeyCode) -> Option<char> {
    if is_printable(key_code) {
        Some(ch)
    } else {
        None
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

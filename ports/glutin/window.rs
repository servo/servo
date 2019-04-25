/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! A windowing implementation using winit.

use crate::keyutils::keyboard_event_from_winit;
use euclid::{TypedPoint2D, TypedVector2D, TypedScale, TypedSize2D};
use gleam::gl;
use glutin::{Api, ContextBuilder, GlContext, GlRequest, GlWindow};
#[cfg(any(target_os = "linux", target_os = "windows"))]
use image;
use keyboard_types::{Key, KeyboardEvent, KeyState};
use rust_webvr::GlWindowVRService;
use servo::compositing::windowing::{AnimationState, MouseWindowEvent, WindowEvent};
use servo::compositing::windowing::{EmbedderCoordinates, WindowMethods};
use servo::embedder_traits::{Cursor, EventLoopWaker};
use servo::script_traits::TouchEventType;
use servo::servo_config::{opts, pref};
use servo::servo_geometry::DeviceIndependentPixel;
use servo::style_traits::DevicePixel;
use servo::webrender_api::{DeviceIntPoint, DeviceIntRect, DeviceIntSize, FramebufferIntSize, ScrollLocation};
use servo::webvr::VRServiceManager;
use servo::webvr_traits::WebVRMainThreadHeartbeat;
use std::cell::{Cell, RefCell};
#[cfg(any(target_os = "linux", target_os = "macos"))]
use std::ffi::CString;
use std::mem;
use std::os::raw::c_void;
use std::ptr;
use std::rc::Rc;
use std::sync::Arc;
use std::thread;
use std::time;
#[cfg(target_os = "windows")]
use winapi;
use glutin::{ElementState, Event, MouseButton, MouseScrollDelta, TouchPhase, KeyboardInput};
#[cfg(any(target_os = "linux", target_os = "windows"))]
use glutin::Icon;
use glutin::dpi::{LogicalPosition, LogicalSize, PhysicalSize};
#[cfg(target_os = "macos")]
use glutin::os::macos::{ActivationPolicy, WindowBuilderExt};

// This should vary by zoom level and maybe actual text size (focused or under cursor)
pub const LINE_HEIGHT: f32 = 38.0;

const MULTISAMPLES: u16 = 16;

#[cfg(target_os = "macos")]
fn builder_with_platform_options(mut builder: glutin::WindowBuilder) -> glutin::WindowBuilder {
    if opts::get().headless || opts::get().output_file.is_some() {
        // Prevent the window from showing in Dock.app, stealing focus,
        // or appearing at all when running in headless mode or generating an
        // output file.
        builder = builder.with_activation_policy(ActivationPolicy::Prohibited)
    }
    builder
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

        let context =
            unsafe { osmesa_sys::OSMesaCreateContextAttribs(attribs.as_ptr(), ptr::null_mut()) };

        assert!(!context.is_null());

        let mut buffer = vec![0; (width * height) as usize];

        unsafe {
            let ret = osmesa_sys::OSMesaMakeCurrent(
                context,
                buffer.as_mut_ptr() as *mut _,
                gl::UNSIGNED_BYTE,
                width as i32,
                height as i32,
            );
            assert_ne!(ret, 0);
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
        unsafe { mem::transmute(osmesa_sys::OSMesaGetProcAddress(c_str.as_ptr())) }
    }

    #[cfg(not(any(target_os = "linux", target_os = "macos")))]
    fn get_proc_address(_: &str) -> *const c_void {
        ptr::null() as *const _
    }
}

enum WindowKind {
    Window(GlWindow, RefCell<glutin::EventsLoop>),
    Headless(HeadlessContext),
}

/// The type of a window.
pub struct Window {
    kind: WindowKind,
    screen_size: TypedSize2D<u32, DeviceIndependentPixel>,
    inner_size: Cell<TypedSize2D<u32, DeviceIndependentPixel>>,
    mouse_down_button: Cell<Option<glutin::MouseButton>>,
    mouse_down_point: Cell<TypedPoint2D<i32, DevicePixel>>,
    event_queue: RefCell<Vec<WindowEvent>>,
    mouse_pos: Cell<TypedPoint2D<i32, DevicePixel>>,
    last_pressed: Cell<Option<KeyboardEvent>>,
    animation_state: Cell<AnimationState>,
    fullscreen: Cell<bool>,
    gl: Rc<dyn gl::Gl>,
    suspended: Cell<bool>,
}

#[cfg(not(target_os = "windows"))]
fn window_creation_scale_factor() -> TypedScale<f32, DeviceIndependentPixel, DevicePixel> {
    TypedScale::new(1.0)
}

#[cfg(target_os = "windows")]
fn window_creation_scale_factor() -> TypedScale<f32, DeviceIndependentPixel, DevicePixel> {
    let hdc = unsafe { winapi::um::winuser::GetDC(::std::ptr::null_mut()) };
    let ppi = unsafe { winapi::um::wingdi::GetDeviceCaps(hdc, winapi::um::wingdi::LOGPIXELSY) };
    TypedScale::new(ppi as f32 / 96.0)
}

impl Window {
    pub fn new(
        is_foreground: bool,
        window_size: TypedSize2D<u32, DeviceIndependentPixel>,
    ) -> Rc<Window> {
        let win_size: DeviceIntSize =
            (window_size.to_f32() * window_creation_scale_factor()).to_i32();
        let width = win_size.to_untyped().width;
        let height = win_size.to_untyped().height;

        // If there's no chrome, start off with the window invisible. It will be set to visible in
        // `load_end()`. This avoids an ugly flash of unstyled content (especially important since
        // unstyled content is white and chrome often has a transparent background). See issue
        // #9996.
        let visible = is_foreground && !opts::get().no_native_titlebar;

        let screen_size;
        let inner_size;
        let window_kind = if opts::get().headless {
            screen_size = TypedSize2D::new(width as u32, height as u32);
            inner_size = TypedSize2D::new(width as u32, height as u32);
            WindowKind::Headless(HeadlessContext::new(width as u32, height as u32))
        } else {
            let events_loop = glutin::EventsLoop::new();
            let mut window_builder = glutin::WindowBuilder::new()
                .with_title("Servo".to_string())
                .with_decorations(!opts::get().no_native_titlebar)
                .with_transparency(opts::get().no_native_titlebar)
                .with_dimensions(LogicalSize::new(width as f64, height as f64))
                .with_visibility(visible)
                .with_multitouch();

            window_builder = builder_with_platform_options(window_builder);

            let mut context_builder = ContextBuilder::new()
                .with_gl(Window::gl_version())
                .with_vsync(opts::get().enable_vsync);

            if opts::get().use_msaa {
                context_builder = context_builder.with_multisampling(MULTISAMPLES)
            }

            let glutin_window = GlWindow::new(window_builder, context_builder, &events_loop)
                .expect("Failed to create window.");

            #[cfg(any(target_os = "linux", target_os = "windows"))]
            {
                let icon_bytes = include_bytes!("../../../resources/servo64.png");
                glutin_window.set_window_icon(Some(load_icon(icon_bytes)));
            }

            unsafe {
                glutin_window
                    .context()
                    .make_current()
                    .expect("Couldn't make window current");
            }

            let PhysicalSize {
                width: screen_width,
                height: screen_height,
            } = events_loop.get_primary_monitor().get_dimensions();
            screen_size = TypedSize2D::new(screen_width as u32, screen_height as u32);
            // TODO(ajeffrey): can this fail?
            let LogicalSize { width, height } = glutin_window
                .get_inner_size()
                .expect("Failed to get window inner size.");
            inner_size = TypedSize2D::new(width as u32, height as u32);

            glutin_window.show();

            WindowKind::Window(glutin_window, RefCell::new(events_loop))
        };

        let gl = match window_kind {
            WindowKind::Window(ref window, ..) => match gl::GlType::default() {
                gl::GlType::Gl => unsafe {
                    gl::GlFns::load_with(|s| window.get_proc_address(s) as *const _)
                },
                gl::GlType::Gles => unsafe {
                    gl::GlesFns::load_with(|s| window.get_proc_address(s) as *const _)
                },
            },
            WindowKind::Headless(..) => unsafe {
                gl::GlFns::load_with(|s| HeadlessContext::get_proc_address(s))
            },
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
            event_queue: RefCell::new(vec![]),
            mouse_down_button: Cell::new(None),
            mouse_down_point: Cell::new(TypedPoint2D::new(0, 0)),
            mouse_pos: Cell::new(TypedPoint2D::new(0, 0)),
            last_pressed: Cell::new(None),
            gl: gl.clone(),
            animation_state: Cell::new(AnimationState::Idle),
            fullscreen: Cell::new(false),
            inner_size: Cell::new(inner_size),
            screen_size,
            suspended: Cell::new(false),
        };

        window.present();

        Rc::new(window)
    }

    pub fn get_events(&self) -> Vec<WindowEvent> {
        mem::replace(&mut *self.event_queue.borrow_mut(), Vec::new())
    }

    pub fn page_height(&self) -> f32 {
        let dpr = self.servo_hidpi_factor();
        match self.kind {
            WindowKind::Window(ref window, _) => {
                let size = window
                    .get_inner_size()
                    .expect("Failed to get window inner size.");
                size.height as f32 * dpr.get()
            },
            WindowKind::Headless(ref context) => context.height as f32 * dpr.get(),
        }
    }

    pub fn set_title(&self, title: &str) {
        if let WindowKind::Window(ref window, _) = self.kind {
            window.set_title(title);
        }
    }

    pub fn set_inner_size(&self, size: DeviceIntSize) {
        if let WindowKind::Window(ref window, _) = self.kind {
            let size = size.to_f32() / self.device_hidpi_factor();
            window.set_inner_size(LogicalSize::new(size.width.into(), size.height.into()))
        }
    }

    pub fn set_position(&self, point: DeviceIntPoint) {
        if let WindowKind::Window(ref window, _) = self.kind {
            let point = point.to_f32() / self.device_hidpi_factor();
            window.set_position(LogicalPosition::new(point.x.into(), point.y.into()))
        }
    }

    pub fn set_fullscreen(&self, state: bool) {
        match self.kind {
            WindowKind::Window(ref window, ..) => {
                if self.fullscreen.get() != state {
                    window.set_fullscreen(Some(window.get_primary_monitor()));
                }
            },
            WindowKind::Headless(..) => {},
        }
        self.fullscreen.set(state);
    }

    pub fn get_fullscreen(&self) -> bool {
        return self.fullscreen.get();
    }

    fn is_animating(&self) -> bool {
        self.animation_state.get() == AnimationState::Animating && !self.suspended.get()
    }

    pub fn run<T>(&self, mut servo_callback: T)
    where
        T: FnMut() -> bool,
    {
        match self.kind {
            WindowKind::Window(_, ref events_loop) => {
                let mut stop = false;
                loop {
                    if self.is_animating() {
                        // We block on compositing (servo_callback ends up calling swap_buffers)
                        events_loop.borrow_mut().poll_events(|e| {
                            self.winit_event_to_servo_event(e);
                        });
                        stop = servo_callback();
                    } else {
                        // We block on winit's event loop (window events)
                        events_loop.borrow_mut().run_forever(|e| {
                            self.winit_event_to_servo_event(e);
                            if !self.event_queue.borrow().is_empty() {
                                if !self.suspended.get() {
                                    stop = servo_callback();
                                }
                            }
                            if stop || self.is_animating() {
                                glutin::ControlFlow::Break
                            } else {
                                glutin::ControlFlow::Continue
                            }
                        });
                    }
                    if stop {
                        break;
                    }
                }
            },
            WindowKind::Headless(..) => {
                loop {
                    // Sleep the main thread to avoid using 100% CPU
                    // This can be done better, see comments in #18777
                    if self.event_queue.borrow().is_empty() {
                        thread::sleep(time::Duration::from_millis(5));
                    }
                    let stop = servo_callback();
                    if stop {
                        break;
                    }
                }
            },
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

    fn handle_received_character(&self, mut ch: char) {
        info!("winit received character: {:?}", ch);
        if ch.is_control() {
            if ch as u8 >= 32 {
                return;
            }
            // shift ASCII control characters to lowercase
            ch = (ch as u8 + 96) as char;
        }
        let mut event = if let Some(event) = self.last_pressed.replace(None) {
            event
        } else if ch.is_ascii() {
            // Some keys like Backspace emit a control character in winit
            // but they are already dealt with in handle_keyboard_input
            // so just ignore the character.
            return
        } else {
            // For combined characters like the letter e with an acute accent
            // no keyboard event is emitted. A dummy event is created in this case.
            KeyboardEvent::default()
        };
        event.key = Key::Character(ch.to_string());
        self.event_queue
            .borrow_mut()
            .push(WindowEvent::Keyboard(event));
    }

    fn handle_keyboard_input(
        &self,
        input: KeyboardInput,
    ) {
        let event = keyboard_event_from_winit(input);
        if event.state == KeyState::Down && event.key == Key::Unidentified {
            // If pressed and probably printable, we expect a ReceivedCharacter event.
            self.last_pressed.set(Some(event));
        } else if event.key != Key::Unidentified {
            self.last_pressed.set(None);
            self.event_queue
                .borrow_mut()
                .push(WindowEvent::Keyboard(event));
        }
    }

    fn winit_event_to_servo_event(&self, event: glutin::Event) {
        if let WindowKind::Window(ref window, _) = self.kind {
            if let Event::WindowEvent { window_id, .. } = event {
                if window.id() != window_id {
                     return;
                }
            }
        }
        match event {
            Event::WindowEvent {
                event: glutin::WindowEvent::ReceivedCharacter(ch),
                ..
            } => self.handle_received_character(ch),
            Event::WindowEvent {
                event:
                    glutin::WindowEvent::KeyboardInput {
                        input,
                        ..
                    },
                ..
            } => self.handle_keyboard_input(input),
            Event::WindowEvent {
                event: glutin::WindowEvent::MouseInput { state, button, .. },
                ..
            } => {
                if button == MouseButton::Left || button == MouseButton::Right {
                    self.handle_mouse(button, state, self.mouse_pos.get());
                }
            },
            Event::WindowEvent {
                event: glutin::WindowEvent::CursorMoved { position, .. },
                ..
            } => {
                let pos = position.to_physical(self.device_hidpi_factor().get() as f64);
                let (x, y): (i32, i32) = pos.into();
                self.mouse_pos.set(TypedPoint2D::new(x, y));
                self.event_queue
                    .borrow_mut()
                    .push(WindowEvent::MouseWindowMoveEventClass(TypedPoint2D::new(
                        x as f32, y as f32,
                    )));
            },
            Event::WindowEvent {
                event: glutin::WindowEvent::MouseWheel { delta, phase, .. },
                ..
            } => {
                let (mut dx, mut dy) = match delta {
                    MouseScrollDelta::LineDelta(dx, dy) => (dx, dy * LINE_HEIGHT),
                    MouseScrollDelta::PixelDelta(position) => {
                        let position =
                            position.to_physical(self.device_hidpi_factor().get() as f64);
                        (position.x as f32, position.y as f32)
                    },
                };
                // Scroll events snap to the major axis of movement, with vertical
                // preferred over horizontal.
                if dy.abs() >= dx.abs() {
                    dx = 0.0;
                } else {
                    dy = 0.0;
                }

                let scroll_location = ScrollLocation::Delta(TypedVector2D::new(dx, dy));
                let phase = winit_phase_to_touch_event_type(phase);
                let event = WindowEvent::Scroll(scroll_location, self.mouse_pos.get(), phase);
                self.event_queue.borrow_mut().push(event);
            },
            Event::WindowEvent {
                event: glutin::WindowEvent::Touch(touch),
                ..
            } => {
                use servo::script_traits::TouchId;

                let phase = winit_phase_to_touch_event_type(touch.phase);
                let id = TouchId(touch.id as i32);
                let position = touch
                    .location
                    .to_physical(self.device_hidpi_factor().get() as f64);
                let point = TypedPoint2D::new(position.x as f32, position.y as f32);
                self.event_queue
                    .borrow_mut()
                    .push(WindowEvent::Touch(phase, id, point));
            },
            Event::WindowEvent {
                event: glutin::WindowEvent::Refresh,
                ..
            } => self.event_queue.borrow_mut().push(WindowEvent::Refresh),
            Event::WindowEvent {
                event: glutin::WindowEvent::CloseRequested,
                ..
            } => {
                self.event_queue.borrow_mut().push(WindowEvent::Quit);
            },
            Event::WindowEvent {
                event: glutin::WindowEvent::Resized(size),
                ..
            } => {
                // size is DeviceIndependentPixel.
                // window.resize() takes DevicePixel.
                if let WindowKind::Window(ref window, _) = self.kind {
                    let size = size.to_physical(self.device_hidpi_factor().get() as f64);
                    window.resize(size);
                }
                // window.set_inner_size() takes DeviceIndependentPixel.
                let (width, height) = size.into();
                let new_size = TypedSize2D::new(width, height);
                if self.inner_size.get() != new_size {
                    self.inner_size.set(new_size);
                    self.event_queue.borrow_mut().push(WindowEvent::Resize);
                }
            },
            Event::Suspended(suspended) => {
                self.suspended.set(suspended);
                if !suspended {
                    self.event_queue.borrow_mut().push(WindowEvent::Idle);
                }
            },
            Event::Awakened => {
                self.event_queue.borrow_mut().push(WindowEvent::Idle);
            },
            _ => {},
        }
    }

    /// Helper function to handle a click
    fn handle_mouse(
        &self,
        button: glutin::MouseButton,
        action: glutin::ElementState,
        coords: TypedPoint2D<i32, DevicePixel>,
    ) {
        use servo::script_traits::MouseButton;

        let max_pixel_dist = 10.0 * self.servo_hidpi_factor().get();
        let event = match action {
            ElementState::Pressed => {
                self.mouse_down_point.set(coords);
                self.mouse_down_button.set(Some(button));
                MouseWindowEvent::MouseDown(MouseButton::Left, coords.to_f32())
            },
            ElementState::Released => {
                let mouse_up_event = MouseWindowEvent::MouseUp(MouseButton::Left, coords.to_f32());
                match self.mouse_down_button.get() {
                    None => mouse_up_event,
                    Some(but) if button == but => {
                        let pixel_dist = self.mouse_down_point.get() - coords;
                        let pixel_dist =
                            ((pixel_dist.x * pixel_dist.x + pixel_dist.y * pixel_dist.y) as f32)
                                .sqrt();
                        if pixel_dist < max_pixel_dist {
                            self.event_queue
                                .borrow_mut()
                                .push(WindowEvent::MouseWindowEventClass(mouse_up_event));
                            MouseWindowEvent::Click(MouseButton::Left, coords.to_f32())
                        } else {
                            mouse_up_event
                        }
                    },
                    Some(_) => mouse_up_event,
                }
            },
        };
        self.event_queue
            .borrow_mut()
            .push(WindowEvent::MouseWindowEventClass(event));
    }

    fn device_hidpi_factor(&self) -> TypedScale<f32, DeviceIndependentPixel, DevicePixel> {
        match self.kind {
            WindowKind::Window(ref window, ..) => TypedScale::new(window.get_hidpi_factor() as f32),
            WindowKind::Headless(..) => TypedScale::new(1.0),
        }
    }

    fn servo_hidpi_factor(&self) -> TypedScale<f32, DeviceIndependentPixel, DevicePixel> {
        match opts::get().device_pixels_per_px {
            Some(device_pixels_per_px) => TypedScale::new(device_pixels_per_px),
            _ => match opts::get().output_file {
                Some(_) => TypedScale::new(1.0),
                None => self.device_hidpi_factor(),
            },
        }
    }

    pub fn set_cursor(&self, cursor: Cursor) {
        match self.kind {
            WindowKind::Window(ref window, ..) => {
                use glutin::MouseCursor;

                let winit_cursor = match cursor {
                    Cursor::Default => MouseCursor::Default,
                    Cursor::Pointer => MouseCursor::Hand,
                    Cursor::ContextMenu => MouseCursor::ContextMenu,
                    Cursor::Help => MouseCursor::Help,
                    Cursor::Progress => MouseCursor::Progress,
                    Cursor::Wait => MouseCursor::Wait,
                    Cursor::Cell => MouseCursor::Cell,
                    Cursor::Crosshair => MouseCursor::Crosshair,
                    Cursor::Text => MouseCursor::Text,
                    Cursor::VerticalText => MouseCursor::VerticalText,
                    Cursor::Alias => MouseCursor::Alias,
                    Cursor::Copy => MouseCursor::Copy,
                    Cursor::Move => MouseCursor::Move,
                    Cursor::NoDrop => MouseCursor::NoDrop,
                    Cursor::NotAllowed => MouseCursor::NotAllowed,
                    Cursor::Grab => MouseCursor::Grab,
                    Cursor::Grabbing => MouseCursor::Grabbing,
                    Cursor::EResize => MouseCursor::EResize,
                    Cursor::NResize => MouseCursor::NResize,
                    Cursor::NeResize => MouseCursor::NeResize,
                    Cursor::NwResize => MouseCursor::NwResize,
                    Cursor::SResize => MouseCursor::SResize,
                    Cursor::SeResize => MouseCursor::SeResize,
                    Cursor::SwResize => MouseCursor::SwResize,
                    Cursor::WResize => MouseCursor::WResize,
                    Cursor::EwResize => MouseCursor::EwResize,
                    Cursor::NsResize => MouseCursor::NsResize,
                    Cursor::NeswResize => MouseCursor::NeswResize,
                    Cursor::NwseResize => MouseCursor::NwseResize,
                    Cursor::ColResize => MouseCursor::ColResize,
                    Cursor::RowResize => MouseCursor::RowResize,
                    Cursor::AllScroll => MouseCursor::AllScroll,
                    Cursor::ZoomIn => MouseCursor::ZoomIn,
                    Cursor::ZoomOut => MouseCursor::ZoomOut,
                    _ => MouseCursor::Default,
                };
                window.set_cursor(winit_cursor);
            },
            WindowKind::Headless(..) => {},
        }
    }
}

impl WindowMethods for Window {
    fn gl(&self) -> Rc<dyn gl::Gl> {
        self.gl.clone()
    }

    fn get_coordinates(&self) -> EmbedderCoordinates {
        match self.kind {
            WindowKind::Window(ref window, _) => {
                // TODO(ajeffrey): can this fail?
                let dpr = self.device_hidpi_factor();
                let LogicalSize { width, height } = window
                    .get_outer_size()
                    .expect("Failed to get window outer size.");
                let LogicalPosition { x, y } = window
                    .get_position()
                    .unwrap_or(LogicalPosition::new(0., 0.));
                let win_size = (TypedSize2D::new(width as f32, height as f32) * dpr).to_i32();
                let win_origin = (TypedPoint2D::new(x as f32, y as f32) * dpr).to_i32();
                let screen = (self.screen_size.to_f32() * dpr).to_i32();

                let LogicalSize { width, height } = window
                    .get_inner_size()
                    .expect("Failed to get window inner size.");
                let inner_size = (TypedSize2D::new(width as f32, height as f32) * dpr).to_i32();
                let viewport = DeviceIntRect::new(TypedPoint2D::zero(), inner_size);
                let framebuffer = FramebufferIntSize::from_untyped(&viewport.size.to_untyped());

                EmbedderCoordinates {
                    viewport,
                    framebuffer,
                    window: (win_size, win_origin),
                    screen: screen,
                    // FIXME: Glutin doesn't have API for available size. Fallback to screen size
                    screen_avail: screen,
                    hidpi_factor: self.servo_hidpi_factor(),
                }
            },
            WindowKind::Headless(ref context) => {
                let dpr = self.servo_hidpi_factor();
                let size =
                    (TypedSize2D::new(context.width, context.height).to_f32() * dpr).to_i32();
                let viewport = DeviceIntRect::new(TypedPoint2D::zero(), size);
                let framebuffer = FramebufferIntSize::from_untyped(&size.to_untyped());
                EmbedderCoordinates {
                    viewport,
                    framebuffer,
                    window: (size, TypedPoint2D::zero()),
                    screen: size,
                    screen_avail: size,
                    hidpi_factor: dpr,
                }
            },
        }
    }

    fn present(&self) {
        match self.kind {
            WindowKind::Window(ref window, ..) => {
                if let Err(err) = window.swap_buffers() {
                    warn!("Failed to swap window buffers ({}).", err);
                }
            },
            WindowKind::Headless(..) => {},
        }
    }

    fn create_event_loop_waker(&self) -> Box<dyn EventLoopWaker> {
        struct GlutinEventLoopWaker {
            proxy: Option<Arc<glutin::EventsLoopProxy>>,
        }
        impl GlutinEventLoopWaker {
            fn new(window: &Window) -> GlutinEventLoopWaker {
                let proxy = match window.kind {
                    WindowKind::Window(_, ref events_loop) => {
                        Some(Arc::new(events_loop.borrow().create_proxy()))
                    },
                    WindowKind::Headless(..) => None,
                };
                GlutinEventLoopWaker { proxy }
            }
        }
        impl EventLoopWaker for GlutinEventLoopWaker {
            fn wake(&self) {
                // kick the OS event loop awake.
                if let Some(ref proxy) = self.proxy {
                    if let Err(err) = proxy.wakeup() {
                        warn!("Failed to wake up event loop ({}).", err);
                    }
                }
            }
            fn clone(&self) -> Box<dyn EventLoopWaker + Send> {
                Box::new(GlutinEventLoopWaker {
                    proxy: self.proxy.clone(),
                })
            }
        }

        Box::new(GlutinEventLoopWaker::new(&self))
    }

    fn set_animation_state(&self, state: AnimationState) {
        self.animation_state.set(state);
    }

    fn prepare_for_composite(&self) -> bool {
        if let WindowKind::Window(ref window, ..) = self.kind {
            if let Err(err) = unsafe { window.context().make_current() } {
                warn!("Couldn't make window current: {}", err);
            }
        };
        true
    }

    fn register_vr_services(
        &self,
        services: &mut VRServiceManager,
        heartbeats: &mut Vec<Box<WebVRMainThreadHeartbeat>>
    ) {
        if pref!(dom.webvr.test) {
            warn!("Creating test VR display");
            // TODO: support dom.webvr.test in headless environments
            if let WindowKind::Window(_, ref events_loop) = self.kind {
                // This is safe, because register_vr_services is called from the main thread.
                let name = String::from("Test VR Display");
                let size = self.inner_size.get().to_f64();
                let size = LogicalSize::new(size.width, size.height);
                let mut window_builder = glutin::WindowBuilder::new()
                    .with_title(name.clone())
                    .with_dimensions(size)
                    .with_visibility(false)
                    .with_multitouch();
                window_builder = builder_with_platform_options(window_builder);
                let context_builder = ContextBuilder::new()
                    .with_gl(Window::gl_version())
                    .with_vsync(false); // Assume the browser vsync is the same as the test VR window vsync
                let gl_window = GlWindow::new(window_builder, context_builder, &*events_loop.borrow())
                    .expect("Failed to create window.");
                let gl = self.gl.clone();
                let (service, heartbeat) = GlWindowVRService::new(name, gl_window, gl);

                services.register(Box::new(service));
                heartbeats.push(Box::new(heartbeat));
            }
        }
    }
}

fn winit_phase_to_touch_event_type(phase: TouchPhase) -> TouchEventType {
    match phase {
        TouchPhase::Started => TouchEventType::Down,
        TouchPhase::Moved => TouchEventType::Move,
        TouchPhase::Ended => TouchEventType::Up,
        TouchPhase::Cancelled => TouchEventType::Cancel,
    }
}

#[cfg(any(target_os = "linux", target_os = "windows"))]
fn load_icon(icon_bytes: &[u8]) -> Icon {
    let (icon_rgba, icon_width, icon_height) = {
        use image::{GenericImageView, Pixel};
        let image = image::load_from_memory(icon_bytes).expect("Failed to load icon");;
        let (width, height) = image.dimensions();
        let mut rgba = Vec::with_capacity((width * height) as usize * 4);
        for (_, _, pixel) in image.pixels() {
            rgba.extend_from_slice(&pixel.to_rgba().data);
        }
        (rgba, width, height)
    };
    Icon::from_rgba(icon_rgba, icon_width, icon_height).expect("Failed to load icon")
}

/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! A glutin window implementation.

use crate::app;
use crate::keyutils::keyboard_event_from_winit;
use crate::window_trait::{WindowPortsMethods, LINE_HEIGHT};
use euclid::{TypedPoint2D, TypedScale, TypedSize2D, TypedVector2D};
use gleam::gl;
use glutin::dpi::{LogicalPosition, LogicalSize, PhysicalSize};
#[cfg(target_os = "macos")]
use glutin::os::macos::{ActivationPolicy, WindowBuilderExt};
#[cfg(any(target_os = "linux", target_os = "windows"))]
use glutin::Icon;
use glutin::{ContextBuilder, GlContext, GlWindow};
use glutin::{ElementState, KeyboardInput, MouseButton, MouseScrollDelta, TouchPhase};
#[cfg(any(target_os = "linux", target_os = "windows"))]
use image;
use keyboard_types::{Key, KeyState, KeyboardEvent};
use servo::compositing::windowing::{AnimationState, MouseWindowEvent, WindowEvent};
use servo::compositing::windowing::{EmbedderCoordinates, WindowMethods};
use servo::embedder_traits::Cursor;
use servo::script_traits::{TouchEventType, WheelMode, WheelDelta};
use servo::servo_config::opts;
use servo::servo_geometry::DeviceIndependentPixel;
use servo::style_traits::DevicePixel;
use servo::webrender_api::{
    DeviceIntPoint, DeviceIntRect, DeviceIntSize, FramebufferIntSize, ScrollLocation,
};
use std::cell::{Cell, RefCell};
use std::mem;
use std::rc::Rc;
#[cfg(target_os = "windows")]
use winapi;

const MULTISAMPLES: u16 = 16;

#[cfg(target_os = "macos")]
fn builder_with_platform_options(mut builder: glutin::WindowBuilder) -> glutin::WindowBuilder {
    if opts::get().output_file.is_some() {
        // Prevent the window from showing in Dock.app, stealing focus,
        // when generating an output file.
        builder = builder.with_activation_policy(ActivationPolicy::Prohibited)
    }
    builder
}

#[cfg(not(target_os = "macos"))]
fn builder_with_platform_options(builder: glutin::WindowBuilder) -> glutin::WindowBuilder {
    builder
}

pub struct Window {
    gl_window: GlWindow,
    screen_size: TypedSize2D<u32, DeviceIndependentPixel>,
    inner_size: Cell<TypedSize2D<u32, DeviceIndependentPixel>>,
    mouse_down_button: Cell<Option<glutin::MouseButton>>,
    mouse_down_point: Cell<TypedPoint2D<i32, DevicePixel>>,
    primary_monitor: glutin::MonitorId,
    event_queue: RefCell<Vec<WindowEvent>>,
    mouse_pos: Cell<TypedPoint2D<i32, DevicePixel>>,
    last_pressed: Cell<Option<KeyboardEvent>>,
    animation_state: Cell<AnimationState>,
    fullscreen: Cell<bool>,
    gl: Rc<dyn gl::Gl>,
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
        win_size: TypedSize2D<u32, DeviceIndependentPixel>,
        events_loop: &glutin::EventsLoop,
    ) -> Rc<dyn WindowPortsMethods> {
        let opts = opts::get();

        // If there's no chrome, start off with the window invisible. It will be set to visible in
        // `load_end()`. This avoids an ugly flash of unstyled content (especially important since
        // unstyled content is white and chrome often has a transparent background). See issue
        // #9996.
        let visible = opts.output_file.is_none() && !opts.no_native_titlebar;

        let win_size: DeviceIntSize = (win_size.to_f32() * window_creation_scale_factor()).to_i32();
        let width = win_size.to_untyped().width;
        let height = win_size.to_untyped().height;

        let mut window_builder = glutin::WindowBuilder::new()
            .with_title("Servo".to_string())
            .with_decorations(!opts.no_native_titlebar)
            .with_transparency(opts.no_native_titlebar)
            .with_dimensions(LogicalSize::new(width as f64, height as f64))
            .with_visibility(visible)
            .with_multitouch();

        window_builder = builder_with_platform_options(window_builder);

        let mut context_builder = ContextBuilder::new()
            .with_gl(app::gl_version())
            .with_vsync(opts.enable_vsync);

        if opts.use_msaa {
            context_builder = context_builder.with_multisampling(MULTISAMPLES)
        }

        let glutin_window = GlWindow::new(window_builder, context_builder, &events_loop)
            .expect("Failed to create window.");

        #[cfg(any(target_os = "linux", target_os = "windows"))]
        {
            let icon_bytes = include_bytes!("../../resources/servo64.png");
            glutin_window.set_window_icon(Some(load_icon(icon_bytes)));
        }

        unsafe {
            glutin_window
                .context()
                .make_current()
                .expect("Couldn't make window current");
        }

        let primary_monitor = events_loop.get_primary_monitor();

        let PhysicalSize {
            width: screen_width,
            height: screen_height,
        } = primary_monitor.get_dimensions();
        let screen_size = TypedSize2D::new(screen_width as u32, screen_height as u32);
        // TODO(ajeffrey): can this fail?
        let LogicalSize { width, height } = glutin_window
            .get_inner_size()
            .expect("Failed to get window inner size.");
        let inner_size = TypedSize2D::new(width as u32, height as u32);

        glutin_window.show();

        let gl = match gl::GlType::default() {
            gl::GlType::Gl => unsafe {
                gl::GlFns::load_with(|s| glutin_window.get_proc_address(s) as *const _)
            },
            gl::GlType::Gles => unsafe {
                gl::GlesFns::load_with(|s| glutin_window.get_proc_address(s) as *const _)
            },
        };

        gl.clear_color(0.6, 0.6, 0.6, 1.0);
        gl.clear(gl::COLOR_BUFFER_BIT);
        gl.finish();

        let window = Window {
            gl_window: glutin_window,
            event_queue: RefCell::new(vec![]),
            mouse_down_button: Cell::new(None),
            mouse_down_point: Cell::new(TypedPoint2D::new(0, 0)),
            mouse_pos: Cell::new(TypedPoint2D::new(0, 0)),
            last_pressed: Cell::new(None),
            gl: gl.clone(),
            animation_state: Cell::new(AnimationState::Idle),
            fullscreen: Cell::new(false),
            inner_size: Cell::new(inner_size),
            primary_monitor,
            screen_size,
        };

        window.present();

        Rc::new(window)
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
            return;
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

    fn handle_keyboard_input(&self, input: KeyboardInput) {
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
        TypedScale::new(self.gl_window.get_hidpi_factor() as f32)
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
}

impl WindowPortsMethods for Window {
    fn get_events(&self) -> Vec<WindowEvent> {
        mem::replace(&mut *self.event_queue.borrow_mut(), Vec::new())
    }

    fn has_events(&self) -> bool {
        !self.event_queue.borrow().is_empty()
    }

    fn page_height(&self) -> f32 {
        let dpr = self.servo_hidpi_factor();
        let size = self
            .gl_window
            .get_inner_size()
            .expect("Failed to get window inner size.");
        size.height as f32 * dpr.get()
    }

    fn set_title(&self, title: &str) {
        self.gl_window.set_title(title);
    }

    fn set_inner_size(&self, size: DeviceIntSize) {
        let size = size.to_f32() / self.device_hidpi_factor();
        self.gl_window
            .set_inner_size(LogicalSize::new(size.width.into(), size.height.into()))
    }

    fn set_position(&self, point: DeviceIntPoint) {
        let point = point.to_f32() / self.device_hidpi_factor();
        self.gl_window
            .set_position(LogicalPosition::new(point.x.into(), point.y.into()))
    }

    fn set_fullscreen(&self, state: bool) {
        if self.fullscreen.get() != state {
            self.gl_window
                .set_fullscreen(Some(self.primary_monitor.clone()));
        }
        self.fullscreen.set(state);
    }

    fn get_fullscreen(&self) -> bool {
        return self.fullscreen.get();
    }

    fn set_cursor(&self, cursor: Cursor) {
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
        self.gl_window.set_cursor(winit_cursor);
    }

    fn is_animating(&self) -> bool {
        self.animation_state.get() == AnimationState::Animating
    }

    fn id(&self) -> Option<glutin::WindowId> {
        Some(self.gl_window.id())
    }

    fn winit_event_to_servo_event(&self, event: glutin::WindowEvent) {
        match event {
            glutin::WindowEvent::ReceivedCharacter(ch) => self.handle_received_character(ch),
            glutin::WindowEvent::KeyboardInput { input, .. } => self.handle_keyboard_input(input),
            glutin::WindowEvent::MouseInput { state, button, .. } => {
                if button == MouseButton::Left || button == MouseButton::Right {
                    self.handle_mouse(button, state, self.mouse_pos.get());
                }
            },
            glutin::WindowEvent::CursorMoved { position, .. } => {
                let pos = position.to_physical(self.device_hidpi_factor().get() as f64);
                let (x, y): (i32, i32) = pos.into();
                self.mouse_pos.set(TypedPoint2D::new(x, y));
                self.event_queue
                    .borrow_mut()
                    .push(WindowEvent::MouseWindowMoveEventClass(TypedPoint2D::new(
                        x as f32, y as f32,
                    )));
            },
            glutin::WindowEvent::MouseWheel { delta, phase, .. } => {
                let (mut dx, mut dy, mode) = match delta {
                    MouseScrollDelta::LineDelta(dx, dy) => (dx as f64, (dy * LINE_HEIGHT) as f64,
                                                            WheelMode::DeltaLine),
                    MouseScrollDelta::PixelDelta(position) => {
                        let position =
                            position.to_physical(self.device_hidpi_factor().get() as f64);
                        (position.x as f64, position.y as f64, WheelMode::DeltaPixel)
                    },
                };

                // Create wheel event before snapping to the major axis of movement
                let wheel_delta = WheelDelta { x: dx, y: dy, z: 0.0, mode };
                let pos = self.mouse_pos.get();
                let position = TypedPoint2D::new(pos.x as f32, pos.y as f32);
                let wheel_event = WindowEvent::Wheel(wheel_delta, position);

                // Scroll events snap to the major axis of movement, with vertical
                // preferred over horizontal.
                if dy.abs() >= dx.abs() {
                    dx = 0.0;
                } else {
                    dy = 0.0;
                }

                let scroll_location = ScrollLocation::Delta(TypedVector2D::new(dx as f32, dy as f32));
                let phase = winit_phase_to_touch_event_type(phase);
                let scroll_event = WindowEvent::Scroll(scroll_location, self.mouse_pos.get(), phase);

                // Send events
                self.event_queue.borrow_mut().push(wheel_event);
                self.event_queue.borrow_mut().push(scroll_event);
            },
            glutin::WindowEvent::Touch(touch) => {
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
            glutin::WindowEvent::Refresh => {
                self.event_queue.borrow_mut().push(WindowEvent::Refresh);
            },
            glutin::WindowEvent::CloseRequested => {
                self.event_queue.borrow_mut().push(WindowEvent::Quit);
            },
            glutin::WindowEvent::Resized(size) => {
                // size is DeviceIndependentPixel.
                // gl_window.resize() takes DevicePixel.
                let size = size.to_physical(self.device_hidpi_factor().get() as f64);
                self.gl_window.resize(size);
                // window.set_inner_size() takes DeviceIndependentPixel.
                let (width, height) = size.into();
                let new_size = TypedSize2D::new(width, height);
                if self.inner_size.get() != new_size {
                    self.inner_size.set(new_size);
                    self.event_queue.borrow_mut().push(WindowEvent::Resize);
                }
            },
            _ => {},
        }
    }
}

impl WindowMethods for Window {
    fn gl(&self) -> Rc<dyn gl::Gl> {
        self.gl.clone()
    }

    fn get_coordinates(&self) -> EmbedderCoordinates {
        // TODO(ajeffrey): can this fail?
        let dpr = self.device_hidpi_factor();
        let LogicalSize { width, height } = self
            .gl_window
            .get_outer_size()
            .expect("Failed to get window outer size.");
        let LogicalPosition { x, y } = self
            .gl_window
            .get_position()
            .unwrap_or(LogicalPosition::new(0., 0.));
        let win_size = (TypedSize2D::new(width as f32, height as f32) * dpr).to_i32();
        let win_origin = (TypedPoint2D::new(x as f32, y as f32) * dpr).to_i32();
        let screen = (self.screen_size.to_f32() * dpr).to_i32();

        let LogicalSize { width, height } = self
            .gl_window
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
    }

    fn present(&self) {
        if let Err(err) = self.gl_window.swap_buffers() {
            warn!("Failed to swap window buffers ({}).", err);
        }
    }

    fn set_animation_state(&self, state: AnimationState) {
        self.animation_state.set(state);
    }

    fn prepare_for_composite(&self) -> bool {
        if let Err(err) = unsafe { self.gl_window.context().make_current() } {
            warn!("Couldn't make window current: {}", err);
        }
        true
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

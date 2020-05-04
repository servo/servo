/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! A winit window implementation.

use crate::events_loop::EventsLoop;
use crate::keyutils::keyboard_event_from_winit;
use crate::window_trait::{WindowPortsMethods, LINE_HEIGHT};
use euclid::{
    Angle, Point2D, Rotation3D, Scale, Size2D, UnknownUnit,
    Vector2D, Vector3D,
};
use winit::dpi::{LogicalPosition, LogicalSize, PhysicalSize};
#[cfg(target_os = "macos")]
use winit::os::macos::{ActivationPolicy, WindowBuilderExt};
#[cfg(any(target_os = "linux", target_os = "windows"))]
use winit::Icon;
use winit::{ElementState, KeyboardInput, MouseButton, MouseScrollDelta, TouchPhase, VirtualKeyCode};
#[cfg(any(target_os = "linux", target_os = "windows"))]
use image;
use keyboard_types::{Key, KeyState, KeyboardEvent};
use servo::compositing::windowing::{AnimationState, MouseWindowEvent, WindowEvent};
use servo::compositing::windowing::{EmbedderCoordinates, WindowMethods};
use servo::embedder_traits::Cursor;
use servo::script_traits::{TouchEventType, WheelMode, WheelDelta};
use servo::servo_config::opts;
use servo::servo_config::pref;
use servo::servo_geometry::DeviceIndependentPixel;
use servo::style_traits::DevicePixel;
use servo::webrender_api::ScrollLocation;
use servo::webrender_api::units::{DeviceIntPoint, DeviceIntRect, DeviceIntSize};
use servo::webrender_surfman::WebrenderSurfman;
use servo_media::player::context::{GlApi, GlContext as PlayerGLContext, NativeDisplay};
use std::cell::{Cell, RefCell};
use std::mem;
use std::rc::Rc;
#[cfg(target_os = "linux")]
use surfman::platform::generic::multi::connection::NativeConnection;
#[cfg(target_os = "linux")]
use surfman::platform::generic::multi::context::NativeContext;
use surfman::Connection;
use surfman::Device;
use surfman::GLApi;
use surfman::GLVersion;
use surfman::NativeWidget;
use surfman::SurfaceType;
#[cfg(target_os = "windows")]
use winapi;

#[cfg(target_os = "macos")]
fn builder_with_platform_options(mut builder: winit::WindowBuilder) -> winit::WindowBuilder {
    if opts::get().output_file.is_some() {
        // Prevent the window from showing in Dock.app, stealing focus,
        // when generating an output file.
        builder = builder.with_activation_policy(ActivationPolicy::Prohibited)
    }
    builder
}

#[cfg(not(target_os = "macos"))]
fn builder_with_platform_options(builder: winit::WindowBuilder) -> winit::WindowBuilder {
    builder
}

pub struct Window {
    winit_window: winit::Window,
    webrender_surfman: WebrenderSurfman,
    screen_size: Size2D<u32, DeviceIndependentPixel>,
    inner_size: Cell<Size2D<u32, DeviceIndependentPixel>>,
    mouse_down_button: Cell<Option<winit::MouseButton>>,
    mouse_down_point: Cell<Point2D<i32, DevicePixel>>,
    primary_monitor: winit::MonitorId,
    event_queue: RefCell<Vec<WindowEvent>>,
    mouse_pos: Cell<Point2D<i32, DevicePixel>>,
    last_pressed: Cell<Option<KeyboardEvent>>,
    animation_state: Cell<AnimationState>,
    fullscreen: Cell<bool>,
    device_pixels_per_px: Option<f32>,
    xr_window_poses: RefCell<Vec<Rc<XRWindowPose>>>,
}

#[cfg(not(target_os = "windows"))]
fn window_creation_scale_factor() -> Scale<f32, DeviceIndependentPixel, DevicePixel> {
    Scale::new(1.0)
}

#[cfg(target_os = "windows")]
fn window_creation_scale_factor() -> Scale<f32, DeviceIndependentPixel, DevicePixel> {
    let hdc = unsafe { winapi::um::winuser::GetDC(::std::ptr::null_mut()) };
    let ppi = unsafe { winapi::um::wingdi::GetDeviceCaps(hdc, winapi::um::wingdi::LOGPIXELSY) };
    Scale::new(ppi as f32 / 96.0)
}

impl Window {
    pub fn new(
        win_size: Size2D<u32, DeviceIndependentPixel>,
        events_loop: Rc<RefCell<EventsLoop>>,
        no_native_titlebar: bool,
        device_pixels_per_px: Option<f32>,
    ) -> Window {
        let opts = opts::get();

        // If there's no chrome, start off with the window invisible. It will be set to visible in
        // `load_end()`. This avoids an ugly flash of unstyled content (especially important since
        // unstyled content is white and chrome often has a transparent background). See issue
        // #9996.
        let visible = opts.output_file.is_none() && !no_native_titlebar;

        let win_size: DeviceIntSize = (win_size.to_f32() * window_creation_scale_factor()).to_i32();
        let width = win_size.to_untyped().width;
        let height = win_size.to_untyped().height;

        let mut window_builder = winit::WindowBuilder::new()
            .with_title("Servo".to_string())
            .with_decorations(!no_native_titlebar)
            .with_transparency(no_native_titlebar)
            .with_dimensions(LogicalSize::new(width as f64, height as f64))
            .with_visibility(visible)
            .with_multitouch();

        window_builder = builder_with_platform_options(window_builder);

        let winit_window = window_builder.build(events_loop.borrow().as_winit()).expect("Failed to create window.");

        #[cfg(any(target_os = "linux", target_os = "windows"))]
        {
            let icon_bytes = include_bytes!("../../resources/servo64.png");
            winit_window.set_window_icon(Some(load_icon(icon_bytes)));
        }

        let primary_monitor = events_loop.borrow().as_winit().get_primary_monitor();

        let PhysicalSize {
            width: screen_width,
            height: screen_height,
        } = primary_monitor.get_dimensions();
        let screen_size = Size2D::new(screen_width as u32, screen_height as u32);
        // TODO(ajeffrey): can this fail?
        let LogicalSize { width, height } = winit_window
            .get_inner_size()
            .expect("Failed to get window inner size.");
        let inner_size = Size2D::new(width as u32, height as u32);

        winit_window.show();

        // Initialize surfman
        let connection = Connection::from_winit_window(&winit_window).expect("Failed to create connection");
        let adapter = connection.create_adapter().expect("Failed to create adapter");
        let native_widget = connection
            .create_native_widget_from_winit_window(&winit_window)
            .expect("Failed to create native widget");
        let surface_type = SurfaceType::Widget { native_widget };
        let webrender_surfman = WebrenderSurfman::create(
            &connection,
            &adapter,
            surface_type,
        ).expect("Failed to create WR surfman");

        debug!("Created window {:?}", winit_window.id());
        Window {
            winit_window,
            webrender_surfman,
            event_queue: RefCell::new(vec![]),
            mouse_down_button: Cell::new(None),
            mouse_down_point: Cell::new(Point2D::new(0, 0)),
            mouse_pos: Cell::new(Point2D::new(0, 0)),
            last_pressed: Cell::new(None),
            animation_state: Cell::new(AnimationState::Idle),
            fullscreen: Cell::new(false),
            inner_size: Cell::new(inner_size),
            primary_monitor,
            screen_size,
            device_pixels_per_px,
            xr_window_poses: RefCell::new(vec![]),
        }
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
        let xr_poses = self.xr_window_poses.borrow();
        for xr_window_pose in &*xr_poses {
            xr_window_pose.handle_xr_translation(&event);
        }
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
            let xr_poses = self.xr_window_poses.borrow();
            for xr_window_pose in &*xr_poses {
                xr_window_pose.handle_xr_rotation(&input);
            }
            self.event_queue
                .borrow_mut()
                .push(WindowEvent::Keyboard(event));
        }
    }

    /// Helper function to handle a click
    fn handle_mouse(
        &self,
        button: winit::MouseButton,
        action: winit::ElementState,
        coords: Point2D<i32, DevicePixel>,
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

    fn device_hidpi_factor(&self) -> Scale<f32, DeviceIndependentPixel, DevicePixel> {
        Scale::new(self.winit_window.get_hidpi_factor() as f32)
    }

    fn servo_hidpi_factor(&self) -> Scale<f32, DeviceIndependentPixel, DevicePixel> {
        match self.device_pixels_per_px {
            Some(device_pixels_per_px) => Scale::new(device_pixels_per_px),
            _ => match opts::get().output_file {
                Some(_) => Scale::new(1.0),
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
            .winit_window
            .get_inner_size()
            .expect("Failed to get window inner size.");
        size.height as f32 * dpr.get()
    }

    fn set_title(&self, title: &str) {
        self.winit_window.set_title(title);
    }

    fn set_inner_size(&self, size: DeviceIntSize) {
        let size = size.to_f32() / self.device_hidpi_factor();
        self.winit_window
            .set_inner_size(LogicalSize::new(size.width.into(), size.height.into()))
    }

    fn set_position(&self, point: DeviceIntPoint) {
        let point = point.to_f32() / self.device_hidpi_factor();
        self.winit_window
            .set_position(LogicalPosition::new(point.x.into(), point.y.into()))
    }

    fn set_fullscreen(&self, state: bool) {
        if self.fullscreen.get() != state {
            self.winit_window
                .set_fullscreen(if state { Some(self.primary_monitor.clone()) } else { None });
        }
        self.fullscreen.set(state);
    }

    fn get_fullscreen(&self) -> bool {
        return self.fullscreen.get();
    }

    fn set_cursor(&self, cursor: Cursor) {
        use winit::MouseCursor;

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
        self.winit_window.set_cursor(winit_cursor);
    }

    fn is_animating(&self) -> bool {
        self.animation_state.get() == AnimationState::Animating
    }

    fn id(&self) -> winit::WindowId {
        self.winit_window.id()
    }

    fn winit_event_to_servo_event(&self, event: winit::WindowEvent) {
        match event {
            winit::WindowEvent::ReceivedCharacter(ch) => self.handle_received_character(ch),
            winit::WindowEvent::KeyboardInput { input, .. } => self.handle_keyboard_input(input),
            winit::WindowEvent::MouseInput { state, button, .. } => {
                if button == MouseButton::Left || button == MouseButton::Right {
                    self.handle_mouse(button, state, self.mouse_pos.get());
                }
            },
            winit::WindowEvent::CursorMoved { position, .. } => {
                let pos = position.to_physical(self.device_hidpi_factor().get() as f64);
                let (x, y): (i32, i32) = pos.into();
                self.mouse_pos.set(Point2D::new(x, y));
                self.event_queue
                    .borrow_mut()
                    .push(WindowEvent::MouseWindowMoveEventClass(Point2D::new(
                        x as f32, y as f32,
                    )));
            },
            winit::WindowEvent::MouseWheel { delta, phase, .. } => {
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
                let position = Point2D::new(pos.x as f32, pos.y as f32);
                let wheel_event = WindowEvent::Wheel(wheel_delta, position);

                // Scroll events snap to the major axis of movement, with vertical
                // preferred over horizontal.
                if dy.abs() >= dx.abs() {
                    dx = 0.0;
                } else {
                    dy = 0.0;
                }

                let scroll_location = ScrollLocation::Delta(Vector2D::new(dx as f32, dy as f32));
                let phase = winit_phase_to_touch_event_type(phase);
                let scroll_event = WindowEvent::Scroll(scroll_location, self.mouse_pos.get(), phase);

                // Send events
                self.event_queue.borrow_mut().push(wheel_event);
                self.event_queue.borrow_mut().push(scroll_event);
            },
            winit::WindowEvent::Touch(touch) => {
                use servo::script_traits::TouchId;

                let phase = winit_phase_to_touch_event_type(touch.phase);
                let id = TouchId(touch.id as i32);
                let position = touch
                    .location
                    .to_physical(self.device_hidpi_factor().get() as f64);
                let point = Point2D::new(position.x as f32, position.y as f32);
                self.event_queue
                    .borrow_mut()
                    .push(WindowEvent::Touch(phase, id, point));
            },
            winit::WindowEvent::Refresh => {
                self.event_queue.borrow_mut().push(WindowEvent::Refresh);
            },
            winit::WindowEvent::CloseRequested => {
                self.event_queue.borrow_mut().push(WindowEvent::Quit);
            },
            winit::WindowEvent::Resized(size) => {
                let (width, height) = size.into();
                let new_size = Size2D::new(width, height);
                if self.inner_size.get() != new_size {
                    let physical_size = size.to_physical(self.device_hidpi_factor().get() as f64);
                    let physical_size = Size2D::new(physical_size.width, physical_size.height);
                    self.webrender_surfman.resize(physical_size.to_i32()).expect("Failed to resize");
                    self.inner_size.set(new_size);
                    self.event_queue.borrow_mut().push(WindowEvent::Resize);
                }
            },
            _ => {},
        }
    }

    fn new_glwindow(&self, events_loop: &EventsLoop) -> Box<dyn webxr::glwindow::GlWindow> {
        let size = self.winit_window.get_outer_size()
            .expect("Failed to get window outer size");

        let mut window_builder = winit::WindowBuilder::new()
            .with_title("Servo XR".to_string())
            .with_dimensions(size)
            .with_visibility(true);

        window_builder = builder_with_platform_options(window_builder);

        let winit_window = window_builder.build(events_loop.as_winit())
            .expect("Failed to create window.");

        let pose = Rc::new(XRWindowPose {
            xr_rotation: Cell::new(Rotation3D::identity()),
            xr_translation: Cell::new(Vector3D::zero()),
        });
        self.xr_window_poses.borrow_mut().push(pose.clone());
        Box::new(XRWindow { winit_window, pose })
    }
}

impl WindowMethods for Window {
    fn get_coordinates(&self) -> EmbedderCoordinates {
        // TODO(ajeffrey): can this fail?
        let dpr = self.device_hidpi_factor();
        let LogicalSize { width, height } = self
            .winit_window
            .get_outer_size()
            .expect("Failed to get window outer size.");
        let LogicalPosition { x, y } = self
            .winit_window
            .get_position()
            .unwrap_or(LogicalPosition::new(0., 0.));
        let win_size = (Size2D::new(width as f32, height as f32) * dpr).to_i32();
        let win_origin = (Point2D::new(x as f32, y as f32) * dpr).to_i32();
        let screen = (self.screen_size.to_f32() * dpr).to_i32();

        let LogicalSize { width, height } = self
            .winit_window
            .get_inner_size()
            .expect("Failed to get window inner size.");
        let inner_size = (Size2D::new(width as f32, height as f32) * dpr).to_i32();
        let viewport = DeviceIntRect::new(Point2D::zero(), inner_size);
        let framebuffer = DeviceIntSize::from_untyped(viewport.size.to_untyped());
        EmbedderCoordinates {
            viewport,
            framebuffer,
            window: (win_size, win_origin),
            screen: screen,
            // FIXME: Winit doesn't have API for available size. Fallback to screen size
            screen_avail: screen,
            hidpi_factor: self.servo_hidpi_factor(),
        }
    }

    fn set_animation_state(&self, state: AnimationState) {
        self.animation_state.set(state);
    }

    fn webrender_surfman(&self) -> WebrenderSurfman {
        self.webrender_surfman.clone()
    }

    fn get_gl_context(&self) -> PlayerGLContext {
        if !pref!(media.glvideo.enabled) {
            return PlayerGLContext::Unknown;
        }

        #[allow(unused_variables)]
        let native_context = self.webrender_surfman.native_context();

        #[cfg(target_os = "windows")]
        return PlayerGLContext::Egl(native_context.egl_context as usize);

        #[cfg(target_os = "linux")]
        return match native_context {
            NativeContext::Default(NativeContext::Default(native_context)) =>
                PlayerGLContext::Egl(native_context.egl_context as usize),
            NativeContext::Default(NativeContext::Alternate(native_context)) =>
                PlayerGLContext::Egl(native_context.egl_context as usize),
            NativeContext::Alternate(_) => unimplemented!(),
        };

        // @TODO(victor): https://github.com/servo/media/pull/315
        #[cfg(target_os = "macos")]
        #[allow(unreachable_code)]
        return unimplemented!();

        #[cfg(not(any(target_os = "linux", target_os = "windows", target_os = "macos")))]
        return unimplemented!();
    }

    fn get_native_display(&self) -> NativeDisplay {
        if !pref!(media.glvideo.enabled) {
            return NativeDisplay::Unknown;
        }

        #[allow(unused_variables)]
        let native_connection = self.webrender_surfman.connection().native_connection();
        #[allow(unused_variables)]
        let native_device = self.webrender_surfman.native_device();

        #[cfg(target_os = "windows")]
        return NativeDisplay::Egl(native_device.egl_display as usize);

        #[cfg(target_os = "linux")]
        return match native_connection {
            NativeConnection::Default(NativeConnection::Default(conn)) =>
                NativeDisplay::Egl(conn.0 as usize),
            NativeConnection::Default(NativeConnection::Alternate(conn)) =>
                NativeDisplay::X11(conn.x11_display as usize),
            NativeConnection::Alternate(_) => unimplemented!(),
        };

        // @TODO(victor): https://github.com/servo/media/pull/315
        #[cfg(target_os = "macos")]
        #[allow(unreachable_code)]
        return unimplemented!();

        #[cfg(not(any(target_os = "linux", target_os = "windows", target_os = "macos")))]
        return unimplemented!();
    }

    fn get_gl_api(&self) -> GlApi {
        let api = self.webrender_surfman.connection().gl_api();
        let attributes = self.webrender_surfman.context_attributes();
        let GLVersion { major, minor } = attributes.version;
        match api {
            GLApi::GL if major >= 3 && minor >= 2 => GlApi::OpenGL3,
            GLApi::GL => GlApi::OpenGL,
            GLApi::GLES if major > 1 => GlApi::Gles2,
            GLApi::GLES => GlApi::Gles1,
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
        let image = image::load_from_memory(icon_bytes).expect("Failed to load icon");
        let (width, height) = image.dimensions();
        let mut rgba = Vec::with_capacity((width * height) as usize * 4);
        for (_, _, pixel) in image.pixels() {
            rgba.extend_from_slice(&pixel.to_rgba().0);
        }
        (rgba, width, height)
    };
    Icon::from_rgba(icon_rgba, icon_width, icon_height).expect("Failed to load icon")
}

struct XRWindow {
    winit_window: winit::Window,
    pose: Rc<XRWindowPose>,
}

struct XRWindowPose {
    xr_rotation: Cell<Rotation3D<f32, UnknownUnit, UnknownUnit>>,
    xr_translation: Cell<Vector3D<f32, UnknownUnit>>,
}

impl webxr::glwindow::GlWindow for XRWindow {
    fn get_native_widget(&self, device: &Device) -> NativeWidget {
        device.connection()
            .create_native_widget_from_winit_window(&self.winit_window)
            .expect("Failed to create native widget")
    }

    fn get_rotation(&self) -> Rotation3D<f32, UnknownUnit, UnknownUnit> {
        self.pose.xr_rotation.get().clone()
    }

    fn get_translation(&self) -> Vector3D<f32, UnknownUnit> {
        self.pose.xr_translation.get().clone()
    }

    fn get_mode(&self) -> webxr::glwindow::GlWindowMode {
        if pref!(dom.webxr.glwindow.red_cyan) {
            webxr::glwindow::GlWindowMode::StereoRedCyan
        } else if pref!(dom.webxr.glwindow.left_right) {
            webxr::glwindow::GlWindowMode::StereoLeftRight
        } else {
            webxr::glwindow::GlWindowMode::Blit
        }
    }
}

impl XRWindowPose {
    fn handle_xr_translation(&self, input: &KeyboardEvent) {
        if input.state != KeyState::Down {
            return;
        }
        const NORMAL_TRANSLATE: f32 = 0.1;
        const QUICK_TRANSLATE: f32 = 1.0;
        let mut x = 0.0;
        let mut z = 0.0;
        match input.key {
            Key::Character(ref k) => match &**k {
                "w" => z = -NORMAL_TRANSLATE,
                "W" => z = -QUICK_TRANSLATE,
                "s" => z = NORMAL_TRANSLATE,
                "S" => z = QUICK_TRANSLATE,
                "a" => x = -NORMAL_TRANSLATE,
                "A" => x = -QUICK_TRANSLATE,
                "d" => x = NORMAL_TRANSLATE,
                "D" => x = QUICK_TRANSLATE,
                _ => return,
            },
            _ => return,
        };
        let (old_x, old_y, old_z) = self.xr_translation.get().to_tuple();
        let vec = Vector3D::new(x + old_x, old_y, z + old_z);
        self.xr_translation.set(vec);
    }

    fn handle_xr_rotation(&self, input: &KeyboardInput) {
        if input.state != winit::ElementState::Pressed {
            return;
        }
        let mut x = 0.0;
        let mut y = 0.0;
        match input.virtual_keycode {
            Some(VirtualKeyCode::Up) => x = 1.0,
            Some(VirtualKeyCode::Down) => x = -1.0,
            Some(VirtualKeyCode::Left) => y = 1.0,
            Some(VirtualKeyCode::Right) => y = -1.0,
            _ => return,
        };
        if input.modifiers.shift {
            x = 10.0 * x;
            y = 10.0 * y;
        }
        let x: Rotation3D<_, UnknownUnit, UnknownUnit> = Rotation3D::around_x(Angle::degrees(x));
        let y: Rotation3D<_, UnknownUnit, UnknownUnit> = Rotation3D::around_y(Angle::degrees(y));
        let rotation = self.xr_rotation.get().post_rotate(&x).post_rotate(&y);
        self.xr_rotation.set(rotation);
    }
}

/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! A winit window implementation.

use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::rc::Rc;

use euclid::num::Zero;
use euclid::{Angle, Length, Point2D, Rotation3D, Scale, Size2D, UnknownUnit, Vector2D, Vector3D};
use log::{debug, info, trace};
use raw_window_handle::{HasRawDisplayHandle, HasRawWindowHandle};
use servo::compositing::windowing::{
    AnimationState, EmbedderCoordinates, EmbedderEvent, MouseWindowEvent, WindowMethods,
};
use servo::embedder_traits::Cursor;
use servo::keyboard_types::{Key, KeyState, KeyboardEvent};
use servo::rendering_context::RenderingContext;
use servo::script_traits::{TouchEventType, WheelDelta, WheelMode};
use servo::servo_config::{opts, pref};
use servo::servo_geometry::DeviceIndependentPixel;
use servo::style_traits::DevicePixel;
use servo::webrender_api::units::{DeviceIntPoint, DeviceIntRect, DeviceIntSize};
use servo::webrender_api::ScrollLocation;
use surfman::{Connection, Context, Device, SurfaceType};
#[cfg(target_os = "windows")]
use winapi;
use winit::dpi::{LogicalPosition, PhysicalPosition, PhysicalSize};
use winit::event::{ElementState, KeyEvent, MouseButton, MouseScrollDelta, TouchPhase};
use winit::keyboard::{Key as LogicalKey, ModifiersState, NamedKey};
#[cfg(any(target_os = "linux", target_os = "windows"))]
use winit::window::Icon;

use crate::events_loop::{EventsLoop, WakerEvent};
use crate::geometry::{winit_position_to_euclid_point, winit_size_to_euclid_size};
use crate::keyutils::keyboard_event_from_winit;
use crate::window_trait::{WindowPortsMethods, LINE_HEIGHT};

pub struct Window {
    winit_window: winit::window::Window,
    rendering_context: RenderingContext,
    screen_size: Size2D<u32, DevicePixel>,
    inner_size: Cell<Size2D<u32, DevicePixel>>,
    toolbar_height: Cell<Length<f32, DeviceIndependentPixel>>,
    mouse_down_button: Cell<Option<winit::event::MouseButton>>,
    mouse_down_point: Cell<Point2D<i32, DevicePixel>>,
    primary_monitor: winit::monitor::MonitorHandle,
    event_queue: RefCell<Vec<EmbedderEvent>>,
    mouse_pos: Cell<Point2D<i32, DevicePixel>>,
    last_pressed: Cell<Option<(KeyboardEvent, Option<LogicalKey>)>>,
    /// A map of winit's key codes to key values that are interpreted from
    /// winit's ReceivedChar events.
    keys_down: RefCell<HashMap<LogicalKey, Key>>,
    animation_state: Cell<AnimationState>,
    fullscreen: Cell<bool>,
    device_pixel_ratio_override: Option<f32>,
    xr_window_poses: RefCell<Vec<Rc<XRWindowPose>>>,
    modifiers_state: Cell<ModifiersState>,
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
        events_loop: &EventsLoop,
        no_native_titlebar: bool,
        device_pixel_ratio_override: Option<f32>,
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

        let window_builder = winit::window::WindowBuilder::new()
            .with_title("Servo".to_string())
            .with_decorations(!no_native_titlebar)
            .with_transparent(no_native_titlebar)
            .with_inner_size(PhysicalSize::new(width as f64, height as f64))
            .with_visible(visible);

        let winit_window = window_builder
            .build(events_loop.as_winit())
            .expect("Failed to create window.");

        #[cfg(any(target_os = "linux", target_os = "windows"))]
        {
            let icon_bytes = include_bytes!("../../resources/servo_64.png");
            winit_window.set_window_icon(Some(load_icon(icon_bytes)));
        }

        let primary_monitor = events_loop
            .as_winit()
            .available_monitors()
            .nth(0)
            .expect("No monitor detected");

        let screen_size = winit_size_to_euclid_size(primary_monitor.size());
        let inner_size = winit_size_to_euclid_size(winit_window.inner_size());

        // Initialize surfman
        let display_handle = winit_window.raw_display_handle();
        let connection = Connection::from_raw_display_handle(display_handle)
            .expect("Failed to create connection");
        let adapter = connection
            .create_adapter()
            .expect("Failed to create adapter");
        let window_handle = winit_window.raw_window_handle();
        let native_widget = connection
            .create_native_widget_from_raw_window_handle(window_handle, Size2D::new(width, height))
            .expect("Failed to create native widget");
        let surface_type = SurfaceType::Widget { native_widget };
        let rendering_context = RenderingContext::create(&connection, &adapter, surface_type)
            .expect("Failed to create WR surfman");

        debug!("Created window {:?}", winit_window.id());
        Window {
            winit_window,
            rendering_context,
            event_queue: RefCell::new(vec![]),
            mouse_down_button: Cell::new(None),
            mouse_down_point: Cell::new(Point2D::new(0, 0)),
            mouse_pos: Cell::new(Point2D::new(0, 0)),
            last_pressed: Cell::new(None),
            keys_down: RefCell::new(HashMap::new()),
            animation_state: Cell::new(AnimationState::Idle),
            fullscreen: Cell::new(false),
            inner_size: Cell::new(inner_size),
            primary_monitor,
            screen_size,
            device_pixel_ratio_override,
            xr_window_poses: RefCell::new(vec![]),
            modifiers_state: Cell::new(ModifiersState::empty()),
            toolbar_height: Cell::new(Default::default()),
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
        let (mut event, key_code) = if let Some((event, key_code)) = self.last_pressed.replace(None)
        {
            (event, key_code)
        } else if ch.is_ascii() {
            // Some keys like Backspace emit a control character in winit
            // but they are already dealt with in handle_keyboard_input
            // so just ignore the character.
            return;
        } else {
            // For combined characters like the letter e with an acute accent
            // no keyboard event is emitted. A dummy event is created in this case.
            (KeyboardEvent::default(), None)
        };
        event.key = Key::Character(ch.to_string());

        if event.state == KeyState::Down {
            // Ensure that when we receive a keyup event from winit, we are able
            // to infer that it's related to this character and set the event
            // properties appropriately.
            if let Some(key_code) = key_code {
                self.keys_down
                    .borrow_mut()
                    .insert(key_code, event.key.clone());
            }
        }

        let xr_poses = self.xr_window_poses.borrow();
        for xr_window_pose in &*xr_poses {
            xr_window_pose.handle_xr_translation(&event);
        }
        self.event_queue
            .borrow_mut()
            .push(EmbedderEvent::Keyboard(event));
    }

    fn handle_keyboard_input(&self, input: KeyEvent) {
        if let Some(input_text) = &input.text {
            for ch in input_text.chars() {
                self.handle_received_character(ch);
            }
        }

        let mut event = keyboard_event_from_winit(&input, self.modifiers_state.get());
        trace!("handling {:?}", event);
        if event.state == KeyState::Down && event.key == Key::Unidentified {
            // If pressed and probably printable, we expect a ReceivedCharacter event.
            // Wait for that to be received and don't queue any event right now.
            self.last_pressed
                .set(Some((event, Some(input.logical_key))));
            return;
        } else if event.state == KeyState::Up && event.key == Key::Unidentified {
            // If release and probably printable, this is following a ReceiverCharacter event.
            if let Some(key) = self.keys_down.borrow_mut().remove(&input.logical_key) {
                event.key = key;
            }
        }

        if event.key != Key::Unidentified {
            self.last_pressed.set(None);
            let xr_poses = self.xr_window_poses.borrow();
            for xr_window_pose in &*xr_poses {
                xr_window_pose.handle_xr_rotation(&input, self.modifiers_state.get());
            }
            self.event_queue
                .borrow_mut()
                .push(EmbedderEvent::Keyboard(event));
        }
    }

    /// Helper function to handle a click
    fn handle_mouse(
        &self,
        button: winit::event::MouseButton,
        action: winit::event::ElementState,
        coords: Point2D<i32, DevicePixel>,
    ) {
        use servo::script_traits::MouseButton;

        let max_pixel_dist = 10.0 * self.hidpi_factor().get();
        let mouse_button = match &button {
            winit::event::MouseButton::Left => MouseButton::Left,
            winit::event::MouseButton::Right => MouseButton::Right,
            winit::event::MouseButton::Middle => MouseButton::Middle,
            _ => MouseButton::Left,
        };
        let event = match action {
            ElementState::Pressed => {
                self.mouse_down_point.set(coords);
                self.mouse_down_button.set(Some(button));
                MouseWindowEvent::MouseDown(mouse_button, coords.to_f32())
            },
            ElementState::Released => {
                let mouse_up_event = MouseWindowEvent::MouseUp(mouse_button, coords.to_f32());
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
                                .push(EmbedderEvent::MouseWindowEventClass(mouse_up_event));
                            MouseWindowEvent::Click(mouse_button, coords.to_f32())
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
            .push(EmbedderEvent::MouseWindowEventClass(event));
    }
}

impl WindowPortsMethods for Window {
    fn get_events(&self) -> Vec<EmbedderEvent> {
        std::mem::take(&mut *self.event_queue.borrow_mut())
    }

    fn has_events(&self) -> bool {
        !self.event_queue.borrow().is_empty()
    }

    fn device_hidpi_factor(&self) -> Scale<f32, DeviceIndependentPixel, DevicePixel> {
        Scale::new(self.winit_window.scale_factor() as f32)
    }

    fn device_pixel_ratio_override(
        &self,
    ) -> Option<Scale<f32, DeviceIndependentPixel, DevicePixel>> {
        self.device_pixel_ratio_override.map(Scale::new)
    }

    fn page_height(&self) -> f32 {
        let dpr = self.hidpi_factor();
        let size = self.winit_window.inner_size();
        size.height as f32 * dpr.get()
    }

    fn set_title(&self, title: &str) {
        self.winit_window.set_title(title);
    }

    fn request_inner_size(&self, size: DeviceIntSize) -> Option<DeviceIntSize> {
        self.winit_window
            .request_inner_size::<PhysicalSize<i32>>(PhysicalSize::new(size.width, size.height))
            .and_then(|size| {
                Some(DeviceIntSize::new(
                    size.width.try_into().ok()?,
                    size.height.try_into().ok()?,
                ))
            })
    }

    fn set_position(&self, point: DeviceIntPoint) {
        self.winit_window
            .set_outer_position::<PhysicalPosition<i32>>(PhysicalPosition::new(point.x, point.y))
    }

    fn set_fullscreen(&self, state: bool) {
        if self.fullscreen.get() != state {
            self.winit_window.set_fullscreen(if state {
                Some(winit::window::Fullscreen::Borderless(Some(
                    self.primary_monitor.clone(),
                )))
            } else {
                None
            });
        }
        self.fullscreen.set(state);
    }

    fn get_fullscreen(&self) -> bool {
        self.fullscreen.get()
    }

    fn set_cursor(&self, cursor: Cursor) {
        use winit::window::CursorIcon;

        let winit_cursor = match cursor {
            Cursor::Default => CursorIcon::Default,
            Cursor::Pointer => CursorIcon::Pointer,
            Cursor::ContextMenu => CursorIcon::ContextMenu,
            Cursor::Help => CursorIcon::Help,
            Cursor::Progress => CursorIcon::Progress,
            Cursor::Wait => CursorIcon::Wait,
            Cursor::Cell => CursorIcon::Cell,
            Cursor::Crosshair => CursorIcon::Crosshair,
            Cursor::Text => CursorIcon::Text,
            Cursor::VerticalText => CursorIcon::VerticalText,
            Cursor::Alias => CursorIcon::Alias,
            Cursor::Copy => CursorIcon::Copy,
            Cursor::Move => CursorIcon::Move,
            Cursor::NoDrop => CursorIcon::NoDrop,
            Cursor::NotAllowed => CursorIcon::NotAllowed,
            Cursor::Grab => CursorIcon::Grab,
            Cursor::Grabbing => CursorIcon::Grabbing,
            Cursor::EResize => CursorIcon::EResize,
            Cursor::NResize => CursorIcon::NResize,
            Cursor::NeResize => CursorIcon::NeResize,
            Cursor::NwResize => CursorIcon::NwResize,
            Cursor::SResize => CursorIcon::SResize,
            Cursor::SeResize => CursorIcon::SeResize,
            Cursor::SwResize => CursorIcon::SwResize,
            Cursor::WResize => CursorIcon::WResize,
            Cursor::EwResize => CursorIcon::EwResize,
            Cursor::NsResize => CursorIcon::NsResize,
            Cursor::NeswResize => CursorIcon::NeswResize,
            Cursor::NwseResize => CursorIcon::NwseResize,
            Cursor::ColResize => CursorIcon::ColResize,
            Cursor::RowResize => CursorIcon::RowResize,
            Cursor::AllScroll => CursorIcon::AllScroll,
            Cursor::ZoomIn => CursorIcon::ZoomIn,
            Cursor::ZoomOut => CursorIcon::ZoomOut,
            _ => CursorIcon::Default,
        };
        self.winit_window.set_cursor_icon(winit_cursor);
    }

    fn is_animating(&self) -> bool {
        self.animation_state.get() == AnimationState::Animating
    }

    fn id(&self) -> winit::window::WindowId {
        self.winit_window.id()
    }

    fn queue_embedder_events_for_winit_event(&self, event: winit::event::WindowEvent) {
        match event {
            winit::event::WindowEvent::KeyboardInput { event, .. } => {
                self.handle_keyboard_input(event)
            },
            winit::event::WindowEvent::ModifiersChanged(modifiers) => {
                self.modifiers_state.set(modifiers.state())
            },
            winit::event::WindowEvent::MouseInput { state, button, .. } => {
                if button == MouseButton::Left || button == MouseButton::Right {
                    self.handle_mouse(button, state, self.mouse_pos.get());
                }
            },
            winit::event::WindowEvent::CursorMoved { position, .. } => {
                let toolbar_height = self.toolbar_height.get() * self.hidpi_factor();
                let mut position = winit_position_to_euclid_point(position).to_f32();
                position -= Size2D::from_lengths(Length::zero(), toolbar_height);
                self.mouse_pos.set(position.to_i32());
                self.event_queue
                    .borrow_mut()
                    .push(EmbedderEvent::MouseWindowMoveEventClass(position.to_f32()));
            },
            winit::event::WindowEvent::MouseWheel { delta, phase, .. } => {
                let (mut dx, mut dy, mode) = match delta {
                    MouseScrollDelta::LineDelta(dx, dy) => {
                        (dx as f64, (dy * LINE_HEIGHT) as f64, WheelMode::DeltaLine)
                    },
                    MouseScrollDelta::PixelDelta(position) => {
                        let position: LogicalPosition<f64> =
                            position.to_logical(self.device_hidpi_factor().get() as f64);
                        (position.x, position.y, WheelMode::DeltaPixel)
                    },
                };

                // Create wheel event before snapping to the major axis of movement
                let wheel_delta = WheelDelta {
                    x: dx,
                    y: dy,
                    z: 0.0,
                    mode,
                };
                let pos = self.mouse_pos.get();
                let position = Point2D::new(pos.x as f32, pos.y as f32);
                let wheel_event = EmbedderEvent::Wheel(wheel_delta, position);

                // Scroll events snap to the major axis of movement, with vertical
                // preferred over horizontal.
                if dy.abs() >= dx.abs() {
                    dx = 0.0;
                } else {
                    dy = 0.0;
                }

                let scroll_location = ScrollLocation::Delta(Vector2D::new(dx as f32, dy as f32));
                let phase = winit_phase_to_touch_event_type(phase);
                let scroll_event =
                    EmbedderEvent::Scroll(scroll_location, self.mouse_pos.get(), phase);

                // Send events
                self.event_queue.borrow_mut().push(wheel_event);
                self.event_queue.borrow_mut().push(scroll_event);
            },
            winit::event::WindowEvent::Touch(touch) => {
                use servo::script_traits::TouchId;

                let phase = winit_phase_to_touch_event_type(touch.phase);
                let id = TouchId(touch.id as i32);
                let position = touch.location;
                let point = Point2D::new(position.x as f32, position.y as f32);
                self.event_queue
                    .borrow_mut()
                    .push(EmbedderEvent::Touch(phase, id, point));
            },
            winit::event::WindowEvent::TouchpadMagnify { delta, .. } => {
                let magnification = delta as f32 + 1.0;
                self.event_queue
                    .borrow_mut()
                    .push(EmbedderEvent::PinchZoom(magnification));
            },
            winit::event::WindowEvent::CloseRequested => {
                self.event_queue.borrow_mut().push(EmbedderEvent::Quit);
            },
            winit::event::WindowEvent::Resized(physical_size) => {
                let (width, height) = physical_size.into();
                let new_size = Size2D::new(width, height);
                if self.inner_size.get() != new_size {
                    let physical_size = Size2D::new(physical_size.width, physical_size.height);
                    self.rendering_context
                        .resize(physical_size.to_i32())
                        .expect("Failed to resize");
                    self.inner_size.set(new_size);
                    self.event_queue.borrow_mut().push(EmbedderEvent::Resize);
                }
            },
            _ => {},
        }
    }

    fn new_glwindow(
        &self,
        event_loop: &winit::event_loop::EventLoopWindowTarget<WakerEvent>,
    ) -> Box<dyn webxr::glwindow::GlWindow> {
        let size = self.winit_window.outer_size();

        let window_builder = winit::window::WindowBuilder::new()
            .with_title("Servo XR".to_string())
            .with_inner_size(size)
            .with_visible(true);

        let winit_window = window_builder
            .build(event_loop)
            .expect("Failed to create window.");

        let pose = Rc::new(XRWindowPose {
            xr_rotation: Cell::new(Rotation3D::identity()),
            xr_translation: Cell::new(Vector3D::zero()),
        });
        self.xr_window_poses.borrow_mut().push(pose.clone());
        Box::new(XRWindow { winit_window, pose })
    }

    fn winit_window(&self) -> Option<&winit::window::Window> {
        Some(&self.winit_window)
    }

    fn set_toolbar_height(&self, height: Length<f32, DeviceIndependentPixel>) {
        self.toolbar_height.set(height);
    }
}

impl WindowMethods for Window {
    fn get_coordinates(&self) -> EmbedderCoordinates {
        let window_size = winit_size_to_euclid_size(self.winit_window.outer_size()).to_i32();
        let window_origin = self.winit_window.outer_position().unwrap_or_default();
        let window_origin = winit_position_to_euclid_point(window_origin).to_i32();
        let inner_size = winit_size_to_euclid_size(self.winit_window.inner_size()).to_f32();

        // Subtract the minibrowser toolbar height if any
        let toolbar_height = self.toolbar_height.get() * self.hidpi_factor();
        let viewport_size = inner_size - Size2D::from_lengths(Length::zero(), toolbar_height);

        let viewport_origin = DeviceIntPoint::zero(); // bottom left
        let viewport = DeviceIntRect::from_origin_and_size(viewport_origin, viewport_size.to_i32());
        let screen = self.screen_size.to_i32();

        EmbedderCoordinates {
            viewport,
            framebuffer: viewport.size(),
            window: (window_size, window_origin),
            screen,
            // FIXME: Winit doesn't have API for available size. Fallback to screen size
            screen_avail: screen,
            hidpi_factor: self.hidpi_factor(),
        }
    }

    fn set_animation_state(&self, state: AnimationState) {
        self.animation_state.set(state);
    }

    fn rendering_context(&self) -> RenderingContext {
        self.rendering_context.clone()
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
    winit_window: winit::window::Window,
    pose: Rc<XRWindowPose>,
}

struct XRWindowPose {
    xr_rotation: Cell<Rotation3D<f32, UnknownUnit, UnknownUnit>>,
    xr_translation: Cell<Vector3D<f32, UnknownUnit>>,
}

impl webxr::glwindow::GlWindow for XRWindow {
    fn get_render_target(
        &self,
        device: &mut Device,
        _context: &mut Context,
    ) -> webxr::glwindow::GlWindowRenderTarget {
        let window_handle = self.winit_window.raw_window_handle();
        let size = self.winit_window.inner_size();
        let size = Size2D::new(size.width as i32, size.height as i32);
        let native_widget = device
            .connection()
            .create_native_widget_from_raw_window_handle(window_handle, size)
            .expect("Failed to create native widget");
        webxr::glwindow::GlWindowRenderTarget::NativeWidget(native_widget)
    }

    fn get_rotation(&self) -> Rotation3D<f32, UnknownUnit, UnknownUnit> {
        self.pose.xr_rotation.get()
    }

    fn get_translation(&self) -> Vector3D<f32, UnknownUnit> {
        self.pose.xr_translation.get()
    }

    fn get_mode(&self) -> webxr::glwindow::GlWindowMode {
        if pref!(dom.webxr.glwindow.red_cyan) {
            webxr::glwindow::GlWindowMode::StereoRedCyan
        } else if pref!(dom.webxr.glwindow.left_right) {
            webxr::glwindow::GlWindowMode::StereoLeftRight
        } else if pref!(dom.webxr.glwindow.spherical) {
            webxr::glwindow::GlWindowMode::Spherical
        } else if pref!(dom.webxr.glwindow.cubemap) {
            webxr::glwindow::GlWindowMode::Cubemap
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

    fn handle_xr_rotation(&self, input: &KeyEvent, modifiers: ModifiersState) {
        if input.state != winit::event::ElementState::Pressed {
            return;
        }
        let mut x = 0.0;
        let mut y = 0.0;
        match input.logical_key {
            LogicalKey::Named(NamedKey::ArrowUp) => x = 1.0,
            LogicalKey::Named(NamedKey::ArrowDown) => x = -1.0,
            LogicalKey::Named(NamedKey::ArrowLeft) => y = 1.0,
            LogicalKey::Named(NamedKey::ArrowRight) => y = -1.0,
            _ => return,
        };
        if modifiers.shift_key() {
            x *= 10.0;
            y *= 10.0;
        }
        let x: Rotation3D<_, UnknownUnit, UnknownUnit> = Rotation3D::around_x(Angle::degrees(x));
        let y: Rotation3D<_, UnknownUnit, UnknownUnit> = Rotation3D::around_y(Angle::degrees(y));
        let rotation = self.xr_rotation.get().then(&x).then(&y);
        self.xr_rotation.set(rotation);
    }
}

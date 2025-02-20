/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! A winit window implementation.

use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::env;
use std::rc::Rc;
use std::time::Duration;

use euclid::{Angle, Length, Point2D, Rotation3D, Scale, Size2D, UnknownUnit, Vector2D, Vector3D};
use keyboard_types::{Modifiers, ShortcutMatcher};
use log::{debug, info};
use raw_window_handle::{HasDisplayHandle, HasWindowHandle};
use servo::compositing::windowing::{
    AnimationState, EmbedderCoordinates, WebRenderDebugOption, WindowMethods,
};
use servo::config::opts::Opts;
use servo::servo_config::pref;
use servo::servo_geometry::DeviceIndependentPixel;
use servo::webrender_api::units::{DeviceIntPoint, DeviceIntRect, DeviceIntSize, DevicePixel};
use servo::webrender_api::ScrollLocation;
use servo::{
    Cursor, ImeEvent, InputEvent, Key, KeyState, KeyboardEvent, MouseButton as ServoMouseButton,
    MouseButtonAction, MouseButtonEvent, MouseMoveEvent, OffscreenRenderingContext,
    RenderingContext, Theme, TouchAction, TouchEvent, TouchEventType, TouchId, WebView, WheelDelta,
    WheelEvent, WheelMode, WindowRenderingContext,
};
use surfman::{Context, Device};
use url::Url;
use winit::dpi::{LogicalSize, PhysicalPosition, PhysicalSize};
use winit::event::{
    ElementState, Ime, KeyEvent, MouseButton, MouseScrollDelta, TouchPhase, WindowEvent,
};
use winit::event_loop::ActiveEventLoop;
use winit::keyboard::{Key as LogicalKey, ModifiersState, NamedKey};
#[cfg(any(target_os = "linux", target_os = "windows"))]
use winit::window::Icon;

use super::app_state::RunningAppState;
use super::geometry::{winit_position_to_euclid_point, winit_size_to_euclid_size};
use super::keyutils::{keyboard_event_from_winit, CMD_OR_ALT};
use super::window_trait::{WindowPortsMethods, LINE_HEIGHT};
use crate::desktop::accelerated_gl_media::setup_gl_accelerated_media;
use crate::desktop::keyutils::CMD_OR_CONTROL;
use crate::prefs::ServoShellPreferences;

pub struct Window {
    winit_window: winit::window::Window,
    screen_size: Size2D<u32, DeviceIndependentPixel>,
    inner_size: Cell<PhysicalSize<u32>>,
    toolbar_height: Cell<Length<f32, DeviceIndependentPixel>>,
    mouse_down_button: Cell<Option<MouseButton>>,
    webview_relative_mouse_down_point: Cell<Point2D<f32, DevicePixel>>,
    monitor: winit::monitor::MonitorHandle,
    webview_relative_mouse_point: Cell<Point2D<f32, DevicePixel>>,
    last_pressed: Cell<Option<(KeyboardEvent, Option<LogicalKey>)>>,
    /// A map of winit's key codes to key values that are interpreted from
    /// winit's ReceivedChar events.
    keys_down: RefCell<HashMap<LogicalKey, Key>>,
    animation_state: Cell<AnimationState>,
    fullscreen: Cell<bool>,
    device_pixel_ratio_override: Option<f32>,
    xr_window_poses: RefCell<Vec<Rc<XRWindowPose>>>,
    modifiers_state: Cell<ModifiersState>,

    /// The RenderingContext that renders directly onto the Window. This is used as
    /// the target of egui rendering and also where Servo rendering results are finally
    /// blitted.
    window_rendering_context: Rc<WindowRenderingContext>,

    /// The `RenderingContext` of Servo itself. This is used to render Servo results
    /// temporarily until they can be blitted into the egui scene.
    rendering_context: Rc<OffscreenRenderingContext>,
}

impl Window {
    pub fn new(
        opts: &Opts,
        servoshell_preferences: &ServoShellPreferences,
        event_loop: &ActiveEventLoop,
    ) -> Window {
        // If there's no chrome, start off with the window invisible. It will be set to visible in
        // `load_end()`. This avoids an ugly flash of unstyled content (especially important since
        // unstyled content is white and chrome often has a transparent background). See issue
        // #9996.
        let no_native_titlebar = servoshell_preferences.no_native_titlebar;
        let visible = opts.output_file.is_none() && !servoshell_preferences.no_native_titlebar;

        let window_size = servoshell_preferences.initial_window_size;
        let window_attr = winit::window::Window::default_attributes()
            .with_title("Servo".to_string())
            .with_decorations(!no_native_titlebar)
            .with_transparent(no_native_titlebar)
            .with_inner_size(LogicalSize::new(window_size.width, window_size.height))
            .with_visible(visible);

        #[allow(deprecated)]
        let winit_window = event_loop
            .create_window(window_attr)
            .expect("Failed to create window.");

        #[cfg(any(target_os = "linux", target_os = "windows"))]
        {
            let icon_bytes = include_bytes!("../../../resources/servo_64.png");
            winit_window.set_window_icon(Some(load_icon(icon_bytes)));
        }

        let monitor = winit_window
            .current_monitor()
            .or_else(|| winit_window.available_monitors().nth(0))
            .expect("No monitor detected");

        let (screen_size, screen_scale) = servoshell_preferences.screen_size_override.map_or_else(
            || (monitor.size(), monitor.scale_factor()),
            |size| (PhysicalSize::new(size.width, size.height), 1.0),
        );
        let screen_scale: Scale<f64, DeviceIndependentPixel, DevicePixel> =
            Scale::new(screen_scale);
        let screen_size = (winit_size_to_euclid_size(screen_size).to_f64() / screen_scale).to_u32();
        let inner_size = winit_window.inner_size();

        let display_handle = event_loop
            .display_handle()
            .expect("could not get display handle from window");
        let window_handle = winit_window
            .window_handle()
            .expect("could not get window handle from window");
        let window_rendering_context = Rc::new(
            WindowRenderingContext::new(display_handle, window_handle, &inner_size)
                .expect("Could not create RenderingContext for Window"),
        );

        // Setup for GL accelerated media handling. This is only active on certain Linux platforms
        // and Windows.
        {
            let details = window_rendering_context.surfman_details();
            setup_gl_accelerated_media(details.0, details.1);
        }

        // Make sure the gl context is made current.
        window_rendering_context.make_current().unwrap();

        let rendering_context_size = Size2D::new(inner_size.width, inner_size.height);
        let rendering_context =
            Rc::new(window_rendering_context.offscreen_context(rendering_context_size));

        debug!("Created window {:?}", winit_window.id());
        Window {
            winit_window,
            mouse_down_button: Cell::new(None),
            webview_relative_mouse_down_point: Cell::new(Point2D::zero()),
            webview_relative_mouse_point: Cell::new(Point2D::zero()),
            last_pressed: Cell::new(None),
            keys_down: RefCell::new(HashMap::new()),
            animation_state: Cell::new(AnimationState::Idle),
            fullscreen: Cell::new(false),
            inner_size: Cell::new(inner_size),
            monitor,
            screen_size,
            device_pixel_ratio_override: servoshell_preferences.device_pixel_ratio_override,
            xr_window_poses: RefCell::new(vec![]),
            modifiers_state: Cell::new(ModifiersState::empty()),
            toolbar_height: Cell::new(Default::default()),
            window_rendering_context,
            rendering_context,
        }
    }

    fn handle_received_character(&self, webview: &WebView, mut character: char) {
        info!("winit received character: {:?}", character);
        if character.is_control() {
            if character as u8 >= 32 {
                return;
            }
            // shift ASCII control characters to lowercase
            character = (character as u8 + 96) as char;
        }
        let (mut event, key_code) = if let Some((event, key_code)) = self.last_pressed.replace(None)
        {
            (event, key_code)
        } else if character.is_ascii() {
            // Some keys like Backspace emit a control character in winit
            // but they are already dealt with in handle_keyboard_input
            // so just ignore the character.
            return;
        } else {
            // For combined characters like the letter e with an acute accent
            // no keyboard event is emitted. A dummy event is created in this case.
            (KeyboardEvent::default(), None)
        };
        event.key = Key::Character(character.to_string());

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
        webview.notify_input_event(InputEvent::Keyboard(event));
    }

    fn handle_keyboard_input(&self, state: Rc<RunningAppState>, winit_event: KeyEvent) {
        // First, handle servoshell key bindings that are not overridable by, or visible to, the page.
        let mut keyboard_event =
            keyboard_event_from_winit(&winit_event, self.modifiers_state.get());
        if self.handle_intercepted_key_bindings(state.clone(), &keyboard_event) {
            return;
        }

        // Then we deliver character and keyboard events to the page in the focused webview.
        let Some(webview) = state.focused_webview() else {
            return;
        };

        if let Some(input_text) = &winit_event.text {
            for character in input_text.chars() {
                self.handle_received_character(&webview, character);
            }
        }

        if keyboard_event.state == KeyState::Down && keyboard_event.key == Key::Unidentified {
            // If pressed and probably printable, we expect a ReceivedCharacter event.
            // Wait for that to be received and don't queue any event right now.
            self.last_pressed
                .set(Some((keyboard_event, Some(winit_event.logical_key))));
            return;
        } else if keyboard_event.state == KeyState::Up && keyboard_event.key == Key::Unidentified {
            // If release and probably printable, this is following a ReceiverCharacter event.
            if let Some(key) = self.keys_down.borrow_mut().remove(&winit_event.logical_key) {
                keyboard_event.key = key;
            }
        }

        if keyboard_event.key != Key::Unidentified {
            self.last_pressed.set(None);
            let xr_poses = self.xr_window_poses.borrow();
            for xr_window_pose in &*xr_poses {
                xr_window_pose.handle_xr_rotation(&winit_event, self.modifiers_state.get());
            }
            webview.notify_input_event(InputEvent::Keyboard(keyboard_event));
        }

        // servoshell also has key bindings that are visible to, and overridable by, the page.
        // See the handler for EmbedderMsg::Keyboard in webview.rs for those.
    }

    /// Helper function to handle a click
    fn handle_mouse(&self, webview: &WebView, button: MouseButton, action: ElementState) {
        let max_pixel_dist = 10.0 * self.hidpi_factor().get();
        let mouse_button = match &button {
            MouseButton::Left => ServoMouseButton::Left,
            MouseButton::Right => ServoMouseButton::Right,
            MouseButton::Middle => ServoMouseButton::Middle,
            MouseButton::Back => ServoMouseButton::Back,
            MouseButton::Forward => ServoMouseButton::Forward,
            MouseButton::Other(value) => ServoMouseButton::Other(*value),
        };

        let point = self.webview_relative_mouse_point.get();
        let action = match action {
            ElementState::Pressed => {
                self.webview_relative_mouse_down_point.set(point);
                self.mouse_down_button.set(Some(button));
                MouseButtonAction::Down
            },
            ElementState::Released => MouseButtonAction::Up,
        };

        webview.notify_input_event(InputEvent::MouseButton(MouseButtonEvent {
            action,
            button: mouse_button,
            point,
        }));

        // Also send a 'click' event if this is release and the press was recorded
        // to be within a 10 pixels.
        //
        // TODO: This should be happening within the ScriptThread.
        if action != MouseButtonAction::Up {
            return;
        }

        if let Some(mouse_down_button) = self.mouse_down_button.get() {
            let pixel_dist = self.webview_relative_mouse_down_point.get() - point;
            let pixel_dist = (pixel_dist.x * pixel_dist.x + pixel_dist.y * pixel_dist.y).sqrt();
            if mouse_down_button == button && pixel_dist < max_pixel_dist {
                webview.notify_input_event(InputEvent::MouseButton(MouseButtonEvent {
                    action: MouseButtonAction::Click,
                    button: mouse_button,
                    point,
                }));
            }
        }
    }

    /// Handle key events before sending them to Servo.
    fn handle_intercepted_key_bindings(
        &self,
        state: Rc<RunningAppState>,
        key_event: &KeyboardEvent,
    ) -> bool {
        let Some(focused_webview) = state.focused_webview() else {
            return false;
        };

        let mut handled = true;
        ShortcutMatcher::from_event(key_event.clone())
            .shortcut(CMD_OR_CONTROL, 'R', || focused_webview.reload())
            .shortcut(CMD_OR_CONTROL, 'W', || {
                state.close_webview(focused_webview.id());
            })
            .shortcut(CMD_OR_CONTROL, 'P', || {
                let rate = env::var("SAMPLING_RATE")
                    .ok()
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(10);
                let duration = env::var("SAMPLING_DURATION")
                    .ok()
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(10);
                focused_webview.toggle_sampling_profiler(
                    Duration::from_millis(rate),
                    Duration::from_secs(duration),
                );
            })
            .shortcut(CMD_OR_CONTROL, 'X', || {
                focused_webview
                    .notify_input_event(InputEvent::EditingAction(servo::EditingActionEvent::Cut))
            })
            .shortcut(CMD_OR_CONTROL, 'C', || {
                focused_webview
                    .notify_input_event(InputEvent::EditingAction(servo::EditingActionEvent::Copy))
            })
            .shortcut(CMD_OR_CONTROL, 'V', || {
                focused_webview
                    .notify_input_event(InputEvent::EditingAction(servo::EditingActionEvent::Paste))
            })
            .shortcut(Modifiers::CONTROL, Key::F9, || {
                focused_webview.capture_webrender();
            })
            .shortcut(Modifiers::CONTROL, Key::F10, || {
                focused_webview.toggle_webrender_debugging(WebRenderDebugOption::RenderTargetDebug);
            })
            .shortcut(Modifiers::CONTROL, Key::F11, || {
                focused_webview.toggle_webrender_debugging(WebRenderDebugOption::TextureCacheDebug);
            })
            .shortcut(Modifiers::CONTROL, Key::F12, || {
                focused_webview.toggle_webrender_debugging(WebRenderDebugOption::Profiler);
            })
            .shortcut(CMD_OR_ALT, Key::ArrowRight, || {
                focused_webview.go_forward(1);
            })
            .optional_shortcut(
                cfg!(not(target_os = "windows")),
                CMD_OR_CONTROL,
                ']',
                || {
                    focused_webview.go_forward(1);
                },
            )
            .shortcut(CMD_OR_ALT, Key::ArrowLeft, || {
                focused_webview.go_back(1);
            })
            .optional_shortcut(
                cfg!(not(target_os = "windows")),
                CMD_OR_CONTROL,
                '[',
                || {
                    focused_webview.go_back(1);
                },
            )
            .optional_shortcut(
                self.get_fullscreen(),
                Modifiers::empty(),
                Key::Escape,
                || focused_webview.exit_fullscreen(),
            )
            // Select the first 8 tabs via shortcuts
            .shortcut(CMD_OR_CONTROL, '1', || state.focus_webview_by_index(0))
            .shortcut(CMD_OR_CONTROL, '2', || state.focus_webview_by_index(1))
            .shortcut(CMD_OR_CONTROL, '3', || state.focus_webview_by_index(2))
            .shortcut(CMD_OR_CONTROL, '4', || state.focus_webview_by_index(3))
            .shortcut(CMD_OR_CONTROL, '5', || state.focus_webview_by_index(4))
            .shortcut(CMD_OR_CONTROL, '6', || state.focus_webview_by_index(5))
            .shortcut(CMD_OR_CONTROL, '7', || state.focus_webview_by_index(6))
            .shortcut(CMD_OR_CONTROL, '8', || state.focus_webview_by_index(7))
            // Cmd/Ctrl 9 is a bit different in that it focuses the last tab instead of the 9th
            .shortcut(CMD_OR_CONTROL, '9', || {
                let len = state.webviews().len();
                if len > 0 {
                    state.focus_webview_by_index(len - 1)
                }
            })
            .shortcut(Modifiers::CONTROL, Key::PageDown, || {
                if let Some(index) = state.get_focused_webview_index() {
                    state.focus_webview_by_index((index + 1) % state.webviews().len())
                }
            })
            .shortcut(Modifiers::CONTROL, Key::PageUp, || {
                if let Some(index) = state.get_focused_webview_index() {
                    let new_index = if index == 0 {
                        state.webviews().len() - 1
                    } else {
                        index - 1
                    };
                    state.focus_webview_by_index(new_index)
                }
            })
            .shortcut(CMD_OR_CONTROL, 'T', || {
                state.new_toplevel_webview(Url::parse("servo:newtab").unwrap());
            })
            .shortcut(CMD_OR_CONTROL, 'Q', || state.servo().start_shutting_down())
            .otherwise(|| handled = false);
        handled
    }

    pub(crate) fn offscreen_rendering_context(&self) -> Rc<OffscreenRenderingContext> {
        self.rendering_context.clone()
    }
}

impl WindowPortsMethods for Window {
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

    fn request_resize(&self, _: &WebView, size: DeviceIntSize) -> Option<DeviceIntSize> {
        let toolbar_height = self.toolbar_height() * self.hidpi_factor();
        let toolbar_height = toolbar_height.get().ceil() as i32;
        let total_size = PhysicalSize::new(size.width, size.height + toolbar_height);
        self.winit_window
            .request_inner_size::<PhysicalSize<i32>>(PhysicalSize::new(
                total_size.width,
                total_size.height,
            ))
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
                    self.monitor.clone(),
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
            Cursor::None => {
                self.winit_window.set_cursor_visible(false);
                return;
            },
        };
        self.winit_window.set_cursor(winit_cursor);
        self.winit_window.set_cursor_visible(true);
    }

    fn is_animating(&self) -> bool {
        self.animation_state.get() == AnimationState::Animating
    }

    fn id(&self) -> winit::window::WindowId {
        self.winit_window.id()
    }

    fn handle_winit_event(&self, state: Rc<RunningAppState>, event: WindowEvent) {
        let Some(webview) = state.focused_webview() else {
            return;
        };

        match event {
            WindowEvent::KeyboardInput { event, .. } => self.handle_keyboard_input(state, event),
            WindowEvent::ModifiersChanged(modifiers) => self.modifiers_state.set(modifiers.state()),
            WindowEvent::MouseInput { state, button, .. } => {
                if button == MouseButton::Left || button == MouseButton::Right {
                    self.handle_mouse(&webview, button, state);
                }
            },
            WindowEvent::CursorMoved { position, .. } => {
                let mut point = winit_position_to_euclid_point(position).to_f32();
                point.y -= (self.toolbar_height() * self.hidpi_factor()).0;

                self.webview_relative_mouse_point.set(point);
                webview.notify_input_event(InputEvent::MouseMove(MouseMoveEvent { point }));
            },
            WindowEvent::MouseWheel { delta, phase, .. } => {
                let (mut dx, mut dy, mode) = match delta {
                    MouseScrollDelta::LineDelta(dx, dy) => {
                        (dx as f64, (dy * LINE_HEIGHT) as f64, WheelMode::DeltaLine)
                    },
                    MouseScrollDelta::PixelDelta(position) => {
                        let scale_factor = self.device_hidpi_factor().inverse().get() as f64;
                        let position = position.to_logical(scale_factor);
                        (position.x, position.y, WheelMode::DeltaPixel)
                    },
                };

                // Create wheel event before snapping to the major axis of movement
                let delta = WheelDelta {
                    x: dx,
                    y: dy,
                    z: 0.0,
                    mode,
                };
                let pos = self.webview_relative_mouse_point.get();
                let point = Point2D::new(pos.x, pos.y);

                // Scroll events snap to the major axis of movement, with vertical
                // preferred over horizontal.
                if dy.abs() >= dx.abs() {
                    dx = 0.0;
                } else {
                    dy = 0.0;
                }

                let scroll_location = ScrollLocation::Delta(Vector2D::new(dx as f32, dy as f32));
                let phase = winit_phase_to_touch_event_type(phase);

                // Send events
                webview.notify_input_event(InputEvent::Wheel(WheelEvent { delta, point }));
                webview.notify_scroll_event(
                    scroll_location,
                    self.webview_relative_mouse_point.get().to_i32(),
                    phase,
                );
            },
            WindowEvent::Touch(touch) => {
                webview.notify_input_event(InputEvent::Touch(TouchEvent {
                    event_type: winit_phase_to_touch_event_type(touch.phase),
                    id: TouchId(touch.id as i32),
                    point: Point2D::new(touch.location.x as f32, touch.location.y as f32),
                    action: TouchAction::NoAction,
                }));
            },
            WindowEvent::PinchGesture { delta, .. } => {
                webview.set_pinch_zoom(delta as f32 + 1.0);
            },
            WindowEvent::CloseRequested => {
                state.servo().start_shutting_down();
            },
            WindowEvent::Resized(new_size) => {
                if self.inner_size.get() != new_size {
                    let rendering_context_size = Size2D::new(new_size.width, new_size.height);
                    self.window_rendering_context
                        .resize(rendering_context_size.to_i32());
                    self.inner_size.set(new_size);
                    webview.notify_rendering_context_resized();
                }
            },
            WindowEvent::ThemeChanged(theme) => {
                webview.notify_theme_change(match theme {
                    winit::window::Theme::Light => Theme::Light,
                    winit::window::Theme::Dark => Theme::Dark,
                });
            },
            WindowEvent::Moved(_new_position) => {
                webview.notify_embedder_window_moved();
            },
            WindowEvent::Ime(ime) => match ime {
                Ime::Enabled => {
                    webview.notify_input_event(InputEvent::Ime(ImeEvent::Composition(
                        servo::CompositionEvent {
                            state: servo::CompositionState::Start,
                            data: String::new(),
                        },
                    )));
                },
                Ime::Preedit(text, _) => {
                    webview.notify_input_event(InputEvent::Ime(ImeEvent::Composition(
                        servo::CompositionEvent {
                            state: servo::CompositionState::Update,
                            data: text,
                        },
                    )));
                },
                Ime::Commit(text) => {
                    webview.notify_input_event(InputEvent::Ime(ImeEvent::Composition(
                        servo::CompositionEvent {
                            state: servo::CompositionState::End,
                            data: text,
                        },
                    )));
                },
                Ime::Disabled => {
                    webview.notify_input_event(InputEvent::Ime(ImeEvent::Dismissed));
                },
            },
            _ => {},
        }
    }

    fn new_glwindow(
        &self,
        event_loop: &ActiveEventLoop,
    ) -> Rc<dyn servo::webxr::glwindow::GlWindow> {
        let size = self.winit_window.outer_size();

        let window_attr = winit::window::Window::default_attributes()
            .with_title("Servo XR".to_string())
            .with_inner_size(size)
            .with_visible(false);

        let winit_window = event_loop
            .create_window(window_attr)
            .expect("Failed to create window.");

        let pose = Rc::new(XRWindowPose {
            xr_rotation: Cell::new(Rotation3D::identity()),
            xr_translation: Cell::new(Vector3D::zero()),
        });
        self.xr_window_poses.borrow_mut().push(pose.clone());
        Rc::new(XRWindow { winit_window, pose })
    }

    fn winit_window(&self) -> Option<&winit::window::Window> {
        Some(&self.winit_window)
    }

    fn toolbar_height(&self) -> Length<f32, DeviceIndependentPixel> {
        self.toolbar_height.get()
    }

    fn set_toolbar_height(&self, height: Length<f32, DeviceIndependentPixel>) {
        self.toolbar_height.set(height);
    }

    fn rendering_context(&self) -> Rc<dyn RenderingContext> {
        self.rendering_context.clone()
    }

    fn show_ime(
        &self,
        _input_type: servo::InputMethodType,
        _text: Option<(String, i32)>,
        _multiline: bool,
        _position: servo::webrender_api::units::DeviceIntRect,
    ) {
        self.winit_window.set_ime_allowed(true);
    }

    fn hide_ime(&self) {
        self.winit_window.set_ime_allowed(false);
    }
}

impl WindowMethods for Window {
    fn get_coordinates(&self) -> EmbedderCoordinates {
        let window_size = winit_size_to_euclid_size(self.winit_window.outer_size()).to_i32();
        let window_origin = self.winit_window.outer_position().unwrap_or_default();
        let window_origin = winit_position_to_euclid_point(window_origin).to_i32();
        let window_rect = DeviceIntRect::from_origin_and_size(window_origin, window_size);
        let window_scale: Scale<f64, DeviceIndependentPixel, DevicePixel> =
            Scale::new(self.winit_window.scale_factor());
        let window_rect = (window_rect.to_f64() / window_scale).to_i32();

        let viewport_origin = DeviceIntPoint::zero(); // bottom left
        let mut viewport_size = winit_size_to_euclid_size(self.winit_window.inner_size()).to_f32();
        viewport_size.height -= (self.toolbar_height() * self.hidpi_factor()).0;

        let viewport = DeviceIntRect::from_origin_and_size(viewport_origin, viewport_size.to_i32());
        let screen_size = self.screen_size.to_i32();

        EmbedderCoordinates {
            viewport,
            framebuffer: viewport.size(),
            window_rect,
            screen_size,
            // FIXME: Winit doesn't have API for available size. Fallback to screen size
            available_screen_size: screen_size,
            hidpi_factor: self.hidpi_factor(),
        }
    }

    fn set_animation_state(&self, state: AnimationState) {
        self.animation_state.set(state);
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

impl servo::webxr::glwindow::GlWindow for XRWindow {
    fn get_render_target(
        &self,
        device: &mut Device,
        _context: &mut Context,
    ) -> servo::webxr::glwindow::GlWindowRenderTarget {
        self.winit_window.set_visible(true);
        let window_handle = self
            .winit_window
            .window_handle()
            .expect("could not get window handle from window");
        let size = self.winit_window.inner_size();
        let size = Size2D::new(size.width as i32, size.height as i32);
        let native_widget = device
            .connection()
            .create_native_widget_from_window_handle(window_handle, size)
            .expect("Failed to create native widget");
        servo::webxr::glwindow::GlWindowRenderTarget::NativeWidget(native_widget)
    }

    fn get_rotation(&self) -> Rotation3D<f32, UnknownUnit, UnknownUnit> {
        self.pose.xr_rotation.get()
    }

    fn get_translation(&self) -> Vector3D<f32, UnknownUnit> {
        self.pose.xr_translation.get()
    }

    fn get_mode(&self) -> servo::webxr::glwindow::GlWindowMode {
        if pref!(dom_webxr_glwindow_red_cyan) {
            servo::webxr::glwindow::GlWindowMode::StereoRedCyan
        } else if pref!(dom_webxr_glwindow_left_right) {
            servo::webxr::glwindow::GlWindowMode::StereoLeftRight
        } else if pref!(dom_webxr_glwindow_spherical) {
            servo::webxr::glwindow::GlWindowMode::Spherical
        } else if pref!(dom_webxr_glwindow_cubemap) {
            servo::webxr::glwindow::GlWindowMode::Cubemap
        } else {
            servo::webxr::glwindow::GlWindowMode::Blit
        }
    }

    fn display_handle(&self) -> raw_window_handle::DisplayHandle {
        self.winit_window.display_handle().unwrap()
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
        if input.state != ElementState::Pressed {
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

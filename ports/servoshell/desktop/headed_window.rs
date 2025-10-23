/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! A winit window implementation.

#![deny(clippy::panic)]
#![deny(clippy::unwrap_used)]

use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::env;
use std::rc::Rc;
use std::time::Duration;

use euclid::{Angle, Length, Point2D, Rotation3D, Scale, Size2D, UnknownUnit, Vector2D, Vector3D};
use keyboard_types::ShortcutMatcher;
use log::{debug, info};
use raw_window_handle::{HasDisplayHandle, HasWindowHandle, RawWindowHandle};
use servo::servo_geometry::{
    DeviceIndependentIntRect, DeviceIndependentPixel, convert_rect_to_css_pixel,
};
use servo::webrender_api::ScrollLocation;
use servo::webrender_api::units::{DeviceIntPoint, DeviceIntRect, DeviceIntSize, DevicePixel};
use servo::{
    Cursor, ImeEvent, InputEvent, InputEventId, InputEventResult, Key, KeyState, KeyboardEvent,
    Modifiers, MouseButton as ServoMouseButton, MouseButtonAction, MouseButtonEvent,
    MouseLeftViewportEvent, MouseMoveEvent, NamedKey, OffscreenRenderingContext, RenderingContext,
    ScreenGeometry, Theme, TouchEvent, TouchEventType, TouchId, WebRenderDebugOption, WebView,
    WheelDelta, WheelEvent, WheelMode, WindowRenderingContext,
};
use url::Url;
use winit::dpi::{LogicalPosition, LogicalSize, PhysicalPosition, PhysicalSize};
use winit::event::{
    ElementState, Ime, KeyEvent, MouseButton, MouseScrollDelta, TouchPhase, WindowEvent,
};
use winit::event_loop::ActiveEventLoop;
use winit::keyboard::{Key as LogicalKey, ModifiersState, NamedKey as WinitNamedKey};
#[cfg(target_os = "linux")]
use winit::platform::wayland::WindowAttributesExtWayland;
#[cfg(any(target_os = "linux", target_os = "windows"))]
use winit::window::Icon;
#[cfg(target_os = "macos")]
use {
    objc2_app_kit::{NSColorSpace, NSView},
    objc2_foundation::MainThreadMarker,
};

use super::app_state::RunningAppState;
use super::geometry::{winit_position_to_euclid_point, winit_size_to_euclid_size};
use super::keyutils::{CMD_OR_ALT, keyboard_event_from_winit};
use super::window_trait::{LINE_HEIGHT, LINE_WIDTH, PIXEL_DELTA_FACTOR, WindowPortsMethods};
use crate::desktop::accelerated_gl_media::setup_gl_accelerated_media;
use crate::desktop::keyutils::CMD_OR_CONTROL;
use crate::desktop::window_trait::MIN_WINDOW_INNER_SIZE;
use crate::prefs::ServoShellPreferences;

pub(crate) const INITIAL_WINDOW_TITLE: &str = "Servo";

pub struct Window {
    screen_size: Size2D<u32, DeviceIndependentPixel>,
    toolbar_height: Cell<Length<f32, DeviceIndependentPixel>>,
    monitor: winit::monitor::MonitorHandle,
    webview_relative_mouse_point: Cell<Point2D<f32, DevicePixel>>,
    last_pressed: Cell<Option<(KeyboardEvent, Option<LogicalKey>)>>,
    /// The inner size of the window in physical pixels which excludes OS decorations.
    /// It equals viewport size + (0, toolbar height).
    inner_size: Cell<PhysicalSize<u32>>,
    /// A map of winit's key codes to key values that are interpreted from
    /// winit's ReceivedChar events.
    keys_down: RefCell<HashMap<LogicalKey, Key>>,
    fullscreen: Cell<bool>,
    device_pixel_ratio_override: Option<f32>,
    xr_window_poses: RefCell<Vec<Rc<XRWindowPose>>>,
    modifiers_state: Cell<ModifiersState>,

    /// The `RenderingContext` of Servo itself. This is used to render Servo results
    /// temporarily until they can be blitted into the egui scene.
    rendering_context: Rc<OffscreenRenderingContext>,
    /// The RenderingContext that renders directly onto the Window. This is used as
    /// the target of egui rendering and also where Servo rendering results are finally
    /// blitted.
    window_rendering_context: Rc<WindowRenderingContext>,
    /// A helper that simulates touch events when the `--simulate-touch-events` flag
    /// is enabled.
    touch_event_simulator: Option<TouchEventSimulator>,
    /// Keyboard events that have been sent to Servo that have still not been handled yet.
    /// When these are handled, they will optionally be used to trigger keybindings that
    /// are overridable by web content.
    pending_keyboard_events: RefCell<HashMap<InputEventId, KeyboardEvent>>,

    // Keep this as the last field of the struct to ensure that the rendering context is
    // dropped first.
    // (https://github.com/servo/servo/issues/36711)
    winit_window: winit::window::Window,

    /// The last title set on this window. We need to store this value here, as `winit::Window::title`
    /// is not supported very many platforms.
    last_title: RefCell<String>,
}

impl Window {
    pub fn new(
        servoshell_preferences: &ServoShellPreferences,
        event_loop: &ActiveEventLoop,
    ) -> Window {
        let no_native_titlebar = servoshell_preferences.no_native_titlebar;
        let inner_size = servoshell_preferences.initial_window_size;
        let window_attr = winit::window::Window::default_attributes()
            .with_title(INITIAL_WINDOW_TITLE.to_string())
            .with_decorations(!no_native_titlebar)
            .with_transparent(no_native_titlebar)
            .with_inner_size(LogicalSize::new(inner_size.width, inner_size.height))
            .with_min_inner_size(LogicalSize::new(
                MIN_WINDOW_INNER_SIZE.width,
                MIN_WINDOW_INNER_SIZE.height,
            ))
            // Must be invisible at startup; accesskit_winit setup needs to
            // happen before the window is shown for the first time.
            .with_visible(false);

        // Set a name so it can be pinned to taskbars in Linux.
        #[cfg(target_os = "linux")]
        let window_attr = window_attr.with_name("org.servo.Servo", "Servo");

        #[allow(deprecated)]
        let winit_window = event_loop
            .create_window(window_attr)
            .expect("Failed to create window.");

        #[cfg(any(target_os = "linux", target_os = "windows"))]
        {
            let icon_bytes = include_bytes!("../../../resources/servo_64.png");
            winit_window.set_window_icon(Some(load_icon(icon_bytes)));
        }

        let window_handle = winit_window
            .window_handle()
            .expect("winit window did not have a window handle");
        Window::force_srgb_color_space(window_handle.as_raw());

        let monitor = winit_window
            .current_monitor()
            .or_else(|| winit_window.available_monitors().nth(0))
            .expect("No monitor detected");

        let (screen_size, screen_scale) = servoshell_preferences.screen_size_override.map_or_else(
            || (monitor.size(), winit_window.scale_factor()),
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
            WindowRenderingContext::new(display_handle, window_handle, inner_size)
                .expect("Could not create RenderingContext for Window"),
        );

        // Setup for GL accelerated media handling. This is only active on certain Linux platforms
        // and Windows.
        {
            let details = window_rendering_context.surfman_details();
            setup_gl_accelerated_media(details.0, details.1);
        }

        // Make sure the gl context is made current.
        window_rendering_context
            .make_current()
            .expect("Could not make window RenderingContext current");

        let rendering_context = Rc::new(window_rendering_context.offscreen_context(inner_size));

        debug!("Created window {:?}", winit_window.id());
        Window {
            winit_window,
            webview_relative_mouse_point: Cell::new(Point2D::zero()),
            last_pressed: Cell::new(None),
            keys_down: RefCell::new(HashMap::new()),
            fullscreen: Cell::new(false),
            inner_size: Cell::new(inner_size),
            monitor,
            screen_size,
            device_pixel_ratio_override: servoshell_preferences.device_pixel_ratio_override,
            xr_window_poses: RefCell::new(vec![]),
            modifiers_state: Cell::new(ModifiersState::empty()),
            toolbar_height: Cell::new(Default::default()),
            window_rendering_context,
            touch_event_simulator: servoshell_preferences
                .simulate_touch_events
                .then(Default::default),
            pending_keyboard_events: Default::default(),
            rendering_context,
            last_title: RefCell::new(String::from(INITIAL_WINDOW_TITLE)),
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
        event.event.key = Key::Character(character.to_string());

        if event.event.state == KeyState::Down {
            // Ensure that when we receive a keyup event from winit, we are able
            // to infer that it's related to this character and set the event
            // properties appropriately.
            if let Some(key_code) = key_code {
                self.keys_down
                    .borrow_mut()
                    .insert(key_code, event.event.key.clone());
            }
        }

        let xr_poses = self.xr_window_poses.borrow();
        for xr_window_pose in &*xr_poses {
            xr_window_pose.handle_xr_translation(&event);
        }

        let id = webview.notify_input_event(InputEvent::Keyboard(event.clone()));
        self.pending_keyboard_events.borrow_mut().insert(id, event);
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

        if keyboard_event.event.state == KeyState::Down &&
            keyboard_event.event.key == Key::Named(NamedKey::Unidentified)
        {
            // If pressed and probably printable, we expect a ReceivedCharacter event.
            // Wait for that to be received and don't queue any event right now.
            self.last_pressed
                .set(Some((keyboard_event, Some(winit_event.logical_key))));
            return;
        } else if keyboard_event.event.state == KeyState::Up &&
            keyboard_event.event.key == Key::Named(NamedKey::Unidentified)
        {
            // If release and probably printable, this is following a ReceiverCharacter event.
            if let Some(key) = self.keys_down.borrow_mut().remove(&winit_event.logical_key) {
                keyboard_event.event.key = key;
            }
        }

        if keyboard_event.event.key != Key::Named(NamedKey::Unidentified) {
            self.last_pressed.set(None);
            let xr_poses = self.xr_window_poses.borrow();
            for xr_window_pose in &*xr_poses {
                xr_window_pose.handle_xr_rotation(&winit_event, self.modifiers_state.get());
            }

            let id = webview.notify_input_event(InputEvent::Keyboard(keyboard_event.clone()));
            self.pending_keyboard_events
                .borrow_mut()
                .insert(id, keyboard_event);
        }

        // servoshell also has key bindings that are visible to, and overridable by, the page.
        // See the handler for EmbedderMsg::Keyboard in webview.rs for those.
    }

    /// Helper function to handle a click
    fn handle_mouse_button_event(
        &self,
        webview: &WebView,
        button: MouseButton,
        action: ElementState,
    ) {
        // `point` can be outside viewport, such as at toolbar with negative y-coordinate.
        let point = self.webview_relative_mouse_point.get();
        if !webview.rect().contains(point) {
            return;
        }

        if self
            .touch_event_simulator
            .as_ref()
            .is_some_and(|touch_event_simulator| {
                touch_event_simulator
                    .maybe_consume_move_button_event(webview, button, action, point)
            })
        {
            return;
        }

        let mouse_button = match &button {
            MouseButton::Left => ServoMouseButton::Left,
            MouseButton::Right => ServoMouseButton::Right,
            MouseButton::Middle => ServoMouseButton::Middle,
            MouseButton::Back => ServoMouseButton::Back,
            MouseButton::Forward => ServoMouseButton::Forward,
            MouseButton::Other(value) => ServoMouseButton::Other(*value),
        };

        let action = match action {
            ElementState::Pressed => MouseButtonAction::Down,
            ElementState::Released => MouseButtonAction::Up,
        };

        webview.notify_input_event(InputEvent::MouseButton(MouseButtonEvent::new(
            action,
            mouse_button,
            point,
        )));
    }

    /// Helper function to handle mouse move events.
    fn handle_mouse_move_event(&self, webview: &WebView, position: PhysicalPosition<f64>) {
        let mut point = winit_position_to_euclid_point(position).to_f32();
        point.y -= (self.toolbar_height() * self.hidpi_scale_factor()).0;

        let previous_point = self.webview_relative_mouse_point.get();
        self.webview_relative_mouse_point.set(point);

        if !webview.rect().contains(point) {
            if webview.rect().contains(previous_point) {
                webview.notify_input_event(InputEvent::MouseLeftViewport(
                    MouseLeftViewportEvent::default(),
                ));
            }
            return;
        }

        if self
            .touch_event_simulator
            .as_ref()
            .is_some_and(|touch_event_simulator| {
                touch_event_simulator.maybe_consume_mouse_move_event(webview, point)
            })
        {
            return;
        }

        webview.notify_input_event(InputEvent::MouseMove(MouseMoveEvent::new(point)));
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
        ShortcutMatcher::from_event(key_event.event.clone())
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
                    .notify_input_event(InputEvent::EditingAction(servo::EditingActionEvent::Cut));
            })
            .shortcut(CMD_OR_CONTROL, 'C', || {
                focused_webview
                    .notify_input_event(InputEvent::EditingAction(servo::EditingActionEvent::Copy));
            })
            .shortcut(CMD_OR_CONTROL, 'V', || {
                focused_webview.notify_input_event(InputEvent::EditingAction(
                    servo::EditingActionEvent::Paste,
                ));
            })
            .shortcut(Modifiers::CONTROL, Key::Named(NamedKey::F9), || {
                focused_webview.capture_webrender();
            })
            .shortcut(Modifiers::CONTROL, Key::Named(NamedKey::F10), || {
                focused_webview.toggle_webrender_debugging(WebRenderDebugOption::RenderTargetDebug);
            })
            .shortcut(Modifiers::CONTROL, Key::Named(NamedKey::F11), || {
                focused_webview.toggle_webrender_debugging(WebRenderDebugOption::TextureCacheDebug);
            })
            .shortcut(Modifiers::CONTROL, Key::Named(NamedKey::F12), || {
                focused_webview.toggle_webrender_debugging(WebRenderDebugOption::Profiler);
            })
            .shortcut(CMD_OR_ALT, Key::Named(NamedKey::ArrowRight), || {
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
            .shortcut(CMD_OR_ALT, Key::Named(NamedKey::ArrowLeft), || {
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
                Key::Named(NamedKey::Escape),
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
            .shortcut(Modifiers::CONTROL, Key::Named(NamedKey::PageDown), || {
                if let Some(index) = state.get_focused_webview_index() {
                    state.focus_webview_by_index((index + 1) % state.webviews().len())
                }
            })
            .shortcut(Modifiers::CONTROL, Key::Named(NamedKey::PageUp), || {
                if let Some(index) = state.get_focused_webview_index() {
                    let len = state.webviews().len();
                    state.focus_webview_by_index((index + len - 1) % len);
                }
            })
            .shortcut(CMD_OR_CONTROL, 'T', || {
                state.create_and_focus_toplevel_webview(
                    Url::parse("servo:newtab")
                        .expect("Should be able to unconditionally parse 'servo:newtab' as URL"),
                );
            })
            .shortcut(CMD_OR_CONTROL, 'Q', || state.servo().start_shutting_down())
            .otherwise(|| handled = false);
        handled
    }

    pub(crate) fn offscreen_rendering_context(&self) -> Rc<OffscreenRenderingContext> {
        self.rendering_context.clone()
    }

    #[allow(unused_variables)]
    fn force_srgb_color_space(window_handle: RawWindowHandle) {
        #[cfg(target_os = "macos")]
        {
            if let RawWindowHandle::AppKit(handle) = window_handle {
                assert!(MainThreadMarker::new().is_some());
                unsafe {
                    let view = handle.ns_view.cast::<NSView>().as_ref();
                    view.window()
                        .expect("Should have a window")
                        .setColorSpace(Some(&NSColorSpace::sRGBColorSpace()));
                }
            }
        }
    }
}

impl WindowPortsMethods for Window {
    fn screen_geometry(&self) -> ScreenGeometry {
        let hidpi_factor = self.hidpi_scale_factor();
        let toolbar_size = Size2D::new(
            0.0,
            (self.toolbar_height.get() * self.hidpi_scale_factor()).0,
        );
        let screen_size = self.screen_size.to_f32() * hidpi_factor;

        // FIXME: In reality, this should subtract screen space used by the system interface
        // elements, but it is difficult to get this value with `winit` currently. See:
        // See https://github.com/rust-windowing/winit/issues/2494
        let available_screen_size = screen_size - toolbar_size;

        let window_rect = DeviceIntRect::from_origin_and_size(
            winit_position_to_euclid_point(self.winit_window.outer_position().unwrap_or_default()),
            winit_size_to_euclid_size(self.winit_window.outer_size()).to_i32(),
        );

        ScreenGeometry {
            size: screen_size.to_i32(),
            available_size: available_screen_size.to_i32(),
            window_rect,
        }
    }

    fn device_hidpi_scale_factor(&self) -> Scale<f32, DeviceIndependentPixel, DevicePixel> {
        Scale::new(self.winit_window.scale_factor() as f32)
    }

    fn hidpi_scale_factor(&self) -> Scale<f32, DeviceIndependentPixel, DevicePixel> {
        self.device_pixel_ratio_override
            .map(Scale::new)
            .unwrap_or_else(|| self.device_hidpi_scale_factor())
    }

    fn set_title(&self, title: &str) {
        self.winit_window.set_title(title);
    }

    fn set_title_if_changed(&self, title: &str) -> bool {
        let mut last = self.last_title.borrow_mut();
        if *last == title {
            return false;
        }

        self.winit_window.set_title(title);
        *last = title.to_owned();
        true
    }

    fn request_resize(&self, _: &WebView, new_outer_size: DeviceIntSize) -> Option<DeviceIntSize> {
        // Allocate space for the window deocrations, but do not let the inner size get
        // smaller than `MIN_WINDOW_INNER_SIZE` or larger than twice the screen size.
        let inner_size = self.winit_window.inner_size();
        let outer_size = self.winit_window.outer_size();
        let decoration_size: DeviceIntSize = Size2D::new(
            outer_size.height - inner_size.height,
            outer_size.width - inner_size.width,
        )
        .cast();

        let screen_size = (self.screen_size.to_f32() * self.hidpi_scale_factor()).to_i32();
        let new_outer_size =
            new_outer_size.clamp(MIN_WINDOW_INNER_SIZE + decoration_size, screen_size * 2);

        if outer_size.width == new_outer_size.width as u32 &&
            outer_size.height == new_outer_size.height as u32
        {
            return Some(new_outer_size);
        }

        let new_inner_size = new_outer_size - decoration_size;
        self.winit_window
            .request_inner_size(PhysicalSize::new(
                new_inner_size.width,
                new_inner_size.height,
            ))
            .map(|resulting_size| {
                DeviceIntSize::new(
                    resulting_size.width as i32 + decoration_size.width,
                    resulting_size.height as i32 + decoration_size.height,
                )
            })
    }

    fn window_rect(&self) -> DeviceIndependentIntRect {
        let outer_size = self.winit_window.outer_size();
        let scale = self.hidpi_scale_factor();

        let outer_size = winit_size_to_euclid_size(outer_size).to_i32();

        let origin = self
            .winit_window
            .outer_position()
            .map(winit_position_to_euclid_point)
            .unwrap_or_default();
        convert_rect_to_css_pixel(
            DeviceIntRect::from_origin_and_size(origin, outer_size),
            scale,
        )
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

    fn id(&self) -> winit::window::WindowId {
        self.winit_window.id()
    }

    fn handle_winit_event(&self, state: Rc<RunningAppState>, event: WindowEvent) {
        // Make sure to handle early resize events even when there are no webviews yet
        if let WindowEvent::Resized(new_inner_size) = event {
            if self.inner_size.get() != new_inner_size {
                self.inner_size.set(new_inner_size);
                // This should always be set to inner size
                // because we are resizing `SurfmanRenderingContext`.
                // See https://github.com/servo/servo/issues/38369#issuecomment-3138378527
                self.window_rendering_context.resize(new_inner_size);
            }
            return;
        }

        let Some(webview) = state.focused_webview() else {
            return;
        };

        match event {
            WindowEvent::KeyboardInput { event, .. } => self.handle_keyboard_input(state, event),
            WindowEvent::ModifiersChanged(modifiers) => self.modifiers_state.set(modifiers.state()),
            WindowEvent::MouseInput { state, button, .. } => {
                self.handle_mouse_button_event(&webview, button, state);
            },
            WindowEvent::CursorMoved { position, .. } => {
                self.handle_mouse_move_event(&webview, position);
            },
            WindowEvent::CursorLeft { .. } => {
                if webview
                    .rect()
                    .contains(self.webview_relative_mouse_point.get())
                {
                    webview.notify_input_event(InputEvent::MouseLeftViewport(
                        MouseLeftViewportEvent::default(),
                    ));
                }
            },
            WindowEvent::MouseWheel { delta, .. } => {
                let (mut dx, mut dy, mode) = match delta {
                    MouseScrollDelta::LineDelta(dx, dy) => (
                        (dx * LINE_WIDTH) as f64,
                        (dy * LINE_HEIGHT) as f64,
                        WheelMode::DeltaLine,
                    ),
                    MouseScrollDelta::PixelDelta(position) => {
                        let position: LogicalPosition<f64> =
                            position.to_logical(self.device_hidpi_scale_factor().get() as f64);
                        (
                            position.x * PIXEL_DELTA_FACTOR,
                            position.y * PIXEL_DELTA_FACTOR,
                            WheelMode::DeltaPixel,
                        )
                    },
                };

                // Create wheel event before snapping to the major axis of movement
                let delta = WheelDelta {
                    x: dx,
                    y: dy,
                    z: 0.0,
                    mode,
                };
                let point = self.webview_relative_mouse_point.get();

                // Scroll events snap to the major axis of movement, with vertical
                // preferred over horizontal.
                if dy.abs() >= dx.abs() {
                    dx = 0.0;
                } else {
                    dy = 0.0;
                }

                // Send events
                webview.notify_input_event(InputEvent::Wheel(WheelEvent::new(delta, point)));
                let scroll_location = ScrollLocation::Delta(-Vector2D::new(dx as f32, dy as f32));
                webview.notify_scroll_event(scroll_location, point.to_i32());
            },
            WindowEvent::Touch(touch) => {
                webview.notify_input_event(InputEvent::Touch(TouchEvent::new(
                    winit_phase_to_touch_event_type(touch.phase),
                    TouchId(touch.id as i32),
                    Point2D::new(touch.location.x as f32, touch.location.y as f32),
                )));
            },
            WindowEvent::PinchGesture { delta, .. } => {
                webview.pinch_zoom(delta as f32 + 1.0, self.webview_relative_mouse_point.get());
            },
            WindowEvent::CloseRequested => {
                state.servo().start_shutting_down();
            },
            WindowEvent::ThemeChanged(theme) => {
                webview.notify_theme_change(match theme {
                    winit::window::Theme::Light => Theme::Light,
                    winit::window::Theme::Dark => Theme::Dark,
                });
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

    #[cfg(feature = "webxr")]
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
        if self.toolbar_height() == height {
            return;
        }
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
        position: servo::webrender_api::units::DeviceIntRect,
    ) {
        self.winit_window.set_ime_allowed(true);
        self.winit_window.set_ime_cursor_area(
            LogicalPosition::new(
                position.min.x,
                position.min.y + (self.toolbar_height.get().0 as i32),
            ),
            LogicalSize::new(
                position.max.x - position.min.x,
                position.max.y - position.min.y,
            ),
        );
    }

    fn hide_ime(&self) {
        self.winit_window.set_ime_allowed(false);
    }

    fn theme(&self) -> servo::Theme {
        match self.winit_window.theme() {
            Some(winit::window::Theme::Dark) => servo::Theme::Dark,
            Some(winit::window::Theme::Light) | None => servo::Theme::Light,
        }
    }

    fn maximize(&self, _webview: &WebView) {
        self.winit_window.set_maximized(true);
    }

    /// Handle servoshell key bindings that may have been prevented by the page in the focused webview.
    fn notify_input_event_handled(
        &self,
        webview: &WebView,
        id: InputEventId,
        result: InputEventResult,
    ) {
        let Some(keyboard_event) = self.pending_keyboard_events.borrow_mut().remove(&id) else {
            return;
        };
        if result.intersects(InputEventResult::DefaultPrevented | InputEventResult::Consumed) {
            return;
        }

        ShortcutMatcher::from_event(keyboard_event.event)
            .shortcut(CMD_OR_CONTROL, '=', || {
                webview.set_page_zoom(webview.page_zoom() + 0.1);
            })
            .shortcut(CMD_OR_CONTROL, '+', || {
                webview.set_page_zoom(webview.page_zoom() + 0.1);
            })
            .shortcut(CMD_OR_CONTROL, '-', || {
                webview.set_page_zoom(webview.page_zoom() - 0.1);
            })
            .shortcut(CMD_OR_CONTROL, '0', || {
                webview.set_page_zoom(1.0);
            });
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

#[cfg(feature = "webxr")]
struct XRWindow {
    winit_window: winit::window::Window,
    pose: Rc<XRWindowPose>,
}

struct XRWindowPose {
    xr_rotation: Cell<Rotation3D<f32, UnknownUnit, UnknownUnit>>,
    xr_translation: Cell<Vector3D<f32, UnknownUnit>>,
}

#[cfg(feature = "webxr")]
impl servo::webxr::glwindow::GlWindow for XRWindow {
    fn get_render_target(
        &self,
        device: &mut surfman::Device,
        _context: &mut surfman::Context,
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
        use servo::servo_config::pref;
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

    fn display_handle(&self) -> raw_window_handle::DisplayHandle<'_> {
        self.winit_window
            .display_handle()
            .expect("Every window should have a display handle")
    }
}

impl XRWindowPose {
    fn handle_xr_translation(&self, input: &KeyboardEvent) {
        if input.event.state != KeyState::Down {
            return;
        }
        const NORMAL_TRANSLATE: f32 = 0.1;
        const QUICK_TRANSLATE: f32 = 1.0;
        let mut x = 0.0;
        let mut z = 0.0;
        match input.event.key {
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
            LogicalKey::Named(WinitNamedKey::ArrowUp) => x = 1.0,
            LogicalKey::Named(WinitNamedKey::ArrowDown) => x = -1.0,
            LogicalKey::Named(WinitNamedKey::ArrowLeft) => y = 1.0,
            LogicalKey::Named(WinitNamedKey::ArrowRight) => y = -1.0,
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

#[derive(Default)]
pub struct TouchEventSimulator {
    pub left_mouse_button_down: Cell<bool>,
}

impl TouchEventSimulator {
    fn maybe_consume_move_button_event(
        &self,
        webview: &WebView,
        button: MouseButton,
        action: ElementState,
        point: Point2D<f32, DevicePixel>,
    ) -> bool {
        if button != MouseButton::Left {
            return false;
        }

        if action == ElementState::Pressed && !self.left_mouse_button_down.get() {
            webview.notify_input_event(InputEvent::Touch(TouchEvent::new(
                TouchEventType::Down,
                TouchId(0),
                point,
            )));
            self.left_mouse_button_down.set(true);
        } else if action == ElementState::Released {
            webview.notify_input_event(InputEvent::Touch(TouchEvent::new(
                TouchEventType::Up,
                TouchId(0),
                point,
            )));
            self.left_mouse_button_down.set(false);
        }

        true
    }

    fn maybe_consume_mouse_move_event(
        &self,
        webview: &WebView,
        point: Point2D<f32, DevicePixel>,
    ) -> bool {
        if !self.left_mouse_button_down.get() {
            return false;
        }

        webview.notify_input_event(InputEvent::Touch(TouchEvent::new(
            TouchEventType::Move,
            TouchId(0),
            point,
        )));
        true
    }
}

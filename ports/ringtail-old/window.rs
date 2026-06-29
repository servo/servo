/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
 
use std::cell::Cell;
use std::rc::Rc;
use std::str::FromStr;

use servo::{
    Code, DeviceIntPoint, DeviceIntRect, DeviceIntSize, DevicePoint, InputEvent, Key, KeyState,
    KeyboardEvent, Location, Modifiers, MouseButton as ServoMouseButton, MouseButtonAction,
    MouseButtonEvent, MouseMoveEvent, NavigationRequest, NamedKey, RenderingContext, WebView,
    WebViewPoint, WheelDelta, WheelEvent, WheelMode, WindowRenderingContext,
};
use url::Url;
use winit::event::{ElementState, KeyEvent, MouseButton, MouseScrollDelta, WindowEvent};
use winit::event_loop::ActiveEventLoop;
use winit::keyboard::{Key as WinitKey, KeyLocation as WinitKeyLocation, ModifiersState};
use winit::raw_window_handle::{HasDisplayHandle, HasWindowHandle};  
use winit::window::{CursorIcon, Window, WindowAttributes};

pub mod resource_protocol;

pub struct RingtailWindow {
    pub winit_window: Rc<Window>,
    webview: Option<WebView>,
    rendering_context: Rc<WindowRenderingContext>,
   
    last_mouse_point: Cell<DevicePoint>,
   
    modifiers_state: Cell<ModifiersState>,
}

impl RingtailWindow {
    pub fn new(active_event_loop: &ActiveEventLoop, _url: Url) -> Self {
        let window_attributes = WindowAttributes::default()
            .with_title("Ringtail")
            .with_inner_size(winit::dpi::PhysicalSize::new(800, 600));

        let winit_window = Rc::new(
            active_event_loop
                .create_window(window_attributes)
                .unwrap(),
        );

        let display_handle = winit_window.display_handle().unwrap();
        let window_handle = winit_window.window_handle().unwrap();
        let inner_size = winit_window.inner_size();

        let rendering_context = Rc::new(
            WindowRenderingContext::new(display_handle, window_handle, inner_size)
                .expect("Could not create RenderingContext"),
        );

        Self {
            winit_window,
            webview: None,
            modifiers_state: Cell::new(ModifiersState::default()),
            rendering_context,
            last_mouse_point: Cell::new(DevicePoint::zero()),
        }
    }

    pub fn load_url(&mut self, url: Url, servo: &servo::Servo) {
        let webview = servo::WebViewBuilder::new(servo, self.rendering_context.clone())
            .url(url)
            .delegate(Rc::new(RingtailWebViewDelegate { window: self.winit_window.clone() }))
            .build();

        self.webview = Some(webview);
    }
    
    pub fn request_redraw(&self) {
        self.winit_window.request_redraw();
    }
    
    pub fn handle_winit_event(&self, event: &WindowEvent) {
        let Some(webview) = &self.webview else { 
            // Log that events are being dropped because the webview isn't loaded yet
            if let WindowEvent::Resized(_) = event {
                log::warn!("Dropped a Resized event because webview is None!");
            }
            return; 
        };

        match event {
            WindowEvent::Resized(new_inner_size) => {
                // 1. Make the context current on this thread
                let _ = self.rendering_context.make_current();
                
                // 2. Resize the underlying hardware surface swapchain
                self.rendering_context.resize(*new_inner_size);
                
                // 3. Inform the layout engine of the size change
                webview.resize(*new_inner_size);

                // 4. Force an immediate layout composition and frame present
                webview.paint();
                self.rendering_context.present();
            }
            WindowEvent::CursorMoved { position, .. } => {
                let point = DevicePoint::new(position.x as f32, position.y as f32);
                self.last_mouse_point.set(point);
                
                webview.notify_input_event(InputEvent::MouseMove(MouseMoveEvent::new(
                    point.into(),
                )));
            }
            WindowEvent::MouseInput { state, button, .. } => {
                let point = self.last_mouse_point.get();
                let webview_size = webview.size();
                let webview_rect = DeviceIntRect::from_size(DeviceIntSize::new(
                    webview_size.width as i32,
                    webview_size.height as i32,
                ));
                if !webview_rect.contains(point.to_i32()) {
                    return;
                }

                let servo_button = match button {
                    MouseButton::Left => ServoMouseButton::Left,
                    MouseButton::Right => ServoMouseButton::Right,
                    MouseButton::Middle => ServoMouseButton::Middle,
                    MouseButton::Back => ServoMouseButton::Back,
                    MouseButton::Forward => ServoMouseButton::Forward,
                    MouseButton::Other(value) => ServoMouseButton::Other(*value),
                };

                let action = match state {
                    ElementState::Pressed => MouseButtonAction::Down,
                    ElementState::Released => MouseButtonAction::Up,
                };

                webview.notify_input_event(InputEvent::MouseButton(MouseButtonEvent::new(
                    action,
                    servo_button,
                    point.into(),
                )));
            }
            WindowEvent::ModifiersChanged(state) => {
                self.modifiers_state.set(state.state());
            }
            WindowEvent::KeyboardInput { event: winit_key_event, .. } => {
                let servo_key_event = keyboard_event_from_winit(winit_key_event, self.modifiers_state.get());
                webview.notify_input_event(InputEvent::Keyboard(servo_key_event));
            }
            WindowEvent::MouseWheel { delta, .. } => {
                const LINE_WIDTH: f32 = 16.0;
                const LINE_HEIGHT: f32 = 38.0;

                let (delta_x, delta_y, mode) = match delta {
                    MouseScrollDelta::LineDelta(delta_x, delta_y) => (
                        (delta_x * LINE_WIDTH) as f64,
                        (delta_y * LINE_HEIGHT) as f64,
                        WheelMode::DeltaPixel,
                    ),
                    MouseScrollDelta::PixelDelta(delta) => {
                        (delta.x, delta.y, WheelMode::DeltaPixel)
                    },
                };

                let wheel_delta = WheelDelta {
                    x: delta_x,
                    y: delta_y,
                    z: 0.0,
                    mode,
                };
                let point: WebViewPoint = self.last_mouse_point.get().into();
                webview.notify_input_event(InputEvent::Wheel(WheelEvent::new(wheel_delta, point)));
            }
            _ => {}
        }
    }

    pub fn paint(&self) {
        if let Some(webview) = &self.webview {
           
            self.rendering_context
                .make_current()
                .expect("Could not make RenderingContext current");

            webview.paint();

            self.rendering_context.present();
        }
    }
}

struct RingtailWebViewDelegate {
    window: Rc<Window>,
}

impl servo::WebViewDelegate for RingtailWebViewDelegate {
    fn notify_cursor_changed(&self, _webview: servo::WebView, cursor: servo::Cursor) {
        let winit_cursor = match cursor {
            servo::Cursor::Default => CursorIcon::Default,
            servo::Cursor::Pointer => CursorIcon::Pointer,
            servo::Cursor::ContextMenu => CursorIcon::ContextMenu,
            servo::Cursor::Help => CursorIcon::Help,
            servo::Cursor::Progress => CursorIcon::Progress,
            servo::Cursor::Wait => CursorIcon::Wait,
            servo::Cursor::Cell => CursorIcon::Cell,
            servo::Cursor::Crosshair => CursorIcon::Crosshair,
            servo::Cursor::Text => CursorIcon::Text,
            servo::Cursor::VerticalText => CursorIcon::VerticalText,
            servo::Cursor::Alias => CursorIcon::Alias,
            servo::Cursor::Copy => CursorIcon::Copy,
            servo::Cursor::Move => CursorIcon::Move,
            servo::Cursor::NoDrop => CursorIcon::NoDrop,
            servo::Cursor::NotAllowed => CursorIcon::NotAllowed,
            servo::Cursor::Grab => CursorIcon::Grab,
            servo::Cursor::Grabbing => CursorIcon::Grabbing,
            servo::Cursor::EResize => CursorIcon::EResize,
            servo::Cursor::NResize => CursorIcon::NResize,
            servo::Cursor::NeResize => CursorIcon::NeResize,
            servo::Cursor::NwResize => CursorIcon::NwResize,
            servo::Cursor::SResize => CursorIcon::SResize,
            servo::Cursor::SeResize => CursorIcon::SeResize,
            servo::Cursor::SwResize => CursorIcon::SwResize,
            servo::Cursor::WResize => CursorIcon::WResize,
            servo::Cursor::EwResize => CursorIcon::EwResize,
            servo::Cursor::NsResize => CursorIcon::NsResize,
            servo::Cursor::NeswResize => CursorIcon::NeswResize,
            servo::Cursor::NwseResize => CursorIcon::NwseResize,
            servo::Cursor::ColResize => CursorIcon::ColResize,
            servo::Cursor::RowResize => CursorIcon::RowResize,
            servo::Cursor::AllScroll => CursorIcon::AllScroll,
            servo::Cursor::ZoomIn => CursorIcon::ZoomIn,
            servo::Cursor::ZoomOut => CursorIcon::ZoomOut,
            servo::Cursor::None => {
                self.window.set_cursor_visible(false);
                return;
            },
        };
        self.window.set_cursor(winit_cursor);
        self.window.set_cursor_visible(true);
    }

    fn request_navigation(&self, _webview: servo::WebView, navigation_request: NavigationRequest) {
        // Allow navigation by default
        navigation_request.allow();
    }
}

fn keyboard_event_from_winit(key_event: &KeyEvent, state: ModifiersState) -> KeyboardEvent {
    let modifiers = modifiers_from_winit(state);
    KeyboardEvent::new_without_event(
        KeyState::from_winit_key_event(key_event),
        Key::from_winit_key_event(key_event),
        Code::from_winit_key_event(key_event),
        Location::from_winit_key_event(key_event),
        modifiers,
        false,
        false,
    )
}

fn modifiers_from_winit(state: ModifiersState) -> Modifiers {
    let mut modifiers = Modifiers::empty();
    
    if state.control_key() {
        modifiers |= Modifiers::CONTROL;
    }
    if state.shift_key() {
        modifiers |= Modifiers::SHIFT;
    }
    if state.alt_key() {
        modifiers |= Modifiers::ALT;
    }
    if state.super_key() {
        modifiers |= Modifiers::META;
    }
    
    modifiers
}

trait FromWinitKeyEvent {
    fn from_winit_key_event(key_event: &KeyEvent) -> Self;
}

impl FromWinitKeyEvent for KeyState {
    fn from_winit_key_event(key_event: &KeyEvent) -> Self {
        match key_event.state {
            winit::event::ElementState::Pressed => KeyState::Down,
            winit::event::ElementState::Released => KeyState::Up,
        }
    }
}

impl FromWinitKeyEvent for Key {
    fn from_winit_key_event(key_event: &KeyEvent) -> Self {
        let named_key = match key_event.logical_key {
            WinitKey::Named(named_key) => named_key,
            WinitKey::Character(ref string) => return Key::Character(string.to_string()),
            WinitKey::Unidentified(_) | WinitKey::Dead(_) => {
                return Key::Named(NamedKey::Unidentified);
            },
        };

        match named_key {
            winit::keyboard::NamedKey::Backspace => Key::Named(NamedKey::Backspace),
            winit::keyboard::NamedKey::Enter => Key::Named(NamedKey::Enter),
            winit::keyboard::NamedKey::Tab => Key::Named(NamedKey::Tab),
            winit::keyboard::NamedKey::Escape => Key::Named(NamedKey::Escape),
            winit::keyboard::NamedKey::F1 => Key::Named(NamedKey::F1),
            winit::keyboard::NamedKey::F2 => Key::Named(NamedKey::F2),
            winit::keyboard::NamedKey::F3 => Key::Named(NamedKey::F3),
            winit::keyboard::NamedKey::F4 => Key::Named(NamedKey::F4),
            winit::keyboard::NamedKey::F5 => Key::Named(NamedKey::F5),
            winit::keyboard::NamedKey::F6 => Key::Named(NamedKey::F6),
            winit::keyboard::NamedKey::F7 => Key::Named(NamedKey::F7),
            winit::keyboard::NamedKey::F8 => Key::Named(NamedKey::F8),
            winit::keyboard::NamedKey::F9 => Key::Named(NamedKey::F9),
            winit::keyboard::NamedKey::F10 => Key::Named(NamedKey::F10),
            winit::keyboard::NamedKey::F11 => Key::Named(NamedKey::F11),
            winit::keyboard::NamedKey::F12 => Key::Named(NamedKey::F12),
            winit::keyboard::NamedKey::ArrowUp => Key::Named(NamedKey::ArrowUp),
            winit::keyboard::NamedKey::ArrowDown => Key::Named(NamedKey::ArrowDown),
            winit::keyboard::NamedKey::ArrowLeft => Key::Named(NamedKey::ArrowLeft),
            winit::keyboard::NamedKey::ArrowRight => Key::Named(NamedKey::ArrowRight),
            winit::keyboard::NamedKey::Home => Key::Named(NamedKey::Home),
            winit::keyboard::NamedKey::End => Key::Named(NamedKey::End),
            winit::keyboard::NamedKey::PageUp => Key::Named(NamedKey::PageUp),
            winit::keyboard::NamedKey::PageDown => Key::Named(NamedKey::PageDown),
            winit::keyboard::NamedKey::Delete => Key::Named(NamedKey::Delete),
            winit::keyboard::NamedKey::Insert => Key::Named(NamedKey::Insert),
            _ => Key::Named(NamedKey::Unidentified),
        }
    }
}

impl FromWinitKeyEvent for Code {
    fn from_winit_key_event(key_event: &KeyEvent) -> Self {
        match key_event.physical_key {
            winit::keyboard::PhysicalKey::Code(code) => {
                Code::from_str(format!("{:?}", code).as_str()).unwrap_or(Code::Unidentified)
            },
            _ => Code::Unidentified,
        }
    }
}

impl FromWinitKeyEvent for Location {
    fn from_winit_key_event(key_event: &KeyEvent) -> Self {
        match key_event.location {
            WinitKeyLocation::Standard => Location::Standard,
            WinitKeyLocation::Left => Location::Left,
            WinitKeyLocation::Right => Location::Right,
            WinitKeyLocation::Numpad => Location::Numpad,
        }
    }
}

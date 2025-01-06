/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use std::rc::Rc;
use std::time::Duration;
use std::vec::Drain;
use std::{env, thread};

use arboard::Clipboard;
use euclid::{Point2D, Vector2D};
use gilrs::ff::{BaseEffect, BaseEffectType, Effect, EffectBuilder, Repeat, Replay, Ticks};
use gilrs::{EventType, Gilrs};
use keyboard_types::{Key, KeyboardEvent, Modifiers, ShortcutMatcher};
use log::{debug, error, info, trace, warn};
use servo::base::id::TopLevelBrowsingContextId as WebViewId;
use servo::compositing::windowing::{EmbedderEvent, WebRenderDebugOption};
use servo::embedder_traits::{
    CompositorEventVariant, ContextMenuResult, DualRumbleEffectParams, EmbedderMsg, FilterPattern,
    GamepadHapticEffectType, PermissionPrompt, PermissionRequest, PromptCredentialsInput,
    PromptDefinition, PromptOrigin, PromptResult,
};
use servo::ipc_channel::ipc::IpcSender;
use servo::script_traits::{
    GamepadEvent, GamepadIndex, GamepadInputBounds, GamepadSupportedHapticEffects,
    GamepadUpdateType, TouchEventType, TraversalDirection,
};
use servo::servo_config::opts;
use servo::servo_url::ServoUrl;
use servo::webrender_api::units::DeviceRect;
use servo::webrender_api::ScrollLocation;
use tinyfiledialogs::{self, MessageBoxIcon, OkCancel, YesNo};

use super::keyutils::{CMD_OR_ALT, CMD_OR_CONTROL};
use super::window_trait::{WindowPortsMethods, LINE_HEIGHT};
use crate::desktop::tracing::{trace_embedder_event, trace_embedder_msg};

pub struct WebViewManager<Window: WindowPortsMethods + ?Sized> {
    status_text: Option<String>,

    /// List of top-level browsing contexts.
    /// Modified by EmbedderMsg::WebViewOpened and EmbedderMsg::WebViewClosed,
    /// and we exit if it ever becomes empty.
    webviews: HashMap<WebViewId, WebView>,

    /// The order in which the webviews were created.
    creation_order: Vec<WebViewId>,

    /// The webview that is currently focused.
    /// Modified by EmbedderMsg::WebViewFocused and EmbedderMsg::WebViewBlurred.
    focused_webview_id: Option<WebViewId>,

    /// Pre-creation state for WebViews.
    /// This is needed because in some situations the WebViewOpened event is sent
    /// after ChangePageTitle and HistoryChanged
    webview_preload_data: HashMap<WebViewId, WebViewPreloadData>,

    window: Rc<Window>,
    event_queue: Vec<EmbedderEvent>,
    clipboard: Option<Clipboard>,
    gamepad: Option<Gilrs>,
    haptic_effects: HashMap<usize, HapticEffect>,
    shutdown_requested: bool,
}

#[derive(Clone, Default)]
struct WebViewPreloadData {
    title: Option<String>,
    url: Option<ServoUrl>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LoadStatus {
    HeadParsed,
    LoadStart,
    LoadComplete,
}

// The state of each Tab/WebView
#[derive(Debug)]
pub struct WebView {
    pub rect: DeviceRect,
    pub title: Option<String>,
    pub url: Option<ServoUrl>,
    pub focused: bool,
    pub load_status: LoadStatus,
}

impl WebView {
    fn new(rect: DeviceRect, preload_data: WebViewPreloadData) -> Self {
        Self {
            rect,
            title: preload_data.title,
            url: preload_data.url,
            focused: false,
            load_status: LoadStatus::LoadComplete,
        }
    }
}

pub struct ServoEventResponse {
    pub need_present: bool,
    pub need_update: bool,
}

pub struct HapticEffect {
    pub effect: Effect,
    pub sender: IpcSender<bool>,
}

impl<Window> WebViewManager<Window>
where
    Window: WindowPortsMethods + ?Sized,
{
    pub fn new(window: Rc<Window>) -> WebViewManager<Window> {
        WebViewManager {
            status_text: None,
            webviews: HashMap::default(),
            creation_order: vec![],
            focused_webview_id: None,
            webview_preload_data: HashMap::default(),
            window,
            clipboard: match Clipboard::new() {
                Ok(c) => Some(c),
                Err(e) => {
                    warn!("Error creating clipboard context ({})", e);
                    None
                },
            },
            gamepad: match Gilrs::new() {
                Ok(g) => Some(g),
                Err(e) => {
                    warn!("Error creating gamepad input connection ({})", e);
                    None
                },
            },
            haptic_effects: HashMap::default(),

            event_queue: Vec::new(),
            shutdown_requested: false,
        }
    }

    pub fn get_mut(&mut self, webview_id: WebViewId) -> Option<&mut WebView> {
        self.webviews.get_mut(&webview_id)
    }

    // Returns the existing preload data for the given WebView, or a new one.
    fn ensure_preload_data_mut(&mut self, webview_id: &WebViewId) -> &mut WebViewPreloadData {
        if let Entry::Vacant(entry) = self.webview_preload_data.entry(*webview_id) {
            entry.insert(WebViewPreloadData::default());
        }
        self.webview_preload_data.get_mut(webview_id).unwrap()
    }

    pub fn focused_webview_id(&self) -> Option<WebViewId> {
        self.focused_webview_id
    }

    #[cfg(feature = "egui")]
    pub fn current_url_string(&self) -> Option<String> {
        match self.focused_webview() {
            Some(webview) => webview.url.as_ref().map(|url| url.to_string()),
            None => None,
        }
    }

    pub fn focused_webview(&self) -> Option<&WebView> {
        match self.focused_webview_id {
            Some(id) => self.webviews.get(&id),
            None => None,
        }
    }

    pub fn load_status(&self) -> LoadStatus {
        match self.focused_webview() {
            Some(webview) => webview.load_status,
            None => LoadStatus::LoadComplete,
        }
    }

    #[cfg(feature = "egui")]
    pub fn status_text(&self) -> Option<String> {
        self.status_text.clone()
    }

    pub fn get_events(&mut self) -> Vec<EmbedderEvent> {
        std::mem::take(&mut self.event_queue)
    }

    // Returns the webviews in the creation order.
    pub fn webviews(&self) -> Vec<(WebViewId, &WebView)> {
        let mut res = vec![];
        for id in &self.creation_order {
            res.push((*id, self.webviews.get(id).unwrap()))
        }
        res
    }

    pub fn handle_window_events(&mut self, events: Vec<EmbedderEvent>) {
        for event in events {
            trace_embedder_event!(event, "{event:?}");
            match event {
                EmbedderEvent::Keyboard(key_event) => {
                    self.handle_key_from_window(key_event);
                },
                event => {
                    self.event_queue.push(event);
                },
            }
        }
    }

    /// Handle updates to connected gamepads from GilRs
    pub fn handle_gamepad_events(&mut self) {
        if let Some(ref mut gilrs) = self.gamepad {
            while let Some(event) = gilrs.next_event() {
                let gamepad = gilrs.gamepad(event.id);
                let name = gamepad.name();
                let index = GamepadIndex(event.id.into());
                let mut gamepad_event: Option<GamepadEvent> = None;
                match event.event {
                    EventType::ButtonPressed(button, _) => {
                        let mapped_index = Self::map_gamepad_button(button);
                        // We only want to send this for a valid digital button, aka on/off only
                        if !matches!(mapped_index, 6 | 7 | 17) {
                            let update_type = GamepadUpdateType::Button(mapped_index, 1.0);
                            gamepad_event = Some(GamepadEvent::Updated(index, update_type));
                        }
                    },
                    EventType::ButtonReleased(button, _) => {
                        let mapped_index = Self::map_gamepad_button(button);
                        // We only want to send this for a valid digital button, aka on/off only
                        if !matches!(mapped_index, 6 | 7 | 17) {
                            let update_type = GamepadUpdateType::Button(mapped_index, 0.0);
                            gamepad_event = Some(GamepadEvent::Updated(index, update_type));
                        }
                    },
                    EventType::ButtonChanged(button, value, _) => {
                        let mapped_index = Self::map_gamepad_button(button);
                        // We only want to send this for a valid non-digital button, aka the triggers
                        if matches!(mapped_index, 6 | 7) {
                            let update_type = GamepadUpdateType::Button(mapped_index, value as f64);
                            gamepad_event = Some(GamepadEvent::Updated(index, update_type));
                        }
                    },
                    EventType::AxisChanged(axis, value, _) => {
                        // Map axis index and value to represent Standard Gamepad axis
                        // <https://www.w3.org/TR/gamepad/#dfn-represents-a-standard-gamepad-axis>
                        let mapped_axis: usize = match axis {
                            gilrs::Axis::LeftStickX => 0,
                            gilrs::Axis::LeftStickY => 1,
                            gilrs::Axis::RightStickX => 2,
                            gilrs::Axis::RightStickY => 3,
                            _ => 4, // Other axes do not map to "standard" gamepad mapping and are ignored
                        };
                        if mapped_axis < 4 {
                            // The Gamepad spec designates down as positive and up as negative.
                            // GilRs does the inverse of this, so correct for it here.
                            let axis_value = match mapped_axis {
                                0 | 2 => value,
                                1 | 3 => -value,
                                _ => 0., // Should not reach here
                            };
                            let update_type =
                                GamepadUpdateType::Axis(mapped_axis, axis_value as f64);
                            gamepad_event = Some(GamepadEvent::Updated(index, update_type));
                        }
                    },
                    EventType::Connected => {
                        let name = String::from(name);
                        let bounds = GamepadInputBounds {
                            axis_bounds: (-1.0, 1.0),
                            button_bounds: (0.0, 1.0),
                        };
                        // GilRs does not yet support trigger rumble
                        let supported_haptic_effects = GamepadSupportedHapticEffects {
                            supports_dual_rumble: true,
                            supports_trigger_rumble: false,
                        };
                        gamepad_event = Some(GamepadEvent::Connected(
                            index,
                            name,
                            bounds,
                            supported_haptic_effects,
                        ));
                    },
                    EventType::Disconnected => {
                        gamepad_event = Some(GamepadEvent::Disconnected(index));
                    },
                    EventType::ForceFeedbackEffectCompleted => {
                        let Some(effect) = self.haptic_effects.get(&event.id.into()) else {
                            warn!("Failed to find haptic effect for id {}", event.id);
                            return;
                        };
                        effect
                            .sender
                            .send(true)
                            .expect("Failed to send haptic effect completion.");
                        self.haptic_effects.remove(&event.id.into());
                    },
                    _ => {},
                }

                if let Some(event) = gamepad_event {
                    self.event_queue.push(EmbedderEvent::Gamepad(event));
                }
            }
        }
    }

    // Map button index and value to represent Standard Gamepad button
    // <https://www.w3.org/TR/gamepad/#dfn-represents-a-standard-gamepad-button>
    fn map_gamepad_button(button: gilrs::Button) -> usize {
        match button {
            gilrs::Button::South => 0,
            gilrs::Button::East => 1,
            gilrs::Button::West => 2,
            gilrs::Button::North => 3,
            gilrs::Button::LeftTrigger => 4,
            gilrs::Button::RightTrigger => 5,
            gilrs::Button::LeftTrigger2 => 6,
            gilrs::Button::RightTrigger2 => 7,
            gilrs::Button::Select => 8,
            gilrs::Button::Start => 9,
            gilrs::Button::LeftThumb => 10,
            gilrs::Button::RightThumb => 11,
            gilrs::Button::DPadUp => 12,
            gilrs::Button::DPadDown => 13,
            gilrs::Button::DPadLeft => 14,
            gilrs::Button::DPadRight => 15,
            gilrs::Button::Mode => 16,
            _ => 17, // Other buttons do not map to "standard" gamepad mapping and are ignored
        }
    }

    fn play_haptic_effect(
        &mut self,
        index: usize,
        params: DualRumbleEffectParams,
        effect_complete_sender: IpcSender<bool>,
    ) {
        let Some(ref mut gilrs) = self.gamepad else {
            debug!("Unable to get gilrs instance!");
            return;
        };

        if let Some(connected_gamepad) = gilrs
            .gamepads()
            .find(|gamepad| usize::from(gamepad.0) == index)
        {
            let start_delay = Ticks::from_ms(params.start_delay as u32);
            let duration = Ticks::from_ms(params.duration as u32);
            let strong_magnitude = (params.strong_magnitude * u16::MAX as f64).round() as u16;
            let weak_magnitude = (params.weak_magnitude * u16::MAX as f64).round() as u16;

            let scheduling = Replay {
                after: start_delay,
                play_for: duration,
                with_delay: Ticks::from_ms(0),
            };
            let effect = EffectBuilder::new()
                .add_effect(BaseEffect {
                    kind: BaseEffectType::Strong { magnitude: strong_magnitude },
                    scheduling,
                    envelope: Default::default(),
                })
                .add_effect(BaseEffect {
                    kind: BaseEffectType::Weak { magnitude: weak_magnitude },
                    scheduling,
                    envelope: Default::default(),
                })
                .repeat(Repeat::For(start_delay + duration))
                .add_gamepad(&connected_gamepad.1)
                .finish(gilrs)
                .expect("Failed to create haptic effect, ensure connected gamepad supports force feedback.");
            self.haptic_effects.insert(
                index,
                HapticEffect {
                    effect,
                    sender: effect_complete_sender,
                },
            );
            self.haptic_effects[&index]
                .effect
                .play()
                .expect("Failed to play haptic effect.");
        } else {
            debug!("Couldn't find connected gamepad to play haptic effect on");
        }
    }

    fn stop_haptic_effect(&mut self, index: usize) -> bool {
        let Some(haptic_effect) = self.haptic_effects.get(&index) else {
            return false;
        };

        let stopped_successfully = match haptic_effect.effect.stop() {
            Ok(()) => true,
            Err(e) => {
                debug!("Failed to stop haptic effect: {:?}", e);
                false
            },
        };
        self.haptic_effects.remove(&index);

        stopped_successfully
    }

    pub fn shutdown_requested(&self) -> bool {
        self.shutdown_requested
    }

    fn focus_webview_by_index(&self, index: usize) -> Option<EmbedderEvent> {
        Some(EmbedderEvent::FocusWebView(self.webviews().get(index)?.0))
    }

    fn get_focused_webview_index(&self) -> Option<usize> {
        let focused_id = self.focused_webview_id?;
        self.webviews()
            .iter()
            .position(|webview| webview.0 == focused_id)
    }

    /// Handle key events before sending them to Servo.
    fn handle_key_from_window(&mut self, key_event: KeyboardEvent) {
        let embedder_event = ShortcutMatcher::from_event(key_event.clone())
            .shortcut(CMD_OR_CONTROL, 'R', || {
                self.focused_webview_id.map(EmbedderEvent::Reload)
            })
            // Select the first 8 tabs via shortcuts
            .shortcut(CMD_OR_CONTROL, '1', || self.focus_webview_by_index(0))
            .shortcut(CMD_OR_CONTROL, '2', || self.focus_webview_by_index(1))
            .shortcut(CMD_OR_CONTROL, '3', || self.focus_webview_by_index(2))
            .shortcut(CMD_OR_CONTROL, '4', || self.focus_webview_by_index(3))
            .shortcut(CMD_OR_CONTROL, '5', || self.focus_webview_by_index(4))
            .shortcut(CMD_OR_CONTROL, '6', || self.focus_webview_by_index(5))
            .shortcut(CMD_OR_CONTROL, '7', || self.focus_webview_by_index(6))
            .shortcut(CMD_OR_CONTROL, '8', || self.focus_webview_by_index(7))
            // Cmd/Ctrl 9 is a bit different in that it focuses the last tab instead of the 9th
            .shortcut(CMD_OR_CONTROL, '9', || {
                let len = self.webviews().len();
                if len > 0 {
                    self.focus_webview_by_index(len - 1)
                } else {
                    None
                }
            })
            .shortcut(Modifiers::CONTROL, Key::PageDown, || {
                let i = self.get_focused_webview_index()?;
                self.focus_webview_by_index((i + 1) % self.webviews().len())
            })
            .shortcut(Modifiers::CONTROL, Key::PageUp, || {
                let index = self.get_focused_webview_index()?;
                let new_index = if index == 0 {
                    self.webviews().len() - 1
                } else {
                    index - 1
                };
                self.focus_webview_by_index(new_index)
            })
            .shortcut(CMD_OR_CONTROL, 'W', || {
                self.focused_webview_id.map(EmbedderEvent::CloseWebView)
            })
            .shortcut(CMD_OR_CONTROL, 'T', || {
                let url = ServoUrl::parse("servo:newtab").unwrap();
                Some(EmbedderEvent::NewWebView(url, WebViewId::new()))
            })
            .shortcut(CMD_OR_CONTROL, 'Q', || Some(EmbedderEvent::Quit))
            .shortcut(CMD_OR_CONTROL, 'P', || {
                let rate = env::var("SAMPLING_RATE")
                    .ok()
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(10);
                let duration = env::var("SAMPLING_DURATION")
                    .ok()
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(10);
                Some(EmbedderEvent::ToggleSamplingProfiler(
                    Duration::from_millis(rate),
                    Duration::from_secs(duration),
                ))
            })
            .shortcut(Modifiers::CONTROL, Key::F9, || {
                Some(EmbedderEvent::CaptureWebRender)
            })
            .shortcut(Modifiers::CONTROL, Key::F10, || {
                Some(EmbedderEvent::ToggleWebRenderDebug(
                    WebRenderDebugOption::RenderTargetDebug,
                ))
            })
            .shortcut(Modifiers::CONTROL, Key::F11, || {
                Some(EmbedderEvent::ToggleWebRenderDebug(
                    WebRenderDebugOption::TextureCacheDebug,
                ))
            })
            .shortcut(Modifiers::CONTROL, Key::F12, || {
                Some(EmbedderEvent::ToggleWebRenderDebug(
                    WebRenderDebugOption::Profiler,
                ))
            })
            .shortcut(CMD_OR_ALT, Key::ArrowRight, || {
                self.focused_webview_id
                    .map(|id| EmbedderEvent::Navigation(id, TraversalDirection::Forward(1)))
            })
            .optional_shortcut(
                cfg!(not(target_os = "windows")),
                CMD_OR_CONTROL,
                ']',
                || {
                    self.focused_webview_id()
                        .map(|id| EmbedderEvent::Navigation(id, TraversalDirection::Forward(1)))
                },
            )
            .shortcut(CMD_OR_ALT, Key::ArrowLeft, || {
                self.focused_webview_id
                    .map(|id| EmbedderEvent::Navigation(id, TraversalDirection::Back(1)))
            })
            .optional_shortcut(
                cfg!(not(target_os = "windows")),
                CMD_OR_CONTROL,
                '[',
                || {
                    self.focused_webview_id
                        .map(|id| EmbedderEvent::Navigation(id, TraversalDirection::Back(1)))
                },
            )
            .optional_shortcut(
                self.window.get_fullscreen(),
                Modifiers::empty(),
                Key::Escape,
                || self.focused_webview_id.map(EmbedderEvent::ExitFullScreen),
            )
            .otherwise(|| {
                self.focused_webview_id
                    .map(|_id| EmbedderEvent::Keyboard(key_event))
            })
            .flatten();
        if let Some(event) = embedder_event {
            self.event_queue.push(event);
        }
    }

    /// Handle key events after they have been handled by Servo.
    fn handle_key_from_servo(&mut self, webview_id: Option<WebViewId>, event: KeyboardEvent) {
        ShortcutMatcher::from_event(event)
            .shortcut(CMD_OR_CONTROL, '=', || {
                self.event_queue.push(EmbedderEvent::Zoom(1.1))
            })
            .shortcut(CMD_OR_CONTROL, '+', || {
                self.event_queue.push(EmbedderEvent::Zoom(1.1))
            })
            .shortcut(CMD_OR_CONTROL, '-', || {
                self.event_queue.push(EmbedderEvent::Zoom(1.0 / 1.1))
            })
            .shortcut(CMD_OR_CONTROL, '0', || {
                self.event_queue.push(EmbedderEvent::ResetZoom)
            })
            .shortcut(Modifiers::empty(), Key::PageDown, || {
                let scroll_location = ScrollLocation::Delta(Vector2D::new(
                    0.0,
                    -self.window.page_height() + 2.0 * LINE_HEIGHT,
                ));
                self.scroll_window_from_key(scroll_location, TouchEventType::Move, webview_id);
            })
            .shortcut(Modifiers::empty(), Key::PageUp, || {
                let scroll_location = ScrollLocation::Delta(Vector2D::new(
                    0.0,
                    self.window.page_height() - 2.0 * LINE_HEIGHT,
                ));
                self.scroll_window_from_key(scroll_location, TouchEventType::Move, webview_id);
            })
            .shortcut(Modifiers::empty(), Key::Home, || {
                self.scroll_window_from_key(
                    ScrollLocation::Start,
                    TouchEventType::Move,
                    webview_id,
                );
            })
            .shortcut(Modifiers::empty(), Key::End, || {
                self.scroll_window_from_key(ScrollLocation::End, TouchEventType::Move, webview_id);
            })
            .shortcut(Modifiers::empty(), Key::ArrowUp, || {
                self.scroll_window_from_key(
                    ScrollLocation::Delta(Vector2D::new(0.0, 3.0 * LINE_HEIGHT)),
                    TouchEventType::Move,
                    webview_id,
                );
            })
            .shortcut(Modifiers::empty(), Key::ArrowDown, || {
                self.scroll_window_from_key(
                    ScrollLocation::Delta(Vector2D::new(0.0, -3.0 * LINE_HEIGHT)),
                    TouchEventType::Move,
                    webview_id,
                );
            })
            .shortcut(Modifiers::empty(), Key::ArrowLeft, || {
                self.scroll_window_from_key(
                    ScrollLocation::Delta(Vector2D::new(LINE_HEIGHT, 0.0)),
                    TouchEventType::Move,
                    webview_id,
                );
            })
            .shortcut(Modifiers::empty(), Key::ArrowRight, || {
                self.scroll_window_from_key(
                    ScrollLocation::Delta(Vector2D::new(-LINE_HEIGHT, 0.0)),
                    TouchEventType::Move,
                    webview_id,
                );
            });
    }

    fn scroll_window_from_key(
        &mut self,
        scroll_location: ScrollLocation,
        phase: TouchEventType,
        webview_id: Option<WebViewId>,
    ) {
        // In minibrowser mode the webview is offset by the toolbar
        let origin = webview_id
            .and_then(|id| self.webviews.get(&id))
            .map(|webview| webview.rect.min.ceil().to_i32())
            .unwrap_or(Point2D::zero());
        let event = EmbedderEvent::Scroll(scroll_location, origin, phase);
        self.event_queue.push(event);
    }

    /// Returns true if the caller needs to manually present a new frame.
    pub fn handle_servo_events(
        &mut self,
        events: Drain<'_, (Option<WebViewId>, EmbedderMsg)>,
    ) -> ServoEventResponse {
        let mut need_present = self.load_status() != LoadStatus::LoadComplete;
        let mut need_update = false;
        for (webview_id, msg) in events {
            if let Some(webview_id) = webview_id {
                trace_embedder_msg!(msg, "{webview_id} {msg:?}");
            } else {
                trace_embedder_msg!(msg, "{msg:?}");
            }
            match msg {
                EmbedderMsg::Status(status) => {
                    self.status_text = status;
                    need_update = true;
                },
                EmbedderMsg::ChangePageTitle(title) => {
                    // Set the title to the target webview, and update the OS window title
                    // if this is the currently focused one.
                    if let Some(webview_id) = webview_id {
                        if let Some(webview) = self.get_mut(webview_id) {
                            webview.title = title.clone();
                            if webview.focused {
                                self.window.set_title(&format!(
                                    "{} - Servo",
                                    title.clone().unwrap_or_default()
                                ));
                            }
                            need_update = true;
                        } else {
                            let data = self.ensure_preload_data_mut(&webview_id);
                            data.title = title.clone();
                        }
                    }
                },
                EmbedderMsg::MoveTo(point) => {
                    self.window.set_position(point);
                },
                EmbedderMsg::ResizeTo(size) => {
                    if let Some(webview_id) = webview_id {
                        let new_rect = self.get_mut(webview_id).and_then(|webview| {
                            if webview.rect.size() != size.to_f32() {
                                webview.rect.set_size(size.to_f32());
                                Some(webview.rect)
                            } else {
                                None
                            }
                        });
                        if let Some(new_rect) = new_rect {
                            self.event_queue
                                .push(EmbedderEvent::MoveResizeWebView(webview_id, new_rect));
                        }
                    }
                    self.window.request_inner_size(size);
                },
                EmbedderMsg::Prompt(definition, origin) => {
                    let res = if opts::get().headless {
                        match definition {
                            PromptDefinition::Alert(_message, sender) => sender.send(()),
                            PromptDefinition::YesNo(_message, sender) => {
                                sender.send(PromptResult::Primary)
                            },
                            PromptDefinition::OkCancel(_message, sender) => {
                                sender.send(PromptResult::Primary)
                            },
                            PromptDefinition::Input(_message, default, sender) => {
                                sender.send(Some(default.to_owned()))
                            },
                            PromptDefinition::Credentials(sender) => {
                                sender.send(PromptCredentialsInput {
                                    username: None,
                                    password: None,
                                })
                            },
                        }
                    } else {
                        thread::Builder::new()
                            .name("AlertDialog".to_owned())
                            .spawn(move || match definition {
                                PromptDefinition::Alert(mut message, sender) => {
                                    if origin == PromptOrigin::Untrusted {
                                        message = tiny_dialog_escape(&message);
                                    }
                                    tinyfiledialogs::message_box_ok(
                                        "Alert!",
                                        &message,
                                        MessageBoxIcon::Warning,
                                    );
                                    sender.send(())
                                },
                                PromptDefinition::YesNo(mut message, sender) => {
                                    if origin == PromptOrigin::Untrusted {
                                        message = tiny_dialog_escape(&message);
                                    }
                                    let result = tinyfiledialogs::message_box_yes_no(
                                        "",
                                        &message,
                                        MessageBoxIcon::Warning,
                                        YesNo::No,
                                    );
                                    sender.send(match result {
                                        YesNo::Yes => PromptResult::Primary,
                                        YesNo::No => PromptResult::Secondary,
                                    })
                                },
                                PromptDefinition::OkCancel(mut message, sender) => {
                                    if origin == PromptOrigin::Untrusted {
                                        message = tiny_dialog_escape(&message);
                                    }
                                    let result = tinyfiledialogs::message_box_ok_cancel(
                                        "",
                                        &message,
                                        MessageBoxIcon::Warning,
                                        OkCancel::Cancel,
                                    );
                                    sender.send(match result {
                                        OkCancel::Ok => PromptResult::Primary,
                                        OkCancel::Cancel => PromptResult::Secondary,
                                    })
                                },
                                PromptDefinition::Input(mut message, mut default, sender) => {
                                    if origin == PromptOrigin::Untrusted {
                                        message = tiny_dialog_escape(&message);
                                        default = tiny_dialog_escape(&default);
                                    }
                                    let result = tinyfiledialogs::input_box("", &message, &default);
                                    sender.send(result)
                                },
                                PromptDefinition::Credentials(sender) => {
                                    // TODO: figure out how to make the message a localized string
                                    let username = tinyfiledialogs::input_box("", "username", "");
                                    let password = tinyfiledialogs::input_box("", "password", "");
                                    sender.send(PromptCredentialsInput { username, password })
                                },
                            })
                            .unwrap()
                            .join()
                            .expect("Thread spawning failed")
                    };
                    if let Err(e) = res {
                        let reason = format!("Failed to send Prompt response: {}", e);
                        self.event_queue
                            .push(EmbedderEvent::SendError(webview_id, reason));
                    }
                },
                EmbedderMsg::AllowUnload(sender) => {
                    // Always allow unload for now.
                    if let Err(e) = sender.send(true) {
                        let reason = format!("Failed to send AllowUnload response: {}", e);
                        self.event_queue
                            .push(EmbedderEvent::SendError(webview_id, reason));
                    }
                },
                EmbedderMsg::AllowNavigationRequest(pipeline_id, _url) => {
                    if let Some(_webview_id) = webview_id {
                        self.event_queue
                            .push(EmbedderEvent::AllowNavigationResponse(pipeline_id, true));
                    }
                },
                EmbedderMsg::AllowOpeningWebView(response_chan) => {
                    // Note: would be a place to handle pop-ups config.
                    // see Step 7 of #the-rules-for-choosing-a-browsing-context-given-a-browsing-context-name
                    if let Err(e) = response_chan.send(true) {
                        warn!("Failed to send AllowOpeningWebView response: {}", e);
                    };
                },
                EmbedderMsg::WebViewOpened(new_webview_id) => {
                    let scale = self.window.hidpi_factor().get();
                    let toolbar = self.window.toolbar_height().get();

                    // Adjust for our toolbar height.
                    // TODO: Adjust for egui window decorations if we end up using those
                    let mut rect = self.window.get_coordinates().get_viewport().to_f32();
                    rect.min.y += toolbar * scale;

                    // Make sure to not add duplicates into the creation_order vector.
                    // This can happen as explained in https://github.com/servo/servo/issues/33075
                    let preload_data = self.ensure_preload_data_mut(&new_webview_id).clone();
                    if let Entry::Vacant(entry) = self.webviews.entry(new_webview_id) {
                        entry.insert(WebView::new(rect, preload_data));
                        self.creation_order.push(new_webview_id);
                        self.event_queue
                            .push(EmbedderEvent::FocusWebView(new_webview_id));
                        self.event_queue
                            .push(EmbedderEvent::MoveResizeWebView(new_webview_id, rect));
                        self.event_queue
                            .push(EmbedderEvent::RaiseWebViewToTop(new_webview_id, true));
                    }
                },
                EmbedderMsg::WebViewClosed(webview_id) => {
                    self.webviews.retain(|&id, _| id != webview_id);
                    self.creation_order.retain(|&id| id != webview_id);
                    self.focused_webview_id = None;
                    if let Some(&newest_webview_id) = self.creation_order.last() {
                        self.event_queue
                            .push(EmbedderEvent::FocusWebView(newest_webview_id));
                    } else {
                        self.event_queue.push(EmbedderEvent::Quit);
                    }
                },
                EmbedderMsg::WebViewFocused(webview_id) => {
                    for (id, webview) in &mut self.webviews {
                        webview.focused = *id == webview_id;
                    }
                    self.focused_webview_id = Some(webview_id);
                    need_update = true;
                    // Show the most recently created webview and hide all others.
                    // TODO: Stop doing this once we have full multiple webviews support
                    self.event_queue
                        .push(EmbedderEvent::ShowWebView(webview_id, true));
                },
                EmbedderMsg::WebViewBlurred => {
                    for webview in self.webviews.values_mut() {
                        webview.focused = false;
                    }
                    self.focused_webview_id = None;
                },
                EmbedderMsg::Keyboard(key_event) => {
                    self.handle_key_from_servo(webview_id, key_event);
                },
                EmbedderMsg::GetClipboardContents(sender) => {
                    let contents = self
                        .clipboard
                        .as_mut()
                        .and_then(|clipboard| clipboard.get_text().ok())
                        .unwrap_or_else(|| {
                            warn!("Error getting clipboard text. Returning empty string.");
                            String::new()
                        });
                    if let Err(e) = sender.send(contents) {
                        warn!("Failed to send clipboard ({})", e);
                    }
                },
                EmbedderMsg::SetClipboardContents(text) => {
                    if let Some(ref mut clipboard) = self.clipboard {
                        if let Err(e) = clipboard.set_text(text) {
                            warn!("Error setting clipboard contents ({})", e);
                        }
                    }
                },
                EmbedderMsg::SetCursor(cursor) => {
                    self.window.set_cursor(cursor);
                },
                EmbedderMsg::NewFavicon(_url) => {
                    // FIXME: show favicons in the UI somehow
                },
                EmbedderMsg::HeadParsed => {
                    if let Some(webview_id) = webview_id {
                        if let Some(webview) = self.get_mut(webview_id) {
                            webview.load_status = LoadStatus::HeadParsed;
                            need_update = true;
                        }
                    }
                },
                EmbedderMsg::HistoryChanged(urls, current) => {
                    if let Some(webview_id) = webview_id {
                        if let Some(webview) = self.get_mut(webview_id) {
                            webview.url = Some(urls[current].clone());
                            need_update = true;
                        } else {
                            let data = self.ensure_preload_data_mut(&webview_id);
                            data.url = Some(urls[current].clone());
                        }
                    }
                },
                EmbedderMsg::SetFullscreenState(state) => {
                    self.window.set_fullscreen(state);
                },
                EmbedderMsg::LoadStart => {
                    if let Some(webview_id) = webview_id {
                        if let Some(webview) = self.get_mut(webview_id) {
                            webview.load_status = LoadStatus::LoadStart;
                            need_update = true;
                        }
                    }
                },
                EmbedderMsg::LoadComplete => {
                    if let Some(webview_id) = webview_id {
                        if let Some(webview) = self.get_mut(webview_id) {
                            webview.load_status = LoadStatus::LoadComplete;
                            need_update = true;
                        }
                    }
                },
                EmbedderMsg::Shutdown => {
                    self.shutdown_requested = true;
                },
                EmbedderMsg::Panic(_reason, _backtrace) => {},
                EmbedderMsg::GetSelectedBluetoothDevice(devices, sender) => {
                    let selected = platform_get_selected_devices(devices);
                    if let Err(e) = sender.send(selected) {
                        let reason =
                            format!("Failed to send GetSelectedBluetoothDevice response: {}", e);
                        self.event_queue
                            .push(EmbedderEvent::SendError(None, reason));
                    };
                },
                EmbedderMsg::SelectFiles(patterns, multiple_files, sender) => {
                    let res = match (
                        opts::get().headless,
                        get_selected_files(patterns, multiple_files),
                    ) {
                        (true, _) | (false, None) => sender.send(None),
                        (false, Some(files)) => sender.send(Some(files)),
                    };
                    if let Err(e) = res {
                        let reason = format!("Failed to send SelectFiles response: {}", e);
                        self.event_queue
                            .push(EmbedderEvent::SendError(None, reason));
                    };
                },
                EmbedderMsg::PromptPermission(prompt, sender) => {
                    let permission_state = prompt_user(prompt);
                    let _ = sender.send(permission_state);
                },
                EmbedderMsg::ShowIME(_kind, _text, _multiline, _rect) => {
                    debug!("ShowIME received");
                },
                EmbedderMsg::HideIME => {
                    debug!("HideIME received");
                },
                EmbedderMsg::ReportProfile(bytes) => {
                    let filename = env::var("PROFILE_OUTPUT").unwrap_or("samples.json".to_string());
                    let result = File::create(&filename).and_then(|mut f| f.write_all(&bytes));
                    if let Err(e) = result {
                        error!("Failed to store profile: {}", e);
                    }
                },
                EmbedderMsg::MediaSessionEvent(_) => {
                    debug!("MediaSessionEvent received");
                    // TODO(ferjm): MediaSession support for winit based browsers.
                },
                EmbedderMsg::OnDevtoolsStarted(port, _token) => match port {
                    Ok(p) => info!("Devtools Server running on port {}", p),
                    Err(()) => error!("Error running devtools server"),
                },
                EmbedderMsg::ShowContextMenu(sender, ..) => {
                    let _ = sender.send(ContextMenuResult::Ignored);
                },
                EmbedderMsg::ReadyToPresent(_webview_ids) => {
                    need_present = true;
                },
                EmbedderMsg::EventDelivered(event) => {
                    if let (Some(webview_id), CompositorEventVariant::MouseButtonEvent) =
                        (webview_id, event)
                    {
                        trace!("{}: Got a mouse button event", webview_id);
                        self.event_queue
                            .push(EmbedderEvent::RaiseWebViewToTop(webview_id, true));
                        self.event_queue
                            .push(EmbedderEvent::FocusWebView(webview_id));
                    }
                },
                EmbedderMsg::PlayGamepadHapticEffect(index, effect, effect_complete_sender) => {
                    match effect {
                        GamepadHapticEffectType::DualRumble(params) => {
                            self.play_haptic_effect(index, params, effect_complete_sender);
                        },
                    }
                },
                EmbedderMsg::StopGamepadHapticEffect(index, haptic_stop_sender) => {
                    let stopped_successfully = self.stop_haptic_effect(index);
                    haptic_stop_sender
                        .send(stopped_successfully)
                        .expect("Failed to send haptic stop result");
                },
            }
        }

        ServoEventResponse {
            need_present,
            need_update,
        }
    }
}

#[cfg(target_os = "linux")]
fn prompt_user(prompt: PermissionPrompt) -> PermissionRequest {
    if opts::get().headless {
        return PermissionRequest::Denied;
    }

    let message = match prompt {
        PermissionPrompt::Request(permission_name) => {
            format!("Do you want to grant permission for {:?}?", permission_name)
        },
        PermissionPrompt::Insecure(permission_name) => {
            format!(
                "The {:?} feature is only safe to use in secure context, but servo can't guarantee\n\
                that the current context is secure. Do you want to proceed and grant permission?",
                permission_name
            )
        },
    };

    match tinyfiledialogs::message_box_yes_no(
        "Permission request dialog",
        &message,
        MessageBoxIcon::Question,
        YesNo::No,
    ) {
        YesNo::Yes => PermissionRequest::Granted,
        YesNo::No => PermissionRequest::Denied,
    }
}

#[cfg(not(target_os = "linux"))]
fn prompt_user(_prompt: PermissionPrompt) -> PermissionRequest {
    // TODO popup only supported on linux
    PermissionRequest::Denied
}

#[cfg(target_os = "linux")]
fn platform_get_selected_devices(devices: Vec<String>) -> Option<String> {
    thread::Builder::new()
        .name("DevicePicker".to_owned())
        .spawn(move || {
            let dialog_rows: Vec<&str> = devices.iter().map(|s| s.as_ref()).collect();
            let dialog_rows: Option<&[&str]> = Some(dialog_rows.as_slice());

            match tinyfiledialogs::list_dialog("Choose a device", &["Id", "Name"], dialog_rows) {
                Some(device) => {
                    // The device string format will be "Address|Name". We need the first part of it.
                    device.split('|').next().map(|s| s.to_string())
                },
                None => None,
            }
        })
        .unwrap()
        .join()
        .expect("Thread spawning failed")
}

#[cfg(not(target_os = "linux"))]
fn platform_get_selected_devices(devices: Vec<String>) -> Option<String> {
    for device in devices {
        if let Some(address) = device.split('|').next().map(|s| s.to_string()) {
            return Some(address);
        }
    }
    None
}

fn get_selected_files(patterns: Vec<FilterPattern>, multiple_files: bool) -> Option<Vec<String>> {
    let picker_name = if multiple_files {
        "Pick files"
    } else {
        "Pick a file"
    };
    thread::Builder::new()
        .name("FilePicker".to_owned())
        .spawn(move || {
            let mut filters = vec![];
            for p in patterns {
                let s = "*.".to_string() + &p.0;
                filters.push(tiny_dialog_escape(&s))
            }
            let filter_ref = &(filters.iter().map(|s| s.as_str()).collect::<Vec<&str>>()[..]);
            let filter_opt = if !filters.is_empty() {
                Some((filter_ref, ""))
            } else {
                None
            };

            if multiple_files {
                tinyfiledialogs::open_file_dialog_multi(picker_name, "", filter_opt)
            } else {
                let file = tinyfiledialogs::open_file_dialog(picker_name, "", filter_opt);
                file.map(|x| vec![x])
            }
        })
        .unwrap()
        .join()
        .expect("Thread spawning failed")
}

// This is a mitigation for #25498, not a verified solution.
// There may be codepaths in tinyfiledialog.c that this is
// inadquate against, as it passes the string via shell to
// different programs depending on what the user has installed.
#[cfg(target_os = "linux")]
fn tiny_dialog_escape(raw: &str) -> String {
    let s: String = raw
        .chars()
        .filter_map(|c| match c {
            '\n' => Some('\n'),
            '\0'..='\x1f' => None,
            '<' => Some('\u{FF1C}'),
            '>' => Some('\u{FF1E}'),
            '&' => Some('\u{FF06}'),
            _ => Some(c),
        })
        .collect();
    shellwords::escape(&s)
}

#[cfg(not(target_os = "linux"))]
fn tiny_dialog_escape(raw: &str) -> String {
    raw.to_string()
}

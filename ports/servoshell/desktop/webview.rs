/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use std::rc::Rc;
use std::{env, thread};

use arboard::Clipboard;
use euclid::Vector2D;
use gilrs::ff::{BaseEffect, BaseEffectType, Effect, EffectBuilder, Repeat, Replay, Ticks};
use gilrs::{EventType, Gilrs};
use keyboard_types::{Key, KeyboardEvent, Modifiers, ShortcutMatcher};
use log::{debug, error, info, warn};
use servo::base::id::WebViewId;
use servo::config::opts::Opts;
use servo::ipc_channel::ipc::IpcSender;
use servo::servo_url::ServoUrl;
use servo::webrender_api::units::DeviceRect;
use servo::webrender_api::ScrollLocation;
use servo::{
    CompositorEventVariant, ContextMenuResult, DualRumbleEffectParams, EmbedderMsg, FilterPattern,
    GamepadEvent, GamepadHapticEffectType, GamepadIndex, GamepadInputBounds,
    GamepadSupportedHapticEffects, GamepadUpdateType, PermissionPrompt, PermissionRequest,
    PromptCredentialsInput, PromptDefinition, PromptOrigin, PromptResult, Servo, TouchEventType,
};
use tinyfiledialogs::{self, MessageBoxIcon, OkCancel, YesNo};

use super::keyutils::CMD_OR_CONTROL;
use super::window_trait::{WindowPortsMethods, LINE_HEIGHT};
use crate::desktop::tracing::trace_embedder_msg;

pub struct WebViewManager {
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

    window: Rc<dyn WindowPortsMethods>,
    gamepad: Option<Gilrs>,
    haptic_effects: HashMap<usize, HapticEffect>,
    shutdown_requested: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LoadStatus {
    HeadParsed,
    LoadStart,
    LoadComplete,
}

// The state of each Tab/WebView
pub struct WebView {
    pub rect: DeviceRect,
    pub title: Option<String>,
    pub url: Option<ServoUrl>,
    pub focused: bool,
    pub load_status: LoadStatus,
    pub servo_webview: ::servo::WebView,
}

impl WebView {
    fn new(servo_webview: ::servo::WebView) -> Self {
        Self {
            rect: DeviceRect::zero(),
            title: None,
            url: None,
            focused: false,
            load_status: LoadStatus::LoadComplete,
            servo_webview,
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

impl WebViewManager {
    pub fn new(window: Rc<dyn WindowPortsMethods>) -> WebViewManager {
        WebViewManager {
            status_text: None,
            webviews: HashMap::default(),
            creation_order: vec![],
            focused_webview_id: None,
            window,
            gamepad: match Gilrs::new() {
                Ok(g) => Some(g),
                Err(e) => {
                    warn!("Error creating gamepad input connection ({})", e);
                    None
                },
            },
            haptic_effects: HashMap::default(),
            shutdown_requested: false,
        }
    }

    pub fn get_mut(&mut self, webview_id: WebViewId) -> Option<&mut WebView> {
        self.webviews.get_mut(&webview_id)
    }

    pub fn get(&self, webview_id: WebViewId) -> Option<&WebView> {
        self.webviews.get(&webview_id)
    }

    pub(crate) fn add(&mut self, webview: ::servo::WebView) {
        self.creation_order.push(webview.id());
        self.webviews.insert(webview.id(), WebView::new(webview));
    }

    pub fn focused_webview_id(&self) -> Option<WebViewId> {
        self.focused_webview_id
    }

    pub fn close_webview(&mut self, servo: &Servo, webview_id: WebViewId) {
        // This can happen because we can trigger a close with a UI action and then get the
        // close event from Servo later.
        if !self.webviews.contains_key(&webview_id) {
            return;
        }

        self.webviews.retain(|&id, _| id != webview_id);
        self.creation_order.retain(|&id| id != webview_id);
        self.focused_webview_id = None;
        match self.last_created_webview() {
            Some(last_created_webview) => last_created_webview.servo_webview.focus(),
            None => servo.start_shutting_down(),
        }
    }

    fn last_created_webview(&self) -> Option<&WebView> {
        self.creation_order
            .last()
            .and_then(|id| self.webviews.get(id))
    }

    pub fn current_url_string(&self) -> Option<String> {
        match self.focused_webview() {
            Some(webview) => webview.url.as_ref().map(|url| url.to_string()),
            None => None,
        }
    }

    pub fn focused_webview(&self) -> Option<&WebView> {
        self.focused_webview_id
            .and_then(|id| self.webviews.get(&id))
    }

    pub fn load_status(&self) -> LoadStatus {
        match self.focused_webview() {
            Some(webview) => webview.load_status,
            None => LoadStatus::LoadComplete,
        }
    }

    pub fn status_text(&self) -> Option<String> {
        self.status_text.clone()
    }

    // Returns the webviews in the creation order.
    pub fn webviews(&self) -> Vec<(WebViewId, &WebView)> {
        let mut res = vec![];
        for id in &self.creation_order {
            res.push((*id, self.webviews.get(id).unwrap()))
        }
        res
    }

    /// Handle updates to connected gamepads from GilRs
    pub fn handle_gamepad_events(&mut self) {
        let Some(webview) = self
            .focused_webview()
            .map(|webview| webview.servo_webview.clone())
        else {
            return;
        };

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
                    webview.notify_gamepad_event(event);
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

    pub(crate) fn focus_webview_by_index(&self, index: usize) {
        if let Some((_, webview)) = self.webviews().get(index) {
            webview.servo_webview.focus();
        }
    }

    pub(crate) fn get_focused_webview_index(&self) -> Option<usize> {
        let focused_id = self.focused_webview_id?;
        self.webviews()
            .iter()
            .position(|webview| webview.0 == focused_id)
    }

    fn send_error(&self, webview_id: WebViewId, error: String) {
        let Some(webview) = self.get(webview_id) else {
            return warn!("{error}");
        };
        webview.servo_webview.send_error(error);
    }

    /// Returns true if the caller needs to manually present a new frame.
    pub fn handle_servo_events(
        &mut self,
        servo: &mut Servo,
        clipboard: &mut Option<Clipboard>,
        opts: &Opts,
        messages: Vec<EmbedderMsg>,
    ) -> ServoEventResponse {
        let mut need_present = self.load_status() != LoadStatus::LoadComplete;
        let mut need_update = false;
        for message in messages {
            trace_embedder_msg!(message, "{message:?}");

            match message {
                EmbedderMsg::Status(_, status) => {
                    self.status_text = status;
                    need_update = true;
                },
                EmbedderMsg::ChangePageTitle(webview_id, title) => {
                    // Set the title to the target webview, and update the OS window title
                    // if this is the currently focused one.
                    if let Some(webview) = self.get_mut(webview_id) {
                        webview.title = title.clone();
                        if webview.focused {
                            self.window.set_title(&format!(
                                "{} - Servo",
                                title.clone().unwrap_or_default()
                            ));
                        }
                        need_update = true;
                    }
                },
                EmbedderMsg::MoveTo(_, point) => {
                    self.window.set_position(point);
                },
                EmbedderMsg::ResizeTo(webview_id, inner_size) => {
                    if let Some(webview) = self.get_mut(webview_id) {
                        if webview.rect.size() != inner_size.to_f32() {
                            webview.rect.set_size(inner_size.to_f32());
                            webview.servo_webview.move_resize(webview.rect);
                        }
                    };
                    if let Some(webview) = self.get(webview_id) {
                        self.window.request_resize(webview, inner_size);
                    }
                },
                EmbedderMsg::Prompt(webview_id, definition, origin) => {
                    let res = if opts.headless {
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
                        self.send_error(webview_id, format!("Failed to send Prompt response: {e}"))
                    }
                },
                EmbedderMsg::AllowUnload(webview_id, sender) => {
                    // Always allow unload for now.
                    if let Err(e) = sender.send(true) {
                        self.send_error(
                            webview_id,
                            format!("Failed to send AllowUnload response: {e}"),
                        )
                    }
                },
                EmbedderMsg::AllowNavigationRequest(_, pipeline_id, _url) => {
                    servo.allow_navigation_response(pipeline_id, true);
                },
                EmbedderMsg::AllowOpeningWebView(_, response_chan) => {
                    let webview = servo.new_auxiliary_webview();
                    match response_chan.send(Some(webview.id())) {
                        Ok(()) => self.add(webview),
                        Err(error) => warn!("Failed to send AllowOpeningWebView response: {error}"),
                    }
                },
                EmbedderMsg::WebViewOpened(new_webview_id) => {
                    let scale = self.window.hidpi_factor().get();
                    let toolbar = self.window.toolbar_height().get();

                    // Adjust for our toolbar height.
                    // TODO: Adjust for egui window decorations if we end up using those
                    let mut rect = self.window.get_coordinates().get_viewport().to_f32();
                    rect.min.y += toolbar * scale;

                    let webview = self
                        .webviews
                        .get(&new_webview_id)
                        .expect("Unknown webview opened.");
                    webview.servo_webview.focus();
                    webview.servo_webview.move_resize(rect);
                    webview.servo_webview.raise_to_top(true);
                },
                EmbedderMsg::WebViewClosed(webview_id) => {
                    self.close_webview(servo, webview_id);
                },
                EmbedderMsg::WebViewFocused(webview_id) => {
                    if let Some(webview) = self.focused_webview_id.and_then(|id| self.get_mut(id)) {
                        webview.focused = false;
                    }

                    // Show the most recently created webview and hide all others.
                    // TODO: Stop doing this once we have full multiple webviews support
                    if let Some(webview) = self.get_mut(webview_id) {
                        webview.focused = true;
                        webview.servo_webview.show(true);
                        self.focused_webview_id = Some(webview_id);
                        need_update = true;
                    };
                },
                EmbedderMsg::WebViewBlurred => {
                    for webview in self.webviews.values_mut() {
                        webview.focused = false;
                    }
                    self.focused_webview_id = None;
                },
                EmbedderMsg::Keyboard(webview_id, key_event) => {
                    self.handle_overridable_key_bindings(webview_id, key_event);
                },
                EmbedderMsg::ClearClipboardContents(_) => {
                    clipboard
                        .as_mut()
                        .and_then(|clipboard| clipboard.clear().ok());
                },
                EmbedderMsg::GetClipboardContents(_, sender) => {
                    let contents = clipboard
                        .as_mut()
                        .and_then(|clipboard| clipboard.get_text().ok())
                        .unwrap_or_default();
                    if let Err(e) = sender.send(contents) {
                        warn!("Failed to send clipboard ({})", e);
                    }
                },
                EmbedderMsg::SetClipboardContents(_, text) => {
                    if let Some(clipboard) = clipboard.as_mut() {
                        if let Err(e) = clipboard.set_text(text) {
                            warn!("Error setting clipboard contents ({})", e);
                        }
                    }
                },
                EmbedderMsg::SetCursor(_, cursor) => {
                    self.window.set_cursor(cursor);
                },
                EmbedderMsg::NewFavicon(_, _url) => {
                    // FIXME: show favicons in the UI somehow
                },
                EmbedderMsg::HeadParsed(webview_id) => {
                    if let Some(webview) = self.get_mut(webview_id) {
                        webview.load_status = LoadStatus::HeadParsed;
                        need_update = true;
                    };
                },
                EmbedderMsg::HistoryChanged(webview_id, urls, current) => {
                    if let Some(webview) = self.get_mut(webview_id) {
                        webview.url = Some(urls[current].clone());
                        need_update = true;
                    };
                },
                EmbedderMsg::SetFullscreenState(_, state) => {
                    self.window.set_fullscreen(state);
                },
                EmbedderMsg::LoadStart(webview_id) => {
                    if let Some(webview) = self.get_mut(webview_id) {
                        webview.load_status = LoadStatus::LoadStart;
                        need_update = true;
                    };
                },
                EmbedderMsg::LoadComplete(webview_id) => {
                    if let Some(webview) = self.get_mut(webview_id) {
                        webview.load_status = LoadStatus::LoadComplete;
                        need_update = true;
                    };
                },
                EmbedderMsg::WebResourceRequested(_, _web_resource_request, _response_sender) => {},
                EmbedderMsg::Shutdown => {
                    self.shutdown_requested = true;
                },
                EmbedderMsg::Panic(_, _reason, _backtrace) => {},
                EmbedderMsg::GetSelectedBluetoothDevice(webview_id, devices, sender) => {
                    let selected = platform_get_selected_devices(devices);
                    if let Err(e) = sender.send(selected) {
                        self.send_error(
                            webview_id,
                            format!("Failed to send GetSelectedBluetoothDevice response: {e}"),
                        );
                    };
                },
                EmbedderMsg::SelectFiles(webview_id, patterns, multiple_files, sender) => {
                    let result = match (opts.headless, get_selected_files(patterns, multiple_files))
                    {
                        (true, _) | (false, None) => sender.send(None),
                        (false, Some(files)) => sender.send(Some(files)),
                    };
                    if let Err(e) = result {
                        self.send_error(
                            webview_id,
                            format!("Failed to send SelectFiles response: {e}"),
                        );
                    };
                },
                EmbedderMsg::PromptPermission(_, prompt, sender) => {
                    let _ = sender.send(match opts.headless {
                        true => PermissionRequest::Denied,
                        false => prompt_user(prompt),
                    });
                },
                EmbedderMsg::ShowIME(_webview_id, _kind, _text, _multiline, _rect) => {
                    debug!("ShowIME received");
                },
                EmbedderMsg::HideIME(_webview_id) => {
                    debug!("HideIME received");
                },
                EmbedderMsg::ReportProfile(bytes) => {
                    let filename = env::var("PROFILE_OUTPUT").unwrap_or("samples.json".to_string());
                    let result = File::create(&filename).and_then(|mut f| f.write_all(&bytes));
                    if let Err(e) = result {
                        error!("Failed to store profile: {}", e);
                    }
                },
                EmbedderMsg::MediaSessionEvent(..) => {
                    debug!("MediaSessionEvent received");
                    // TODO(ferjm): MediaSession support for winit based browsers.
                },
                EmbedderMsg::OnDevtoolsStarted(port, _token) => match port {
                    Ok(p) => info!("Devtools Server running on port {}", p),
                    Err(()) => error!("Error running devtools server"),
                },
                EmbedderMsg::RequestDevtoolsConnection(response_sender) => {
                    let _ = response_sender.send(true);
                },
                EmbedderMsg::ShowContextMenu(_, sender, ..) => {
                    let _ = sender.send(ContextMenuResult::Ignored);
                },
                EmbedderMsg::ReadyToPresent(_webview_ids) => {
                    need_present = true;
                },
                EmbedderMsg::EventDelivered(webview_id, event) => {
                    if let Some(webview) = self.get_mut(webview_id) {
                        if let CompositorEventVariant::MouseButtonEvent = event {
                            webview.servo_webview.raise_to_top(true);
                            webview.servo_webview.focus();
                        }
                    };
                },
                EmbedderMsg::PlayGamepadHapticEffect(_, index, effect, effect_complete_sender) => {
                    match effect {
                        GamepadHapticEffectType::DualRumble(params) => {
                            self.play_haptic_effect(index, params, effect_complete_sender);
                        },
                    }
                },
                EmbedderMsg::StopGamepadHapticEffect(_, index, haptic_stop_sender) => {
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

    /// Handle servoshell key bindings that may have been prevented by the page in the focused webview.
    fn handle_overridable_key_bindings(&mut self, webview_id: WebViewId, event: KeyboardEvent) {
        let Some(webview) = self.get(webview_id) else {
            return;
        };

        let origin = webview.rect.min.ceil().to_i32();
        let webview = &webview.servo_webview;
        ShortcutMatcher::from_event(event)
            .shortcut(CMD_OR_CONTROL, '=', || {
                webview.set_zoom(1.1);
            })
            .shortcut(CMD_OR_CONTROL, '+', || {
                webview.set_zoom(1.1);
            })
            .shortcut(CMD_OR_CONTROL, '-', || {
                webview.set_zoom(1.0 / 1.1);
            })
            .shortcut(CMD_OR_CONTROL, '0', || {
                webview.reset_zoom();
            })
            .shortcut(Modifiers::empty(), Key::PageDown, || {
                let scroll_location = ScrollLocation::Delta(Vector2D::new(
                    0.0,
                    -self.window.page_height() + 2.0 * LINE_HEIGHT,
                ));
                webview.notify_scroll_event(scroll_location, origin, TouchEventType::Move);
            })
            .shortcut(Modifiers::empty(), Key::PageUp, || {
                let scroll_location = ScrollLocation::Delta(Vector2D::new(
                    0.0,
                    self.window.page_height() - 2.0 * LINE_HEIGHT,
                ));
                webview.notify_scroll_event(scroll_location, origin, TouchEventType::Move);
            })
            .shortcut(Modifiers::empty(), Key::Home, || {
                webview.notify_scroll_event(ScrollLocation::Start, origin, TouchEventType::Move);
            })
            .shortcut(Modifiers::empty(), Key::End, || {
                webview.notify_scroll_event(ScrollLocation::End, origin, TouchEventType::Move);
            })
            .shortcut(Modifiers::empty(), Key::ArrowUp, || {
                let location = ScrollLocation::Delta(Vector2D::new(0.0, 3.0 * LINE_HEIGHT));
                webview.notify_scroll_event(location, origin, TouchEventType::Move);
            })
            .shortcut(Modifiers::empty(), Key::ArrowDown, || {
                let location = ScrollLocation::Delta(Vector2D::new(0.0, -3.0 * LINE_HEIGHT));
                webview.notify_scroll_event(location, origin, TouchEventType::Move);
            })
            .shortcut(Modifiers::empty(), Key::ArrowLeft, || {
                let location = ScrollLocation::Delta(Vector2D::new(LINE_HEIGHT, 0.0));
                webview.notify_scroll_event(location, origin, TouchEventType::Move);
            })
            .shortcut(Modifiers::empty(), Key::ArrowRight, || {
                let location = ScrollLocation::Delta(Vector2D::new(-LINE_HEIGHT, 0.0));
                webview.notify_scroll_event(location, origin, TouchEventType::Move);
            });
    }
}

#[cfg(target_os = "linux")]
fn prompt_user(prompt: PermissionPrompt) -> PermissionRequest {
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

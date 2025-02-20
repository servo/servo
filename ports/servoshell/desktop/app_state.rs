/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::{Ref, RefCell, RefMut};
use std::collections::HashMap;
use std::path::PathBuf;
use std::rc::Rc;
use std::thread;

use euclid::Vector2D;
use keyboard_types::{Key, KeyboardEvent, Modifiers, ShortcutMatcher};
use log::{error, info};
use servo::base::id::WebViewId;
use servo::config::pref;
use servo::ipc_channel::ipc::IpcSender;
use servo::webrender_api::units::{DeviceIntPoint, DeviceIntSize};
use servo::webrender_api::ScrollLocation;
use servo::{
    AllowOrDenyRequest, AuthenticationRequest, FilterPattern, GamepadHapticEffectType, LoadStatus,
    PermissionRequest, PromptDefinition, PromptOrigin, PromptResult, Servo, ServoDelegate,
    ServoError, TouchEventType, WebView, WebViewDelegate,
};
#[cfg(target_os = "linux")]
use tinyfiledialogs::MessageBoxIcon;
use url::Url;

use super::app::PumpResult;
use super::dialog::Dialog;
use super::gamepad::GamepadSupport;
use super::keyutils::CMD_OR_CONTROL;
use super::window_trait::{WindowPortsMethods, LINE_HEIGHT};

pub(crate) enum AppState {
    Initializing,
    Running(Rc<RunningAppState>),
    ShuttingDown,
}

pub(crate) struct RunningAppState {
    /// A handle to the Servo instance of the [`RunningAppState`]. This is not stored inside
    /// `inner` so that we can keep a reference to Servo in order to spin the event loop,
    /// which will in turn call delegates doing a mutable borrow on `inner`.
    servo: Servo,
    inner: RefCell<RunningAppStateInner>,
}

pub struct RunningAppStateInner {
    /// Whether or not this is a headless servoshell window.
    headless: bool,

    /// List of top-level browsing contexts.
    /// Modified by EmbedderMsg::WebViewOpened and EmbedderMsg::WebViewClosed,
    /// and we exit if it ever becomes empty.
    webviews: HashMap<WebViewId, WebView>,

    /// The order in which the webviews were created.
    creation_order: Vec<WebViewId>,

    /// The webview that is currently focused.
    /// Modified by EmbedderMsg::WebViewFocused and EmbedderMsg::WebViewBlurred.
    focused_webview_id: Option<WebViewId>,

    /// The current set of open dialogs.
    dialogs: HashMap<WebViewId, Vec<Dialog>>,

    /// A handle to the Window that Servo is rendering in -- either headed or headless.
    window: Rc<dyn WindowPortsMethods>,

    /// Gamepad support, which may be `None` if it failed to initialize.
    gamepad_support: Option<GamepadSupport>,

    /// Whether or not the application interface needs to be updated.
    need_update: bool,

    /// Whether or not Servo needs to repaint its display. Currently this is global
    /// because every `WebView` shares a `RenderingContext`.
    need_repaint: bool,
}

impl Drop for RunningAppState {
    fn drop(&mut self) {
        self.servo.deinit();
    }
}

impl RunningAppState {
    pub fn new(
        servo: Servo,
        window: Rc<dyn WindowPortsMethods>,
        headless: bool,
    ) -> RunningAppState {
        servo.set_delegate(Rc::new(ServoShellServoDelegate));
        RunningAppState {
            servo,
            inner: RefCell::new(RunningAppStateInner {
                headless,
                webviews: HashMap::default(),
                creation_order: Default::default(),
                focused_webview_id: None,
                dialogs: Default::default(),
                window,
                gamepad_support: GamepadSupport::maybe_new(),
                need_update: false,
                need_repaint: false,
            }),
        }
    }

    pub(crate) fn new_toplevel_webview(self: &Rc<Self>, url: Url) {
        let webview = self.servo().new_webview(url);
        webview.set_delegate(self.clone());
        self.add(webview);
    }

    pub(crate) fn inner(&self) -> Ref<RunningAppStateInner> {
        self.inner.borrow()
    }

    pub(crate) fn inner_mut(&self) -> RefMut<RunningAppStateInner> {
        self.inner.borrow_mut()
    }

    pub(crate) fn servo(&self) -> &Servo {
        &self.servo
    }

    pub(crate) fn repaint_servo_if_necessary(&self) {
        if !self.inner().need_repaint {
            return;
        }
        let Some(webview) = self.focused_webview() else {
            return;
        };

        webview.paint();
        self.inner().window.rendering_context().present();
        self.inner_mut().need_repaint = false;
    }

    /// Spins the internal application event loop.
    ///
    /// - Notifies Servo about incoming gamepad events
    /// - Spin the Servo event loop, which will run the compositor and trigger delegate methods.
    pub(crate) fn pump_event_loop(&self) -> PumpResult {
        if pref!(dom_gamepad_enabled) {
            self.handle_gamepad_events();
        }

        if !self.servo().spin_event_loop() {
            return PumpResult::Shutdown;
        }

        // Delegate handlers may have asked us to present or update compositor contents.
        // Currently, egui-file-dialog dialogs need to be constantly redrawn or animations aren't fluid.
        let need_window_redraw = self.inner().need_repaint || self.has_active_dialog();
        let need_update = std::mem::replace(&mut self.inner_mut().need_update, false);

        PumpResult::Continue {
            need_update,
            need_window_redraw,
        }
    }

    pub(crate) fn add(&self, webview: WebView) {
        self.inner_mut().creation_order.push(webview.id());
        self.inner_mut().webviews.insert(webview.id(), webview);
    }

    pub(crate) fn shutdown(&self) {
        self.inner_mut().webviews.clear();
    }

    pub(crate) fn for_each_active_dialog(&self, callback: impl Fn(&mut Dialog) -> bool) {
        let last_created_webview_id = self.inner().creation_order.last().cloned();
        let Some(webview_id) = self
            .focused_webview()
            .as_ref()
            .map(WebView::id)
            .or(last_created_webview_id)
        else {
            return;
        };

        if let Some(dialogs) = self.inner_mut().dialogs.get_mut(&webview_id) {
            dialogs.retain_mut(callback);
        }
    }

    pub fn close_webview(&self, webview_id: WebViewId) {
        // This can happen because we can trigger a close with a UI action and then get the
        // close event from Servo later.
        let mut inner = self.inner_mut();
        if !inner.webviews.contains_key(&webview_id) {
            return;
        }

        inner.webviews.retain(|&id, _| id != webview_id);
        inner.creation_order.retain(|&id| id != webview_id);
        inner.focused_webview_id = None;
        inner.dialogs.remove(&webview_id);

        let last_created = inner
            .creation_order
            .last()
            .and_then(|id| inner.webviews.get(id));

        match last_created {
            Some(last_created_webview) => last_created_webview.focus(),
            None => self.servo.start_shutting_down(),
        }
    }

    pub fn focused_webview(&self) -> Option<WebView> {
        self.inner()
            .focused_webview_id
            .and_then(|id| self.inner().webviews.get(&id).cloned())
    }

    // Returns the webviews in the creation order.
    pub fn webviews(&self) -> Vec<(WebViewId, WebView)> {
        let inner = self.inner();
        inner
            .creation_order
            .iter()
            .map(|id| (*id, inner.webviews.get(id).unwrap().clone()))
            .collect()
    }

    pub fn handle_gamepad_events(&self) {
        let Some(active_webview) = self.focused_webview() else {
            return;
        };
        if let Some(gamepad_support) = self.inner_mut().gamepad_support.as_mut() {
            gamepad_support.handle_gamepad_events(active_webview);
        }
    }

    pub(crate) fn focus_webview_by_index(&self, index: usize) {
        if let Some((_, webview)) = self.webviews().get(index) {
            webview.focus();
        }
    }

    fn add_dialog(&self, webview: servo::WebView, dialog: Dialog) {
        let mut inner_mut = self.inner_mut();
        inner_mut
            .dialogs
            .entry(webview.id())
            .or_default()
            .push(dialog);
        inner_mut.need_update = true;
    }

    fn has_active_dialog(&self) -> bool {
        let Some(webview) = self.focused_webview() else {
            return false;
        };
        let inner = self.inner();
        let Some(dialogs) = inner.dialogs.get(&webview.id()) else {
            return false;
        };
        !dialogs.is_empty()
    }

    pub(crate) fn get_focused_webview_index(&self) -> Option<usize> {
        let focused_id = self.inner().focused_webview_id?;
        self.webviews()
            .iter()
            .position(|webview| webview.0 == focused_id)
    }

    /// Handle servoshell key bindings that may have been prevented by the page in the focused webview.
    fn handle_overridable_key_bindings(&self, webview: ::servo::WebView, event: KeyboardEvent) {
        let origin = webview.rect().min.ceil().to_i32();
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
                    -self.inner().window.page_height() + 2.0 * LINE_HEIGHT,
                ));
                webview.notify_scroll_event(scroll_location, origin, TouchEventType::Move);
            })
            .shortcut(Modifiers::empty(), Key::PageUp, || {
                let scroll_location = ScrollLocation::Delta(Vector2D::new(
                    0.0,
                    self.inner().window.page_height() - 2.0 * LINE_HEIGHT,
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

struct ServoShellServoDelegate;
impl ServoDelegate for ServoShellServoDelegate {
    fn notify_devtools_server_started(&self, _servo: &Servo, port: u16, _token: String) {
        info!("Devtools Server running on port {port}");
    }

    fn request_devtools_connection(&self, _servo: &Servo, request: AllowOrDenyRequest) {
        request.allow();
    }

    fn notify_error(&self, _servo: &Servo, error: ServoError) {
        error!("Saw Servo error: {error:?}!");
    }
}

impl WebViewDelegate for RunningAppState {
    fn notify_status_text_changed(&self, _webview: servo::WebView, _status: Option<String>) {
        self.inner_mut().need_update = true;
    }

    fn notify_page_title_changed(&self, webview: servo::WebView, title: Option<String>) {
        if webview.focused() {
            let window_title = format!("{} - Servo", title.clone().unwrap_or_default());
            self.inner().window.set_title(&window_title);
            self.inner_mut().need_update = true;
        }
    }

    fn request_move_to(&self, _: servo::WebView, new_position: DeviceIntPoint) {
        self.inner().window.set_position(new_position);
    }

    fn request_resize_to(&self, webview: servo::WebView, new_size: DeviceIntSize) {
        let mut rect = webview.rect();
        rect.set_size(new_size.to_f32());
        webview.move_resize(rect);
        self.inner().window.request_resize(&webview, new_size);
    }

    fn show_prompt(
        &self,
        webview: servo::WebView,
        definition: PromptDefinition,
        _origin: PromptOrigin,
    ) {
        if self.inner().headless {
            let _ = match definition {
                PromptDefinition::Alert(_message, sender) => sender.send(()),
                PromptDefinition::OkCancel(_message, sender) => sender.send(PromptResult::Primary),
                PromptDefinition::Input(_message, default, sender) => {
                    sender.send(Some(default.to_owned()))
                },
            };
            return;
        }
        match definition {
            PromptDefinition::Alert(message, sender) => {
                let alert_dialog = Dialog::new_alert_dialog(message, sender);
                self.add_dialog(webview, alert_dialog);
            },
            PromptDefinition::OkCancel(message, sender) => {
                let okcancel_dialog = Dialog::new_okcancel_dialog(message, sender);
                self.add_dialog(webview, okcancel_dialog);
            },
            PromptDefinition::Input(message, default, sender) => {
                let input_dialog = Dialog::new_input_dialog(message, default, sender);
                self.add_dialog(webview, input_dialog);
            },
        }
    }

    fn request_authentication(
        &self,
        webview: WebView,
        authentication_request: AuthenticationRequest,
    ) {
        if self.inner().headless {
            return;
        }

        self.add_dialog(
            webview,
            Dialog::new_authentication_dialog(authentication_request),
        );
    }

    fn request_open_auxiliary_webview(
        &self,
        parent_webview: servo::WebView,
    ) -> Option<servo::WebView> {
        let webview = self.servo.new_auxiliary_webview();
        webview.set_delegate(parent_webview.delegate());
        self.add(webview.clone());
        Some(webview)
    }

    fn notify_ready_to_show(&self, webview: servo::WebView) {
        let rect = self
            .inner()
            .window
            .get_coordinates()
            .get_viewport()
            .to_f32();

        webview.focus();
        webview.move_resize(rect);
        webview.raise_to_top(true);
        webview.notify_rendering_context_resized();
    }

    fn notify_closed(&self, webview: servo::WebView) {
        self.close_webview(webview.id());
    }

    fn notify_focus_changed(&self, webview: servo::WebView, focused: bool) {
        let mut inner_mut = self.inner_mut();
        if focused {
            webview.show(true);
            inner_mut.need_update = true;
            inner_mut.focused_webview_id = Some(webview.id());
        } else if inner_mut.focused_webview_id == Some(webview.id()) {
            inner_mut.focused_webview_id = None;
        }
    }

    fn notify_keyboard_event(&self, webview: servo::WebView, keyboard_event: KeyboardEvent) {
        self.handle_overridable_key_bindings(webview, keyboard_event);
    }

    fn notify_cursor_changed(&self, _webview: servo::WebView, cursor: servo::Cursor) {
        self.inner().window.set_cursor(cursor);
    }

    fn notify_load_status_changed(&self, _webview: servo::WebView, _status: LoadStatus) {
        self.inner_mut().need_update = true;
    }

    fn notify_fullscreen_state_changed(&self, _webview: servo::WebView, fullscreen_state: bool) {
        self.inner().window.set_fullscreen(fullscreen_state);
    }

    fn show_bluetooth_device_dialog(
        &self,
        webview: servo::WebView,
        devices: Vec<String>,
        response_sender: IpcSender<Option<String>>,
    ) {
        let selected = platform_get_selected_devices(devices);
        if let Err(e) = response_sender.send(selected) {
            webview.send_error(format!(
                "Failed to send GetSelectedBluetoothDevice response: {e}"
            ));
        }
    }

    fn show_file_selection_dialog(
        &self,
        webview: servo::WebView,
        filter_pattern: Vec<FilterPattern>,
        allow_select_mutiple: bool,
        response_sender: IpcSender<Option<Vec<PathBuf>>>,
    ) {
        let file_dialog =
            Dialog::new_file_dialog(allow_select_mutiple, response_sender, filter_pattern);
        self.add_dialog(webview, file_dialog);
    }

    fn request_permission(&self, _webview: servo::WebView, request: PermissionRequest) {
        if !self.inner().headless {
            prompt_user(request);
        }
    }

    fn notify_new_frame_ready(&self, _webview: servo::WebView) {
        self.inner_mut().need_repaint = true;
    }

    fn play_gamepad_haptic_effect(
        &self,
        _webview: servo::WebView,
        index: usize,
        effect_type: GamepadHapticEffectType,
        effect_complete_sender: IpcSender<bool>,
    ) {
        match self.inner_mut().gamepad_support.as_mut() {
            Some(gamepad_support) => {
                gamepad_support.play_haptic_effect(index, effect_type, effect_complete_sender);
            },
            None => {
                let _ = effect_complete_sender.send(false);
            },
        }
    }

    fn stop_gamepad_haptic_effect(
        &self,
        _webview: servo::WebView,
        index: usize,
        haptic_stop_sender: IpcSender<bool>,
    ) {
        let stopped = match self.inner_mut().gamepad_support.as_mut() {
            Some(gamepad_support) => gamepad_support.stop_haptic_effect(index),
            None => false,
        };
        let _ = haptic_stop_sender.send(stopped);
    }
    fn show_ime(
        &self,
        _webview: WebView,
        input_type: servo::InputMethodType,
        text: Option<(String, i32)>,
        multiline: bool,
        position: servo::webrender_api::units::DeviceIntRect,
    ) {
        self.inner()
            .window
            .show_ime(input_type, text, multiline, position);
    }

    fn hide_ime(&self, _webview: WebView) {
        self.inner().window.hide_ime();
    }
}

#[cfg(target_os = "linux")]
fn prompt_user(request: PermissionRequest) {
    use tinyfiledialogs::YesNo;

    let message = format!(
        "Do you want to grant permission for {:?}?",
        request.feature()
    );
    match tinyfiledialogs::message_box_yes_no(
        "Permission request dialog",
        &message,
        MessageBoxIcon::Question,
        YesNo::No,
    ) {
        YesNo::Yes => request.allow(),
        YesNo::No => request.deny(),
    }
}

#[cfg(not(target_os = "linux"))]
fn prompt_user(_request: PermissionRequest) {
    // Requests are denied by default.
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

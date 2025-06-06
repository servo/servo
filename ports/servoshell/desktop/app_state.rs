/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::{Ref, RefCell, RefMut};
use std::collections::HashMap;
use std::path::PathBuf;
use std::rc::Rc;

use euclid::Vector2D;
use keyboard_types::{Key, KeyboardEvent, Modifiers, ShortcutMatcher};
use log::{error, info};
use servo::base::id::WebViewId;
use servo::config::pref;
use servo::ipc_channel::ipc::IpcSender;
use servo::webrender_api::ScrollLocation;
use servo::webrender_api::units::{DeviceIntPoint, DeviceIntSize};
use servo::{
    AllowOrDenyRequest, AuthenticationRequest, FilterPattern, FormControl, GamepadHapticEffectType,
    LoadStatus, PermissionRequest, Servo, ServoDelegate, ServoError, SimpleDialog, TouchEventType,
    WebView, WebViewBuilder, WebViewDelegate,
};
use url::Url;

use super::app::PumpResult;
use super::dialog::Dialog;
use super::gamepad::GamepadSupport;
use super::keyutils::CMD_OR_CONTROL;
use super::window_trait::{LINE_HEIGHT, WindowPortsMethods};
use crate::output_image::save_output_image_if_necessary;
use crate::prefs::ServoShellPreferences;

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
    /// The preferences for this run of servoshell. This is not mutable, so doesn't need to
    /// be stored inside the [`RunningAppStateInner`].
    servoshell_preferences: ServoShellPreferences,
    inner: RefCell<RunningAppStateInner>,
}

pub struct RunningAppStateInner {
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
        servoshell_preferences: ServoShellPreferences,
    ) -> RunningAppState {
        servo.set_delegate(Rc::new(ServoShellServoDelegate));
        RunningAppState {
            servo,
            servoshell_preferences,
            inner: RefCell::new(RunningAppStateInner {
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
        let webview = WebViewBuilder::new(self.servo())
            .url(url)
            .hidpi_scale_factor(self.inner().window.hidpi_scale_factor())
            .delegate(self.clone())
            .build();

        webview.notify_theme_change(self.inner().window.theme());
        webview.focus();
        webview.raise_to_top(true);

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

    pub(crate) fn hidpi_scale_factor_changed(&self) {
        let inner = self.inner();
        let new_scale_factor = inner.window.hidpi_scale_factor();
        for webview in inner.webviews.values() {
            webview.set_hidpi_scale_factor(new_scale_factor);
        }
    }

    /// Repaint the Servo view is necessary, returning true if anything was actually
    /// painted or false otherwise. Something may not be painted if Servo is waiting
    /// for a stable image to paint.
    pub(crate) fn repaint_servo_if_necessary(&self) {
        if !self.inner().need_repaint {
            return;
        }
        let Some(webview) = self.focused_webview() else {
            return;
        };
        if !webview.paint() {
            return;
        }

        save_output_image_if_necessary(
            &self.servoshell_preferences,
            &self.inner().window.rendering_context(),
        );

        let mut inner_mut = self.inner_mut();
        inner_mut.window.rendering_context().present();
        inner_mut.need_repaint = false;

        if self.servoshell_preferences.exit_after_stable_image {
            self.servo().start_shutting_down();
        }
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
        inner.dialogs.remove(&webview_id);
        if Some(webview_id) == inner.focused_webview_id {
            inner.focused_webview_id = None;
        }

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

    pub(crate) fn has_active_dialog(&self) -> bool {
        let last_created_webview_id = self.inner().creation_order.last().cloned();
        let Some(webview_id) = self
            .focused_webview()
            .as_ref()
            .map(WebView::id)
            .or(last_created_webview_id)
        else {
            return false;
        };

        let inner = self.inner();
        inner
            .dialogs
            .get(&webview_id)
            .is_some_and(|dialogs| !dialogs.is_empty())
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
    fn screen_geometry(&self, _webview: WebView) -> Option<servo::ScreenGeometry> {
        Some(self.inner().window.screen_geometry())
    }

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

    fn show_simple_dialog(&self, webview: servo::WebView, dialog: SimpleDialog) {
        if self.servoshell_preferences.headless {
            // TODO: Avoid copying this from the default trait impl?
            // Return the DOM-specified default value for when we **cannot show simple dialogs**.
            let _ = match dialog {
                SimpleDialog::Alert {
                    response_sender, ..
                } => response_sender.send(Default::default()),
                SimpleDialog::Confirm {
                    response_sender, ..
                } => response_sender.send(Default::default()),
                SimpleDialog::Prompt {
                    response_sender, ..
                } => response_sender.send(Default::default()),
            };
            return;
        }
        let dialog = Dialog::new_simple_dialog(dialog);
        self.add_dialog(webview, dialog);
    }

    fn request_authentication(
        &self,
        webview: WebView,
        authentication_request: AuthenticationRequest,
    ) {
        if self.servoshell_preferences.headless {
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
        let webview = WebViewBuilder::new_auxiliary(&self.servo)
            .hidpi_scale_factor(self.inner().window.hidpi_scale_factor())
            .delegate(parent_webview.delegate())
            .build();

        webview.notify_theme_change(self.inner().window.theme());
        webview.focus();
        webview.raise_to_top(true);

        self.add(webview.clone());
        Some(webview)
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
        self.add_dialog(
            webview,
            Dialog::new_device_selection_dialog(devices, response_sender),
        );
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

    fn request_permission(&self, webview: servo::WebView, permission_request: PermissionRequest) {
        if self.servoshell_preferences.headless {
            permission_request.deny();
            return;
        }

        let permission_dialog = Dialog::new_permission_request_dialog(permission_request);
        self.add_dialog(webview, permission_dialog);
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

    fn show_form_control(&self, webview: WebView, form_control: FormControl) {
        if self.servoshell_preferences.headless {
            return;
        }

        match form_control {
            FormControl::SelectElement(prompt) => {
                // FIXME: Reading the toolbar height is needed here to properly position the select dialog.
                // But if the toolbar height changes while the dialog is open then the position won't be updated
                let offset = self.inner().window.toolbar_height();
                self.add_dialog(webview, Dialog::new_select_element_dialog(prompt, offset));
            },
            FormControl::ColorPicker(color_picker) => {
                // FIXME: Reading the toolbar height is needed here to properly position the select dialog.
                // But if the toolbar height changes while the dialog is open then the position won't be updated
                let offset = self.inner().window.toolbar_height();
                self.add_dialog(
                    webview,
                    Dialog::new_color_picker_dialog(color_picker, offset),
                );
            },
        }
    }
}

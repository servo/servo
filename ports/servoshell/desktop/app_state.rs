/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::{Cell, Ref, RefCell, RefMut};
use std::collections::HashMap;
use std::collections::hash_map::Entry;
use std::mem;
use std::rc::Rc;

use crossbeam_channel::{Receiver, Sender};
use image::{DynamicImage, ImageFormat};
use log::{error, info};
use servo::base::generic_channel::GenericSender;
use servo::base::id::WebViewId;
use servo::config::pref;
use servo::ipc_channel::ipc::IpcSender;
use servo::webrender_api::units::{DeviceIntPoint, DeviceIntSize};
use servo::{
    AllowOrDenyRequest, AuthenticationRequest, EmbedderControl, EmbedderControlId,
    GamepadHapticEffectType, InputEvent, InputEventId, InputEventResult, JSValue, LoadStatus,
    PermissionRequest, Servo, ServoDelegate, ServoError, SimpleDialog, TraversalId,
    WebDriverCommandMsg, WebDriverJSResult, WebDriverLoadStatus, WebDriverSenders,
    WebDriverUserPrompt, WebView, WebViewBuilder, WebViewDelegate,
};
use url::Url;

use super::app::PumpResult;
use super::dialog::Dialog;
use super::gamepad::GamepadSupport;
use super::window_trait::WindowPortsMethods;
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
    /// A [`Receiver`] for receiving commands from a running WebDriver server, if WebDriver
    /// was enabled.
    webdriver_receiver: Option<Receiver<WebDriverCommandMsg>>,
    webdriver_senders: RefCell<WebDriverSenders>,
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

    /// Whether or not the amount of dialogs on the currently rendered webview
    /// has just changed.
    dialog_amount_changed: bool,

    /// List of webviews that have favicon textures which are not yet uploaded
    /// to the GPU by egui.
    pending_favicon_loads: Vec<WebViewId>,

    /// Whether or not the application has achieved stable image output. This is used
    /// for the `exit_after_stable_image` option.
    achieved_stable_image: Rc<Cell<bool>>,

    /// A [`HashMap`] of pending WebDriver events. It is the WebDriver embedder's responsibility
    /// to inform the WebDriver server when the event has been fully handled. This map is used
    /// to report back to WebDriver when that happens.
    pending_webdriver_events: HashMap<InputEventId, Sender<()>>,

    /// A list of showing [`InputMethod`] interfaces.
    visible_input_methods: Vec<EmbedderControlId>,
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
        webdriver_receiver: Option<Receiver<WebDriverCommandMsg>>,
    ) -> RunningAppState {
        servo.set_delegate(Rc::new(ServoShellServoDelegate));
        let gamepad_support = if pref!(dom_gamepad_enabled) {
            GamepadSupport::maybe_new()
        } else {
            None
        };
        RunningAppState {
            servo,
            servoshell_preferences,
            webdriver_receiver,
            webdriver_senders: RefCell::default(),
            inner: RefCell::new(RunningAppStateInner {
                webviews: HashMap::default(),
                creation_order: Default::default(),
                focused_webview_id: None,
                dialogs: Default::default(),
                window,
                gamepad_support,
                need_update: false,
                need_repaint: false,
                dialog_amount_changed: false,
                pending_favicon_loads: Default::default(),
                achieved_stable_image: Default::default(),
                pending_webdriver_events: Default::default(),
                visible_input_methods: Default::default(),
            }),
        }
    }

    pub(crate) fn create_and_focus_toplevel_webview(self: &Rc<Self>, url: Url) {
        let webview = self.create_toplevel_webview(url);
        webview.focus_and_raise_to_top(true);
    }

    pub(crate) fn create_toplevel_webview(self: &Rc<Self>, url: Url) -> WebView {
        let webview = WebViewBuilder::new(self.servo())
            .url(url)
            .hidpi_scale_factor(self.inner().window.hidpi_scale_factor())
            .delegate(self.clone())
            .build();

        webview.notify_theme_change(self.inner().window.theme());
        self.add(webview.clone());
        webview
    }

    pub(crate) fn inner(&self) -> Ref<'_, RunningAppStateInner> {
        self.inner.borrow()
    }

    pub(crate) fn inner_mut(&self) -> RefMut<'_, RunningAppStateInner> {
        self.inner.borrow_mut()
    }

    pub(crate) fn servo(&self) -> &Servo {
        &self.servo
    }

    pub(crate) fn webdriver_receiver(&self) -> Option<&Receiver<WebDriverCommandMsg>> {
        self.webdriver_receiver.as_ref()
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

        webview.paint();

        let mut inner_mut = self.inner_mut();
        inner_mut.window.rendering_context().present();
        inner_mut.need_repaint = false;
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
        let need_window_redraw = self.inner().need_repaint ||
            self.has_active_dialog() ||
            self.inner().dialog_amount_changed;
        let need_update = std::mem::replace(&mut self.inner_mut().need_update, false);

        self.inner_mut().dialog_amount_changed = false;

        if self.servoshell_preferences.exit_after_stable_image &&
            self.inner().achieved_stable_image.get()
        {
            self.servo.start_shutting_down();
        }

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

        let mut inner = self.inner_mut();
        if let Some(dialogs) = inner.dialogs.get_mut(&webview_id) {
            let length = dialogs.len();
            dialogs.retain_mut(callback);
            if length != dialogs.len() {
                inner.dialog_amount_changed = true;
            }
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
            Some(last_created_webview) => {
                last_created_webview.focus();
            },
            None if self.servoshell_preferences.webdriver_port.is_none() => {
                self.servo.start_shutting_down()
            },
            None => {
                // For WebDriver, don't shut down when last webview closed
                // https://github.com/servo/servo/issues/37408
            },
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

    pub fn webview_by_id(&self, id: WebViewId) -> Option<WebView> {
        self.inner().webviews.get(&id).cloned()
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

    pub(crate) fn webview_has_active_dialog(&self, webview_id: WebViewId) -> bool {
        self.inner()
            .dialogs
            .get(&webview_id)
            .is_some_and(|dialogs| !dialogs.is_empty())
    }

    pub(crate) fn get_current_active_dialog_webdriver_type(
        &self,
        webview_id: WebViewId,
    ) -> Option<WebDriverUserPrompt> {
        self.inner()
            .dialogs
            .get(&webview_id)
            .and_then(|dialogs| dialogs.last())
            .map(|dialog| dialog.webdriver_diaglog_type())
    }

    pub(crate) fn accept_active_dialogs(&self, webview_id: WebViewId) {
        if let Some(dialogs) = self.inner_mut().dialogs.get_mut(&webview_id) {
            dialogs.drain(..).for_each(|dialog| {
                dialog.accept();
            });
        }
    }

    pub(crate) fn dismiss_active_dialogs(&self, webview_id: WebViewId) {
        if let Some(dialogs) = self.inner_mut().dialogs.get_mut(&webview_id) {
            dialogs.drain(..).for_each(|dialog| {
                dialog.dismiss();
            });
        }
    }

    pub(crate) fn alert_text_of_newest_dialog(&self, webview_id: WebViewId) -> Option<String> {
        self.inner()
            .dialogs
            .get(&webview_id)
            .and_then(|dialogs| dialogs.last())
            .and_then(|dialog| dialog.message())
    }

    pub(crate) fn set_alert_text_of_newest_dialog(&self, webview_id: WebViewId, text: String) {
        if let Some(dialogs) = self.inner_mut().dialogs.get_mut(&webview_id) {
            if let Some(dialog) = dialogs.last_mut() {
                dialog.set_message(text);
            }
        }
    }

    pub(crate) fn get_focused_webview_index(&self) -> Option<usize> {
        let focused_id = self.inner().focused_webview_id?;
        self.webviews()
            .iter()
            .position(|webview| webview.0 == focused_id)
    }

    pub(crate) fn set_pending_traversal(
        &self,
        traversal_id: TraversalId,
        sender: GenericSender<WebDriverLoadStatus>,
    ) {
        self.webdriver_senders
            .borrow_mut()
            .pending_traversals
            .insert(traversal_id, sender);
    }

    pub(crate) fn set_load_status_sender(
        &self,
        webview_id: WebViewId,
        sender: GenericSender<WebDriverLoadStatus>,
    ) {
        self.webdriver_senders
            .borrow_mut()
            .load_status_senders
            .insert(webview_id, sender);
    }

    pub(crate) fn set_script_command_interrupt_sender(
        &self,
        sender: Option<IpcSender<WebDriverJSResult>>,
    ) {
        self.webdriver_senders
            .borrow_mut()
            .script_evaluation_interrupt_sender = sender;
    }

    /// Interrupt any ongoing WebDriver-based script evaluation.
    ///
    /// From <https://w3c.github.io/webdriver/#dfn-execute-a-function-body>:
    /// > The rules to execute a function body are as follows. The algorithm returns
    /// > an ECMAScript completion record.
    /// >
    /// > If at any point during the algorithm a user prompt appears, immediately return
    /// > Completion { Type: normal, Value: null, Target: empty }, but continue to run the
    /// >  other steps of this algorithm in parallel.
    fn interrupt_webdriver_script_evaluation(&self) {
        if let Some(sender) = &self
            .webdriver_senders
            .borrow()
            .script_evaluation_interrupt_sender
        {
            sender.send(Ok(JSValue::Null)).unwrap_or_else(|err| {
                info!(
                    "Notify dialog appear failed. Maybe the channel to webdriver is closed: {err}"
                );
            });
        }
    }

    pub(crate) fn remove_load_status_sender(&self, webview_id: WebViewId) {
        self.webdriver_senders
            .borrow_mut()
            .load_status_senders
            .remove(&webview_id);
    }

    /// Return a list of all webviews that have favicons that have not yet been loaded by egui.
    pub(crate) fn take_pending_favicon_loads(&self) -> Vec<WebViewId> {
        mem::take(&mut self.inner_mut().pending_favicon_loads)
    }

    /// If we are exiting after achieving a stable image or we want to save the display of the
    /// [`WebView`] to an image file, request a screenshot of the [`WebView`].
    fn maybe_request_screenshot(&self, webview: WebView) {
        let output_path = self.servoshell_preferences.output_image_path.clone();
        if !self.servoshell_preferences.exit_after_stable_image && output_path.is_none() {
            return;
        }

        // Never request more than a single screenshot for now.
        let achieved_stable_image = self.inner().achieved_stable_image.clone();
        if achieved_stable_image.get() {
            return;
        }

        webview.take_screenshot(None, move |image| {
            achieved_stable_image.set(true);

            let Some(output_path) = output_path else {
                return;
            };

            let image = match image {
                Ok(image) => image,
                Err(error) => {
                    error!("Could not take screenshot: {error:?}");
                    return;
                },
            };

            let image_format = ImageFormat::from_path(&output_path).unwrap_or(ImageFormat::Png);
            if let Err(error) =
                DynamicImage::ImageRgba8(image).save_with_format(output_path, image_format)
            {
                error!("Failed to save screenshot: {error}.");
            }
        });
    }

    pub(crate) fn handle_webdriver_input_event(
        &self,
        webview_id: WebViewId,
        input_event: InputEvent,
        response_sender: Option<Sender<()>>,
    ) {
        let Some(webview) = self.webview_by_id(webview_id) else {
            error!("Could not find WebView ({webview_id:?}) for WebDriver event: {input_event:?}");
            return;
        };

        let event_id = webview.notify_input_event(input_event);

        if let Some(response_sender) = response_sender {
            self.inner_mut()
                .pending_webdriver_events
                .insert(event_id, response_sender);
        }
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

    fn notify_history_changed(&self, _webview: WebView, _entries: Vec<Url>, _current: usize) {
        self.inner_mut().need_update = true;
    }

    fn notify_page_title_changed(&self, webview: servo::WebView, title: Option<String>) {
        if webview.focused() {
            let window_title = format!("{} - Servo", title.clone().unwrap_or_default());
            self.inner().window.set_title(&window_title);
            self.inner_mut().need_update = true;
        }
    }

    fn notify_traversal_complete(&self, _webview: servo::WebView, traversal_id: TraversalId) {
        let mut webdriver_state = self.webdriver_senders.borrow_mut();
        if let Entry::Occupied(entry) = webdriver_state.pending_traversals.entry(traversal_id) {
            let sender = entry.remove();
            let _ = sender.send(WebDriverLoadStatus::Complete);
        }
    }

    fn request_move_to(&self, _: servo::WebView, new_position: DeviceIntPoint) {
        self.inner().window.set_position(new_position);
    }

    fn request_resize_to(&self, webview: servo::WebView, requested_outer_size: DeviceIntSize) {
        // We need to update compositor's view later as we not sure about resizing result.
        self.inner()
            .window
            .request_resize(&webview, requested_outer_size);
    }

    fn show_simple_dialog(&self, webview: servo::WebView, dialog: SimpleDialog) {
        self.interrupt_webdriver_script_evaluation();

        // Dialogs block the page load, so need need to notify WebDriver
        let webview_id = webview.id();
        if let Some(sender) = self
            .webdriver_senders
            .borrow_mut()
            .load_status_senders
            .get(&webview_id)
        {
            let _ = sender.send(WebDriverLoadStatus::Blocked);
        };

        if self.servoshell_preferences.headless &&
            self.servoshell_preferences.webdriver_port.is_none()
        {
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
        if self.servoshell_preferences.headless &&
            self.servoshell_preferences.webdriver_port.is_none()
        {
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
        // When WebDriver is enabled, do not focus and raise the WebView to the top,
        // as that is what the specification expects. Otherwise, we would like `window.open()`
        // to create a new foreground tab
        if self.servoshell_preferences.webdriver_port.is_none() {
            webview.focus_and_raise_to_top(true);
        }
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

    fn notify_input_event_handled(
        &self,
        webview: WebView,
        id: InputEventId,
        result: InputEventResult,
    ) {
        self.inner()
            .window
            .notify_input_event_handled(&webview, id, result);

        if let Some(response_sender) = self.inner_mut().pending_webdriver_events.remove(&id) {
            let _ = response_sender.send(());
        }
    }

    fn notify_cursor_changed(&self, _webview: servo::WebView, cursor: servo::Cursor) {
        self.inner().window.set_cursor(cursor);
    }

    fn notify_load_status_changed(&self, webview: servo::WebView, status: LoadStatus) {
        self.inner_mut().need_update = true;

        if status == LoadStatus::Complete {
            if let Some(sender) = self
                .webdriver_senders
                .borrow_mut()
                .load_status_senders
                .remove(&webview.id())
            {
                let _ = sender.send(WebDriverLoadStatus::Complete);
            }
            self.maybe_request_screenshot(webview);
        }
    }

    fn notify_fullscreen_state_changed(&self, _webview: servo::WebView, fullscreen_state: bool) {
        self.inner().window.set_fullscreen(fullscreen_state);
    }

    fn show_bluetooth_device_dialog(
        &self,
        webview: servo::WebView,
        devices: Vec<String>,
        response_sender: GenericSender<Option<String>>,
    ) {
        self.add_dialog(
            webview,
            Dialog::new_device_selection_dialog(devices, response_sender),
        );
    }

    fn request_permission(&self, webview: servo::WebView, permission_request: PermissionRequest) {
        if self.servoshell_preferences.headless &&
            self.servoshell_preferences.webdriver_port.is_none()
        {
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

    fn show_embedder_control(&self, webview: WebView, embedder_control: EmbedderControl) {
        if self.servoshell_preferences.headless &&
            self.servoshell_preferences.webdriver_port.is_none()
        {
            return;
        }

        let control_id = embedder_control.id();
        match embedder_control {
            EmbedderControl::SelectElement(prompt) => {
                // FIXME: Reading the toolbar height is needed here to properly position the select dialog.
                // But if the toolbar height changes while the dialog is open then the position won't be updated
                let offset = self.inner().window.toolbar_height();
                self.add_dialog(webview, Dialog::new_select_element_dialog(prompt, offset));
            },
            EmbedderControl::ColorPicker(color_picker) => {
                // FIXME: Reading the toolbar height is needed here to properly position the select dialog.
                // But if the toolbar height changes while the dialog is open then the position won't be updated
                let offset = self.inner().window.toolbar_height();
                self.add_dialog(
                    webview,
                    Dialog::new_color_picker_dialog(color_picker, offset),
                );
            },
            EmbedderControl::InputMethod(input_method_control) => {
                self.inner_mut().visible_input_methods.push(control_id);
                self.inner().window.show_ime(input_method_control);
            },
            EmbedderControl::FilePicker(file_picker) => {
                self.add_dialog(webview, Dialog::new_file_dialog(file_picker));
            },
        }
    }

    fn hide_embedder_control(&self, webview: WebView, control_id: EmbedderControlId) {
        {
            let mut inner_mut = self.inner_mut();
            if let Some(index) = inner_mut
                .visible_input_methods
                .iter()
                .position(|visible_id| *visible_id == control_id)
            {
                inner_mut.visible_input_methods.remove(index);
                inner_mut.window.hide_ime();
            }
        }

        if let Some(dialogs) = self.inner_mut().dialogs.get_mut(&webview.id()) {
            dialogs.retain(|dialog| dialog.embedder_control_id() != Some(control_id));
        }
    }

    fn notify_favicon_changed(&self, webview: WebView) {
        let mut inner = self.inner_mut();
        inner.pending_favicon_loads.push(webview.id());
        inner.need_repaint = true;
    }
}

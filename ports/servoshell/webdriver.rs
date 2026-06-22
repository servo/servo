/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use euclid::{Box2D, Point2D};
use log::warn;
use servo::{
    DeviceIndependentIntRect, DeviceIndependentPixel, DeviceIntRect, DeviceIntSize,
    EmbedderControl, EmbedderControlId, GenericCallback, GenericSender, NewWindowTypeHint,
    SimpleDialog, WebDriverCommandMsg, WebDriverDelegate, WebDriverUserPrompt,
    WebDriverUserPromptAction, WebViewId,
};
use url::Url;
use webdriver_traits::bidi::ErrorCode;
use webdriver_traits::bidi::browser::{
    ClientWindowInfoState, ClientWindowNamedStateOrClientWindowRectState,
    ClientWindowNamedStateState,
};
use webdriver_traits::{GetClientWindowResponse, WebDriverToEmbedderMsg, WebViewCreateRequest};

use crate::running_app_state::RunningAppState;
use crate::window::{PlatformWindow, ServoShellWindow, ServoShellWindowId};

#[derive(Default)]
pub(crate) struct WebDriverEmbedderControls {
    embedder_controls: RefCell<HashMap<WebViewId, Vec<EmbedderControl>>>,
}

impl WebDriverEmbedderControls {
    pub(crate) fn show_embedder_control(
        &self,
        webview_id: WebViewId,
        embedder_control: EmbedderControl,
    ) {
        self.embedder_controls
            .borrow_mut()
            .entry(webview_id)
            .or_default()
            .push(embedder_control)
    }

    pub(crate) fn hide_embedder_control(
        &self,
        webview_id: WebViewId,
        embedder_control_id: EmbedderControlId,
    ) {
        let mut embedder_controls = self.embedder_controls.borrow_mut();
        if let Some(controls) = embedder_controls.get_mut(&webview_id) {
            controls.retain(|control| control.id() != embedder_control_id);
        }
        embedder_controls.retain(|_, controls| !controls.is_empty());
    }

    pub(crate) fn current_active_dialog_webdriver_type(
        &self,
        webview_id: WebViewId,
    ) -> Option<WebDriverUserPrompt> {
        // From <https://w3c.github.io/webdriver/#dfn-handle-any-user-prompts>
        // > Step 3: If the current user prompt is an alert dialog, set type to "alert". Otherwise,
        // > if the current user prompt is a beforeunload dialog, set type to
        // > "beforeUnload". Otherwise, if the current user prompt is a confirm dialog, set
        // > type to "confirm". Otherwise, if the current user prompt is a prompt dialog,
        // > set type to "prompt".
        let embedder_controls = self.embedder_controls.borrow();
        match embedder_controls.get(&webview_id)?.last()? {
            EmbedderControl::SimpleDialog(SimpleDialog::Alert(..)) => {
                Some(WebDriverUserPrompt::Alert)
            },
            EmbedderControl::SimpleDialog(SimpleDialog::Confirm(..)) => {
                Some(WebDriverUserPrompt::Confirm)
            },
            EmbedderControl::SimpleDialog(SimpleDialog::Prompt(..)) => {
                Some(WebDriverUserPrompt::Prompt)
            },
            EmbedderControl::FilePicker { .. } => Some(WebDriverUserPrompt::File),
            EmbedderControl::SelectElement { .. } => Some(WebDriverUserPrompt::Default),
            _ => None,
        }
    }

    /// Respond to the most recently added dialog if it was a `SimpleDialog` and return
    /// its message string or return an error if there is no active dialog or the most
    /// recently added dialog is not a `SimpleDialog`.
    pub(crate) fn respond_to_active_simple_dialog(
        &self,
        webview_id: WebViewId,
        action: WebDriverUserPromptAction,
    ) -> Result<String, ()> {
        let mut embedder_controls = self.embedder_controls.borrow_mut();
        let Some(controls) = embedder_controls.get_mut(&webview_id) else {
            return Err(());
        };
        let Some(&EmbedderControl::SimpleDialog(simple_dialog)) = controls.last().as_ref() else {
            return Err(());
        };

        let result_text = simple_dialog.message().to_owned();
        if action == WebDriverUserPromptAction::Ignore {
            return Ok(result_text);
        }

        let Some(EmbedderControl::SimpleDialog(simple_dialog)) = controls.pop() else {
            return Err(());
        };
        match action {
            WebDriverUserPromptAction::Accept => simple_dialog.confirm(),
            WebDriverUserPromptAction::Dismiss => simple_dialog.dismiss(),
            WebDriverUserPromptAction::Ignore => unreachable!("Should have returned early above"),
        }
        Ok(result_text)
    }

    pub(crate) fn message_of_newest_dialog(&self, webview_id: WebViewId) -> Option<String> {
        let embedder_controls = self.embedder_controls.borrow();
        match embedder_controls.get(&webview_id)?.last()? {
            EmbedderControl::SimpleDialog(simple_dialog) => Some(simple_dialog.message().into()),
            _ => None,
        }
    }

    pub(crate) fn set_prompt_value_of_newest_dialog(&self, webview_id: WebViewId, text: String) {
        let mut embedder_controls = self.embedder_controls.borrow_mut();
        let Some(controls) = embedder_controls.get_mut(&webview_id) else {
            return;
        };
        let Some(&mut EmbedderControl::SimpleDialog(SimpleDialog::Prompt(ref mut prompt_dialog))) =
            controls.last_mut()
        else {
            return;
        };
        prompt_dialog.set_current_value(&text);
    }
}

impl RunningAppState {
    pub(crate) fn handle_webdriver_messages(
        self: &Rc<Self>,
        create_platform_window: Option<&dyn Fn(Url) -> Rc<dyn PlatformWindow>>,
    ) {
        let Some(webdriver_receiver) = self.webdriver_receiver() else {
            return;
        };

        while let Ok(msg) = webdriver_receiver.try_recv() {
            match msg {
                WebDriverCommandMsg::ResetAllCookies(sender) => {
                    self.servo().site_data_manager().clear_cookies(None);
                    let _ = sender.send(());
                },
                WebDriverCommandMsg::Shutdown => {
                    self.schedule_exit();
                },
                WebDriverCommandMsg::IsWebViewOpen(webview_id, sender) => {
                    let context = self.webview_by_id(webview_id);

                    if let Err(error) = sender.send(context.is_some()) {
                        warn!("Failed to send response of IsWebViewOpen: {error}");
                    }
                },
                WebDriverCommandMsg::IsBrowsingContextOpen(..) => {
                    self.servo().execute_webdriver_command(msg);
                },
                WebDriverCommandMsg::NewWindow(type_hint, response_sender, load_status_sender) => {
                    let url = Url::parse("about:blank").unwrap();
                    let new_webview = match (type_hint, create_platform_window) {
                        (
                            NewWindowTypeHint::Window | NewWindowTypeHint::Auto,
                            Some(create_platform_window),
                        ) => {
                            let window = self.open_window(create_platform_window(url.clone()), url);
                            window
                                .active_webview()
                                .expect("Should have at last one WebView in new window")
                        },
                        _ => self
                            .windows()
                            .values()
                            .nth(0)
                            .expect("Expected at least one window to be open")
                            .create_toplevel_webview(self.clone(), url),
                    };

                    if let Err(error) = response_sender.send(new_webview.id()) {
                        warn!("Failed to send response of NewWebview: {error}");
                    }
                    if let Some(load_status_sender) = load_status_sender {
                        self.set_load_status_sender(new_webview.id(), load_status_sender);
                    }
                },
                WebDriverCommandMsg::CloseWebView(webview_id, response_sender) => {
                    self.window_for_webview_id(webview_id)
                        .close_webview(webview_id);
                    if let Err(error) = response_sender.send(()) {
                        warn!("Failed to send response of CloseWebView: {error}");
                    }
                },
                WebDriverCommandMsg::FocusWebView(webview_id) => {
                    let window = self.window_for_webview_id(webview_id);
                    window.activate_webview(webview_id);
                    self.focus_window(window);
                },
                WebDriverCommandMsg::FocusBrowsingContext(..) => {
                    self.servo().execute_webdriver_command(msg);
                },
                WebDriverCommandMsg::GetAllWebViews(response_sender) => {
                    let webviews = self
                        .windows()
                        .values()
                        .flat_map(|window| window.webview_ids())
                        .collect();
                    if let Err(error) = response_sender.send(webviews) {
                        warn!("Failed to send response of GetAllWebViews: {error}");
                    }
                },
                WebDriverCommandMsg::GetWindowRect(webview_id, response_sender) => {
                    let platform_window = self.platform_window_for_webview_id(webview_id);
                    if let Err(error) = response_sender.send(platform_window.window_rect()) {
                        warn!("Failed to send response of GetWindowSize: {error}");
                    }
                },
                WebDriverCommandMsg::MaximizeWebView(webview_id, response_sender) => {
                    let Some(webview) = self.webview_by_id(webview_id) else {
                        continue;
                    };
                    let platform_window = self.platform_window_for_webview_id(webview_id);
                    platform_window.maximize(&webview);

                    if let Err(error) = response_sender.send(platform_window.window_rect()) {
                        warn!("Failed to send response of GetWindowSize: {error}");
                    }
                },
                WebDriverCommandMsg::SetWindowRect(webview_id, requested_rect, size_sender) => {
                    let Some(webview) = self.webview_by_id(webview_id) else {
                        continue;
                    };

                    let platform_window = self.platform_window_for_webview_id(webview_id);
                    let scale = platform_window.hidpi_scale_factor();

                    let requested_physical_rect =
                        (requested_rect.to_f32() * scale).round().to_i32();

                    // Step 17. Set Width/Height.
                    platform_window.request_resize(&webview, requested_physical_rect.size());

                    // Step 18. Set position of the window.
                    platform_window.set_position(requested_physical_rect.min);

                    if let Err(error) = size_sender.send(platform_window.window_rect()) {
                        warn!("Failed to send window size: {error}");
                    }
                },
                WebDriverCommandMsg::GetViewportSize(webview_id, response_sender) => {
                    let platform_window = self.platform_window_for_webview_id(webview_id);
                    let size = platform_window.rendering_context().size2d().to_f32()
                        / platform_window.hidpi_scale_factor();
                    if let Err(error) = response_sender.send(size) {
                        warn!("Failed to send response of GetViewportSize: {error}");
                    }
                },
                // This is only received when start new session.
                WebDriverCommandMsg::GetFocusedWebView(sender) => {
                    let focused_webview = self
                        .focused_window()
                        .and_then(|window| window.active_webview())
                        .map(|webview| webview.id());
                    if let Err(error) = sender.send(focused_webview) {
                        warn!("Failed to send response of GetFocusedWebView: {error}");
                    };
                },
                WebDriverCommandMsg::LoadUrl(webview_id, url, load_status_sender) => {
                    self.handle_webdriver_load_url(webview_id, url, load_status_sender);
                },
                WebDriverCommandMsg::Refresh(webview_id, load_status_sender) => {
                    if let Some(webview) = self.webview_by_id(webview_id) {
                        self.set_load_status_sender(webview_id, load_status_sender);
                        webview.reload();
                    }
                },
                WebDriverCommandMsg::GoBack(webview_id, load_status_sender) => {
                    if let Some(webview) = self.webview_by_id(webview_id) {
                        let traversal_id = webview.go_back(1);
                        self.set_pending_traversal(traversal_id, load_status_sender);
                    }
                },
                WebDriverCommandMsg::GoForward(webview_id, load_status_sender) => {
                    if let Some(webview) = self.webview_by_id(webview_id) {
                        let traversal_id = webview.go_forward(1);
                        self.set_pending_traversal(traversal_id, load_status_sender);
                    }
                },
                WebDriverCommandMsg::InputEvent(webview_id, input_event, response_sender) => {
                    self.handle_webdriver_input_event(webview_id, input_event, response_sender);
                },
                WebDriverCommandMsg::ScriptCommand(_, ref webdriver_script_command) => {
                    self.handle_webdriver_script_command(webdriver_script_command);
                    self.servo().execute_webdriver_command(msg);
                },
                WebDriverCommandMsg::CurrentUserPrompt(webview_id, response_sender) => {
                    let current_dialog = self
                        .webdriver_embedder_controls
                        .current_active_dialog_webdriver_type(webview_id);
                    if let Err(error) = response_sender.send(current_dialog) {
                        warn!("Failed to send response of CurrentUserPrompt: {error}");
                    };
                },
                WebDriverCommandMsg::HandleUserPrompt(webview_id, action, response_sender) => {
                    let controls = &self.webdriver_embedder_controls;
                    let result = controls.respond_to_active_simple_dialog(webview_id, action);
                    if let Err(error) = response_sender.send(result) {
                        warn!("Failed to send response of HandleUserPrompt: {error}");
                    };
                },
                WebDriverCommandMsg::GetAlertText(webview_id, response_sender) => {
                    let response = match self
                        .webdriver_embedder_controls
                        .message_of_newest_dialog(webview_id)
                    {
                        Some(text) => Ok(text),
                        None => Err(()),
                    };

                    if let Err(error) = response_sender.send(response) {
                        warn!("Failed to send response of GetAlertText: {error}");
                    };
                },
                WebDriverCommandMsg::SendAlertText(webview_id, text) => {
                    self.webdriver_embedder_controls
                        .set_prompt_value_of_newest_dialog(webview_id, text);
                },
                WebDriverCommandMsg::TakeScreenshot(webview_id, rect, result_sender) => {
                    self.handle_webdriver_screenshot(webview_id, rect, result_sender);
                },
            };
        }
    }
}

#[derive(Default)]
pub struct ServoshellWebDriverDelegate {
    pub requests: RefCell<Vec<WebDriverToEmbedderMsg>>,
}

impl WebDriverDelegate for ServoshellWebDriverDelegate {
    fn support_multiple_window(&self) -> bool {
        // TODO: is this true for mobile platforms?
        true
    }

    fn pend_request(&self, request: WebDriverToEmbedderMsg) {
        self.requests.borrow_mut().push(request);
    }
}

impl RunningAppState {
    pub(crate) fn handle_pending_webdriver_requests(
        self: &Rc<Self>,
        create_platform_window: Option<&dyn Fn(Url) -> Rc<dyn PlatformWindow>>,
    ) {
        use webdriver_traits::bidi::browsing_context::CreateType;

        for request in self.webdriver_delegate.requests.borrow_mut().drain(..) {
            match request {
                WebDriverToEmbedderMsg::Activate(webview_id, callback) => {
                    let msg = match self.maybe_window_for_webview_id(webview_id) {
                        Some(window) => {
                            // Step 1. navigable's visibility state
                            window.activate_webview(webview_id);
                            // Step 2. system focus
                            self.focus_window(window);
                            true
                        },
                        None => false,
                    };
                    if let Err(err) = callback.send(msg) {
                        warn!("Sending response to webdriver failed ({err:?})");
                    }
                },
                WebDriverToEmbedderMsg::Exit => self.schedule_exit(),
                WebDriverToEmbedderMsg::SetClientWindowState(parameters, callback) => {
                    let Some(window) = parameters
                        .client_window
                        .parse::<u64>()
                        .ok()
                        .map(ServoShellWindowId::from)
                        .and_then(|id| self.window(id))
                    else {
                        if let Err(err) = callback.send(Err(ErrorCode::UnknownError)) {
                            warn!("Sending SetClientWindowState response failed ({err:?})");
                        }
                        continue;
                    };
                    let webview = window
                        .webviews()
                        .first()
                        .expect("window should have at least one webview")
                        .1
                        .clone();
                    // TODO: bad name, change codegen
                    match parameters.client_window_named_state_or_client_window_rect_state {
                        ClientWindowNamedStateOrClientWindowRectState::ClientWindowNamedState(
                            state,
                        ) => match state.state {
                            ClientWindowNamedStateState::Fullscreen => {
                                window.platform_window().set_fullscreen(true);
                            },
                            ClientWindowNamedStateState::Maximized => {
                                window.platform_window().maximize(&webview);
                            },
                            ClientWindowNamedStateState::Minimized => {
                                // TODO: minimize not exposed
                                if let Err(err) = callback.send(Err(ErrorCode::UnknownError)) {
                                    warn!("Sending SetClientWindowState response failed ({err:?})");
                                }
                                continue;
                            },
                        },
                        ClientWindowNamedStateOrClientWindowRectState::ClientWindowRectState(
                            state,
                        ) => {
                            window.platform_window().set_fullscreen(false);
                            let old = window.platform_window().screen_geometry().window_rect;
                            let scale = window.platform_window().hidpi_scale_factor();

                            let x = state.x.unwrap_or(old.min.x as i64);
                            let y = state.y.unwrap_or(old.min.y as i64);
                            let w = state.width.unwrap_or(old.width() as u64) as i64;
                            let h = state.height.unwrap_or(old.height() as u64) as i64;

                            let new_rect = DeviceIndependentIntRect::new(
                                Point2D::new(x as i32, y as i32),
                                Point2D::new((x + w) as i32, (y + h) as i32),
                            );
                            let new_physical = (new_rect.to_f32() * scale).to_i32();
                            window
                                .platform_window()
                                .request_resize(&webview, new_physical.size());
                            window.platform_window().set_position(new_physical.min);
                        },
                    }
                    let client_window_info = self.get_the_client_window_info(&window);
                    if let Err(err) = callback.send(Ok(client_window_info)) {
                        warn!("Error sending SetClientWindowState response ({err:?})");
                    }
                },
                WebDriverToEmbedderMsg::WebViewCreate(WebViewCreateRequest {
                    create_type,
                    opener,
                    callback,
                }) => {
                    let url = Url::parse("about:blank").unwrap();

                    let response = match create_type {
                        CreateType::Tab => {
                            let webview = opener
                                .and_then(|id| self.maybe_window_for_webview_id(id))
                                .or_else(|| self.focused_window())
                                .unwrap_or_else(|| {
                                    self.windows()
                                        .values()
                                        .last()
                                        .expect("Expected at least one window to be open")
                                        .clone()
                                })
                                .create_toplevel_webview(self.clone(), url.clone());
                            Ok(webview.id())
                        },
                        CreateType::Window => match create_platform_window {
                            Some(create_platform_window) => {
                                let window = self
                                    .open_window(create_platform_window(url.clone()), url.clone());
                                let webview = window
                                    .active_webview()
                                    .expect("Should have at least one WebView in a new window");
                                Ok(webview.id())
                            },
                            None => Err(ErrorCode::UnknownError),
                        },
                    };

                    if let Err(err) = callback.send(response) {
                        warn!("Sending create window response to WebDriver failed ({err:?})");
                    }
                },
                WebDriverToEmbedderMsg::GetClientWindows(callback) => {
                    // Step 1. skip, we use window directly
                    // Step 2.
                    let mut client_windows = vec![];
                    // Step 3. we use window directly
                    for client_window in self.windows().values() {
                        let client_window_info = self.get_the_client_window_info(client_window);
                        client_windows.push(client_window_info);
                    }
                    if let Err(err) = callback.send(client_windows) {
                        warn!("Sending GetClientWindow response to WebDriver failed ({err:?})");
                    }
                },
            }
        }
    }

    fn get_the_client_window_info(
        &self,
        client_window: &Rc<ServoShellWindow>,
    ) -> GetClientWindowResponse {
        let webview_id = *client_window
            .webview_ids()
            .first()
            .expect("Window should have at least one webview");
        let platform_window = client_window.platform_window();
        let active = self
            .focused_window()
            .is_some_and(|window| window.id() == client_window.id());
        let state = match platform_window.get_fullscreen() {
            true => ClientWindowInfoState::Fullscreen,
            // TODO: maximized or minimized not exposed in api
            false => ClientWindowInfoState::Normal,
        };
        let window_rect = platform_window.window_rect();
        let client_window_info = GetClientWindowResponse {
            active,
            webview_id,
            height: window_rect.height() as u64,
            width: window_rect.width() as u64,
            x: window_rect.min.x as i64,
            y: window_rect.min.y as i64,
            state,
        };
        client_window_info
    }
}

/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use log::warn;
use servo::{
    EmbedderControl, EmbedderControlId, SimpleDialog, WebDriverCommandMsg, WebDriverUserPrompt,
    WebDriverUserPromptAction, WebViewId,
};
use url::Url;

use crate::running_app_state::RunningAppState;

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
    pub(crate) fn handle_webdriver_messages(self: &Rc<Self>) {
        let Some(webdriver_receiver) = self.webdriver_receiver() else {
            return;
        };

        while let Ok(msg) = webdriver_receiver.try_recv() {
            match msg {
                WebDriverCommandMsg::ResetAllCookies(sender) => {
                    self.servo().cookie_manager().clear_cookies();
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
                WebDriverCommandMsg::NewWebView(response_sender, load_status_sender) => {
                    let new_webview = self
                        .any_window()
                        .create_toplevel_webview(self.clone(), Url::parse("about:blank").unwrap());

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
                    self.window_for_webview_id(webview_id)
                        .activate_webview(webview_id);
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
                    let size = platform_window.rendering_context().size2d();
                    if let Err(error) = response_sender.send(size) {
                        warn!("Failed to send response of GetViewportSize: {error}");
                    }
                },
                // This is only received when start new session.
                WebDriverCommandMsg::GetFocusedWebView(sender) => {
                    let active_webview = self.any_window().active_webview();
                    if let Err(error) = sender.send(active_webview.map(|w| w.id())) {
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

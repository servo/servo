/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use embedder_traits::{WebDriverCommandMsg, WebDriverUserPromptAction};
use ipc_channel::ipc;
use webdriver::error::{ErrorStatus, WebDriverError, WebDriverResult};
use webdriver::response::{ValueResponse, WebDriverResponse};

use crate::{Handler, wait_for_script_response};

impl Handler {
    /// <https://w3c.github.io/webdriver/#dismiss-alert>
    pub(crate) fn handle_dismiss_alert(&mut self) -> WebDriverResult<WebDriverResponse> {
        // Step 1. If session's current top-level browsing context is no longer open,
        // return error with error code no such window.
        self.verify_top_level_browsing_context_is_open(self.session()?.webview_id)?;

        // Step 3. Dismiss the current user prompt.
        let (sender, receiver) = ipc::channel().unwrap();
        self.send_message_to_embedder(WebDriverCommandMsg::HandleUserPrompt(
            self.verified_webview_id(),
            WebDriverUserPromptAction::Dismiss,
            sender,
        ))?;

        match wait_for_script_response(receiver)? {
            // Step 2. If the current user prompt is null, return error with error code no such alert.
            Err(()) => Err(WebDriverError::new(
                ErrorStatus::NoSuchAlert,
                "No user prompt is currently active.",
            )),
            // Step 4. Return success with data null.
            Ok(()) => Ok(WebDriverResponse::Void),
        }
    }

    /// <https://w3c.github.io/webdriver/#accept-alert>
    pub(crate) fn handle_accept_alert(&mut self) -> WebDriverResult<WebDriverResponse> {
        // Step 1. If session's current top-level browsing context is no longer open,
        // return error with error code no such window.
        self.verify_top_level_browsing_context_is_open(self.session()?.webview_id)?;

        // Step 3. Accept the current user prompt.
        let (sender, receiver) = ipc::channel().unwrap();
        self.send_message_to_embedder(WebDriverCommandMsg::HandleUserPrompt(
            self.verified_webview_id(),
            WebDriverUserPromptAction::Accept,
            sender,
        ))?;

        match wait_for_script_response(receiver)? {
            // Step 2. If the current user prompt is null, return error with error code no such alert.
            Err(()) => Err(WebDriverError::new(
                ErrorStatus::NoSuchAlert,
                "No user prompt is currently active.",
            )),
            // Step 4. Return success with data null.
            Ok(()) => Ok(WebDriverResponse::Void),
        }
    }

    pub(crate) fn handle_get_alert_text(&mut self) -> WebDriverResult<WebDriverResponse> {
        // Step 1. If session's current top-level browsing context is no longer open,
        // return error with error code no such window.
        self.verify_top_level_browsing_context_is_open(self.session()?.webview_id)?;

        let (sender, receiver) = ipc::channel().unwrap();
        self.send_message_to_embedder(WebDriverCommandMsg::GetAlertText(
            self.verified_webview_id(),
            sender,
        ))?;

        match wait_for_script_response(receiver)? {
            // Step 2. If the current user prompt is null, return error with error code no such alert.
            Err(()) => Err(WebDriverError::new(
                ErrorStatus::NoSuchAlert,
                "No user prompt is currently active.",
            )),
            // Step 3. Let message be the text message associated with the current user prompt
            // or otherwise be null
            // Step 4. Return success with data message.
            Ok(message) => Ok(WebDriverResponse::Generic(ValueResponse(
                serde_json::to_value(message).map_err(|e| {
                    WebDriverError::new(
                        ErrorStatus::UnknownError,
                        format!("Failed to serialize alert text: {}", e),
                    )
                })?,
            ))),
        }
    }
}

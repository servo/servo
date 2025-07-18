/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::borrow::Cow;
use std::collections::{BTreeMap, HashMap};

use base::id::WebViewId;
use embedder_traits::{WebDriverCommandMsg, WebDriverUserPrompt, WebDriverUserPromptAction};
use ipc_channel::ipc;
use serde_json::{Map, Value};
use webdriver::error::{ErrorStatus, WebDriverError, WebDriverResult};
use webdriver::response::{ValueResponse, WebDriverResponse};

use crate::{Handler, wait_for_script_response};

const KNOWN_PROMPT_HANDLERS: [&str; 5] = [
    "dismiss",
    "accept",
    "dismiss and notify",
    "accept and notify",
    "ignore",
];

const VALID_PROMPT_TYPES: [&str; 6] = [
    "alert",
    "beforeUnload",
    "confirm",
    "default",
    "file",
    "prompt",
];

/// <https://w3c.github.io/webdriver/#dfn-prompt-handler-configuration>
#[derive(Clone, Debug)]
pub(crate) struct PromptHandlerConfiguration {
    handler: WebDriverUserPromptAction,
    notify: bool,
}

pub(crate) type UserPromptHandler = HashMap<WebDriverUserPrompt, PromptHandlerConfiguration>;

/// <https://w3c.github.io/webdriver/#dfn-deserialize-as-an-unhandled-prompt-behavior>
pub(crate) fn deserialize_unhandled_prompt_behaviour(
    value_param: Value,
) -> Result<UserPromptHandler, WebDriverError> {
    // Step 2-5.
    let (value, is_string_value) = match value_param {
        Value::Object(map) => (map, false),
        Value::String(..) => {
            let mut map = Map::new();
            map.insert("fallbackDefault".to_string(), value_param);
            (map, true)
        },
        _ => {
            return Err(WebDriverError::new(
                ErrorStatus::InvalidArgument,
                "Expected an object or a string for unhandled prompt behavior.",
            ));
        },
    };

    // Step 6. Let user prompt handler be a new empty map.
    let mut user_prompt_handler = UserPromptHandler::new();

    // Step 7. For each key-value pair in value:
    for (prompt_type, handler) in value {
        // Step 7.1. If `is_string_value` is false and prompt type is not one of
        // the valid prompt types, return error with error code invalid argument.
        if !is_string_value && !VALID_PROMPT_TYPES.contains(&prompt_type.as_str()) {
            return Err(WebDriverError::new(
                ErrorStatus::InvalidArgument,
                format!("Invalid prompt type: {}", prompt_type),
            ));
        }

        // Step 7.2. If known prompt handlers does not contain an entry with
        // handler key `handler` return error with error code invalid argument.
        let handle_str = match handler {
            Value::String(s) => s,
            _ => {
                return Err(WebDriverError::new(
                    ErrorStatus::InvalidArgument,
                    format!("Expected a string for handler, got: {:?}", handler),
                ));
            },
        };
        if !KNOWN_PROMPT_HANDLERS.contains(&handle_str.as_str()) {
            return Err(WebDriverError::new(
                ErrorStatus::InvalidArgument,
                format!("Unknown prompt handler: {}", handle_str),
            ));
        }

        // Step 7.3 - 7.6.
        let (handler, notify) = match handle_str.as_str() {
            "accept and notify" => (
                WebDriverUserPromptAction::new_from_str("accept").unwrap(),
                true,
            ),
            "dismiss and notify" => (
                WebDriverUserPromptAction::new_from_str("dismiss").unwrap(),
                true,
            ),
            "ignore" => (
                WebDriverUserPromptAction::new_from_str("ignore").unwrap(),
                false,
            ),
            "accept" => (
                WebDriverUserPromptAction::new_from_str("accept").unwrap(),
                false,
            ),
            "dismiss" => (
                WebDriverUserPromptAction::new_from_str("dismiss").unwrap(),
                false,
            ),
            _ => unreachable!(),
        };

        // Step 7.7 - 7.8.
        user_prompt_handler.insert(
            WebDriverUserPrompt::new_from_str(&prompt_type).unwrap(),
            PromptHandlerConfiguration { handler, notify },
        );
    }

    Ok(user_prompt_handler)
}

pub(crate) fn default_unhandled_prompt_behavior() -> &'static str {
    "dismiss and notify"
}

/// <https://www.w3.org/TR/webdriver2/#dfn-get-the-prompt-handler>
fn get_user_prompt_handler(
    user_prompt_handler: &UserPromptHandler,
    prompt_type: WebDriverUserPrompt,
) -> PromptHandlerConfiguration {
    // Step 2. If handlers contains type return handlers[type].
    if let Some(handler) = user_prompt_handler.get(&prompt_type) {
        return (*handler).clone();
    }

    // Step 3. If handlers contains default return handlers[default].
    if let Some(handler) = user_prompt_handler.get(&WebDriverUserPrompt::Default) {
        return (*handler).clone();
    }

    // Step 4. If prompt type is "beforeUnload" return a configuration with handler "accept" and notify false.
    if prompt_type == WebDriverUserPrompt::BeforeUnload {
        return PromptHandlerConfiguration {
            handler: WebDriverUserPromptAction::Accept,
            notify: false,
        };
    }

    // Step 5. If handlers contains fallbackDefault return handlers[fallbackDefault].
    if let Some(handler) = user_prompt_handler.get(&WebDriverUserPrompt::FallbackDefault) {
        return (*handler).clone();
    }

    // Step 6. Return a configuration with handler "dismiss" and notify true.
    PromptHandlerConfiguration {
        handler: WebDriverUserPromptAction::Dismiss,
        notify: true,
    }
}

fn webdriver_response_single_data(
    key: &'static str,
    value: Value,
) -> Option<BTreeMap<Cow<'static, str>, Value>> {
    Some([(Cow::Borrowed(key), value)].into_iter().collect())
}

impl Handler {
    /// <https://w3c.github.io/webdriver/#dismiss-alert>
    pub(crate) fn handle_dismiss_alert(&self) -> WebDriverResult<WebDriverResponse> {
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
            Ok(_) => Ok(WebDriverResponse::Void),
        }
    }

    /// <https://w3c.github.io/webdriver/#accept-alert>
    pub(crate) fn handle_accept_alert(&self) -> WebDriverResult<WebDriverResponse> {
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
            Ok(_) => Ok(WebDriverResponse::Void),
        }
    }

    /// <https://www.w3.org/TR/webdriver2/#dfn-get-alert-text>
    pub(crate) fn handle_get_alert_text(&self) -> WebDriverResult<WebDriverResponse> {
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

    /// <https://www.w3.org/TR/webdriver2/#dfn-send-alert-text>
    pub(crate) fn handle_send_alert_text(
        &self,
        text: String,
    ) -> WebDriverResult<WebDriverResponse> {
        let webview_id = self.session()?.webview_id;

        // Step 3. If session's current top-level browsing context is no longer open,
        // return error with error code no such window.
        self.verify_top_level_browsing_context_is_open(webview_id)?;

        let (sender, receiver) = ipc::channel().unwrap();

        self.send_message_to_embedder(WebDriverCommandMsg::CurrentUserPrompt(webview_id, sender))?;

        match wait_for_script_response(receiver)? {
            // Step 4. If the current user prompt is null, return error with error code no such alert.
            None => Err(WebDriverError::new(
                ErrorStatus::NoSuchAlert,
                "No user prompt is currently active.",
            )),
            Some(prompt_type) => {
                match prompt_type {
                    // Step 5. If the current user prompt is alert or confirm,
                    // return error with error code element not interactable.
                    WebDriverUserPrompt::Alert | WebDriverUserPrompt::Confirm => {
                        Err(WebDriverError::new(
                            ErrorStatus::ElementNotInteractable,
                            "Cannot send text to an alert or confirm prompt.",
                        ))
                    },
                    // Step 5. If the current user prompt is prompt
                    WebDriverUserPrompt::Prompt => {
                        // Step 6. Send the text to the current user prompt.
                        self.send_message_to_embedder(WebDriverCommandMsg::SendAlertText(
                            webview_id, text,
                        ))?;

                        Ok(WebDriverResponse::Void)
                    },
                    // Step 5. Otherwise, return error with error code unsupported operation.
                    _ => Err(WebDriverError::new(
                        ErrorStatus::UnsupportedOperation,
                        "Current user prompt type is not supported.",
                    )),
                }
            },
        }
    }

    /// <https://w3c.github.io/webdriver/#dfn-handle-any-user-prompts>
    pub(crate) fn handle_any_user_prompts(
        &self,
        webview_id: WebViewId,
    ) -> WebDriverResult<WebDriverResponse> {
        let (sender, receiver) = ipc::channel().unwrap();

        self.send_message_to_embedder(WebDriverCommandMsg::CurrentUserPrompt(webview_id, sender))?;

        match wait_for_script_response(receiver)? {
            // Step 1. If the current user prompt is null, return success with data null.
            None => Ok(WebDriverResponse::Void),
            Some(prompt_type) => {
                // Step 2 - 4. Get user prompt handler for the prompt type.
                let handler =
                    get_user_prompt_handler(&self.session()?.user_prompt_handler, prompt_type);

                // Step 5. Perform the substeps based on handler's handler
                let (sender, receiver) = ipc::channel().unwrap();
                self.send_message_to_embedder(WebDriverCommandMsg::HandleUserPrompt(
                    webview_id,
                    handler.handler.clone(),
                    sender,
                ))?;

                if handler.notify || handler.handler == WebDriverUserPromptAction::Ignore {
                    // Step 6. If handler's notify is true, return annotated unexpected alert open error.
                    let alert_text = wait_for_script_response(receiver)?
                        .unwrap_or_default()
                        .unwrap_or_default();

                    Err(WebDriverError::new_with_data(
                        ErrorStatus::UnexpectedAlertOpen,
                        "Handle any user prompt: Unexpected alert open.",
                        webdriver_response_single_data("text", Value::String(alert_text)),
                        None,
                    ))
                } else {
                    // Step 7. Return success with data null.
                    Ok(WebDriverResponse::Void)
                }
            },
        }
    }
}

/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use base::id::{BrowsingContextId, WebViewId};
use serde_json::{Map, Value, json};
use uuid::Uuid;
use webdriver::error::WebDriverResult;

use crate::capabilities::ServoCapabilities;
use crate::user_prompt::{
    default_unhandled_prompt_behavior, deserialize_unhandled_prompt_behaviour,
};
use crate::{Handler, WebDriverSession};

#[derive(Debug, PartialEq, serde::Serialize)]
pub enum PageLoadStrategy {
    None,
    Eager,
    Normal,
}

impl Handler {
    /// <https://w3c.github.io/webdriver/#dfn-create-a-session>
    pub(crate) fn create_session(
        &mut self,
        capabilities: &mut Map<String, Value>,
        servo_capabilities: &ServoCapabilities,
        webview_id: WebViewId,
        browsing_context_id: BrowsingContextId,
    ) -> WebDriverResult<Uuid> {
        // Step 2. Let session be a new session
        let mut session = WebDriverSession::new(browsing_context_id, webview_id);

        // Step 3. Let proxy be the result of getting property "proxy" from capabilities
        match capabilities.get("proxy") {
            // Proxy is a proxy configuration object
            Some(_) => {
                // TODO:
                // Take implementation-defined steps to set the user agent proxy
                // using the extracted proxy configuration.
                // If the defined proxy cannot be configured return error with error code
                // session not created. Otherwise set the has proxy configuration flag to true.
            },
            // Otherwise, set a property of capabilities with name "proxy"
            // and a value that is a new JSON Object.
            None => {
                capabilities.insert(String::from("proxy"), json!({}));
            },
        }

        // Step 4. If capabilites has a property named "acceptInsecureCerts"
        match capabilities.get("acceptInsecureCerts") {
            Some(_accept_insecure_certs) => {
                // TODO: Set the endpoint node's accept insecure TLS flag
            },
            None => {
                capabilities.insert(String::from("acceptInsecureCerts"), json!(false));
            },
        }

        // Step 5. Let user prompt handler capability be the result of
        // getting property "unhandledPromptBehavior" from capabilities
        match capabilities.get("unhandledPromptBehavior") {
            // Step 6. If user prompt handler capability is not undefined
            Some(unhandled_prompt_behavior) => {
                session.user_prompt_handler =
                    deserialize_unhandled_prompt_behaviour(unhandled_prompt_behavior.clone())?;
            },
            // Step 7. Let serialized user prompt handler be serialize the user prompt handler.
            // Step 8. Set a property on capabilities with the name "unhandledPromptBehavior",
            // and the value serialized user prompt handler.
            // Ignore because the user prompt handler is already in the capabilities object
            None => {
                capabilities.insert(
                    String::from("unhandledPromptBehavior"),
                    json!(default_unhandled_prompt_behavior()),
                );
            },
        }

        // TODO: flag is http by default for now
        // Step 9. If flags contains "http"
        // Step 9.1. Let strategy be the result of getting property "pageLoadStrategy" from capabilities.
        match capabilities.get("pageLoadStrategy") {
            // If strategy is a string, set the session's page loading strategy to strategy.
            Some(strategy) => match strategy.to_string().as_str() {
                "none" => session.page_loading_strategy = PageLoadStrategy::None,
                "eager" => session.page_loading_strategy = PageLoadStrategy::Eager,
                _ => session.page_loading_strategy = PageLoadStrategy::Normal,
            },
            // Otherwise, set the page loading strategy to normal and set a property of capabilities
            // with name "pageLoadStrategy" and value "normal".
            None => {
                capabilities.insert(
                    String::from("pageLoadStrategy"),
                    json!(session.page_loading_strategy),
                );
                session.page_loading_strategy = PageLoadStrategy::Normal;
            },
        }

        // Step 9.2. Let strictFileInteractability be the result of getting property
        // "strictFileInteractability" from capabilities
        match capabilities.get("strictFileInteractability") {
            Some(Value::Bool(strict_file_interactability)) => {
                session.strict_file_interactability = *strict_file_interactability;
            },
            _ => {
                // Set a default value to cappabilities
                capabilities.insert(
                    "strictFileInteractability".to_string(),
                    json!(session.strict_file_interactability),
                );
            },
        }

        // Step 9.3. Let timeouts be the result of getting a property "timeouts" from capabilities.
        // If timeouts is not undefined, set session's session timeouts to timeouts.
        if let Some(timeouts) = capabilities.get("timeouts") {
            if let Some(script_timeout_value) = timeouts.get("script") {
                session.script_timeout = script_timeout_value.as_u64();
            }
            if let Some(load_timeout_value) = timeouts.get("pageLoad") {
                if let Some(load_timeout) = load_timeout_value.as_u64() {
                    session.load_timeout = load_timeout;
                }
            }
            if let Some(implicit_wait_timeout_value) = timeouts.get("implicit") {
                if let Some(implicit_wait_timeout) = implicit_wait_timeout_value.as_u64() {
                    session.implicit_wait_timeout = implicit_wait_timeout;
                }
            }
        }

        // Step 9.4 Set a property on capabilities with name "timeouts"
        // and value serialize the timeouts configuration with session's session timeouts.
        capabilities.insert(
            "timeouts".to_string(),
            json!({
                "script": session.script_timeout,
                "pageLoad": session.load_timeout,
                "implicit": session.implicit_wait_timeout,
            }),
        );

        // Step 10. Process any extension capabilities in capabilities in an implementation-defined manner
        // There is no extension capabilities.

        // Step 11. Run any WebDriver new session algorithm defined in external specifications
        capabilities.insert(
            "browserName".to_string(),
            json!(servo_capabilities.browser_name),
        );
        capabilities.insert(
            "browserVersion".to_string(),
            json!(servo_capabilities.browser_version),
        );
        capabilities.insert(
            "platformName".to_string(),
            json!(
                servo_capabilities
                    .platform_name
                    .clone()
                    .unwrap_or("unknown".to_string())
            ),
        );
        capabilities.insert(
            "setWindowRect".to_string(),
            json!(servo_capabilities.set_window_rect),
        );
        capabilities.insert(
            "userAgent".to_string(),
            servo_config::pref!(user_agent).into(),
        );

        // Step 12. Append session to active sessions
        let id = session.id;
        self.session = Some(session);

        Ok(id)
    }
}

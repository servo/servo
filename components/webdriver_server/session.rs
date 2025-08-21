/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::{Ref, RefCell, RefMut};
use std::collections::HashMap;

use base::id::{BrowsingContextId, WebViewId};
use serde_json::{Map, Value, json};
use uuid::Uuid;
use webdriver::error::WebDriverResult;

use crate::Handler;
use crate::actions::{ActionItem, InputSourceState};
use crate::capabilities::ServoCapabilities;
use crate::timeout::{
    TimeoutsConfiguration, deserialize_as_timeouts_configuration, serialize_timeouts_configuration,
};
use crate::user_prompt::{
    UserPromptHandler, default_unhandled_prompt_behavior, deserialize_unhandled_prompt_behaviour,
};

#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub enum PageLoadStrategy {
    None,
    Eager,
    Normal,
}

// Need a different implementation for ToString than Display
#[allow(clippy::to_string_trait_impl)]
impl ToString for PageLoadStrategy {
    fn to_string(&self) -> String {
        match self {
            PageLoadStrategy::None => String::from("none"),
            PageLoadStrategy::Eager => String::from("eager"),
            PageLoadStrategy::Normal => String::from("normal"),
        }
    }
}

/// Represents the current WebDriver session and holds relevant session state.
/// Currently, only 1 webview is supported per session.
/// So only there is only 1 InputState.
pub struct WebDriverSession {
    /// <https://www.w3.org/TR/webdriver2/#dfn-session-id>
    id: Uuid,

    /// <https://www.w3.org/TR/webdriver2/#dfn-current-top-level-browsing-context>
    /// The id of the current top-level browsing context
    webview_id: Option<WebViewId>,

    /// <https://www.w3.org/TR/webdriver2/#dfn-current-browsing-context>
    /// The id of the current browsing context
    browsing_context_id: Option<BrowsingContextId>,

    timeouts: TimeoutsConfiguration,

    page_loading_strategy: PageLoadStrategy,

    strict_file_interactability: bool,

    user_prompt_handler: UserPromptHandler,

    /// <https://w3c.github.io/webdriver/#dfn-input-state-map>
    input_state_table: RefCell<HashMap<String, InputSourceState>>,

    /// <https://w3c.github.io/webdriver/#dfn-input-cancel-list>
    input_cancel_list: RefCell<Vec<(String, ActionItem)>>,
}

impl WebDriverSession {
    pub fn new() -> WebDriverSession {
        WebDriverSession {
            id: Uuid::new_v4(),
            webview_id: None,
            browsing_context_id: None,
            timeouts: TimeoutsConfiguration::default(),
            page_loading_strategy: PageLoadStrategy::Normal,
            strict_file_interactability: false,
            user_prompt_handler: UserPromptHandler::new(),
            input_state_table: RefCell::new(HashMap::new()),
            input_cancel_list: RefCell::new(Vec::new()),
        }
    }

    pub fn set_webview_id(&mut self, webview_id: Option<WebViewId>) {
        self.webview_id = webview_id;
    }

    pub fn set_browsing_context_id(&mut self, browsing_context_id: Option<BrowsingContextId>) {
        self.browsing_context_id = browsing_context_id;
    }

    pub fn current_webview_id(&self) -> Option<WebViewId> {
        self.webview_id
    }

    pub fn current_browsing_context_id(&self) -> Option<BrowsingContextId> {
        self.browsing_context_id
    }

    pub fn session_timeouts(&self) -> &TimeoutsConfiguration {
        &self.timeouts
    }

    pub fn session_timeouts_mut(&mut self) -> &mut TimeoutsConfiguration {
        &mut self.timeouts
    }

    pub fn page_loading_strategy(&self) -> PageLoadStrategy {
        self.page_loading_strategy.clone()
    }

    pub fn strict_file_interactability(&self) -> bool {
        self.strict_file_interactability
    }

    pub fn user_prompt_handler(&self) -> &UserPromptHandler {
        &self.user_prompt_handler
    }

    pub fn input_state_table(&self) -> Ref<'_, HashMap<String, InputSourceState>> {
        self.input_state_table.borrow()
    }

    pub fn input_state_table_mut(&self) -> RefMut<'_, HashMap<String, InputSourceState>> {
        self.input_state_table.borrow_mut()
    }

    pub fn input_cancel_list_mut(&self) -> RefMut<'_, Vec<(String, ActionItem)>> {
        self.input_cancel_list.borrow_mut()
    }
}

impl Handler {
    /// <https://w3c.github.io/webdriver/#dfn-create-a-session>
    pub(crate) fn create_session(
        &mut self,
        capabilities: &mut Map<String, Value>,
        servo_capabilities: &ServoCapabilities,
    ) -> WebDriverResult<Uuid> {
        // Step 2. Let session be a new session
        let mut session = WebDriverSession::new();

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
                    json!(session.page_loading_strategy.to_string()),
                );
                session.page_loading_strategy = PageLoadStrategy::Normal;
            },
        }

        // Step 9.2. Let strictFileInteractability be the result of getting property
        // "strictFileInteractability" from capabilities
        if let Some(Value::Bool(strict_file_interactability)) =
            capabilities.get("strictFileInteractability")
        {
            session.strict_file_interactability = *strict_file_interactability;
        } else {
            capabilities.insert(String::from("strictFileInteractability"), json!(false));
        }

        // Step 9.3. Let timeouts be the result of getting a property "timeouts" from capabilities.
        // If timeouts is not undefined, set session's session timeouts to timeouts.
        if let Some(timeouts) = capabilities.get("timeouts") {
            session.timeouts = deserialize_as_timeouts_configuration(timeouts)?;
        }

        // Step 9.4 Set a property on capabilities with name "timeouts"
        // and value serialize the timeouts configuration with session's session timeouts.
        capabilities.insert(
            "timeouts".to_string(),
            json!(serialize_timeouts_configuration(&session.timeouts)),
        );

        // Step 10. Process any extension capabilities in capabilities in an implementation-defined manner
        // Nothing to processed

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

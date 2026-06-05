//! Decouple handling logic using trait.
//!
//! Also provide a default implementation.

use crossbeam_channel::{Sender, bounded};
use embedder_traits::{
    EventLoopWaker,
    webdriver_bidi::{RequestId, WaitCondition, WebDriverBidiCommandMsg},
};
use rustenium_bidi_definitions::{
    Command,
    base::{CommandMessage, ErrorCode},
    browser::commands::BrowserCommand,
    browsing_context::{self, commands::BrowsingContextCommand, types::ReadinessState},
    emulation::commands::EmulationCommand,
    input::commands::InputCommand,
    network::commands::NetworkCommand,
    script::commands::ScriptCommand,
    session::{self, commands::SessionCommand},
    storage::commands::StorageCommand,
    web_extension::commands::WebExtensionCommand,
};
use servo_base::id::{BrowsingContextId, WebViewId};
use uuid::Uuid;

use crate::{
    dispatcher::DispatchMessage,
    error::{WebDriverBidiError, WebDriverBidiResult},
    model::{Message as BidiMessage, ResultData, SessionResult},
    session::SessionId,
};

pub trait WebDriverBidiHandler: Send + Sized {
    fn to_sessioned(&self) -> Option<Self>;

    /// Start processing of a command.
    fn handle(
        &self,
        request_id: RequestId,
        command: &CommandMessage,
        tx: Sender<DispatchMessage>,
    ) -> WebDriverBidiResult<()>;

    fn try_recv(&self) -> WebDriverBidiResult<(Option<RequestId>, BidiMessage)>;

    // TODO: do we need
    // post update after receiving message
    // fn update(&mut self, message: &Message);
}

// TODO: bidi session has different meaning to classic session.
// and webviewid is not bound to session.

pub struct Handler {
    event_loop_waker: Box<dyn EventLoopWaker>,
    embedder_sender: Sender<WebDriverBidiCommandMsg>,
    is_static: bool,
    webview_id: Option<WebViewId>,
}

/// Util methods.
impl Handler {
    pub fn new(
        event_loop_waker: Box<dyn EventLoopWaker>,
        embedder_sender: crossbeam_channel::Sender<WebDriverBidiCommandMsg>,
    ) -> Self {
        Self {
            event_loop_waker,
            embedder_sender,
            is_static: true,
            webview_id: None,
        }
    }

    // TODO: bidi use BrowsingContextId not WebId
    pub fn webview_id(&self) -> WebDriverBidiResult<&WebViewId> {
        self.webview_id
            .as_ref()
            .ok_or_else(|| WebDriverBidiError::unknown("No webview available"))
    }

    fn send_message_to_embedder(&self, msg: WebDriverBidiCommandMsg) -> WebDriverBidiResult<()> {
        self.embedder_sender.send(msg)?;
        self.event_loop_waker.wake();
        Ok(())
    }
}

impl WebDriverBidiHandler for Handler {
    fn to_sessioned(&self) -> Option<Self> {
        if !self.is_static {
            return None;
        }

        Some(Self {
            event_loop_waker: self.event_loop_waker.clone_box(),
            embedder_sender: self.embedder_sender.clone(),
            is_static: false,
            webview_id: None,
        })
    }

    fn handle(
        &self,
        request_id: RequestId,
        command: &CommandMessage,
        dispatch_tx: Sender<DispatchMessage>,
    ) -> WebDriverBidiResult<()> {
        match &command.command_data {
            Command::Browser(browser) => match browser {
                BrowserCommand::Close(close) => self.handle_browser_close(),
                BrowserCommand::CreateUserContext(_) => self.handle_browser_create_user_context(),
                BrowserCommand::GetClientWindows(get_client_windows) => {
                    self.handle_browser_get_client_windows()
                },
                BrowserCommand::GetUserContexts(_) => self.handle_browser_get_user_contexts(),
                BrowserCommand::RemoveUserContext(_) => self.handle_browser_remove_user_context(),
                BrowserCommand::SetClientWindowState(set_client_window_state) => {
                    self.handle_browser_set_client_window_state()
                },
                BrowserCommand::SetDownloadBehavior(set_download_behavior) => {
                    self.handle_browser_set_download_behavior()
                },
            },
            Command::BrowsingContext(browsing_context) => match browsing_context {
                BrowsingContextCommand::Activate(activate) => {
                    self.handle_browsing_context_activate()
                },
                BrowsingContextCommand::CaptureScreenshot(capture_screenshot) => {
                    self.handle_browsing_context_capture_screenshot()
                },
                BrowsingContextCommand::Close(close) => self.handle_browsing_context_close(),
                BrowsingContextCommand::Create(create) => self.handle_browsing_context_create(),
                BrowsingContextCommand::GetTree(get_tree) => {
                    self.handle_browsing_context_get_tree()
                },
                BrowsingContextCommand::HandleUserPrompt(handle_user_prompt) => {
                    self.handle_browsing_context_handle_user_prompt()
                },
                BrowsingContextCommand::LocateNodes(locate_nodes) => {
                    self.handle_browsing_context_locate_nodes()
                },
                BrowsingContextCommand::Navigate(navigate) => {
                    self.handle_browsing_context_navigate()
                },
                BrowsingContextCommand::Print(print) => self.handle_browsing_context_print(),
                BrowsingContextCommand::Reload(reload) => {
                    self.handle_browsing_context_reload(reload)
                },
                BrowsingContextCommand::SetViewport(set_viewport) => {
                    self.handle_browsing_context_set_viewport()
                },
                BrowsingContextCommand::TraverseHistory(traverse_history) => {
                    self.handle_browsing_context_traverse_history(traverse_history.params.delta)
                },
            },
            Command::Emulation(emulation) => match emulation {
                EmulationCommand::SetForcedColorsModeThemeOverride(
                    set_forced_colors_mode_theme_override,
                ) => self.handle_emulation_set_forced_colors_mode_theme_override(),
                EmulationCommand::SetGeolocationOverride(set_geolocation_override) => {
                    self.handle_emulation_set_geolocation_override()
                },
                EmulationCommand::SetLocaleOverride(set_locale_override) => {
                    self.handle_emulation_set_locale_override()
                },
                EmulationCommand::SetNetworkConditions(set_network_conditions) => {
                    self.handle_emulation_set_network_conditions()
                },
                EmulationCommand::SetScreenOrientationOverride(set_screen_orientation_override) => {
                    self.handle_emulation_set_screen_orientation_override()
                },
                EmulationCommand::SetUserAgentOverride(set_user_agent_override) => {
                    self.handle_emulation_set_user_agent_override()
                },
                EmulationCommand::SetScriptingEnabled(set_scripting_enabled) => {
                    self.handle_emulation_set_scripting_enabled()
                },
                EmulationCommand::SetTimezoneOverride(set_timezone_override) => {
                    self.handle_emulation_set_timezone_override()
                },
            },
            Command::Input(input) => match input {
                InputCommand::PerformActions(perform_actions) => {
                    self.handle_input_perform_actions()
                },
                InputCommand::ReleaseActions(release_actions) => {
                    self.handle_input_release_actions()
                },
                InputCommand::SetFiles(set_files) => self.handle_input_set_files(),
            },
            Command::Network(network) => match network {
                NetworkCommand::AddDataCollector(add_data_collector) => {
                    self.handle_network_add_data_collector()
                },
                NetworkCommand::AddIntercept(add_intercept) => self.handle_network_add_intercept(),
                NetworkCommand::ContinueRequest(continue_request) => {
                    self.handle_network_continue_request()
                },
                NetworkCommand::ContinueResponse(continue_response) => {
                    self.handle_network_continue_response()
                },
                NetworkCommand::ContinueWithAuth(continue_with_auth) => {
                    self.handle_network_continue_with_auth()
                },
                NetworkCommand::DisownData(disown_data) => self.handle_network_disown_data(),
                NetworkCommand::FailRequest(fail_request) => self.handle_network_fail_request(),
                NetworkCommand::GetData(get_data) => self.handle_network_get_data(),
                NetworkCommand::ProvideResponse(provide_response) => {
                    self.handle_network_provide_response()
                },
                NetworkCommand::RemoveDataCollector(remove_data_collector) => {
                    self.handle_network_remove_data_collector()
                },
                NetworkCommand::RemoveIntercept(remove_intercept) => {
                    self.handle_network_remove_intercept()
                },
                NetworkCommand::SetCacheBehavior(set_cache_behavior) => {
                    self.handle_network_set_cache_behavior()
                },
                NetworkCommand::SetExtraHeaders(set_extra_headers) => {
                    self.handle_network_set_extra_headers()
                },
            },
            Command::Script(script) => match script {
                ScriptCommand::AddPreloadScript(add_preload_script) => {
                    self.handle_script_add_preload_script()
                },
                ScriptCommand::Disown(disown) => self.handle_script_disown(),
                ScriptCommand::CallFunction(call_function) => self.handle_script_call_function(),
                ScriptCommand::Evaluate(evaluate) => self.handle_script_evaluate(),
                ScriptCommand::GetRealms(get_realms) => self.handle_script_get_realms(),
                ScriptCommand::RemovePreloadScript(remove_preload_script) => {
                    self.handle_script_remove_preload_script()
                },
            },
            Command::Session(session) => match session {
                SessionCommand::Status(status) => self.handle_session_status(),
                SessionCommand::New(session_new) => {
                    self.handle_session_new(session_new, dispatch_tx)
                },
                SessionCommand::End(end) => self.handle_session_end(),
                SessionCommand::Subscribe(subscribe) => self.handle_session_subscribe(),
                SessionCommand::Unsubscribe(unsubscribe) => self.handle_session_unsubscribe(),
            },
            Command::Storage(storage) => match storage {
                StorageCommand::GetCookies(get_cookies) => self.handle_storage_get_cookies(),
                StorageCommand::SetCookie(set_cookie) => self.handle_storage_set_cookie(),
                StorageCommand::DeleteCookies(delete_cookies) => {
                    self.handle_storage_delete_cookies()
                },
            },
            Command::WebExtension(web_extension) => match web_extension {
                WebExtensionCommand::Install(_) => self.handle_web_extension_install(),
                WebExtensionCommand::Uninstall(_) => self.handle_web_extension_uninstall(),
            },
        }
    }

    fn try_recv(&self) -> WebDriverBidiResult<(Option<RequestId>, BidiMessage)> {
        todo!()
    }
}

/// Concrete handle methods.
impl Handler {
    // TODO: can non-static handlers be considered ready
    fn handle_session_status(&self) -> WebDriverBidiResult<()> {
        // TODO: currently we can directly return ready, but in future we may need to wait for
        // capabilities communication between handler and embedder.

        // Step 1. let body
        let body = session::results::StatusResult {
            ready: true,
            message: "".to_string(),
        };

        // Step 2. return success
        // TODO: move to recv
        // Ok(Some(ResultData::Session(SessionResult::Status(body))))
        Ok(())
    }

    fn handle_session_new(
        &self,
        _session_new: &session::commands::New,
        dispatch_tx: Sender<DispatchMessage>,
    ) -> WebDriverBidiResult<()> {
        // Step 1. if session is not null, return "session not created" error.
        if !self.is_static {
            return Err(WebDriverBidiError::new(
                ErrorCode::SessionNotCreated,
                "session is not null",
            ));
        }

        // Step 2. if impl is unable to start a new session, skip.

        // TODO: process capabilities and `create a session`
        let session_id = Uuid::new_v4();

        // instruct dispatcher to create session.
        let (tx, rx) = bounded(1);
        dispatch_tx.send(DispatchMessage::SessionNew(SessionId(session_id), tx));

        // TODO: move the following to recv

        // Step 8. let body be session.NewResult

        let body = session::results::NewResult {
            session_id: session_id.to_string(),
            capabilities: session::types::NewResultCapabilities {
                // TODO: fields below are hard coded or randomly filled
                accept_insecure_certs: false,
                browser_name: "servoshell".to_string(),
                browser_version: "0.2.0".to_string(),
                platform_name: "unknown".to_string(),
                set_window_rect: false,
                user_agent: None,
                proxy: None,
                unhandled_prompt_behavior: None,
                web_socket_url: None,
                extensible: Default::default(),
            },
        };

        // Step 9. return success
        // Ok(Some(ResultData::Session(SessionResult::New(Box::new(
        //     body,
        // )))))
        Ok(())
    }

    fn handle_session_end(&self) -> WebDriverBidiResult<()> {
        // Step 1. end the session, skip.
        // TODO: do we need to notify embedder to stop subscription or other status,
        // or is this automatically done with channel drop?

        // Step 2. return success
        // TODO: move to recv
        // Ok(Some(ResultData::Session(SessionResult::End(
        //     session::results::EndResult {
        //         extensible: Default::default(),
        //     },
        // ))))
        Ok(())
    }

    fn handle_session_subscribe(&self) -> WebDriverBidiResult<()> {
        todo!()
    }

    fn handle_session_unsubscribe(&self) -> WebDriverBidiResult<()> {
        todo!()
    }

    fn handle_browser_close(&self) -> WebDriverBidiResult<()> {
        todo!()
    }

    fn handle_browser_create_user_context(&self) -> WebDriverBidiResult<()> {
        Err(WebDriverBidiError::unknown(
            "user context is not implemented yet",
        ))
    }

    fn handle_browser_get_client_windows(&self) -> WebDriverBidiResult<()> {
        todo!()
    }

    fn handle_browser_get_user_contexts(&self) -> WebDriverBidiResult<()> {
        Err(WebDriverBidiError::unknown(
            "user context is not implemented yet",
        ))
    }
    fn handle_browser_remove_user_context(&self) -> WebDriverBidiResult<()> {
        Err(WebDriverBidiError::unknown(
            "user context is not implemented yet",
        ))
    }
    fn handle_browser_set_client_window_state(&self) -> WebDriverBidiResult<()> {
        todo!()
    }
    fn handle_browser_set_download_behavior(&self) -> WebDriverBidiResult<()> {
        todo!()
    }

    fn handle_browsing_context_activate(&self) -> WebDriverBidiResult<()> {
        todo!()
    }
    fn handle_browsing_context_capture_screenshot(&self) -> WebDriverBidiResult<()> {
        todo!()
    }
    fn handle_browsing_context_close(&self) -> WebDriverBidiResult<()> {
        todo!()
    }
    fn handle_browsing_context_create(&self) -> WebDriverBidiResult<()> {
        todo!()
    }
    fn handle_browsing_context_get_tree(&self) -> WebDriverBidiResult<()> {
        todo!()
    }
    fn handle_browsing_context_handle_user_prompt(&self) -> WebDriverBidiResult<()> {
        todo!()
    }
    fn handle_browsing_context_locate_nodes(&self) -> WebDriverBidiResult<()> {
        todo!()
    }
    fn handle_browsing_context_navigate(&self) -> WebDriverBidiResult<()> {
        todo!()
    }
    fn handle_browsing_context_print(&self) -> WebDriverBidiResult<()> {
        todo!()
    }

    fn handle_browsing_context_reload(
        &self,
        cmd: &browsing_context::commands::Reload,
    ) -> WebDriverBidiResult<()> {
        // Step 1: let navigable id be "context"
        let navigable_id = &cmd.params.context;

        // Step 2: let navigable, trying to get a navigable with id
        // TODO: impl
        // TODO: should we keep the id here? id may be changed by other session

        // Step 3: Assert navigable is not null
        // TODO: let else

        // Step 4: let ignore cache if present, or false otherwise
        let ignore_cache = cmd.params.ignore_cache.unwrap_or(false);

        // Step 5: let `wait condition` be `"committed"`
        // TODO: should move webdriver type to shared, like webdriver classic
        let mut wait_condition = WaitCondition::Committed;

        // Step 6: if wait and wait not "none", set `wait condition`
        if let Some(wait) = &cmd.params.wait {
            match wait {
                ReadinessState::None => {},
                ReadinessState::Interactive => {
                    wait_condition = WaitCondition::Interactive;
                },
                ReadinessState::Complete => wait_condition = WaitCondition::Complete,
            }
        }

        // Step 7: let document be active document

        // Step 8: let url be document's url

        // Step 9: let request be a new request

        // Step 10: await a navigation
        // TODO: check id
        let browser_context_id = BrowsingContextId::new();
        let cmd_msg =
            WebDriverBidiCommandMsg::BrowsingContextReload(todo!(), ignore_cache, wait_condition);
        self.send_message_to_embedder(cmd_msg);

        // wait for response in `try_recv`

        Ok(())
    }

    fn handle_browsing_context_set_viewport(&self) -> WebDriverBidiResult<()> {
        todo!()
    }
    fn handle_browsing_context_traverse_history(&self, delta: i64) -> WebDriverBidiResult<()> {
        let webview_id = self.webview_id()?;
        // TODO: verify context open? is this in bidi spec
        self.send_message_to_embedder(WebDriverBidiCommandMsg::TraverseHistory(
            *webview_id,
            delta,
        ))?;
        // TODO: fix return type
        Ok(())
    }

    fn handle_emulation_set_forced_colors_mode_theme_override(&self) -> WebDriverBidiResult<()> {
        todo!()
    }
    fn handle_emulation_set_geolocation_override(&self) -> WebDriverBidiResult<()> {
        todo!()
    }
    fn handle_emulation_set_locale_override(&self) -> WebDriverBidiResult<()> {
        todo!()
    }
    fn handle_emulation_set_network_conditions(&self) -> WebDriverBidiResult<()> {
        todo!()
    }
    fn handle_emulation_set_screen_orientation_override(&self) -> WebDriverBidiResult<()> {
        todo!()
    }
    fn handle_emulation_set_user_agent_override(&self) -> WebDriverBidiResult<()> {
        todo!()
    }
    fn handle_emulation_set_scripting_enabled(&self) -> WebDriverBidiResult<()> {
        todo!()
    }
    fn handle_emulation_set_timezone_override(&self) -> WebDriverBidiResult<()> {
        todo!()
    }

    fn handle_network_add_data_collector(&self) -> WebDriverBidiResult<()> {
        todo!()
    }
    fn handle_network_add_intercept(&self) -> WebDriverBidiResult<()> {
        todo!()
    }
    fn handle_network_continue_request(&self) -> WebDriverBidiResult<()> {
        todo!()
    }
    fn handle_network_continue_response(&self) -> WebDriverBidiResult<()> {
        todo!()
    }
    fn handle_network_continue_with_auth(&self) -> WebDriverBidiResult<()> {
        todo!()
    }
    fn handle_network_disown_data(&self) -> WebDriverBidiResult<()> {
        todo!()
    }
    fn handle_network_fail_request(&self) -> WebDriverBidiResult<()> {
        todo!()
    }
    fn handle_network_get_data(&self) -> WebDriverBidiResult<()> {
        todo!()
    }
    fn handle_network_provide_response(&self) -> WebDriverBidiResult<()> {
        todo!()
    }
    fn handle_network_remove_data_collector(&self) -> WebDriverBidiResult<()> {
        todo!()
    }
    fn handle_network_remove_intercept(&self) -> WebDriverBidiResult<()> {
        todo!()
    }
    fn handle_network_set_cache_behavior(&self) -> WebDriverBidiResult<()> {
        todo!()
    }
    fn handle_network_set_extra_headers(&self) -> WebDriverBidiResult<()> {
        todo!()
    }

    fn handle_script_add_preload_script(&self) -> WebDriverBidiResult<()> {
        todo!()
    }
    fn handle_script_disown(&self) -> WebDriverBidiResult<()> {
        todo!()
    }
    fn handle_script_call_function(&self) -> WebDriverBidiResult<()> {
        todo!()
    }
    fn handle_script_evaluate(&self) -> WebDriverBidiResult<()> {
        todo!()
    }
    fn handle_script_get_realms(&self) -> WebDriverBidiResult<()> {
        todo!()
    }
    fn handle_script_remove_preload_script(&self) -> WebDriverBidiResult<()> {
        todo!()
    }

    fn handle_storage_get_cookies(&self) -> WebDriverBidiResult<()> {
        todo!()
    }
    fn handle_storage_set_cookie(&self) -> WebDriverBidiResult<()> {
        todo!()
    }
    fn handle_storage_delete_cookies(&self) -> WebDriverBidiResult<()> {
        todo!()
    }

    fn handle_input_perform_actions(&self) -> WebDriverBidiResult<()> {
        todo!()
    }
    fn handle_input_release_actions(&self) -> WebDriverBidiResult<()> {
        todo!()
    }
    fn handle_input_set_files(&self) -> WebDriverBidiResult<()> {
        todo!()
    }

    fn handle_web_extension_install(&self) -> WebDriverBidiResult<()> {
        Err(WebDriverBidiError::unknown(
            "Web Extension is not implemented yet",
        ))
    }
    fn handle_web_extension_uninstall(&self) -> WebDriverBidiResult<()> {
        Err(WebDriverBidiError::unknown(
            "Web Extension is not implemented yet",
        ))
    }
}

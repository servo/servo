//! Decouple handling logic using trait.
//!
//! Also provide a default implementation.

use embedder_traits::{EventLoopWaker, webdriver_bidi::WebDriverBidiCommandMsg};
use rustenium_bidi_definitions::{
    Command, base::CommandMessage, browser::commands::BrowserCommand,
    browsing_context::commands::BrowsingContextCommand, emulation::commands::EmulationCommand,
    input::commands::InputCommand, network::commands::NetworkCommand,
    script::commands::ScriptCommand, session::commands::SessionCommand,
    storage::commands::StorageCommand, web_extension::commands::WebExtensionCommand,
};
use servo_base::id::WebViewId;

use crate::{
    error::{WebDriverBidiError, WebDriverBidiResult},
    model::Message,
    transport::Session,
};

// TODO: should handler be per session?
pub trait WebDriverBidiHandler: Send {
    /// Start processing of a command.
    fn handle(
        &self,
        session: &Option<Session>,
        command: &CommandMessage,
    ) -> WebDriverBidiResult<()>;

    fn try_recv(&self) -> WebDriverBidiResult<(Option<Session>, Message)>;

    // TODO: do we need
    // post update after receiving message
    // fn update(&mut self, message: &Message);
}

// TODO: bidi session has different meaning to classic session.
// and webviewid is not bound to session.

pub struct Handler {
    event_loop_waker: Box<dyn EventLoopWaker>,
    embedder_sender: crossbeam_channel::Sender<WebDriverBidiCommandMsg>,
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
    fn handle(
        &self,
        session: &Option<Session>,
        command: &CommandMessage,
    ) -> WebDriverBidiResult<()> {
        match &command.command_data {
            Command::Browser(browser) => match browser {
                BrowserCommand::Close(close) => self.handle_browser_close(),
                BrowserCommand::CreateUserContext(create_user_context) => {
                    self.handle_browser_create_user_context()
                },
                BrowserCommand::GetClientWindows(get_client_windows) => {
                    self.handle_browser_get_client_windows()
                },
                BrowserCommand::GetUserContexts(get_user_contexts) => {
                    self.handle_browser_get_user_contexts()
                },
                BrowserCommand::RemoveUserContext(remove_user_context) => {
                    self.handle_browser_remove_user_context()
                },
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
                BrowsingContextCommand::Reload(reload) => self.handle_browsing_context_reload(),
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
                SessionCommand::New(_) => self.handle_session_new(),
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
                WebExtensionCommand::Install(install) => self.handle_web_extension_install(),
                WebExtensionCommand::Uninstall(uninstall) => self.handle_web_extension_uninstall(),
            },
        }
    }

    fn try_recv(&self) -> WebDriverBidiResult<(Option<Session>, Message)> {
        todo!()
    }
}

/// Concrete handle methods.
impl Handler {
    fn handle_session_status(&self) -> WebDriverBidiResult<()> {
        todo!()
    }
    fn handle_session_new(&self) -> WebDriverBidiResult<()> {
        todo!()
    }
    fn handle_session_end(&self) -> WebDriverBidiResult<()> {
        todo!()
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
    fn handle_browsing_context_reload(&self) -> WebDriverBidiResult<()> {
        todo!()
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

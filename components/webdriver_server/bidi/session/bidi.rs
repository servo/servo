use std::{
    collections::{HashMap, HashSet},
    hash::Hash,
    sync::LazyLock,
};

use indexmap::{IndexMap, IndexSet};
use servo_base::id::BrowsingContextId;
use tokio::sync::RwLock;
use uuid::Uuid;
use webdriver_traits::{
    WebDriverToConstellationMessage, WebDriverToScriptMessage,
    bidi::{
        BrowserCommand, BrowserResult, BrowsingContextCommand, BrowsingContextResult, CommandData,
        EmptyParams, EmptyResult, EmulationCommand, EmulationResult, ErrorCode, Event,
        InputCommand, InputResult, LogEvent, NetworkCommand, NetworkResult, ResultData,
        ScriptCommand, ScriptResult, SessionCommand, SessionResult, StorageCommand, StorageResult,
        WebExtensionCommand, WebExtensionResult, browser, browsing_context, emulation, input,
        network,
        script::{self, PreloadScript as PreloadScriptId},
        session::{self, Subscription as SubscriptionId},
        storage, web_extension,
    },
};

use crate::bidi::{
    ActiveSessions,
    callback::new_oneshot_callback,
    connection::Connection,
    session::common::{CommonPart, SessionId, SessionMessage},
};

/// BiDi-specific components of a session.
#[derive(Default)]
pub struct BidiPart {
    /// A set of session WebSocket connections associated with this session.
    /// Deviation: we cannot use (ordered) set as we need `iter_mut`.
    /// <https://www.w3.org/TR/webdriver-bidi/#session-websocket-connections>
    pub(crate) connections: Vec<Connection>,

    /// A list of subscriptions for the session.
    /// Deviation: use map so that item removal is more efficient.
    /// <https://www.w3.org/TR/webdriver-bidi/#event-subscriptions>
    pub(crate) subscriptions: HashMap<SubscriptionId, Subscription>,

    /// <https://www.w3.org/TR/webdriver-bidi/#event-known-subscription-ids>
    pub(crate) known_subscription_ids: HashSet<SubscriptionId>,

    /// A map from UUID to preload script.
    /// <https://www.w3.org/TR/webdriver-bidi/#preload-script-map>
    pub(crate) preload_script_map: IndexMap<PreloadScriptId, PreloadScript>,

    /// A map from navigable id to list of log events buffered.
    /// We impose a maximum size of 1000 events per navigable.
    /// <https://www.w3.org/TR/webdriver-bidi/#log-event-buffer>
    pub(crate) log_event_buffer: IndexMap<BrowsingContextId, Vec<LogEvent>>,
}

/// A "subclass" view of BiDi session.
/// With this abstraction, we can follow spec steps more easily.
pub struct BidiSession<'a> {
    pub(crate) id: &'a SessionId,
    pub(crate) common: &'a mut CommonPart,
    pub(crate) bidi: &'a mut BidiPart,
}

impl<'a> BidiSession<'a> {
    // TODO: handle should be changed to &self to avoid blocking each other
    /// Remote end steps, the entry point.
    pub(crate) async fn handle_command(
        &mut self,
        command: &CommandData,
    ) -> Result<ResultData, ErrorCode> {
        match command {
            CommandData::BrowserCommand(cmd) => match cmd {
                BrowserCommand::Close(cmd) => self
                    .handle_browser_close(&cmd.params)
                    .await
                    .map(BrowserResult::CloseResult),
                BrowserCommand::CreateUserContext(cmd) => self
                    .handle_browser_create_user_context(&cmd.params)
                    .await
                    .map(BrowserResult::CreateUserContextResult),
                BrowserCommand::GetClientWindows(cmd) => self
                    .handle_browser_get_client_windows(&cmd.params)
                    .await
                    .map(BrowserResult::GetClientWindowsResult),
                BrowserCommand::GetUserContexts(cmd) => self
                    .handle_browser_get_user_contexts(&cmd.params)
                    .await
                    .map(BrowserResult::GetUserContextsResult),
                BrowserCommand::RemoveUserContext(cmd) => self
                    .handle_browser_remove_user_context(&cmd.params)
                    .await
                    .map(BrowserResult::RemoveUserContextResult),
                BrowserCommand::SetClientWindowState(cmd) => self
                    .handle_browser_set_client_window_state(&cmd.params)
                    .await
                    .map(BrowserResult::SetClientWindowStateResult),
                BrowserCommand::SetDownloadBehavior(cmd) => self
                    .handle_browser_set_download_behavior(&cmd.params)
                    .await
                    .map(BrowserResult::SetDownloadBehaviorResult),
            }
            .map(ResultData::BrowserResult),
            CommandData::BrowsingContextCommand(cmd) => match cmd {
                BrowsingContextCommand::Activate(cmd) => self
                    .handle_browsing_context_activate(&cmd.params)
                    .await
                    .map(BrowsingContextResult::ActivateResult),
                BrowsingContextCommand::CaptureScreenshot(cmd) => self
                    .handle_browsing_context_capture_screenshot(&cmd.params)
                    .await
                    .map(BrowsingContextResult::CaptureScreenshotResult),
                BrowsingContextCommand::Close(cmd) => self
                    .handle_browsing_context_close(&cmd.params)
                    .await
                    .map(BrowsingContextResult::CloseResult),
                BrowsingContextCommand::Create(cmd) => self
                    .handle_browsing_context_create(&cmd.params)
                    .await
                    .map(BrowsingContextResult::CreateResult),
                BrowsingContextCommand::GetTree(cmd) => self
                    .handle_browsing_context_get_tree(&cmd.params)
                    .await
                    .map(BrowsingContextResult::GetTreeResult),
                BrowsingContextCommand::HandleUserPrompt(cmd) => self
                    .handle_browsing_context_handle_user_prompt(&cmd.params)
                    .await
                    .map(BrowsingContextResult::HandleUserPromptResult),
                BrowsingContextCommand::LocateNodes(cmd) => self
                    .handle_browsing_context_locate_nodes(&cmd.params)
                    .await
                    .map(BrowsingContextResult::LocateNodesResult),
                BrowsingContextCommand::Navigate(cmd) => self
                    .handle_browsing_context_navigate(&cmd.params)
                    .await
                    .map(BrowsingContextResult::NavigateResult),
                BrowsingContextCommand::Print(cmd) => self
                    .handle_browsing_context_print(&cmd.params)
                    .await
                    .map(BrowsingContextResult::PrintResult),
                BrowsingContextCommand::Reload(cmd) => self
                    .handle_browsing_context_reload(&cmd.params)
                    .await
                    .map(BrowsingContextResult::ReloadResult),
                BrowsingContextCommand::SetBypassCsp(cmd) => self
                    .handle_browsing_context_set_bypass_csp(&cmd.params)
                    .await
                    .map(BrowsingContextResult::SetBypassCspResult),
                BrowsingContextCommand::SetViewport(cmd) => self
                    .handle_browsing_context_set_viewport(&cmd.params)
                    .await
                    .map(BrowsingContextResult::SetViewportResult),
                BrowsingContextCommand::StartScreencast(cmd) => self
                    .handle_browsing_context_start_screencast(&cmd.params)
                    .await
                    .map(BrowsingContextResult::StartScreencastResult),
                BrowsingContextCommand::StopScreencast(cmd) => self
                    .handle_browsing_context_stop_screencast(&cmd.params)
                    .await
                    .map(BrowsingContextResult::StopScreencastResult),
                BrowsingContextCommand::TraverseHistory(cmd) => self
                    .handle_browsing_context_traverse_history(&cmd.params)
                    .await
                    .map(BrowsingContextResult::TraverseHistoryResult),
            }
            .map(ResultData::BrowsingContextResult),
            CommandData::EmulationCommand(cmd) => match cmd {
                EmulationCommand::SetForcedColorsModeThemeOverride(cmd) => self
                    .handle_emulation_set_forced_colors_mode_theme_overrde(&cmd.params)
                    .await
                    .map(EmulationResult::SetForcedColorsModeThemeOverrideResult),
                EmulationCommand::SetGeolocationOverride(cmd) => self
                    .handle_emulation_set_geolocation_override(&cmd.params)
                    .await
                    .map(EmulationResult::SetGeolocationOverrideResult),
                EmulationCommand::SetLocaleOverride(cmd) => self
                    .handle_emulation_set_locale_override(&cmd.params)
                    .await
                    .map(EmulationResult::SetLocaleOverrideResult),
                EmulationCommand::SetNetworkConditions(cmd) => self
                    .handle_emulation_set_network_conditions(&cmd.params)
                    .await
                    .map(EmulationResult::SetNetworkConditionsResult),
                EmulationCommand::SetScreenOrientationOverride(cmd) => self
                    .handle_emulation_set_screen_orientation_override(&cmd.params)
                    .await
                    .map(EmulationResult::SetScreenOrientationOverrideResult),
                EmulationCommand::SetScreenSettingsOverride(cmd) => self
                    .handle_emulation_set_screen_settings_override(&cmd.params)
                    .await
                    .map(EmulationResult::SetScreenSettingsOverrideResult),
                EmulationCommand::SetScriptingEnabled(cmd) => self
                    .handle_emulation_set_scripting_enabled(&cmd.params)
                    .await
                    .map(EmulationResult::SetScriptingEnabledResult),
                EmulationCommand::SetScrollbarTypeOverride(cmd) => self
                    .handle_emulation_set_scrollbar_type_override(&cmd.params)
                    .await
                    .map(EmulationResult::SetScrollbarTypeOverrideResult),
                EmulationCommand::SetTimezoneOverride(cmd) => self
                    .handle_emulation_set_timezone_override(&cmd.params)
                    .await
                    .map(EmulationResult::SetTimezoneOverrideResult),
                EmulationCommand::SetTouchOverride(cmd) => self
                    .handle_emulation_set_touch_override(&cmd.params)
                    .await
                    .map(EmulationResult::SetTouchOverrideResult),
                EmulationCommand::SetUserAgentOverride(cmd) => self
                    .handle_emulation_set_user_agent_override(&cmd.params)
                    .await
                    .map(EmulationResult::SetUserAgentOverrideResult),
            }
            .map(ResultData::EmulationResult),
            CommandData::InputCommand(cmd) => match cmd {
                InputCommand::PerformActions(cmd) => self
                    .handle_input_perform_actions(&cmd.params)
                    .await
                    .map(InputResult::PerformActionsResult),
                InputCommand::ReleaseActions(cmd) => self
                    .handle_input_release_actions(&cmd.params)
                    .await
                    .map(InputResult::ReleaseActionsResult),
                InputCommand::SetFiles(cmd) => self
                    .handle_input_set_files(&cmd.params)
                    .await
                    .map(InputResult::SetFilesResult),
            }
            .map(ResultData::InputResult),
            CommandData::NetworkCommand(cmd) => match cmd {
                NetworkCommand::AddDataCollector(cmd) => self
                    .handle_network_add_data_collector(&cmd.params)
                    .await
                    .map(NetworkResult::AddDataCollectorResult),
                NetworkCommand::AddIntercept(cmd) => self
                    .handle_network_add_intercept(&cmd.params)
                    .await
                    .map(NetworkResult::AddInterceptResult),
                NetworkCommand::ContinueRequest(cmd) => self
                    .handle_network_continue_request(&cmd.params)
                    .await
                    .map(NetworkResult::ContinueRequestResult),
                NetworkCommand::ContinueResponse(cmd) => self
                    .handle_network_continue_response(&cmd.params)
                    .await
                    .map(NetworkResult::ContinueResponseResult),
                NetworkCommand::ContinueWithAuth(cmd) => self
                    .handle_network_continue_with_auth(&cmd.params)
                    .await
                    .map(NetworkResult::ContinueWithAuthResult),
                NetworkCommand::DisownData(cmd) => self
                    .handle_network_disown_data(&cmd.params)
                    .await
                    .map(NetworkResult::DisownDataResult),
                NetworkCommand::FailRequest(cmd) => self
                    .handle_network_fail_request(&cmd.params)
                    .await
                    .map(NetworkResult::FailRequestResult),
                NetworkCommand::GetData(cmd) => self
                    .handle_network_get_data(&cmd.params)
                    .await
                    .map(NetworkResult::GetDataResult),
                NetworkCommand::ProvideResponse(cmd) => self
                    .handle_network_provide_response(&cmd.params)
                    .await
                    .map(NetworkResult::ProvideResponseResult),
                NetworkCommand::RemoveDataCollector(cmd) => self
                    .handle_network_remove_data_collector(&cmd.params)
                    .await
                    .map(NetworkResult::RemoveDataCollectorResult),
                NetworkCommand::RemoveIntercept(cmd) => self
                    .handle_network_remove_intercept(&cmd.params)
                    .await
                    .map(NetworkResult::RemoveInterceptResult),
                NetworkCommand::SetCacheBehavior(cmd) => self
                    .handle_network_set_cache_behavior(&cmd.params)
                    .await
                    .map(NetworkResult::SetCacheBehaviorResult),
                NetworkCommand::SetExtraHeaders(cmd) => self
                    .handle_network_set_extra_headers(&cmd.params)
                    .await
                    .map(NetworkResult::SetExtraHeadersResult),
            }
            .map(ResultData::NetworkResult),
            CommandData::ScriptCommand(cmd) => match cmd {
                ScriptCommand::AddPreloadScript(cmd) => self
                    .handle_script_add_preload_script(&cmd.params)
                    .await
                    .map(ScriptResult::AddPreloadScriptResult),
                ScriptCommand::CallFunction(cmd) => self
                    .handle_script_call_function(&cmd.params)
                    .await
                    .map(ScriptResult::CallFunctionResult),
                ScriptCommand::Disown(cmd) => self
                    .handle_script_disown(&cmd.params)
                    .await
                    .map(ScriptResult::DisownResult),
                ScriptCommand::Evaluate(cmd) => self
                    .handle_script_evaluate(&cmd.params)
                    .await
                    .map(ScriptResult::EvaluateResult),
                ScriptCommand::GetRealms(cmd) => self
                    .handle_script_get_realms(&cmd.params)
                    .await
                    .map(ScriptResult::GetRealmsResult),
                ScriptCommand::RemovePreloadScript(cmd) => self
                    .handle_script_remove_preload_script(&cmd.params)
                    .await
                    .map(ScriptResult::RemovePreloadScriptResult),
            }
            .map(|r| ResultData::ScriptResult(Box::new(r))),
            CommandData::SessionCommand(cmd) => match cmd {
                SessionCommand::End(cmd) => self.handle_session_end(&cmd.params).await,
                SessionCommand::New(cmd) => self.handle_session_new(&cmd.params).await,
                SessionCommand::Status(cmd) => self.handle_session_status(&cmd.params).await,
                SessionCommand::Subscribe(cmd) => self.handle_session_subscribe(&cmd.params).await,
                SessionCommand::Unsubscribe(cmd) => {
                    self.handle_session_unsubscribe(&cmd.params).await
                },
            },
            CommandData::StorageCommand(cmd) => match cmd {
                StorageCommand::GetCookies(cmd) => self
                    .handle_storage_get_cookies(&cmd.params)
                    .await
                    .map(StorageResult::GetCookiesResult),
                StorageCommand::SetCookie(cmd) => self
                    .handle_storage_set_cookie(&cmd.params)
                    .await
                    .map(StorageResult::SetCookieResult),
                StorageCommand::DeleteCookies(cmd) => self
                    .handle_storage_delete_cookies(&cmd.params)
                    .await
                    .map(StorageResult::DeleteCookiesResult),
            }
            .map(|r| ResultData::StorageResult(Box::new(r))),
            CommandData::WebExtensionCommand(cmd) => match cmd {
                WebExtensionCommand::Install(cmd) => self
                    .handle_web_extension_install(&cmd.params)
                    .await
                    .map(WebExtensionResult::InstallResult),
                WebExtensionCommand::Uninstall(cmd) => self
                    .handle_web_extension_uninstall(&cmd.params)
                    .await
                    .map(WebExtensionResult::UninstallResult),
            }
            .map(ResultData::WebExtensionResult),
        }
    }

    /// <https://www.w3.org/TR/webdriver-bidi/#command-session-status>
    async fn handle_session_status(&mut self, _: &EmptyParams) -> Result<ResultData, ErrorCode> {
        // 1.
        let body = session::StatusResult {
            // Though BiDi spec does not mention this,
            // we infer from classsic spec that ready should be false
            ready: false,
            // implementation-defined
            message: "".to_string(),
        };
        // 2.
        Ok(ResultData::SessionResult(SessionResult::StatusResult(body)))
    }

    /// <https://www.w3.org/TR/webdriver-bidi/#command-session-new>
    async fn handle_session_new(
        &mut self,
        _: &session::NewParameters,
    ) -> Result<ResultData, ErrorCode> {
        // 1. in bidi session, session if not null
        Err(ErrorCode::SessionNotCreated)
        // 2-9. SKIP
    }

    /// <https://www.w3.org/TR/webdriver-bidi/#command-session-end>
    async fn handle_session_end(&mut self, _: &EmptyParams) -> Result<ResultData, ErrorCode> {
        // 1.
        self.end_the_session().await;
        // 3. cleanup should happens after response, see `handle_receiver`
        if let Err(e) = self.common.session_sender.send(SessionMessage::Cleanup) {
            log::warn!("Cleanup message sent failed: {e:?}");
        };
        // 2.
        Ok(ResultData::SessionResult(SessionResult::EndResult(
            EmptyResult {
                extensible: Default::default(),
            },
        )))
    }

    /// <https://www.w3.org/TR/webdriver-bidi/#command-session-subscribe>
    async fn handle_session_subscribe(
        &mut self,
        command_parameters: &session::SubscribeParameters,
    ) -> Result<ResultData, ErrorCode> {
        // 1.
        let mut event_names = HashSet::<String>::new();
        // 2.
        for name in &command_parameters.events {
            // union is slow, insert instead
            for event_name in obtain_a_set_of_event_names(&name)? {
                event_names.insert(event_name);
            }
        }
        // 3.
        let input_user_context_ids: HashSet<_> = command_parameters
            .user_contexts
            .iter()
            .flat_map(|s| s.iter())
            .collect();
        // 4.
        let input_context_ids: HashSet<_> = command_parameters
            .contexts
            .iter()
            .flat_map(|s| s.iter())
            .collect();
        // 5.
        if !input_user_context_ids.is_empty() && !input_context_ids.is_empty() {
            return Err(ErrorCode::InvalidArgument);
        }
        // 6.
        let subscription_naviables = HashSet::<()>::new();
        // 7.
        let top_level_traversable_context_ids = HashSet::new();
        // 8.
        if !input_context_ids.is_empty() {
            // 8.1.
            // 8.2.
            // 8.3.
            // 8.3.1.
        }
        // 9.
        else if !input_user_context_ids.is_empty() {
            // 9.1.
            for user_context_id in input_user_context_ids {
                // TODO: user context not implemented
            }
        }
        // 10. TODO: should have event to sync from constellation
        // 11.
        let subscription = Subscription {
            id: Uuid::new_v4(),
            event_names: event_names.clone(),
            top_level_traversable_ids: top_level_traversable_context_ids,
            user_context_ids: Default::default(),
        };
        // 12.
        let subscribe_step_events = HashMap::<(), ()>::new();
        // 13.
        for event_name in event_names.iter() {
            // 13.1.
            if !EVENT_NAMES.contains(event_name) {
                continue;
            }
            // 13.2. TODO: how to compute existing
            // 13.3. TODO:
        }
        // 14.
        let subscription_id = subscription.id;
        let subscription_is_global = subscription.is_global();
        self.bidi
            .subscriptions
            .insert(subscription_id, subscription);
        // 15.
        self.bidi.known_subscription_ids.insert(subscription_id);
        // 16. TODO: sort, subscribe step, how to dispatch for each event
        // 17.
        let include_global = subscription_is_global;
        // 18.
        for event_name in event_names {
            // 18.1.
            self.subscribe_event(&event_name, include_global).await;
        }
        // 19.
        let body = session::SubscribeResult {
            subscription: subscription_id,
        };
        // 20.
        Ok(ResultData::SessionResult(SessionResult::SubscribeResult(
            body,
        )))
    }

    /// <https://www.w3.org/TR/webdriver-bidi/#command-session-unsubscribe>
    async fn handle_session_unsubscribe(
        &mut self,
        command_parameters: &session::UnsubscribeParameters,
    ) -> Result<ResultData, ErrorCode> {
        match command_parameters {
            // 1.
            session::UnsubscribeParameters::UnsubscribeByAttributesRequest(command_parameters) => {
                // 1.1.
                let mut event_names = HashSet::<String>::new();
                // 1.2.
                for name in command_parameters.events.iter() {
                    // union is slow, insert instead
                    for event_name in obtain_a_set_of_event_names(&name)? {
                        event_names.insert(event_name);
                    }
                }
                // 1.3.
                let mut new_subscriptions = HashMap::new();
                // 1.4.
                let mut matched_events = HashSet::<String>::new();
                // 1.5.
                for (id, subscription) in self.bidi.subscriptions.drain() {
                    // 1.5.1.
                    if subscription.event_names.intersection(&event_names).count() == 0 {
                        // 1.5.1.1
                        new_subscriptions.insert(id, subscription);
                        // 1.5.1.2
                        continue;
                    }
                    // 1.5.2.
                    if !subscription.is_global() {
                        // 1.5.2.1.
                        new_subscriptions.insert(id, subscription);
                        // 1.5.2.2.
                        continue;
                    }
                    // 1.5.3.
                    let mut subscription_event_names = subscription.event_names.clone();
                    // 1.5.4.
                    for event_name in event_names.iter() {
                        // 1.5.4.1.
                        if subscription_event_names.contains(event_name) {
                            // 1.5.4.1.1.
                            matched_events.insert(event_name.clone());
                            // 1.5.4.1.2.
                            subscription_event_names.remove(event_name);
                        }
                    }
                    // 1.5.5.
                    if !subscription_event_names.is_empty() {
                        // 1.5.5.1
                        let cloned_subscription = Subscription {
                            id: subscription.id,
                            event_names: subscription_event_names.clone(),
                            top_level_traversable_ids: Default::default(),
                            user_context_ids: Default::default(),
                        };
                        // 1.5.5.2.
                        new_subscriptions.insert(cloned_subscription.id, cloned_subscription);
                    }
                }
                // 1.6.
                if matched_events != event_names {
                    return Err(ErrorCode::InvalidArgument);
                }
                // 1.7.
                self.bidi.subscriptions = new_subscriptions;
            },
            // 2.
            session::UnsubscribeParameters::UnsubscribeByIdRequest(command_parameters) => {
                // 2.1.
                let subscriptions: HashSet<_> =
                    command_parameters.subscriptions.iter().copied().collect();
                // 2.2.
                let unknown_subscription_ids =
                    subscriptions.difference(&self.bidi.known_subscription_ids);
                // 2.3.
                if unknown_subscription_ids.count() == 0 {
                    // 2.3.1.
                    return Err(ErrorCode::InvalidArgument);
                }
                // 2.4.
                let mut subscriptions_to_remove = HashSet::new();
                // 2.5. we use idx instead
                for subsription in self.bidi.subscriptions.keys() {
                    // 2.5.1.
                    if subscriptions.contains(&subsription) {
                        // 2.5.1.1.
                        subscriptions_to_remove.insert(*subsription);
                    }
                }
                // 2.6. remove is more efficient
                for subscription in subscriptions {
                    self.bidi.known_subscription_ids.remove(&subscription);
                }
                // 2.7.
                for subscription in subscriptions_to_remove {
                    self.bidi.subscriptions.remove(&subscription);
                }
            },
        }
        // 3.
        Ok(ResultData::SessionResult(SessionResult::UnsubscribeResult(
            EmptyResult {
                extensible: Default::default(),
            },
        )))
    }

    /// <https://www.w3.org/TR/webdriver-bidi/#command-browser-close>
    async fn handle_browser_close(
        &self,
        command_parameters: &EmptyParams,
    ) -> Result<browser::CloseResult, ErrorCode> {
        todo!()
    }

    /// <https://www.w3.org/TR/webdriver-bidi/#command-browser-createUsetContext>
    async fn handle_browser_create_user_context(
        &self,
        _: &browser::CreateUserContextParameters,
    ) -> Result<browser::CreateUserContextResult, ErrorCode> {
        // TODO: blocked by user context not implemented
        Err(ErrorCode::UnknownError)
    }

    /// <https://www.w3.org/TR/webdriver-bidi/#command-browser-getUsetContexts>
    async fn handle_browser_get_user_contexts(
        &self,
        _: &EmptyParams,
    ) -> Result<browser::GetUserContextsResult, ErrorCode> {
        // TODO: blocked by user context not implemented
        Err(ErrorCode::UnknownError)
    }

    /// <https://www.w3.org/TR/webdriver-bidi/#command-browser-getClientWindows>
    async fn handle_browser_get_client_windows(
        &self,
        command_parameters: &EmptyParams,
    ) -> Result<browser::GetClientWindowsResult, ErrorCode> {
        todo!()
    }

    /// <https://www.w3.org/TR/webdriver-bidi/#command-browser-removeUserContext>
    async fn handle_browser_remove_user_context(
        &self,
        _: &browser::RemoveUserContextParameters,
    ) -> Result<browser::RemoveUserContextResult, ErrorCode> {
        // TODO: blocked by user context not implemented
        Err(ErrorCode::UnknownError)
    }

    /// <https://www.w3.org/TR/webdriver-bidi/#command-browser-setClientWindowState>
    async fn handle_browser_set_client_window_state(
        &self,
        command_parameters: &browser::SetClientWindowStateParameters,
    ) -> Result<browser::SetClientWindowStateResult, ErrorCode> {
        todo!()
    }

    /// <https://www.w3.org/TR/webdriver-bidi/#command-browser-setDownloadBehavior>
    async fn handle_browser_set_download_behavior(
        &self,
        _: &browser::SetDownloadBehaviorParameters,
    ) -> Result<browser::SetDownloadBehaviorResult, ErrorCode> {
        // TODO: blocked by download not implemented
        Err(ErrorCode::UnknownError)
    }

    /// <https://www.w3.org/TR/webdriver-bidi/#command-browsingContext-activate>
    async fn handle_browsing_context_activate(
        &self,
        command_parameters: &browsing_context::ActivateParameters,
    ) -> Result<browsing_context::ActivateResult, ErrorCode> {
        // 1.
        let navigable_id = BrowsingContextId::from_string(&command_parameters.context)
            .ok_or(ErrorCode::InvalidArgument)?;
        // 2.
        let navigable = self
            .common
            .remote_end_state
            .get_a_navigable(navigable_id)
            .await?
            .unwrap();
        // 3.
        if navigable.webview_id.is_none() {
            return Err(ErrorCode::InvalidArgument);
        }
        // 4.
        self.activate_a_navigable(navigable_id).await
    }

    /// <https://www.w3.org/TR/webdriver-bidi/#activate-a-navigable>
    async fn activate_a_navigable(
        &self,
        navigable_id: BrowsingContextId,
    ) -> Result<EmptyResult, ErrorCode> {
        // TODO:
        // 1-2. continue in constellation
        let (callback, receiver) = new_oneshot_callback();
        self.send_to_constellation(WebDriverToConstellationMessage::Activate(
            navigable_id,
            callback,
        ));
        let possible = receiver
            .await
            .map_err(|_| ErrorCode::UnknownError)?
            .map_err(|_| ErrorCode::UnknownError)?;
        if !possible {
            return Err(ErrorCode::UnsupportedOperation);
        }
        // 3.
        Ok(EmptyResult {
            extensible: Default::default(),
        })
    }

    // TODO: link
    async fn handle_browsing_context_capture_screenshot(
        &self,
        command_parameters: &browsing_context::CaptureScreenshotParameters,
    ) -> Result<browsing_context::CaptureScreenshotResult, ErrorCode> {
        // TODO:
        todo!()
    }

    // TODO: change to &self
    async fn handle_browsing_context_close(
        &mut self,
        command_parameters: &browsing_context::CloseParameters,
    ) -> Result<browsing_context::CloseResult, ErrorCode> {
        // 1.
        let navigable_id = BrowsingContextId::from_string(&command_parameters.context)
            .ok_or(ErrorCode::InvalidArgument)?;
        // 2.
        let prompt_unload = command_parameters.prompt_unload.unwrap_or(false);
        // 3.
        let navigable = self
            .common
            .remote_end_state
            .get_a_navigable(navigable_id)
            .await?
            .unwrap();
        // 4. SKIP: Assert
        // 5.
        if navigable.webview_id.is_none() {
            return Err(ErrorCode::InvalidArgument);
        }
        // 6. & 7.
        let (callback, receiver) = new_oneshot_callback();
        navigable.send_to_script(WebDriverToScriptMessage::CloseNavigable(
            prompt_unload,
            callback,
        ));
        if let Err(e) = receiver.await {
            log::warn!("Receiving callback failed: {e:?}");
            return Err(ErrorCode::UnknownError);
        }
        // 8.
        Ok(EmptyResult {
            extensible: Default::default(),
        })
    }

    // TODO: link
    async fn handle_browsing_context_create(
        &self,
        command_parameters: &browsing_context::CreateParameters,
    ) -> Result<browsing_context::CreateResult, ErrorCode> {
        // TODO:
        todo!()
    }

    /// <https://www.w3.org/TR/webdriver-bidi/#command-browsingContext-getTree>
    async fn handle_browsing_context_get_tree(
        &self,
        command_parameters: &browsing_context::GetTreeParameters,
    ) -> Result<browsing_context::GetTreeResult, ErrorCode> {
        // 1.
        let root_id = &command_parameters.root;
        // 2.
        let max_depth = command_parameters.max_depth;
        // 3.
        let mut navigables = vec![];
        // 4.
        if let Some(root_id) = root_id {
            let root_id =
                BrowsingContextId::from_string(root_id).ok_or(ErrorCode::InvalidArgument)?;
            navigables.push(
                self.common
                    .remote_end_state
                    .get_a_navigable(root_id)
                    .await?
                    .unwrap(),
            );
        } else {
            for navigable in self
                .common
                .remote_end_state
                .navigables
                .read()
                .await
                .values()
            {
                // top-level only
                // TODO: better distinguishable
                if navigable.webview_id.is_some() {
                    navigables.push(navigable.clone());
                }
            }
        }
        // 5.
        let mut navigable_infos = vec![];
        // 6.
        for navigable in navigables {
            // 6.1.
            let info = navigable.get_the_navigable_info(max_depth);
            navigable_infos.push(info);
        }
        // 7.
        let body = browsing_context::GetTreeResult {
            contexts: navigable_infos,
        };
        // 8.
        Ok(body)
    }

    /// <https://www.w3.org/TR/webdriver-bidi/#command-browsingContext-handleUserPrompt>
    async fn handle_browsing_context_handle_user_prompt(
        &self,
        command_parameters: &browsing_context::HandleUserPromptParameters,
    ) -> Result<browsing_context::HandleUserPromptResult, ErrorCode> {
        // TODO: should be done in embedder thread
        // TODO: should design a webdriver to embedder msg
        todo!()
    }

    /// <https://www.w3.org/TR/webdriver-bidi/#command-browsingContext-locateNodes>
    async fn handle_browsing_context_locate_nodes(
        &self,
        command_parameters: &browsing_context::LocateNodesParameters,
    ) -> Result<browsing_context::LocateNodesResult, ErrorCode> {
        // TODO:
        todo!()
    }

    /// <https://www.w3.org/TR/webdriver-bidi/#command-browsingContext-navigate>
    async fn handle_browsing_context_navigate(
        &self,
        command_parameters: &browsing_context::NavigateParameters,
    ) -> Result<browsing_context::NavigateResult, ErrorCode> {
        // 1.
        let navigable_id = BrowsingContextId::from_string(&command_parameters.context)
            .ok_or(ErrorCode::InvalidArgument)?;
        // 2.
        let navigable = self
            .common
            .remote_end_state
            .get_a_navigable(navigable_id)
            .await?
            .unwrap();
        // 3. SKIP: Assert
        // 4.
        let wait_condition = "committed";
        // 5.
        if let Some(wait) = &command_parameters.wait
            && !matches!(wait, browsing_context::ReadinessState::None)
        {
            // TODO: set wait_condition, they have different type
        }
        // 6.
        let url = &command_parameters.url;
        // 7.
        let document = navigable.active_document;
        // 8. TODO: we have not receive url information of a document
        let base = "".to_string();
        // 9. TODO: import servo-url parser
        let url_record = false;
        // 10.
        if !url_record {
            return Err(ErrorCode::InvalidArgument);
        }
        // 11. TODO: id
        let request_id = 0;
        self.send_to_constellation(WebDriverToConstellationMessage::Request("".to_string()));
        // 12.
        self.await_a_navigation().await
    }

    /// <https://www.w3.org/TR/webdriver-bidi/#command-browsingContext-print>
    async fn handle_browsing_context_print(
        &self,
        _: &browsing_context::PrintParameters,
    ) -> Result<browsing_context::PrintResult, ErrorCode> {
        // TODO: blocked by PDF not implemented
        Err(ErrorCode::UnknownError)
    }

    /// <https://www.w3.org/TR/webdriver-bidi/#command-browsingContext-reload>
    async fn handle_browsing_context_reload(
        &self,
        command_parameters: &browsing_context::ReloadParameters,
    ) -> Result<browsing_context::ReloadResult, ErrorCode> {
        // 1.
        let navigable_id = BrowsingContextId::from_string(&command_parameters.context)
            .ok_or(ErrorCode::InvalidArgument)?;
        // 2.
        let navigable = self
            .common
            .remote_end_state
            .get_a_navigable(navigable_id)
            .await?;
        // 3. SKIP: assert
        // 4.
        let ignore_cache = command_parameters.ignore_cache;
        // 5.
        let wait_condition = "commited";
        // 6. TODO: wait and wait condition have different type
        // 7. TODO: since active document, this should be sent to script thread to handle
        // 8.
        // 9.
        let request_id = 0;
        // TODO: this is different from classic, in classic it is top level, but in bidi it seems to be arbitary.
        // instead, we should send it to navigable (script thread).
        // TODO: should allow empty or use seperate variant?
        self.send_to_constellation(WebDriverToConstellationMessage::Request("".to_string()));
        // 10.
        todo!()
    }

    /// <https://www.w3.org/TR/webdriver-bidi/#command-browsingContext-setBypassCSP>
    async fn handle_browsing_context_set_bypass_csp(
        &self,
        command_parameters: &browsing_context::SetBypassCspParameters,
    ) -> Result<browsing_context::SetBypassCspResult, ErrorCode> {
        // TODO:
        todo!()
    }

    /// <https://www.w3.org/TR/webdriver-bidi/#command-browsingContext-setViewport>
    async fn handle_browsing_context_set_viewport(
        &self,
        _: &browsing_context::SetViewportParameters,
    ) -> Result<browsing_context::SetViewportResult, ErrorCode> {
        // TODO: blocked by viewport not actually implemented
        Err(ErrorCode::UnknownError)
    }

    /// <https://www.w3.org/TR/webdriver-bidi/#command-browsingContext-startScreencast>
    async fn handle_browsing_context_start_screencast(
        &self,
        _: &browsing_context::StartScreencastParameters,
    ) -> Result<browsing_context::StartScreencastResult, ErrorCode> {
        // TODO: spec is actively changing, deferred
        Err(ErrorCode::UnsupportedOperation)
    }

    /// <https://www.w3.org/TR/webdriver-bidi/#command-browsingContext-stopScreencast>
    async fn handle_browsing_context_stop_screencast(
        &self,
        _: &browsing_context::StopScreencastParameters,
    ) -> Result<browsing_context::StopScreencastResult, ErrorCode> {
        // TODO: spec is actively changing, deferred
        Err(ErrorCode::UnsupportedOperation)
    }

    /// <https://www.w3.org/TR/webdriver-bidi/#command-browsingContext-traverseHistory>
    async fn handle_browsing_context_traverse_history(
        &self,
        command_parameters: &browsing_context::TraverseHistoryParameters,
    ) -> Result<browsing_context::TraverseHistoryResult, ErrorCode> {
        // TODO:
        todo!()
    }

    /// <https://www.w3.org/TR/webdriver-bidi/#command-emulation-setForcedColorsModeThemeOverride>
    async fn handle_emulation_set_forced_colors_mode_theme_overrde(
        &self,
        _: &emulation::SetForcedColorsModeThemeOverrideParameters,
    ) -> Result<emulation::SetForcedColorsModeThemeOverrideResult, ErrorCode> {
        // TODO: blocked by forced colors mode not implemented
        Err(ErrorCode::UnknownError)
    }

    /// <https://www.w3.org/TR/webdriver-bidi/#command-emulation-setGeolocationOverride>
    async fn handle_emulation_set_geolocation_override(
        &self,
        _: &emulation::SetGeolocationOverrideParameters,
    ) -> Result<emulation::SetGeolocationOverrideResult, ErrorCode> {
        // TODO: blocked by geolocation not implemented
        Err(ErrorCode::UnknownError)
    }

    /// <https://www.w3.org/TR/webdriver-bidi/#command-emulation-setLocaleOverride>
    async fn handle_emulation_set_locale_override(
        &self,
        _: &emulation::SetLocaleOverrideParameters,
    ) -> Result<emulation::SetLocaleOverrideResult, ErrorCode> {
        // TODO: blocked by `CURRENT_LOCAL` is `OnceLock`
        Err(ErrorCode::UnknownError)
    }

    /// <https://www.w3.org/TR/webdriver-bidi/#command-emulation-setNetworkConditions>
    async fn handle_emulation_set_network_conditions(
        &self,
        _: &emulation::SetNetworkConditionsParameters,
    ) -> Result<emulation::SetNetworkConditionsResult, ErrorCode> {
        // TODO: blocked by network condition not implemented
        Err(ErrorCode::UnknownError)
    }

    /// <https://www.w3.org/TR/webdriver-bidi/#command-emulation-setScreenSettingsOverride>
    async fn handle_emulation_set_screen_settings_override(
        &self,
        _: &emulation::SetScreenSettingsOverrideParameters,
    ) -> Result<emulation::SetScreenSettingsOverrideResult, ErrorCode> {
        // TODO: blocked, need to add a layer
        Err(ErrorCode::UnknownError)
    }

    /// <https://www.w3.org/TR/webdriver-bidi/#command-emulation-setScreenSettingsOverride>
    async fn handle_emulation_set_scripting_enabled(
        &self,
        _: &emulation::SetScriptingEnabledParameters,
    ) -> Result<emulation::SetScriptingEnabledResult, ErrorCode> {
        // TODO: blocked, need a flag
        Err(ErrorCode::UnknownError)
    }

    /// <https://www.w3.org/TR/webdriver-bidi/#command-emulation-setScreenOrientationOverride>
    async fn handle_emulation_set_screen_orientation_override(
        &self,
        _: &emulation::SetScreenOrientationOverrideParameters,
    ) -> Result<emulation::SetScreenOrientationOverrideResult, ErrorCode> {
        // TODO: blocked by screen orientation not implemented
        Err(ErrorCode::UnknownError)
    }

    /// <https://www.w3.org/TR/webdriver-bidi/#command-emulation-setScrollbarTypeOverride>
    async fn handle_emulation_set_scrollbar_type_override(
        &self,
        _: &emulation::SetScrollbarTypeOverrideParameters,
    ) -> Result<emulation::SetScrollbarTypeOverrideResult, ErrorCode> {
        // TODO: blocked by scrollbar type not implemented
        Err(ErrorCode::UnknownError)
    }

    /// <https://www.w3.org/TR/webdriver-bidi/#command-emulation-setTimezoneOverride>
    async fn handle_emulation_set_timezone_override(
        &self,
        _: &emulation::SetTimezoneOverrideParameters,
    ) -> Result<emulation::SetTimezoneOverrideResult, ErrorCode> {
        // TODO: blocked, `setRealmTimezoneOverride` is not exported by `mozjs`
        Err(ErrorCode::UnknownError)
    }

    /// <https://www.w3.org/TR/webdriver-bidi/#command-emulation-setTouchOverride>
    async fn handle_emulation_set_touch_override(
        &self,
        _: &emulation::SetTouchOverrideParameters,
    ) -> Result<emulation::SetTouchOverrideResult, ErrorCode> {
        // TODO: blocked,by max touch not implemented
        Err(ErrorCode::UnknownError)
    }

    /// <https://www.w3.org/TR/webdriver-bidi/#command-emulation-setUserAgentOverride>
    async fn handle_emulation_set_user_agent_override(
        &self,
        _: &emulation::SetUserAgentOverrideParameters,
    ) -> Result<emulation::SetUserAgentOverrideResult, ErrorCode> {
        // TODO: blocked by user agent being global
        // TODO: this may be easy to implement
        Err(ErrorCode::UnknownError)
    }

    /// <https://www.w3.org/TR/webdriver-bidi/#command-network-addDataCollector>
    async fn handle_network_add_data_collector(
        &self,
        _: &network::AddDataCollectorParameters,
    ) -> Result<network::AddDataCollectorResult, ErrorCode> {
        todo!()
    }

    /// <https://www.w3.org/TR/webdriver-bidi/#command-network-addIntercept>
    async fn handle_network_add_intercept(
        &self,
        _: &network::AddInterceptParameters,
    ) -> Result<network::AddInterceptResult, ErrorCode> {
        todo!()
    }

    /// <https://www.w3.org/TR/webdriver-bidi/#command-network-continueRequest>
    async fn handle_network_continue_request(
        &self,
        _: &network::ContinueRequestParameters,
    ) -> Result<network::ContinueRequestResult, ErrorCode> {
        todo!()
    }

    /// <https://www.w3.org/TR/webdriver-bidi/#command-network-continueResponse>
    async fn handle_network_continue_response(
        &self,
        _: &network::ContinueResponseParameters,
    ) -> Result<network::ContinueResponseResult, ErrorCode> {
        todo!()
    }

    /// <https://www.w3.org/TR/webdriver-bidi/#command-network-continueWithAuth>
    async fn handle_network_continue_with_auth(
        &self,
        _: &network::ContinueWithAuthParameters,
    ) -> Result<network::ContinueWithAuthResult, ErrorCode> {
        todo!()
    }

    /// <https://www.w3.org/TR/webdriver-bidi/#command-network-disownData>
    async fn handle_network_disown_data(
        &self,
        _: &network::DisownDataParameters,
    ) -> Result<network::DisownDataResult, ErrorCode> {
        todo!()
    }

    /// <https://www.w3.org/TR/webdriver-bidi/#command-network-failRequest>
    async fn handle_network_fail_request(
        &self,
        _: &network::FailRequestParameters,
    ) -> Result<network::FailRequestResult, ErrorCode> {
        todo!()
    }

    /// <https://www.w3.org/TR/webdriver-bidi/#command-network-getData>
    async fn handle_network_get_data(
        &self,
        _: &network::GetDataParameters,
    ) -> Result<network::GetDataResult, ErrorCode> {
        todo!()
    }

    /// <https://www.w3.org/TR/webdriver-bidi/#command-network-provideResponse>
    async fn handle_network_provide_response(
        &self,
        _: &network::ProvideResponseParameters,
    ) -> Result<network::ProvideResponseResult, ErrorCode> {
        todo!()
    }

    /// <https://www.w3.org/TR/webdriver-bidi/#command-network-removeDataCollector>
    async fn handle_network_remove_data_collector(
        &self,
        _: &network::RemoveDataCollectorParameters,
    ) -> Result<network::RemoveDataCollectorResult, ErrorCode> {
        todo!()
    }

    /// <https://www.w3.org/TR/webdriver-bidi/#command-network-removeIntercept>
    async fn handle_network_remove_intercept(
        &self,
        _: &network::RemoveInterceptParameters,
    ) -> Result<network::RemoveInterceptResult, ErrorCode> {
        todo!()
    }

    /// <https://www.w3.org/TR/webdriver-bidi/#command-network-setCacheBehavior>
    async fn handle_network_set_cache_behavior(
        &self,
        _: &network::SetCacheBehaviorParameters,
    ) -> Result<network::SetCacheBehaviorResult, ErrorCode> {
        todo!()
    }

    /// <https://www.w3.org/TR/webdriver-bidi/#command-network-setExtraHeaders>
    async fn handle_network_set_extra_headers(
        &self,
        _: &network::SetExtraHeadersParameters,
    ) -> Result<network::SetExtraHeadersResult, ErrorCode> {
        todo!()
    }

    /// <https://www.w3.org/TR/webdriver-bidi/#command-script-addPreloadScript>
    async fn handle_script_add_preload_script(
        &self,
        _: &script::AddPreloadScriptParameters,
    ) -> Result<script::AddPreloadScriptResult, ErrorCode> {
        todo!()
    }

    /// <https://www.w3.org/TR/webdriver-bidi/#command-script-disown>
    async fn handle_script_disown(
        &self,
        _: &script::DisownParameters,
    ) -> Result<script::DisownResult, ErrorCode> {
        todo!()
    }

    /// <https://www.w3.org/TR/webdriver-bidi/#command-script-callFunction>
    async fn handle_script_call_function(
        &self,
        _: &script::CallFunctionParameters,
    ) -> Result<script::CallFunctionResult, ErrorCode> {
        todo!()
    }

    /// <https://www.w3.org/TR/webdriver-bidi/#command-script-evaluate>
    async fn handle_script_evaluate(
        &self,
        _: &script::EvaluateParameters,
    ) -> Result<script::EvaluateResult, ErrorCode> {
        todo!()
    }

    /// <https://www.w3.org/TR/webdriver-bidi/#command-script-getRealms>
    async fn handle_script_get_realms(
        &self,
        _: &script::GetRealmsParameters,
    ) -> Result<script::GetRealmsResult, ErrorCode> {
        todo!()
    }

    /// <https://www.w3.org/TR/webdriver-bidi/#command-script-removePreloadScript>
    async fn handle_script_remove_preload_script(
        &self,
        _: &script::RemovePreloadScriptParameters,
    ) -> Result<script::RemovePreloadScriptResult, ErrorCode> {
        todo!()
    }

    /// <https://www.w3.org/TR/webdriver-bidi/#command-storage-getGookies>
    async fn handle_storage_get_cookies(
        &self,
        command_parameters: &storage::GetCookiesParameters,
    ) -> Result<storage::GetCookiesResult, ErrorCode> {
        // 1.
        let filter = command_parameters
            .filter
            .clone()
            .unwrap_or(storage::CookieFilter {
                name: None,
                value: None,
                domain: None,
                path: None,
                size: None,
                http_only: None,
                secure: None,
                same_site: None,
                expiry: None,
                extensible: Default::default(),
            });
        // 2.
        let partition_spec = &command_parameters.partition;
        // 3.
        let partition_key = self
            .expand_a_storage_partition(partition_spec.clone())
            .await?;
        // TODO: 4-7. may should happen in resource thread.
        // here we do not need to query script thread because associated storage partition is synced before.
        // cookie store can be defined as a key-channel pair to resource thread
        // 4. TODO: get cookie store
        // 5. TODO: get matching cookies
        // 6.
        // 7.
        let serialized_cookies = vec![];
        // 8.
        let body = storage::GetCookiesResult {
            cookies: serialized_cookies,
            partition_key,
        };
        // 9.
        Ok(body)
    }

    /// <https://www.w3.org/TR/webdriver-bidi/#command-storage-setCookie>
    async fn handle_storage_set_cookie(
        &self,
        command_parameters: &storage::SetCookieParameters,
    ) -> Result<storage::SetCookieResult, ErrorCode> {
        // 1.
        let cookie_spec = &command_parameters.cookie;
        // 2.
        let partition_sepc = command_parameters.partition.clone();
        // 3.
        let partition_key = self.expand_a_storage_partition(partition_sepc).await?;
        // 4.
        // 5-6. TODO: continue in resource thread
        // 7.
        let body = storage::SetCookieResult { partition_key };
        // 8.
        Ok(body)
    }

    /// <https://www.w3.org/TR/webdriver-bidi/#command-storage-deleteCookies>
    async fn handle_storage_delete_cookies(
        &self,
        command_parameters: &storage::DeleteCookiesParameters,
    ) -> Result<storage::DeleteCookiesResult, ErrorCode> {
        // 1.
        let filter = command_parameters
            .filter
            .clone()
            .unwrap_or(storage::CookieFilter {
                name: None,
                value: None,
                domain: None,
                path: None,
                size: None,
                http_only: None,
                secure: None,
                same_site: None,
                expiry: None,
                extensible: Default::default(),
            });
        // 2.
        let partition_spec = &command_parameters.partition;
        // 3.
        let partition_key = self
            .expand_a_storage_partition(partition_spec.clone())
            .await?;
        // 4-6. TODO: continue in resource thread
        // 7.
        let body = storage::DeleteCookiesResult { partition_key };
        // 8.
        Ok(body)
    }

    /// <https://www.w3.org/TR/webdriver-bidi/#command-input-performActions>
    async fn handle_input_perform_actions(
        &self,
        _: &input::PerformActionsParameters,
    ) -> Result<input::PerformActionsResult, ErrorCode> {
        todo!()
    }

    /// <https://www.w3.org/TR/webdriver-bidi/#command-input-releaseActions>
    async fn handle_input_release_actions(
        &self,
        _: &input::ReleaseActionsParameters,
    ) -> Result<input::ReleaseActionsResult, ErrorCode> {
        todo!()
    }

    /// <https://www.w3.org/TR/webdriver-bidi/#command-input-setFiles>
    async fn handle_input_set_files(
        &self,
        _: &input::SetFilesParameters,
    ) -> Result<input::SetFilesResult, ErrorCode> {
        todo!()
    }

    /// <https://www.w3.org/TR/webdriver-bidi/#expand-a-partition-key>
    async fn expand_a_storage_partition(
        &self,
        partition_spec: Option<storage::PartitionDescriptor>,
    ) -> Result<storage::PartitionKey, ErrorCode> {
        // TODO
        todo!()
    }

    /// <https://www.w3.org/TR/webdriver-bidi/#command-webExtension-install>
    async fn handle_web_extension_install(
        &self,
        _: &web_extension::InstallParameters,
    ) -> Result<web_extension::InstallResult, ErrorCode> {
        // TODO: blocked by web extension not implemented
        Err(ErrorCode::UnsupportedOperation)
    }

    /// <https://www.w3.org/TR/webdriver-bidi/#command-webExtension-uninstall>
    async fn handle_web_extension_uninstall(
        &self,
        _: &web_extension::UninstallParameters,
    ) -> Result<web_extension::UninstallResult, ErrorCode> {
        // TODO: blocked by web extension not implemented
        Err(ErrorCode::UnsupportedOperation)
    }

    /// The "remote end subscribe" step for the event
    async fn subscribe_event(&mut self, event_name: &str, include_global: bool) {
        // TODO: match event_name and dispatch e.g. subscribe_log_entry_added
        match event_name {
            "log.entryAdded" => self.subscribe_log_entry_added(include_global).await,
            _ => {},
        }
    }

    async fn subscribe_log_entry_added(&mut self, include_global: bool) {
        // TODO:
    }

    /// <https://www.w3.org/TR/webdriver-bidi/#await-a-navigation>
    async fn await_a_navigation(&self) -> Result<browsing_context::NavigateResult, ErrorCode> {
        // TODO
        todo!()
    }

    /// <https://www.w3.org/TR/webdriver-bidi/#end-the-session>
    async fn end_the_session(&self) {
        // 1.
        self.active_sessions().write().await.remove(&self.id);
        // 2. TODO: blocked by webdriver classic active flag not implemented
    }

    /// <https://www.w3.org/TR/webdriver-bidi/#cleanup-the-session>
    pub(crate) async fn cleanup_the_session(&mut self) {
        // 1.
        self.close_the_websocket_connections().await;
        // 2. TODO: blocked by user contet not implemented.
        // 3. TODO: network module not implemented
        // 4. TODO: network module not implemented
        // 5. TODO: screencast not implemented
        // 6.
        if self.active_sessions().read().await.is_empty() {
            self.common.remote_end_state.cleanup();
        }
        // 7. SKIP: implementation specific
    }

    /// <https://www.w3.org/TR/webdriver-bidi/#close-the-websocket-connections>
    async fn close_the_websocket_connections(&mut self) {
        // 1.
        for connection in self.bidi.connections.iter_mut() {
            // 1.1.
            if let Err(e) = connection.close(None).await {
                log::warn!("Closing websocket connection failed: {e:?}");
            }
        }
        // result in handle a connection closing
        self.bidi.connections.clear();
    }

    /// <https://www.w3.org/TR/webdriver-bidi/#buffer-a-log-event>
    fn buffer_a_log_event(&mut self, navigable_ids: &[BrowsingContextId], event: LogEvent) {
        // 1.
        let buffer = &mut self.bidi.log_event_buffer;
        // 2-3. SKIP: we can only use naviable id directly
        // 4.
        for navigable_id in navigable_ids {
            // NOTE: the spec self-contradicts here,
            // we choose to follow the `log_event_buffer` type rather than the steps.
            buffer.entry(*navigable_id).or_default().push(event.clone());
        }
    }

    /// <https://w3c.org/TR/webdriver-bidi/#event-is-enabled>
    fn event_is_enabled(&self, event_name: &str, navigables: &[BrowsingContextId]) -> bool {
        // 1. TODO
        // 2. TODO
        // 2.1. TODO
        // 2.2. TODO
        // 2.3. TODO
        // 2.4. TODO
        // 3.
        false
    }

    /// <https://w3c.org/TR/webdriver-bidi/#emit-an-event>
    async fn emit_an_event(&mut self, body: &Event) {
        // 1. SKIP Assert
        // 2.
        let serialized = serde_json::to_string(&body).expect("Event serialization failed");
        // 3.
        for connection in self.bidi.connections.iter_mut() {
            // 3.1.
            connection.send(serialized.clone().into()).await;
        }
    }

    fn connections_mut(&mut self) -> &mut Vec<Connection> {
        &mut self.bidi.connections
    }

    fn connection_mut(&mut self, conn_idx: usize) -> Option<&mut Connection> {
        self.connections_mut().get_mut(conn_idx)
    }

    pub(crate) fn active_sessions(&self) -> &RwLock<ActiveSessions> {
        &self.common.remote_end_state.active_sessions
    }

    fn send_to_constellation(&self, message: WebDriverToConstellationMessage) {
        if let Err(e) = self.common.constellation_sender.send(message) {
            log::warn!("Sending message to constellation failed: {e:?}");
        }
    }
}

/// <https://www.w3.org/TR/webdriver-bidi/#event-subscription>
pub struct Subscription {
    id: SubscriptionId,
    event_names: HashSet<String>,
    top_level_traversable_ids: HashSet<BrowsingContextId>,
    user_context_ids: HashSet<()>,
}

impl Subscription {
    /// <https://www.w3.org/TR/webdriver-bidi/#subscription-global>
    pub fn is_global(&self) -> bool {
        self.top_level_traversable_ids.is_empty() && self.user_context_ids.is_empty()
    }
}

impl Hash for Subscription {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

impl PartialEq for Subscription {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for Subscription {}

/// <https://www.w3.org/TR/webdriver-bidi/#preload-script-map>
pub struct PreloadScript {
    function_declaration: String,
    arguments: Vec<String>,
    contexts: Option<Vec<String>>,
    sandbox: Option<String>,
    user_contexts: IndexSet<()>,
}

/// <https://www.w3.org/TR/webdriver-bidi/#obtain-a-set-of-event-names>
fn obtain_a_set_of_event_names(name: &str) -> Result<HashSet<String>, ErrorCode> {
    // 1.
    let mut events = HashSet::new();
    // 2.
    if name.contains('\u{002E}') {
        // 2.1.
        if EVENT_NAMES.contains(name) {
            events.insert(name.to_string());
            return Ok(events);
        } else {
            // 2.2.
            return Err(ErrorCode::InvalidArgument);
        }
    }
    // 3.
    let Some(module_events) = MODULE_EVENT_MAP.get(&name) else {
        return Err(ErrorCode::InvalidArgument);
    };
    // 4.
    for event in module_events.iter() {
        events.insert(format!("{name}.{event}"));
    }
    // 5.
    Ok(events)
}

// TODO: how to support custom module?
static MODULE_EVENT_SLICE: &'static [(&'static str, &'static [&'static str])] = &[
    ("session", &[] as &[&str]),
    ("browser", &[]),
    (
        "browsingContext",
        &[
            "contextCreated",
            "contextDestroyed",
            "navigationStarted",
            "fragmentNavigated",
            "historyUpdated",
            "domContentLoaded",
            "load",
            "downloadWillBegin",
            "downloadEnd",
            "navigationAborted",
            "navigationCommitted",
            "navigationFailed",
            "userPromptClosed",
            "userPromptOpened",
        ],
    ),
    ("emulation", &[]),
    (
        "network",
        &[
            "authRequired",
            "beforeRequestSent",
            "fetchError",
            "responseCompleted",
            "responseStarted",
        ],
    ),
    ("script", &["message", "realmCreated", "realmDestroyed"]),
    ("storage", &[]),
    ("log", &["entryAdded"]),
    ("input", &["fileDialogOpened"]),
    ("webExtension", &[]),
];

static EVENT_NAMES: LazyLock<HashSet<String>> = LazyLock::new(|| {
    MODULE_EVENT_SLICE
        .iter()
        .flat_map(|(module, events)| events.iter().map(move |e| format!("{module}.{e}")))
        .collect()
});

static MODULE_EVENT_MAP: LazyLock<HashMap<&'static str, &'static [&'static str]>> =
    LazyLock::new(|| MODULE_EVENT_SLICE.iter().copied().collect());

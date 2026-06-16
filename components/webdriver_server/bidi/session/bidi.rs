use std::{
    collections::{HashMap, HashSet},
    hash::Hash,
    sync::LazyLock,
};

use indexmap::{IndexMap, IndexSet};
use servo_base::{generic_channel::GenericCallback, id::BrowsingContextId};
use tokio::sync::{RwLock, mpsc};
use uuid::Uuid;
use webdriver_traits::{
    WebDriverToConstellationMessage, WebDriverToScriptMessage,
    bidi::{
        BrowsingContextCommand, BrowsingContextResult, CommandData, EmptyParams, EmptyResult,
        ErrorCode, Event, LogEvent, ResultData, SessionCommand, SessionResult,
        browsing_context::{self, NavigateResult},
        script::PreloadScript as PreloadScriptId,
        session::{self, Subscription as SubscriptionId},
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
            CommandData::BrowserCommand(cmd) => todo!(),
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
            CommandData::EmulationCommand(cmd) => todo!(),
            CommandData::InputCommand(cmd) => todo!(),
            CommandData::NetworkCommand(cmd) => todo!(),
            CommandData::ScriptCommand(cmd) => todo!(),
            CommandData::SessionCommand(cmd) => match cmd {
                SessionCommand::End(cmd) => self.handle_session_end(&cmd.params).await,
                SessionCommand::New(cmd) => self.handle_session_new(&cmd.params).await,
                SessionCommand::Status(cmd) => self.handle_session_status(&cmd.params).await,
                SessionCommand::Subscribe(cmd) => self.handle_session_subscribe(&cmd.params).await,
                SessionCommand::Unsubscribe(cmd) => {
                    self.handle_session_unsubscribe(&cmd.params).await
                },
            },
            CommandData::StorageCommand(cmd) => todo!(),
            CommandData::WebExtensionCommand(cmd) => todo!(),
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

    // TODO: link
    async fn handle_browsing_context_handle_user_prompt(
        &self,
        command_parameters: &browsing_context::HandleUserPromptParameters,
    ) -> Result<browsing_context::HandleUserPromptResult, ErrorCode> {
        // TODO:
        todo!()
    }

    // TODO: link
    async fn handle_browsing_context_locate_nodes(
        &self,
        command_parameters: &browsing_context::LocateNodesParameters,
    ) -> Result<browsing_context::LocateNodesResult, ErrorCode> {
        // TODO:
        todo!()
    }

    // TODO: link
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

    // TODO: link
    async fn handle_browsing_context_print(
        &self,
        _: &browsing_context::PrintParameters,
    ) -> Result<browsing_context::PrintResult, ErrorCode> {
        // TODO: blocked by PDF not implemented
        Err(ErrorCode::UnknownError)
    }

    // TODO: link
    async fn handle_browsing_context_reload(
        &self,
        _: &browsing_context::ReloadParameters,
    ) -> Result<browsing_context::ReloadResult, ErrorCode> {
        // TODO
        todo!()
    }

    // TODO: link
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
        command_parameters: &browsing_context::StartScreencastParameters,
    ) -> Result<browsing_context::StartScreencastResult, ErrorCode> {
        // TODO:
        todo!()
    }

    /// <https://www.w3.org/TR/webdriver-bidi/#command-browsingContext-stopScreencast>
    async fn handle_browsing_context_stop_screencast(
        &self,
        command_parameters: &browsing_context::StopScreencastParameters,
    ) -> Result<browsing_context::StopScreencastResult, ErrorCode> {
        // TODO:
        todo!()
    }

    /// <https://www.w3.org/TR/webdriver-bidi/#command-browsingContext-traverseHistory>
    async fn handle_browsing_context_traverse_history(
        &self,
        command_parameters: &browsing_context::TraverseHistoryParameters,
    ) -> Result<browsing_context::TraverseHistoryResult, ErrorCode> {
        // TODO:
        todo!()
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
    async fn await_a_navigation(&self) -> Result<NavigateResult, ErrorCode> {
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

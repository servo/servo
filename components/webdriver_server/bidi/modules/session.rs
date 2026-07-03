use std::{cmp::Ordering, collections::HashSet};

use async_tungstenite::tungstenite::Message as WsMessage;
use indexmap::IndexMap;
use log::warn;
use servo_base::id::BrowsingContextId;
use webdriver_traits::{
    bidi::{
        ErrorCode, ResultData, SessionCommand, SessionResult,
        session::{
            EndResult, NewParameters, NewResult, NewResultCapabilities, StatusResult,
            SubscribeParameters, SubscribeResult, UnsubscribeParameters, UnsubscribeResult,
        },
    },
    ids::{ResumeId, SessionId, SubscriptionId},
};

use crate::bidi::{
    WebDriverBidiThread, WebDriverToServerMessage,
    error::BidiError,
    modules::{CommandHandled, ResponseSent},
    session::{Session, SessionFlags, Subscription},
};

impl WebDriverBidiThread {
    pub(super) fn handle_session(
        &mut self,
        command_id: ResumeId,
        resume_id: ResumeId,
        session: Option<SessionId>,
        command: SessionCommand,
    ) {
        match command {
            SessionCommand::End(_) => {
                self.handle_session_end(command_id, resume_id, session.unwrap())
            },
            SessionCommand::New(cmd) => self.handle_session_new(resume_id, session, cmd.params),
            SessionCommand::Status(_) => self.handle_session_status(resume_id, session),
            SessionCommand::Subscribe(cmd) => {
                self.handle_session_subscribe(resume_id, session.unwrap(), cmd.params)
            },
            SessionCommand::Unsubscribe(cmd) => {
                self.handle_session_unsubscribe(resume_id, session.unwrap(), cmd.params)
            },
        }
    }

    fn handle_session_status(&mut self, resume_id: ResumeId, session: Option<SessionId>) {
        // Step 1.
        let body = match session {
            None => StatusResult {
                ready: true,
                message: String::new(),
            },
            Some(s) => StatusResult {
                ready: false,
                message: format!("Already in session: {s}"),
            },
        };
        // Step 2.
        self.resume::<CommandHandled>(
            resume_id,
            Ok(ResultData::Session(SessionResult::Status(body))),
        );
    }

    fn handle_session_new(
        &mut self,
        resume_id: ResumeId,
        session: Option<SessionId>,
        command_parameters: NewParameters,
    ) {
        // Step 1. check session
        if let Some(session_id) = session {
            warn!(
                "Client attempted to create a session when already in session (id: {:?})",
                session_id
            );
            self.resume::<CommandHandled>(resume_id, Err(ErrorCode::SessionNotCreated.into()));
            return;
        }
        // Step 2. skip implementation-defined
        // Step 3.
        let flags = SessionFlags::BIDI;
        // Step 4-5. process capabilities
        let capabilities = match self.process_capabilities(command_parameters, &flags) {
            Err(err) => {
                self.resume::<CommandHandled>(resume_id, Err(err));
                return;
            },
            Ok(x) => x,
        };
        // Step 6. create a session
        let session = self.create_a_session(&capabilities, flags);
        // Step 7. set bidi flag
        if let Some(session) = self.sessions.get_mut(&session) {
            session.flags.insert(SessionFlags::BIDI);
            session.flags.remove(SessionFlags::HTTP);
            // TODO: CHAN flag for embedder message?
        }
        // Step 8.
        let body = NewResult {
            session_id: session,
            capabilities,
        };
        // Step 9.
        self.resume::<CommandHandled>(resume_id, Ok(ResultData::Session(SessionResult::New(body))));
    }

    /// The remote end steps for `session.end`.
    /// See <https://www.w3.org/TR/webdriver-bidi/#command-session-end>.
    fn handle_session_end(
        &mut self,
        command_id: ResumeId,
        resume_id: ResumeId,
        session: SessionId,
    ) {
        // Step 1. end the session.
        let Some(session) = self.end_the_session(session) else {
            self.resume::<CommandHandled>(resume_id, Err(ErrorCode::InvalidArgument.into()));
            return;
        };
        // Step 2. return success.
        self.resume::<CommandHandled>(
            resume_id,
            Ok(ResultData::Session(
                SessionResult::End(EndResult::default()),
            )),
        );
        // Step 2.1. wait until response sent and cleanup session.
        self.awaits(command_id, ResponseSent::SessionEnd(session));
        // See [`handle_session_end_resume`] for remaining steps.
    }

    /// Resume remote end steps for `session.end`.
    /// This resumes after the response message is sent, starting from Step 2.
    /// See <https://www.w3.org/TR/webdriver-bidi/#command-session-end>.
    pub(crate) fn handle_session_end_resume(&mut self, session: Session) {
        // Step 2.2. cleanup the session
        self.cleanup_the_session(session);
    }

    /// See <https://www.w3.org/TR/webdriver-bidi/#command-session-subscribe>.
    fn handle_session_subscribe(
        &mut self,
        resume_id: ResumeId,
        session: SessionId,
        command_parameters: SubscribeParameters,
    ) {
        // Step 1.
        let mut event_names = HashSet::<String>::new();
        // Step 2. obtain event names
        for name in command_parameters.events {
            event_names = event_names
                .union(&match obtain_a_set_of_event_names(&name) {
                    Ok(set) => set,
                    Err(err) => {
                        self.resume::<CommandHandled>(resume_id, Err(err));
                        return;
                    },
                })
                .cloned()
                .collect();
        }
        // Step 3.
        let input_user_context_id = command_parameters
            .user_contexts
            .into_iter()
            .flat_map(|c| c.into_iter())
            .collect::<HashSet<_>>();
        // Step 4.
        let input_context_ids = command_parameters
            .contexts
            .into_iter()
            .flat_map(|c| c.into_iter())
            .collect::<HashSet<_>>();
        // Step 5. if both not empty, error with "invalid arguments"
        if !input_user_context_id.is_empty() && !input_context_ids.is_empty() {
            self.resume::<CommandHandled>(resume_id, Err(ErrorCode::InvalidArgument.into()));
            return;
        }
        // Step 6.
        let mut subscription_navigables = HashSet::<BrowsingContextId>::new();
        // Step 7.
        let mut top_level_traversable_ids = HashSet::<BrowsingContextId>::new();
        // Step 8. "input context ids" not empty
        if !input_context_ids.is_empty() {
            // Step 8.1.
            let navigables = match self.get_valid_navigables_by_ids(&input_context_ids) {
                Ok(x) => x,
                Err(err) => {
                    self.resume::<CommandHandled>(resume_id, Err(err));
                    return;
                },
            };
            // Step 8.2.
            subscription_navigables = self.get_top_level_traversables(&navigables);
            // Step 8.3.
            for navigable in subscription_navigables.iter() {
                top_level_traversable_ids.insert(*navigable);
            }
        }
        // Step 9.
        else if !input_user_context_id.is_empty() {
            // Step 9.1.
            for user_context_id in input_user_context_id.iter() {
                // Step 9.1.1.
                let user_context = self.get_user_context(&user_context_id);
                // Step 9.1.2. if invalid user context, error with "no such user context"
                if user_context.is_none() {
                    self.resume::<CommandHandled>(
                        resume_id,
                        Err(ErrorCode::NoSuchUserContext.into()),
                    );
                    return;
                }
                // Step 9.1.3.
                // TODO: user context related
            }
        }
        // Step 10.
        else {
            subscription_navigables = self
                .navigables
                .values()
                .filter(|navigable| navigable.top_level == true)
                .map(|navigable| navigable.id)
                .collect();
        }
        // Step 11.
        let subscription = Subscription {
            id: SubscriptionId::new(),
            event_names,
            top_level_traversable_ids,
            user_context_ids: input_user_context_id,
        };
        // Step 12.
        let mut subscribe_step_events = IndexMap::<String, HashSet<BrowsingContextId>>::new();
        // Step 13.
        for event_name in subscription.event_names.iter() {
            // Step 13.1.
            // TODO: impl check
            // Step 13.2.
            let existing_navigables = self
                .set_of_top_level_traversables_for_which_an_event_is_enabled(event_name, session);
            // Step 13.3.
            subscribe_step_events.insert(
                event_name.to_string(),
                subscription_navigables
                    .difference(&existing_navigables)
                    .copied()
                    .collect(),
            );
        }
        // Step {14,15}. append to subscription
        if let Some(session) = self.sessions.get_mut(&session) {
            session
                .subscriptions
                .insert(subscription.id, subscription.clone());
        }
        // Step 16. sort inascending order
        // NOTE: the spec may have problem here, subscribe with smaller priority appears first.
        subscribe_step_events = subscribe_step_events
            .sorted_by(|event_name_one, _, event_name_two, _| {
                // Step 16.{1,2,3}
                if subscribe_priority(event_name_one) < subscribe_priority(event_name_two) {
                    Ordering::Less
                } else {
                    Ordering::Greater
                }
            })
            .collect();
        // Step 17. if global
        let include_global = subscription.is_global();
        // Step 18. run subsctibe steps
        for (event_name, navigables) in subscribe_step_events {
            self.run_remote_end_subscribe_steps(&event_name, session, &navigables, include_global);
        }
        // Step 19.
        let body = SubscribeResult {
            subscription: subscription.id,
        };
        // Step 20. return success
        self.resume::<CommandHandled>(
            resume_id,
            Ok(ResultData::Session(SessionResult::Subscribe(body))),
        );
    }

    /// See <https://www.w3.org/TR/webdriver-bidi/#command-session-unsubscribe>.
    fn handle_session_unsubscribe(
        &mut self,
        resume_id: ResumeId,
        session: SessionId,
        command_parameters: UnsubscribeParameters,
    ) {
        match command_parameters {
            // Step 1. if is byAttribute
            UnsubscribeParameters::ByAttributes(params) => {
                // Step 1.1.
                let mut event_names = HashSet::<String>::new();
                // Step 1.2.
                for name in params.events {
                    event_names = event_names
                        .union(&match obtain_a_set_of_event_names(&name) {
                            Ok(set) => set,
                            Err(err) => {
                                self.resume::<CommandHandled>(resume_id, Err(err));
                                return;
                            },
                        })
                        .cloned()
                        .collect();
                }
                // Step 1.3.
                let mut new_subscriptions = Vec::<Subscription>::new();
                // Step 1.4.
                let mut matched_event = HashSet::<String>::new();
                // Step 1.5.
                for subscription in self
                    .sessions
                    .get(&session)
                    .iter()
                    .flat_map(|s| s.subscriptions.values())
                {
                    // Step 1.5.1. if no intersection
                    if subscription.event_names.intersection(&event_names).count() == 0 {
                        // Step 1.5.1.1.
                        new_subscriptions.push(subscription.clone());
                        // Step 1.5.1.2.
                        continue;
                    }
                    // Step 1.5.2. if global
                    if subscription.is_global() {
                        new_subscriptions.push(subscription.clone());
                    }
                    // Step 1.5.3. clone event names
                    let mut subscription_event_names = subscription.event_names.clone();
                    // Step 1.5.4. for each event name
                    for event_name in event_names.iter() {
                        // Step 1.5.4.1.
                        if subscription_event_names.contains(event_name) {
                            // Step 1.5.4.1.1.
                            matched_event.insert(event_name.clone());
                            // Step 1.5.4.1.2.
                            subscription_event_names.remove(event_name);
                        }
                    }
                    // Step 1.5.5. if not empty
                    if !subscription_event_names.is_empty() {
                        // Step 1.5.5.1. new subscription
                        let cloned_subscription = Subscription {
                            id: subscription.id,
                            event_names: event_names.clone(),
                            top_level_traversable_ids: subscription
                                .top_level_traversable_ids
                                .clone(),
                            user_context_ids: subscription.user_context_ids.clone(),
                        };
                        // Step 1.5.5.2. append
                        new_subscriptions.push(cloned_subscription);
                    }
                }
                // Step 1.6.
                if matched_event != event_names {
                    self.resume::<CommandHandled>(
                        resume_id,
                        Err(ErrorCode::InvalidArgument.into()),
                    );
                }
                // Step 1.7. update session's subscriptions
                if let Some(session) = self.sessions.get_mut(&session) {
                    session.subscriptions =
                        new_subscriptions.into_iter().map(|s| (s.id, s)).collect();
                }
            },
            // Step 2. if otherwise is by id
            UnsubscribeParameters::ById(params) => {
                // Step 2.1.
                let subscriptions =
                    HashSet::<SubscriptionId>::from_iter(params.subscriptions.into_iter());
                // Step 2.2. calculate unknown ids
                let known_subscription_ids = self
                    .sessions
                    .get(&session)
                    .iter()
                    .flat_map(|s| s.subscriptions.keys())
                    .copied()
                    .collect();
                let unknown_subscription_ids = subscriptions.difference(&known_subscription_ids);
                // Step 2.3. if unknown, error with "invalid argument"
                if unknown_subscription_ids.count() == 0 {
                    self.resume::<CommandHandled>(
                        resume_id,
                        Err(ErrorCode::InvalidArgument.into()),
                    );
                }
                // Step 2.4.
                let mut subscriptions_to_remove = HashSet::new();
                // Step 2.5. for each subscription
                for subscription in self
                    .sessions
                    .get_mut(&session)
                    .iter_mut()
                    .flat_map(|s| s.subscriptions.values_mut())
                {
                    // Step 2.5.1. if contained in argument subscriptions
                    if subscriptions.contains(&subscription.id) {
                        // Step 2.5.1.1. append to remove list
                        subscriptions_to_remove.insert(subscription.id);
                    }
                    // Step 2.6. skip as we store known id together with session
                }
                // Step 2.7. remove all subscriptions in remove list
                if let Some(session) = self.sessions.get_mut(&session) {
                    for subscription_id in subscriptions_to_remove {
                        session.subscriptions.remove(&subscription_id);
                    }
                }
            },
        }
        // Step 3. return success
        self.resume::<CommandHandled>(
            resume_id,
            Ok(ResultData::Session(SessionResult::Unsubscribe(
                UnsubscribeResult::default(),
            ))),
        );
    }

    /// Remove active session.
    /// See <https://www.w3.org/TR/webdriver-bidi/#end-the-session>.
    pub(super) fn end_the_session(&mut self, session: SessionId) -> Option<Session> {
        // Step 1. remove session from active sessions
        let session = self.sessions.remove(&session);
        // Step 2. if active sessions is empty, set the webdriver flag to false
        if self.sessions.is_empty() {
            // TODO: webdriver-active flag is not implemented
        }
        session
    }

    /// Cleanup all resources of a session, including connections, blocked requests, etc.
    /// See <https://www.w3.org/TR/webdriver-bidi/#cleanup-the-session>.
    pub(super) fn cleanup_the_session(&mut self, session: Session) {
        // Step 1. close the websocket connection
        self.close_the_websocket_connections(&session);
        // Step 2. remove user contexts
        // TODO: user context is not implemented
        // Step 3. resume blocked requests map
        // TODO: intercept is not implemented
        // Step 4. remove collectors
        // TODO: collector is not implemented
        // Step 5. stop screencast recording
        // TODO: screencast recording is not implemented
        // Step 6. if no active session, cleanup remote end state
        if self.sessions.is_empty() {
            self.cleanup_remote_end_state();
        }
    }

    /// Close a session's all WebSocket connections.
    /// See <https://www.w3.org/TR/webdriver-bidi/#close-the-websocket-connections>.
    fn close_the_websocket_connections(&mut self, session: &Session) {
        // Step 1. for each associated connection
        for connection in session.connections.iter() {
            // Step 1.1. start the ws closing handshake
            _ = self.server_sender.send(WebDriverToServerMessage::Message(
                *connection,
                WsMessage::Close(None),
            ));
        }
    }

    /// Cleanup global states of remote end.
    /// See <https://www.w3.org/TR/webdriver-bidi/#cleanup-remote-end-state>.
    fn cleanup_remote_end_state(&mut self) {
        // Step 1. clear before request sent map
        // Step 2. restore default cache behavior
        // Step 3. restore navigable cache behabior
        // Step 4. implementation-defined
        // TODO: network not implemented
    }

    /// <https://w3c.github.io/webdriver/#dfn-capabilities-processing>
    fn process_capabilities(
        &self,
        _parameters: NewParameters,
        _flags: &SessionFlags,
    ) -> Result<NewResultCapabilities, BidiError> {
        // TODO: this is part of webdriver classic
        // for now we just hard coded a fixed capabilities
        Ok(NewResultCapabilities {
            accept_insecure_certs: true,
            browser_name: "servoshell".into(),
            browser_version: "0.3.0".into(),
            platform_name: "unknown".into(),
            set_window_rect: false,
            user_agent: "unknown".into(),
            proxy: None,
            unhandled_prompt_behavior: None,
            websocket_url: None,
            extensible: Default::default(),
        })
    }

    /// <https://w3c.github.io/webdriver/#dfn-create-a-session>
    fn create_a_session(
        &mut self,
        _capabilities: &NewResultCapabilities,
        flags: SessionFlags,
    ) -> SessionId {
        // Step 1. generate session id
        let session_id = SessionId::new();
        // TODO: complete the steps after we merge webdriver and webdriver bidi

        self.sessions.insert(
            session_id,
            Session {
                id: session_id,
                flags: flags,
                connections: Default::default(),
                preload_script_map: Default::default(),
                sandbox_map: Default::default(),
                subscriptions: Default::default(),
            },
        );
        session_id
    }

    /// <https://www.w3.org/TR/webdriver-bidi/#get-valid-navigables-by-ids>
    fn get_valid_navigables_by_ids(
        &self,
        navigable_ids: &HashSet<BrowsingContextId>,
    ) -> Result<HashSet<BrowsingContextId>, BidiError> {
        // Step 1.
        let mut result = HashSet::new();
        // Step 2.
        for navigable_id in navigable_ids {
            // Step 2.1.
            if !self.navigables.contains_key(navigable_id) {
                return Err(ErrorCode::InvalidArgument.into());
            }
            result.insert(*navigable_id);
        }
        // Step 3.
        Ok(result)
    }

    /// <https://www.w3.org/TR/webdriver-bidi/#get-top-level-traversables>
    fn get_top_level_traversables(
        &self,
        navigables: &HashSet<BrowsingContextId>,
    ) -> HashSet<BrowsingContextId> {
        // Step 1.
        let mut result = HashSet::new();
        // Step 2.
        for navigable in navigables {
            // Step 2.1.
            if let Some(navigable) = self.navigables.get(navigable)
                && navigable.webview_id.is_some()
            {
                if let Some(top_level) = self.navigables.values().find(|candidate| {
                    candidate.top_level
                        && candidate.webview_id == navigable.webview_id
                        && candidate.id != navigable.id
                }) {
                    result.insert(top_level.id);
                }
            }
        }
        // Step 3.
        result
    }

    /// <https://www.w3.org/TR/webdriver-bidi/#get-user-context>
    fn get_user_context(&self, _user_context_id: &str) -> Option<String> {
        None
    }

    fn run_remote_end_subscribe_steps(
        &mut self,
        event_name: &str,
        session: SessionId,
        navigables: &HashSet<BrowsingContextId>,
        include_global: bool,
    ) {
        match event_name {
            "script.realmCreated" => {
                self.subscribe_script_realm_created(session, navigables, include_global);
            },
            _ => {},
        }
    }

    /// <https://www.w3.org/TR/webdriver-bidi/#set-of-top-level-traversables-for-which-an-event-is-enabled>
    fn set_of_top_level_traversables_for_which_an_event_is_enabled(
        &self,
        event_name: &str,
        session: SessionId,
    ) -> HashSet<BrowsingContextId> {
        // Step 1.
        let mut result = HashSet::new();
        // Step 2.
        for subscription in self
            .sessions
            .get(&session)
            .iter()
            .flat_map(|s| s.subscriptions.values())
        {
            // Step 2.1.
            if !subscription.event_names.contains(event_name) {
                continue;
            }
            // Step 2.2.
            if subscription.is_global() {
                // Step 2.2.1.
                for traverable in self.navigables.values().filter(|n| n.top_level) {
                    result.insert(traverable.id);
                }
                // Step 2.2.2.
                break;
            }
            // Step 2.3.
            // TODO: user copntext related
            // Step 2.4.
            else {
                // Step 2.4.1.
                let top_level_traversables = &subscription.top_level_traversable_ids;
                // Step 2.4.2.
                top_level_traversables
                    .iter()
                    .for_each(|id| _ = result.insert(*id));
            }
        }
        // Step 3.
        result
    }
}

fn obtain_a_set_of_event_names(name: &str) -> Result<HashSet<String>, BidiError> {
    // Step 1.
    let mut events = HashSet::new();
    // Step 2.
    if name.contains('.') {
        match name {
            "script.message" | "script.realmCreated" | "script.realmDestroyed" => {
                events.insert(name.to_string());
            },
            _ => {
                return Err(ErrorCode::InvalidArgument.into());
            },
        }
    }
    // Step {3,4}.
    match name {
        "script" => {
            [
                "script.message",
                "script.realmCreated",
                "script.realmDestroyed",
            ]
            .iter()
            .for_each(|event| _ = events.insert(event.to_string()));
        },
        "session" => {},
        _ => return Err(ErrorCode::InvalidArgument.into()),
    }

    // Step 5.
    Ok(events)
}

/// <https://www.w3.org/TR/webdriver-bidi/#event-subscribe-priority>
// TODO: adhoc, we should have typed event name
fn subscribe_priority(name: &str) -> usize {
    match name {
        "script.realmCreated" => 2,
        _ => 0,
    }
}

use crossbeam_channel::{Sender, bounded};
use indexmap::{IndexMap, IndexSet};
use rustenium_bidi_definitions::{
    base::ErrorCode,
    session::{
        self,
        commands::{New, SessionCommand},
        types::UnsubscribeParameters,
    },
};
use uuid::Uuid;

use crate::{
    dispatcher::DispatchMessage, error::WebDriverBidiError, handler::Handler, model::SessionResult,
    session::SessionId,
};

impl Handler {
    pub(super) async fn handle_session(
        &self,
        cmd: &SessionCommand,
    ) -> Result<SessionResult, WebDriverBidiError> {
        match cmd {
            SessionCommand::Status(status) => self.handle_session_status().await,
            SessionCommand::New(session_new) => {
                self.handle_session_new(
                    session_new,
                    // self.0.dispatch_tx
                    todo!(),
                )
                .await
            },
            SessionCommand::End(end) => self.handle_session_end().await,
            SessionCommand::Subscribe(subscribe) => self.handle_session_subscribe().await,
            SessionCommand::Unsubscribe(unsubscribe) => {
                self.handle_session_unsubscribe(&unsubscribe.params.unsubscribe_parameters)
                    .await
            },
        }
    }

    // TODO: can non-static handlers be considered ready
    async fn handle_session_status(&self) -> Result<SessionResult, WebDriverBidiError> {
        // TODO: currently we can directly return ready, but in future we may need to wait for
        // capabilities communication between handler and embedder.

        // Step 1. let body
        let body = session::results::StatusResult {
            ready: true,
            message: "".to_string(),
        };

        // Step 2. return success
        // TODO: move to recv
        // Ok(Some(SessionResult::Session(SessionResult::Status(body))))
        Ok(todo!())
    }

    async fn handle_session_new(
        &self,
        _session_new: &New,
        dispatch_tx: Sender<DispatchMessage>,
    ) -> Result<SessionResult, WebDriverBidiError> {
        // Step 1. if session is not null, return "session not created" error.
        if !self.0.is_static {
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
        // Ok(Some(SessionResult::Session(SessionResult::New(Box::new(
        //     body,
        // )))))
        Ok(todo!())
    }

    async fn handle_session_end(&self) -> Result<SessionResult, WebDriverBidiError> {
        // Step 1. end the session, skip.
        // TODO: do we need to notify embedder to stop subscription or other status,
        // or is this automatically done with channel drop?

        // Step 2. return success
        // TODO: move to recv
        // Ok(Some(SessionResult::Session(SessionResult::End(
        //     session::results::EndResult {
        //         extensible: Default::default(),
        //     },
        // ))))
        Ok(todo!())
    }

    async fn handle_session_subscribe(&self) -> Result<SessionResult, WebDriverBidiError> {
        // Step 1: let `event names` be empty set
        let event_names = IndexSet::<()>::new();

        // Step 2: for each name in param["events"], try obtain a set of event names, union

        // Step 3: let `input user context ids` be a set with params[userContexts]
        // Step 4: let `input context ids` be a set of params[contexts]
        // Step 5: if `input user context ids` is not empty and `input context ids` is not empty, return invalid argument error

        // Step 6: let `subscription navigables` be a set
        let subscription_navigables = IndexSet::<()>::new();

        // Step 7: let `top-level traversable context ids` be a set
        let top_level_traversable_context_ids = IndexSet::<()>::new();

        // Step 8: if `input context ids` is not empty

        // Step 9: otherwise if `input user context ids` is not empty

        // Step 10: otherwise set `subscription navigables` to a set of all top-level traversables
        // TODO: subscription_navigables = self.get_all_top_level_traversables().await;

        // Step 11: let `subscription` with `subscription id` UUID, ...
        let subscription_id = Uuid::new_v4();

        // Step 12: let `subscribe step events` be a new map
        let subscribe_step_events = IndexMap::<(), ()>::new();

        // Step 13: for each `event name` in the `event names`
        for event_name in event_names {}

        // Step 14: append `subscription` to `session`'s subscriptions
        // TODO: session needs a `subscriptions` field

        // Step 15: append `subscription`'s `subscription id` to `session`'s `known subscription ids`

        // Step 16: Sort in ascending order

        // Step 17: if subscription is global

        // 18. For each `event name` -> `navigables` in `subscribe step events`:

        // 19. Let body be session.SubscribeResult
        let body = todo!();

        // Step 20: return success
        todo!()
    }

    async fn handle_session_unsubscribe(
        &self,
        cmd_params: &UnsubscribeParameters,
    ) -> Result<SessionResult, WebDriverBidiError> {
        use session::types::UnsubscribeParameters;

        match cmd_params {
            // 1. If `command parameters` does not contain `subscriptions`:
            UnsubscribeParameters::UnsubscribeByIdRequest(unsubscribe_by_id_request) => {
                // 1.1. Let `event names` be an empty set.
                let event_names = IndexSet::<()>::new();

                // 1.2. For each entry `name` in `command parameters`["events"], let
                // `event names` be the union of `event names` and the result of trying
                // to `obtain a set of event names` with `name`

                // 1.3. Let `new subscriptions` to be a list.
                let new_subscriptions = Vec::<()>::new();

                // 1.4. Let `matched events` to be a set.
                let matched_events = IndexSet::<()>::new();

                // 1.5. For each `subscription` of `session`'s `subscriptions`:
                {
                    // 1.5.1. If `intersection` of `subscription`'s `event names` and
                    // `event names` is an empty set:
                    // 1.5.2.
                    // 1.5.3.
                    // 1.5.4.
                    // 1.5.5.
                    // 1.5.6.
                    // 1.5.7. Set `session`'s `subscriptions` to `new subscriptions`.
                }
                todo!()
            },
            // 2. Otherwise:
            UnsubscribeParameters::UnsubscribeByAttributesRequest(
                unsubscribe_by_attributes_request,
            ) => todo!(),
        };

        // 3. return success with data null.
        Ok(SessionResult::Unsubscribe(
            session::results::UnsubscribeResult {
                extensible: Default::default(),
            },
        ))
    }
}

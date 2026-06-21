use std::{collections::HashSet, rc::Rc};

use log::warn;
use tokio::{sync::oneshot::Receiver, task};
use webdriver_traits::bidi::{
    EmptyParams, EmptyResult, ErrorCode, SessionCommand, SessionResult,
    session::{
        NewParameters, NewResult, StatusResult, SubscribeParameters, SubscribeResult,
        UnsubscribeParameters, UnsubscribeResult,
    },
};

use crate::bidi::{
    error::{BidiError, BidiResult},
    remote_end::RemoteEnd,
    session::SessionId,
};

impl RemoteEnd {
    pub(crate) async fn handle_session_command(
        self: Rc<Self>,
        session_id: Option<SessionId>,
        command: SessionCommand,
        msg_sent: Receiver<()>,
    ) -> BidiResult<SessionResult> {
        match command {
            SessionCommand::Status(cmd) => self
                .handle_session_status(session_id, cmd.params)
                .await
                .map(SessionResult::StatusResult),
            SessionCommand::New(cmd) => self
                .handle_session_new(session_id, cmd.params)
                .await
                .map(|r| SessionResult::NewResult(Box::new(r))),
            SessionCommand::End(cmd) => self
                .handle_session_end(session_id.unwrap(), cmd.params, msg_sent)
                .await
                .map(SessionResult::EndResult),
            SessionCommand::Subscribe(cmd) => self
                .handle_session_subscribe(session_id.unwrap(), cmd.params)
                .await
                .map(SessionResult::SubscribeResult),
            SessionCommand::Unsubscribe(cmd) => self
                .handle_session_unsubscribe(session_id.unwrap(), cmd.params)
                .await
                .map(SessionResult::UnsubscribeResult),
        }
    }

    /// <https://www.w3.org/TR/webdriver-bidi/#command-session-status>
    async fn handle_session_status(
        self: Rc<Self>,
        session_id: Option<SessionId>,
        _command_parameters: EmptyParams,
    ) -> BidiResult<StatusResult> {
        // Step 1.
        let body = match session_id {
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
        Ok(body)
    }

    /// <https://www.w3.org/TR/webdriver-bidi/#command-session-new>
    async fn handle_session_new(
        self: Rc<Self>,
        session_id: Option<SessionId>,
        command_parameters: NewParameters,
    ) -> BidiResult<NewResult> {
        // Step 1. check session
        if let Some(session_id) = session_id {
            warn!(
                "Client attempted to create a session when already in session (id: {:?})",
                session_id
            );
            return Err(BidiError {
                code: ErrorCode::SessionNotCreated,
                message: format!("Already in session {}", session_id),
                ..Default::default()
            });
        }
        // Step 2. skip implementation-defined
        // Step 3.
        let flags = HashSet::<&'static str>::from_iter(["bidi"]);
        // Step 4-5. process capabilities
        let capabilities = self.process_capabilities(command_parameters, &flags)?;
        // Step 6. create a session
        let mut session = self.create_a_session(&capabilities, &flags)?;
        // Step 7. set bidi flag
        session.bidi = true;
        session.http = false;
        // Step 8.
        let body = NewResult {
            session_id: session.id.to_string(),
            capabilities,
        };
        // Step 9.
        Ok(body)
    }

    /// <https://www.w3.org/TR/webdriver-bidi/#command-session-end>
    async fn handle_session_end(
        self: Rc<Self>,
        session_id: SessionId,
        _: EmptyParams,
        msg_sent: Receiver<()>,
    ) -> BidiResult<EmptyResult> {
        // Step 1. end current session
        self.end_the_session(session_id);
        // Step 2. return success before do cleanup
        task::spawn_local(async move {
            if let Err(err) = msg_sent.await {
                warn!("Waiting sending a WebSocket message failed ({err:?})");
            }
            self.cleanup_the_session(session_id);
        });
        Ok(EmptyResult {
            extensible: Default::default(),
        })
    }

    /// <https://www.w3.org/TR/webdriver-bidi/#command-session-subscribe>
    async fn handle_session_subscribe(
        self: Rc<Self>,
        session_id: SessionId,
        command_parameters: SubscribeParameters,
    ) -> BidiResult<SubscribeResult> {
        Err(ErrorCode::UnknownError.into())
    }

    /// <https://www.w3.org/TR/webdriver-bidi/#command-session-unsubscribe>
    async fn handle_session_unsubscribe(
        self: Rc<Self>,
        session_id: SessionId,
        command_parameters: UnsubscribeParameters,
    ) -> BidiResult<UnsubscribeResult> {
        Err(ErrorCode::UnknownError.into())
    }
}

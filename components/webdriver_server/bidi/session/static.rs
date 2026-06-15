use std::sync::Arc;

use tokio::sync::RwLock;
use uuid::Uuid;
use webdriver_traits::bidi::{
    self, CommandData, EmptyParams, ErrorCode, ResultData, SessionCommand, SessionResult,
    session::{NewParameters, NewResult, NewResultCapabilities, StatusResult},
};

use crate::bidi::{
    ActiveSessions,
    connection::Connection,
    session::{
        Session,
        bidi::BidiPart,
        common::{CommonPart, SessionId},
    },
};

pub(crate) struct StaticSession<'a> {
    pub(crate) common: &'a mut CommonPart,
    pub(crate) connections: &'a mut Vec<Connection>,
}

impl<'a> StaticSession<'a> {
    pub(crate) async fn handle_static_command(
        &mut self,
        command: StaticCommand<'a>,
    ) -> Result<ResultData, ErrorCode> {
        match command {
            StaticCommand::SessionStatus(cmd) => self.handle_session_status(&cmd.params).await,
            StaticCommand::SessionNew(cmd) => self.handle_session_new(&cmd.params).await,
        }
    }

    /// <https://www.w3.org/TR/webdriver-bidi/#command-session-end>
    async fn handle_session_status(&mut self, _: &EmptyParams) -> Result<ResultData, ErrorCode> {
        // 1.
        let body = StatusResult {
            ready: true,
            // implementation-defined
            message: "".to_string(),
        };
        // 2.
        Ok(ResultData::SessionResult(SessionResult::StatusResult(body)))
    }

    /// <https://www.w3.org/TR/webdriver-bidi/#command-session-end>
    async fn handle_session_new(
        &mut self,
        command_parameters: &NewParameters,
    ) -> Result<ResultData, ErrorCode> {
        // 1. SKIP: session is null in static session
        // 2. SKIP: implementation-defined
        // 3. SKIP:
        let flags = vec!["bidi"];
        // 4.
        let capabilities_json = self.process_capabilities(command_parameters, &flags)?;
        // 5.
        let capabilities = capabilities_json;
        // 6.
        let session = self.create_a_session(&capabilities, &flags).await?;
        // 7. SKIP: we use enum instead of flag
        // 8.
        let body = NewResult {
            session_id: session.to_string(),
            capabilities,
        };
        // 9.
        Ok(ResultData::SessionResult(SessionResult::NewResult(
            Box::new(body),
        )))
    }

    /// <https://w3c.github.io/webdriver/#dfn-capabilities-processing>
    fn process_capabilities(
        &self,
        parameters: &NewParameters,
        _flags: &[&str],
    ) -> Result<NewResultCapabilities, ErrorCode> {
        // 1.
        let _capabilities_request = &parameters.capabilities;
        // 1.1. SKIP: we already deserialize in handling incoming
        // 2-8. TODO: we do not validate capabilities for now
        // 9.
        // TODO: should get some field from pref
        Ok(NewResultCapabilities {
            accept_insecure_certs: true,
            browser_name: "servo".to_string(),
            browser_version: "0.2".to_string(),
            platform_name: "linux".to_string(),
            set_window_rect: false,
            user_agent: "servo".to_string(),
            proxy: None,
            unhandled_prompt_behavior: None,
            web_socket_url: None,
            extensible: Default::default(),
        })
    }

    /// <https://w3c.github.io/webdriver/#dfn-create-a-session>
    async fn create_a_session(
        &self,
        capabilities: &NewResultCapabilities,
        _flags: &[&str],
    ) -> Result<SessionId, ErrorCode> {
        // 1.
        let session_id = SessionId(Uuid::new_v4());
        // 2. no http flag
        let session = self.new_bidi_session(session_id);
        // 3. TODO: proxy is now global and once inited in create_http_client, blocked
        // TODO: also need to open an issue in webdriver-bidi to ask if proxy is really per session rather than per user context.
        // 4. TODO: tls not implemented
        // 5.
        let user_prompt_handler = &capabilities.unhandled_prompt_behavior;
        // 6.
        if user_prompt_handler.is_none() {
            // TODO: user_prompt_handler is in classic
        }
        // 7. & 8. SKIP: serialize step contradicts
        // 9. SKIP: flags no http
        // 10. SKIP: implementation-defined
        // 11. SKIP: external spec
        // 12.
        self.active_sessions()
            .write()
            .await
            .insert(session_id, session.to_proxy());
        // 13. SKIP: flags no http
        // 14. TODO: webdriver-active flag in classic
        todo!()
    }

    fn new_bidi_session(&self, session_id: SessionId) -> Session {
        let common = self.common.clone();
        let bidi = BidiPart::default();
        Session::BidiOnly {
            id: session_id,
            common,
            bidi,
        }
    }

    pub(crate) fn active_sessions(&self) -> &Arc<RwLock<ActiveSessions>> {
        &self.common.remote_end_state.active_sessions
    }
}

/// Narrow down the `CommandData` enum.
pub enum StaticCommand<'a> {
    SessionStatus(&'a bidi::session::Status),
    SessionNew(&'a Box<bidi::session::New>),
}

impl<'a> TryFrom<&'a CommandData> for StaticCommand<'a> {
    type Error = ();
    fn try_from(value: &'a CommandData) -> Result<Self, Self::Error> {
        match value {
            CommandData::SessionCommand(cmd) => match cmd {
                SessionCommand::New(cmd) => Ok(StaticCommand::SessionNew(cmd)),
                SessionCommand::Status(cmd) => Ok(StaticCommand::SessionStatus(cmd)),
                _ => Err(()),
            },
            _ => Err(()),
        }
    }
}

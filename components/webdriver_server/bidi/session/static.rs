use webdriver_traits::bidi::{
    self, CommandData, EmptyParams, ErrorCode, ResultData, SessionCommand, SessionResult,
    session::{NewParameters, StatusResult},
};

use crate::bidi::{
    connection::Connection,
    session::common::{CommonPart, SessionId},
};

pub(crate) struct StaticSession<'a> {
    pub(crate) common: &'a mut CommonPart,
    pub(crate) connections: &'a mut Vec<Connection>,
}

impl<'a> StaticSession<'a> {
    pub(crate) fn id(&self) -> Option<&SessionId> {
        None
    }

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
    async fn handle_session_new(&mut self, _: &NewParameters) -> Result<ResultData, ErrorCode> {
        todo!()
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

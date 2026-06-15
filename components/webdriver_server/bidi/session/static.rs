use webdriver_traits::bidi::{self, CommandData, ErrorCode, ResultData, SessionCommand};

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

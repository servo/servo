use async_tungstenite::tungstenite::Message as WsMessage;
use log::warn;
use webdriver_traits::{
    bidi::{CommandData, CommandResponse, ErrorCode, Message, ResultData, SessionResult},
    ids::{ConnectionId, ResumeId, SessionId},
};

use crate::bidi::{WebDriverBidiThread, error::BidiError, session::Session, wait::Resumable};

use super::messages::WebDriverToServerMessage;

pub(super) mod script;
pub(super) mod session;

impl WebDriverBidiThread {
    pub(crate) fn handle_command(
        &mut self,
        command_id: ResumeId,
        response_id: ResumeId,
        session: Option<SessionId>,
        command: CommandData,
    ) {
        match command {
            CommandData::Session(command) => {
                self.handle_session(command_id, response_id, session, command)
            },
            CommandData::Script(command) => {
                self.handle_script(
                    command_id,
                    // PRE: checked in non static command
                    session.unwrap(),
                    command,
                )
            },
            CommandData::Unknown(_) => self.handle_unknown(command_id),
        }
    }

    /// Currently we have not implemented all commands, so for
    /// unrecognzied command "unknown error" error code is returned.
    fn handle_unknown(&mut self, command_id: ResumeId) {
        self.resume::<CommandHandled>(command_id, Err(ErrorCode::UnsupportedOperation.into()));
    }
}

/// Two possible algorithms can resume after the response is sent:
/// `session.end` or `browser.close`.
pub(crate) enum ResponseSent {
    SessionEnd(Session),
    // BrowserClose,
}

impl Resumable for ResponseSent {
    type Event = ();

    fn resume(self, this: &mut WebDriverBidiThread, _event: Self::Event) {
        match self {
            ResponseSent::SessionEnd(session) => this.handle_session_end_resume(session),
        }
    }
}

pub struct CommandHandled(pub(crate) ConnectionId, pub(crate) u64, pub(crate) ResumeId);

impl Resumable for CommandHandled {
    type Event = Result<ResultData, BidiError>;

    fn resume(self, this: &mut WebDriverBidiThread, result: Self::Event) {
        let connection_id = self.0;

        let value = match result {
            // Step 6.7.2. send error response
            Err(error) => {
                this.send_an_error_response(connection_id, Some(self.1), error);
                return;
            },
            // Step 6.7.3.
            Ok(value) => value,
        };

        // Step 6.7.4. skip assert
        // Step 6.7.5. if session.new, associate conn to session
        if let ResultData::Session(SessionResult::New(result)) = &value
            && let session_id = &result.session_id
            && let Some(session) = this.sessions.get_mut(session_id)
        {
            session.connections.insert(connection_id);
            this.connections.remove(&connection_id);
        }

        // Step 6.7.6.
        let response = Message::CommandResponse(CommandResponse {
            id: self.1,
            result: value,
            extensible: Default::default(),
        });

        // Step 6.7.7. serialize
        let serialized =
            serde_json::to_string(&response).expect("CommandResponse serialization is infallible");

        // Step 6.7.8. send response message
        if let Err(err) = this.server_sender.send(WebDriverToServerMessage::Message(
            connection_id,
            WsMessage::Text(serialized.into()),
        )) {
            warn!(
                "Sending message to connection failde (id: {:?}, err: {:?})",
                connection_id, err
            );
        }

        // In addition, notify msg sent to resume `session.end` and `browser.close`
        this.resume::<ResponseSent>(self.2, ());
    }
}

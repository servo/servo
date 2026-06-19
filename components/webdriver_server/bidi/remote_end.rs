use std::{
    cell::RefCell,
    collections::{HashMap, HashSet},
    rc::Rc,
};

use async_tungstenite::tungstenite::Message as WsMessage;
use futures_util::StreamExt;
use log::warn;
use serde::Deserialize;
use serde_json::Value;
use tokio::task;
use webdriver_traits::bidi::{
    Command, CommandData, CommandResponse, ErrorCode, ErrorResponse, Message, ResultData,
    SessionCommand, SessionResult, session::NewResult,
};

use crate::bidi::{
    connection::{Connection, ConnectionId},
    error::BidiError,
    session::{Session, common::SessionId},
};

pub(crate) struct RemoteEnd {
    /// All the
    pub(crate) connections: RefCell<HashMap<ConnectionId, Connection>>,

    /// An associated list of all sessions that are currently started.
    pub(crate) active_sessions: RefCell<HashMap<SessionId, Session>>,

    /// A set of WebSocket connections not associated with a session.
    pub(crate) unassociated_connections: RefCell<HashSet<ConnectionId>>,
}

impl RemoteEnd {
    /// The main loop of a remote end.
    pub(crate) async fn run(self: Rc<Self>) {
        loop {
            // 1. when receive message
            let (message, idx, _) = futures_util::future::select_all(
                self.connections
                    .borrow_mut()
                    .values_mut()
                    .map(|c| c.0.next()),
            )
            .await;
            // TODO: bad
            let connection_id = ConnectionId(idx as u64);
            task::spawn_local({
                let this = self.clone();
                async move { this.handle_an_incoming_message(connection_id, message.unwrap().unwrap()) }
            });
        }
    }

    /// Handle an incoming message from specific bidi connection.
    ///
    /// <https://www.w3.org/TR/webdriver-bidi/#handle-an-incoming-message>
    async fn handle_an_incoming_message(
        self: Rc<Self>,
        connection_id: ConnectionId,
        data: WsMessage,
    ) {
        // Step 1. if not text message, error "invalid argument"
        let WsMessage::Text(data) = data else {
            self.send_an_error_response(connection_id, None, ErrorCode::InvalidArgument)
                .await;
            return;
        };
        // Step 2. skip assert.
        // Step 3. construct session id,
        let session_id = {
            if let Some(session_id) = self
                .active_sessions
                .borrow()
                .values()
                .find(|s| s.is_associated_with_connection(&connection_id))
                .map(|s| s.id)
            {
                Some(session_id)
            } else if self
                .unassociated_connections
                .borrow()
                .contains(&connection_id)
            {
                None
            } else {
                return;
            }
        };
        // Step 4. parse json
        let parsed = match serde_json::from_str::<Value>(&data) {
            Ok(parsed) => parsed,
            Err(err) => {
                warn!("Parsing JSON value from message failed: {err:?}");
                self.send_an_error_response(connection_id, None, ErrorCode::InvalidArgument)
                    .await;
                return;
            },
        };
        // Step 5. check if session is in active session
        if let Some(session_id) = session_id
            && !self.active_sessions.borrow().contains_key(&session_id)
        {
            return;
        }
        // Step 6. match remote end definition
        match Command::deserialize(&parsed) {
            // Step 6.1.
            Ok(matched) => {
                // Step 6.2. skip assert
                // Step 6.3.
                let command_id = matched.id;
                // Step 6.4.
                let method_is_session_new = matches!(
                    matched.command_data,
                    CommandData::SessionCommand(SessionCommand::New(_))
                );
                // Step 6.5.
                let command = matched.command_data;
                // Step 6.6. check static
                if session_id.is_none() && !command.is_static() {
                    self.send_an_error_response(
                        connection_id,
                        Some(command_id),
                        ErrorCode::InvalidSessionId,
                    )
                    .await;
                    return;
                }
                // Step 6.7.1.
                let result = self.clone().handle_command(session_id, command).await;
                let value = match result {
                    // Step 6.7.2. send error response
                    Err(error) => {
                        self.send_an_error_response(connection_id, Some(command_id), error)
                            .await;
                        return;
                    },
                    // Step 6.7.3.
                    Ok(value) => value,
                };
                // Step 6.7.4. skip assert
                // Step 6.7.5. if session.new, associate conn to session
                if method_is_session_new
                    && let ResultData::SessionResult(SessionResult::NewResult(result)) = &value
                    && let session_id = &result.session_id
                    && let Some(session) = self
                        .active_sessions
                        .borrow_mut()
                        // TODO: codegen should gen session id
                        .get_mut(&SessionId(session_id.parse().unwrap()))
                {
                    session.connections.insert(connection_id);
                    self.unassociated_connections
                        .borrow_mut()
                        .remove(&connection_id);
                    session.connections.insert(connection_id);
                }
                // Step 6.7.6.
                let response = Message::CommandResponse(Box::new(CommandResponse {
                    id: command_id,
                    result: value,
                    extensible: Default::default(),
                }));
                // Step 6.7.7. serialize
                let serialized = serde_json::to_string(&response)
                    .expect("CommandResponse serialization is infallible");
                // Step 6.7.8. send response message
                match self.connections.borrow_mut().get_mut(&connection_id) {
                    Some(connection) => {
                        if let Err(err) = connection.send(WsMessage::Text(serialized.into())).await
                        {
                            warn!(
                                "Sending message to connection failde (id: {:?}, err: {:?})",
                                connection_id, err
                            );
                        }
                    },
                    None => {
                        warn!(
                            "Sending command response to an invalid connection (id: {:?})",
                            connection_id
                        );
                    },
                }
            },
            // Step 7.
            Err(err) => {
                warn!("JSON does not match known definition: {err:?}");
                // Step 7.1.
                let mut command_id = None;
                // Step 7.2. get command "id"
                if let Value::Object(map) = &parsed
                    && let Some(value) = map.get("id")
                    && let Some(uint) = value.as_u64()
                {
                    command_id = Some(uint);
                }
                // Step 7.3.
                let mut error_code = ErrorCode::InvalidArgument;
                // Step 7.4. set error code given "method"
                if let Value::Object(map) = &parsed
                    && let Some(value) = map.get("method")
                    && let Some(str) = value.as_str()
                    && self.set_of_all_command_names().contains(str)
                {
                    error_code = ErrorCode::UnknownCommand;
                }
                // Step 7.5. send error response
                self.send_an_error_response(connection_id, command_id, error_code)
                    .await;
            },
        }
    }

    /// Send an error reponse to the specified bidi connection.
    ///
    /// <https://www.w3.org/TR/webdriver-bidi/#send-an-error-response>
    async fn send_an_error_response(
        self: Rc<Self>,
        connection_id: ConnectionId,
        command_id: Option<u64>,
        error: impl Into<BidiError>,
    ) {
        // Step 1. Construct error data
        let error_data = {
            let error = error.into();
            ErrorResponse {
                id: command_id,
                error: error.code,
                message: error.message,
                stacktrace: error.stacktrace,
                extensible: Default::default(),
            }
        };
        // Step 2. Serialize to text
        let response =
            serde_json::to_string(&error_data).expect("ErrorResponse serialization is infallible");
        // Step 3. Send websocket message
        match self.connections.borrow_mut().get_mut(&connection_id) {
            Some(connection) => {
                if let Err(err) = connection.send(WsMessage::Text(response.into())).await {
                    warn!("Sending error response to ws connection failed: {err:?}");
                }
            },
            None => {
                warn!(
                    "Sending error response to an invalid connection (id: {:?})",
                    connection_id
                );
            },
        }
    }

    /// Cleanup steps when a bidi connection is closing or closed.
    ///
    /// <https://www.w3.org/TR/webdriver-bidi/#handle-a-connection-closing>
    fn handle_a_connection_closing(self: Rc<Self>, connection_id: ConnectionId) {
        // Step 1. remove conn from associated session
        for Session { connections, .. } in self.active_sessions.borrow_mut().values_mut() {
            if connections.remove(&connection_id) {
                break;
            };
        }
        // Step 2. remove conn from unassociated set
        self.unassociated_connections
            .borrow_mut()
            .remove(&connection_id);
    }
}

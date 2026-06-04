use async_tungstenite::tungstenite;
use embedder_traits::webdriver_bidi::RequestId;
use log::error;
use rustenium_bidi_definitions::base::CommandMessage;
use uuid::Uuid;

use crate::{
    connection::{Connection, ConnectionId, ConnectionMap},
    handler::WebDriverBidiHandler,
    model::{Message as BidiMessage, ResultData, SessionResult},
    session::SessionId,
};

/// Messages sent from WebSocket connections to [`Dispatcher`].
#[derive(Debug)]
pub enum DispatchMessage {
    /// Deserialized BiDi command message, along with connection id.
    Command(ConnectionId, Box<CommandMessage>),
    // TODO: new connection may connect to existing session
    NewConnection(ConnectionId, Connection),
}

// TODO: add request ids, so that dispatcher can hide connection and session detail from handler and
// behind. pending_requests: enum Session/Connection.

#[derive(Debug)]
pub struct Dispatcher<T: WebDriverBidiHandler> {
    static_handler: T,
    conn_map: ConnectionMap,
}

impl<T: WebDriverBidiHandler> Dispatcher<T> {
    pub fn new(static_handler: T) -> Self {
        Self {
            static_handler,
            conn_map: ConnectionMap::default(),
        }
    }

    pub fn run(&mut self, rx: crossbeam_channel::Receiver<DispatchMessage>) {
        loop {
            while let Ok(dispatch_msg) = rx.try_recv() {
                self.handle_dispatch(dispatch_msg);
            }
            while let Ok((session, bidi_msg)) = self.static_handler.try_recv() {
                self.handle_bidi(session, bidi_msg);
            }
        }
    }

    fn handle_dispatch(&mut self, dispatch: DispatchMessage) {
        match dispatch {
            DispatchMessage::Command(conn_id, command_message) => {
                let session = self.conn_map.session(conn_id).cloned();
                // handle early error
                // TODO: map session_id to handler
                // TODO: handle option resultdata
                if let Err(err) = self.static_handler.handle(&command_message) {
                    let msg: BidiMessage =
                        BidiMessage::ErrorResponse(err.into_response(Some(command_message.id)));
                    // if associated with session, send to all session. otherwise send to
                    // connection.
                    if let Some(session) = session {
                        self.send_to_session(&session, msg);
                    } else {
                        if let Ok(serialized) = serde_json::to_string(&msg) {
                            let msg = tungstenite::Message::Text(serialized.into());
                            if let Some(conn) = self.conn_map.unassociated.get(&conn_id)
                                && let Err(err) = conn.tx.send(msg)
                            {
                                error!("Error sending error message to bidi server: {err}");
                            }
                        }
                    }
                };
            },
            DispatchMessage::NewConnection(uuid, connection) => {
                self.conn_map.add_connection(None, uuid, connection);
            },
        }
    }

    // TODO: since now different handler use different channel, there is no need to know session.
    fn handle_bidi(&mut self, session: Option<RequestId>, bidi: BidiMessage) {
        // TODO: get ?session info from request id registry.
        let session = todo!();
        self.update(&bidi);
        match session {
            Some(session) => {
                self.send_to_session(&session, bidi);
            },
            None => {
                // TODO: find a way to get the correct unassociated connection. See pending_requests
            },
        }
    }

    // TODO: move to session method
    fn send_to_session(&self, session: &SessionId, message: BidiMessage) {
        let text = match serde_json::to_string(&message) {
            Ok(text) => text,
            Err(err) => {
                // TODO: should we send error message to client?
                error!("Error serializing bidi response message: {err}");
                return;
            },
        };
        let message = tungstenite::Message::Text(text.into());
        for conn in self.conn_map.connections(session) {
            if let Err(err) = conn.tx.send(message.clone()) {
                error!("Error sending error message to bidi server: {err}");
            }
        }
    }

    // TODO: update should only be run on command response.
    // and thus the logic can be shared both in handle sync result and async (recv) result.
    fn update(&mut self, msg: &BidiMessage) {
        if let BidiMessage::CommandResponse(command_response) = msg
            && let ResultData::Session(session_result) = &command_response.result
        {
            // TODO: where does session happen? in dispatcher or handler?
            match session_result {
                SessionResult::New(new_result) => {
                    let _session_id = Uuid::parse_str(&new_result.session_id)
                        .expect("unexpected non uuid session id");
                    // TODO: should associate to session, but now we cannot get uuid in update fn.
                    // wait for paending_requests

                    // self.conn_map.associate(uuid,Session {
                    //     id: new_result.session_id,
                    // });
                },
                SessionResult::End(end_result) => {
                    todo!()
                },
                _ => {},
            }
        }
    }
}

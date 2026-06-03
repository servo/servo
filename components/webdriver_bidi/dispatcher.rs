use log::error;
use rustenium_bidi_definitions::base::CommandMessage;
use uuid::Uuid;

use crate::{
    handler::WebDriverBidiHandler,
    model::{Message, ResultData, SessionResult},
    transport::{Connection, ConnectionMap, Session},
};

#[derive(Debug)]
pub enum DispatchMessage {
    Command(Uuid, Box<CommandMessage>),
    // TODO: new connection may connect to existing session
    NewConnection(Uuid, Connection),
}

// TODO: add request ids, so that dispatcher can hide connection and session detail from handler and
// behind. pending_requests: enum Session/Connection.

#[derive(Debug)]
pub struct Dispatcher<T: WebDriverBidiHandler> {
    handler: T,
    conn_map: ConnectionMap,
}

impl<T: WebDriverBidiHandler> Dispatcher<T> {
    pub fn new(handler: T) -> Self {
        Self {
            handler,
            conn_map: ConnectionMap::default(),
        }
    }

    pub fn run(&mut self, rx: crossbeam_channel::Receiver<DispatchMessage>) {
        loop {
            while let Ok(dispatch_msg) = rx.try_recv() {
                self.handle_dispatch(dispatch_msg);
            }
            while let Ok((session, bidi_msg)) = self.handler.try_recv() {
                self.handle_bidi(session, bidi_msg);
            }
        }
    }

    fn handle_dispatch(&mut self, dispatch: DispatchMessage) {
        match dispatch {
            DispatchMessage::Command(uuid, command_message) => {
                let session = self.conn_map.session(&uuid).cloned();
                // handle early error
                if let Err(err) = self.handler.handle(&session, &command_message) {
                    let msg: Message =
                        Message::ErrorResponse(err.into_response(Some(command_message.id)));
                    // if associated with session, send to all session. otherwise send to
                    // connection.
                    if let Some(session) = session {
                        self.send_to_session(&session, msg);
                    } else {
                        if let Some(conn) = self.conn_map.unassociated.get(&uuid)
                            && let Err(err) = conn.tx.send(msg)
                        {
                            error!("Error sending error message to bidi server: {err}");
                        }
                    }
                };
            },
            DispatchMessage::NewConnection(uuid, connection) => {
                self.conn_map.add_connection(None, uuid, connection);
            },
        }
    }

    fn handle_bidi(&mut self, session: Option<Session>, bidi: Message) {
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

    // TODO: use Arc to avoid clone
    fn send_to_session(&self, session: &Session, message: Message) {
        for conn in self.conn_map.connections(session) {
            if let Err(err) = conn.tx.send(message.clone()) {
                error!("Error sending error message to bidi server: {err}");
            }
        }
    }

    fn update(&mut self, msg: &Message) {
        if let Message::CommandResponse(command_response) = msg
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

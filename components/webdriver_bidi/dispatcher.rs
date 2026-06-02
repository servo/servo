use rustenium_bidi_definitions::base::CommandMessage;
use tokio::sync::mpsc;
use uuid::Uuid;

use crate::{
    handler::WebDriverBidiHandler,
    model::{Message, ResultData, SessionResult},
    transport::{Connection, ConnectionMap, Session},
};

#[derive(Debug)]
pub enum DispatchMessage {
    Command(Uuid, Box<CommandMessage>),
    // TODO: option session
    NewConnection(Uuid, Connection),
}

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

    pub async fn run(&mut self, mut rx: mpsc::UnboundedReceiver<DispatchMessage>) {
        loop {
            tokio::select! {
                dispatch = rx.recv() => {
                    if let Some(dispatch) = dispatch {
                        self.handle_dispatch(dispatch);
                    } else {
                        // quit dispatcher when server close
                        return;
                    }
                }
                (session, bidi) = self.handler.recv() => self.handle_bidi(session, bidi)
            };
        }
    }

    fn handle_dispatch(&mut self, dispatch: DispatchMessage) {
        match dispatch {
            DispatchMessage::Command(_, command_message) => {
                // TODO: find using uuid
                self.handler.process(&None, &command_message);
            },
            DispatchMessage::NewConnection(uuid, connection) => {
                // TODO: add to conn_map
            },
        }
    }

    fn handle_bidi(&mut self, session: Option<Session>, bidi: Message) {
        self.update(&bidi);
        match session {
            Some(session) => {
                for conn in self.conn_map.connections(&session) {
                    conn.tx.send(bidi.clone());
                }
            },
            None => todo!(),
        }
    }

    fn update(&mut self, msg: &Message) {
        if let Message::CommandResponse(command_response) = msg
            && let ResultData::Session(session_result) = &command_response.result
        {
            match session_result {
                SessionResult::New(new_result) => {
                    // Step 7. if method is session.new, bind connection to session
                    // TODO:
                },
                SessionResult::End(end_result) => {
                    todo!()
                },
                _ => {},
            }
        }
    }
}

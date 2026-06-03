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

    pub fn run(&mut self, rx: crossbeam_channel::Receiver<DispatchMessage>) {
        loop {
            while let Ok(dispatch_msg) = rx.try_recv() {
                self.handle_dispatch(dispatch_msg);
            }
            // TODO: refactor handler to be non-blocking try-recv
            while let Ok((session, bidi_msg)) = self.handler.try_recv() {
                self.handle_bidi(session, bidi_msg);
            }
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

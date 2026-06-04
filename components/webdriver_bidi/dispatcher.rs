use std::collections::HashMap;

use crossbeam_channel::{Receiver, Sender};
use embedder_traits::webdriver_bidi::RequestId;
use indexmap::IndexMap;
use log::error;
use rustenium_bidi_definitions::base::{CommandMessage, SuccessEnum};
use uuid::Uuid;

use crate::{
    connection::{Connection, ConnectionId},
    error::WebDriverBidiResult,
    handler::WebDriverBidiHandler,
    model::{CommandResponse, Message as BidiMessage, ResultData, SessionResult},
    session::{Session, SessionId},
};

// TODO: overall check which errors should be sent to client.

/// Messages sent from WebSocket connections to [`Dispatcher`].
#[derive(Debug)]
pub enum DispatchMessage {
    /// Deserialized BiDi command message, along with connection id.
    Command(ConnectionId, Box<CommandMessage>),
    // TODO: new connection may connect to existing session
    NewConnection(Connection),
    // TODO: connection close/lost
    /// The [`Handler`] instructs [`Dispatcher`] to create a new sesion with give session id.
    SessionNew(SessionId, Sender<bool>),
    /// Step 7 of `To handle an incoming message`
    Associate(SessionId, ConnectionId, RequestId),
    /// Pending request resolved, remove it from record.
    PendingResolved(RequestId),
}

/// Where the response should be sent to.
#[derive(Debug)]
enum Destination {
    /// For requests from unassociated connection,
    /// Response should be sent to the particular connection.
    Connection(ConnectionId),
    /// For request from an associated connection,
    /// Response should be sent to all connections associated with
    /// the session.
    Session(SessionId),
}

/// [`Destination`] along with command id.
#[derive(Debug)]
struct ResponseTarget {
    pub destination: Destination,
    pub id: u64,
}

#[derive(Debug)]
pub struct Dispatcher<T: WebDriverBidiHandler> {
    /// The default handler which handles static commands.
    static_handler: T,
    /// A remote end has a (ordered) set of WebSocket connections not associated
    /// with a session, which is initially empty.
    unassociated: IndexMap<ConnectionId, Connection>,
    /// Active sessions.
    active_sessions: IndexMap<SessionId, Session<T>>,
    /// A reverse map from connection to its associated session (can be `None`).
    /// Used for fast lookup when a command arrives from a connection
    session_by_connection: HashMap<ConnectionId, SessionId>,
    /// Current request id.
    request_id: RequestId,
    /// Recording pending requests and their target.
    pending_requests: HashMap<RequestId, ResponseTarget>,
    tx: Sender<DispatchMessage>,
    rx: Receiver<DispatchMessage>,
}

impl<T: WebDriverBidiHandler> Dispatcher<T> {
    pub fn new(
        static_handler: T,
        tx: Sender<DispatchMessage>,
        rx: Receiver<DispatchMessage>,
    ) -> Self {
        Self {
            static_handler,
            unassociated: Default::default(),
            active_sessions: Default::default(),
            session_by_connection: Default::default(),
            request_id: Default::default(),
            pending_requests: Default::default(),
            tx,
            rx,
        }
    }

    pub fn run(&mut self) {
        loop {
            // dispatch message
            while let Ok(dispatch_msg) = self.rx.try_recv() {
                self.handle_dispatch(dispatch_msg);
            }
            // static handler
            while let Ok((request_id, bidi_msg)) = self.static_handler.try_recv() {
                self.handle_bidi(request_id, None, bidi_msg);
            }
            // session handlers
            for (session_id, session) in self.active_sessions.iter() {
                let handler = session.handler();
                while let Ok((request_id, bidi_msg)) = handler.try_recv() {
                    self.handle_bidi(request_id, Some(*session_id), bidi_msg);
                }
            }
        }
    }
}

impl<T: WebDriverBidiHandler> Dispatcher<T> {
    /// Get all connections associated with the session
    fn session_connections(&self, session_id: SessionId) -> impl Iterator<Item = &Connection> {
        self.active_sessions
            .get(&session_id)
            .into_iter()
            .flat_map(Session::connections)
    }

    /// Get all connections related to the specific pending request.
    fn target_connections(
        &self,
        destination: &Destination,
    ) -> Box<dyn Iterator<Item = &Connection> + '_> {
        match destination {
            Destination::Connection(conn_id) => {
                Box::new(self.unassociated.get(conn_id).into_iter())
            },
            Destination::Session(session_id) => Box::new(self.session_connections(*session_id)),
        }
    }

    /// Get the target for the request (command) given its connection.
    fn get_response_target(&self, conn_id: ConnectionId, id: u64) -> Option<ResponseTarget> {
        let destination = self
            .unassociated
            .contains_key(&conn_id)
            .then_some(Destination::Connection(conn_id))
            .or_else(|| {
                self.session_by_connection
                    .get(&conn_id)
                    .copied()
                    .map(Destination::Session)
            })?;

        Some(ResponseTarget { destination, id })
    }

    /// Associate a connection with a session.
    // TODO: return Error instead of false
    // PRECONDITION: handle_session already checks conn is not associated
    fn associate(&mut self, conn_id: ConnectionId, session_id: SessionId) -> bool {
        let Some(connection) = self.unassociated.shift_remove(&conn_id) else {
            return false;
        };
        let Some(session) = self.active_sessions.get_mut(&session_id) else {
            error!("Failed to associate: Session {session_id} not found");
            return false;
        };

        session.append(connection);
        self.session_by_connection.insert(conn_id, session_id);

        true
    }

    fn handle_dispatch(&mut self, dispatch: DispatchMessage) {
        match dispatch {
            DispatchMessage::Command(conn_id, cmd_msg) => {
                self.handle_dispatch_command(conn_id, cmd_msg);
            },
            // TODO: session
            DispatchMessage::NewConnection(conn) => {
                self.unassociated.insert(conn.id(), conn);
            },
            DispatchMessage::SessionNew(session_id, sender) => {
                match self.static_handler.with_session_id(session_id) {
                    Some(handler) => {
                        let session = Session::new(session_id, handler);
                        self.active_sessions.insert(session_id, session);
                        sender.send(true);
                    },
                    None => {
                        sender.send(false);
                    },
                }
            },
            // After associate, finally we can resolve the pending session.new
            DispatchMessage::Associate(session_id, connection_id, request_id) => {
                // TODO: resolve session.new
            },
            DispatchMessage::PendingResolved(request_id) => {
                self.pending_requests.remove(&request_id);
            },
        }
    }

    /// Handle dispatch message, command branch
    fn handle_dispatch_command(&mut self, conn_id: ConnectionId, cmd_msg: Box<CommandMessage>) {
        let request_id = self.request_id.inc();

        let Some(target) = self.get_response_target(conn_id, cmd_msg.id) else {
            error!("Connection not registered");
            return;
        };

        let handler = match target.destination {
            Destination::Connection(_) => &self.static_handler,
            Destination::Session(session_id) => {
                &self.active_sessions.get(&session_id).unwrap().handler()
            },
        };

        match handler
            .handle(request_id, &cmd_msg, self.tx.clone())
            .transpose()
        {
            // TODO: immediate is really rare in bidi, may be removed to unify architecture.
            // immediate success or immediate error
            Some(_immediate) => {},
            // async result not reached
            None => {
                self.pending_requests.insert(request_id, target);
            },
        }
    }

    fn handle_bidi(
        &self,
        request_id: Option<RequestId>,
        session_id: Option<SessionId>,
        bidi: BidiMessage,
    ) {
        match request_id {
            Some(request_id) => {
                // TODO: special handle for some like session.new
                if let Some(target) = self.pending_requests.get(&request_id) {
                    self.send_to_target(target, bidi);
                    self.tx.send(DispatchMessage::PendingResolved(request_id));
                    // TODO: log error
                }
            },
            // Event
            None => match session_id {
                Some(session_id) => {
                    self.send_to_session(session_id, bidi);
                },
                // TODO: should refactor to avoid this
                None => unreachable!(),
            },
        }
    }

    // TODO: we cannot get complete BiDi message here, it should be constructed from target id
    fn send_to_session(&self, session_id: SessionId, event: BidiMessage) {
        for conn in self.session_connections(session_id) {
            conn.send(&event);
        }
    }

    fn send_to_target(&self, target: &ResponseTarget, bidi_msg: BidiMessage) {
        for conn in self.target_connections(&target.destination) {
            conn.send(&bidi_msg);
        }
    }
}

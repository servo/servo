use std::collections::HashMap;

use async_tungstenite::tungstenite;
use tokio::sync::mpsc;

use crate::session::SessionId;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct ConnectionId(usize);

// TODO: see how id.rs does self increment.
impl ConnectionId {
    pub fn inc(&mut self) -> Self {
        let old = self.clone();
        self.0 += 1;
        old
    }
}

/// A handle to the actual WebSocket connection.
#[derive(Debug)]
pub struct Connection {
    /// Send message to connection.
    pub tx: mpsc::UnboundedSender<tungstenite::Message>,
}

// TODO: flatten this into Dispatcher
/// A dimap containing all connections in a remote end, both associated
/// and unassociated.
#[derive(Debug, Default)]
pub struct ConnectionMap {
    pub associated: HashMap<SessionId, Vec<(ConnectionId, Connection)>>,
    pub unassociated: HashMap<ConnectionId, Connection>,
    pub reverse: HashMap<ConnectionId, Option<SessionId>>,
}

impl ConnectionMap {
    pub fn new(init_sessions: impl IntoIterator<Item = SessionId>) -> Self {
        let associated = init_sessions.into_iter().map(|s| (s, vec![])).collect();
        Self {
            associated,
            ..Default::default()
        }
    }

    pub fn associate(&mut self, conn_id: ConnectionId, session: &SessionId) -> bool {
        if let Some(conn) = self.unassociated.remove(&conn_id) {
            self.associated
                .entry(session.clone())
                .or_default()
                .push((conn_id, conn));
            self.reverse.insert(conn_id, Some(session.clone()));
            true
        } else {
            false
        }
    }

    pub fn session(&self, conn_id: ConnectionId) -> Option<&SessionId> {
        self.reverse.get(&conn_id).and_then(Option::as_ref)
    }

    pub fn connections(&self, session: &SessionId) -> impl Iterator<Item = &Connection> {
        self.associated
            .get(session)
            .into_iter()
            .flat_map(|v| v.iter().map(|i| &i.1))
    }

    pub fn add_session(&mut self, session: SessionId) -> bool {
        if self.associated.contains_key(&session) {
            return false;
        }
        self.associated.insert(session, vec![]);
        true
    }

    pub fn add_connection(
        &mut self,
        session: Option<SessionId>,
        conn_id: ConnectionId,
        conn: Connection,
    ) -> bool {
        if self.reverse.contains_key(&conn_id) {
            return false;
        }
        match &session {
            Some(session) => {
                let Some(v) = self.associated.get_mut(session) else {
                    return false;
                };
                v.push((conn_id, conn));
            },
            None => {
                self.unassociated.insert(conn_id, conn);
            },
        }
        self.reverse.insert(conn_id, session);
        true
    }
}

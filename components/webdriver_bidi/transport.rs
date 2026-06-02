use std::collections::HashMap;

use tokio::sync::mpsc;
use uuid::Uuid;

use crate::model::Message;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Session {
    pub id: String,
}

/// A handle to the actual WebSocket connection.
#[derive(Debug)]
pub struct Connection {
    /// Send message to connection.
    pub tx: mpsc::UnboundedSender<Message>,
}

/// A dimap containing all connections in a remote end, both associated
/// and unassociated.
#[derive(Debug, Default)]
pub struct ConnectionMap {
    pub associated: HashMap<Session, Vec<(Uuid, Connection)>>,
    pub unassociated: HashMap<Uuid, Connection>,
    pub reverse: HashMap<Uuid, Option<Session>>,
}

impl ConnectionMap {
    pub fn new(init_sessions: impl IntoIterator<Item = Session>) -> Self {
        let associated = init_sessions.into_iter().map(|s| (s, vec![])).collect();
        Self {
            associated,
            ..Default::default()
        }
    }

    pub fn associate(&mut self, uuid: &Uuid, session: &Session) -> bool {
        if let Some(conn) = self.unassociated.remove(uuid) {
            self.associated
                .entry(session.clone())
                .or_default()
                .push((*uuid, conn));
            self.reverse.insert(*uuid, Some(session.clone()));
            true
        } else {
            false
        }
    }

    pub fn session(&self, uuid: &Uuid) -> Option<&Session> {
        self.reverse.get(uuid).and_then(Option::as_ref)
    }

    pub fn connections(&self, session: &Session) -> impl Iterator<Item = &Connection> {
        self.associated
            .get(session)
            .into_iter()
            .map(|v| v.iter().map(|i| &i.1))
            .flatten()
    }

    pub fn add_session(&mut self, session: Session) -> bool {
        if self.associated.contains_key(&session) {
            return false;
        }
        self.associated.insert(session, vec![]);
        true
    }

    pub fn add_connection(
        &mut self,
        session: Option<Session>,
        uuid: Uuid,
        conn: Connection,
    ) -> bool {
        if self.reverse.contains_key(&uuid) {
            return false;
        }
        match &session {
            Some(session) => {
                let Some(v) = self.associated.get_mut(session) else {
                    return false;
                };
                v.push((uuid, conn));
            },
            None => {
                self.unassociated.insert(uuid, conn);
            },
        }
        self.reverse.insert(uuid, session);
        true
    }
}

use core::fmt;

use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use servo_base::id::{BrowsingContextId, WebViewId};
use uuid::Uuid;

use crate::{
    connection::{Connection, ConnectionId},
    handler::WebDriverBidiHandler,
};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct SessionId(pub Uuid);

impl fmt::Display for SessionId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

pub(crate) struct WebDriverBidiSession {
    id: Uuid,
    webview_id: Option<WebViewId>,
    browsing_context_id: Option<BrowsingContextId>,
}

#[derive(Debug)]
pub struct Session<T: WebDriverBidiHandler> {
    id: SessionId,
    /// A BiDi session has a (ordered) set of session WebSocket connections.
    /// This is initially empty.
    connections: IndexMap<ConnectionId, Connection>,
    handler: T,
}

impl<T: WebDriverBidiHandler> Session<T> {
    pub fn new(id: SessionId, handler: T) -> Self {
        Self {
            id,
            handler,
            connections: Default::default(),
        }
    }

    pub fn id(&self) -> SessionId {
        self.id
    }

    pub fn connections(&self) -> impl Iterator<Item = &Connection> {
        self.connections.values()
    }

    pub fn handler(&self) -> &T {
        &self.handler
    }

    pub fn contains(&self, connection_id: &ConnectionId) -> bool {
        self.connections.contains_key(connection_id)
    }

    /// Append connection to session's WebSocket connections ordered set.
    pub fn append(&mut self, connection: Connection) {
        self.connections.insert(connection.id(), connection);
    }
}

// TODO: doc naming convention
// - handle_xxx: remote end step for command
// - subscribe_xxx: remote end step for subscribe
// - trigger_xxx: trigger event
//
// TODO: doc deviation of async
// we do not rely on the concept of wait queue, reasons:
// 1. many steps implicitly async, but no described with wait queue.

use std::{
    cell::RefCell,
    collections::{HashMap, HashSet},
    fmt::Display,
};

use serde::{Deserialize, Serialize};
use servo_base::id::BrowsingContextId;
use uuid::Uuid;
use webdriver_traits::bidi::LogEvent;

use crate::bidi::connection::ConnectionId;

pub(crate) struct Session {
    pub(crate) id: SessionId,
    /// The BiDi flag.
    pub(crate) bidi: bool,
    /// The http flag.
    pub(crate) http: bool,
    // TODO: re-model flags as bitflag
    pub(crate) flags: HashSet<&'static str>,
    /// The session's associated connections.
    pub(crate) connections: HashSet<ConnectionId>,

    pub(crate) log_event_buffer:
        RefCell<HashMap<BrowsingContextId, Vec<(LogEvent, Vec<BrowsingContextId>)>>>,
}

impl Session {
    /// <https://www.w3.org/TR/webdriver-bidi/#associated-with-connection>
    pub(crate) fn is_associated_with_connection(&self, connection_id: &ConnectionId) -> bool {
        self.connections.contains(connection_id)
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub(crate) struct SessionId(pub(crate) Uuid);

impl Display for SessionId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

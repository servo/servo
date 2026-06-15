//! Common components of a session.

use core::fmt;
use std::{ops::Deref, rc::Rc};

use crossbeam_channel::Sender;
use embedder_traits::{EmbedderMsg, GenericEmbedderProxy};
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};
use uuid::Uuid;
use webdriver_traits::{
    ScriptToWebDriverMessage, WebDriverMessage, WebDriverToConstellationMessage,
};

use crate::bidi::{RemoteEndState, connection::Connection, session::Session};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct SessionId(pub Uuid);

/// The messages sent to a session.
pub enum SessionMessage {
    /// Append connection to session's connection set.
    Associate(Connection),
    /// The session should do run "cleanup the session" when
    /// receiving this in next tick. This is used to defer
    /// actual cleanup to after sending resposne to connection.
    Cleanup,
    // Constellation messages are forwarded to all session, we use Rc to avoid cloning.
    WebDriver(Rc<WebDriverMessage>),
    // Script messages are forwarded to all session, we use Rc to avoid cloning.
    Script(Rc<ScriptToWebDriverMessage>),
}

/// The common components of a session, regardless of static, http or bidi.
pub struct CommonPart {
    pub(crate) remote_end_state: Rc<RemoteEndState>,
    pub(crate) embedder_proxy: GenericEmbedderProxy<EmbedderMsg>,
    pub(crate) constellation_sender: Sender<WebDriverToConstellationMessage>,
    pub(crate) session_sender: UnboundedSender<SessionMessage>,
    pub(crate) session_receiver: UnboundedReceiver<SessionMessage>,
}

impl Deref for Session {
    type Target = CommonPart;
    fn deref(&self) -> &Self::Target {
        match self {
            Session::Static { common, .. } | Session::BidiOnly { common, .. } => common,
        }
    }
}

impl fmt::Display for SessionId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<Uuid> for SessionId {
    fn from(value: Uuid) -> Self {
        Self(value)
    }
}

impl From<SessionId> for Uuid {
    fn from(value: SessionId) -> Self {
        value.0
    }
}

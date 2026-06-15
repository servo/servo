use tokio::sync::mpsc::UnboundedSender;

use crate::bidi::{connection::Connection, session::common::SessionMessage};

/// In rust we have single ownership rule.
/// So only session itself owns the data, while others only channel to it.
#[derive(Clone)]
pub struct SessionProxy {
    pub(crate) bidi_flag: bool,
    pub(crate) sender: UnboundedSender<SessionMessage>,
}

impl SessionProxy {
    pub(crate) async fn associate(&self, connection: Connection) {
        self.sender.send(SessionMessage::Associate(connection));
    }
}

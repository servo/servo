use tokio::sync::{mpsc::UnboundedSender, oneshot};
use webdriver_traits::bidi::ErrorCode;

use crate::bidi::{connection::ConnectionOld, session::common::SessionMessage};

/// In rust we have single ownership rule.
/// So only session itself owns the data, while others only channel to it.
#[derive(Clone)]
pub struct SessionProxy {
    pub(crate) bidi_flag: bool,
    pub(crate) sender: UnboundedSender<SessionMessage>,
}

impl SessionProxy {
    pub(crate) async fn associate(&self, connection: ConnectionOld) {
        self.sender.send(SessionMessage::Associate(connection));
    }

    /// <https://www.w3.org/TR/webdriver-bidi/#cleanup-the-session>
    /// On the proxy side, we send a `CleanupSession` message to the session,
    /// and wait for response.
    pub(crate) async fn cleanup_the_session(&self) -> Result<(), ErrorCode> {
        let (sender, receiver) = oneshot::channel();
        self.sender
            .send(SessionMessage::CleanupSession(Some(sender)));
        match receiver.await {
            Ok(true) => Ok(()),
            Ok(false) => {
                log::warn!("Unable to cleanup the session");
                Err(ErrorCode::UnknownError)
            },
            Err(e) => {
                log::warn!("Receiving cleanup callback failed: {e:?}");
                Err(ErrorCode::UnknownError)
            },
        }
    }
}

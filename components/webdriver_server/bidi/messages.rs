use async_tungstenite::tungstenite::Message as WsMessage;
use log::warn;
use webdriver_traits::{
    ids::{ConnectionId, RealmId},
    messages::WebDriverToScriptMessage,
};

use crate::bidi::WebDriverBidiThread;

impl WebDriverBidiThread {
    pub(crate) fn send_to_realm(&self, realm_id: RealmId, msg: WebDriverToScriptMessage) {
        match self.realms.get(&realm_id) {
            None => warn!("Missing sender for realm {realm_id}"),
            Some(realm) => {
                if let Err(err) = realm.sender.send(msg) {
                    warn!("Sending message to realm {realm_id} failed ({err:?})");
                }
            },
        }
    }
}

pub(crate) enum WebDriverToServerMessage {
    Message(ConnectionId, WsMessage),
}

pub(crate) enum ServerToWebDriverMessage {
    /// new connection
    Connection(ConnectionId, Option<String>),
    /// Websocket message
    Message(ConnectionId, WsMessage),
}

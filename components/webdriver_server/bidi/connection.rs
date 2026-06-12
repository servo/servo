use std::sync::atomic::{AtomicU64, Ordering::SeqCst};

use async_tungstenite::{WebSocketStream, tokio::TokioAdapter, tungstenite::Message as WsMessage};
use log::error;
use tokio::{net::TcpStream, sync::mpsc::UnboundedSender};

use webdriver_traits::bidi::Message as BidiMessage;

/// A WebSocket connection is a network connection that follows the requirements of
/// the WebSocket protocol.
pub enum Connection {
    Tcp(WebSocketStream<TokioAdapter<TcpStream>>),
    // TODO: Tls
}

impl From<WebSocketStream<TokioAdapter<TcpStream>>> for Connection {
    fn from(value: WebSocketStream<TokioAdapter<TcpStream>>) -> Self {
        Self::Tcp(value)
    }
}

// TODO: do we need connection id now? e.g. for log?

static CONNECTION_ID: AtomicU64 = AtomicU64::new(0);

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, PartialOrd, Ord)]
pub struct ConnectionId(u64);

impl From<u64> for ConnectionId {
    fn from(value: u64) -> Self {
        Self(value)
    }
}

impl ConnectionId {
    pub fn next() -> Self {
        Self(CONNECTION_ID.fetch_add(1, SeqCst))
    }
}

/// A handle to the WebSocket connection on BiDi server thread.
#[derive(Debug)]
pub struct ConnectionOld {
    id: ConnectionId,
    tx: UnboundedSender<WsMessage>,
}

impl ConnectionOld {
    pub fn new(id: ConnectionId, tx: UnboundedSender<WsMessage>) -> Self {
        Self { id, tx }
    }

    pub fn id(&self) -> ConnectionId {
        self.id
    }

    pub fn send(&self, bidi_msg: &BidiMessage) {
        // PERF: duplicate serialize here, for each connection in target.
        // but we should abstract away ws message type from dispatcher.
        // an once cache type should be used.
        let Ok(serialized) = serde_json::to_string(&bidi_msg) else {
            error!("fail to serialize: {:?}", bidi_msg);
            return;
        };
        // TODO: serialize error should also be sent.
        let ws_msg = WsMessage::Text(serialized.into());
        if let Err(err) = self.tx.send(ws_msg) {
            // As channel is already broken, there is no need to retry send.
            error!("fail to send ws message: {:?}", err);
        }
    }
}

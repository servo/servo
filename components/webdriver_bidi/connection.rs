use async_tungstenite::tungstenite::Message as WsMessage;
use log::error;
use tokio::sync::mpsc::UnboundedSender;

use crate::model::Message as BidiMessage;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct ConnectionId(usize);

// TODO: see how id.rs does self increment.
impl ConnectionId {
    pub fn inc(&mut self) -> Self {
        let old = *self;
        self.0 += 1;
        old
    }
}

/// A handle to the WebSocket connection on BiDi server thread.
#[derive(Debug)]
pub struct Connection {
    id: ConnectionId,
    tx: UnboundedSender<WsMessage>,
}

impl Connection {
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

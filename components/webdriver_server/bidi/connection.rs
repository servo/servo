use std::ops::{Deref, DerefMut};

use async_tungstenite::{WebSocketReceiver, WebSocketSender, WebSocketStream, tokio::TokioAdapter};
use tokio::net::TcpStream;
use webdriver_traits::bidi::{ErrorCode, ErrorResponse, Message as BidiMessage};

// TODO: support wss in the future. at that time a enum should be used.

pub(crate) type Connection = (ConnectionId, ConnectionSender, ConnectionReceiver);

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(crate) struct ConnectionId(pub(crate) u64);

pub(crate) type ConnectionSender = WebSocketSender<TokioAdapter<TcpStream>>;
pub(crate) type ConnectionReceiver = WebSocketReceiver<TokioAdapter<TcpStream>>;

/// A WebSocket connection.
pub(crate) struct ConnectionOld(pub(crate) WebSocketStream<TokioAdapter<TcpStream>>);

impl From<WebSocketStream<TokioAdapter<TcpStream>>> for ConnectionOld {
    fn from(value: WebSocketStream<TokioAdapter<TcpStream>>) -> Self {
        Self(value)
    }
}

impl Deref for ConnectionOld {
    type Target = WebSocketStream<TokioAdapter<TcpStream>>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for ConnectionOld {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl ConnectionOld {
    /// Send an error response to the WebSocket connection.
    ///
    /// <https://www.w3.org/TR/webdriver-bidi/#send-an-error-response>
    pub(crate) async fn send_an_error_response(
        &mut self,
        command_id: Option<u64>,
        error_code: ErrorCode,
    ) {
        // 1
        let error_data = BidiMessage::ErrorResponse(Box::new(ErrorResponse {
            id: command_id,
            error: error_code,
            // SKIP: implementation-defined
            message: "".to_string(),
            stacktrace: None,
            extensible: Default::default(),
        }));
        // 2.
        let response = match serde_json::to_string(&error_data) {
            Ok(response) => response,
            Err(e) => {
                log::warn!("Serializing error response failed: {e:?}");
                return;
            },
        };
        // 3.
        if let Err(e) = self.0.send(response.into()).await {
            log::warn!("Sending error response failed: {e:?}");
        }
    }
}

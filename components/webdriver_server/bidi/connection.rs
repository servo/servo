use std::{
    cell::Cell,
    ops::{Deref, DerefMut},
};

use async_tungstenite::{WebSocketReceiver, WebSocketSender, WebSocketStream, tokio::TokioAdapter};
use tokio::net::TcpStream;
use webdriver_traits::bidi::{ErrorCode, ErrorResponse, Message as BidiMessage};

thread_local! {
    static CONNECTITON_ID: Cell<u64> = Cell::new(0);
}

// TODO: support wss in the future. at that time a enum should be used.

pub(crate) type Connection = (ConnectionId, ConnectionSender, ConnectionReceiver);

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(crate) struct ConnectionId(u64);

impl ConnectionId {
    pub(crate) fn next() -> Self {
        Self(CONNECTITON_ID.replace(CONNECTITON_ID.get() + 1))
    }
}

pub(crate) type ConnectionSender = WebSocketSender<TokioAdapter<TcpStream>>;
pub(crate) type ConnectionReceiver = WebSocketReceiver<TokioAdapter<TcpStream>>;

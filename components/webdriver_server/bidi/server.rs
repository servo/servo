use std::{
    collections::HashMap,
    net::{SocketAddr, SocketAddrV4},
    thread::JoinHandle,
};

use async_tungstenite::{
    WebSocketStream,
    tokio::{TokioAdapter, accept_async},
    tungstenite,
};
use crossbeam_channel::Sender;
use futures_util::{FutureExt, StreamExt, future};
use log::warn;
use tokio::{
    net::{TcpListener, TcpStream},
    sync::mpsc::UnboundedReceiver,
};
use webdriver_traits::ids::ConnectionId;

use crate::bidi::messages::{ServerToWebDriverMessage, WebDriverToServerMessage};

pub(crate) struct WebDriverServer {
    w2s_recever: UnboundedReceiver<WebDriverToServerMessage>,
    s2w_sender: Sender<ServerToWebDriverMessage>,
    port: u16,
    streams: HashMap<ConnectionId, WebSocketStream<TokioAdapter<TcpStream>>>,
}

impl WebDriverServer {
    pub(crate) fn start(
        w2s_recever: UnboundedReceiver<WebDriverToServerMessage>,
        s2w_sender: Sender<ServerToWebDriverMessage>,
        port: u16,
    ) -> JoinHandle<()> {
        std::thread::Builder::new()
            .name("WebDriverServer".to_string())
            .spawn(move || {
                let rt = tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()
                    .expect("Build tokio rt failed");
                rt.block_on(Self::new(w2s_recever, s2w_sender, port).run())
            })
            .expect("start webdriver server failed")
    }

    fn new(
        w2s_recever: UnboundedReceiver<WebDriverToServerMessage>,
        s2w_sender: Sender<ServerToWebDriverMessage>,
        port: u16,
    ) -> Self {
        Self {
            w2s_recever,
            s2w_sender,
            port,
            streams: Default::default(),
        }
    }

    async fn run(&mut self) {
        let address = SocketAddrV4::new("0.0.0.0".parse().unwrap(), self.port);
        let listener = TcpListener::bind(address)
            .await
            .expect("Binding listener failed");
        loop {
            let receiver_next = self.w2s_recever.recv();
            let listener_next = listener.accept();
            let streams_next = future::select_all(
                self.streams
                    .iter_mut()
                    .map(|(conn_id, conn)| conn.next().map(move |msg| (*conn_id, msg))),
            )
            .map(|(id_msg, _, _)| id_msg);

            tokio::select! {
                msg = receiver_next => self.handle_from_webdriver(msg).await,
                msg = listener_next => self.handle_from_listener(msg).await,
                (conn_id, msg) = streams_next => self.handle_from_stream(conn_id, msg),
            }
        }
    }

    async fn handle_from_webdriver(&mut self, msg: Option<WebDriverToServerMessage>) {
        if let Some(msg) = msg {
            match msg {
                WebDriverToServerMessage::Message(conn_id, msg) => {
                    if let Some(stream) = self.streams.get_mut(&conn_id) {
                        if let Err(err) = stream.send(msg).await {
                            warn!("Sending message to websocket failed ({err:?})");
                        }
                    }
                },
            }
        }
    }

    async fn handle_from_listener(
        &mut self,
        msg: Result<(TcpStream, SocketAddr), tokio::io::Error>,
    ) {
        match msg {
            Err(err) => {
                warn!("Accepting new connection failed ({err:?})");
            },
            Ok((stream, _)) => {
                let ws_stream = match accept_async(stream).await {
                    Err(err) => {
                        warn!("Accepting websocket failed ({err:?})");
                        return;
                    },
                    Ok(stream) => stream,
                };
                let conn_id = ConnectionId::next();
                self.streams.insert(conn_id, ws_stream);
                if let Err(err) = self
                    .s2w_sender
                    .send(ServerToWebDriverMessage::Connection(conn_id, None))
                {
                    warn!("Notifying new connection failed ({err:?})");
                }
            },
        }
    }

    fn handle_from_stream(
        &mut self,
        conn_id: ConnectionId,
        msg: Option<Result<tungstenite::Message, tungstenite::Error>>,
    ) {
        let Some(Ok(msg)) = msg else {
            warn!("Receiving message from connection ({conn_id}) failed");
            return;
        };
        if let Err(err) = self
            .s2w_sender
            .send(ServerToWebDriverMessage::Message(conn_id, msg))
        {
            warn!("Forwarding message to webdriver failed ({err:?})");
        }
    }
}

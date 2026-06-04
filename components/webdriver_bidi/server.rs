use std::{
    net::{SocketAddr, TcpListener as StdTcpListener},
    thread::{self, JoinHandle},
};

use async_tungstenite::{
    WebSocketStream,
    tokio::{TokioAdapter, accept_hdr_async},
    tungstenite::{
        self, Message,
        handshake::server::{ErrorResponse as WsErrorResponse, Request, Response},
    },
};
use futures_util::StreamExt;
use log::error;
use rustenium_bidi_definitions::base::{CommandMessage, ErrorCode, ErrorEnum, ErrorResponse};
use tokio::{
    net::{TcpListener, TcpStream},
    sync::mpsc,
};

use crate::{
    connection::{Connection, ConnectionId},
    dispatcher::{DispatchMessage, Dispatcher},
    handler::WebDriverBidiHandler,
};

/// A WebSocket Listener.
///
/// See <https://www.w3.org/TR/webdriver-bidi/#websocket-listener>.
pub struct Listener {
    guard: Option<thread::JoinHandle<()>>,
    /// Host and port
    pub socket: SocketAddr,
}

impl Drop for Listener {
    fn drop(&mut self) {
        let _ = self.guard.take().map(JoinHandle::join);
    }
}

/// To start listening for a WebSocket connection.
///
/// See <https://www.w3.org/TR/webdriver-bidi/#start-listening-for-a-websocket-connection>.
///
/// ## NOTE
///
/// Currently this implementation only supports [BiDi-only sesion](https://www.w3.org/TR/webdriver-bidi/#supports-bidi-only-sessions)
/// and does not support upgrade from WebDriver classic. So the only WebSocket resource is `/session`.
///
/// ## NOTE
///
/// WebDriver Bidi allows implementation to reuse existing listener, and there is no reason to
/// have multiple active listeners for non-intermediary node like servo, thus step 4 is ignored.
pub fn start<T>(
    mut address: SocketAddr,
    handler: T,
    // TODO: implementation defined check like allow_hosts
) -> ::std::io::Result<Listener>
where
    T: 'static + WebDriverBidiHandler,
{
    let listener = StdTcpListener::bind(address)?;
    listener.set_nonblocking(true)?;
    let addr = listener.local_addr()?;
    if address.port() == 0 {
        address.set_port(addr.port());
    }

    let (dispatch_tx, dispatch_rx) = crossbeam_channel::unbounded::<DispatchMessage>();

    let builder = thread::Builder::new().name("webdriver bidi server".to_string());
    let handle = builder.spawn({
        let dispatch_tx = dispatch_tx.clone();
        {
            move || {
                let rt = tokio::runtime::Builder::new_current_thread()
                    .enable_io()
                    .build()
                    .expect("fail to create tokio runtime");
                rt.block_on(async {
                    // from_std must be called in IO-enabled runtime
                    let listener = TcpListener::from_std(listener)
                        .expect("fail to convert TcpListener to tokio");
                    serve(listener, dispatch_tx).await
                });
            }
        }
    })?;

    let builder = thread::Builder::new().name("webdriver dispatcher".to_string());
    builder.spawn(move || {
        Dispatcher::new(handler, dispatch_tx, dispatch_rx).run();
    })?;

    Ok(Listener {
        guard: Some(handle),
        socket: addr,
    })
}

async fn serve(listener: TcpListener, dispatch_tx: crossbeam_channel::Sender<DispatchMessage>) {
    let mut conn_id = ConnectionId::default();

    while let Ok((stream, _)) = listener.accept().await {
        let conn_id = conn_id.inc();
        let (conn_tx, conn_rx) = mpsc::unbounded_channel::<tungstenite::Message>();

        dispatch_tx
            .send(DispatchMessage::NewConnection(Connection::new(
                conn_id, conn_tx,
            )))
            .expect("fail to send tx");

        tokio::spawn({
            let dispatch_tx = dispatch_tx.clone();
            async move {
                let ws_stream = accept_hdr_async(stream, should_accept_connection())
                    .await
                    .unwrap();
                handle_ws_stream(conn_id, ws_stream, dispatch_tx, conn_rx).await
            }
        });
    }
}

fn should_accept_connection() -> impl FnOnce(&Request, Response) -> Result<Response, WsErrorResponse>
{
    // CLIPPY: we cannot change external type in tungstenite
    #[allow(clippy::result_large_err)]
    |request, response| {
        // TODO: 7 steps
        let _path = request.uri().path().to_string();
        Ok(response)
    }
}

async fn handle_ws_stream(
    conn_id: ConnectionId,
    mut stream: WebSocketStream<TokioAdapter<TcpStream>>,
    dispatch_tx: crossbeam_channel::Sender<DispatchMessage>,
    mut conn_rx: mpsc::UnboundedReceiver<tungstenite::Message>,
) {
    tokio::select! {
        Some(ws) = stream.next() => {
            handle_ws(conn_id, &mut stream, ws, dispatch_tx).await
        }
        Some(msg) = conn_rx.recv() => {
            handle_bidi(&mut stream, msg).await
        }
    }

    // TODO: resource path and init session
    async fn handle_ws(
        conn_id: ConnectionId,
        stream: &mut WebSocketStream<TokioAdapter<TcpStream>>,
        ws: Result<Message, tungstenite::Error>,
        dispatch_tx: crossbeam_channel::Sender<DispatchMessage>,
    ) {
        let Ok(ws) = ws else {
            return;
        };
        let Message::Text(text) = ws else {
            send_invalid_argument_error(stream, None).await;
            return;
        };
        let Ok(command) = serde_json::from_str::<CommandMessage>(&text) else {
            send_invalid_argument_error(stream, None).await;
            return;
        };
        if let Err(err) = dispatch_tx.send(DispatchMessage::Command(conn_id, Box::new(command))) {
            error!("Error sending message to dispatcher: {err}");
        }
    }

    async fn handle_bidi(
        stream: &mut WebSocketStream<TokioAdapter<TcpStream>>,
        msg: tungstenite::Message,
    ) {
        if let Err(err) = stream.send(msg).await {
            error!("Error sending message to webdriver bidi client: {err}");
        }
    }

    async fn send_invalid_argument_error(
        stream: &mut WebSocketStream<TokioAdapter<TcpStream>>,
        id: Option<u64>,
    ) {
        let error = ErrorResponse {
            r#type: ErrorEnum::Error,
            id,
            error: ErrorCode::InvalidArgument,
            message: "invalid argumennt".to_string(),
            stacktrace: None,
            extensible: Default::default(),
        };
        let response = match serde_json::to_string(&error) {
            Ok(response) => response,
            Err(err) => {
                format!(
                    r#"{{"type":"error","id":{},"error":"unknown error","message":"fail to serializie error response: {}"}}"#,
                    id.map(|i| i.to_string()).unwrap_or("null".to_string()),
                    err
                )
            },
        };
        if let Err(err) = stream.send(Message::Text(response.into())).await {
            error!("Error sending error to client: {err}");
        }
    }
}

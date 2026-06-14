use std::{
    net::{SocketAddr, TcpListener as StdTcpListener},
    rc::Rc,
};

use async_tungstenite::{
    tokio::accept_hdr_async,
    tungstenite::handshake::server::{ErrorResponse as WsErrorResponse, Request, Response},
};
use log::info;
use tokio::{net::TcpListener, sync::RwLock};

use crate::bidi::ActiveSessions;

pub struct Listener {
    address: SocketAddr,
    active_sessions: Rc<RwLock<ActiveSessions>>,
}

impl Listener {
    pub fn new(address: SocketAddr, active_sessions: Rc<RwLock<ActiveSessions>>) -> Self {
        Self {
            address,
            active_sessions,
        }
    }

    pub async fn run(mut self) {
        let listener = TcpListener::bind(self.address).await.unwrap();
        let addr = listener.local_addr().unwrap();
        if self.address.port() == 0 {
            self.address.set_port(addr.port());
        }
        info!("WebDriver BiDi server listening on {}", self.address);

        while let Ok((stream, _)) = listener.accept().await {
            let ws_stream = accept_hdr_async(stream, should_accept_connection())
                .await
                .expect("Accept websocket stream fails");
            {
                self.active_sessions
                    .read()
                    .await
                    // TODO: clone to minimize
                    // TODO: impl parse session id and send to specific session
                    .get(&None)
                    .expect("static session missing")
                    .associate(ws_stream.into())
                    .await;
            }
        }
    }

    // TODO: previous should call this fn
    // The spec does not have a link for this.
    pub fn accept_the_incoming_connection(&self) {
        // 1. Let resource name be the resource name from reading
        // the client’s opening handshake. If resource name is not
        // n listener’s list of WebSocket resources, then stop running
        // these steps and act as if the requested service is not available.
        todo!()
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

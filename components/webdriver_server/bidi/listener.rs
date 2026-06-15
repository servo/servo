use std::{net::SocketAddr, sync::Arc};

use async_tungstenite::{
    tokio::accept_hdr_async,
    tungstenite::handshake::server::{ErrorResponse as WsErrorResponse, Request, Response},
};
use log::info;
use tokio::{
    net::TcpListener,
    sync::{RwLock, mpsc::UnboundedSender},
    task,
};

use crate::bidi::{ActiveSessions, session::common::SessionMessage};

pub struct Listener {
    address: SocketAddr,
    active_sessions: Arc<RwLock<ActiveSessions>>,
    static_sender: UnboundedSender<SessionMessage>,
}

impl Listener {
    /// Start `Listener` as a tokio local task.
    pub(crate) fn start(
        address: SocketAddr,
        active_sessions: Arc<RwLock<ActiveSessions>>,
        static_sender: UnboundedSender<SessionMessage>,
    ) -> task::JoinHandle<()> {
        task::spawn_local(Self::new(address, active_sessions, static_sender).run())
    }

    fn new(
        address: SocketAddr,
        active_sessions: Arc<RwLock<ActiveSessions>>,
        static_sender: UnboundedSender<SessionMessage>,
    ) -> Self {
        Self {
            address,
            active_sessions,
            static_sender,
        }
    }

    async fn run(mut self) {
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
                // TODO: impl parse session id and send to specific session
                if let Err(e) = self
                    .static_sender
                    .send(SessionMessage::Associate(ws_stream.into()))
                {
                    log::warn!("Send connection error: {e:?}");
                };
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

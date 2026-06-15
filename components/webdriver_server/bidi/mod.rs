mod connection;
mod error;
mod listener;
mod session;

use std::{
    collections::HashMap,
    net::{SocketAddr, SocketAddrV4},
    rc::Rc,
    sync::Arc,
    thread::{self},
};

use crossbeam_channel::{Receiver, Sender, unbounded};
use embedder_traits::{EmbedderMsg, GenericEmbedderProxy};
use tokio::{
    join,
    sync::{
        RwLock,
        mpsc::{self, UnboundedReceiver, UnboundedSender},
    },
};
use webdriver_traits::{WebDriverMessage, WebDriverToConstellationMessage};

use crate::bidi::{
    listener::Listener,
    session::{
        Session,
        common::{SessionId, SessionMessage},
        proxy::SessionProxy,
    },
};

// TODO: this should later be renamed to `WebDriverServer`
// after classic is merged.
pub struct WebDriverBidiThread {
    port: u16,
    embedder_proxy: GenericEmbedderProxy<EmbedderMsg>,
    constellation_sender: Sender<WebDriverToConstellationMessage>,

    // Remote end states are shared across all sessions.
    // Though this is a single threaded
    remote_end_state: Rc<RemoteEndState>,
}

impl WebDriverBidiThread {
    pub fn start(
        embedder_proxy: GenericEmbedderProxy<EmbedderMsg>,
    ) -> (
        UnboundedSender<WebDriverMessage>,
        Receiver<WebDriverToConstellationMessage>,
    ) {
        let (c2w_sender, c2w_receiver) = mpsc::unbounded_channel();
        let (w2c_sender, w2c_receiver) = unbounded();

        thread::Builder::new()
            .name("WebDriverBiDi".to_string())
            .spawn(move || {
                WebDriverBidiThread::new(0, embedder_proxy, w2c_sender).run(c2w_receiver);
            })
            .expect("Thread spawning failed");

        (c2w_sender, w2c_receiver)
    }

    fn new(
        port: u16,
        embedder_proxy: GenericEmbedderProxy<EmbedderMsg>,
        constellation_sender: Sender<WebDriverToConstellationMessage>,
    ) -> Self {
        Self {
            port,
            embedder_proxy,
            constellation_sender,
            remote_end_state: Default::default(),
        }
    }

    fn run(&self, receiver: UnboundedReceiver<WebDriverMessage>) {
        let address = SocketAddr::V4(SocketAddrV4::new("0.0.0.0".parse().unwrap(), self.port));
        tokio::runtime::LocalRuntime::new()
            .expect("Runtime creation failed")
            .block_on(async move {
                let remote_end_state = &self.remote_end_state;
                let active_sessions = &remote_end_state.active_sessions;

                let (session, sender) = Session::start_static(remote_end_state.clone());
                let listener = Listener::start(address, active_sessions.clone(), sender);
                let bridge =
                    Self::forward_constellation_messages(active_sessions.clone(), receiver);

                join!(listener, session, bridge);
            });
    }

    /// Forward constellation message to each active session.
    async fn forward_constellation_messages(
        active_session: Arc<RwLock<ActiveSessions>>,
        mut receiver: UnboundedReceiver<WebDriverMessage>,
    ) {
        while let Some(msg) = receiver.recv().await {
            let msg = Rc::new(msg);
            for session in active_session.read().await.values() {
                if let Err(e) = session.sender.send(SessionMessage::WebDriver(msg.clone())) {
                    log::warn!("Sending constellation message to session failed: {e:?}");
                }
            }
        }
    }
}

/// Global state of a remote end is grouped here to be shared
/// across sessions.
#[derive(Default)]
pub struct RemoteEndState {
    /// The active sessions of a remote end.
    /// This uses `Arc` to allow access from ROUTER thread.
    active_sessions: Arc<RwLock<ActiveSessions>>,
}

type ActiveSessions = HashMap<SessionId, SessionProxy>;

impl RemoteEndState {
    /// <https://www.w3.org/TR/webdriver-bidi/#cleanup-remote-end-state>
    fn cleanup(&self) {
        // 1. TODO: blocked by network module
        // 2. TODO: blocked by network module
        // 3. TODO: blocked by network module
        // 4. SKIP: implementation-defined
    }
}

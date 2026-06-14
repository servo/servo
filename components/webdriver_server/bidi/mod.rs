mod connection;
mod error;
mod listener;
mod session;

use std::{
    collections::HashMap,
    net::{SocketAddr, SocketAddrV4},
    rc::Rc,
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
use webdriver_traits::{ConstellationToWebDriverMessage, WebDriverToConstellationMessage};

use crate::bidi::{
    listener::Listener,
    session::{Session, SessionId, SessionProxy},
};

// TODO: this should later be renamed to `WebDriverServer`
// after classic is merged.
pub struct WebDriverBidi {
    port: u16,
    embedder_proxy: GenericEmbedderProxy<EmbedderMsg>,
    constellation_sender: Sender<WebDriverToConstellationMessage>,
    active_sessions: Rc<RwLock<ActiveSessions>>,
}

impl WebDriverBidi {
    pub fn start(
        embedder_proxy: GenericEmbedderProxy<EmbedderMsg>,
    ) -> (
        UnboundedSender<ConstellationToWebDriverMessage>,
        Receiver<WebDriverToConstellationMessage>,
    ) {
        let (c2w_sender, c2w_receiver) = mpsc::unbounded_channel();
        let (w2c_sender, w2c_receiver) = unbounded();

        thread::Builder::new()
            .name("WebDriverBiDi".to_string())
            .spawn(move || {
                WebDriverBidi::new(0, embedder_proxy, w2c_sender).run(c2w_receiver);
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
            active_sessions: Default::default(),
        }
    }

    fn run(&self, receiver: UnboundedReceiver<ConstellationToWebDriverMessage>) {
        let address = SocketAddr::V4(SocketAddrV4::new("0.0.0.0".parse().unwrap(), self.port));
        tokio::runtime::LocalRuntime::new()
            .expect("Runtime creation failed")
            .block_on(async move {
                let listener = Listener::new(address, self.active_sessions.clone()).run();
                let session = Session::new(self.active_sessions.clone()).run();
                let bridge =
                    Self::bridge_constellation_messages(self.active_sessions.clone(), receiver);
                join!(listener, session, bridge);
            });
    }

    /// bridge constellation message
    async fn bridge_constellation_messages(
        active_session: Rc<RwLock<ActiveSessions>>,
        mut receiver: UnboundedReceiver<ConstellationToWebDriverMessage>,
    ) {
        // TODO:
    }
}

type ActiveSessions = HashMap<Option<SessionId>, SessionProxy>;

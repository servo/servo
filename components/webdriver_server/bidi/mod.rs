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

use crossbeam_channel::Sender;
use embedder_traits::{EventLoopWaker, webdriver_bidi::WebDriverBidiToEmbedderMessage};
use tokio::{join, sync::RwLock};

use crate::bidi::{
    listener::Listener,
    session::{Session, SessionId, SessionProxy},
};

// Later BiDi and Classic may be merged into one thread.
pub struct WebDriverBidiThread {
    port: u16,
    embedder_sender: Sender<WebDriverBidiToEmbedderMessage>,
    event_loop_waker: Box<dyn EventLoopWaker>,
    active_sessions: Rc<RwLock<ActiveSessions>>,
}

impl WebDriverBidiThread {
    pub fn new(
        port: u16,
        embedder_sender: Sender<WebDriverBidiToEmbedderMessage>,
        event_loop_waker: Box<dyn EventLoopWaker>,
    ) -> Self {
        Self {
            port,
            embedder_sender,
            event_loop_waker,
            active_sessions: Rc::new(RwLock::new(Default::default())),
        }
    }

    pub fn start(&self) {
        let address = SocketAddr::V4(SocketAddrV4::new("0.0.0.0".parse().unwrap(), self.port));
        tokio::runtime::LocalRuntime::new()
            .expect("Runtime creation failed")
            .block_on(async move {
                let mut listener = Listener::new(address, self.active_sessions.clone());
                let mut session = Session::new(self.active_sessions.clone());
                join!(listener.start(), session.start());
            });
    }

    // TODO: return a channel, like in devtools
    pub fn spawn(
        port: u16,
        embedder_sender: Sender<WebDriverBidiToEmbedderMessage>,
        event_loop_waker: Box<dyn EventLoopWaker>,
    ) {
        thread::Builder::new()
            .name("WebDriverBiDi".to_string())
            .spawn(move || {
                WebDriverBidiThread::new(port, embedder_sender, event_loop_waker).start();
            })
            .expect("Thread spawning failed");
    }
}

type ActiveSessions = HashMap<Option<SessionId>, SessionProxy>;

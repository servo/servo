//! Decouple handling logic using trait.
//!
//! Also provide a default implementation.

use embedder_traits::EventLoopWaker;
use rustenium_bidi_definitions::base::CommandMessage;

use crate::{model::Message, transport::Session};

// TODO: should handler be per session?
pub trait WebDriverBidiHandler: Send {
    /// Start processing of a command.
    fn process(&self, session: &Option<Session>, command: &CommandMessage);

    /// Receive BiDi message.
    fn recv(&self) -> impl Future<Output = (Option<Session>, Message)>;

    // TODO: do we need
    // post update after receiving message
    // fn update(&mut self, message: &Message);
}

pub struct Handler {
    event_loop_waker: Box<dyn EventLoopWaker>,
    embedder_tx: crossbeam_channel::Sender<()>,
    embedder_rx: crossbeam_channel::Receiver<()>,
}

impl Handler {
    pub fn new(
        event_loop_waker: Box<dyn EventLoopWaker>,
        embedder_tx: crossbeam_channel::Sender<()>,
        embedder_rx: crossbeam_channel::Receiver<()>,
    ) -> Self {
        Self {
            event_loop_waker,
            embedder_tx,
            embedder_rx,
        }
    }
}

impl WebDriverBidiHandler for Handler {
    fn process(&self, session: &Option<Session>, command: &CommandMessage) {
        todo!()
    }

    fn recv(&self) -> impl Future<Output = (Option<Session>, Message)> {
        async { todo!() }
    }
}

//! Decouple handling logic using trait.
//!
//! Also provide a default implementation.

mod browser;
mod browsing_context;
mod common;
mod emulation;
mod input;
mod network;
mod script;
mod session;
mod storage;
mod util;
mod web_extension;

use std::rc::Rc;

use crossbeam_channel::Sender;
use embedder_traits::{
    EventLoopWaker,
    webdriver_bidi::{RequestId, WebDriverBidiToEmbedderMsg},
};
use futures_util::Stream;
use rustenium_bidi_definitions::{
    Command,
    base::{CommandMessage, EventResponse},
};

use crate::{dispatcher::DispatchMessage, error::WebDriverBidiError, model::ResultData};

pub trait WebDriverBidiHandler: Sized {
    fn to_sessioned(&self) -> Option<Self>;

    /// Start processing of a command.
    fn handle(
        &self,
        request_id: RequestId,
        command: CommandMessage,
        tx: Sender<DispatchMessage>,
    ) -> impl Future<Output = Result<ResultData, WebDriverBidiError>>;

    fn event_stream(&self) -> impl Stream<Item = Result<EventResponse, WebDriverBidiError>>;
}

struct HandlerInner {
    event_loop_waker: Box<dyn EventLoopWaker>,
    embedder_sender: Sender<WebDriverBidiToEmbedderMsg>,
    is_static: bool,
}

pub struct Handler(Rc<HandlerInner>);

/// Util methods.
impl Handler {
    pub fn new(
        event_loop_waker: Box<dyn EventLoopWaker>,
        embedder_sender: crossbeam_channel::Sender<WebDriverBidiToEmbedderMsg>,
    ) -> Self {
        Self(Rc::new(HandlerInner {
            event_loop_waker,
            embedder_sender,
            is_static: true,
        }))
    }
}

impl WebDriverBidiHandler for Handler {
    fn to_sessioned(&self) -> Option<Self> {
        if !self.0.is_static {
            return None;
        }

        Some(Self(Rc::new(HandlerInner {
            event_loop_waker: self.0.event_loop_waker.clone_box(),
            embedder_sender: self.0.embedder_sender.clone(),
            is_static: false,
        })))
    }

    async fn handle(
        &self,
        request_id: RequestId,
        command: CommandMessage,
        dispatch_tx: Sender<DispatchMessage>,
    ) -> Result<ResultData, WebDriverBidiError> {
        match command.command_data {
            Command::Browser(cmd) => self.handle_browser(cmd).await.map(ResultData::Browser),
            Command::BrowsingContext(cmd) => self
                .handle_browsing_context(cmd)
                .await
                .map(ResultData::BrowsingContext),
            Command::Emulation(cmd) => self.handle_emulation(cmd).await.map(ResultData::Emulation),
            Command::Input(cmd) => self.handle_input(cmd).await.map(ResultData::Input),
            Command::Network(cmd) => self.handle_network(cmd).await.map(ResultData::Network),
            Command::Script(cmd) => self.handle_script(cmd).await.map(ResultData::Script),
            Command::Session(cmd) => self.handle_session(cmd).await.map(ResultData::Session),
            Command::Storage(cmd) => self.handle_storage(cmd).await.map(ResultData::Storage),
            Command::WebExtension(cmd) => self
                .handle_web_extension(cmd)
                .await
                .map(ResultData::WebExtension),
        }
    }

    fn event_stream(&self) -> impl Stream<Item = Result<EventResponse, WebDriverBidiError>> {
        // TODO: actual stream
        futures_util::stream::empty()
    }
}

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
use servo_webdriver::bidi::{Command, CommandData, Event, ResultData};

use crate::{dispatcher::DispatchMessage, error::WebDriverBidiError};

pub trait WebDriverBidiHandler: Sized {
    fn to_sessioned(&self) -> Option<Self>;

    /// Start processing of a command.
    fn handle(
        &self,
        request_id: RequestId,
        command: Command,
        tx: Sender<DispatchMessage>,
    ) -> impl Future<Output = Result<ResultData, WebDriverBidiError>>;

    fn event_stream(&self) -> impl Stream<Item = Result<Event, WebDriverBidiError>>;
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
        command: Command,
        dispatch_tx: Sender<DispatchMessage>,
    ) -> Result<ResultData, WebDriverBidiError> {
        match command.command_data {
            CommandData::BrowserCommand(cmd) => self
                .handle_browser(cmd)
                .await
                .map(ResultData::BrowserResult),
            CommandData::BrowsingContextCommand(cmd) => self
                .handle_browsing_context(cmd)
                .await
                .map(ResultData::BrowsingContextResult),
            CommandData::EmulationCommand(cmd) => self
                .handle_emulation(cmd)
                .await
                .map(ResultData::EmulationResult),
            CommandData::InputCommand(cmd) => {
                self.handle_input(cmd).await.map(ResultData::InputResult)
            },
            CommandData::NetworkCommand(cmd) => self
                .handle_network(cmd)
                .await
                .map(ResultData::NetworkResult),
            CommandData::ScriptCommand(cmd) => self
                .handle_script(cmd)
                .await
                .map(|r| ResultData::ScriptResult(Box::new(r))),
            CommandData::SessionCommand(cmd) => self
                .handle_session(cmd)
                .await
                .map(ResultData::SessionResult),
            CommandData::StorageCommand(cmd) => self
                .handle_storage(cmd)
                .await
                .map(|r| ResultData::StorageResult(Box::new(r))),
            CommandData::WebExtensionCommand(cmd) => self
                .handle_web_extension(cmd)
                .await
                .map(ResultData::WebExtensionResult),
        }
    }

    fn event_stream(&self) -> impl Stream<Item = Result<Event, WebDriverBidiError>> {
        // TODO: actual stream
        futures_util::stream::empty()
    }
}

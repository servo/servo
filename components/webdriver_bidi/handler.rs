//! Decouple handling logic using trait.
//!
//! Also provide a default implementation.

use embedder_traits::{EventLoopWaker, webdriver_bidi::WebDriverBidiCommandMsg};
use rustenium_bidi_definitions::{Command, base::CommandMessage, browsing_context};
use servo_base::id::WebViewId;

use crate::{
    error::{WebDriverBidiError, WebDriverBidiResult},
    model::Message,
    transport::Session,
};

// TODO: should handler be per session?
pub trait WebDriverBidiHandler: Send {
    /// Start processing of a command.
    fn process(&self, session: &Option<Session>, command: &CommandMessage);

    fn try_recv(&self) -> WebDriverBidiResult<(Option<Session>, Message)>;

    // TODO: do we need
    // post update after receiving message
    // fn update(&mut self, message: &Message);
}

// TODO: bidi session has different meaning to classic session.
// and webviewid is not bound to session.

pub struct Handler {
    event_loop_waker: Box<dyn EventLoopWaker>,
    embedder_sender: crossbeam_channel::Sender<WebDriverBidiCommandMsg>,
    webview_id: Option<WebViewId>,
}

impl Handler {
    pub fn new(
        event_loop_waker: Box<dyn EventLoopWaker>,
        embedder_sender: crossbeam_channel::Sender<WebDriverBidiCommandMsg>,
    ) -> Self {
        Self {
            event_loop_waker,
            embedder_sender,
            webview_id: None,
        }
    }

    // TODO: bidi use BrowsingContextId not WebId
    pub fn webview_id(&self) -> WebDriverBidiResult<&WebViewId> {
        self.webview_id
            .as_ref()
            .ok_or_else(|| WebDriverBidiError::unknown("No webview available"))
    }

    fn send_message_to_embedder(&self, msg: WebDriverBidiCommandMsg) -> WebDriverBidiResult<()> {
        self.embedder_sender.send(msg)?;
        self.event_loop_waker.wake();
        Ok(())
    }

    fn handle_traverse_history(&self, delta: i64) -> WebDriverBidiResult<()> {
        let webview_id = self.webview_id()?;
        // TODO: verify context open? is this in bidi spec
        self.send_message_to_embedder(WebDriverBidiCommandMsg::TraverseHistory(
            *webview_id,
            delta,
        ))?;
        Ok(())
    }
}

impl WebDriverBidiHandler for Handler {
    fn process(&self, session: &Option<Session>, command: &CommandMessage) {
        match &command.command_data {
            Command::Browser(browser_command) => todo!(),
            Command::BrowsingContext(browsing_context_command) => match browsing_context_command {
                browsing_context::commands::BrowsingContextCommand::Activate(activate) => todo!(),
                browsing_context::commands::BrowsingContextCommand::CaptureScreenshot(
                    capture_screenshot,
                ) => todo!(),
                browsing_context::commands::BrowsingContextCommand::Close(close) => todo!(),
                browsing_context::commands::BrowsingContextCommand::Create(create) => todo!(),
                browsing_context::commands::BrowsingContextCommand::GetTree(get_tree) => todo!(),
                browsing_context::commands::BrowsingContextCommand::HandleUserPrompt(
                    handle_user_prompt,
                ) => todo!(),
                browsing_context::commands::BrowsingContextCommand::LocateNodes(locate_nodes) => {
                    todo!()
                },
                browsing_context::commands::BrowsingContextCommand::Navigate(navigate) => todo!(),
                browsing_context::commands::BrowsingContextCommand::Print(print) => todo!(),
                browsing_context::commands::BrowsingContextCommand::Reload(reload) => todo!(),
                browsing_context::commands::BrowsingContextCommand::SetViewport(set_viewport) => {
                    todo!()
                },
                browsing_context::commands::BrowsingContextCommand::TraverseHistory(
                    traverse_history,
                ) => {
                    self.handle_traverse_history(traverse_history.params.delta);
                },
            },
            Command::Emulation(emulation_command) => todo!(),
            Command::Input(input_command) => todo!(),
            Command::Network(network_command) => todo!(),
            Command::Script(script_command) => todo!(),
            Command::Session(session_command) => todo!(),
            Command::Storage(storage_command) => todo!(),
            Command::WebExtension(web_extension_command) => todo!(),
        }
    }

    fn try_recv(&self) -> WebDriverBidiResult<(Option<Session>, Message)> {
        todo!()
    }
}

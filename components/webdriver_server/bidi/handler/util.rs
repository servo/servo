use embedder_traits::webdriver_bidi::WebDriverBidiToEmbedderMessage;

use crate::bidi::{error::WebDriverBidiError, handler::Handler};

impl Handler {
    pub(super) fn send_message_to_embedder(
        &self,
        msg: WebDriverBidiToEmbedderMessage,
    ) -> Result<(), WebDriverBidiError> {
        self.0.embedder_sender.send(msg)?;
        self.0.event_loop_waker.wake();
        Ok(())
    }
}

use embedder_traits::webdriver_bidi::WebDriverBidiToEmbedderMsg;

use crate::bidi::{error::WebDriverBidiError, handler::Handler};

impl Handler {
    pub(super) fn send_message_to_embedder(
        &self,
        msg: WebDriverBidiToEmbedderMsg,
    ) -> Result<(), WebDriverBidiError> {
        self.0.embedder_sender.send(msg)?;
        self.0.event_loop_waker.wake();
        Ok(())
    }
}

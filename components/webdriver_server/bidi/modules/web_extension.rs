use std::rc::Rc;

use webdriver_traits::bidi::{WebExtensionCommand, WebExtensionResult};

use crate::bidi::{
    error::{BidiError, BidiResult},
    remote_end::RemoteEnd,
};

impl RemoteEnd {
    pub(crate) async fn handle_web_extension_command(
        self: Rc<Self>,
        command: WebExtensionCommand,
    ) -> BidiResult<WebExtensionResult> {
        // TODO:
        Err(BidiError::default())
    }
}

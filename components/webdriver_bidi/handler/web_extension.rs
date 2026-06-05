use rustenium_bidi_definitions::web_extension::commands::WebExtensionCommand;

use crate::{error::WebDriverBidiError, handler::Handler, model::WebExtensionResult};

impl Handler {
    pub(super) async fn handle_web_extension(
        &self,
        cmd: &WebExtensionCommand,
    ) -> Result<WebExtensionResult, WebDriverBidiError> {
        match cmd {
            WebExtensionCommand::Install(_) => self.handle_web_extension_install().await,
            WebExtensionCommand::Uninstall(_) => self.handle_web_extension_uninstall().await,
        }
    }

    async fn handle_web_extension_install(&self) -> Result<WebExtensionResult, WebDriverBidiError> {
        Err(WebDriverBidiError::unknown(
            "Web Extension is not implemented yet",
        ))
    }

    async fn handle_web_extension_uninstall(
        &self,
    ) -> Result<WebExtensionResult, WebDriverBidiError> {
        Err(WebDriverBidiError::unknown(
            "Web Extension is not implemented yet",
        ))
    }
}

use std::rc::Rc;

use webdriver_traits::bidi::{
    ErrorCode, WebExtensionCommand, WebExtensionResult,
    web_extension::{InstallParameters, InstallResult, UninstallParameters, UninstallResult},
};

use crate::bidi::{error::BidiResult, remote_end::RemoteEnd};

impl RemoteEnd {
    pub(crate) async fn handle_web_extension_command(
        self: Rc<Self>,
        command: WebExtensionCommand,
    ) -> BidiResult<WebExtensionResult> {
        match command {
            WebExtensionCommand::Install(cmd) => self
                .handle_web_extension_install(cmd.params)
                .await
                .map(WebExtensionResult::InstallResult),
            WebExtensionCommand::Uninstall(cmd) => self
                .handle_web_extension_uninstall(cmd.params)
                .await
                .map(WebExtensionResult::UninstallResult),
        }
    }

    /// <https://www.w3.org/TR/webdriver-bidi/#command-webExtension-install>
    async fn handle_web_extension_install(
        self: Rc<Self>,
        _: InstallParameters,
    ) -> BidiResult<InstallResult> {
        // TODO: blocked by web extension not implemented
        Err(ErrorCode::UnsupportedOperation.into())
    }

    /// <https://www.w3.org/TR/webdriver-bidi/#command-webExtension-uninstall>
    async fn handle_web_extension_uninstall(
        self: Rc<Self>,
        _: UninstallParameters,
    ) -> BidiResult<UninstallResult> {
        // TODO: blocked by web extension not implemented
        Err(ErrorCode::UnsupportedOperation.into())
    }
}

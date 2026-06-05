use rustenium_bidi_definitions::input::commands::InputCommand;

use crate::{error::WebDriverBidiError, handler::Handler, model::InputResult};

impl Handler {
    pub(super) async fn handle_input(
        &self,
        cmd: InputCommand,
    ) -> Result<InputResult, WebDriverBidiError> {
        match cmd {
            InputCommand::PerformActions(perform_actions) => {
                self.handle_input_perform_actions().await
            },
            InputCommand::ReleaseActions(release_actions) => {
                self.handle_input_release_actions().await
            },
            InputCommand::SetFiles(set_files) => self.handle_input_set_files().await,
        }
    }

    async fn handle_input_perform_actions(&self) -> Result<InputResult, WebDriverBidiError> {
        todo!()
    }
    async fn handle_input_release_actions(&self) -> Result<InputResult, WebDriverBidiError> {
        todo!()
    }
    async fn handle_input_set_files(&self) -> Result<InputResult, WebDriverBidiError> {
        todo!()
    }
}

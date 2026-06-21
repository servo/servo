use std::rc::Rc;

use webdriver_traits::bidi::{
    InputCommand, InputResult,
    input::{
        PerformActionsParameters, PerformActionsResult, ReleaseActionsParameters,
        ReleaseActionsResult, SetFilesParameters, SetFilesResult,
    },
};

use crate::bidi::{error::BidiResult, remote_end::RemoteEnd};

impl RemoteEnd {
    pub(crate) async fn handle_input_command(
        self: Rc<Self>,
        command: InputCommand,
    ) -> BidiResult<InputResult> {
        match command {
            InputCommand::PerformActions(cmd) => self
                .handle_input_perform_actions(cmd.params)
                .await
                .map(InputResult::PerformActionsResult),
            InputCommand::ReleaseActions(cmd) => self
                .handle_input_release_actions(cmd.params)
                .await
                .map(InputResult::ReleaseActionsResult),
            InputCommand::SetFiles(cmd) => self
                .handle_input_set_files(cmd.params)
                .await
                .map(InputResult::SetFilesResult),
        }
    }

    /// <https://www.w3.org/TR/webdriver-bidi/#command-input-performActions>
    async fn handle_input_perform_actions(
        self: Rc<Self>,
        _: PerformActionsParameters,
    ) -> BidiResult<PerformActionsResult> {
        todo!()
    }

    /// <https://www.w3.org/TR/webdriver-bidi/#command-input-releaseActions>
    async fn handle_input_release_actions(
        self: Rc<Self>,
        _: ReleaseActionsParameters,
    ) -> BidiResult<ReleaseActionsResult> {
        todo!()
    }

    /// <https://www.w3.org/TR/webdriver-bidi/#command-input-setFiles>
    async fn handle_input_set_files(
        self: Rc<Self>,
        _: SetFilesParameters,
    ) -> BidiResult<SetFilesResult> {
        todo!()
    }

    /// Remote end event trigger for `input.fileDialogOpened`.
    pub(crate) async fn trigger_input_file_dialog_opened(self: Rc<Self>) {
        todo!()
    }
}

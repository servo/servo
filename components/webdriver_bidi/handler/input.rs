use servo_webdriver::bidi::{
    InputCommand, InputResult,
    input::{
        PerformActionsParameters, PerformActionsResult, ReleaseActionsParameters,
        ReleaseActionsResult, SetFilesParameters, SetFilesResult,
    },
};

use crate::{error::WebDriverBidiError, handler::Handler};

impl Handler {
    pub(super) async fn handle_input(
        &self,
        cmd: InputCommand,
    ) -> Result<InputResult, WebDriverBidiError> {
        match cmd {
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
        &self,
        command_parameters: PerformActionsParameters,
    ) -> Result<PerformActionsResult, WebDriverBidiError> {
        // 1. Let `navigable id` be the value of the `context` field of `command parameters`.

        // 2. Let `navigable` be the result of [trying] to [get a navigable] with `navigable id`.

        // 3. Let `input state` be [get the input state] with `session` and `navigable`’s [top-level traversable].

        // 4. Let `actions options` be a new [actions options] with the [is element origin] steps set to [is
        // input.ElementOrigin], and the [get element origin] steps set to the result of [get Element from
        // input.ElementOrigin] steps given `session`.

        // 5. Let `actions by tick` be the result of trying to [extract an action sequence] with `input state`, `command
        // parameters`, and `actions options`.
        //
        // 6. [Try] to [dispatch actions] with `input state`, `actions by tick`, `navigable`, and `actions options`.

        // 7. Return [success] with data null.
        Ok(PerformActionsResult {
            extensible: Default::default(),
        })
    }

    /// <https://www.w3.org/TR/webdriver-bidi/#command-input-releaseActions>
    async fn handle_input_release_actions(
        &self,
        command_parameters: ReleaseActionsParameters,
    ) -> Result<ReleaseActionsResult, WebDriverBidiError> {
        // 1. Let `navigable id` be the value of the context field of `command parameters`.

        // 2. Let `navigable` be the result of [trying] to [get a navigable] with `navigable id`.

        // 3. Let `top-level traversable` be `navigable`’s [top-level traversable].

        // 4. Let `input state` be [get the input state] with `session` and `top-level traversable`.

        // 5. Let `actions options` be a new [actions options] with the [is element origin] steps set to [is
        // input.ElementOrigin], and the [get element origin] steps set to [get Element from input.ElementOrigin
        // steps] given `session`.

        // 6. Let `undo actions` be `input state`’s [input cancel list] in reverse order.

        // 7. [Try] to `dispatch tick actions` with `undo actions`, 0, `navigable`, and `actions options`.

        // 8. [Reset the input state] with `session` and `top-level traversable`.

        // 9. Return [success] with data null.
        Ok(ReleaseActionsResult {
            extensible: Default::default(),
        })
    }

    /// <https://www.w3.org/TR/webdriver-bidi/#command-input-setFiles>
    async fn handle_input_set_files(
        &self,
        command_parameters: SetFilesParameters,
    ) -> Result<SetFilesResult, WebDriverBidiError> {
        // 1. Let `navigable id` be the value of the context field of `command parameters`.

        // 2. Let `navigable` be the result of [trying] to [get a navigable] with `navigable id`.

        // 3. Let `document` be `navigable`’s [active document].

        // 4. Let `environment settings` be the [environment settings object] whose [relevant global object]’s
        // [associated `Document`] is `document`.

        // 5. Let `realm` be `environment settings`’s [realm execution context]’s Realm component.

        // 6. Let `element` be the result of [trying] to [deserialize remote reference] with `command
        // parameters["element"]`, `realm`, and `session`.

        // 7. If `element` doesn’t implement [`Element`], return [error] with [error code] [no such element].

        // 8. If `element` doesn’t implement [`HTMLInputElement`], `element`’s [type] is not in the [File Upload state], or
        // `element` is [disabled], return [error] with [error code] [unable to set file input].

        // 9. If the [size] of `files` is greater than 1 and `element`’s [multiple] attribute is not set, return [error] with [error
        // code] [unable to set file input].

        // 10. Let `files` be the value of the `command parameters["files"]` field.

        // 11. Let `selected files` be `element`’s [selected files].

        // 12. If the [size] of the [intersection] of `files` and `selected files` is equal to the [size] of `selected files` and equal to
        // the [size] of `files`, [queue an element task] on the [user interaction task source] given `element` to fire an
        // event named `cancel` at `element`, with the `bubbles` attribute initialized to true.

        // 13. Otherwise, [update the file selection] for `element` with `files` as the user’s selection.

        // 14. If, for any reason, the remote end is unable to set the [selected files] of `element` to the files with paths
        // given in `files`, return error with [error code] [unsupported operation].

        // 15. Return [success] with data null.
        Ok(SetFilesResult {
            extensible: Default::default(),
        })
    }
}

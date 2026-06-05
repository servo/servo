use std::collections::HashSet;

use rustenium_bidi_definitions::browser::{
    self, commands::BrowserCommand, results::GetClientWindowsResult, types::ClientWindowInfo,
};

use crate::{error::WebDriverBidiError, handler::Handler, model::BrowserResult};

impl Handler {
    pub(super) async fn handle_browser(
        &self,
        cmd: &BrowserCommand,
    ) -> Result<BrowserResult, WebDriverBidiError> {
        match cmd {
            BrowserCommand::Close(cmd) => self.handle_browser_close(&cmd.params).await,
            BrowserCommand::CreateUserContext(cmd) => {
                self.handle_browser_create_user_context(&cmd.params).await
            },
            BrowserCommand::GetClientWindows(_cmd) => {
                self.handle_browser_get_client_windows().await
            },
            BrowserCommand::GetUserContexts(_) => self.handle_browser_get_user_contexts().await,
            BrowserCommand::RemoveUserContext(cmd) => {
                self.handle_browser_remove_user_context(&cmd.params).await
            },
            BrowserCommand::SetClientWindowState(cmd) => {
                self.handle_browser_set_client_window_state(&cmd.params)
                    .await
            },
            BrowserCommand::SetDownloadBehavior(cmd) => {
                self.handle_browser_set_download_behavior(&cmd.params).await
            },
        }
    }

    async fn handle_browser_close(
        &self,
        _command_parameters: &browser::commands::CloseParams,
    ) -> Result<BrowserResult, WebDriverBidiError> {
        // 1. `End the session` with `session`.

        // 2. If active sessions is not empty an implementation may return error with error code unable to close browser, and then run the following steps in parallel:
        // 2.1. Wait until the Send a WebSocket message steps have been called with the response to this command.
        // 2.2. [Cleanup the session] with `session`.

        // 3. For each `active session` in [active sessions]:

        // 4. Return [success] with data null, and run the following steps [in parallel].
        todo!()
    }

    async fn handle_browser_create_user_context(
        &self,
        _command_parameters: &browser::commands::CreateUserContextParams,
    ) -> Result<BrowserResult, WebDriverBidiError> {
        // 1. Let user context be a new user context.

        // 2. If command parameters contain "acceptInsecureCerts":

        // 3. If command parameters contains "unhandledPromptBehavior", set unhandled prompt behavior overrides map[user context] to command parameters["unhandledPromptBehavior"].

        // 4. If command parameters contains "proxy":

        // 5. Append user context to the set of user contexts.

        // 6. Let user context info be a map matching the browser.UserContextInfo production with the userContext field set to user context’s user context id.

        // 7. Return success with data user context info.

        Err(WebDriverBidiError::unknown(
            "user context is not implemented yet",
        ))
    }

    async fn handle_browser_get_client_windows(&self) -> Result<BrowserResult, WebDriverBidiError> {
        // 1. Let client window ids be an empty set.
        let client_window_ids = HashSet::<()>::new();

        // 2. Let client windows be an empty list.
        let mut client_windows = Vec::<ClientWindowInfo>::new();

        // 3. For each top-level traversable traversable:
        // 3.1. Let client window be traversable’s associated client window
        // 3.2. Let client window id be the client window id for client window.
        // 3.3. If client window ids contains client window id, continue.
        // 3.4. Append client window id to client window ids.
        // 3.5. Let client window info be get the client window info with client window.
        // 3.6. Append client window info to `client windows`.

        // 4. Let `result` be a [map] matching the `browser.GetClientWindowsResult` production with the
        // `clientWindows` field set to `client windows`.
        let result = GetClientWindowsResult { client_windows };

        // 5. Return success with data result.
        Ok(BrowserResult::GetClientWindows(result))
    }

    async fn handle_browser_get_user_contexts(&self) -> Result<BrowserResult, WebDriverBidiError> {
        // 1. Let user contexts be an empty list.
        // 2. For each user context in the set of user contexts:
        // 2.1. Let user context info be a map matching the browser.UserContextInfo production with the userContext field set to user context’s user context id.
        // 2.2. Append user context info to user contexts.
        // 3. Let result be a map matching the browser.GetUserContextsResult production with the userContexts field set to user contexts.
        // 4. Return success with data result.
        Err(WebDriverBidiError::unknown(
            "user context is not implemented yet",
        ))
    }

    async fn handle_browser_remove_user_context(
        &self,
        command_parameters: &browser::commands::RemoveUserContextParams,
    ) -> Result<BrowserResult, WebDriverBidiError> {
        // 1. Let user context id be command parameters["userContext"].
        // 2. If user context id is "default", return error with error code invalid argument.
        // 3. Set user context to get user context with user context id.
        // 4. If user context is null, return error with error code no such user context.
        // 5. For each top-level traversable navigable:
        // 5.1. If navigable’s associated user context is user context:
        // 5.1.1. Close navigable without prompting to unload.
        // 6. Remove user context for the set of user contexts.
        // 7. Return [success] with data null.
        Err(WebDriverBidiError::unknown(
            "user context is not implemented yet",
        ))
    }

    async fn handle_browser_set_client_window_state(
        &self,
        command_parameters: &browser::commands::SetClientWindowStateParams,
    ) -> Result<BrowserResult, WebDriverBidiError> {
        // The remote end steps with session and command parameters are:
        //
        // If the implementation does not support setting the client window state at all, then return error with error code unsupported operation.
        //
        // If there is a client window with client window id command parameters["clientWindow"], let client window be that client window. Otherwise return error with error code no such client window.
        //
        // Try to set the client window state with client window and command parameters["state"].
        //
        // If command parameters["state"] is "normal":
        //
        // If command parameters contains "x" and the implementation supports positioning client windows, set the x-coordinate of client window to a value that is as close as possible command parameters["x"].
        //
        // If command parameters contains "y" and the implementation supports positioning client windows, set the y-coordinate of client window to a value that is as close as possible command parameters["y"].
        //
        // If command parameters contains "width" and the implementation supports resizing client windows, set the width of client window to a value that is as close as possible command parameters["width"].
        //
        // If command parameters contains "width" and the implementation supports resizing client windows, set the width of client window to a value that is as close as possible command parameters["width"].
        //
        // Let client window info be get the client window info with client window.
        //
        // 6. Return [success] with data `client window info`.
        todo!()
    }

    async fn handle_browser_set_download_behavior(
        &self,
        command_parameters: &browser::commands::SetDownloadBehaviorParams,
    ) -> Result<BrowserResult, WebDriverBidiError> {
        // If command parameters["downloadBehavior"] is null, let download behavior be null.
        //
        // Otherwise:
        //
        // If command parameters["downloadBehavior"]["type"] is "allowed", let allowed be true, otherwise let allowed be false.
        //
        // If command parameters["downloadBehavior"] contains "destinationFolder", let destinationFolder be command parameters["downloadBehavior"]["destinationFolder"], otherwise let destinationFolder be null.
        //
        // Let download behavior be a download behavior struct with allowed set to allowed and destinationFolder set to destinationFolder.
        //
        // If the implementation does not support required download behavior, then return error with error code unsupported operation.
        //
        // If the userContexts field of command parameters is present:
        //
        // Let user contexts be the result of trying to get valid user contexts with command parameters["userContexts"].
        //
        // For each user context of user contexts:
        //
        // If download behavior is null, remove user context from download behavior’s user context download behavior.
        //
        // Otherwise, set download behavior’s user context download behavior[user context] to download behavior.
        //
        // Otherwise, set download behavior’s default download behavior to download behavior.
        //
        // Return success with data null.
        todo!()
    }
}

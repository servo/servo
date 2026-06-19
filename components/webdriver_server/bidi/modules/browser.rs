use std::rc::Rc;

use webdriver_traits::bidi::{
    BrowserCommand, BrowserResult, EmptyParams, EmptyResult, ErrorCode,
    browser::{
        CloseResult, CreateUserContextParameters, CreateUserContextResult, GetClientWindowsResult,
        GetUserContextsResult, RemoveUserContextParameters, RemoveUserContextResult,
        SetClientWindowStateParameters, SetClientWindowStateResult, SetDownloadBehaviorParameters,
        SetDownloadBehaviorResult,
    },
};

use crate::bidi::{
    error::{BidiError, BidiResult},
    remote_end::RemoteEnd,
    session::common::SessionId,
    wait_queue::ResumeEvent,
};

impl RemoteEnd {
    pub(crate) async fn handle_browser_command(
        self: Rc<Self>,
        session_id: SessionId,
        command: BrowserCommand,
    ) -> BidiResult<BrowserResult> {
        match command {
            BrowserCommand::Close(cmd) => self
                .handle_browser_close(session_id, cmd.params)
                .await
                .map(BrowserResult::CloseResult),
            BrowserCommand::CreateUserContext(cmd) => self
                .handle_browser_create_user_context(session_id, cmd.params)
                .await
                .map(BrowserResult::CreateUserContextResult),
            BrowserCommand::GetClientWindows(cmd) => self
                .handle_browser_get_client_windows(session_id, cmd.params)
                .await
                .map(BrowserResult::GetClientWindowsResult),
            BrowserCommand::GetUserContexts(_) => self
                .handle_browser_get_user_contexts()
                .await
                .map(BrowserResult::GetUserContextsResult),
            BrowserCommand::RemoveUserContext(cmd) => self
                .handle_browser_remove_user_context(cmd.params)
                .await
                .map(BrowserResult::RemoveUserContextResult),
            BrowserCommand::SetClientWindowState(cmd) => self
                .handle_browser_set_client_window_state(session_id, cmd.params)
                .await
                .map(BrowserResult::SetClientWindowStateResult),
            BrowserCommand::SetDownloadBehavior(cmd) => self
                .handle_browser_set_download_behavior(session_id, cmd.params)
                .await
                .map(BrowserResult::CloseResult),
        }
    }

    /// <https://www.w3.org/TR/webdriver-bidi/#command-browser-close>
    async fn handle_browser_close(
        self: Rc<Self>,
        session_id: SessionId,
        _: EmptyParams,
    ) -> BidiResult<CloseResult> {
        // Step 1. end this session.
        self.end_the_session(session_id);
        if !self.active_sessions.borrow().is_empty() {
            // Step 2.1.
            self.wait_queue.awaits(
                &[ResumeEvent::SessionResponded(session_id)],
                // Step 2.2.
                self.clone().cleanup_the_session(session_id),
            );
            // Step 2.
            return Err(BidiError {
                code: ErrorCode::UnableToCloseBrowser,
                message: "active sessions is not empty".into(),
                ..Default::default()
            });
        }
        // Step 3. end each active session
        // make a copy to avoid breaking the RefCell
        let active_sessions = self
            .active_sessions
            .borrow()
            .keys()
            .copied()
            .collect::<Vec<_>>();
        for active_session in active_sessions {
            self.end_the_session(active_session);
            self.clone().cleanup_the_session(session_id).await;
        }
        // Step 4.1.
        let this = self.clone();
        self.wait_queue
            .awaits(&[ResumeEvent::SessionResponded(session_id)], async {
                // Step 6.2. cleanup this session
                this.clone().cleanup_the_session(session_id).await;
                // Step 6.3. close traversables
                this.close_all_traversables(false).await;
                // Step 6.4. send close to embedder
                this.close_browser();
            });
        // Step 4.
        Ok(EmptyResult::default())
    }

    /// <https://www.w3.org/TR/webdriver-bidi/#command-browser-createUserContext>
    async fn handle_browser_create_user_context(
        self: Rc<Self>,
        _: SessionId,
        _: CreateUserContextParameters,
    ) -> BidiResult<CreateUserContextResult> {
        Err(BidiError {
            code: ErrorCode::UnknownError,
            message: "user context is not implemented yet".into(),
            ..Default::default()
        })
    }

    /// <https://www.w3.org/TR/webdriver-bidi/#command-browser-getClientWindows>
    async fn handle_browser_get_client_windows(
        self: Rc<Self>,
        session_id: SessionId,
        _: EmptyParams,
    ) -> BidiResult<GetClientWindowsResult> {
        todo!()
    }

    /// <https://www.w3.org/TR/webdriver-bidi/#command-browser-getUserContexts>
    async fn handle_browser_get_user_contexts(self: Rc<Self>) -> BidiResult<GetUserContextsResult> {
        Err(BidiError {
            code: ErrorCode::UnknownError,
            message: "user context is not implemented yet".into(),
            ..Default::default()
        })
    }

    /// <https://www.w3.org/TR/webdriver-bidi/#command-browser-removeUserContext>
    async fn handle_browser_remove_user_context(
        self: Rc<Self>,
        _: RemoveUserContextParameters,
    ) -> BidiResult<RemoveUserContextResult> {
        Err(BidiError {
            code: ErrorCode::UnknownError,
            message: "user context is not implemented yet".into(),
            ..Default::default()
        })
    }

    /// <https://www.w3.org/TR/webdriver-bidi/#command-browser-setClientWindowState>
    async fn handle_browser_set_client_window_state(
        self: Rc<Self>,
        session_id: SessionId,
        _: SetClientWindowStateParameters,
    ) -> BidiResult<SetClientWindowStateResult> {
        Err(ErrorCode::UnknownError.into())
    }

    /// <https://www.w3.org/TR/webdriver-bidi/#command-browser-setDownloadBehavior>
    async fn handle_browser_set_download_behavior(
        self: Rc<Self>,
        session_id: SessionId,
        _: SetDownloadBehaviorParameters,
    ) -> BidiResult<SetDownloadBehaviorResult> {
        Err(ErrorCode::UnknownError.into())
    }
}

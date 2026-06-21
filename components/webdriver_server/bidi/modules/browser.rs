use std::rc::Rc;

use log::warn;
use tokio::{sync::oneshot::Receiver, task};
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
    session::SessionId,
};

impl RemoteEnd {
    pub(crate) async fn handle_browser_command(
        self: Rc<Self>,
        session_id: SessionId,
        command: BrowserCommand,
        msg_sent: Receiver<()>,
    ) -> BidiResult<BrowserResult> {
        match command {
            BrowserCommand::Close(cmd) => self
                .handle_browser_close(session_id, cmd.params, msg_sent)
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
        msg_sent: Receiver<()>,
    ) -> BidiResult<CloseResult> {
        // Step 1. end this session.
        self.end_the_session(session_id);
        if !self.active_sessions.borrow().is_empty() {
            // Step 2.1.
            task::spawn_local(async move {
                if let Err(err) = msg_sent.await {
                    warn!("Waiting sending a WebSocket message failed ({err:?})");
                }
                self.clone().cleanup_the_session(session_id)
            });
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
        task::spawn_local(async move {
            if let Err(err) = msg_sent.await {
                warn!("Waiting sending a WebSocket message failed ({err:?})");
            }
            self.clone().cleanup_the_session(session_id).await;
            self.clone().close_all_traversables(false).await;
            self.close_browser();
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

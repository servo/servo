use std::{collections::HashSet, rc::Rc};

use servo_base::id::BrowsingContextId;
use webdriver_traits::bidi::{
    BrowsingContextCommand, BrowsingContextResult, ErrorCode,
    browser::CloseResult,
    browsing_context::{
        ActivateParameters, ActivateResult, CaptureScreenshotParameters, CaptureScreenshotResult,
        CloseParameters, CreateParameters, CreateResult, GetTreeParameters, GetTreeResult,
        HandleUserPromptParameters, HandleUserPromptResult, LocateNodesParameters,
        LocateNodesResult, NavigateParameters, NavigateResult, PrintParameters, PrintResult,
        ReadinessState, ReloadParameters, ReloadResult, SetBypassCspParameters, SetBypassCspResult,
        SetViewportParameters, SetViewportResult, StartScreencastParameters, StartScreencastResult,
        StopScreencastParameters, StopScreencastResult, TraverseHistoryParameters,
        TraverseHistoryResult,
    },
};

use crate::bidi::{
    error::{BidiError, BidiResult},
    remote_end::{Navigable, RemoteEnd},
    session::SessionId,
    util::new_oneshot_callback,
};

impl RemoteEnd {
    pub(crate) async fn handle_browsing_context_command(
        self: Rc<Self>,
        session_id: SessionId,
        command: BrowsingContextCommand,
    ) -> BidiResult<BrowsingContextResult> {
        match command {
            BrowsingContextCommand::Activate(cmd) => self
                .handle_browsing_context_activate(session_id, cmd.params)
                .await
                .map(BrowsingContextResult::ActivateResult),
            BrowsingContextCommand::CaptureScreenshot(cmd) => self
                .handle_browsing_context_capture_screenshot(session_id, cmd.params)
                .await
                .map(BrowsingContextResult::CaptureScreenshotResult),
            BrowsingContextCommand::Close(cmd) => self
                .handle_browsing_context_close(session_id, cmd.params)
                .await
                .map(BrowsingContextResult::CloseResult),
            BrowsingContextCommand::Create(cmd) => self
                .handle_browsing_context_create(cmd.params)
                .await
                .map(BrowsingContextResult::CreateResult),
            BrowsingContextCommand::GetTree(cmd) => self
                .handle_browsing_context_get_tree(session_id, cmd.params)
                .await
                .map(BrowsingContextResult::GetTreeResult),
            BrowsingContextCommand::HandleUserPrompt(cmd) => self
                .handle_browsing_context_handle_user_prompt(session_id, cmd.params)
                .await
                .map(BrowsingContextResult::HandleUserPromptResult),
            BrowsingContextCommand::LocateNodes(cmd) => self
                .handle_browsing_context_locate_nodes(session_id, cmd.params)
                .await
                .map(BrowsingContextResult::LocateNodesResult),
            BrowsingContextCommand::Navigate(cmd) => self
                .handle_browsing_context_navigate(session_id, cmd.params)
                .await
                .map(BrowsingContextResult::NavigateResult),
            BrowsingContextCommand::Print(cmd) => self
                .handle_browsing_context_print(cmd.params)
                .await
                .map(BrowsingContextResult::PrintResult),
            BrowsingContextCommand::Reload(cmd) => self
                .handle_browsing_context_reload(cmd.params)
                .await
                .map(BrowsingContextResult::ReloadResult),
            BrowsingContextCommand::SetBypassCsp(cmd) => self
                .handle_browsing_context_set_bypass_csp(cmd.params)
                .await
                .map(BrowsingContextResult::SetBypassCspResult),
            BrowsingContextCommand::SetViewport(cmd) => self
                .handle_browsing_context_set_viewport(cmd.params)
                .await
                .map(BrowsingContextResult::SetViewportResult),
            BrowsingContextCommand::StartScreencast(cmd) => self
                .handle_browsing_context_start_screencast(cmd.params)
                .await
                .map(BrowsingContextResult::StartScreencastResult),
            BrowsingContextCommand::StopScreencast(cmd) => self
                .handle_browsing_context_stop_screencast(cmd.params)
                .await
                .map(BrowsingContextResult::StopScreencastResult),
            BrowsingContextCommand::TraverseHistory(cmd) => self
                .handle_browsing_context_traverse_history(cmd.params)
                .await
                .map(BrowsingContextResult::TraverseHistoryResult),
        }
    }

    /// <https://www.w3.org/TR/webdriver-bidi/#command-browsingContext-activate>
    async fn handle_browsing_context_activate(
        self: Rc<Self>,
        session_id: SessionId,
        command_parameters: ActivateParameters,
    ) -> BidiResult<ActivateResult> {
        // 1.
        let navigable_id = BrowsingContextId::from_string(&command_parameters.context)
            .ok_or(ErrorCode::InvalidArgument)?;
        // 2.
        let navigable = self.get_a_navigable(navigable_id)?;
        // 3.
        if !navigable.is_top_level_traversable {
            return Err(BidiError {
                code: ErrorCode::InvalidArgument,
                message: "browsing context is not top level traversable".into(),
                ..Default::default()
            });
        }
        // 4.
        self.activate_a_navigable(navigable_id).await
    }

    /// <https://www.w3.org/TR/webdriver-bidi/#command-browsingContext-captureScreenshot>
    async fn handle_browsing_context_capture_screenshot(
        &self,
        session_id: SessionId,
        command_parameters: CaptureScreenshotParameters,
    ) -> BidiResult<CaptureScreenshotResult> {
        // TODO:
        todo!()
    }

    /// <https://www.w3.org/TR/webdriver-bidi/#command-browsingContext-close>
    async fn handle_browsing_context_close(
        self: Rc<Self>,
        session_id: SessionId,
        command_parameters: CloseParameters,
    ) -> BidiResult<CloseResult> {
        // 1.
        let navigable_id = BrowsingContextId::from_string(&command_parameters.context)
            .ok_or(ErrorCode::InvalidArgument)?;
        // 2.
        let prompt_unload = command_parameters.prompt_unload.unwrap_or(false);
        // 3.
        let navigable = self.get_a_navigable(navigable_id)?;
        // 4. SKIP: Assert
        // 5.
        if !navigable.is_top_level_traversable {
            return Err(ErrorCode::InvalidArgument.into());
        }
        // 6. & 7.
        let (callback, receiver) = new_oneshot_callback::<()>();
        // navigable.send_to_script(WebDriverToScriptMessage::CloseNavigable(
        //     prompt_unload,
        //     callback,
        // ));
        if let Err(e) = receiver.await {
            log::warn!("Receiving callback failed: {e:?}");
            return Err(ErrorCode::UnknownError.into());
        }
        // 8.
        Ok(CloseResult::default())
    }

    /// <https://www.w3.org/TR/webdriver-bidi/#command-browsingContext-create>
    async fn handle_browsing_context_create(
        self: Rc<Self>,
        command_parameters: CreateParameters,
    ) -> BidiResult<CreateResult> {
        // 1.
        let r#type = &command_parameters.r#type;
        // 2.
        let reference_navigable_id = command_parameters
            .reference_context
            .as_deref()
            .map(|s| BrowsingContextId::from_string(s).ok_or(ErrorCode::InvalidArgument))
            .transpose()?;
        // 3.
        let reference_naviable = match reference_navigable_id {
            Some(id) => Some(self.get_a_navigable(id)?),
            None => None,
        };
        // 4.
        if let Some(reference_navigable) = reference_naviable
            && !reference_navigable.is_top_level_traversable
        {
            return Err(ErrorCode::InvalidArgument.into());
        }
        // 5. SKIP: implementation-defined
        // 6-9: TODO: blocked by user context not implemented
        // 10. SKIP: implementation-defined
        // 11. TODO: setting user context
        // let traversable = self
        //     .create_a_new_top_level_traversable(None, "".to_string(), r#type.clone())
        //     .await?;
        // 12.
        // if !command_parameters.background.unwrap_or(false) {
        //     // 12.1.
        //     let activate_result = self.activate_a_navigable(traversable.1).await;
        //     // 12.2.
        //     activate_result?;
        // }
        // 13.
        let body = CreateResult {
            context: "".to_string(),
            user_context: None,
        };
        // 14.
        Ok(body)
    }

    /// <https://www.w3.org/TR/webdriver-bidi/#command-browsingContext-getTree>
    async fn handle_browsing_context_get_tree(
        self: Rc<Self>,
        session_id: SessionId,
        command_parameters: GetTreeParameters,
    ) -> BidiResult<GetTreeResult> {
        // 1.
        let root_id = &command_parameters.root;
        // 2.
        let max_depth = command_parameters.max_depth;
        // 3.
        let mut navigables = vec![];
        // 4.
        if let Some(root_id) = root_id {
            let root_id =
                BrowsingContextId::from_string(root_id).ok_or(ErrorCode::InvalidArgument)?;
            navigables.push(self.get_a_navigable(root_id)?);
        } else {
            // for navigable in self
            //     .common
            //     .remote_end_state
            //     .navigables
            //     .read()
            //     .await
            //     .values()
            // {
            //     // top-level only
            //     // TODO: better distinguishable
            //     if navigable.is_top_level_traversable() {
            //         navigables.push(navigable.clone());
            //     }
            // }
        }
        // 5.
        let mut navigable_infos = vec![];
        // 6.
        for navigable in navigables {
            // 6.1.
            // let info = navigable.get_the_navigable_info(max_depth);
            // navigable_infos.push(info);
        }
        // 7.
        let body = GetTreeResult {
            contexts: navigable_infos,
        };
        // 8.
        Ok(body)
    }

    /// <https://www.w3.org/TR/webdriver-bidi/#command-browsingContext-handleUserPrompt>
    async fn handle_browsing_context_handle_user_prompt(
        self: Rc<Self>,
        session_id: SessionId,
        command_parameters: HandleUserPromptParameters,
    ) -> BidiResult<HandleUserPromptResult> {
        // 1.
        let navigable_id = &command_parameters.context;
        let navigable_id =
            BrowsingContextId::from_string(navigable_id).ok_or(ErrorCode::InvalidArgument)?;
        // 2.
        let navigable = self.get_a_navigable(navigable_id);
        // 3.
        let accept = command_parameters.accept.unwrap_or(true);
        // 4.
        let user_text = command_parameters.user_text.clone().unwrap_or_default();
        // 5. TODO: create a webdriver to embedder message, and callback bool
        // 6.
        Ok(HandleUserPromptResult {
            extensible: Default::default(),
        })
    }

    /// <https://www.w3.org/TR/webdriver-bidi/#command-browsingContext-locateNodes>
    async fn handle_browsing_context_locate_nodes(
        self: Rc<Self>,
        session_id: SessionId,
        command_parameters: LocateNodesParameters,
    ) -> BidiResult<LocateNodesResult> {
        // TODO:
        todo!()
    }

    /// <https://www.w3.org/TR/webdriver-bidi/#command-browsingContext-navigate>
    async fn handle_browsing_context_navigate(
        self: Rc<Self>,
        session_id: SessionId,
        command_parameters: NavigateParameters,
    ) -> BidiResult<NavigateResult> {
        // 1.
        let navigable_id = BrowsingContextId::from_string(&command_parameters.context)
            .ok_or(ErrorCode::InvalidArgument)?;
        // 2.
        let navigable = self.get_a_navigable(navigable_id)?;
        // 3. SKIP: Assert
        // 4.
        let wait_condition = "committed";
        // 5.
        if let Some(wait) = &command_parameters.wait
            && !matches!(wait, ReadinessState::None)
        {
            // TODO: set wait_condition, they have different type
        }
        // 6.
        let url = &command_parameters.url;
        // 7.
        // let document = navigable.active_document;
        // // 8. TODO: we have not receive url information of a document
        // let base = "".to_string();
        // // 9. TODO: import servo-url parser
        // let url_record = false;
        // // 10.
        // if !url_record {
        //     return Err(ErrorCode::InvalidArgument.into());
        // }
        // // 11. TODO: id
        // let request_id = 0;
        // self.send_to_constellation(WebDriverToConstellationMessage::Request("".to_string()));
        // // 12.
        // self.await_a_navigation().await
        todo!()
    }

    /// <https://www.w3.org/TR/webdriver-bidi/#command-browsingContext-print>
    async fn handle_browsing_context_print(
        self: Rc<Self>,
        _: PrintParameters,
    ) -> BidiResult<PrintResult> {
        // TODO: blocked by PDF not implemented
        Err(ErrorCode::UnknownError.into())
    }

    /// <https://www.w3.org/TR/webdriver-bidi/#command-browsingContext-reload>
    async fn handle_browsing_context_reload(
        self: Rc<Self>,
        command_parameters: ReloadParameters,
    ) -> BidiResult<ReloadResult> {
        // 1.
        let navigable_id = BrowsingContextId::from_string(&command_parameters.context)
            .ok_or(ErrorCode::InvalidArgument)?;
        // 2.
        let navigable = self.get_a_navigable(navigable_id)?;
        // 3. SKIP: assert
        // 4.
        let ignore_cache = command_parameters.ignore_cache;
        // 5.
        let wait_condition = "commited";
        // 6. TODO: wait and wait condition have different type
        // 7. TODO: since active document, this should be sent to script thread to handle
        // 8.
        // 9.
        let request_id = 0;
        // TODO: this is different from classic, in classic it is top level, but in bidi it seems to be arbitary.
        // instead, we should send it to navigable (script thread).
        // TODO: should allow empty or use seperate variant?
        // self.send_to_constellation(WebDriverToConstellationMessage::Request("".to_string()));
        // 10.
        todo!()
    }

    /// <https://www.w3.org/TR/webdriver-bidi/#command-browsingContext-setBypassCSP>
    async fn handle_browsing_context_set_bypass_csp(
        self: Rc<Self>,
        command_parameters: SetBypassCspParameters,
    ) -> BidiResult<SetBypassCspResult> {
        // TODO:
        todo!()
    }

    /// <https://www.w3.org/TR/webdriver-bidi/#command-browsingContext-setViewport>
    async fn handle_browsing_context_set_viewport(
        self: Rc<Self>,
        _: SetViewportParameters,
    ) -> BidiResult<SetViewportResult> {
        // TODO: blocked by viewport not actually implemented
        Err(ErrorCode::UnknownError.into())
    }

    /// <https://www.w3.org/TR/webdriver-bidi/#command-browsingContext-startScreencast>
    async fn handle_browsing_context_start_screencast(
        self: Rc<Self>,
        _: StartScreencastParameters,
    ) -> BidiResult<StartScreencastResult> {
        // TODO: spec is actively changing, deferred
        Err(ErrorCode::UnsupportedOperation.into())
    }

    /// <https://www.w3.org/TR/webdriver-bidi/#command-browsingContext-stopScreencast>
    async fn handle_browsing_context_stop_screencast(
        self: Rc<Self>,
        _: StopScreencastParameters,
    ) -> BidiResult<StopScreencastResult> {
        // TODO: spec is actively changing, deferred
        Err(ErrorCode::UnsupportedOperation.into())
    }

    /// <https://www.w3.org/TR/webdriver-bidi/#command-browsingContext-traverseHistory>
    async fn handle_browsing_context_traverse_history(
        self: Rc<Self>,
        command_parameters: TraverseHistoryParameters,
    ) -> BidiResult<TraverseHistoryResult> {
        // 1.
        let navigable = self.get_a_navigable(
            BrowsingContextId::from_string(&command_parameters.context)
                .ok_or(ErrorCode::InvalidArgument)?,
        )?;
        // 2.
        if !navigable.is_top_level_traversable {
            return Err(ErrorCode::InvalidArgument.into());
        }
        // 3. skip asset
        // 4.
        let delta = command_parameters.delta;
        // 5. SKIP: we do not use wait queue
        // 6-9. TODO: continue in constellation thread
        // self.traverse_the_history_by_a_delta(delta, navigable.id)
        //     .await?;
        // 10. SKIP: spec is todo here
        // 11.
        let body = TraverseHistoryResult::default();
        // 12.
        Ok(body)
    }

    /// Remote end subscribe steps for `browsingContext.contextCreated`.
    pub(crate) async fn subscribe_browsing_context_context_created() {
        todo!()
    }

    /// Remote end event trigger for `browsingContext.contextCreated`.
    pub(crate) fn trigger_browsing_context_context_created(
        self: Rc<Self>,
        mut navigable: Navigable,
        opener_navigable: BrowsingContextId,
    ) {
        // Step 1. set original opener
        // TODO: the spec is bad, many irrelevant steps are mixed in,
        // "trigger" is a trigger once event happens, not a monitering thread.
        navigable.original_opener = None;
        // Step 2.
        // TODO: the implementation-specific step to disable cache behavior
        // Step 3.
        let related_navigables = HashSet::<BrowsingContextId>::from_iter([navigable.id]);
        // Step 4.
        for session in self.set_of_sessions_for_which_an_event_is_enabled(
            "browsingContext.contextCreated",
            related_navigables.into_iter(),
        ) {
            // Step 4.1.
            tokio::task::spawn_local(
                self.clone()
                    .emit_a_context_created_event(session, navigable.id),
            );
        }
        todo!()
    }

    /// Remote end event trigger for `browsingContext.ContextDestroyed`.
    pub(crate) async fn trigger_browsing_context_context_destroyed() {
        todo!()
    }

    /// Remote end event trigger for `browsingContext.navigationStarted`.
    pub(crate) async fn trigger_browsing_context_navigation_started() {
        todo!()
    }

    /// Remote end event trigger for `browsingContext.fragmentNavigated`.
    pub(crate) async fn trigger_browsing_context_fragment_navigated() {
        todo!()
    }

    /// Remote end event trigger for `browsingContext.historyUpdated`.
    pub(crate) async fn trigger_browsing_context_history_updated() {
        todo!()
    }

    /// Remote end event trigger for `browsingContext.domContentLoaded`.
    pub(crate) async fn trigger_browsing_context_dom_content_loaded() {
        todo!()
    }

    /// Remote end event trigger for `browsingContext.load`.
    pub(crate) async fn trigger_browsing_context_load() {
        todo!()
    }

    /// Remote end event trigger for `browsingContext.downloadWillBegin`.
    pub(crate) async fn trigger_browsing_context_download_will_begin() {
        todo!()
    }

    /// Remote end event trigger for `browsingContext.downloadEnd`.
    pub(crate) async fn trigger_browsing_context_download_end() {
        todo!()
    }

    /// Remote end event trigger for `browsingContext.navigationAborted`.
    pub(crate) async fn trigger_browsing_context_navigation_aborted() {
        todo!()
    }

    /// Remote end event trigger for `browsingContext.navigationFailed`.
    pub(crate) async fn trigger_browsing_context_navigation_failed() {
        todo!()
    }

    /// Remote end event trigger for `browsingContext.userPromptClosed`.
    pub(crate) async fn trigger_browsing_user_prompt_closed() {
        todo!()
    }

    /// Remote end event trigger for `browsingContext.userPromptOpened`.
    pub(crate) async fn trigger_browsing_user_prompt_opened() {
        todo!()
    }

    /// <https://www.w3.org/TR/webdriver-bidi/#recursively-emit-context-created-events>
    pub(crate) async fn recursively_emit_context_created_events(
        self: Rc<Self>,
        session_id: SessionId,
        navigable: BrowsingContextId,
    ) {
        // Step 1. emit current
        self.clone()
            .emit_a_context_created_event(session_id, navigable)
            .await;
        // Step 2. emit child
        // TODO: save child navigable id
    }

    /// <https://www.w3.org/TR/webdriver-bidi/#emit-a-context-created-event>
    pub(crate) async fn emit_a_context_created_event(
        self: Rc<Self>,
        session_id: SessionId,
        navigable: BrowsingContextId,
    ) {
        todo!()
    }
}

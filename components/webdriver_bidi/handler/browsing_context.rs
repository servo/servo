use embedder_traits::webdriver_bidi::{WaitCondition, WebDriverBidiToEmbedderMsg};
use rustenium_bidi_definitions::{
    base::ErrorCode,
    browsing_context::{self, commands::BrowsingContextCommand, types::ReadinessState},
};
use servo_base::id::{BrowsingContextId, WebViewId};

use crate::{error::WebDriverBidiError, handler::Handler, model::BrowsingContextResult};

impl Handler {
    pub(super) async fn handle_browsing_context(
        &self,
        cmd: BrowsingContextCommand,
    ) -> Result<BrowsingContextResult, WebDriverBidiError> {
        match cmd {
            BrowsingContextCommand::Activate(cmd) => {
                self.handle_browsing_context_activate(cmd.params).await
            },
            BrowsingContextCommand::CaptureScreenshot(cmd) => {
                self.handle_browsing_context_capture_screenshot(cmd.params)
                    .await
            },
            BrowsingContextCommand::Close(cmd) => {
                self.handle_browsing_context_close(cmd.params).await
            },
            BrowsingContextCommand::Create(create) => self.handle_browsing_context_create().await,
            BrowsingContextCommand::GetTree(get_tree) => {
                self.handle_browsing_context_get_tree().await
            },
            BrowsingContextCommand::HandleUserPrompt(handle_user_prompt) => {
                self.handle_browsing_context_handle_user_prompt().await
            },
            BrowsingContextCommand::LocateNodes(locate_nodes) => {
                self.handle_browsing_context_locate_nodes().await
            },
            BrowsingContextCommand::Navigate(navigate) => {
                self.handle_browsing_context_navigate().await
            },
            BrowsingContextCommand::Print(print) => self.handle_browsing_context_print().await,
            BrowsingContextCommand::Reload(reload) => {
                self.handle_browsing_context_reload(reload).await
            },
            BrowsingContextCommand::SetViewport(set_viewport) => {
                self.handle_browsing_context_set_viewport().await
            },
            BrowsingContextCommand::TraverseHistory(traverse_history) => {
                self.handle_browsing_context_traverse_history(traverse_history.params.delta)
                    .await
            },
        }
    }

    async fn handle_browsing_context_activate(
        &self,
        command_parameters: browsing_context::commands::ActivateParams,
    ) -> Result<BrowsingContextResult, WebDriverBidiError> {
        // 1. Let `navigable id` be the value of the `command parameters["context"]` field.
        let navigable_id = &command_parameters.context;

        // 2. Let `navigable` be the result of trying to get a navigable with navigable id.
        let navigable = self.get_a_navigable(navigable_id.as_ref()).await;

        // 3. If `navigable` is not a [top-level traversable], return [error] with [error code invalid argument].
        if !self.is_navigable_top_level_traversable(navigable).await {
            return Err(WebDriverBidiError::new(
                ErrorCode::InvalidArgument,
                "navigable is not top level traversable",
            ));
        }

        // 4. Return [activate a navigable] with `navigable`.
        self.activate_navigable(navigable).await
    }

    async fn handle_browsing_context_capture_screenshot(
        &self,
        command_parameters: browsing_context::commands::CaptureScreenshotParams,
    ) -> Result<BrowsingContextResult, WebDriverBidiError> {
        // Let navigable id be the value of the context field of command parameters if present, or null otherwise.
        //
        // Let navigable be the result of trying to get a navigable with navigable id.
        //
        // If the implementation is unable to capture a screenshot of navigable for any reason then return error with error code unsupported operation.
        //
        // Let document be navigable’s active document.
        //
        // Immediately after the next invocation of the run the animation frame callbacks algorithm for document:
        //
        // Let origin be the value of the context field of command parameters if present, or "viewport" otherwise.
        //
        // Let origin rect be the result of trying to get the origin rectangle given origin and document.
        //
        // Let clip rect be origin rect.
        //
        // If command parameters contains "clip":
        //
        // Let clip be command parameters["clip"].
        //
        // Run the steps under the first matching condition:
        //
        // clip matches the browsingContext.ElementClipRectangle production:
        // Let environment settings be the environment settings object whose relevant global object’s associated Document is document.
        //
        // Let realm be environment settings’ realm execution context’s Realm component.
        //
        // Let element be the result of trying to deserialize remote reference with clip["element"], realm, and session.
        //
        // If element doesn’t implement Element return error with error code no such element.
        //
        // If element’s node document is not document, return error with error code no such element.
        //
        // Let viewport rect be get the origin rectangle given "viewport" and document.
        //
        // Let element rect be get the bounding box for element.
        //
        // Let clip rect be a DOMRectReadOnly with x coordinate element rect["x"] + viewport rect["x"], y coordinate element rect["y"] + viewport rect["y"], width element rect["width"], and height element rect["height"].
        //
        // clip matches the browsingContext.BoxClipRectangle production:
        // Let clip x be clip["x"] plus origin rect’s x coordinate.
        //
        // Let clip y be clip["y"] plus origin rect’s y coordinate.
        //
        // Let clip rect be a DOMRectReadOnly with x coordinate clip x, y coordinate clip y, width clip["width"], and height clip["height"].
        //
        // Note: All coordinates are now measured from the origin of the document.
        //
        // Let rect be the rectangle intersection of origin rect and clip rect.
        //
        // If rect’s width dimension is 0 or rect’s height dimension is 0, return error with error code unable to capture screen.
        //
        // Let canvas be render document to a canvas with document and rect.
        //
        // Let format be the format field of command parameters.
        //
        // Let encoding result be the result of trying to encode a canvas as Base64 with canvas and format.
        //
        // Let body be a map matching the browsingContext.CaptureScreenshotResult production, with the data field set to encoding result.
        //
        // Return success with data body.
        todo!()
    }

    /// The browsingContext.close command closes a top-level traversable.
    ///
    /// The remote end steps with command parameters are:
    async fn handle_browsing_context_close(
        &self,
        command_paramters: browsing_context::commands::CloseParams,
    ) -> Result<BrowsingContextResult, WebDriverBidiError> {
        // The remote end steps with command parameters are:
        // Let navigable id be the value of the context field of command parameters.
        //
        // Let prompt unload be the value of the promptUnload field of command parameters.
        //
        // Let navigable be the result of trying to get a navigable with navigable id.
        //
        // Assert: navigable is not null.
        //
        // If navigable is not a top-level traversable, return error with error code invalid argument.
        //
        // If prompt unload is true:
        //
        // Close navigable.
        //
        // Otherwise:
        //
        // Close navigable without prompting to unload.
        //
        // Return success with data null.
        todo!()
    }

    /// The browsingContext.create command creates a new navigable, either in a new tab or in a new window, and returns its navigable id.
    ///
    /// The remote end steps with command parameters are:
    async fn handle_browsing_context_create(
        &self,
    ) -> Result<BrowsingContextResult, WebDriverBidiError> {
        todo!()
    }

    /// The browsingContext.getTree command returns a tree of all descendent navigables including the given parent itself, or all top-level contexts when no parent is provided.
    ///
    /// The remote end steps with session and command parameters are:
    async fn handle_browsing_context_get_tree(
        &self,
    ) -> Result<BrowsingContextResult, WebDriverBidiError> {
        todo!()
    }
    async fn handle_browsing_context_handle_user_prompt(
        &self,
    ) -> Result<BrowsingContextResult, WebDriverBidiError> {
        todo!()
    }
    async fn handle_browsing_context_locate_nodes(
        &self,
    ) -> Result<BrowsingContextResult, WebDriverBidiError> {
        todo!()
    }
    async fn handle_browsing_context_navigate(
        &self,
    ) -> Result<BrowsingContextResult, WebDriverBidiError> {
        todo!()
    }
    async fn handle_browsing_context_print(
        &self,
    ) -> Result<BrowsingContextResult, WebDriverBidiError> {
        todo!()
    }

    async fn handle_browsing_context_reload(
        &self,
        cmd: browsing_context::commands::Reload,
    ) -> Result<BrowsingContextResult, WebDriverBidiError> {
        // Step 1: let navigable id be "context"
        let navigable_id = &cmd.params.context;

        // Step 2: let navigable, trying to get a navigable with id
        // TODO: impl
        // TODO: should we keep the id here? id may be changed by other session

        // Step 3: Assert navigable is not null
        // TODO: let else

        // Step 4: let ignore cache if present, or false otherwise
        let ignore_cache = cmd.params.ignore_cache.unwrap_or(false);

        // Step 5: let `wait condition` be `"committed"`
        // TODO: should move webdriver type to shared, like webdriver classic
        let mut wait_condition = WaitCondition::Committed;

        // Step 6: if wait and wait not "none", set `wait condition`
        if let Some(wait) = &cmd.params.wait {
            match wait {
                ReadinessState::None => {},
                ReadinessState::Interactive => {
                    wait_condition = WaitCondition::Interactive;
                },
                ReadinessState::Complete => wait_condition = WaitCondition::Complete,
            }
        }

        // Step 7: let document be active document

        // Step 8: let url be document's url

        // Step 9: let request be a new request

        // Step 10: await a navigation
        // TODO: check id
        let browser_context_id = BrowsingContextId::new();
        let cmd_msg = WebDriverBidiToEmbedderMsg::BrowsingContextReload(
            todo!(),
            ignore_cache,
            wait_condition,
        );
        self.send_message_to_embedder(cmd_msg);

        // wait for response in `try_recv`

        Ok(todo!())
    }

    async fn handle_browsing_context_set_viewport(
        &self,
    ) -> Result<BrowsingContextResult, WebDriverBidiError> {
        todo!()
    }
    async fn handle_browsing_context_traverse_history(
        &self,
        delta: i64,
    ) -> Result<BrowsingContextResult, WebDriverBidiError> {
        let webview_id: WebViewId = todo!();
        // TODO: verify context open? is this in bidi spec
        self.send_message_to_embedder(WebDriverBidiToEmbedderMsg::TraverseHistory(
            webview_id, delta,
        ))?;
        // TODO: fix return type
        Ok(todo!())
    }
}

impl Handler {
    async fn get_a_navigable(&self, navigable_id: &str) -> () {
        todo!()
    }

    async fn is_navigable_top_level_traversable(&self, navigable: ()) -> bool {
        todo!()
    }

    /// To activate a navigable given `navigable`:
    async fn activate_navigable(
        &self,
        navigable: (),
    ) -> Result<BrowsingContextResult, WebDriverBidiError> {
        // 1. Run implementation-specific steps so that navigable’s system visibility state becomes visible. If this is
        // not possible return error with error code unsupported operation.

        // 2. Run implementation-specific steps to set the system focus on the navigable if it is not focused.

        // 3. Return [success] with data null.
        Ok(crate::model::BrowsingContextResult::Activate(
            browsing_context::results::ActivateResult {
                extensible: Default::default(),
            },
        ))
    }
}

use std::rc::Rc;

use devtools_traits::WorkerId;
use servo_base::{
    generic_channel::GenericSender,
    id::{BrowsingContextId, PainterId, PipelineId, WebViewId},
};
use uuid::Uuid;
use webdriver_traits::{ScriptToWebDriverMsg, WebDriverToScriptMsg};

use crate::bidi::remote_end::{ClientWindow, RemoteEnd, Traversable};

use super::remote_end::{Document, Navigable, Realm};

impl RemoteEnd {
    pub(crate) fn handle_script(self: Rc<Self>, msg: ScriptToWebDriverMsg) {
        match msg {
            ScriptToWebDriverMsg::LogEntryAddedConsole(items, entry_added) => {
                self.handle_log_entry_added_console()
            },
            ScriptToWebDriverMsg::LogEntryAddedErrorReporting(items, entry_added) => {
                self.handle_log_entry_added_error_reporting()
            },
            ScriptToWebDriverMsg::RealmCreated(
                (browsing_context_id, pipeline_id, worker_id, webview_id),
                script_sender,
            ) => self.handle_realm_created(
                browsing_context_id,
                pipeline_id,
                worker_id,
                webview_id,
                script_sender,
            ),
            ScriptToWebDriverMsg::ChannelMessage { channel, data } => self.handle_channel_message(),
            ScriptToWebDriverMsg::FileDialogOpened(file_dialog_opened) => {
                self.handle_file_dialog_opened()
            },
            ScriptToWebDriverMsg::RealmDestroyed(namespace_index, worker_id) => {
                self.handle_realm_destroyed()
            },
            ScriptToWebDriverMsg::UserPromptClosed(user_prompt_closed_parameters) => {
                self.handle_user_prompt_closed()
            },
            ScriptToWebDriverMsg::UserPromptOpened(user_prompt_opened_parameters) => {
                self.handle_user_prompt_opened()
            },
        }
    }

    fn handle_realm_created(
        &self,
        browsing_context_id: BrowsingContextId,
        pipeline_id: PipelineId,
        worker_id: Option<WorkerId>,
        webview_id: WebViewId,
        script_sender: GenericSender<WebDriverToScriptMsg>,
    ) {
        // add sender
        self.script_senders
            .borrow_mut()
            .insert(pipeline_id, script_sender);
        // TODO: remove sender of inactive document

        // build client window
        let painter_id = PainterId::from(webview_id);
        self.client_windows
            .borrow_mut()
            .entry(painter_id)
            .or_insert_with(|| ClientWindow {
                id: painter_id,
                traversables: vec![],
            })
            .traversables
            .push(webview_id);

        // build traversable
        self.traversables
            .borrow_mut()
            .entry(webview_id)
            .or_insert_with(|| Traversable {
                id: webview_id,
                window_id: painter_id,
                navigables: vec![],
            })
            .navigables
            .push(browsing_context_id);

        // build navigable
        // XXX: this is problematic, relation between relamcreated and navigation is uncertain.
        // e.g. navigate back => relam not created
        self.navigables
            .borrow_mut()
            .entry(browsing_context_id)
            .or_insert_with(|| Navigable {
                id: browsing_context_id,
                traversable_id: webview_id,
                // TODO: multiple pipeline and one active
                documents: vec![],
                active_index: 0,
                original_opener: None,
                is_top_level_traversable: false,
            });
        // TODO: we need to cleanup

        // build document
        // TODO: should generate uuid newtype in codegen
        let realm_id = Uuid::new_v4().to_string();
        self.documents
            .borrow_mut()
            .entry(pipeline_id)
            .or_insert_with(|| Document {
                id: pipeline_id,
                navigable_id: browsing_context_id,
                realms: vec![],
            })
            .realms
            .push(realm_id.clone());

        // build realm
        self.realms
            .borrow_mut()
            .entry(realm_id.clone())
            .or_insert_with(|| Realm {
                id: realm_id,
                document_id: pipeline_id,
                worker_id,
            });
    }

    fn handle_realm_destroyed(self: Rc<Self>) {
        todo!()
    }

    fn handle_log_entry_added_console(self: Rc<Self>) {
        todo!()
    }

    fn handle_log_entry_added_error_reporting(self: Rc<Self>) {
        todo!()
    }

    fn handle_channel_message(self: Rc<Self>) {
        todo!()
    }

    fn handle_file_dialog_opened(self: Rc<Self>) {
        todo!()
    }

    fn handle_user_prompt_closed(self: Rc<Self>) {
        todo!()
    }

    fn handle_user_prompt_opened(self: Rc<Self>) {
        todo!()
    }
}

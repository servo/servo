pub(crate) mod browser;
pub(crate) mod browsing_context;
pub(crate) mod emulation;
pub(crate) mod input;
pub(crate) mod log;
pub(crate) mod network;
pub(crate) mod script;
pub(crate) mod session;
pub(crate) mod storage;
pub(crate) mod web_extension;

use std::{borrow::Cow, collections::HashSet, rc::Rc, sync::OnceLock};

use webdriver_traits::bidi::{Command, CommandData, ResultData};

use crate::bidi::{error::BidiResult, remote_end::RemoteEnd, session::common::SessionId};

static SET_OF_ALL_COMMAND_NAMES: OnceLock<HashSet<Cow<'static, str>>> = OnceLock::new();

impl RemoteEnd {
    /// <https://www.w3.org/TR/webdriver-bidi/#set-of-all-command-names>
    pub(crate) fn set_of_all_command_names(&self) -> &HashSet<Cow<'static, str>> {
        SET_OF_ALL_COMMAND_NAMES.get_or_init(|| {
            // TODO: init from standard and custom
            HashSet::new()
        })
    }

    pub(crate) async fn handle_command(
        self: Rc<Self>,
        session_id: Option<SessionId>,
        command: CommandData,
    ) -> BidiResult<ResultData> {
        match command {
            CommandData::BrowserCommand(cmd) => self
                .handle_browser_command(session_id.unwrap(), cmd)
                .await
                .map(ResultData::BrowserResult),
            CommandData::BrowsingContextCommand(cmd) => self
                .handle_browsing_context_command(session_id.unwrap(), cmd)
                .await
                .map(ResultData::BrowsingContextResult),
            CommandData::EmulationCommand(cmd) => self
                .handle_emulation_command(cmd)
                .await
                .map(ResultData::EmulationResult),
            CommandData::InputCommand(cmd) => self
                .handle_input_command(cmd)
                .await
                .map(ResultData::InputResult),
            CommandData::NetworkCommand(cmd) => self
                .handle_network_command(cmd)
                .await
                .map(ResultData::NetworkResult),
            CommandData::ScriptCommand(cmd) => self
                .handle_script_command(cmd)
                .await
                .map(|r| ResultData::ScriptResult(Box::new(r))),
            CommandData::SessionCommand(cmd) => self
                .handle_session_command(session_id, cmd)
                .await
                .map(ResultData::SessionResult),
            CommandData::StorageCommand(cmd) => self
                .handle_storage_command(cmd)
                .await
                .map(|r| ResultData::StorageResult(Box::new(r))),
            CommandData::WebExtensionCommand(cmd) => self
                .handle_web_extension_command(cmd)
                .await
                .map(ResultData::WebExtensionResult),
        }
    }
}

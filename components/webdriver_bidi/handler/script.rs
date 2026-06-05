use rustenium_bidi_definitions::script::commands::ScriptCommand;

use crate::{error::WebDriverBidiError, handler::Handler, model::ScriptResult};

impl Handler {
    pub(super) async fn handle_script(
        &self,
        cmd: &ScriptCommand,
    ) -> Result<ScriptResult, WebDriverBidiError> {
        match cmd {
            ScriptCommand::AddPreloadScript(add_preload_script) => {
                self.handle_script_add_preload_script().await
            },
            ScriptCommand::Disown(disown) => self.handle_script_disown().await,
            ScriptCommand::CallFunction(call_function) => self.handle_script_call_function().await,
            ScriptCommand::Evaluate(evaluate) => self.handle_script_evaluate().await,
            ScriptCommand::GetRealms(get_realms) => self.handle_script_get_realms().await,
            ScriptCommand::RemovePreloadScript(remove_preload_script) => {
                self.handle_script_remove_preload_script().await
            },
        }
    }

    async fn handle_script_add_preload_script(&self) -> Result<ScriptResult, WebDriverBidiError> {
        todo!()
    }
    async fn handle_script_disown(&self) -> Result<ScriptResult, WebDriverBidiError> {
        todo!()
    }
    async fn handle_script_call_function(&self) -> Result<ScriptResult, WebDriverBidiError> {
        todo!()
    }
    async fn handle_script_evaluate(&self) -> Result<ScriptResult, WebDriverBidiError> {
        todo!()
    }
    async fn handle_script_get_realms(&self) -> Result<ScriptResult, WebDriverBidiError> {
        todo!()
    }
    async fn handle_script_remove_preload_script(
        &self,
    ) -> Result<ScriptResult, WebDriverBidiError> {
        todo!()
    }
}

use std::rc::Rc;

use webdriver_traits::bidi::{
    ScriptCommand, ScriptResult,
    script::{
        AddPreloadScriptParameters, AddPreloadScriptResult, CallFunctionParameters,
        CallFunctionResult, DisownParameters, DisownResult, EvaluateParameters, EvaluateResult,
        EvaluateResultSuccess, GetRealmsParameters, GetRealmsResult, RemovePreloadScriptParameters,
        RemovePreloadScriptResult, ResultOwnership,
    },
};

use crate::bidi::{error::BidiResult, remote_end::RemoteEnd};

impl RemoteEnd {
    pub(crate) async fn handle_script_command(
        self: Rc<Self>,
        command: ScriptCommand,
    ) -> BidiResult<ScriptResult> {
        match command {
            ScriptCommand::AddPreloadScript(cmd) => self
                .handle_script_add_preload_script(cmd.params)
                .await
                .map(ScriptResult::AddPreloadScriptResult),
            ScriptCommand::CallFunction(cmd) => self
                .handle_script_call_function(cmd.params)
                .await
                .map(ScriptResult::CallFunctionResult),
            ScriptCommand::Disown(cmd) => self
                .handle_script_disown(cmd.params)
                .await
                .map(ScriptResult::DisownResult),
            ScriptCommand::Evaluate(cmd) => self
                .handle_script_evaluate(cmd.params)
                .await
                .map(ScriptResult::EvaluateResult),
            ScriptCommand::GetRealms(cmd) => self
                .handle_script_get_realms(cmd.params)
                .await
                .map(ScriptResult::GetRealmsResult),
            ScriptCommand::RemovePreloadScript(cmd) => self
                .handle_script_remove_preload_script(cmd.params)
                .await
                .map(ScriptResult::RemovePreloadScriptResult),
        }
    }

    /// <https://www.w3.org/TR/webdriver-bidi/#command-script-addPreloadScript>
    async fn handle_script_add_preload_script(
        self: Rc<Self>,
        _: AddPreloadScriptParameters,
    ) -> BidiResult<AddPreloadScriptResult> {
        todo!()
    }

    /// <https://www.w3.org/TR/webdriver-bidi/#command-script-disown>
    async fn handle_script_disown(self: Rc<Self>, _: DisownParameters) -> BidiResult<DisownResult> {
        todo!()
    }

    /// <https://www.w3.org/TR/webdriver-bidi/#command-script-callFunction>
    async fn handle_script_call_function(
        self: Rc<Self>,
        _: CallFunctionParameters,
    ) -> BidiResult<CallFunctionResult> {
        todo!()
    }

    /// <https://www.w3.org/TR/webdriver-bidi/#command-script-evaluate>
    async fn handle_script_evaluate(
        self: Rc<Self>,
        command_parameters: EvaluateParameters,
    ) -> BidiResult<EvaluateResult> {
        // 1.
        // let realm = todo!();
        // 2.
        // let realm_id = realm.id;
        // 3.
        // let environment_setting = realm.environment_setting_object;
        // 4.
        let source = &command_parameters.expression;
        // 5.
        let await_promise = &command_parameters.await_promise;
        // 6.
        let serialization_options = &command_parameters
            .serialization_options
            .clone()
            .unwrap_or_default();
        // 7.
        let result_ownership = command_parameters
            .result_ownership
            .clone()
            .unwrap_or(ResultOwnership::None);
        // 8. TODO:
        // 9. TODO:
        // 10.
        let bypass_disable_scripting = true;
        // 11. TODO: script thread
        // 12.
        if command_parameters.user_activation.unwrap_or(false) {
            // TODO: activation notification
        }
        // 13. TODO: should be in script thread
        // 14.
        // 15.
        // 16.
        // 17.
        let evaluation_value = todo!();
        // 18. SKIP: assert
        // 19. TODO: await for script callback
        let result = todo!();
        // 20.
        Ok(EvaluateResult::EvaluateResultSuccess(
            EvaluateResultSuccess {
                result: result,
                realm: todo!(),
            },
        ))
    }

    /// <https://www.w3.org/TR/webdriver-bidi/#command-script-getRealms>
    async fn handle_script_get_realms(
        self: Rc<Self>,
        _: GetRealmsParameters,
    ) -> BidiResult<GetRealmsResult> {
        todo!()
    }

    /// <https://www.w3.org/TR/webdriver-bidi/#command-script-removePreloadScript>
    async fn handle_script_remove_preload_script(
        self: Rc<Self>,
        _: RemovePreloadScriptParameters,
    ) -> BidiResult<RemovePreloadScriptResult> {
        todo!()
    }

    /// Remote end subscribe steps for `script.realmCreated`.
    pub(crate) async fn subscribe_script_realm_created(self: Rc<Self>) {
        todo!()
    }
}

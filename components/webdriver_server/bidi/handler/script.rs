use std::collections::{HashMap, HashSet};

use servo_webdriver::bidi::{
    ErrorCode, ScriptCommand, ScriptResult,
    browsing_context::BrowsingContext,
    script::{
        AddPreloadScriptParameters, AddPreloadScriptResult, CallFunctionParameters,
        CallFunctionResult, DisownParameters, DisownResult, EvaluateParameters, EvaluateResult,
        EvaluateResultSuccess, GetRealmsParameters, GetRealmsResult, Handle, PreloadScript,
        RealmInfo, RemovePreloadScriptParameters, RemovePreloadScriptResult, Target,
    },
};
use uuid::Uuid;

use crate::bidi::{error::WebDriverBidiError, handler::Handler};

impl Handler {
    pub(super) async fn handle_script(
        &self,
        cmd: ScriptCommand,
    ) -> Result<ScriptResult, WebDriverBidiError> {
        match cmd {
            ScriptCommand::AddPreloadScript(cmd) => self
                .handle_script_add_preload_script(cmd.params)
                .await
                .map(ScriptResult::AddPreloadScriptResult),
            ScriptCommand::Disown(cmd) => self
                .handle_script_disown(cmd.params)
                .await
                .map(ScriptResult::DisownResult),
            ScriptCommand::CallFunction(cmd) => self
                .handle_script_call_function(cmd.params)
                .await
                .map(ScriptResult::CallFunctionResult),
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
        &self,
        command_parameters: AddPreloadScriptParameters,
    ) -> Result<AddPreloadScriptResult, WebDriverBidiError> {
        // 1. If `command parameters` [contains] "userContexts" and `command parameters` [contains] "contexts",
        // return [error] with [error code] [invalid argument].
        if command_parameters.user_contexts.is_some() && command_parameters.contexts.is_some() {
            return Err(WebDriverBidiError::new(
                ErrorCode::InvalidArgument,
                "both `userContexts` and `contexts` exist in command parameters",
            ));
        }

        // 2. Let `function declaration` be the `functionDeclaration` field of `command parameters`.
        let function_declaration = command_parameters.function_declaration;

        // 3. Let `arguments` be the `arguments` field of `command parameters` if present, or an empty [list] otherwise.
        let arguments = command_parameters.arguments.unwrap_or_default();

        // 4. Let `user contexts` to be a set.
        let user_contexts = HashSet::<()>::new();

        // 5. Let navigables be null.
        let navigables: Option<HashSet<()>> = None;

        // 6. If the `contexts` field of `command parameters` is present:
        if let Some(contexts) = command_parameters.contexts {
            // 6.1 Set `navigables` to an empty [set].
            let mut navigables = HashSet::<()>::new();

            // 6.2 For each `navigable id` of `command parameters["contexts"]`
            for navigable_id in contexts {
                // 6.2.1 Let `navigable` be the result of [trying] to [get a navigable] with `navigable id`.
                let navigable = self.get_navigable(&navigable_id)?;

                // 6.2.2 If `navigable` is not a [top-level traversable], return [error] with [error code] [invalid argument].
                if true {
                    return Err(WebDriverBidiError::new(
                        ErrorCode::InvalidArgument,
                        "navigable is not a top-level traverable",
                    ));
                }

                // 6.2.3 Append `navigable` to `navigables`.
                navigables.insert(navigable);
            }
        }
        // 7. Otherwise, if `command parameters` contains `userContexts`:
        else if let Some(cmd_user_contexts) = command_parameters.user_contexts {
            // 7.1. Set `user contexts` to [create a set] with `command parameters["userContexts"]`.
            let user_contexts = HashSet::<()>::new();

            // 7.2. For each `user context id` of `user contexts`:
            for user_context_id in cmd_user_contexts {
                // TODO: get user context is not implemented
                // 7.2.1. Set `user context` to [get user context] with `user context id`.
                // 7.2.2. If `user context` is null, return [error] with [error code] [no such user context].
            }
        }

        // 8. Let `sandbox` be the value of the "sandbox" field in `command parameters`, if present, or null otherwise.
        let sandbox = command_parameters.sandbox;

        // 9. Let `script` be the string representation of a [UUID].
        let script = Uuid::new_v4();

        // 10. Let `preload script map` be `session`’s [preload script map].
        let preload_script_map = self.preload_script_map();

        // 11. Set `preload script map[script]` to a struct with function declaration `function declaration`, arguments `arguments`, contexts `navigables`, sandbox `sandbox`, and user contexts `user contexts`.
        // TODO:

        // 12. Return a new [map] matching the `script.AddPreloadScriptResult` with the `script` field set to `script`.
        Ok(AddPreloadScriptResult { script: todo!() })
    }

    /// <https://www.w3.org/TR/webdriver-bidi/#command-script-disown>
    async fn handle_script_disown(
        &self,
        command_parameters: DisownParameters,
    ) -> Result<DisownResult, WebDriverBidiError> {
        // 1. Let `realm` be the result of [trying] to [get a realm from a target] given the value of the `target` field of
        // `command parameters`.
        let realm = self.get_realm_from_target(&command_parameters.target)?;

        // 2. Let `handles` the value of the `handles` field of `command parameters`.
        let handles = command_parameters.handles;

        // 3. For each `handle id` of `handles`:
        for handle_id in handles {
            // 3.1. Let `handle map` be `realm`’s [handle object map]
            let mut handle_map = realm.handle_object_map();

            // 3.2. If `handle map` contains `handle id`, remove `handle id` from the `handle map`.
            if handle_map.contains_key(&handle_id) {
                handle_map.remove(&handle_id);
            }
        }
        // 4. Return [success] with data null.
        todo!()
    }

    /// <https://www.w3.org/TR/webdriver-bidi/#command-script-callFunction>
    async fn handle_script_call_function(
        &self,
        command_parameters: CallFunctionParameters,
    ) -> Result<CallFunctionResult, WebDriverBidiError> {
        // 1. Let realm be the result of trying to get a realm from a target given the value of the target field of command parameters.
        // 2. Let realm id be realm’s realm id.
        // 3. Let environment settings be the environment settings object whose realm execution context’s Realm component is realm.
        // 4. Let command arguments be the value of the arguments field of command parameters.
        // 5. Let deserialized arguments be an empty list.
        // 6. If command arguments is not null, set deserialized arguments to the result of trying to deserialize arguments given realm, command arguments and session.
        // 7. Let this parameter be the value of the this field of command parameters.
        // 8. Let this object be null.
        // 9. If this parameter is not null, set this object to the result of trying to deserialize local value given this parameter, realm and session.
        // 10. Let function declaration be the value of the functionDeclaration field of command parameters.
        // 11. Let await promise be the value of the awaitPromise field of command parameters.
        // 12. Let serialization options be the value of the serializationOptions field of command parameters, if present, or otherwise a map matching the script.SerializationOptions production with the fields set to their default values.
        // 13. Let result ownership be the value of the resultOwnership field of command parameters, if present, or none otherwise.
        // 14. Let base URL be the API base URL of environment settings.
        // 15. Let options be the default script fetch options.
        // 16. Let function body evaluation status be the result of evaluate function body with function declaration, environment settings, base URL, and options.
        // 17. If function body evaluation status.[[Type]] is throw:
        // 17.1. Let exception details be the result of get exception details given realm, function body evaluation status, result ownership and session.
        // 17.2. Return a new map matching the script.EvaluateResultException production, with the exceptionDetails field set to exception details.
        // 18. Let function object be function body evaluation status.[[Value]].
        // 19. If IsCallable(function object) is false:
        // 19.1. Return an error with error code invalid argument
        // 20. If command parameters["userActivation"] is true, run activation notification steps.
        // 21. Prepare to run script with environment settings.
        // 22. Set evaluation status to Call(function object, this object, deserialized arguments).
        // 23. If evaluation status.[[Type]] is normal, and await promise is true, and IsPromise(evaluation status.[[Value]]):
        // 23.1. Set evaluation status to Await(evaluation status.[[Value]]).
        // 24. Clean up after running script with environment settings.
        // 25. If evaluation status.[[Type]] is throw:
        // 25.1. Let exception details be the result of get exception details given realm, evaluation status, result ownership and session.
        // 25.2. Return a new map matching the script.EvaluateResultException production, with the exceptionDetails field set to exception details.
        // 26. Assert: evaluation status.[[Type]] is normal.
        // 27. Let result be the result of serialize as a remote value with evaluation status.[[Value]], serialization options, result ownership, a new map as serialization internal map, realm and session.
        // 28. Return a new map matching the script.EvaluateResultSuccess production, with the realm field set to realm id, and the result field set to result.
        todo!()
    }

    /// <https://www.w3.org/TR/webdriver-bidi/#command-script-evaluate>
    async fn handle_script_evaluate(
        &self,
        command_parameters: EvaluateParameters,
    ) -> Result<EvaluateResult, WebDriverBidiError> {
        // 1. Let realm be the result of trying to get a realm from a target given the value of the target field of command parameters.
        // 2. Let realm id be realm’s realm id.
        // 3. Let environment settings be the environment settings object whose realm execution context’s Realm component is realm.
        // 4. Let source be the value of the expression field of command parameters.
        // 5. Let await promise be the value of the awaitPromise field of command parameters.
        // 6. Let serialization options be the value of the serializationOptions field of command parameters, if present, or otherwise a map matching the script.SerializationOptions production with the fields set to their default values.
        // 7. Let result ownership be the value of the resultOwnership field of command parameters, if present, or none otherwise.
        // 8. Let options be the default script fetch options.
        // 9. Let base URL be the API base URL of environment settings.
        // 10. Let bypassDisabledScripting be true.
        // 11. Let script be the result of create a classic script with source, environment settings, base URL, options and bypassDisabledScripting.
        // 12. If command parameters["userActivation"] is true, run activation notification steps.
        // 13. Prepare to run script with environment settings.
        // 14. Set evaluation status to ScriptEvaluation(script’s record).
        // 15. If evaluation status.[[Type]] is normal, await promise is true, and IsPromise(evaluation status.[[Value]]):
        // 15.1. Set evaluation status to Await(evaluation status.[[Value]]).
        // 16. Clean up after running script with environment settings.

        // 17. If evaluation status.[[Type]] is throw:
        // 17.1. Let exception details be the result of get exception details with realm, evaluation status, result ownership and session.
        // 17.2. Return a new map matching the script.EvaluateResultException production, with the realm field set to realm id, and the exceptionDetails field set to exception details.

        // 18. Assert: evaluation status.[[Type]] is normal.

        // 19. Let `result` be the result of [serialize as a remote value] with `evaluation status.[[Value]]`, `serialization
        // options`, `result ownership`, a new [map] as serialization internal map, `realm` and `session`.

        // 20  Return a new [map] matching the `script.EvaluateResultSuccess` production, with the with the `realm`
        // field set to `realm id`, and the `result` field set to result.
        Ok(EvaluateResult::EvaluateResultSuccess(
            EvaluateResultSuccess {
                result: todo!(),
                realm: todo!(),
            },
        ))
    }

    /// <https://www.w3.org/TR/webdriver-bidi/#command-script-getRealms>
    async fn handle_script_get_realms(
        &self,
        command_parameters: GetRealmsParameters,
    ) -> Result<GetRealmsResult, WebDriverBidiError> {
        // 1. Let `environment settings` be a [list] of all the [environment settings objects] that have their [execution
        // ready flag] set.
        let mut environment_settings = Vec::<()>::new();

        // 2. If `command parameters` contains `context`:
        if let Some(context) = command_parameters.context {
            // 2.1. Let `navigable` be the result of [trying] to [get a navigable] with `command parameters["context"]`.
            let navigable = self.get_navigable(&context)?;

            // 2.2. Let `document` be `navigable`’s [active document].
            let document = ();

            // 2.3. Let `navigable environment settings` be a [list].
            let mut navigable_environment_settings = Vec::<()>::new();

            // 2.4. For each `settings` of `environment settings`:
            for settings in environment_settings {
                // 2.4.1. If any of the following conditions hold:
                // - The [associated `Document`] of `settings`’ [relevant global object] is `document`
                // - The [global object] specified by `settings` is a [WorkerGlobalScope] with `document` in its
                // [owner set]
                let cond1 = true;
                let cond2 = true;
                if cond1 && cond2 {
                    // Append `settings` to `navigable environment settings`.
                    navigable_environment_settings.push(settings);
                }
            }

            // 2.5. Set `environment settings` to `navigable environment settings`.
            environment_settings = navigable_environment_settings;
        }

        // 3. Let `realms` be a list.
        let mut realms = Vec::<RealmInfo>::new();

        // 4. For each `settings` of `environment settings`:
        for settings in environment_settings {
            // 4.1. Let `realm info` be the result of [get the realm info] given `settings`.
            let realm_info = self.get_realm_info(&settings);

            // 4.2. If `command parameters` contains `type` and `realm info["type"]`` is not equal to `command
            // parameters["type"]` then [continue].
            if let Some(r#type) = &command_parameters.r#type
                && let Some(realm_info) = realm_info
            // TODO: need type method
            // && realm_info.r#type != r#type
            {
                continue;
            }

            // 4.3. If `realm info` is not null, append `realm info` to `realms`.
            if let Some(realm_info) = realm_info {
                realms.push(realm_info);
            }
        }

        // 5. Let `body` be a [map] matching the `script.GetRealmsResult` production, with the `realms` field set to
        // realms.
        let body = GetRealmsResult { realms };

        // 6. Return success with data body.
        Ok(body)
    }

    /// <https://www.w3.org/TR/webdriver-bidi/#command-script-removePreloadScript>
    async fn handle_script_remove_preload_script(
        &self,
        command_parameters: RemovePreloadScriptParameters,
    ) -> Result<RemovePreloadScriptResult, WebDriverBidiError> {
        // 1. Let `script` be the value of the `"script"` field in `command parameters`.
        let script = command_parameters.script;

        // 2. Let `preload script map` be `session`’s [preload script map].
        let mut preload_script_map = self.preload_script_map();

        // 3. If `preload script map` does not [contain] `script`, return [error] with [error code] [no such script].
        if !preload_script_map.contains_key(&script) {
            return Err(WebDriverBidiError::new(
                ErrorCode::NoSuchScript,
                "script not in preload script map",
            ));
        }

        // 4. [Remove] `script` from `preload script map`.
        preload_script_map.remove(&script);

        // 5. Return null
        Ok(RemovePreloadScriptResult {
            extensible: Default::default(),
        })
    }
}

impl Handler {
    fn get_navigable(&self, context: &BrowsingContext) -> Result<(), WebDriverBidiError> {
        todo!()
    }

    /// <https://www.w3.org/TR/webdriver-bidi/#get-the-realm-info>
    fn get_realm_info(&self, settings: &()) -> Option<RealmInfo> {
        todo!()
    }

    fn preload_script_map(&self) -> HashMap<PreloadScript, ()> {
        todo!()
    }

    fn get_realm_from_target(&self, target: &Target) -> Result<Realm, WebDriverBidiError> {
        todo!()
    }
}

struct Realm;

impl Realm {
    fn handle_object_map(&self) -> HashMap<Handle, ()> {
        todo!()
    }
}

use std::collections::HashSet;

use servo_base::id::BrowsingContextId;
use webdriver_traits::{
    bidi::{
        EmptyResult, ErrorCode, Event, EventData, ResultData, ScriptCommand, ScriptEvent,
        ScriptResult,
        script::{
            AddPreloadScriptParameters, AddPreloadScriptResult, CallFunctionParameters,
            CallFunctionResult, DisownParameters, DisownResult, EvaluateParameters,
            EvaluateResultException, EvaluateResultSuccess, GetRealmsResult, RealmCreated,
            RealmInfo, RemovePreloadScriptParameters, ResultOwnership,
        },
    },
    ids::{PreloadScriptId, RealmId, ResumeId, SessionId},
    messages::{
        CallFunctionBody, EvaluateBody, EvaluationResultBody, PreloadScriptBody,
        WebDriverToScriptMessage,
    },
};

use crate::bidi::{
    WebDriverBidiThread, modules::CommandHandled, session::PreloadScriptMapValue, wait::Resumable,
};

impl WebDriverBidiThread {
    pub(super) fn handle_script(
        &mut self,
        command_id: ResumeId,
        session: SessionId,
        command: ScriptCommand,
    ) {
        match command {
            ScriptCommand::AddPreloadScript(cmd) => {
                self.handle_script_add_preload_script(command_id, session, cmd.params)
            },
            ScriptCommand::CallFunction(cmd) => {
                self.handle_script_call_function(command_id, cmd.params)
            },
            ScriptCommand::Disown(cmd) => self.handle_script_disown(command_id, cmd.params),
            ScriptCommand::Evaluate(cmd) => self.handle_script_evalute(command_id, cmd.params),
            ScriptCommand::GetRealms(_) => self.handle_script_get_realms(command_id),
            ScriptCommand::RemovePreloadScript(cmd) => {
                self.handle_script_remove_preload_script(command_id, session, cmd.params)
            },
        }
    }

    /// Add ... to session ...
    /// In our implementation, each req ...
    /// See <https://www.w3.org/TR/webdriver-bidi/#command-script-addPreloadScript>.
    fn handle_script_add_preload_script(
        &mut self,
        command_id: ResumeId,
        session: SessionId,
        command_parameters: AddPreloadScriptParameters,
    ) {
        // Step 1. guard out both context and user context
        if command_parameters.contexts.is_some() && command_parameters.user_contexts.is_some() {
            self.resume::<CommandHandled>(command_id, Err(ErrorCode::InvalidArgument.into()));
            return;
        }
        // Step 2. let function declaration
        let function_declaration = command_parameters.function_declaration;
        // Step 3. let arguments
        let arguments = command_parameters.arguments.unwrap_or_default();
        // Step 4. let user contexts
        // TODO: user context is not implemented
        // Step 5 & 6. let navigables
        let navigables = match command_parameters.contexts {
            None => None,
            // Step 6. if contexts
            Some(contexts) => {
                // Step 6.1. let empty set
                let mut navigables = HashSet::new();
                // Step 6.2. for each navigable id
                for navigable_id in contexts {
                    // Step 6.2.1. "get a navigable"
                    let navigable = match self.get_a_navigable(navigable_id) {
                        Ok(navigable) => navigable,
                        Err(err) => {
                            self.resume::<CommandHandled>(command_id, Err(err));
                            return;
                        },
                    };
                    // Step 6.2.2. if not top-level, error with "invalid argument"
                    if !navigable.top_level {
                        self.resume::<CommandHandled>(
                            command_id,
                            Err(ErrorCode::InvalidArgument.into()),
                        );
                        return;
                    }
                    // Step 6.2.3.
                    navigables.insert(navigable.id);
                }
                Some(navigables)
            },
        };
        // Step 7. if userContexts
        // TODO: user context is not implemented
        // Step 8. let sandbox
        let sandbox = command_parameters.sandbox;
        // Step 9. let scipt uuid
        let script = PreloadScriptId::default();
        // also send delta script to current remote
        let mut realms = vec![];
        for navigable_id in navigables.iter().flat_map(|o| o.iter()) {
            if let Some(realm_id) = self.navigables.get(navigable_id).map(|n| n.active_realm) {
                realms.push(realm_id);
                self.send_to_realm(
                    realm_id,
                    WebDriverToScriptMessage::AddPreloadScripts(
                        realm_id,
                        vec![(
                            script,
                            PreloadScriptBody {
                                function_declaration: function_declaration.clone(),
                                arguments: arguments.clone(),
                                sandbox: sandbox.clone(),
                            },
                        )],
                    ),
                );
            }
        }
        // Step 10. get session's preload script map
        let preload_script_map = &mut self
            .sessions
            .get_mut(&session)
            // PRE: session existence checked before
            .unwrap()
            .preload_script_map;
        // Step 11. set "preload script map"[script]
        preload_script_map.insert(
            script,
            PreloadScriptMapValue {
                function_declaration,
                arguments,
                navigables: navigables.iter().flat_map(|s| s.iter()).copied().collect(),
                sandbox,
                user_contexts: vec![],
                realms,
            },
        );
        // Step 12. return result
        self.resume::<CommandHandled>(
            command_id,
            Ok(ResultData::Script(ScriptResult::AddPreloadScript(
                AddPreloadScriptResult { script },
            ))),
        );
    }

    /// See <https://www.w3.org/TR/webdriver-bidi/#command-script-disown>.
    fn handle_script_disown(&mut self, command_id: ResumeId, command_parameters: DisownParameters) {
        // Step 1. get a realm from a target
        let realm = match self.get_a_realm_from_a_target(command_parameters.target) {
            Err(err) => {
                self.resume::<CommandHandled>(command_id, Err(err));
                return;
            },
            Ok(realm) => realm,
        };
        let realm_id = realm.id;
        // Step 2. let handles be the handles field
        let handles = command_parameters.handles;
        // Step 3. remove each handle from handle map.
        // This step in done on script thread.
        let disowned_id = ResumeId::next();
        self.awaits(disowned_id, Disowned(command_id));
        self.send_to_realm(
            realm_id,
            WebDriverToScriptMessage::Disown(disowned_id, realm_id, handles),
        );
    }

    /// See <https://www.w3.org/TR/webdriver-bidi/#command-script-callFunction>.
    fn handle_script_call_function(
        &mut self,
        command_id: ResumeId,
        command_parameters: CallFunctionParameters,
    ) {
        // Step 1. get a realm from target
        let realm = match self.get_a_realm_from_a_target(command_parameters.target) {
            Err(err) => {
                self.resume::<CommandHandled>(command_id, Err(err));
                return;
            },
            Ok(realm) => realm,
        };
        // Step 2. let "realm id"
        let realm_id = realm.id;
        // Step 3. get environment settings
        // Step 4. let "command arguments"
        let command_arguments = command_parameters.arguments.unwrap_or_default();
        // Step 7. let "this parameter"
        let this_parameter = command_parameters.this;
        // Step 10. let "function declaration"
        let function_declaration = command_parameters.function_declaration;
        // Step 11. let "await promise"
        let await_promise = command_parameters.await_promise;
        // Step 12. let "serialization options"
        let serialization_options = command_parameters.serialization_options.unwrap_or_default();
        // Step 13. let "result ownership"
        let result_ownership = command_parameters
            .result_ownership
            .unwrap_or(ResultOwnership::None);

        // Send the function to realm and await for response
        let eval_id = ResumeId::next();
        self.send_to_realm(
            realm_id,
            WebDriverToScriptMessage::CallFunction(
                eval_id,
                realm_id,
                CallFunctionBody {
                    function_declaration,
                    await_promise,
                    arguments: command_arguments,
                    result_ownership,
                    serialization_options,
                    this: this_parameter,
                    user_activation: command_parameters.user_activation.unwrap_or(false),
                },
            ),
        );
        self.awaits(
            eval_id,
            Evaluated {
                function: true,
                command_id,
                realm_id,
            },
        );
    }

    /// See <https://www.w3.org/TR/webdriver-bidi/#command-script-evaluate>.
    fn handle_script_evalute(
        &mut self,
        command_id: ResumeId,
        command_parameters: EvaluateParameters,
    ) {
        // Step 1. "get a realm from a target"
        let realm = match self.get_a_realm_from_a_target(command_parameters.target) {
            Err(err) => {
                self.resume::<CommandHandled>(command_id, Err(err));
                return;
            },
            Ok(realm) => realm,
        };
        // Step 2. realm id
        let realm_id = realm.id;
        // Step 4. let "source"
        let source = command_parameters.expression;
        // Step 5. let "await promise"
        let await_promise = command_parameters.await_promise;
        // Step 6. let "serialization options"
        let serialization_options = command_parameters.serialization_options.unwrap_or_default();
        // Step 7. let "result ownership"
        let result_ownership = command_parameters
            .result_ownership
            .unwrap_or(ResultOwnership::None);

        // Send the function to realm and await for response
        let eval_id = ResumeId::next();
        self.send_to_realm(
            realm_id,
            WebDriverToScriptMessage::Evaluate(
                eval_id,
                realm_id,
                EvaluateBody {
                    expression: source,
                    await_promise,
                    result_ownership,
                    serialization_options,
                    user_activation: command_parameters.user_activation.unwrap_or(false),
                },
            ),
        );
        self.awaits(
            eval_id,
            Evaluated {
                function: false,
                command_id,
                realm_id,
            },
        );
    }

    /// See <https://www.w3.org/TR/webdriver-bidi/#command-script-getRealms>.
    fn handle_script_get_realms(&mut self, command_id: ResumeId) {
        // Step 1 & 2. skip, we build cache from messages, and actually
        // there is nowhere to query global realm information.
        // Step 3. let realms be a list.
        let mut realms = vec![];
        // Step 4. populate relams
        for realm in self.realms.values() {
            realms.push(realm.info.clone());
        }
        // Step 5. let body.
        let body = GetRealmsResult { realms };
        // Step 6. return success.
        self.resume::<CommandHandled>(
            command_id,
            Ok(ResultData::Script(ScriptResult::GetRealms(body))),
        );
    }

    /// See <https://www.w3.org/TR/webdriver-bidi/#command-script-removePreloadScript>.
    fn handle_script_remove_preload_script(
        &mut self,
        command_id: ResumeId,
        session: SessionId,
        command_parameters: RemovePreloadScriptParameters,
    ) {
        // Step 1. let script from params
        let script = command_parameters.script;
        // Step 2. get session's preload script map.
        let preload_script_map = &mut self
            .sessions
            .get_mut(&session)
            // PRE: session is checked before.
            .unwrap()
            .preload_script_map;
        match preload_script_map.remove(&script) {
            // Step 3. if no script, error with "no such script"
            None => self.resume::<CommandHandled>(command_id, Err(ErrorCode::NoSuchScript.into())),
            // Step 4. remove script from preload script map.
            Some(value) => {
                self.resume::<CommandHandled>(
                    command_id,
                    Ok(ResultData::Script(ScriptResult::RemovePreloadScript(
                        EmptyResult::default(),
                    ))),
                );

                // also send delta script to current remote
                for realm_id in value.realms {
                    self.send_to_realm(
                        realm_id,
                        WebDriverToScriptMessage::RemovePreloadScripts(realm_id, vec![script]),
                    );
                }
            },
        }
    }

    pub(crate) fn subscribe_script_realm_created(
        &mut self,
        session_id: SessionId,
        navigables: &HashSet<BrowsingContextId>,
        include_global: bool,
    ) {
        // Step 1. skip abstract step
        // Step 2.
        for realm in self.realms.values() {
            // Step 2.1.
            let mut related_navigables = HashSet::new();
            // Step 2.2.
            if let RealmInfo::Window(_) = &realm.info {
                // Step 2.2.1.
                let navigable = realm.navigable_id().and_then(|id| self.navigables.get(&id));
                // Step 2.2.2.
                let Some(navigable) = navigable else {
                    continue;
                };
                // Step 2.2.3.
                if navigable.webview_id.is_some() {
                    if let Some(top_level_navigable) = self.navigables.values().find(|candidate| {
                        candidate.top_level
                            && candidate.id != navigable.id
                            && candidate.webview_id == navigable.webview_id
                    }) {
                        // Step 2.2.4.
                        if !navigables.contains(&top_level_navigable.id) {
                            continue;
                        }
                        // Step 2.2.5.
                        related_navigables.insert(top_level_navigable.id);
                    }
                } else if include_global {
                    continue;
                }
            }
            // Step 2.3.
            let realm_info = &realm.info;
            // Step 2.4. skip as we do not have other realm type now
            // Step 2.5.
            let body = RealmCreated {
                params: realm_info.clone(),
            };
            // Step 2.6. if event is enabled
            if let Some(session) = self.sessions.get(&session_id)
                && session.event_is_enabled("script.realmCreated")
            {
                self.emit_an_event(
                    session,
                    Event {
                        event_data: EventData::Script(ScriptEvent::RealmCreated(body)),
                        extensible: Default::default(),
                    },
                );
            }
        }
    }
}

pub(crate) struct Disowned(ResumeId);

impl Resumable for Disowned {
    type Event = ();

    /// See <https://www.w3.org/TR/webdriver-bidi/#command-script-disown>.
    fn resume(self, this: &mut WebDriverBidiThread, _: Self::Event) {
        // Step 4. return success
        this.resume::<CommandHandled>(
            self.0,
            Ok(ResultData::Script(ScriptResult::Disown(
                DisownResult::default(),
            ))),
        );
    }
}

pub(crate) struct Evaluated {
    /// callFunction or evaluate
    function: bool,
    command_id: ResumeId,
    realm_id: RealmId,
}

impl Resumable for Evaluated {
    type Event = Result<EvaluationResultBody, ErrorCode>;

    /// See <https://www.w3.org/TR/webdriver-bidi/#command-script-callFunction>.
    fn resume(self, this: &mut WebDriverBidiThread, event: Self::Event) {
        let Self {
            function,
            command_id,
            realm_id,
        } = self;
        let map = event.map(|event| match event {
            // Step 25. return exception
            EvaluationResultBody::Exception(exception_details) => {
                CallFunctionResult::Exception(EvaluateResultException {
                    exception_details,
                    realm: realm_id,
                })
            },
            // Step 28. return success
            EvaluationResultBody::Success(remote_value) => {
                CallFunctionResult::Success(EvaluateResultSuccess {
                    result: remote_value,
                    realm: realm_id,
                })
            },
        });
        this.resume::<CommandHandled>(
            command_id,
            map.map_err(|e| e.into())
                .map(match function {
                    true => ScriptResult::CallFunction,
                    false => ScriptResult::Evaluate,
                })
                .map(ResultData::Script),
        );
    }
}

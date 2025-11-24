/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use serde::Serialize;
use serde_json::{Map, Value};

use crate::StreamId;
use crate::actor::{Actor, ActorError, ActorRegistry};
use crate::actors::object::ObjectActorMsg;
use crate::protocol::ClientRequest;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub enum EnvironmentType {
    Function,
    _Block,
    _Object,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub enum EnvironmentScope {
    Function,
    _Global,
}

#[derive(Serialize)]
struct EnvironmentBindings {
    arguments: Vec<Value>,
    variables: Map<String, Value>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct EnvironmentFunction {
    display_name: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EnvironmentActorMsg {
    actor: String,
    #[serde(rename = "type")]
    type_: EnvironmentType,
    scope_kind: EnvironmentScope,
    #[serde(skip_serializing_if = "Option::is_none")]
    parent: Option<Box<EnvironmentActorMsg>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    bindings: Option<EnvironmentBindings>,
    /// Should be set if `type_` is `EnvironmentType::Function`
    #[serde(skip_serializing_if = "Option::is_none")]
    function: Option<EnvironmentFunction>,
    /// Should be set if `type_` is `EnvironmentType::Object`
    #[serde(skip_serializing_if = "Option::is_none")]
    object: Option<ObjectActorMsg>,
}

/// Resposible for listing the bindings in an environment and assigning new values to them.
/// Referenced by `FrameActor` when replying to `getEnvironment` messages.
/// <https://searchfox.org/firefox-main/source/devtools/server/actors/environment.js>
pub struct EnvironmentActor {
    pub name: String,
    pub parent: Option<String>,
}

impl Actor for EnvironmentActor {
    fn name(&self) -> String {
        self.name.clone()
    }

    fn handle_message(
        &self,
        _request: ClientRequest,
        _registry: &ActorRegistry,
        _msg_type: &str,
        _msg: &Map<String, Value>,
        _id: StreamId,
    ) -> Result<(), ActorError> {
        // TODO: Handle messages.
        Err(ActorError::UnrecognizedPacketType)
    }
}

pub trait EnvironmentToProtocol {
    fn encode(&self, actors: &ActorRegistry) -> EnvironmentActorMsg;
}

impl EnvironmentToProtocol for EnvironmentActor {
    fn encode(&self, actors: &ActorRegistry) -> EnvironmentActorMsg {
        let parent = self
            .parent
            .as_ref()
            .map(|p| actors.find::<EnvironmentActor>(p))
            .map(|p| Box::new(p.encode(actors)));
        // TODO: Change hardcoded values.
        EnvironmentActorMsg {
            actor: self.name(),
            type_: EnvironmentType::Function,
            scope_kind: EnvironmentScope::Function,
            parent,
            bindings: None,
            function: None,
            object: None,
        }
    }
}

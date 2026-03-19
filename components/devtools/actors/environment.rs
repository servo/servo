/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::collections::HashMap;

use devtools_traits::EnvironmentInfo;
use malloc_size_of_derive::MallocSizeOf;
use serde::Serialize;
use serde_json::Value;

use crate::actor::{Actor, ActorEncode, ActorRegistry};
use crate::actors::object::ObjectActorMsg;

#[derive(Serialize)]
struct EnvironmentBindings {
    arguments: Vec<Value>,
    variables: HashMap<String, EnvironmentVariableDesc>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct EnvironmentFunction {
    display_name: String,
}

#[derive(Serialize)]
struct EnvironmentVariableDesc {
    value: String,
    configurable: bool,
    enumerable: bool,
    writable: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct EnvironmentActorMsg {
    actor: String,
    #[serde(rename = "type")]
    type_: Option<String>,
    scope_kind: Option<String>,
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
#[derive(MallocSizeOf)]
pub(crate) struct EnvironmentActor {
    name: String,
    environment: EnvironmentInfo,
    parent: Option<String>,
}

impl Actor for EnvironmentActor {
    fn name(&self) -> String {
        self.name.clone()
    }
}

impl EnvironmentActor {
    pub fn register(
        registry: &ActorRegistry,
        environment: EnvironmentInfo,
        parent: Option<String>,
    ) -> String {
        let name = registry.new_name::<Self>();
        let actor = Self {
            name: name.clone(),
            parent,
            environment,
        };
        registry.register(actor);
        name
    }
}

impl ActorEncode<EnvironmentActorMsg> for EnvironmentActor {
    fn encode(&self, registry: &ActorRegistry) -> EnvironmentActorMsg {
        let parent = self
            .parent
            .as_ref()
            .map(|p| registry.find::<EnvironmentActor>(p))
            .map(|p| Box::new(p.encode(registry)));
        // TODO: Change hardcoded values.
        EnvironmentActorMsg {
            actor: self.name(),
            type_: self.environment.type_.clone(),
            scope_kind: self.environment.scope_kind.clone(),
            parent,
            function: self
                .environment
                .function_display_name
                .clone()
                .map(|display_name| EnvironmentFunction { display_name }),
            object: None,
            bindings: Some(EnvironmentBindings {
                arguments: [].to_vec(),
                variables: self
                    .environment
                    .binding_variables
                    .clone()
                    .into_iter()
                    .map(|(key, value)| {
                        (
                            key,
                            EnvironmentVariableDesc {
                                value,
                                configurable: false,
                                enumerable: true,
                                writable: false,
                            },
                        )
                    })
                    .collect(),
            }),
        }
    }
}

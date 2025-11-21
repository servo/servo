/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use serde::Serialize;
use serde_json::{Map, Value};

use crate::StreamId;
use crate::actor::{Actor, ActorError, ActorRegistry};
use crate::protocol::ClientRequest;

#[derive(Serialize)]
pub struct ObjectPreview {
    kind: String,
    url: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ObjectActorMsg {
    actor: String,
    #[serde(rename = "type")]
    type_: String,
    class: String,
    own_property_length: i32,
    extensible: bool,
    frozen: bool,
    sealed: bool,
    is_error: bool,
    preview: ObjectPreview,
}

pub struct ObjectActor {
    pub name: String,
    pub _uuid: String,
}

impl Actor for ObjectActor {
    fn name(&self) -> String {
        self.name.clone()
    }
    fn handle_message(
        &self,
        _request: ClientRequest,
        _: &ActorRegistry,
        _: &str,
        _: &Map<String, Value>,
        _: StreamId,
    ) -> Result<(), ActorError> {
        // TODO: Handle enumSymbols for console object inspection
        Err(ActorError::UnrecognizedPacketType)
    }
}

impl ObjectActor {
    pub fn register(registry: &ActorRegistry, uuid: String) -> String {
        if !registry.script_actor_registered(uuid.clone()) {
            let name = registry.new_name("object");
            let actor = ObjectActor {
                name: name.clone(),
                _uuid: uuid.clone(),
            };

            registry.register_script_actor(uuid, name.clone());
            registry.register_later(Box::new(actor));

            name
        } else {
            registry.script_to_actor(uuid)
        }
    }
}

// TODO: Remove once the message is used.
#[allow(dead_code)]
pub trait ObjectToProtocol {
    fn encode(self) -> ObjectActorMsg;
}

impl ObjectToProtocol for ObjectActor {
    fn encode(self) -> ObjectActorMsg {
        // TODO: Review hardcoded values here
        ObjectActorMsg {
            actor: self.name,
            type_: "object".into(),
            class: "Window".into(),
            own_property_length: 0,
            extensible: true,
            frozen: false,
            sealed: false,
            is_error: false,
            preview: ObjectPreview {
                kind: "ObjectWithURL".into(),
                url: "".into(),
            },
        }
    }
}

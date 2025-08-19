/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use serde_json::{Map, Value, json};
use serde::Serialize;

use crate::StreamId;
use crate::actor::{Actor, ActorError, ActorRegistry};
use crate::protocol::ClientRequest;

#[derive(Serialize, Clone, Debug)]
pub struct ObjectActor {
    pub name: String,
    pub _uuid: String,
    pub class: String,
    pub extensible: bool,
    pub frozen: bool,
    pub sealed: bool,
    pub is_error: bool,
    pub own_property_length: u32,
    pub type_: String,
}


// https://searchfox.org/mozilla-central/source/devtools/client/fronts/object.js#1
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
                
                // Default values for Firefox protocol fields
                class: "Object".to_owned(),
                extensible: true,
                frozen: false,
                sealed: false,
                is_error: false,
                own_property_length: 0,
                type_: "object".to_owned(),
            };

            registry.register_script_actor(uuid, name.clone());
            registry.register_later(Box::new(actor));

            name
        } else {
            registry.script_to_actor(uuid)
        }
    }
}

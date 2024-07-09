/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::net::TcpStream;

use serde_json::{Map, Value};

use crate::actor::{Actor, ActorMessageStatus, ActorRegistry};
use crate::StreamId;

pub struct ObjectActor {
    pub name: String,
    pub uuid: String,
}

impl Actor for ObjectActor {
    fn name(&self) -> String {
        self.name.clone()
    }
    fn handle_message(
        &self,
        _: &ActorRegistry,
        _: &str,
        _: &Map<String, Value>,
        _: &mut TcpStream,
        _: StreamId,
    ) -> Result<ActorMessageStatus, ()> {
        // TODO: Handle enumSymbols for console object inspection
        Ok(ActorMessageStatus::Ignored)
    }
}

impl ObjectActor {
    pub fn register(registry: &ActorRegistry, uuid: String) -> String {
        if !registry.script_actor_registered(uuid.clone()) {
            let name = registry.new_name("object");
            let actor = ObjectActor {
                name: name.clone(),
                uuid: uuid.clone(),
            };

            registry.register_script_actor(uuid, name.clone());
            registry.register_later(Box::new(actor));

            name
        } else {
            registry.script_to_actor(uuid)
        }
    }
}

/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use actor::{Actor, ActorRegistry, ActorMessageStatus};
use rustc_serialize::json;
use std::net::TcpStream;

pub struct ObjectActor {
    pub name: String,
    pub uuid: String,
}

impl Actor for ObjectActor {
    fn name(&self) -> String {
        self.name.clone()
    }
    fn handle_message(&self,
                      _: &ActorRegistry,
                      _: &str,
                      _: &json::Object,
                      _: &mut TcpStream) -> Result<ActorMessageStatus, ()> {
        Ok(ActorMessageStatus::Ignored)
    }
}

impl ObjectActor {
    pub fn new(registry: &ActorRegistry, uuid: String) -> String {
        if !registry.script_actor_registered(uuid.clone()) {
            let name = registry.new_name("object");
            let actor = ObjectActor {
                name: name.clone(),
                uuid: uuid.clone(),
            };

            registry.register_script_actor(uuid, name.clone());
            registry.register_later(box actor);

            name
        } else {
            registry.script_to_actor(uuid)
        }
    }
}

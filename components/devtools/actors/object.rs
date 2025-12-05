/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use serde::Serialize;

use crate::actor::{Actor, ActorEncode, ActorRegistry};

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

    // TODO: Handle messages
    // https://searchfox.org/firefox-main/source/devtools/shared/specs/object.js
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
            registry.register_later(actor);

            name
        } else {
            registry.script_to_actor(uuid)
        }
    }
}

impl ActorEncode<ObjectActorMsg> for ObjectActor {
    fn encode(&self, _: &ActorRegistry) -> ObjectActorMsg {
        // TODO: Review hardcoded values here
        ObjectActorMsg {
            actor: self.name(),
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

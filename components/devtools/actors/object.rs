/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use serde::Serialize;
use serde_json::{Map, Value};

use crate::StreamId;
use crate::actor::{Actor, ActorEncode, ActorError, ActorRegistry};
use crate::protocol::ClientRequest;

#[derive(Serialize)]
pub(crate) struct ObjectPreview {
    kind: String,
    url: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
enum EnumIteratorType {
    PropertyIterator,
    SymbolIterator,
}

#[derive(Serialize)]
struct EnumIterator {
    actor: String,
    #[serde(rename = "type")]
    type_: EnumIteratorType,
    count: u32,
}

#[derive(Serialize)]
struct EnumReply {
    from: String,
    iterator: EnumIterator,
}

#[derive(Serialize)]
struct PrototypeReply {
    from: String,
    prototype: ObjectActorMsg,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ObjectActorMsg {
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

pub(crate) struct ObjectActor {
    name: String,
    _uuid: Option<String>,
}

impl Actor for ObjectActor {
    fn name(&self) -> String {
        self.name.clone()
    }

    // https://searchfox.org/firefox-main/source/devtools/shared/specs/object.js
    fn handle_message(
        &self,
        request: ClientRequest,
        registry: &ActorRegistry,
        msg_type: &str,
        _msg: &Map<String, Value>,
        _id: StreamId,
    ) -> Result<(), ActorError> {
        match msg_type {
            "enumProperties" => {
                let property_iterator = PropertyIteratorActor {
                    name: registry.new_name::<PropertyIteratorActor>(),
                };
                let msg = EnumReply {
                    from: self.name(),
                    iterator: EnumIterator {
                        actor: property_iterator.name(),
                        type_: EnumIteratorType::PropertyIterator,
                        count: 0,
                    },
                };
                registry.register(property_iterator);
                request.reply_final(&msg)?
            },

            "enumSymbols" => {
                let symbol_iterator = SymbolIteratorActor {
                    name: registry.new_name::<SymbolIteratorActor>(),
                };
                let msg = EnumReply {
                    from: self.name(),
                    iterator: EnumIterator {
                        actor: symbol_iterator.name(),
                        type_: EnumIteratorType::SymbolIterator,
                        count: 0,
                    },
                };
                registry.register(symbol_iterator);
                request.reply_final(&msg)?
            },

            "prototype" => {
                let msg = PrototypeReply {
                    from: self.name(),
                    prototype: self.encode(registry),
                };
                request.reply_final(&msg)?
            },

            _ => return Err(ActorError::UnrecognizedPacketType),
        };
        Ok(())
    }
}

impl ObjectActor {
    pub fn register(registry: &ActorRegistry, uuid: Option<String>) -> String {
        let Some(uuid) = uuid else {
            let name = registry.new_name::<Self>();
            let actor = ObjectActor {
                name: name.clone(),
                _uuid: None,
            };
            registry.register(actor);
            return name;
        };
        if !registry.script_actor_registered(uuid.clone()) {
            let name = registry.new_name::<Self>();
            let actor = ObjectActor {
                name: name.clone(),
                _uuid: Some(uuid.clone()),
            };

            registry.register_script_actor(uuid, name.clone());
            registry.register(actor);

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
                url: "".into(), // TODO: Use the correct url
            },
        }
    }
}

// TODO: Implement functionality of property and symbol iterators
struct PropertyIteratorActor {
    name: String,
}

impl Actor for PropertyIteratorActor {
    fn name(&self) -> String {
        self.name.clone()
    }
}

struct SymbolIteratorActor {
    name: String,
}

impl Actor for SymbolIteratorActor {
    fn name(&self) -> String {
        self.name.clone()
    }
}

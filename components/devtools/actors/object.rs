/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use devtools_traits::PropertyPreview;
use malloc_size_of_derive::MallocSizeOf;
use serde::Serialize;
use serde_json::{Map, Number, Value};

use crate::StreamId;
use crate::actor::{Actor, ActorEncode, ActorError, ActorRegistry};
use crate::actors::property_iterator::PropertyIteratorActor;
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

#[derive(Serialize)]
pub(crate) struct PropertyDescriptor {
    pub configurable: bool,
    pub enumerable: bool,
    pub writable: bool,
    pub value: Value,
}

impl From<&PropertyPreview> for PropertyDescriptor {
    fn from(prop: &PropertyPreview) -> Self {
        Self {
            configurable: prop.configurable,
            enumerable: prop.enumerable,
            writable: prop.writable,
            value: property_value_to_json(prop),
        }
    }
}

/// <https://searchfox.org/mozilla-central/source/devtools/server/actors/object/utils.js#148>
fn property_value_to_json(prop: &PropertyPreview) -> Value {
    match prop.value_type.as_str() {
        "undefined" => {
            let mut v = Map::new();
            v.insert("type".to_owned(), Value::String("undefined".to_owned()));
            Value::Object(v)
        },
        "null" => Value::Null,
        "boolean" => Value::Bool(prop.boolean_value.unwrap_or(false)),
        "number" => {
            let num = prop.number_value.unwrap_or(0.0);
            if num.is_nan() {
                let mut v = Map::new();
                v.insert("type".to_owned(), Value::String("NaN".to_owned()));
                Value::Object(v)
            } else if num.is_infinite() {
                let mut v = Map::new();
                let type_str = if num.is_sign_positive() {
                    "Infinity"
                } else {
                    "-Infinity"
                };
                v.insert("type".to_owned(), Value::String(type_str.to_owned()));
                Value::Object(v)
            } else {
                Value::Number(Number::from_f64(num).unwrap_or(Number::from(0)))
            }
        },
        "string" => Value::String(prop.string_value.clone().unwrap_or_default()),
        "object" => {
            let mut v = Map::new();
            v.insert("type".to_owned(), Value::String("object".to_owned()));
            if let Some(ref obj_class) = prop.object_class {
                v.insert("class".to_owned(), Value::String(obj_class.clone()));
            }
            if let Some(ref obj_name) = prop.value_name {
                v.insert("name".to_owned(), Value::String(obj_name.clone()));
            }
            Value::Object(v)
        },
        _ => Value::Null,
    }
}

#[derive(MallocSizeOf)]
pub(crate) struct ObjectActor {
    name: String,
    _uuid: Option<String>,
    class: String,
    properties: Vec<PropertyPreview>,
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
                let property_iterator_name =
                    PropertyIteratorActor::register(registry, self.properties.clone());
                let property_iterator =
                    registry.find::<PropertyIteratorActor>(&property_iterator_name);
                let count = property_iterator.count();
                let msg = EnumReply {
                    from: self.name(),
                    iterator: EnumIterator {
                        actor: property_iterator_name,
                        type_: EnumIteratorType::PropertyIterator,
                        count,
                    },
                };

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
    pub fn register(registry: &ActorRegistry, uuid: Option<String>, class: String) -> String {
        Self::register_with_properties(registry, uuid, class, Vec::new())
    }

    pub fn register_with_properties(
        registry: &ActorRegistry,
        uuid: Option<String>,
        class: String,
        properties: Vec<PropertyPreview>,
    ) -> String {
        let Some(uuid) = uuid else {
            let name = registry.new_name::<Self>();
            let actor = ObjectActor {
                name: name.clone(),
                _uuid: None,
                class,
                properties,
            };
            registry.register(actor);
            return name;
        };
        if !registry.script_actor_registered(uuid.clone()) {
            let name = registry.new_name::<Self>();
            let actor = ObjectActor {
                name: name.clone(),
                _uuid: Some(uuid.clone()),
                class,
                properties,
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
        ObjectActorMsg {
            actor: self.name(),
            type_: "object".into(),
            class: self.class.clone(),
            own_property_length: self.properties.len() as i32,
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

#[derive(MallocSizeOf)]
struct SymbolIteratorActor {
    name: String,
}

impl Actor for SymbolIteratorActor {
    fn name(&self) -> String {
        self.name.clone()
    }
}

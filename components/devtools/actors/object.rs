/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use devtools_traits::{DebuggerValue, ObjectPreview, PropertyDescriptor};
use malloc_size_of_derive::MallocSizeOf;
use serde::Serialize;
use serde_json::{Map, Number, Value};

use crate::StreamId;
use crate::actor::{Actor, ActorEncode, ActorError, ActorRegistry};
use crate::actors::property_iterator::PropertyIteratorActor;
use crate::protocol::ClientRequest;

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
    prototype: Value,
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
    #[serde(skip_serializing_if = "Option::is_none")]
    preview: Option<ObjectPreview>,
}

#[derive(Serialize)]
pub(crate) struct ObjectPropertyDescriptor {
    pub configurable: bool,
    pub enumerable: bool,
    pub writable: bool,
    pub value: Value,
}

impl ObjectPropertyDescriptor {
    pub(crate) fn from_property_descriptor(
        registry: &ActorRegistry,
        prop: &PropertyDescriptor,
    ) -> Self {
        Self {
            configurable: prop.configurable,
            enumerable: prop.enumerable,
            writable: prop.writable,
            value: debugger_value_to_json(registry, prop.value.clone()),
        }
    }
}

/// <https://searchfox.org/mozilla-central/source/devtools/server/actors/object/utils.js#148>
pub(crate) fn debugger_value_to_json(registry: &ActorRegistry, value: DebuggerValue) -> Value {
    let mut v = Map::new();
    match value {
        DebuggerValue::VoidValue => {
            v.insert("type".to_owned(), Value::String("undefined".to_owned()));
            Value::Object(v)
        },
        DebuggerValue::NullValue => {
            v.insert("type".to_owned(), Value::String("null".to_owned()));
            Value::Object(v)
        },
        DebuggerValue::BooleanValue(boolean) => Value::Bool(boolean),
        DebuggerValue::NumberValue(val) => {
            if val.is_nan() {
                v.insert("type".to_owned(), Value::String("NaN".to_owned()));
                Value::Object(v)
            } else if val.is_infinite() {
                if val < 0. {
                    v.insert("type".to_owned(), Value::String("-Infinity".to_owned()));
                } else {
                    v.insert("type".to_owned(), Value::String("Infinity".to_owned()));
                }
                Value::Object(v)
            } else if val == 0. && val.is_sign_negative() {
                v.insert("type".to_owned(), Value::String("-0".to_owned()));
                Value::Object(v)
            } else {
                Value::Number(Number::from_f64(val).unwrap())
            }
        },
        DebuggerValue::StringValue(str) => Value::String(str),
        DebuggerValue::ObjectValue {
            uuid,
            class,
            preview,
            ..
        } => {
            let object_name = ObjectActor::register(registry, Some(uuid), class, preview);
            let object_msg = registry.encode::<ObjectActor, _>(&object_name);
            let value = serde_json::to_value(object_msg).unwrap_or_default();
            Value::Object(value.as_object().cloned().unwrap_or_default())
        },
    }
}

#[derive(MallocSizeOf)]
pub(crate) struct ObjectActor {
    name: String,
    _uuid: Option<String>,
    class: String,
    preview: Option<ObjectPreview>,
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
                let properties = self
                    .preview
                    .as_ref()
                    .and_then(|preview| preview.own_properties.clone())
                    .unwrap_or_default();
                let property_iterator_name = PropertyIteratorActor::register(registry, properties);
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
                let symbol_iterator_actor = SymbolIteratorActor {
                    name: registry.new_name::<SymbolIteratorActor>(),
                };
                let msg = EnumReply {
                    from: self.name(),
                    iterator: EnumIterator {
                        actor: symbol_iterator_actor.name(),
                        type_: EnumIteratorType::SymbolIterator,
                        count: 0,
                    },
                };
                registry.register(symbol_iterator_actor);
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
    pub fn register(
        registry: &ActorRegistry,
        uuid: Option<String>,
        class: String,
        preview: Option<ObjectPreview>,
    ) -> String {
        let Some(uuid) = uuid else {
            let name = registry.new_name::<Self>();
            let actor = ObjectActor {
                name: name.clone(),
                _uuid: None,
                class,
                preview,
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
                preview,
            };

            registry.register_script_actor(uuid, name.clone());
            registry.register(actor);

            name
        } else {
            registry.script_to_actor(uuid)
        }
    }
}

impl ActorEncode<Value> for ObjectActor {
    fn encode(&self, registry: &ActorRegistry) -> Value {
        // TODO: convert to a serialize struct instead
        let mut m = Map::new();
        m.insert("type".to_owned(), Value::String("object".to_owned()));
        m.insert("class".to_owned(), Value::String(self.class.clone()));
        m.insert("actor".to_owned(), Value::String(self.name()));
        m.insert("extensible".to_owned(), Value::Bool(true));
        m.insert("frozen".to_owned(), Value::Bool(false));
        m.insert("sealed".to_owned(), Value::Bool(false));

        // Build preview
        // <https://searchfox.org/firefox-main/source/devtools/server/actors/object/previewers.js#849>
        let Some(preview) = self.preview.clone() else {
            return Value::Object(m);
        };
        let mut preview_map = Map::new();

        if preview.kind == "ArrayLike" {
            if let Some(length) = preview.array_length {
                preview_map.insert("length".to_owned(), Value::Number(length.into()));
            }
        } else {
            if let Some(ref props) = preview.own_properties {
                let mut own_props_map = Map::new();
                for prop in props {
                    let descriptor = serde_json::to_value(
                        ObjectPropertyDescriptor::from_property_descriptor(registry, prop),
                    )
                    .unwrap();
                    own_props_map.insert(prop.name.clone(), descriptor);
                }
                preview_map.insert("ownProperties".to_owned(), Value::Object(own_props_map));
            }

            if let Some(length) = preview.own_properties_length {
                preview_map.insert(
                    "ownPropertiesLength".to_owned(),
                    Value::Number(length.into()),
                );
                m.insert("ownPropertyLength".to_owned(), Value::Number(length.into()));
            }
        }
        preview_map.insert("kind".to_owned(), Value::String(preview.kind));

        // Function-specific metadata
        if let Some(function) = preview.function {
            if let Some(name) = function.name {
                m.insert("name".to_owned(), Value::String(name));
            }
            if let Some(display_name) = function.display_name {
                m.insert("displayName".to_owned(), Value::String(display_name));
            }
            m.insert(
                "parameterNames".to_owned(),
                Value::Array(
                    function
                        .parameter_names
                        .into_iter()
                        .map(Value::String)
                        .collect(),
                ),
            );
            m.insert("isAsync".to_owned(), Value::Bool(function.is_async));
            m.insert("isGenerator".to_owned(), Value::Bool(function.is_generator));
        }

        m.insert("preview".to_owned(), Value::Object(preview_map));

        Value::Object(m)
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

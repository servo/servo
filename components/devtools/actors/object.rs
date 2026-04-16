/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::collections::HashMap;

use devtools_traits::{DebuggerValue, PropertyDescriptor};
use malloc_size_of_derive::MallocSizeOf;
use serde::Serialize;
use serde_json::{Map, Value};

use crate::actor::{Actor, ActorEncode, ActorError, ActorRegistry};
use crate::actors::property_iterator::PropertyIteratorActor;
use crate::actors::symbol_iterator::SymbolIteratorActor;
use crate::protocol::ClientRequest;
use crate::{StreamId, debugger_value_to_json};

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
pub(crate) struct ObjectPreview {
    pub kind: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub own_properties: Option<HashMap<String, ObjectPropertyDescriptor>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub own_properties_length: Option<u32>,
    #[serde(flatten)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub function: Option<FunctionPreview>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub length: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub items: Option<Vec<Value>>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FunctionPreview {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
    pub parameter_names: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_async: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_generator: Option<bool>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ObjectActorMsg {
    actor: String,
    #[serde(rename = "type")]
    type_: String,
    class: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    own_property_length: Option<u32>,
    extensible: bool,
    frozen: bool,
    sealed: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    preview: Option<ObjectPreview>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ObjectPropertyDescriptor {
    pub value: Value,
    pub configurable: bool,
    pub enumerable: bool,
    pub writable: bool,
    pub is_accessor: bool,
}

impl ObjectPropertyDescriptor {
    pub(crate) fn from_property_descriptor(
        registry: &ActorRegistry,
        prop: &PropertyDescriptor,
    ) -> Self {
        Self {
            value: debugger_value_to_json(registry, prop.value.clone()),
            configurable: prop.configurable,
            enumerable: prop.enumerable,
            writable: prop.writable,
            is_accessor: prop.is_accessor,
        }
    }
}

#[derive(MallocSizeOf)]
pub(crate) struct ObjectActor {
    name: String,
    _uuid: Option<String>,
    class: String,
    preview: Option<devtools_traits::ObjectPreview>,
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
                let properties = self.preview.as_ref().map_or_else(Vec::new, |preview| {
                    if preview.kind == "ArrayLike" {
                        // For arrays, convert items to indexed properties
                        // <https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Object/getOwnPropertyDescriptor#description>
                        let mut props: Vec<PropertyDescriptor> = preview
                            .items
                            .as_ref()
                            .map(|items| {
                                items
                                    .iter()
                                    .enumerate()
                                    .map(|(index, value)| PropertyDescriptor {
                                        name: index.to_string(),
                                        value: value.clone(),
                                        configurable: true,
                                        enumerable: true,
                                        writable: true,
                                        is_accessor: false,
                                    })
                                    .collect()
                            })
                            .unwrap_or_default();
                        // Add length property
                        if let Some(length) = preview.array_length {
                            // <https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Array/length#value>
                            props.push(PropertyDescriptor {
                                name: "length".to_string(),
                                value: DebuggerValue::NumberValue(length as f64),
                                configurable: false,
                                enumerable: false,
                                writable: true,
                                is_accessor: false,
                            });
                        }
                        props
                    } else {
                        preview.own_properties.clone().unwrap_or_default()
                    }
                });
                let property_iterator_name = PropertyIteratorActor::register(registry, properties);
                let property_iterator_actor =
                    registry.find::<PropertyIteratorActor>(&property_iterator_name);
                let count = property_iterator_actor.count();
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
                let symbol_iterator_name = SymbolIteratorActor::register(registry);
                let msg = EnumReply {
                    from: self.name(),
                    iterator: EnumIterator {
                        actor: symbol_iterator_name,
                        type_: EnumIteratorType::SymbolIterator,
                        count: 0,
                    },
                };
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
        preview: Option<devtools_traits::ObjectPreview>,
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

impl ActorEncode<ObjectActorMsg> for ObjectActor {
    fn encode(&self, registry: &ActorRegistry) -> ObjectActorMsg {
        let mut msg = ObjectActorMsg {
            actor: self.name(),
            type_: "object".into(),
            class: self.class.clone(),
            extensible: true,
            frozen: false,
            sealed: false,
            preview: None,
            own_property_length: None,
        };

        // Build preview
        // <https://searchfox.org/firefox-main/source/devtools/server/actors/object/previewers.js#849>
        let Some(preview) = self.preview.clone() else {
            return msg;
        };
        msg.own_property_length = preview.own_properties_length;

        let function = preview.function.map(|function| FunctionPreview {
            name: function.name.clone(),
            display_name: function.display_name.clone(),
            parameter_names: function.parameter_names.clone(),
            is_async: function.is_async,
            is_generator: function.is_generator,
        });

        let preview = ObjectPreview {
            kind: preview.kind.clone(),
            own_properties: preview.own_properties.map(|own_properties| {
                own_properties
                    .iter()
                    .map(|prop| {
                        (
                            prop.name.clone(),
                            ObjectPropertyDescriptor::from_property_descriptor(registry, prop),
                        )
                    })
                    .collect()
            }),
            own_properties_length: preview.own_properties_length,
            function,
            length: preview.array_length,
            items: preview.items.map(|items| {
                items
                    .iter()
                    .map(|item| debugger_value_to_json(registry, item.clone()))
                    .collect()
            }),
        };

        msg.preview = Some(preview);
        msg
    }
}

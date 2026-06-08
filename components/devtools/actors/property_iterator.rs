/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::collections::HashMap;
use std::sync::Arc;

use devtools_traits::{DebuggerValue, PropertyDescriptor};
use malloc_size_of_derive::MallocSizeOf;
use serde::Serialize;
use serde_json::{Map, Value};

use crate::actor::{Actor, ActorError, ActorRegistry, new_actor_name};
use crate::actors::object::ObjectPropertyDescriptor;
use crate::protocol::ClientRequest;
use crate::{StreamId, debugger_value_to_json};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SliceReply {
    from: String,
    own_properties: HashMap<String, ObjectPropertyDescriptor>,
}

#[derive(Serialize)]
struct MapEntryPreview {
    key: Value,
    value: Value,
}

#[derive(Serialize)]
struct MapEntryGrip {
    #[serde(rename = "type")]
    type_: &'static str,
    preview: MapEntryPreview,
}

impl ObjectPropertyDescriptor {
    fn from_map_entry(
        registry: &ActorRegistry,
        key: &DebuggerValue,
        value: &DebuggerValue,
    ) -> Self {
        Self {
            value: serde_json::to_value(MapEntryGrip {
                type_: "mapEntry",
                preview: MapEntryPreview {
                    key: debugger_value_to_json(registry, key.clone()),
                    value: debugger_value_to_json(registry, value.clone()),
                },
            })
            .unwrap_or_default(),
            configurable: None,
            enumerable: true,
            writable: None,
            is_accessor: None,
        }
    }
}

#[derive(MallocSizeOf)]
pub(crate) enum PropertyIteratorEntry {
    Property(PropertyDescriptor),
    MapEntry(DebuggerValue, DebuggerValue),
}

#[derive(MallocSizeOf)]
pub(crate) struct PropertyIteratorActor {
    name: String,
    entries: Vec<PropertyIteratorEntry>,
}

impl PropertyIteratorActor {
    pub fn register(registry: &ActorRegistry, entries: Vec<PropertyIteratorEntry>) -> Arc<Self> {
        let name = new_actor_name::<Self>();
        let actor = Self { name, entries };
        registry.register::<Self>(actor)
    }

    pub fn count(&self) -> u32 {
        self.entries.len() as u32
    }
}

impl Actor for PropertyIteratorActor {
    fn name(&self) -> &str {
        &self.name
    }

    fn handle_message(
        &self,
        request: ClientRequest,
        registry: &ActorRegistry,
        msg_type: &str,
        msg: &Map<String, Value>,
        _id: StreamId,
    ) -> Result<(), ActorError> {
        match msg_type {
            "slice" => {
                let start = msg.get("start").and_then(|v| v.as_u64()).unwrap_or(0) as usize;
                let count = msg
                    .get("count")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(self.entries.len() as u64) as usize;

                let mut own_properties = HashMap::new();
                for (index, entry) in self.entries.iter().enumerate().skip(start).take(count) {
                    match entry {
                        PropertyIteratorEntry::Property(prop) => {
                            own_properties.insert(
                                prop.name.clone(),
                                ObjectPropertyDescriptor::from_property_descriptor(registry, prop),
                            );
                        },
                        PropertyIteratorEntry::MapEntry(key, value) => {
                            own_properties.insert(
                                index.to_string(),
                                ObjectPropertyDescriptor::from_map_entry(registry, key, value),
                            );
                        },
                    }
                }

                let reply = SliceReply {
                    from: self.name().into(),
                    own_properties,
                };
                request.reply_final(&reply)?
            },
            _ => return Err(ActorError::UnrecognizedPacketType),
        }
        Ok(())
    }
}

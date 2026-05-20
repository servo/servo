/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::collections::HashMap;
use std::sync::Arc;

use devtools_traits::PropertyDescriptor;
use malloc_size_of_derive::MallocSizeOf;
use serde::Serialize;
use serde_json::{Map, Value};

use crate::StreamId;
use crate::actor::{Actor, ActorError, ActorRegistry, new_actor_name};
use crate::actors::object::ObjectPropertyDescriptor;
use crate::protocol::ClientRequest;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SliceReply {
    from: String,
    own_properties: HashMap<String, ObjectPropertyDescriptor>,
}

#[derive(MallocSizeOf)]
pub(crate) struct PropertyIteratorActor {
    name: String,
    properties: Vec<PropertyDescriptor>,
}

impl PropertyIteratorActor {
    pub fn register(registry: &ActorRegistry, properties: Vec<PropertyDescriptor>) -> Arc<Self> {
        let name = new_actor_name::<Self>();
        let actor = Self { name, properties };
        registry.register::<Self>(actor)
    }

    pub fn count(&self) -> u32 {
        self.properties.len() as u32
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
                    .unwrap_or(self.properties.len() as u64) as usize;

                let mut own_properties = HashMap::new();
                for prop in self.properties.iter().skip(start).take(count) {
                    own_properties.insert(
                        prop.name.clone(),
                        ObjectPropertyDescriptor::from_property_descriptor(registry, prop),
                    );
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

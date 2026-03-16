/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// TODO: Remove once the actor is used
#![allow(dead_code)]

use std::collections::HashMap;

use malloc_size_of_derive::MallocSizeOf;
use serde::Serialize;
use serde_json::{Map, Value};

use crate::StreamId;
use crate::actor::{Actor, ActorError, ActorRegistry};
use crate::protocol::ClientRequest;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SliceReply {
    from: String,
    own_properties: HashMap<String, Value>,
}

#[derive(MallocSizeOf)]
pub struct PropertyIteratorActor {
    name: String,
}

impl PropertyIteratorActor {
    pub fn new(name: String) -> Self {
        Self { name }
    }
}

// <https://searchfox.org/firefox-main/source/devtools/server/actors/object/property-iterator.js>
impl Actor for PropertyIteratorActor {
    fn name(&self) -> String {
        self.name.clone()
    }

    fn handle_message(
        &self,
        request: ClientRequest,
        _registry: &ActorRegistry,
        msg_type: &str,
        _msg: &Map<String, Value>,
        _id: StreamId,
    ) -> Result<(), ActorError> {
        match msg_type {
            "slice" => {
                // TODO: Return actual properties based on start/count from msg
                let reply = SliceReply {
                    from: self.name(),
                    own_properties: HashMap::new(),
                };
                request.reply_final(&reply)?
            },
            _ => return Err(ActorError::UnrecognizedPacketType),
        }
        Ok(())
    }
}

/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
use serde::Serialize;
use serde_json::{Map, Value};

use crate::StreamId;
use crate::actor::{Actor, ActorError, ActorRegistry};
use crate::protocol::ClientRequest;

const INITIAL_LENGTH: usize = 500;

pub struct LongStringActor {
    name: String,
    full_string: String,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LongStringObj {
    #[serde(rename = "type")]
    type_: String,
    actor: String,
    length: usize,
    initial: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SubstringReply {
    from: String,
    substring: String,
}

impl Actor for LongStringActor {
    fn name(&self) -> String {
        self.name.clone()
    }

    fn handle_message(
        &self,
        request: ClientRequest,
        _registry: &ActorRegistry,
        msg_type: &str,
        msg: &Map<String, Value>,
        _id: StreamId,
    ) -> Result<(), ActorError> {
        match msg_type {
            "substring" => {
                let start = msg.get("start").and_then(|v| v.as_u64()).unwrap_or(0) as usize;
                let end = msg
                    .get("end")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(self.full_string.len() as u64) as usize;
                let substring: String = self
                    .full_string
                    .chars()
                    .skip(start)
                    .take(end - start)
                    .collect();
                let reply = SubstringReply {
                    from: self.name(),
                    substring,
                };
                request.reply_final(&reply)?
            },
            _ => return Err(ActorError::UnrecognizedPacketType),
        }
        Ok(())
    }
}

impl LongStringActor {
    pub fn new(registry: &ActorRegistry, full_string: String) -> Self {
        let name = registry.new_name("longStringActor");
        LongStringActor { name, full_string }
    }

    pub fn long_string_obj(&self) -> LongStringObj {
        LongStringObj {
            type_: "longString".to_string(),
            actor: self.name.clone(),
            length: self.full_string.len(),
            initial: self.full_string.chars().take(INITIAL_LENGTH).collect(),
        }
    }
}

/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::net::TcpStream;

use serde::Serialize;
use serde_json::{Map, Value};

use crate::actor::{Actor, ActorMessageStatus, ActorRegistry};
use crate::protocol::JsonPacketStream;
use crate::StreamId;

#[derive(Serialize)]
struct GetStyleSheetsReply {
    from: String,
    styleSheets: Vec<u32>, // TODO: real JSON structure.
}

pub struct StyleSheetsActor {
    pub name: String,
}

impl Actor for StyleSheetsActor {
    fn name(&self) -> String {
        self.name.clone()
    }
    fn handle_message(
        &self,
        _registry: &ActorRegistry,
        msg_type: &str,
        _msg: &Map<String, Value>,
        stream: &mut TcpStream,
        _id: StreamId,
    ) -> Result<ActorMessageStatus, ()> {
        Ok(match msg_type {
            "getStyleSheets" => {
                let msg = GetStyleSheetsReply {
                    from: self.name(),
                    styleSheets: vec![],
                };
                let _ = stream.write_json_packet(&msg);
                ActorMessageStatus::Processed
            },

            _ => ActorMessageStatus::Ignored,
        })
    }
}

impl StyleSheetsActor {
    pub fn new(name: String) -> StyleSheetsActor {
        StyleSheetsActor { name: name }
    }
}

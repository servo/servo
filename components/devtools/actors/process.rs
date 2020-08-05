/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::actor::{Actor, ActorMessageStatus, ActorRegistry};
use crate::protocol::JsonPacketStream;
use crate::StreamId;
use serde_json::{Map, Value};
use std::net::TcpStream;

#[derive(Serialize)]
struct ListWorkersReply {
    from: String,
    workers: Vec<u32>, // TODO: use proper JSON structure.
}

pub struct ProcessActor {
    name: String,
}

impl ProcessActor {
    pub fn new(name: String) -> Self {
        Self { name }
    }
}

impl Actor for ProcessActor {
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
            "listWorkers" => {
                let reply = ListWorkersReply {
                    from: self.name(),
                    workers: vec![],
                };
                let _ = stream.write_json_packet(&reply);
                ActorMessageStatus::Processed
            },

            _ => ActorMessageStatus::Ignored,
        })
    }
}

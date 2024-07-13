/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::net::TcpStream;

use serde::Serialize;
use serde_json::{self, Map, Value};

use crate::actor::{Actor, ActorMessageStatus, ActorRegistry};
use crate::protocol::JsonPacketStream;
use crate::StreamId;

#[derive(Serialize)]
pub struct HighlighterMsg {
    pub actor: String,
}

pub struct HighlighterActor {
    pub name: String,
}

#[derive(Serialize)]
struct ShowReply {
    from: String,
    value: bool,
}

#[derive(Serialize)]
struct HideReply {
    from: String,
}

impl Actor for HighlighterActor {
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
            "show" => {
                let msg = ShowReply {
                    from: self.name(),
                    value: true,
                };
                let _ = stream.write_json_packet(&msg);
                ActorMessageStatus::Processed
            },

            "hide" => {
                let msg = HideReply { from: self.name() };
                let _ = stream.write_json_packet(&msg);
                ActorMessageStatus::Processed
            },

            _ => ActorMessageStatus::Ignored,
        })
    }
}

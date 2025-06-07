/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use serde::Serialize;

use crate::EmptyReplyMsg;
use crate::actor::{Actor, ActorMessageStatus};
use crate::protocol::JsonPacketStream;

#[derive(Serialize)]
pub struct BreakpointListActorMsg {
    actor: String,
}

pub struct BreakpointListActor {
    name: String,
}

impl Actor for BreakpointListActor {
    fn name(&self) -> String {
        self.name.clone()
    }

    fn handle_message(
        &self,
        _registry: &crate::actor::ActorRegistry,
        msg_type: &str,
        _msg: &serde_json::Map<String, serde_json::Value>,
        stream: &mut std::net::TcpStream,
        _stream_id: crate::StreamId,
    ) -> Result<crate::actor::ActorMessageStatus, ()> {
        Ok(match msg_type {
            "setBreakpoint" => {
                let msg = EmptyReplyMsg { from: self.name() };
                let _ = stream.write_json_packet(&msg);

                ActorMessageStatus::Processed
            },
            "setActiveEventBreakpoints" => {
                let msg = EmptyReplyMsg { from: self.name() };
                let _ = stream.write_json_packet(&msg);

                ActorMessageStatus::Processed
            },
            _ => ActorMessageStatus::Ignored,
        })
    }
}

impl BreakpointListActor {
    pub fn new(name: String) -> Self {
        Self { name }
    }
}

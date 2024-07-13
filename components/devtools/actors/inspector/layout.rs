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
pub struct LayoutInspectorActorMsg {
    actor: String,
}

pub struct LayoutInspectorActor {
    name: String,
}

#[derive(Serialize)]
pub struct GetGridsReply {
    from: String,
    grids: Vec<String>,
}

#[derive(Serialize)]
pub struct GetCurrentFlexboxReply {
    from: String,
    flexbox: Option<()>,
}

impl Actor for LayoutInspectorActor {
    fn name(&self) -> String {
        self.name.clone()
    }

    /// The layout inspector actor can handle the following messages:
    ///
    /// -
    fn handle_message(
        &self,
        _registry: &ActorRegistry,
        msg_type: &str,
        _msg: &Map<String, Value>,
        stream: &mut TcpStream,
        _id: StreamId,
    ) -> Result<ActorMessageStatus, ()> {
        Ok(match msg_type {
            "getGrids" => {
                let msg = GetGridsReply {
                    from: self.name(),
                    grids: vec![],
                };
                let _ = stream.write_json_packet(&msg);
                ActorMessageStatus::Processed
            },
            "getCurrentFlexbox" => {
                let msg = GetCurrentFlexboxReply {
                    from: self.name(),
                    flexbox: None,
                };
                let _ = stream.write_json_packet(&msg);
                ActorMessageStatus::Processed
            },
            _ => ActorMessageStatus::Ignored,
        })
    }
}

impl LayoutInspectorActor {
    pub fn new(name: String) -> Self {
        Self { name }
    }

    pub fn encodable(&self) -> LayoutInspectorActorMsg {
        LayoutInspectorActorMsg { actor: self.name() }
    }
}

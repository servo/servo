/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::net::TcpStream;

use serde::Serialize;
use serde_json::{Map, Value};

use crate::actor::{Actor, ActorError, ActorRegistry};
use crate::protocol::{ActorReplied, JsonPacketStream};
use crate::{EmptyReplyMsg, StreamId};

#[derive(Serialize)]
pub struct NetworkParentActorMsg {
    actor: String,
}

pub struct NetworkParentActor {
    name: String,
}

impl Actor for NetworkParentActor {
    fn name(&self) -> String {
        self.name.clone()
    }

    /// The network parent actor can handle the following messages:
    ///
    /// - `setSaveRequestAndResponseBodies`: Doesn't do anything yet
    fn handle_message(
        &self,
        _registry: &ActorRegistry,
        msg_type: &str,
        _msg: &Map<String, Value>,
        stream: &mut TcpStream,
        _id: StreamId,
    ) -> Result<ActorReplied, ActorError> {
        Ok(match msg_type {
            "setSaveRequestAndResponseBodies" => {
                let msg = EmptyReplyMsg { from: self.name() };
                stream.write_json_packet(&msg)?
            },
            _ => return Err(ActorError::UnrecognizedPacketType),
        })
    }
}

impl NetworkParentActor {
    pub fn new(name: String) -> Self {
        Self { name }
    }

    pub fn encodable(&self) -> NetworkParentActorMsg {
        NetworkParentActorMsg { actor: self.name() }
    }
}

/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::actor::{Actor, ActorMessageStatus, ActorRegistry};
use crate::StreamId;
use serde_json::{Map, Value};
use std::net::TcpStream;

pub struct EmulationActor {
    pub name: String,
}

impl Actor for EmulationActor {
    fn name(&self) -> String {
        self.name.clone()
    }

    fn handle_message(
        &self,
        _registry: &ActorRegistry,
        msg_type: &str,
        _msg: &Map<String, Value>,
        _stream: &mut TcpStream,
        _id: StreamId,
    ) -> Result<ActorMessageStatus, ()> {
        Ok(match msg_type {
            _ => ActorMessageStatus::Ignored,
        })
    }
}

impl EmulationActor {
    pub fn new(name: String) -> EmulationActor {
        EmulationActor { name: name }
    }
}

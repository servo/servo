/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::net::TcpStream;

//use serde::Serialize;
use serde_json::{Map, Value};

use crate::actor::{Actor, ActorMessageStatus, ActorRegistry};
//use crate::protocol::JsonPacketStream;
use crate::StreamId;

pub struct AccesibilityActor {
    name: String,
}

//#[derive(Serialize)]
//pub struct AccesibilityActorMsg {
//    actor: String,
//}

impl Actor for AccesibilityActor {
    fn name(&self) -> String {
        self.name.clone()
    }

    /// The accesibility actor can handle the following messages:
    ///
    /// -
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

impl AccesibilityActor {
    pub fn new(name: String) -> Self {
        Self { name }
    }

    //pub fn encodable(&self) -> AccesibilityActorMsg {
    //    AccesibilityActorMsg { actor: self.name() }
    //}
}

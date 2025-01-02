/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! This actor is used for protocol purposes, it forwards the reflow events to clients.

use std::net::TcpStream;

use serde_json::{Map, Value};

use crate::actor::{Actor, ActorMessageStatus, ActorRegistry};
use crate::StreamId;

pub struct ReflowActor {
    name: String,
}

impl Actor for ReflowActor {
    fn name(&self) -> String {
        self.name.clone()
    }

    /// The reflow actor can handle the following messages:
    ///
    /// - `start`: Does nothing yet. This doesn't need a reply like other messages.
    fn handle_message(
        &self,
        _registry: &ActorRegistry,
        msg_type: &str,
        _msg: &Map<String, Value>,
        _stream: &mut TcpStream,
        _id: StreamId,
    ) -> Result<ActorMessageStatus, ()> {
        Ok(match msg_type {
            "start" => {
                // TODO: Create an observer on "reflows" events
                ActorMessageStatus::Processed
            },
            _ => ActorMessageStatus::Ignored,
        })
    }
}

impl ReflowActor {
    pub fn new(name: String) -> Self {
        Self { name }
    }
}

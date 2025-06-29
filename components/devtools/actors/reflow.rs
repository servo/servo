/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! This actor is used for protocol purposes, it forwards the reflow events to clients.

use serde_json::{Map, Value};

use crate::actor::{Actor, ActorError, ActorRegistry};
use crate::protocol::ClientRequest;
use crate::{EmptyReplyMsg, StreamId};

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
        request: ClientRequest,
        _registry: &ActorRegistry,
        msg_type: &str,
        _msg: &Map<String, Value>,
        _id: StreamId,
    ) -> Result<(), ActorError> {
        match msg_type {
            "start" => {
                // TODO: Create an observer on "reflows" events
                let msg = EmptyReplyMsg { from: self.name() };
                request.reply_final(&msg)?
            },
            _ => return Err(ActorError::UnrecognizedPacketType),
        };
        Ok(())
    }

    fn cleanup(&self, _id: StreamId) {}
}

impl ReflowActor {
    pub fn new(name: String) -> Self {
        Self { name }
    }
}

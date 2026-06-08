/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! This actor is used for protocol purposes, it forwards the reflow events to clients.

use std::sync::Arc;

use malloc_size_of_derive::MallocSizeOf;
use serde_json::{Map, Value};

use crate::actor::{Actor, ActorError, ActorRegistry, new_actor_name};
use crate::protocol::ClientRequest;
use crate::{EmptyReplyMsg, StreamId};

#[derive(MallocSizeOf)]
pub(crate) struct ReflowActor {
    name: String,
}

impl Actor for ReflowActor {
    fn name(&self) -> &str {
        &self.name
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
                let msg = EmptyReplyMsg {
                    from: self.name().into(),
                };
                request.reply_final(&msg)?
            },
            _ => return Err(ActorError::UnrecognizedPacketType),
        };
        Ok(())
    }
}

impl ReflowActor {
    pub fn register(registry: &ActorRegistry) -> Arc<Self> {
        let name = new_actor_name::<Self>();
        let actor = Self { name };
        registry.register::<Self>(actor)
    }
}

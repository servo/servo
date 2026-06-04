/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::sync::Arc;

use malloc_size_of_derive::MallocSizeOf;
use serde_json::{Map, Value};

use crate::actor::{Actor, ActorEncode, ActorError, ActorRegistry, new_actor_name};
use crate::protocol::ClientRequest;
use crate::{ActorMsg, EmptyReplyMsg, StreamId};

#[derive(MallocSizeOf)]
pub(crate) struct NetworkParentActor {
    name: String,
}

impl Actor for NetworkParentActor {
    fn name(&self) -> &str {
        &self.name
    }

    /// The network parent actor can handle the following messages:
    ///
    /// - `setSaveRequestAndResponseBodies`: Doesn't do anything yet
    fn handle_message(
        &self,
        request: ClientRequest,
        _registry: &ActorRegistry,
        msg_type: &str,
        _msg: &Map<String, Value>,
        _id: StreamId,
    ) -> Result<(), ActorError> {
        match msg_type {
            "setSaveRequestAndResponseBodies" => {
                let msg = EmptyReplyMsg {
                    from: self.name().into(),
                };
                request.reply_final(&msg)?
            },
            "setPersist" => {
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

impl NetworkParentActor {
    pub fn register(registry: &ActorRegistry) -> Arc<Self> {
        let name = new_actor_name::<Self>();
        let actor = Self { name };
        registry.register::<Self>(actor)
    }
}

impl ActorEncode<ActorMsg> for NetworkParentActor {
    fn encode(&self, _: &ActorRegistry) -> ActorMsg {
        ActorMsg {
            actor: self.name().into(),
        }
    }
}

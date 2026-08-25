/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Liberally derived from <https://searchfox.org/mozilla-central/source/devtools/server/actors/thread-configuration.js>
//! This actor manages the configuration flags that the devtools host can apply to threads.

use std::collections::HashMap;
use std::sync::Arc;

use malloc_size_of_derive::MallocSizeOf;
use serde_json::{Map, Value};

use crate::actor::{Actor, ActorEncode, ActorError, ActorRegistry, new_actor_name};
use crate::protocol::ClientRequest;
use crate::{ActorMsg, EmptyReplyMsg, StreamId};

#[derive(MallocSizeOf)]
pub(crate) struct ThreadConfigurationActor {
    name: String,
    _configuration: HashMap<&'static str, bool>,
}

impl Actor for ThreadConfigurationActor {
    fn name(&self) -> &str {
        &self.name
    }

    /// The thread configuration actor can handle the following messages:
    ///
    /// - `updateConfiguration`: Receives new configuration flags from the devtools host.
    fn handle_message(
        &self,
        request: ClientRequest,
        _registry: &ActorRegistry,
        msg_type: &str,
        _msg: &Map<String, Value>,
        _id: StreamId,
    ) -> Result<(), ActorError> {
        match msg_type {
            "updateConfiguration" => {
                // TODO: Actually update configuration
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

impl ThreadConfigurationActor {
    pub fn register(registry: &ActorRegistry) -> Arc<Self> {
        let name = new_actor_name::<Self>();
        let actor = Self {
            name,
            _configuration: HashMap::new(),
        };
        registry.register::<Self>(actor)
    }
}

impl ActorEncode<ActorMsg> for ThreadConfigurationActor {
    fn encode(&self, _: &ActorRegistry) -> ActorMsg {
        ActorMsg {
            actor: self.name().into(),
        }
    }
}

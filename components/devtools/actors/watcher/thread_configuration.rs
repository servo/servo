/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Liberally derived from <https://searchfox.org/mozilla-central/source/devtools/server/actors/thread-configuration.js>
//! This actor manages the configuration flags that the devtools host can apply to threads.

use std::collections::HashMap;
use std::net::TcpStream;

use serde::Serialize;
use serde_json::{Map, Value};

use crate::actor::{Actor, ActorMessageStatus, ActorRegistry};
use crate::protocol::JsonPacketStream;
use crate::{EmptyReplyMsg, StreamId};

#[derive(Serialize)]
pub struct ThreadConfigurationActorMsg {
    actor: String,
}

pub struct ThreadConfigurationActor {
    name: String,
    _configuration: HashMap<&'static str, bool>,
}

impl Actor for ThreadConfigurationActor {
    fn name(&self) -> String {
        self.name.clone()
    }

    /// The thread configuration actor can handle the following messages:
    ///
    /// - `updateConfiguration`: Receives new configuration flags from the devtools host.
    fn handle_message(
        &self,
        _registry: &ActorRegistry,
        msg_type: &str,
        _msg: &Map<String, Value>,
        stream: &mut TcpStream,
        _id: StreamId,
    ) -> Result<ActorMessageStatus, ()> {
        Ok(match msg_type {
            "updateConfiguration" => {
                // TODO: Actually update configuration
                let msg = EmptyReplyMsg { from: self.name() };
                let _ = stream.write_json_packet(&msg);
                ActorMessageStatus::Processed
            },
            _ => ActorMessageStatus::Ignored,
        })
    }
}

impl ThreadConfigurationActor {
    pub fn new(name: String) -> Self {
        Self {
            name,
            _configuration: HashMap::new(),
        }
    }

    pub fn encodable(&self) -> ThreadConfigurationActorMsg {
        ThreadConfigurationActorMsg { actor: self.name() }
    }
}

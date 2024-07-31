/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Liberally derived from <https://searchfox.org/mozilla-central/source/devtools/server/actors/thread-configuration.js>
//! This actor represents one css rule from a node, allowing the inspector to view it and change it.

use std::collections::HashMap;
use std::net::TcpStream;

use serde::Serialize;
use serde_json::{Map, Value};

use crate::actor::{Actor, ActorMessageStatus, ActorRegistry};
use crate::protocol::JsonPacketStream;
use crate::{EmptyReplyMsg, StreamId};

pub struct StyleRuleActor {
    name: String,
}

impl Actor for StyleRuleActor {
    fn name(&self) -> String {
        self.name.clone()
    }

    /// The style rule configuration actor can handle the following messages:
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
            // TODO: "updateConfiguration" => {
            //     let msg = EmptyReplyMsg { from: self.name() };
            //     let _ = stream.write_json_packet(&msg);
            //     ActorMessageStatus::Processed
            // },
            _ => ActorMessageStatus::Ignored,
        })
    }
}

impl StyleRuleActor {
    pub fn new(name: String) -> Self {
        Self { name }
    }
}

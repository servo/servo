/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use actor::{Actor, ActorMessageStatus, ActorRegistry};
use protocol::JsonPacketStream;
use serde_json::Value;
use std::collections::BTreeMap;
use std::net::TcpStream;

pub struct PerformanceActor {
    name: String,
}

#[derive(Serialize)]
struct PerformanceFeatures {
    withMarkers: bool,
    withMemory: bool,
    withTicks: bool,
    withAllocations: bool,
    withJITOptimizations: bool,
}

#[derive(Serialize)]
struct PerformanceTraits {
    features: PerformanceFeatures,
}

#[derive(Serialize)]
struct ConnectReply {
    from: String,
    traits: PerformanceTraits,
}

impl Actor for PerformanceActor {
    fn name(&self) -> String {
        self.name.clone()
    }

    fn handle_message(&self,
                      _registry: &ActorRegistry,
                      msg_type: &str,
                      _msg: &BTreeMap<String, Value>,
                      stream: &mut TcpStream) -> Result<ActorMessageStatus, ()> {
        Ok(match msg_type {
            "connect" => {
                let msg = ConnectReply {
                    from: self.name(),
                    traits: PerformanceTraits {
                        features: PerformanceFeatures {
                            withMarkers: true,
                            withMemory: true,
                            withTicks: true,
                            withAllocations: true,
                            withJITOptimizations: true,
                        },
                    },
                };
                stream.write_json_packet(&msg);
                ActorMessageStatus::Processed
            },
            _ => ActorMessageStatus::Ignored,
        })
    }
}

impl PerformanceActor {
    pub fn new(name: String) -> PerformanceActor {
        PerformanceActor {
            name: name,
        }
    }
}

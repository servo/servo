/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use actor::{Actor, ActorMessageStatus, ActorRegistry};
use protocol::JsonPacketStream;
use serde_json::Value;
use std::collections::BTreeMap;
use std::net::TcpStream;

#[derive(Serialize)]
struct ThreadAttachedReply {
    from: String,
    __type__: String,
    actor: String,
    poppedFrames: Vec<PoppedFrameMsg>,
    why: WhyMsg,
}

#[derive(Serialize)]
enum PoppedFrameMsg {}

#[derive(Serialize)]
struct WhyMsg {
    __type__: String,
}

#[derive(Serialize)]
struct ThreadResumedReply {
    from: String,
    __type__: String,
}

#[derive(Serialize)]
struct ReconfigureReply {
    from: String
}

pub struct ThreadActor {
    name: String,
}

impl ThreadActor {
    pub fn new(name: String) -> ThreadActor {
        ThreadActor {
            name: name,
        }
    }
}

impl Actor for ThreadActor {
    fn name(&self) -> String {
        self.name.clone()
    }

    fn handle_message(&self,
                      registry: &ActorRegistry,
                      msg_type: &str,
                      _msg: &BTreeMap<String, Value>,
                      stream: &mut TcpStream) -> Result<ActorMessageStatus, ()> {
        Ok(match msg_type {
            "attach" => {
                let msg = ThreadAttachedReply {
                    from: self.name(),
                    __type__: "paused".to_owned(),
                    actor: registry.new_name("pause"),
                    poppedFrames: vec![],
                    why: WhyMsg { __type__: "attached".to_owned() },
                };
                stream.write_json_packet(&msg);
                ActorMessageStatus::Processed
            },

            "resume" => {
                let msg = ThreadResumedReply {
                    from: self.name(),
                    __type__: "resumed".to_owned(),
                };
                stream.write_json_packet(&msg);
                ActorMessageStatus::Processed
            },

            "reconfigure" => {
                stream.write_json_packet(&ReconfigureReply { from: self.name() });
                ActorMessageStatus::Processed
            }

            _ => ActorMessageStatus::Ignored,
        })
    }
}

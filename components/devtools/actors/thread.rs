/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use actor::{Actor, ActorMessageStatus, ActorRegistry};
use protocol::JsonPacketStream;
use serde_json::{Map, Value};
use std::net::TcpStream;

#[derive(Serialize)]
struct ThreadAttachedReply {
    from: String,
    #[serde(rename = "type")]
    type_: String,
    actor: String,
    poppedFrames: Vec<PoppedFrameMsg>,
    why: WhyMsg,
}

#[derive(Serialize)]
enum PoppedFrameMsg {}

#[derive(Serialize)]
struct WhyMsg {
    #[serde(rename = "type")]
    type_: String,
}

#[derive(Serialize)]
struct ThreadResumedReply {
    from: String,
    #[serde(rename = "type")]
    type_: String,
}

#[derive(Serialize)]
struct ReconfigureReply {
    from: String
}

#[derive(Serialize)]
struct SourcesReply {
    from: String,
    sources: Vec<Source>,
}

#[derive(Serialize)]
enum Source {}

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
                      _msg: &Map<String, Value>,
                      stream: &mut TcpStream) -> Result<ActorMessageStatus, ()> {
        Ok(match msg_type {
            "attach" => {
                let msg = ThreadAttachedReply {
                    from: self.name(),
                    type_: "paused".to_owned(),
                    actor: registry.new_name("pause"),
                    poppedFrames: vec![],
                    why: WhyMsg { type_: "attached".to_owned() },
                };
                stream.write_json_packet(&msg);
                ActorMessageStatus::Processed
            },

            "resume" => {
                let msg = ThreadResumedReply {
                    from: self.name(),
                    type_: "resumed".to_owned(),
                };
                stream.write_json_packet(&msg);
                ActorMessageStatus::Processed
            },

            "reconfigure" => {
                stream.write_json_packet(&ReconfigureReply { from: self.name() });
                ActorMessageStatus::Processed
            }

            "sources" => {
                let msg = SourcesReply {
                    from: self.name(),
                    sources: vec![],
                };
                stream.write_json_packet(&msg);
                ActorMessageStatus::Processed
            }

            _ => ActorMessageStatus::Ignored,
        })
    }
}

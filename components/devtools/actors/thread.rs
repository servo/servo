/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::actor::{Actor, ActorMessageStatus, ActorRegistry};
use crate::protocol::JsonPacketStream;
use crate::StreamId;
use serde_json::{Map, Value};
use std::net::TcpStream;

#[derive(Serialize)]
struct ThreadAttached {
    from: String,
    #[serde(rename = "type")]
    type_: String,
    actor: String,
    frame: u32,
    error: u32,
    recordingEndpoint: u32,
    executionPoint: u32,
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
struct ThreadInterruptedReply {
    from: String,
    #[serde(rename = "type")]
    type_: String,
}

#[derive(Serialize)]
struct ReconfigureReply {
    from: String,
}

#[derive(Serialize)]
struct SourcesReply {
    from: String,
    sources: Vec<Source>,
}

#[derive(Serialize)]
enum Source {}

#[derive(Serialize)]
struct VoidAttachedReply {
    from: String,
}

pub struct ThreadActor {
    name: String,
}

impl ThreadActor {
    pub fn new(name: String) -> ThreadActor {
        ThreadActor { name: name }
    }
}

impl Actor for ThreadActor {
    fn name(&self) -> String {
        self.name.clone()
    }

    fn handle_message(
        &self,
        registry: &ActorRegistry,
        msg_type: &str,
        _msg: &Map<String, Value>,
        stream: &mut TcpStream,
        _id: StreamId,
    ) -> Result<ActorMessageStatus, ()> {
        Ok(match msg_type {
            "attach" => {
                let msg = ThreadAttached {
                    from: self.name(),
                    type_: "paused".to_owned(),
                    actor: registry.new_name("pause"),
                    frame: 0,
                    error: 0,
                    recordingEndpoint: 0,
                    executionPoint: 0,
                    poppedFrames: vec![],
                    why: WhyMsg {
                        type_: "attached".to_owned(),
                    },
                };
                let _ = stream.write_json_packet(&msg);
                let _ = stream.write_json_packet(&VoidAttachedReply { from: self.name() });
                ActorMessageStatus::Processed
            },

            "resume" => {
                let msg = ThreadResumedReply {
                    from: self.name(),
                    type_: "resumed".to_owned(),
                };
                let _ = stream.write_json_packet(&msg);
                let _ = stream.write_json_packet(&VoidAttachedReply { from: self.name() });
                ActorMessageStatus::Processed
            },

            "interrupt" => {
                let msg = ThreadInterruptedReply {
                    from: self.name(),
                    type_: "interrupted".to_owned(),
                };
                let _ = stream.write_json_packet(&msg);
                ActorMessageStatus::Processed
            },

            "reconfigure" => {
                let _ = stream.write_json_packet(&ReconfigureReply { from: self.name() });
                ActorMessageStatus::Processed
            },

            "sources" => {
                let msg = SourcesReply {
                    from: self.name(),
                    sources: vec![],
                };
                let _ = stream.write_json_packet(&msg);
                ActorMessageStatus::Processed
            },

            _ => ActorMessageStatus::Ignored,
        })
    }
}

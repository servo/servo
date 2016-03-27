/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use actor::{Actor, ActorMessageStatus, ActorRegistry};
use protocol::JsonPacketStream;
use rustc_serialize::json;
use std::net::TcpStream;

#[derive(RustcEncodable)]
struct ThreadAttachedReply {
    from: String,
    __type__: String,
    actor: String,
    poppedFrames: Vec<PoppedFrameMsg>,
    why: WhyMsg,
}

#[derive(RustcEncodable)]
enum PoppedFrameMsg {}

#[derive(RustcEncodable)]
struct WhyMsg {
    __type__: String,
}

#[derive(RustcEncodable)]
struct ThreadResumedReply {
    from: String,
    __type__: String,
}

#[derive(RustcEncodable)]
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
                      _msg: &json::Object,
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

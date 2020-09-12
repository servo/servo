/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::actor::{Actor, ActorMessageStatus, ActorRegistry};
use crate::protocol::JsonPacketStream;
use crate::StreamId;
use serde_json::{Map, Value};
use servo_config::pref_util::PrefValue;
use servo_config::prefs::pref_map;
use std::net::TcpStream;

pub struct PreferenceActor {
    name: String,
}

impl PreferenceActor {
    pub fn new(name: String) -> Self {
        Self { name }
    }
}

impl Actor for PreferenceActor {
    fn name(&self) -> String {
        self.name.clone()
    }

    fn handle_message(
        &self,
        _registry: &ActorRegistry,
        msg_type: &str,
        _msg: &Map<String, Value>,
        stream: &mut TcpStream,
        _id: StreamId,
    ) -> Result<ActorMessageStatus, ()> {
        let pref_value = pref_map().get(msg_type);
        Ok(match pref_value {
            PrefValue::Float(value) => {
                let reply = FloatReply {
                    from: self.name(),
                    value: value,
                };
                let _ = stream.write_json_packet(&reply);
                ActorMessageStatus::Processed
            },
            PrefValue::Int(value) => {
                let reply = IntReply {
                    from: self.name(),
                    value: value,
                };
                let _ = stream.write_json_packet(&reply);
                ActorMessageStatus::Processed
            },
            PrefValue::Str(value) => {
                let reply = CharReply {
                    from: self.name(),
                    value: value,
                };
                let _ = stream.write_json_packet(&reply);
                ActorMessageStatus::Processed
            },
            PrefValue::Bool(value) => {
                let reply = BoolReply {
                    from: self.name(),
                    value: value,
                };
                let _ = stream.write_json_packet(&reply);
                ActorMessageStatus::Processed
            },
            PrefValue::Missing => ActorMessageStatus::Ignored,
        })
    }
}

#[derive(Serialize)]
struct BoolReply {
    from: String,
    value: bool,
}

#[derive(Serialize)]
struct CharReply {
    from: String,
    value: String,
}

#[derive(Serialize)]
struct IntReply {
    from: String,
    value: i64,
}

#[derive(Serialize)]
struct FloatReply {
    from: String,
    value: f64,
}

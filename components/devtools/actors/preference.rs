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
            PrefValue::Missing => handle_missing_preference(self.name(), msg_type, stream),
        })
    }
}

// if the preferences are missing from pref_map then we return a
// fake preference response based on msg_type.
fn handle_missing_preference(
    name: String,
    msg_type: &str,
    stream: &mut TcpStream,
) -> ActorMessageStatus {
    match msg_type {
        "getBoolPref" => {
            let reply = BoolReply {
                from: name,
                value: false,
            };
            let _ = stream.write_json_packet(&reply);
            ActorMessageStatus::Processed
        },

        "getCharPref" => {
            let reply = CharReply {
                from: name,
                value: "".to_owned(),
            };
            let _ = stream.write_json_packet(&reply);
            ActorMessageStatus::Processed
        },

        "getIntPref" => {
            let reply = IntReply {
                from: name,
                value: 0,
            };
            let _ = stream.write_json_packet(&reply);
            ActorMessageStatus::Processed
        },

        _ => ActorMessageStatus::Ignored,
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

/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::net::TcpStream;

use log::warn;
use serde::Serialize;
use serde_json::{Map, Value};
use servo_config::pref;

use crate::StreamId;
use crate::actor::{Actor, ActorMessageStatus, ActorRegistry};
use crate::protocol::JsonPacketStream;

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
        msg: &Map<String, Value>,
        stream: &mut TcpStream,
        _id: StreamId,
    ) -> Result<ActorMessageStatus, ()> {
        let Some(key) = msg.get("value").and_then(|v| v.as_str()) else {
            warn!("PreferenceActor: handle_message: value is not a string");
            return Ok(ActorMessageStatus::Ignored);
        };

        // TODO: Map more preferences onto their Servo values.
        Ok(match key {
            "dom.serviceWorkers.enabled" => {
                self.write_bool(pref!(dom_serviceworker_enabled), stream)
            },
            _ => self.handle_missing_preference(msg_type, stream),
        })
    }
}

impl PreferenceActor {
    fn handle_missing_preference(
        &self,
        msg_type: &str,
        stream: &mut TcpStream,
    ) -> ActorMessageStatus {
        match msg_type {
            "getBoolPref" => self.write_bool(false, stream),
            "getCharPref" => self.write_char("".into(), stream),
            "getIntPref" => self.write_int(0, stream),
            "getFloatPref" => self.write_float(0., stream),
            _ => ActorMessageStatus::Ignored,
        }
    }

    fn write_bool(&self, pref_value: bool, stream: &mut TcpStream) -> ActorMessageStatus {
        #[derive(Serialize)]
        struct BoolReply {
            from: String,
            value: bool,
        }

        let reply = BoolReply {
            from: self.name.clone(),
            value: pref_value,
        };
        let _ = stream.write_json_packet(&reply);
        ActorMessageStatus::Processed
    }

    fn write_char(&self, pref_value: String, stream: &mut TcpStream) -> ActorMessageStatus {
        #[derive(Serialize)]
        struct CharReply {
            from: String,
            value: String,
        }

        let reply = CharReply {
            from: self.name.clone(),
            value: pref_value,
        };
        let _ = stream.write_json_packet(&reply);
        ActorMessageStatus::Processed
    }

    fn write_int(&self, pref_value: i64, stream: &mut TcpStream) -> ActorMessageStatus {
        #[derive(Serialize)]
        struct IntReply {
            from: String,
            value: i64,
        }

        let reply = IntReply {
            from: self.name.clone(),
            value: pref_value,
        };
        let _ = stream.write_json_packet(&reply);
        ActorMessageStatus::Processed
    }

    fn write_float(&self, pref_value: f64, stream: &mut TcpStream) -> ActorMessageStatus {
        #[derive(Serialize)]
        struct FloatReply {
            from: String,
            value: f64,
        }

        let reply = FloatReply {
            from: self.name.clone(),
            value: pref_value,
        };
        let _ = stream.write_json_packet(&reply);
        ActorMessageStatus::Processed
    }
}

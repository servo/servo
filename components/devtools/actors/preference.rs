/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use serde::Serialize;
use serde_json::{Map, Value};
use servo_config::pref;

use crate::StreamId;
use crate::actor::{Actor, ActorError, ActorRegistry};
use crate::protocol::ClientRequest;

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
        request: ClientRequest,
        _registry: &ActorRegistry,
        msg_type: &str,
        msg: &Map<String, Value>,
        _id: StreamId,
    ) -> Result<(), ActorError> {
        let key = msg
            .get("value")
            .ok_or(ActorError::MissingParameter)?
            .as_str()
            .ok_or(ActorError::BadParameterType)?;

        // TODO: Map more preferences onto their Servo values.
        match key {
            "dom.serviceWorkers.enabled" => {
                self.write_bool(request, pref!(dom_serviceworker_enabled))
            },
            _ => self.handle_missing_preference(request, msg_type),
        }
    }
}

impl PreferenceActor {
    fn handle_missing_preference(
        &self,
        request: ClientRequest,
        msg_type: &str,
    ) -> Result<(), ActorError> {
        match msg_type {
            "getBoolPref" => self.write_bool(request, false),
            "getCharPref" => self.write_char(request, "".into()),
            "getIntPref" => self.write_int(request, 0),
            "getFloatPref" => self.write_float(request, 0.),
            _ => Err(ActorError::UnrecognizedPacketType),
        }
    }

    fn write_bool(&self, request: ClientRequest, pref_value: bool) -> Result<(), ActorError> {
        #[derive(Serialize)]
        struct BoolReply {
            from: String,
            value: bool,
        }

        let reply = BoolReply {
            from: self.name.clone(),
            value: pref_value,
        };
        request.reply_final(&reply)
    }

    fn write_char(&self, request: ClientRequest, pref_value: String) -> Result<(), ActorError> {
        #[derive(Serialize)]
        struct CharReply {
            from: String,
            value: String,
        }

        let reply = CharReply {
            from: self.name.clone(),
            value: pref_value,
        };
        request.reply_final(&reply)
    }

    fn write_int(&self, request: ClientRequest, pref_value: i64) -> Result<(), ActorError> {
        #[derive(Serialize)]
        struct IntReply {
            from: String,
            value: i64,
        }

        let reply = IntReply {
            from: self.name.clone(),
            value: pref_value,
        };
        request.reply_final(&reply)
    }

    fn write_float(&self, request: ClientRequest, pref_value: f64) -> Result<(), ActorError> {
        #[derive(Serialize)]
        struct FloatReply {
            from: String,
            value: f64,
        }

        let reply = FloatReply {
            from: self.name.clone(),
            value: pref_value,
        };
        request.reply_final(&reply)
    }
}

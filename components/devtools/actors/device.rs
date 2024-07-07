/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::net::TcpStream;

use serde::Serialize;
use serde_json::{Map, Value};

use crate::actor::{Actor, ActorMessageStatus, ActorRegistry};
use crate::protocol::{ActorDescription, JsonPacketStream, Method};
use crate::StreamId;

#[derive(Serialize)]
struct GetDescriptionReply {
    from: String,
    value: SystemInfo,
}

// This is only a minimal subset of the properties exposed/expected by Firefox
// (see https://searchfox.org/mozilla-central/source/devtools/shared/system.js#45)
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SystemInfo {
    apptype: String,
    // Display version
    version: String,
    // Build ID (timestamp with format YYYYMMDDhhmmss), used for compatibility checks
    // (see https://searchfox.org/mozilla-central/source/devtools/client/shared/remote-debugging/version-checker.js#82)
    appbuildid: String,
    // Firefox major.minor version number, use for compatibility checks
    platformversion: String,
    // Display name
    brand_name: String,
}

include!(concat!(env!("OUT_DIR"), "/build_id.rs"));

pub struct DeviceActor {
    pub name: String,
}

impl Actor for DeviceActor {
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
        Ok(match msg_type {
            "getDescription" => {
                let msg = GetDescriptionReply {
                    from: self.name(),
                    value: SystemInfo {
                        apptype: "servo".to_string(),
                        version: env!("CARGO_PKG_VERSION").to_string(),
                        appbuildid: BUILD_ID.to_string(),
                        platformversion: "125.0".to_string(),
                        brand_name: "Servo".to_string(),
                    },
                };
                let _ = stream.write_json_packet(&msg);
                ActorMessageStatus::Processed
            },

            _ => ActorMessageStatus::Ignored,
        })
    }
}

impl DeviceActor {
    pub fn new(name: String) -> DeviceActor {
        DeviceActor { name }
    }

    pub fn description() -> ActorDescription {
        ActorDescription {
            category: "actor",
            type_name: "device",
            methods: vec![Method {
                name: "getDescription",
                request: Value::Null,
                response: Value::Object(
                    vec![(
                        "value".to_owned(),
                        Value::Object(
                            vec![("_retval".to_owned(), Value::String("json".to_owned()))]
                                .into_iter()
                                .collect(),
                        ),
                    )]
                    .into_iter()
                    .collect(),
                ),
            }],
        }
    }
}

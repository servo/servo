/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::actor::{Actor, ActorMessageStatus, ActorRegistry};
use crate::protocol::JsonPacketStream;
use crate::protocol::{ActorDescription, Method};
use crate::StreamId;
use serde_json::{Map, Value};
use std::net::TcpStream;

#[derive(Serialize)]
struct GetDescriptionReply {
    from: String,
    value: SystemInfo,
}

#[derive(Serialize)]
struct SystemInfo {
    apptype: String,
    platformVersion: String,
}

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
                        platformVersion: "71.0".to_string(),
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
        DeviceActor { name: name }
    }

    pub fn description() -> ActorDescription {
        ActorDescription {
            category: "actor",
            typeName: "device",
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

/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! This actor holds a database of available css properties, their supported values and
//! alternative names

use std::collections::HashMap;
use std::net::TcpStream;

use devtools_traits::CssDatabaseProperty;
use serde::Serialize;
use serde_json::{Map, Value};

use crate::actor::{Actor, ActorMessageStatus, ActorRegistry};
use crate::protocol::JsonPacketStream;
use crate::StreamId;

pub struct CssPropertiesActor {
    name: String,
    properties: HashMap<String, CssDatabaseProperty>,
}

#[derive(Serialize)]
struct GetCssDatabaseReply<'a> {
    from: String,
    properties: &'a HashMap<String, CssDatabaseProperty>,
}

impl Actor for CssPropertiesActor {
    fn name(&self) -> String {
        self.name.clone()
    }

    /// The css properties actor can handle the following messages:
    ///
    /// - `getCSSDatabase`: Returns a big list of every supported css property so that the
    ///   inspector can show the available options
    fn handle_message(
        &self,
        _registry: &ActorRegistry,
        msg_type: &str,
        _msg: &Map<String, Value>,
        stream: &mut TcpStream,
        _id: StreamId,
    ) -> Result<ActorMessageStatus, ()> {
        Ok(match msg_type {
            "getCSSDatabase" => {
                let _ = stream.write_json_packet(&GetCssDatabaseReply {
                    from: self.name(),
                    properties: &self.properties,
                });

                ActorMessageStatus::Processed
            },
            _ => ActorMessageStatus::Ignored,
        })
    }
}

impl CssPropertiesActor {
    pub fn new(name: String, properties: HashMap<String, CssDatabaseProperty>) -> Self {
        Self { name, properties }
    }
}

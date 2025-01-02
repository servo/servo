/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::net::TcpStream;

use serde::Serialize;
use serde_json::{Map, Value};

use crate::actor::{Actor, ActorMessageStatus, ActorRegistry};
use crate::StreamId;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TimelineMemoryReply {
    js_object_size: u64,
    js_string_size: u64,
    js_other_size: u64,
    dom_size: u64,
    style_size: u64,
    other_size: u64,
    total_size: u64,
    js_milliseconds: f64,
    #[serde(rename = "nonJSMilliseconds")]
    non_js_milliseconds: f64,
}

pub struct MemoryActor {
    pub name: String,
}

impl Actor for MemoryActor {
    fn name(&self) -> String {
        self.name.clone()
    }

    fn handle_message(
        &self,
        _registry: &ActorRegistry,
        _msg_type: &str,
        _msg: &Map<String, Value>,
        _stream: &mut TcpStream,
        _id: StreamId,
    ) -> Result<ActorMessageStatus, ()> {
        Ok(ActorMessageStatus::Ignored)
    }
}

impl MemoryActor {
    /// return name of actor
    pub fn create(registry: &ActorRegistry) -> String {
        let actor_name = registry.new_name("memory");
        let actor = MemoryActor {
            name: actor_name.clone(),
        };

        registry.register_later(Box::new(actor));
        actor_name
    }

    pub fn measure(&self) -> TimelineMemoryReply {
        TimelineMemoryReply {
            js_object_size: 1,
            js_string_size: 1,
            js_other_size: 1,
            dom_size: 1,
            style_size: 1,
            other_size: 1,
            total_size: 1,
            js_milliseconds: 1.1,
            non_js_milliseconds: 1.1,
        }
    }
}

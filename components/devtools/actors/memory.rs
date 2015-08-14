/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use rustc_serialize::json;
use std::net::TcpStream;

use actor::{Actor, ActorRegistry, ActorMessageStatus};

#[derive(RustcEncodable)]
pub struct TimelineMemoryReply {
    jsObjectSize: u64,
    jsStringSize: u64,
    jsOtherSize: u64,
    domSize: u64,
    styleSize: u64,
    otherSize: u64,
    totalSize: u64,
    jsMilliseconds: f64,
    nonJSMilliseconds: f64,
}

pub struct MemoryActor {
    pub name: String,
}

impl Actor for MemoryActor {
    fn name(&self) -> String {
        self.name.clone()
    }

    fn handle_message(&self,
                      _registry: &ActorRegistry,
                      _msg_type: &str,
                      _msg: &json::Object,
                      _stream: &mut TcpStream) -> Result<ActorMessageStatus, ()> {
        Ok(ActorMessageStatus::Ignored)
    }
}

impl MemoryActor {
    /// return name of actor
    pub fn create(registry: &ActorRegistry) -> String {
        let actor_name = registry.new_name("memory");
        let actor = MemoryActor {
            name: actor_name.clone()
        };

        registry.register_later(box actor);
        actor_name
    }

    pub fn measure(&self) -> TimelineMemoryReply {
        //TODO:
        TimelineMemoryReply {
            jsObjectSize: 1,
            jsStringSize: 1,
            jsOtherSize: 1,
            domSize: 1,
            styleSize: 1,
            otherSize: 1,
            totalSize: 1,
            jsMilliseconds: 1.1,
            nonJSMilliseconds: 1.1,
        }
    }
}

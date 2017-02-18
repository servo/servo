/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use actor::{Actor, ActorMessageStatus, ActorRegistry};
use serde_json::{Map, Value};
use std::net::TcpStream;

pub struct ProfilerActor {
    name: String,
}

impl Actor for ProfilerActor {
    fn name(&self) -> String {
        self.name.clone()
    }

    fn handle_message(&self,
                      _registry: &ActorRegistry,
                      _msg_type: &str,
                      _msg: &Map<String, Value>,
                      _stream: &mut TcpStream) -> Result<ActorMessageStatus, ()> {
        Ok(ActorMessageStatus::Ignored)
    }
}

impl ProfilerActor {
    pub fn new(name: String) -> ProfilerActor {
        ProfilerActor {
            name: name,
        }
    }
}

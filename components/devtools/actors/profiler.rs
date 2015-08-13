/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use actor::{Actor, ActorRegistry};

use rustc_serialize::json;
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
                      _msg: &json::Object,
                      _stream: &mut TcpStream) -> Result<bool, ()> {
        Ok(false)
    }
}

impl ProfilerActor {
    pub fn new(name: String) -> ProfilerActor {
        ProfilerActor {
            name: name,
        }
    }
}

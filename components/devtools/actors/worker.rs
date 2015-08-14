/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use actor::{Actor, ActorRegistry, ActorMessageStatus};
use msg::constellation_msg::WorkerId;
use rustc_serialize::json;
use std::net::TcpStream;

pub struct WorkerActor {
    pub name: String,
    pub console: String,
    pub id: WorkerId,
}

impl Actor for WorkerActor {
    fn name(&self) -> String {
        self.name.clone()
    }
    fn handle_message(&self,
                      _: &ActorRegistry,
                      _: &str,
                      _: &json::Object,
                      _: &mut TcpStream) -> Result<ActorMessageStatus, ()> {
        Ok(ActorMessageStatus::Processed)
    }
}

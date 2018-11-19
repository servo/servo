/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::actor::{Actor, ActorMessageStatus, ActorRegistry};
use serde_json::{Map, Value};
use std::net::TcpStream;

pub struct StyleSheetsActor {
    pub name: String,
}

impl Actor for StyleSheetsActor {
    fn name(&self) -> String {
        self.name.clone()
    }
    fn handle_message(
        &self,
        _: &ActorRegistry,
        _: &str,
        _: &Map<String, Value>,
        _: &mut TcpStream,
    ) -> Result<ActorMessageStatus, ()> {
        Ok(ActorMessageStatus::Ignored)
    }
}

impl StyleSheetsActor {
    pub fn new(name: String) -> StyleSheetsActor {
        StyleSheetsActor { name: name }
    }
}

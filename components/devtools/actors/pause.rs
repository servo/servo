/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use serde_json::{Map, Value};
use serde::Serialize;

use crate::StreamId;
use crate::actor::{Actor, ActorError, ActorRegistry};
use crate::protocol::ClientRequest;

#[derive(Serialize)]
#[derive(Clone, Debug)]
pub struct PauseActor {
    pub name: String,
}

impl PauseActor {
    pub fn new(
        name: String,
    ) -> PauseActor {
        PauseActor {
            name,
        }
    }

    pub fn register(
        registry: &ActorRegistry,
    ) -> String {
        let name = registry.new_name("pause");

        let actor = PauseActor::new(
            name.clone(),
        );

        registry.register_later(Box::new(actor));
        name
    }
}

impl Actor for PauseActor {
    fn name(&self) -> String {
        self.name.clone()
    }

    fn handle_message(
        &self,
        _request: ClientRequest,
        _registry: &ActorRegistry,
        _msg_type: &str,
        _msg: &Map<String, Value>,
        _id: StreamId,
    ) -> Result<(), ActorError> {
        Err(ActorError::UnrecognizedPacketType)
    }
}

/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use serde_json::{Map, Value};

use crate::StreamId;
use crate::actor::{Actor, ActorError, ActorRegistry};
use crate::protocol::ClientRequest;

// TODO: Remove once the actor is used.
#[allow(dead_code)]
/// Referenced by `ThreadActor` when replying to `interupt` messages.
/// <https://searchfox.org/firefox-main/source/devtools/server/actors/thread.js#1699>
pub struct PauseActor {
    pub name: String,
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
        // TODO: Handle messages.
        Err(ActorError::UnrecognizedPacketType)
    }
}

/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::actor::{Actor, ActorEncode, ActorError, ActorRegistry};
use crate::protocol::ClientRequest;
use crate::{ActorMsg, EmptyReplyMsg};

pub struct BreakpointListActor {
    name: String,
}

impl Actor for BreakpointListActor {
    fn name(&self) -> String {
        self.name.clone()
    }

    fn handle_message(
        &self,
        request: ClientRequest,
        _registry: &crate::actor::ActorRegistry,
        msg_type: &str,
        _msg: &serde_json::Map<String, serde_json::Value>,
        _stream_id: crate::StreamId,
    ) -> Result<(), ActorError> {
        match msg_type {
            // Client wants to set a breakpoint.
            // Seems to be infallible, unlike the thread actor’s `setBreakpoint`.
            // <https://firefox-source-docs.mozilla.org/devtools/backend/protocol.html#breakpoints>
            "setBreakpoint" => {
                let msg = EmptyReplyMsg { from: self.name() };
                request.reply_final(&msg)?
            },
            "setActiveEventBreakpoints" => {
                let msg = EmptyReplyMsg { from: self.name() };
                request.reply_final(&msg)?
            },
            "removeBreakpoint" => {
                let msg = EmptyReplyMsg { from: self.name() };
                request.reply_final(&msg)?
            },
            _ => return Err(ActorError::UnrecognizedPacketType),
        };
        Ok(())
    }
}

impl BreakpointListActor {
    pub fn new(name: String) -> Self {
        Self { name }
    }
}

impl ActorEncode<ActorMsg> for BreakpointListActor {
    fn encode(&self, _: &ActorRegistry) -> ActorMsg {
        ActorMsg { actor: self.name() }
    }
}

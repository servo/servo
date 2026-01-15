/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use base::generic_channel::channel;
use devtools_traits::DevtoolScriptControlMsg;

use crate::actor::{Actor, ActorEncode, ActorError, ActorRegistry};
use crate::actors::browsing_context::BrowsingContextActor;
use crate::actors::thread::ThreadActor;
use crate::protocol::ClientRequest;
use crate::{ActorMsg, EmptyReplyMsg};

pub(crate) struct BreakpointListActor {
    name: String,
    browsing_context: String,
}

impl Actor for BreakpointListActor {
    fn name(&self) -> String {
        self.name.clone()
    }

    fn handle_message(
        &self,
        request: ClientRequest,
        registry: &crate::actor::ActorRegistry,
        msg_type: &str,
        msg: &serde_json::Map<String, serde_json::Value>,
        _stream_id: crate::StreamId,
    ) -> Result<(), ActorError> {
        match msg_type {
            // Client wants to set a breakpoint.
            // Seems to be infallible, unlike the thread actor’s `setBreakpoint`.
            // <https://firefox-source-docs.mozilla.org/devtools/backend/protocol.html#breakpoints>
            "setBreakpoint" => {
                let location = msg.get("location");
                let column = location
                    .and_then(|location| location.get("column"))
                    .and_then(|column| column.as_number())
                    .and_then(|column| column.as_u64())
                    .ok_or(ActorError::Internal)?;
                let line = location
                    .and_then(|location| location.get("line"))
                    .and_then(|line| line.as_number())
                    .and_then(|line| line.as_u64())
                    .ok_or(ActorError::Internal)?;
                let source_url = location
                    .and_then(|location| location.get("sourceUrl"))
                    .and_then(|source_url| source_url.as_str())
                    .ok_or(ActorError::Internal)?;

                let browsing_context =
                    registry.find::<BrowsingContextActor>(&self.browsing_context);
                let thread = registry.find::<ThreadActor>(&browsing_context.thread);
                let source = thread
                    .source_manager
                    .find_source(registry, source_url)
                    .ok_or(ActorError::Internal)?;
                let offset = source.find_offset(column as u32, line as u32);

                // set-breakpoint
                let (tx, rx) = channel().ok_or(ActorError::Internal)?;
                source
                    .script_sender
                    .send(DevtoolScriptControlMsg::SetBreakpoint(
                        source.spidermonkey_id,
                        offset,
                        tx,
                    ))
                    .map_err(|_| ActorError::Internal)?;
                let _ = rx.recv().map_err(|_| ActorError::Internal)?;

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
    pub fn new(name: String, browsing_context: String) -> Self {
        Self {
            name,
            browsing_context,
        }
    }
}

impl ActorEncode<ActorMsg> for BreakpointListActor {
    fn encode(&self, _: &ActorRegistry) -> ActorMsg {
        ActorMsg { actor: self.name() }
    }
}

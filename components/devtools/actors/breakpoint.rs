/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use devtools_traits::DevtoolScriptControlMsg;
use serde::Deserialize;
use serde_json::Map;

use crate::actor::{Actor, ActorEncode, ActorError, ActorRegistry};
use crate::actors::browsing_context::BrowsingContextActor;
use crate::actors::thread::ThreadActor;
use crate::protocol::ClientRequest;
use crate::{ActorMsg, EmptyReplyMsg};

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BreakpointRequestLocation {
    pub line: u32,
    pub column: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_url: Option<String>,
}

#[derive(Deserialize)]
struct BreakpointRequest {
    location: BreakpointRequestLocation,
}

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
        msg: &Map<String, serde_json::Value>,
        _stream_id: crate::StreamId,
    ) -> Result<(), ActorError> {
        match msg_type {
            // Client wants to set a breakpoint.
            // Seems to be infallible, unlike the thread actor’s `setBreakpoint`.
            // <https://firefox-source-docs.mozilla.org/devtools/backend/protocol.html#breakpoints>
            "setBreakpoint" => {
                let msg: BreakpointRequest =
                    serde_json::from_value(msg.clone().into()).map_err(|_| ActorError::Internal)?;
                let BreakpointRequestLocation {
                    line,
                    column,
                    source_url,
                } = msg.location;
                let source_url = source_url.ok_or(ActorError::Internal)?;

                let browsing_context =
                    registry.find::<BrowsingContextActor>(&self.browsing_context);
                let thread = registry.find::<ThreadActor>(&browsing_context.thread);
                let source = thread
                    .source_manager
                    .find_source(registry, &source_url)
                    .ok_or(ActorError::Internal)?;
                let (script_id, offset) = source.find_offset(line, column);

                source
                    .script_sender
                    .send(DevtoolScriptControlMsg::SetBreakpoint(
                        source.spidermonkey_id,
                        script_id,
                        offset,
                    ))
                    .map_err(|_| ActorError::Internal)?;

                let msg = EmptyReplyMsg { from: self.name() };
                request.reply_final(&msg)?
            },
            "setActiveEventBreakpoints" => {
                let msg = EmptyReplyMsg { from: self.name() };
                request.reply_final(&msg)?
            },
            "removeBreakpoint" => {
                let msg: BreakpointRequest =
                    serde_json::from_value(msg.clone().into()).map_err(|_| ActorError::Internal)?;
                let BreakpointRequestLocation {
                    line,
                    column,
                    source_url,
                } = msg.location;
                let source_url = source_url.ok_or(ActorError::Internal)?;

                let browsing_context =
                    registry.find::<BrowsingContextActor>(&self.browsing_context);
                let thread = registry.find::<ThreadActor>(&browsing_context.thread);
                let source = thread
                    .source_manager
                    .find_source(registry, &source_url)
                    .ok_or(ActorError::Internal)?;
                let (script_id, offset) = source.find_offset(line, column);

                source
                    .script_sender
                    .send(DevtoolScriptControlMsg::ClearBreakpoint(
                        source.spidermonkey_id,
                        script_id,
                        offset,
                    ))
                    .map_err(|_| ActorError::Internal)?;

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

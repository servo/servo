/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use devtools_traits::{DevtoolScriptControlMsg, SourceLocation};
use malloc_size_of_derive::MallocSizeOf;
use serde::Deserialize;

use crate::actor::{Actor, ActorEncode, ActorError, ActorRegistry};
use crate::actors::browsing_context::BrowsingContextActor;
use crate::actors::thread::ThreadActor;
use crate::{ActorMsg, EmptyReplyMsg};

/// Used for both 'blackbox' and 'unblackbox' request
#[derive(Deserialize)]
struct BlackboxRequest {
    range: Vec<BlackboxRange>,
    url: String,
}

#[derive(Deserialize, Debug)]
struct BlackboxRange {
    start: BlackboxSourceLocation,
    end: BlackboxSourceLocation,
}

#[derive(Deserialize, Debug)]
struct BlackboxSourceLocation {
    line: u32,
    column: u32,
}

#[derive(MallocSizeOf)]
pub(crate) struct BlackboxingActor {
    name: String,
    browsing_context_name: String,
}

impl Actor for BlackboxingActor {
    fn name(&self) -> String {
        self.name.clone()
    }

    fn handle_message(
        &self,
        request: crate::protocol::ClientRequest,
        registry: &ActorRegistry,
        msg_type: &str,
        msg: &serde_json::Map<String, serde_json::Value>,
        _: crate::StreamId,
    ) -> Result<(), crate::actor::ActorError> {
        if msg_type != "blackbox" && msg_type != "unblackbox" {
            return Err(ActorError::UnrecognizedPacketType);
        }

        let blackbox_request: BlackboxRequest =
            serde_json::from_value(msg.clone().into()).map_err(|_| ActorError::Internal)?;

        let (start, end) = match &blackbox_request.range[..] {
            [] => (
                SourceLocation { line: 0, column: 0 },
                SourceLocation {
                    line: u32::MAX,
                    column: u32::MAX,
                },
            ),
            [range] => (
                SourceLocation {
                    line: range.start.line,
                    column: range.start.column,
                },
                SourceLocation {
                    line: range.end.line,
                    column: range.end.column,
                },
            ),
            _ => {
                log::warn!(
                    "Expected 0 or 1 range elements, got {:?}",
                    blackbox_request.range
                );
                return Err(ActorError::Internal);
            },
        };

        let browsing_context_actor =
            registry.find::<BrowsingContextActor>(&self.browsing_context_name);
        let thread_actor = registry.find::<ThreadActor>(&browsing_context_actor.thread_name);
        let source = thread_actor
            .source_manager
            .find_source(registry, &blackbox_request.url)
            .ok_or(ActorError::Internal)?;

        let control_msg = match msg_type {
            "blackbox" => DevtoolScriptControlMsg::Blackbox(source.spidermonkey_id, start, end),
            "unblackbox" => DevtoolScriptControlMsg::Unblackbox(source.spidermonkey_id, start, end),
            _ => unreachable!(),
        };
        source
            .script_sender
            .send(control_msg)
            .map_err(|_| ActorError::Internal)?;

        request.reply_final(&EmptyReplyMsg { from: self.name() })?;
        Ok(())
    }
}

impl BlackboxingActor {
    pub fn register(registry: &ActorRegistry, browsing_context_name: String) -> String {
        let name = registry.new_name::<Self>();
        let actor = Self {
            name: name.clone(),
            browsing_context_name,
        };
        registry.register::<Self>(actor);
        name
    }
}

impl ActorEncode<ActorMsg> for BlackboxingActor {
    fn encode(&self, _: &ActorRegistry) -> ActorMsg {
        ActorMsg { actor: self.name() }
    }
}

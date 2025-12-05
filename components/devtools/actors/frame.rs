/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// TODO: Remove once the actor is used
#![allow(dead_code)]

use serde::Serialize;
use serde_json::{Map, Value};

use crate::StreamId;
use crate::actor::{Actor, ActorEncode, ActorError, ActorRegistry};
use crate::actors::environment::{EnvironmentActor, EnvironmentActorMsg};
use crate::protocol::ClientRequest;

#[derive(Serialize)]
struct FrameEnvironmentReply {
    from: String,
    #[serde(flatten)]
    environment: EnvironmentActorMsg,
}

#[derive(Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum FrameState {
    OnStack,
    Suspended,
    Dead,
}

#[derive(Serialize)]
pub struct FrameWhere {
    actor: String,
    line: u32,
    column: u32,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FrameActorMsg {
    actor: String,
    #[serde(rename = "type")]
    type_: String,
    arguments: Vec<Value>,
    async_cause: Option<String>,
    display_name: String,
    oldest: bool,
    state: FrameState,
    #[serde(rename = "where")]
    where_: FrameWhere,
}

/// Represents an stack frame. Used by `ThreadActor` when replying to interrupt messages.
/// <https://searchfox.org/firefox-main/source/devtools/server/actors/frame.js>
pub struct FrameActor {
    pub name: String,
    pub source_actor: String,
}

impl Actor for FrameActor {
    fn name(&self) -> String {
        self.name.clone()
    }

    // https://searchfox.org/firefox-main/source/devtools/shared/specs/frame.js
    fn handle_message(
        &self,
        request: ClientRequest,
        registry: &ActorRegistry,
        msg_type: &str,
        _msg: &Map<String, Value>,
        _id: StreamId,
    ) -> Result<(), ActorError> {
        match msg_type {
            "getEnvironment" => {
                let environment = EnvironmentActor {
                    name: registry.new_name("environment"),
                    parent: None,
                };
                let msg = FrameEnvironmentReply {
                    from: self.name(),
                    environment: environment.encode(registry),
                };
                registry.register_later(environment);
                request.reply_final(&msg)?
            },
            _ => return Err(ActorError::UnrecognizedPacketType),
        };
        Ok(())
    }
}

impl ActorEncode<FrameActorMsg> for FrameActor {
    fn encode(&self, _: &ActorRegistry) -> FrameActorMsg {
        // TODO: Handle other states
        let state = FrameState::OnStack;
        let async_cause = if let FrameState::OnStack = state {
            None
        } else {
            Some("await".into())
        };
        FrameActorMsg {
            actor: self.name(),
            type_: "call".into(),
            arguments: vec![],
            async_cause,
            display_name: "".into(), // TODO: get display name
            oldest: true,
            state,
            where_: FrameWhere {
                actor: self.source_actor.clone(),
                line: 1, // TODO: get from breakpoint?
                column: 1,
            },
        }
    }
}

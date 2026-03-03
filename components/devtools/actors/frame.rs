/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use atomic_refcell::AtomicRefCell;
use devtools_traits::FrameInfo;
use malloc_size_of_derive::MallocSizeOf;
use serde::Serialize;
use serde_json::{Map, Value};

use crate::StreamId;
use crate::actor::{Actor, ActorEncode, ActorError, ActorRegistry};
use crate::actors::environment::{EnvironmentActor, EnvironmentActorMsg};
use crate::actors::object::{ObjectActor, ObjectActorMsg};
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
    _Suspended,
    _Dead,
}

#[derive(Serialize)]
pub(crate) struct FrameWhere {
    actor: String,
    line: u32,
    column: u32,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct FrameActorMsg {
    actor: String,
    #[serde(rename = "type")]
    type_: String,
    arguments: Vec<Value>,
    async_cause: Option<String>,
    display_name: String,
    oldest: bool,
    state: FrameState,
    this: ObjectActorMsg,
    #[serde(rename = "where")]
    where_: FrameWhere,
}

/// Represents an stack frame. Used by `ThreadActor` when replying to interrupt messages.
/// <https://searchfox.org/firefox-main/source/devtools/server/actors/frame.js>
#[derive(MallocSizeOf)]
pub(crate) struct FrameActor {
    name: String,
    object_actor: String,
    source_actor: String,
    frame_result: FrameInfo,
    current_offset: AtomicRefCell<(u32, u32)>,
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
                    name: registry.new_name::<EnvironmentActor>(),
                    parent: None,
                };
                let msg = FrameEnvironmentReply {
                    from: self.name(),
                    environment: environment.encode(registry),
                };
                registry.register(environment);
                // This reply has a `type` field but it doesn't need a followup,
                // unlike most messages. We need to skip the validity check.
                request.reply_unchecked(&msg)?;
            },
            _ => return Err(ActorError::UnrecognizedPacketType),
        };
        Ok(())
    }
}

impl FrameActor {
    pub fn register(
        registry: &ActorRegistry,
        source_actor: String,
        frame_result: FrameInfo,
    ) -> String {
        let object_actor = ObjectActor::register(registry, None);

        let name = registry.new_name::<Self>();
        let actor = Self {
            name: name.clone(),
            object_actor,
            source_actor,
            frame_result,
            current_offset: Default::default(),
        };
        registry.register::<Self>(actor);
        name
    }

    pub(crate) fn set_offset(&self, column: u32, line: u32) {
        *self.current_offset.borrow_mut() = (column, line);
    }
}

impl ActorEncode<FrameActorMsg> for FrameActor {
    fn encode(&self, registry: &ActorRegistry) -> FrameActorMsg {
        // TODO: Handle other states
        let state = FrameState::OnStack;
        let async_cause = if let FrameState::OnStack = state {
            None
        } else {
            Some("await".into())
        };
        let (column, line) = *self.current_offset.borrow();
        // <https://searchfox.org/firefox-main/source/devtools/docs/user/debugger-api/debugger.frame/index.rst>
        FrameActorMsg {
            actor: self.name(),
            type_: self.frame_result.type_.clone(),
            arguments: vec![],
            async_cause,
            // TODO: Should be optional
            display_name: self.frame_result.display_name.clone(),
            this: registry.encode::<ObjectActor, _>(&self.object_actor),
            oldest: self.frame_result.oldest,
            state,
            where_: FrameWhere {
                actor: self.source_actor.clone(),
                line,
                column,
            },
        }
    }
}

/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::collections::HashSet;

use atomic_refcell::AtomicRefCell;
use base::generic_channel::GenericSender;
use devtools_traits::{DevtoolScriptControlMsg, PauseReason};
use malloc_size_of_derive::MallocSizeOf;
use serde::Serialize;
use serde_json::{Map, Value};

use super::source::{SourceManager, SourcesReply};
use crate::actor::{Actor, ActorError, ActorRegistry};
use crate::actors::frame::{FrameActor, FrameActorMsg};
use crate::actors::pause::PauseActor;
use crate::protocol::{ClientRequest, JsonPacketStream};
use crate::{EmptyReplyMsg, StreamId};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ThreadAttached {
    from: String,
    #[serde(rename = "type")]
    type_: String,
    actor: String,
    frame: u32,
    error: u32,
    recording_endpoint: u32,
    execution_point: u32,
    popped_frames: Vec<PoppedFrameMsg>,
    why: PauseReason,
}

#[derive(Serialize)]
enum PoppedFrameMsg {}

#[derive(Serialize)]
struct ThreadResumedReply {
    from: String,
    #[serde(rename = "type")]
    type_: String,
}

#[derive(Serialize)]
pub(crate) struct ThreadInterruptedReply {
    pub from: String,
    #[serde(rename = "type")]
    pub type_: String,
    pub actor: String,
    pub frame: FrameActorMsg,
    pub why: PauseReason,
}

#[derive(Serialize)]
struct GetAvailableEventBreakpointsReply {
    from: String,
    value: Vec<()>,
}

#[derive(Serialize)]
struct FramesReply {
    from: String,
    frames: Vec<FrameActorMsg>,
}

#[derive(MallocSizeOf)]
pub(crate) struct ThreadActor {
    name: String,
    pub source_manager: SourceManager,
    script_sender: GenericSender<DevtoolScriptControlMsg>,
    pub frames: AtomicRefCell<HashSet<String>>,
}

impl ThreadActor {
    pub fn new(name: String, script_sender: GenericSender<DevtoolScriptControlMsg>) -> ThreadActor {
        ThreadActor {
            name: name.clone(),
            source_manager: SourceManager::new(),
            script_sender,
            frames: Default::default(),
        }
    }
}

impl Actor for ThreadActor {
    fn name(&self) -> String {
        self.name.clone()
    }

    fn handle_message(
        &self,
        mut request: ClientRequest,
        registry: &ActorRegistry,
        msg_type: &str,
        _msg: &Map<String, Value>,
        _id: StreamId,
    ) -> Result<(), ActorError> {
        match msg_type {
            "attach" => {
                let pause = registry.new_name::<PauseActor>();
                registry.register(PauseActor {
                    name: pause.clone(),
                });
                let msg = ThreadAttached {
                    from: self.name(),
                    type_: "paused".to_owned(),
                    actor: pause,
                    frame: 0,
                    error: 0,
                    recording_endpoint: 0,
                    execution_point: 0,
                    popped_frames: vec![],
                    why: PauseReason {
                        type_: "attached".to_owned(),
                        on_next: None,
                    },
                };
                request.write_json_packet(&msg)?;
                request.reply_final(&EmptyReplyMsg { from: self.name() })?
            },

            "resume" => {
                let _ = self.script_sender.send(DevtoolScriptControlMsg::Resume);

                let msg = ThreadResumedReply {
                    from: self.name(),
                    type_: "resumed".to_owned(),
                };
                request.write_json_packet(&msg)?;
                request.reply_final(&EmptyReplyMsg { from: self.name() })?
            },

            "interrupt" => {
                self.script_sender
                    .send(DevtoolScriptControlMsg::Interrupt)
                    .map_err(|_| ActorError::Internal)?;

                request.reply_final(&EmptyReplyMsg { from: self.name() })?
            },

            "reconfigure" => request.reply_final(&EmptyReplyMsg { from: self.name() })?,

            "getAvailableEventBreakpoints" => {
                // TODO: Send list of available event breakpoints (animation, clipboard, load...)
                let msg = GetAvailableEventBreakpointsReply {
                    from: self.name(),
                    value: vec![],
                };
                request.reply_final(&msg)?
            },

            // Client has attached to the thread and wants to load script sources.
            // <https://firefox-source-docs.mozilla.org/devtools/backend/protocol.html#loading-script-sources>
            "sources" => {
                let msg = SourcesReply {
                    from: self.name(),
                    sources: self.source_manager.source_forms(registry),
                };
                request.reply_final(&msg)?
            },

            "frames" => {
                let msg = FramesReply {
                    from: self.name(),
                    frames: self
                        .frames
                        .borrow()
                        .iter()
                        .map(|frame| registry.encode::<FrameActor, _>(frame))
                        .collect(),
                };
                request.reply_final(&msg)?
            },

            _ => return Err(ActorError::UnrecognizedPacketType),
        };
        Ok(())
    }
}

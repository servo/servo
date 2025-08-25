/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use devtools_traits::DevtoolScriptControlMsg;
use ipc_channel::ipc::IpcSender;
use serde::Serialize;
use serde_json::{Map, Value};

use super::source::{SourceManager, SourcesReply};
use crate::actor::{Actor, ActorError, ActorRegistry};
use crate::actors::frame::FrameActor;
use crate::actors::object::ObjectActor;
use crate::actors::pause::PauseActor;
use crate::actors::source::SourceActor;
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
    why: WhyMsg,
}

#[derive(Debug, Serialize)]
struct WhyMsg {
    #[serde(rename = "type")]
    type_: String,
    #[serde(rename = "onNext")]
    on_next: bool,
}

#[derive(Serialize)]
struct ThreadResumedReply {
    from: String,
    #[serde(rename = "type")]
    type_: String,
}

#[derive(Debug, Serialize)]
struct ThreadInterruptedReply {
    actor:String,
    frame: FrameActor,
    from: String,
    #[serde(rename = "type")]
    type_: String,
    why: WhyMsg,
}


#[derive(Serialize)]
struct InterruptAck {
    from: String,
}

pub struct ThreadActor {
    pub name: String,
    pub source_manager: SourceManager,
    script_sender: IpcSender<DevtoolScriptControlMsg>,
}

impl ThreadActor {
    pub fn new(name: String, script_sender: IpcSender<DevtoolScriptControlMsg>,) -> ThreadActor {
        ThreadActor {
            name: name.clone(),
            source_manager: SourceManager::new(),
            script_sender,
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
                let msg = ThreadAttached {
                    from: self.name(),
                    type_: "paused".to_owned(),
                    actor: registry.new_name("pause"),
                    frame: 0,
                    error: 0,
                    recording_endpoint: 0,
                    execution_point: 0,
                    why: WhyMsg {
                        type_: "attached".to_owned(),
                        on_next: false,
                    },
                };
                request.write_json_packet(&msg)?;
                request.reply_final(&EmptyReplyMsg { from: self.name() })?
            },

            "resume" => {
                let msg = ThreadResumedReply {
                    from: self.name(),
                    type_: "resumed".to_owned(),
                };
                request.write_json_packet(&msg)?;
                request.reply_final(&EmptyReplyMsg { from: self.name() })?
            },

            "interrupt" => {
                self.script_sender
                    .send(DevtoolScriptControlMsg::Pause())
                    .map_err(|_| ActorError::Internal)?;

                // Next step:
                // investigate crash on "pause" click
                let pause_actor = registry.new_name("pause");

                let source_forms = self.source_manager.source_forms(registry);

                let source_actor = source_forms[0].actor.clone();

                // let object_uuid = format!("object-{}", self.name);
                // let object_actor_name = ObjectActor::register(registry, object_uuid);

                // let object_actor = registry.find::<ObjectActor>("object");

                let frame = FrameActor::new(
                    self.name.clone(),
                    source_actor,
                    5,
                    16,
                    // object_actor_name,
                 );

                let msg = ThreadInterruptedReply {
                    from: self.name(),
                    frame: frame,
                    type_: "paused".to_owned(),
                    actor: pause_actor.clone(),
                    why: WhyMsg {
                        type_: "interrupted".to_owned(),
                        on_next: true,
                    },
                };

                request.write_json_packet(&msg)?;
                let ack = InterruptAck { from: self.name()};
                request.reply_final(&ack)?;
            },

            "reconfigure" => request.reply_final(&EmptyReplyMsg { from: self.name() })?,

            // Client has attached to the thread and wants to load script sources.
            // <https://firefox-source-docs.mozilla.org/devtools/backend/protocol.html#loading-script-sources>
            "sources" => {
                let msg = SourcesReply {
                    from: self.name(),
                    sources: self.source_manager.source_forms(registry),
                };
                request.reply_final(&msg)?
            },
            _ => return Err(ActorError::UnrecognizedPacketType),
        };
        Ok(())
    }
}

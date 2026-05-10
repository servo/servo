/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::collections::HashSet;

use atomic_refcell::AtomicRefCell;
use devtools_traits::{DevtoolScriptControlMsg, PauseReason};
use malloc_size_of_derive::MallocSizeOf;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use servo_base::generic_channel::GenericSender;

use super::source::{SourceManager, SourcesReply};
use crate::actor::{Actor, ActorError, ActorRegistry};
use crate::actors::frame::{FrameActor, FrameActorMsg};
use crate::actors::pause::PauseActor;
use crate::generic_channel::channel;
use crate::protocol::{ClientRequest, JsonPacketStream};
use crate::{BrowsingContextActor, EmptyReplyMsg, StreamId};

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

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
enum ResumeLimitType {
    Break,
    Finish,
    Next,
    Restart,
    Step,
}

#[derive(Debug, Deserialize, Serialize)]
struct ResumeLimit {
    #[serde(rename = "type")]
    type_: ResumeLimitType,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct ResumeRequest {
    resume_limit: Option<ResumeLimit>,
    #[serde(rename = "frameActorID")]
    #[serde(skip_serializing_if = "Option::is_none")]
    frame_actor_id: Option<String>,
}

#[derive(Deserialize, Debug)]
struct FramesRequest {
    start: u32,
    count: u32,
}

impl ResumeRequest {
    fn get_type(&self) -> Option<String> {
        let resume_limit = self.resume_limit.as_ref()?;
        serde_json::to_string(&resume_limit.type_)
            .ok()
            .map(|s| s.trim_matches('"').into())
    }
}

#[derive(MallocSizeOf)]
pub(crate) struct ThreadActor {
    name: String,
    pub source_manager: SourceManager,
    script_sender: GenericSender<DevtoolScriptControlMsg>,
    pub frames: AtomicRefCell<HashSet<String>>,
    browsing_context_name: Option<String>,
}

impl ThreadActor {
    pub fn register(
        registry: &ActorRegistry,
        script_sender: GenericSender<DevtoolScriptControlMsg>,
        browsing_context_name: Option<String>,
    ) -> String {
        let name = registry.new_name::<Self>();
        let actor = ThreadActor {
            name: name.clone(),
            source_manager: SourceManager::new(),
            script_sender,
            frames: Default::default(),
            browsing_context_name,
        };
        registry.register::<Self>(actor);
        name
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
        msg: &Map<String, Value>,
        _id: StreamId,
    ) -> Result<(), ActorError> {
        match msg_type {
            "attach" => {
                let pause_name = PauseActor::register(registry);
                let msg = ThreadAttached {
                    from: self.name(),
                    type_: "paused".to_owned(),
                    actor: pause_name,
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
                let resume: ResumeRequest =
                    serde_json::from_value(msg.clone().into()).map_err(|_| ActorError::Internal)?;

                let _ = self.script_sender.send(DevtoolScriptControlMsg::Resume(
                    resume.get_type(),
                    resume.frame_actor_id,
                ));

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
                let Some(ref browsing_context_name) = self.browsing_context_name else {
                    return Err(ActorError::Internal);
                };
                let browsing_context_actor =
                    registry.find::<BrowsingContextActor>(browsing_context_name);

                let frames: FramesRequest =
                    serde_json::from_value(msg.clone().into()).map_err(|_| ActorError::Internal)?;

                let Some((tx, rx)) = channel() else {
                    return Err(ActorError::Internal);
                };
                self.script_sender
                    .send(DevtoolScriptControlMsg::ListFrames(
                        browsing_context_actor.pipeline_id(),
                        frames.start,
                        frames.count,
                        tx,
                    ))
                    .map_err(|_| ActorError::Internal)?;

                let result = rx.recv().map_err(|_| ActorError::Internal)?;
                // Frame actors should be registered here
                // https://searchfox.org/firefox-main/source/devtools/server/actors/thread.js#1425
                let msg = FramesReply {
                    from: self.name(),
                    frames: result
                        .iter()
                        .map(|frame_name| registry.encode::<FrameActor, _>(frame_name))
                        .collect(),
                };
                request.reply_final(&msg)?
            },

            _ => return Err(ActorError::UnrecognizedPacketType),
        };
        Ok(())
    }
}

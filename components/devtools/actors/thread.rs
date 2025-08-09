/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use servo_url::ServoUrl;

use super::source::{SourceManager, SourcesReply};
use crate::actor::{Actor, ActorError, ActorRegistry};
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
    why: WhyMsg,
}

#[derive(Serialize)]
enum PoppedFrameMsg {}

#[derive(Serialize)]
struct WhyMsg {
    #[serde(rename = "type")]
    type_: String,
}

#[derive(Serialize)]
struct ThreadResumedReply {
    from: String,
    #[serde(rename = "type")]
    type_: String,
}

#[derive(Serialize)]
struct ThreadInterruptedReply {
    from: String,
    #[serde(rename = "type")]
    type_: String,
}

pub struct ThreadActor {
    pub name: String,
    pub source_manager: SourceManager,
}

impl ThreadActor {
    pub fn new(name: String) -> ThreadActor {
        ThreadActor {
            name: name.clone(),
            source_manager: SourceManager::new(),
        }
    }
}

/// Location inside a source.
///
/// This can be in the code for the source itself, or the code of an `eval()` or `new Function()`
/// called from that source.
///
/// <https://firefox-source-docs.mozilla.org/devtools/backend/protocol.html#source-locations>
#[derive(Deserialize, Serialize)]
#[serde(untagged)]
pub enum Location {
    Url {
        url: ServoUrl,
        line: usize,
        column: usize,
    },
    Eval {
        eval: Box<Location>,
        id: String,
        line: usize,
        column: usize,
    },
    Function {
        function: Box<Location>,
        id: String,
        line: usize,
        column: usize,
    },
}

#[derive(Serialize)]
struct GetAvailableEventBreakpointsReply {
    from: String,
    value: Vec<()>,
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
                    popped_frames: vec![],
                    why: WhyMsg {
                        type_: "attached".to_owned(),
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
                let msg = ThreadInterruptedReply {
                    from: self.name(),
                    type_: "interrupted".to_owned(),
                };
                request.write_json_packet(&msg)?;
                request.reply_final(&EmptyReplyMsg { from: self.name() })?
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

            // Client wants to know what Event Listener Breakpoints are available for this thread.
            "getAvailableEventBreakpoints" => {
                let msg = GetAvailableEventBreakpointsReply {
                    from: self.name(),
                    // TODO: populate this.
                    value: vec![],
                };
                request.reply_final(&msg)?
            },

            _ => return Err(ActorError::UnrecognizedPacketType),
        };
        Ok(())
    }
}

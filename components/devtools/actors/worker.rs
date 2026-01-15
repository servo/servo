/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::collections::HashMap;
use std::net::TcpStream;

use atomic_refcell::AtomicRefCell;
use base::generic_channel::GenericSender;
use base::id::TEST_PIPELINE_ID;
use devtools_traits::DevtoolScriptControlMsg::WantsLiveNotifications;
use devtools_traits::{DevtoolScriptControlMsg, WorkerId};
use serde::Serialize;
use serde_json::{Map, Value};
use servo_url::ServoUrl;

use crate::StreamId;
use crate::actor::{Actor, ActorEncode, ActorError, ActorRegistry};
use crate::protocol::{ClientRequest, JsonPacketStream};
use crate::resource::ResourceAvailable;

#[derive(Clone, Copy)]
#[expect(dead_code)]
pub enum WorkerType {
    Dedicated = 0,
    Shared = 1,
    Service = 2,
}

pub(crate) struct WorkerActor {
    pub(crate) name: String,
    pub(crate) console: String,
    pub(crate) thread: String,
    pub(crate) worker_id: WorkerId,
    pub(crate) url: ServoUrl,
    pub(crate) type_: WorkerType,
    pub(crate) script_chan: GenericSender<DevtoolScriptControlMsg>,
    pub(crate) streams: AtomicRefCell<HashMap<StreamId, TcpStream>>,
}

impl ResourceAvailable for WorkerActor {
    fn actor_name(&self) -> String {
        self.name.clone()
    }
}

impl Actor for WorkerActor {
    fn name(&self) -> String {
        self.name.clone()
    }
    fn handle_message(
        &self,
        mut request: ClientRequest,
        _registry: &ActorRegistry,
        msg_type: &str,
        _msg: &Map<String, Value>,
        stream_id: StreamId,
    ) -> Result<(), ActorError> {
        match msg_type {
            "attach" => {
                let msg = AttachedReply {
                    from: self.name(),
                    type_: "attached".to_owned(),
                    url: self.url.as_str().to_owned(),
                };
                // FIXME: we don’t send an actual reply (message without type), which seems to be a bug?
                request.write_json_packet(&msg)?;
                self.streams
                    .borrow_mut()
                    .insert(stream_id, request.try_clone_stream().unwrap());
                // FIXME: fix messages to not require forging a pipeline for worker messages
                self.script_chan
                    .send(WantsLiveNotifications(TEST_PIPELINE_ID, true))
                    .unwrap();
            },

            "connect" => {
                let msg = ConnectReply {
                    from: self.name(),
                    type_: "connected".to_owned(),
                    thread_actor: self.thread.clone(),
                    console_actor: self.console.clone(),
                };
                // FIXME: we don’t send an actual reply (message without type), which seems to be a bug?
                request.write_json_packet(&msg)?;
            },

            "detach" => {
                let msg = DetachedReply {
                    from: self.name(),
                    type_: "detached".to_string(),
                };
                self.cleanup(stream_id);
                // FIXME: we don’t send an actual reply (message without type), which seems to be a bug?
                request.write_json_packet(&msg)?;
            },

            _ => return Err(ActorError::UnrecognizedPacketType),
        };
        Ok(())
    }

    fn cleanup(&self, stream_id: StreamId) {
        self.streams.borrow_mut().remove(&stream_id);
        if self.streams.borrow().is_empty() {
            self.script_chan
                .send(WantsLiveNotifications(TEST_PIPELINE_ID, false))
                .unwrap();
        }
    }
}

#[derive(Serialize)]
struct DetachedReply {
    from: String,
    #[serde(rename = "type")]
    type_: String,
}

#[derive(Serialize)]
struct AttachedReply {
    from: String,
    #[serde(rename = "type")]
    type_: String,
    url: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ConnectReply {
    from: String,
    #[serde(rename = "type")]
    type_: String,
    thread_actor: String,
    console_actor: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct WorkerTraits {
    is_parent_intercept_enabled: bool,
    supports_top_level_target_flag: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct WorkerActorMsg {
    actor: String,
    console_actor: String,
    thread_actor: String,
    id: String,
    url: String,
    traits: WorkerTraits,
    #[serde(rename = "type")]
    type_: u32,
    #[serde(rename = "targetType")]
    target_type: String,
}

impl ActorEncode<WorkerActorMsg> for WorkerActor {
    fn encode(&self, _: &ActorRegistry) -> WorkerActorMsg {
        WorkerActorMsg {
            actor: self.name(),
            console_actor: self.console.clone(),
            thread_actor: self.thread.clone(),
            id: self.worker_id.0.to_string(),
            url: self.url.to_string(),
            traits: WorkerTraits {
                is_parent_intercept_enabled: false,
                supports_top_level_target_flag: false,
            },
            type_: self.type_ as u32,
            target_type: "worker".to_string(),
        }
    }
}

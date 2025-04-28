/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::RefCell;
use std::collections::HashMap;
use std::net::TcpStream;

use base::id::TEST_PIPELINE_ID;
use devtools_traits::DevtoolScriptControlMsg::WantsLiveNotifications;
use devtools_traits::{DevtoolScriptControlMsg, WorkerId};
use ipc_channel::ipc::IpcSender;
use serde::Serialize;
use serde_json::{Map, Value};
use servo_url::ServoUrl;

use crate::StreamId;
use crate::actor::{Actor, ActorMessageStatus, ActorRegistry};
use crate::protocol::JsonPacketStream;
use crate::resource::{ResourceAvailable, ResourceAvailableReply};

#[derive(Clone, Copy)]
#[allow(dead_code)]
pub enum WorkerType {
    Dedicated = 0,
    Shared = 1,
    Service = 2,
}

pub(crate) struct WorkerActor {
    pub name: String,
    pub console: String,
    pub thread: String,
    pub worker_id: WorkerId,
    pub url: ServoUrl,
    pub type_: WorkerType,
    pub script_chan: IpcSender<DevtoolScriptControlMsg>,
    pub streams: RefCell<HashMap<StreamId, TcpStream>>,
}

impl WorkerActor {
    pub(crate) fn encodable(&self) -> WorkerMsg {
        WorkerMsg {
            actor: self.name.clone(),
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

impl ResourceAvailable for WorkerActor {
    fn actor_name(&self) -> String {
        self.name.clone()
    }

    fn get_streams(&self) -> &RefCell<HashMap<StreamId, TcpStream>> {
        &self.streams
    }
}

impl Actor for WorkerActor {
    fn name(&self) -> String {
        self.name.clone()
    }
    fn handle_message(
        &self,
        _registry: &ActorRegistry,
        msg_type: &str,
        _msg: &Map<String, Value>,
        stream: &mut TcpStream,
        stream_id: StreamId,
    ) -> Result<ActorMessageStatus, ()> {
        Ok(match msg_type {
            "attach" => {
                let msg = AttachedReply {
                    from: self.name(),
                    type_: "attached".to_owned(),
                    url: self.url.as_str().to_owned(),
                };
                if stream.write_json_packet(&msg).is_err() {
                    return Ok(ActorMessageStatus::Processed);
                }
                self.streams
                    .borrow_mut()
                    .insert(stream_id, stream.try_clone().unwrap());
                // FIXME: fix messages to not require forging a pipeline for worker messages
                self.script_chan
                    .send(WantsLiveNotifications(TEST_PIPELINE_ID, true))
                    .unwrap();
                ActorMessageStatus::Processed
            },

            "connect" => {
                let msg = ConnectReply {
                    from: self.name(),
                    type_: "connected".to_owned(),
                    thread_actor: self.thread.clone(),
                    console_actor: self.console.clone(),
                };
                let _ = stream.write_json_packet(&msg);
                ActorMessageStatus::Processed
            },

            "detach" => {
                let msg = DetachedReply {
                    from: self.name(),
                    type_: "detached".to_string(),
                };
                let _ = stream.write_json_packet(&msg);
                self.cleanup(stream_id);
                ActorMessageStatus::Processed
            },

            _ => ActorMessageStatus::Ignored,
        })
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

impl WorkerActor {
    pub(crate) fn resource_available<T: Serialize>(&self, resource: T, resource_type: String) {
        self.resources_available(vec![resource], resource_type);
    }

    pub(crate) fn resources_available<T: Serialize>(
        &self,
        resources: Vec<T>,
        resource_type: String,
    ) {
        let msg = ResourceAvailableReply::<T> {
            from: self.name(),
            type_: "resources-available-array".into(),
            array: vec![(resource_type, resources)],
        };

        for stream in self.streams.borrow_mut().values_mut() {
            let _ = stream.write_json_packet(&msg);
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
pub(crate) struct WorkerMsg {
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

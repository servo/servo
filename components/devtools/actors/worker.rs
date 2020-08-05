/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::actor::{Actor, ActorMessageStatus, ActorRegistry};
use crate::protocol::JsonPacketStream;
use crate::StreamId;
use devtools_traits::DevtoolScriptControlMsg::WantsLiveNotifications;
use devtools_traits::{DevtoolScriptControlMsg, WorkerId};
use ipc_channel::ipc::IpcSender;
use msg::constellation_msg::TEST_PIPELINE_ID;
use serde_json::{Map, Value};
use servo_url::ServoUrl;
use std::cell::RefCell;
use std::collections::HashMap;
use std::net::TcpStream;

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
    pub id: WorkerId,
    pub url: ServoUrl,
    pub type_: WorkerType,
    pub script_chan: IpcSender<DevtoolScriptControlMsg>,
    pub streams: RefCell<HashMap<StreamId, TcpStream>>,
}

impl WorkerActor {
    pub(crate) fn encodable(&self) -> WorkerMsg {
        WorkerMsg {
            actor: self.name.clone(),
            consoleActor: self.console.clone(),
            threadActor: self.thread.clone(),
            id: self.id.0.to_string(),
            url: self.url.to_string(),
            traits: WorkerTraits {
                isParentInterceptEnabled: false,
            },
            type_: self.type_ as u32,
        }
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
        id: StreamId,
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
                    .insert(id, stream.try_clone().unwrap());
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
                    threadActor: self.thread.clone(),
                    consoleActor: self.console.clone(),
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
                self.cleanup(id);
                ActorMessageStatus::Processed
            },

            _ => ActorMessageStatus::Ignored,
        })
    }

    fn cleanup(&self, id: StreamId) {
        self.streams.borrow_mut().remove(&id);
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
struct ConnectReply {
    from: String,
    #[serde(rename = "type")]
    type_: String,
    threadActor: String,
    consoleActor: String,
}

#[derive(Serialize)]
struct WorkerTraits {
    isParentInterceptEnabled: bool,
}

#[derive(Serialize)]
pub(crate) struct WorkerMsg {
    actor: String,
    consoleActor: String,
    threadActor: String,
    id: String,
    url: String,
    traits: WorkerTraits,
    #[serde(rename = "type")]
    type_: u32,
}

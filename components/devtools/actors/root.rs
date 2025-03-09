/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Connection point for all new remote devtools interactions, providing lists of know actors
//! that perform more specific actions (targets, addons, browser chrome, etc.)
//!
//! Liberally derived from the [Firefox JS implementation].
//!
//! [Firefox JS implementation]: https://searchfox.org/mozilla-central/source/devtools/server/actors/root.js

use std::net::TcpStream;

use serde::Serialize;
use serde_json::{Map, Value, json};

use crate::StreamId;
use crate::actor::{Actor, ActorMessageStatus, ActorRegistry};
use crate::actors::device::DeviceActor;
use crate::actors::performance::PerformanceActor;
use crate::actors::process::{ProcessActor, ProcessActorMsg};
use crate::actors::tab::{TabDescriptorActor, TabDescriptorActorMsg};
use crate::actors::worker::{WorkerActor, WorkerMsg};
use crate::protocol::{ActorDescription, JsonPacketStream};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ActorTraits {
    sources: bool,
    highlightable: bool,
    custom_highlighters: bool,
    network_monitor: bool,
}

#[derive(Serialize)]
struct ListAddonsReply {
    from: String,
    addons: Vec<AddonMsg>,
}

#[derive(Serialize)]
enum AddonMsg {}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct GetRootReply {
    from: String,
    selected: u32,
    performance_actor: String,
    device_actor: String,
    preference_actor: String,
}

#[derive(Serialize)]
struct ListTabsReply {
    from: String,
    tabs: Vec<TabDescriptorActorMsg>,
}

#[derive(Serialize)]
struct GetTabReply {
    from: String,
    tab: TabDescriptorActorMsg,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RootActorMsg {
    from: String,
    application_type: String,
    traits: ActorTraits,
}

#[derive(Serialize)]
pub struct ProtocolDescriptionReply {
    from: String,
    types: Types,
}

#[derive(Serialize)]
struct ListWorkersReply {
    from: String,
    workers: Vec<WorkerMsg>,
}

#[derive(Serialize)]
struct ListServiceWorkerRegistrationsReply {
    from: String,
    registrations: Vec<u32>, // TODO: follow actual JSON structure.
}

#[derive(Serialize)]
pub struct Types {
    performance: ActorDescription,
    device: ActorDescription,
}

#[derive(Serialize)]
struct ListProcessesResponse {
    from: String,
    processes: Vec<ProcessActorMsg>,
}

#[derive(Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DescriptorTraits {
    pub(crate) watcher: bool,
    pub(crate) supports_reload_descriptor: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct GetProcessResponse {
    from: String,
    process_descriptor: ProcessActorMsg,
}

#[derive(Serialize)]
struct ErrorResponse {
    from: String,
    error: String,
    message: String,
}

pub struct RootActor {
    pub tabs: Vec<String>,
    pub workers: Vec<String>,
    pub performance: String,
    pub device: String,
    pub preference: String,
    pub process: String,
}

impl Actor for RootActor {
    fn name(&self) -> String {
        "root".to_owned()
    }

    fn handle_message(
        &self,
        registry: &ActorRegistry,
        msg_type: &str,
        msg: &Map<String, Value>,
        stream: &mut TcpStream,
        _id: StreamId,
    ) -> Result<ActorMessageStatus, ()> {
        Ok(match msg_type {
            "connect" => {
                let message = json!({
                    "from": "root",
                });
                let _ = stream.write_json_packet(&message);
                ActorMessageStatus::Processed
            },
            "listAddons" => {
                let actor = ListAddonsReply {
                    from: "root".to_owned(),
                    addons: vec![],
                };
                let _ = stream.write_json_packet(&actor);
                ActorMessageStatus::Processed
            },

            "listProcesses" => {
                let process = registry.find::<ProcessActor>(&self.process).encodable();
                let reply = ListProcessesResponse {
                    from: self.name(),
                    processes: vec![process],
                };
                let _ = stream.write_json_packet(&reply);
                ActorMessageStatus::Processed
            },

            // TODO: Unexpected message getTarget for process (when inspecting)
            "getProcess" => {
                let process = registry.find::<ProcessActor>(&self.process).encodable();
                let reply = GetProcessResponse {
                    from: self.name(),
                    process_descriptor: process,
                };
                let _ = stream.write_json_packet(&reply);
                ActorMessageStatus::Processed
            },

            "getRoot" => {
                let actor = GetRootReply {
                    from: "root".to_owned(),
                    selected: 0,
                    performance_actor: self.performance.clone(),
                    device_actor: self.device.clone(),
                    preference_actor: self.preference.clone(),
                };
                let _ = stream.write_json_packet(&actor);
                ActorMessageStatus::Processed
            },

            "listTabs" => {
                let actor = ListTabsReply {
                    from: "root".to_owned(),
                    tabs: self
                        .tabs
                        .iter()
                        .filter_map(|target| {
                            let tab_actor = registry.find::<TabDescriptorActor>(target);
                            // Filter out iframes and workers
                            if tab_actor.is_top_level_global() {
                                Some(tab_actor.encodable(registry, false))
                            } else {
                                None
                            }
                        })
                        .collect(),
                };
                let _ = stream.write_json_packet(&actor);
                ActorMessageStatus::Processed
            },

            "listServiceWorkerRegistrations" => {
                let reply = ListServiceWorkerRegistrationsReply {
                    from: self.name(),
                    registrations: vec![],
                };
                let _ = stream.write_json_packet(&reply);
                ActorMessageStatus::Processed
            },

            "listWorkers" => {
                let reply = ListWorkersReply {
                    from: self.name(),
                    workers: self
                        .workers
                        .iter()
                        .map(|name| registry.find::<WorkerActor>(name).encodable())
                        .collect(),
                };
                let _ = stream.write_json_packet(&reply);
                ActorMessageStatus::Processed
            },

            "getTab" => {
                let Some(serde_json::Value::Number(browser_id)) = msg.get("browserId") else {
                    return Ok(ActorMessageStatus::Ignored);
                };

                let browser_id = browser_id.as_u64().unwrap();
                let Some(tab) = self.get_tab_msg_by_browser_id(registry, browser_id as u32) else {
                    return Ok(ActorMessageStatus::Ignored);
                };

                let reply = GetTabReply {
                    from: self.name(),
                    tab,
                };
                let _ = stream.write_json_packet(&reply);
                ActorMessageStatus::Processed
            },

            "protocolDescription" => {
                let msg = ProtocolDescriptionReply {
                    from: self.name(),
                    types: Types {
                        performance: PerformanceActor::description(),
                        device: DeviceActor::description(),
                    },
                };
                let _ = stream.write_json_packet(&msg);
                ActorMessageStatus::Processed
            },

            _ => {
                let reply = ErrorResponse {
                    from: self.name(),
                    error: "unrecognizedPacketType".to_owned(),
                    message: format!(
                        "Actor {} does not recognize the packet type '{}'",
                        self.name(),
                        msg_type,
                    ),
                };
                let _ = stream.write_json_packet(&reply);
                ActorMessageStatus::Ignored
            },
        })
    }
}

impl RootActor {
    pub fn encodable(&self) -> RootActorMsg {
        RootActorMsg {
            from: "root".to_owned(),
            application_type: "browser".to_owned(),
            traits: ActorTraits {
                sources: false,
                highlightable: true,
                custom_highlighters: true,
                network_monitor: false,
            },
        }
    }

    fn get_tab_msg_by_browser_id(
        &self,
        registry: &ActorRegistry,
        browser_id: u32,
    ) -> Option<TabDescriptorActorMsg> {
        self.tabs
            .iter()
            .map(|target| {
                registry
                    .find::<TabDescriptorActor>(target)
                    .encodable(registry, true)
            })
            .find(|tab| tab.id() == browser_id)
    }
}

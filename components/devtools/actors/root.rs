/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::net::TcpStream;

use serde::Serialize;
use serde_json::{Map, Value};

/// Liberally derived from the [Firefox JS implementation]
/// (http://mxr.mozilla.org/mozilla-central/source/toolkit/devtools/server/actors/root.js).
/// Connection point for all new remote devtools interactions, providing lists of know actors
/// that perform more specific actions (targets, addons, browser chrome, etc.)
use crate::actor::{Actor, ActorMessageStatus, ActorRegistry};
use crate::actors::device::DeviceActor;
use crate::actors::performance::PerformanceActor;
use crate::actors::tab::{TabDescriptorActor, TabDescriptorActorMsg};
use crate::actors::worker::{WorkerActor, WorkerMsg};
use crate::protocol::{ActorDescription, JsonPacketStream};
use crate::StreamId;

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
    selected: u32,
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
    processes: Vec<ProcessForm>,
}

#[derive(Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DescriptorTraits {
    pub(crate) watcher: bool,
    pub(crate) supports_reload_descriptor: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ProcessForm {
    actor: String,
    id: u32,
    is_parent: bool,
    is_windowless_parent: bool,
    traits: DescriptorTraits,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct GetProcessResponse {
    from: String,
    process_descriptor: ProcessForm,
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
            "listAddons" => {
                let actor = ListAddonsReply {
                    from: "root".to_owned(),
                    addons: vec![],
                };
                let _ = stream.write_json_packet(&actor);
                ActorMessageStatus::Processed
            },

            "listProcesses" => {
                let reply = ListProcessesResponse {
                    from: self.name(),
                    processes: vec![ProcessForm {
                        actor: self.process.clone(),
                        id: 0,
                        is_parent: true,
                        is_windowless_parent: false,
                        traits: Default::default(),
                    }],
                };
                let _ = stream.write_json_packet(&reply);
                ActorMessageStatus::Processed
            },

            "getProcess" => {
                let reply = GetProcessResponse {
                    from: self.name(),
                    process_descriptor: ProcessForm {
                        actor: self.process.clone(),
                        id: 0,
                        is_parent: true,
                        is_windowless_parent: false,
                        traits: Default::default(),
                    },
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

            // https://firefox-source-docs.mozilla.org/devtools/backend/protocol.html#listing-browser-tabs
            "listTabs" => {
                let actor = ListTabsReply {
                    from: "root".to_owned(),
                    selected: 0,
                    tabs: self
                        .tabs
                        .iter()
                        .map(|target| {
                            registry
                                .find::<TabDescriptorActor>(target)
                                .encodable(registry, false)
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
                if let Some(serde_json::Value::Number(browser_id)) = msg.get("browserId") {
                    let target_tab = self
                        .tabs
                        .iter()
                        .map(|target| {
                            registry
                                .find::<TabDescriptorActor>(target)
                                .encodable(registry, true)
                        })
                        .find(|tab| tab.id() as u64 == browser_id.as_u64().unwrap());

                    if let Some(tab) = target_tab {
                        let reply = GetTabReply {
                            from: self.name(),
                            tab: tab,
                        };
                        let _ = stream.write_json_packet(&reply);
                        ActorMessageStatus::Processed
                    } else {
                        ActorMessageStatus::Ignored
                    }
                } else {
                    ActorMessageStatus::Ignored
                }
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

            _ => ActorMessageStatus::Ignored,
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
}

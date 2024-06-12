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
struct ActorTraits {
    sources: bool,
    highlightable: bool,
    customHighlighters: bool,
    networkMonitor: bool,
}

#[derive(Serialize)]
struct ListAddonsReply {
    from: String,
    addons: Vec<AddonMsg>,
}

#[derive(Serialize)]
enum AddonMsg {}

#[derive(Serialize)]
struct GetRootReply {
    from: String,
    selected: u32,
    performanceActor: String,
    deviceActor: String,
    preferenceActor: String,
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
pub struct RootActorMsg {
    from: String,
    applicationType: String,
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
pub struct DescriptorTraits {
    pub(crate) watcher: bool,
    #[serde(rename(serialize = "supportsReloadDescriptor"))]
    pub(crate) supports_reload_descriptor: bool,
}

#[derive(Serialize)]
struct ProcessForm {
    actor: String,
    id: u32,
    isParent: bool,
    isWindowlessParent: bool,
    traits: DescriptorTraits,
}

#[derive(Serialize)]
struct GetProcessResponse {
    from: String,
    processDescriptor: ProcessForm,
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
                        isParent: true,
                        isWindowlessParent: false,
                        traits: Default::default(),
                    }],
                };
                let _ = stream.write_json_packet(&reply);
                ActorMessageStatus::Processed
            },

            "getProcess" => {
                let reply = GetProcessResponse {
                    from: self.name(),
                    processDescriptor: ProcessForm {
                        actor: self.process.clone(),
                        id: 0,
                        isParent: true,
                        isWindowlessParent: false,
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
                    performanceActor: self.performance.clone(),
                    deviceActor: self.device.clone(),
                    preferenceActor: self.preference.clone(),
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
                        .find(|tab| tab.id() as u64 == browserId.as_u64().unwrap());

                    if let Some(tab) = targetTab {
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
            applicationType: "browser".to_owned(),
            traits: ActorTraits {
                sources: false,
                highlightable: true,
                customHighlighters: true,
                networkMonitor: false,
            },
        }
    }
}

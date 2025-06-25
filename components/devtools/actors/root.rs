/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Connection point for all new remote devtools interactions, providing lists of know actors
//! that perform more specific actions (targets, addons, browser chrome, etc.)
//!
//! Liberally derived from the [Firefox JS implementation].
//!
//! [Firefox JS implementation]: https://searchfox.org/mozilla-central/source/devtools/server/actors/root.js

use std::cell::RefCell;
use std::net::TcpStream;

use serde::Serialize;
use serde_json::{Map, Value, json};

use crate::StreamId;
use crate::actor::{Actor, ActorError, ActorRegistry};
use crate::actors::device::DeviceActor;
use crate::actors::performance::PerformanceActor;
use crate::actors::process::{ProcessActor, ProcessActorMsg};
use crate::actors::tab::{TabDescriptorActor, TabDescriptorActorMsg};
use crate::actors::worker::{WorkerActor, WorkerMsg};
use crate::protocol::{ActorDescription, ActorReplied, JsonPacketStream};

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

pub struct RootActor {
    pub tabs: Vec<String>,
    pub workers: Vec<String>,
    pub performance: String,
    pub device: String,
    pub preference: String,
    pub process: String,
    pub active_tab: RefCell<Option<String>>,
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
    ) -> Result<ActorReplied, ActorError> {
        Ok(match msg_type {
            "connect" => {
                let message = json!({
                    "from": "root",
                });
                stream.write_json_packet(&message)?
            },
            "listAddons" => {
                let actor = ListAddonsReply {
                    from: "root".to_owned(),
                    addons: vec![],
                };
                stream.write_json_packet(&actor)?
            },

            "listProcesses" => {
                let process = registry.find::<ProcessActor>(&self.process).encodable();
                let reply = ListProcessesResponse {
                    from: self.name(),
                    processes: vec![process],
                };
                stream.write_json_packet(&reply)?
            },

            // TODO: Unexpected message getTarget for process (when inspecting)
            "getProcess" => {
                let process = registry.find::<ProcessActor>(&self.process).encodable();
                let reply = GetProcessResponse {
                    from: self.name(),
                    process_descriptor: process,
                };
                stream.write_json_packet(&reply)?
            },

            "getRoot" => {
                let actor = GetRootReply {
                    from: "root".to_owned(),
                    selected: 0,
                    performance_actor: self.performance.clone(),
                    device_actor: self.device.clone(),
                    preference_actor: self.preference.clone(),
                };
                stream.write_json_packet(&actor)?
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
                stream.write_json_packet(&actor)?
            },

            "listServiceWorkerRegistrations" => {
                let reply = ListServiceWorkerRegistrationsReply {
                    from: self.name(),
                    registrations: vec![],
                };
                stream.write_json_packet(&reply)?
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
                stream.write_json_packet(&reply)?
            },

            "getTab" => {
                let Some(serde_json::Value::Number(browser_id)) = msg.get("browserId") else {
                    //  is this correct error?
                    return Err(ActorError::MissingParameter);
                };

                let browser_id = browser_id.as_u64().unwrap();
                let Some(tab) = self.get_tab_msg_by_browser_id(registry, browser_id as u32) else {
                    //  is this correct error?
                    return Err(ActorError::MissingParameter);
                };

                let reply = GetTabReply {
                    from: self.name(),
                    tab,
                };
                stream.write_json_packet(&reply)?
            },

            "protocolDescription" => {
                let msg = ProtocolDescriptionReply {
                    from: self.name(),
                    types: Types {
                        performance: PerformanceActor::description(),
                        device: DeviceActor::description(),
                    },
                };
                stream.write_json_packet(&msg)?
            },

            _ => return Err(ActorError::UnrecognizedPacketType),
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
                network_monitor: true,
            },
        }
    }

    fn get_tab_msg_by_browser_id(
        &self,
        registry: &ActorRegistry,
        browser_id: u32,
    ) -> Option<TabDescriptorActorMsg> {
        let tab_msg = self
            .tabs
            .iter()
            .map(|target| {
                registry
                    .find::<TabDescriptorActor>(target)
                    .encodable(registry, true)
            })
            .find(|tab| tab.browser_id() == browser_id);

        if let Some(ref msg) = tab_msg {
            *self.active_tab.borrow_mut() = Some(msg.actor());
        }
        tab_msg
    }

    #[allow(dead_code)]
    pub fn active_tab(&self) -> Option<String> {
        self.active_tab.borrow().clone()
    }
}

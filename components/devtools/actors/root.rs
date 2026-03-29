/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Connection point for all new remote devtools interactions, providing lists of know actors
//! that perform more specific actions (targets, addons, browser chrome, etc.)
//!
//! Liberally derived from the [Firefox JS implementation].
//!
//! [Firefox JS implementation]: https://searchfox.org/mozilla-central/source/devtools/server/actors/root.js

use std::collections::HashMap;

use atomic_refcell::AtomicRefCell;
use malloc_size_of_derive::MallocSizeOf;
use serde::Serialize;
use serde_json::{Map, Value};

use crate::actor::{Actor, ActorEncode, ActorError, ActorRegistry};
use crate::actors::device::DeviceActor;
use crate::actors::performance::PerformanceActor;
use crate::actors::preference::PreferenceActor;
use crate::actors::process::{ProcessActor, ProcessActorMsg};
use crate::actors::tab::{TabDescriptorActor, TabDescriptorActorMsg};
use crate::actors::worker::{WorkerActor, WorkerActorMsg};
use crate::protocol::{ActorDescription, ClientRequest};
use crate::{EmptyReplyMsg, StreamId};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ServiceWorkerInfo {
    actor: String,
    url: String,
    state: u32,
    state_text: String,
    id: String,
    fetch: bool,
    traits: HashMap<&'static str, bool>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ServiceWorkerRegistrationMsg {
    actor: String,
    scope: String,
    url: String,
    registration_state: String,
    last_update_time: u64,
    traits: HashMap<&'static str, bool>,
    // Firefox DevTools (LegacyServiceWorkersWatcher) matches workers via these
    // four named fields, not via a `workers` array.
    evaluating_worker: Option<ServiceWorkerInfo>,
    installing_worker: Option<ServiceWorkerInfo>,
    waiting_worker: Option<ServiceWorkerInfo>,
    active_worker: Option<ServiceWorkerInfo>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct RootTraits {
    sources: bool,
    highlightable: bool,
    custom_highlighters: bool,
    network_monitor: bool,
    resources: HashMap<&'static str, bool>,
}

#[derive(Serialize)]
struct ListAddonsReply {
    from: String,
    addons: Vec<AddonMsg>,
}

#[derive(Serialize)]
enum AddonMsg {}

#[derive(Clone, Default, Serialize, MallocSizeOf)]
#[serde(rename_all = "camelCase")]
struct GlobalActors {
    device_actor: String,
    perf_actor: String,
    preference_actor: String,
    // Not implemented in Servo
    // addons_actor
    // heap_snapshot_file_actor
    // parent_accessibility_actor
    // screenshot_actor
}

#[derive(Serialize)]
struct GetRootReply {
    from: String,
    #[serde(flatten)]
    global_actors: GlobalActors,
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
pub(crate) struct RootActorMsg {
    from: String,
    application_type: String,
    traits: RootTraits,
}

#[derive(Serialize)]
pub(crate) struct ProtocolDescriptionReply {
    from: String,
    types: Types,
}

#[derive(Serialize)]
struct ListWorkersReply {
    from: String,
    workers: Vec<WorkerActorMsg>,
}

#[derive(Serialize)]
struct ListServiceWorkerRegistrationsReply {
    from: String,
    registrations: Vec<ServiceWorkerRegistrationMsg>,
}

#[derive(Serialize)]
pub(crate) struct Types {
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
pub(crate) struct DescriptorTraits {
    pub(crate) watcher: bool,
    pub(crate) supports_reload_descriptor: bool,
    pub(crate) supports_navigation: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct GetProcessResponse {
    from: String,
    process_descriptor: ProcessActorMsg,
}

#[derive(Default, MallocSizeOf)]
pub(crate) struct RootActor {
    active_tab: AtomicRefCell<Option<String>>,
    global_actors: GlobalActors,
    process: String,
    pub tabs: AtomicRefCell<Vec<String>>,
    pub workers: AtomicRefCell<Vec<String>>,
    pub service_workers: AtomicRefCell<Vec<String>>,
}

impl Actor for RootActor {
    fn name(&self) -> String {
        "root".to_owned()
    }

    fn handle_message(
        &self,
        request: ClientRequest,
        registry: &ActorRegistry,
        msg_type: &str,
        msg: &Map<String, Value>,
        _id: StreamId,
    ) -> Result<(), ActorError> {
        match msg_type {
            "connect" => {
                let message = EmptyReplyMsg {
                    from: "root".into(),
                };
                request.reply_final(&message)?
            },

            // TODO: Unexpected message getTarget for process (when inspecting)
            "getProcess" => {
                let process = registry.encode::<ProcessActor, _>(&self.process);
                let reply = GetProcessResponse {
                    from: self.name(),
                    process_descriptor: process,
                };
                request.reply_final(&reply)?
            },

            "getRoot" => {
                let reply = GetRootReply {
                    from: "root".to_owned(),
                    global_actors: self.global_actors.clone(),
                };
                request.reply_final(&reply)?
            },

            "getTab" => {
                let browser_id = msg
                    .get("browserId")
                    .ok_or(ActorError::MissingParameter)?
                    .as_u64()
                    .ok_or(ActorError::BadParameterType)?;
                let Some(tab) = self.get_tab_msg_by_browser_id(registry, browser_id as u32) else {
                    return Err(ActorError::Internal);
                };

                let reply = GetTabReply {
                    from: self.name(),
                    tab,
                };
                request.reply_final(&reply)?
            },

            "listAddons" => {
                let reply = ListAddonsReply {
                    from: "root".to_owned(),
                    addons: vec![],
                };
                request.reply_final(&reply)?
            },

            "listProcesses" => {
                let process = registry.encode::<ProcessActor, _>(&self.process);
                let reply = ListProcessesResponse {
                    from: self.name(),
                    processes: vec![process],
                };
                request.reply_final(&reply)?
            },

            "listServiceWorkerRegistrations" => {
                let registrations = self
                    .service_workers
                    .borrow()
                    .iter()
                    .map(|worker_name| {
                        let worker = registry.find::<WorkerActor>(worker_name);
                        let url = worker.url.to_string();
                        // Find correct scope url in the service worker
                        let scope = url.clone();
                        ServiceWorkerRegistrationMsg {
                            actor: worker.name(),
                            scope,
                            url: url.clone(),
                            registration_state: "".to_string(),
                            last_update_time: 0,
                            traits: HashMap::new(),
                            evaluating_worker: None,
                            installing_worker: None,
                            waiting_worker: None,
                            active_worker: Some(ServiceWorkerInfo {
                                actor: worker.name(),
                                url,
                                state: 4, // activated
                                state_text: "activated".to_string(),
                                id: worker.worker_id.to_string(),
                                fetch: false,
                                traits: HashMap::new(),
                            }),
                        }
                    })
                    .collect();
                let reply = ListServiceWorkerRegistrationsReply {
                    from: self.name(),
                    registrations,
                };
                request.reply_final(&reply)?
            },

            "listTabs" => {
                let reply = ListTabsReply {
                    from: "root".to_owned(),
                    tabs: self
                        .tabs
                        .borrow()
                        .iter()
                        .filter_map(|tab_descriptor_name| {
                            let tab_descriptor_actor =
                                registry.find::<TabDescriptorActor>(tab_descriptor_name);
                            // Filter out iframes and workers
                            if tab_descriptor_actor.is_top_level_global() {
                                Some(tab_descriptor_actor.encode(registry))
                            } else {
                                None
                            }
                        })
                        .collect(),
                };
                request.reply_final(&reply)?
            },

            "listWorkers" => {
                let reply = ListWorkersReply {
                    from: self.name(),
                    workers: self
                        .workers
                        .borrow()
                        .iter()
                        .map(|worker_name| registry.encode::<WorkerActor, _>(worker_name))
                        .collect(),
                };
                request.reply_final(&reply)?
            },

            "protocolDescription" => {
                let msg = ProtocolDescriptionReply {
                    from: self.name(),
                    types: Types {
                        performance: PerformanceActor::description(),
                        device: DeviceActor::description(),
                    },
                };
                request.reply_final(&msg)?
            },

            "watchResources" => {
                // TODO: Respond to watch resource requests
                request.reply_final(&EmptyReplyMsg { from: self.name() })?
            },

            "unwatchResources" => {
                // TODO: Respond to unwatch resource requests
                request.reply_final(&EmptyReplyMsg { from: self.name() })?
            },

            _ => return Err(ActorError::UnrecognizedPacketType),
        };
        Ok(())
    }
}

impl RootActor {
    /// Registers the root actor and its global actors (those not associated with a specific target).
    pub fn register(registry: &mut ActorRegistry) {
        // Global actors
        let device_actor = DeviceActor::new(registry.new_name::<DeviceActor>());
        let perf = PerformanceActor::new(registry.new_name::<PerformanceActor>());
        let preference_actor = PreferenceActor::new(registry.new_name::<PreferenceActor>());

        // Process descriptor
        let process = ProcessActor::new(registry.new_name::<ProcessActor>());

        // Root actor
        let root_actor = Self {
            global_actors: GlobalActors {
                device_actor: device_actor.name(),
                perf_actor: perf.name(),
                preference_actor: preference_actor.name(),
            },
            process: process.name(),
            ..Default::default()
        };

        registry.register(perf);
        registry.register(device_actor);
        registry.register(preference_actor);
        registry.register(process);
        registry.register(root_actor);
    }

    fn get_tab_msg_by_browser_id(
        &self,
        registry: &ActorRegistry,
        browser_id: u32,
    ) -> Option<TabDescriptorActorMsg> {
        let mut tab_msg = self
            .tabs
            .borrow()
            .iter()
            .map(|tab_descriptor_name| {
                registry.encode::<TabDescriptorActor, _>(tab_descriptor_name)
            })
            .find(|tab_descriptor_actor| tab_descriptor_actor.browser_id() == browser_id);

        if let Some(ref mut msg) = tab_msg {
            msg.selected = true;
            *self.active_tab.borrow_mut() = Some(msg.actor());
        }
        tab_msg
    }

    pub fn active_tab(&self) -> Option<String> {
        self.active_tab.borrow().clone()
    }
}

impl ActorEncode<RootActorMsg> for RootActor {
    fn encode(&self, _: &ActorRegistry) -> RootActorMsg {
        RootActorMsg {
            from: "root".to_owned(),
            application_type: "browser".to_owned(),
            traits: RootTraits {
                sources: false,
                highlightable: true,
                custom_highlighters: true,
                network_monitor: true,
                resources: HashMap::from([("extensions-backgroundscript-status", true)]),
            },
        }
    }
}

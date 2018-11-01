/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

/// Liberally derived from the [Firefox JS implementation]
/// (http://mxr.mozilla.org/mozilla-central/source/toolkit/devtools/server/actors/root.js).
/// Connection point for all new remote devtools interactions, providing lists of know actors
/// that perform more specific actions (targets, addons, browser chrome, etc.)
use crate::actor::{Actor, ActorMessageStatus, ActorRegistry};
use crate::actors::browsing_context::{BrowsingContextActor, BrowsingContextActorMsg};
use crate::actors::device::DeviceActor;
use crate::actors::performance::PerformanceActor;
use crate::protocol::{ActorDescription, JsonPacketStream};
use serde_json::{Map, Value};
use std::net::TcpStream;

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
}

#[derive(Serialize)]
struct ListTabsReply {
    from: String,
    selected: u32,
    tabs: Vec<BrowsingContextActorMsg>,
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
pub struct Types {
    performance: ActorDescription,
    device: ActorDescription,
}

pub struct RootActor {
    pub tabs: Vec<String>,
    pub performance: String,
    pub device: String,
}

impl Actor for RootActor {
    fn name(&self) -> String {
        "root".to_owned()
    }

    fn handle_message(
        &self,
        registry: &ActorRegistry,
        msg_type: &str,
        _msg: &Map<String, Value>,
        stream: &mut TcpStream,
    ) -> Result<ActorMessageStatus, ()> {
        Ok(match msg_type {
            "listAddons" => {
                let actor = ListAddonsReply {
                    from: "root".to_owned(),
                    addons: vec![],
                };
                stream.write_json_packet(&actor);
                ActorMessageStatus::Processed
            },

            "getRoot" => {
                let actor = GetRootReply {
                    from: "root".to_owned(),
                    selected: 0,
                    performanceActor: self.performance.clone(),
                    deviceActor: self.device.clone(),
                };
                stream.write_json_packet(&actor);
                ActorMessageStatus::Processed
            },

            // https://docs.firefox-dev.tools/backend/protocol.html#listing-browser-tabs
            "listTabs" => {
                let actor = ListTabsReply {
                    from: "root".to_owned(),
                    selected: 0,
                    tabs: self
                        .tabs
                        .iter()
                        .map(|target| registry.find::<BrowsingContextActor>(target).encodable())
                        .collect(),
                };
                stream.write_json_packet(&actor);
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
                stream.write_json_packet(&msg);
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
                sources: true,
                highlightable: true,
                customHighlighters: true,
                networkMonitor: true,
            },
        }
    }
}

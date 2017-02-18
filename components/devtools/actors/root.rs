/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

/// Liberally derived from the [Firefox JS implementation]
/// (http://mxr.mozilla.org/mozilla-central/source/toolkit/devtools/server/actors/root.js).
/// Connection point for all new remote devtools interactions, providing lists of know actors
/// that perform more specific actions (tabs, addons, browser chrome, etc.)

use actor::{Actor, ActorMessageStatus, ActorRegistry};
use actors::performance::PerformanceActor;
use actors::tab::{TabActor, TabActorMsg};
use protocol::{ActorDescription, JsonPacketStream};
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
struct ListTabsReply {
    from: String,
    selected: u32,
    tabs: Vec<TabActorMsg>,
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
}

pub struct RootActor {
    pub tabs: Vec<String>,
}

impl Actor for RootActor {
    fn name(&self) -> String {
        "root".to_owned()
    }

    fn handle_message(&self,
                      registry: &ActorRegistry,
                      msg_type: &str,
                      _msg: &Map<String, Value>,
                      stream: &mut TcpStream) -> Result<ActorMessageStatus, ()> {
        Ok(match msg_type {
            "listAddons" => {
                let actor = ListAddonsReply {
                    from: "root".to_owned(),
                    addons: vec![],
                };
                stream.write_json_packet(&actor);
                ActorMessageStatus::Processed
            }

            //https://wiki.mozilla.org/Remote_Debugging_Protocol#Listing_Browser_Tabs
            "listTabs" => {
                let actor = ListTabsReply {
                    from: "root".to_owned(),
                    selected: 0,
                    tabs: self.tabs.iter().map(|tab| {
                        registry.find::<TabActor>(tab).encodable()
                    }).collect()
                };
                stream.write_json_packet(&actor);
                ActorMessageStatus::Processed
            }

            "protocolDescription" => {
                let msg = ProtocolDescriptionReply {
                    from: self.name(),
                    types: Types {
                        performance: PerformanceActor::description(),
                    },
                };
                stream.write_json_packet(&msg);
                ActorMessageStatus::Processed
            }

            _ => ActorMessageStatus::Ignored
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
                networkMonitor: true
            },
        }
    }
}

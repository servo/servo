/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

/// Liberally derived from the [Firefox JS implementation](http://mxr.mozilla.org/mozilla-central/source/toolkit/devtools/server/actors/root.js).
/// Connection point for all new remote devtools interactions, providing lists of know actors
/// that perform more specific actions (tabs, addons, browser chrome, etc.)

use actor::{Actor, ActorRegistry};
use actors::tab::{TabActor, TabActorMsg};
use protocol::JsonPacketSender;

use serialize::json;
use std::io::TcpStream;

#[deriving(Encodable)]
struct ActorTraits {
    sources: bool,
    highlightable: bool,
    customHighlighters: Vec<String>,
}

#[deriving(Encodable)]
struct ErrorReply {
    from: String,
    error: String,
    message: String,
}

#[deriving(Encodable)]
struct ListTabsReply {
    from: String,
    selected: uint,
    tabs: Vec<TabActorMsg>,
}

#[deriving(Encodable)]
struct RootActorMsg {
    from: String,
    applicationType: String,
    traits: ActorTraits,
}

pub struct RootActor {
    pub tabs: Vec<String>,
}

impl Actor for RootActor {
    fn name(&self) -> String {
        "root".to_string()
    }

    fn handle_message(&self,
                      registry: &ActorRegistry,
                      msg_type: &String,
                      _msg: &json::JsonObject,
                      stream: &mut TcpStream) -> bool {
        match msg_type.as_slice() {
            "listAddons" => {
                let actor = ErrorReply {
                    from: "root".to_string(),
                    error: "noAddons".to_string(),
                    message: "This root actor has no browser addons.".to_string(),
                };
                stream.write_json_packet(&actor);
                true
            }

            //https://wiki.mozilla.org/Remote_Debugging_Protocol#Listing_Browser_Tabs
            "listTabs" => {
                let actor = ListTabsReply {
                    from: "root".to_string(),
                    selected: 0,
                    tabs: self.tabs.iter().map(|tab| {
                        registry.find::<TabActor>(tab.as_slice()).encodable()
                    }).collect()
                };
                stream.write_json_packet(&actor);
                true
            }

            _ => false
        }
    }
}

impl RootActor {
    pub fn encodable(&self) -> RootActorMsg {
        RootActorMsg {
            from: "root".to_string(),
            applicationType: "browser".to_string(),
            traits: ActorTraits {
                sources: true,
                highlightable: true,
                customHighlighters: vec!("BoxModelHighlighter".to_string()),
            },
        }
    }
}

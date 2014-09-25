/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

/// Liberally derived from the [Firefox JS implementation](http://mxr.mozilla.org/mozilla-central/source/toolkit/devtools/server/actors/webbrowser.js).
/// Connection point for remote devtools that wish to investigate a particular tab's contents.
/// Supports dynamic attaching and detaching which control notifications of navigation, etc.

use actor::{Actor, ActorRegistry};
use protocol::JsonPacketSender;

use serialize::json;
use std::io::TcpStream;

#[deriving(Encodable)]
struct TabTraits;

#[deriving(Encodable)]
struct TabAttachedReply {
    from: String,
    __type__: String,
    threadActor: String,
    cacheDisabled: bool,
    javascriptEnabled: bool,
    traits: TabTraits,
}

#[deriving(Encodable)]
struct TabDetachedReply {
    from: String,
    __type__: String,
}

#[deriving(Encodable)]
struct ReconfigureReply {
    from: String
}

#[deriving(Encodable)]
struct ListFramesReply {
    from: String,
    frames: Vec<FrameMsg>,
}

#[deriving(Encodable)]
struct FrameMsg {
    id: uint,
    url: String,
    title: String,
    parentID: uint,
}

#[deriving(Encodable)]
pub struct TabActorMsg {
    actor: String,
    title: String,
    url: String,
    outerWindowID: uint,
    consoleActor: String,
    inspectorActor: String,
}

pub struct TabActor {
    pub name: String,
    pub title: String,
    pub url: String,
    pub console: String,
    pub inspector: String,
}

impl Actor for TabActor {
    fn name(&self) -> String {
        self.name.clone()
    }

    fn handle_message(&self,
                      _registry: &ActorRegistry,
                      msg_type: &String,
                      _msg: &json::JsonObject,
                      stream: &mut TcpStream) -> bool {
        match msg_type.as_slice() {
            "reconfigure" => {
                stream.write_json_packet(&ReconfigureReply { from: self.name() });
                true
            }

            // https://wiki.mozilla.org/Remote_Debugging_Protocol#Listing_Browser_Tabs
            // (see "To attach to a _tabActor_")
            "attach" => {
                let msg = TabAttachedReply {
                    from: self.name(),
                    __type__: "tabAttached".to_string(),
                    threadActor: self.name(),
                    cacheDisabled: false,
                    javascriptEnabled: true,
                    traits: TabTraits,
                };
                stream.write_json_packet(&msg);
                true
            }

            "detach" => {
                let msg = TabDetachedReply {
                    from: self.name(),
                    __type__: "detached".to_string(),
                };
                stream.write_json_packet(&msg);
                true
            }

            "listFrames" => {
                let msg = ListFramesReply {
                    from: self.name(),
                    frames: vec!(),
                };
                stream.write_json_packet(&msg);
                true
            }

            _ => false
        }
    }
}

impl TabActor {
    pub fn encodable(&self) -> TabActorMsg {
        TabActorMsg {
            actor: self.name(),
            title: self.title.clone(),
            url: self.url.clone(),
            outerWindowID: 0, //FIXME: this should probably be the pipeline id
            consoleActor: self.console.clone(),
            inspectorActor: self.inspector.clone(),
        }
    }
}

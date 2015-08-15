/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Liberally derived from the [Firefox JS implementation]
//! (http://mxr.mozilla.org/mozilla-central/source/toolkit/devtools/server/actors/webbrowser.js).
//! Connection point for remote devtools that wish to investigate a particular tab's contents.
//! Supports dynamic attaching and detaching which control notifications of navigation, etc.

use actor::{Actor, ActorRegistry, ActorMessageStatus};
use actors::console::ConsoleActor;
use devtools_traits::DevtoolScriptControlMsg::WantsLiveNotifications;
use protocol::JsonPacketStream;

use rustc_serialize::json;
use std::net::TcpStream;

#[derive(RustcEncodable)]
struct TabTraits;

#[derive(RustcEncodable)]
struct TabAttachedReply {
    from: String,
    __type__: String,
    threadActor: String,
    cacheDisabled: bool,
    javascriptEnabled: bool,
    traits: TabTraits,
}

#[derive(RustcEncodable)]
struct TabDetachedReply {
    from: String,
    __type__: String,
}

#[derive(RustcEncodable)]
struct ReconfigureReply {
    from: String
}

#[derive(RustcEncodable)]
struct ListFramesReply {
    from: String,
    frames: Vec<FrameMsg>,
}

#[derive(RustcEncodable)]
struct FrameMsg {
    id: u32,
    url: String,
    title: String,
    parentID: u32,
}

#[derive(RustcEncodable)]
pub struct TabActorMsg {
    actor: String,
    title: String,
    url: String,
    outerWindowID: u32,
    consoleActor: String,
    inspectorActor: String,
    timelineActor: String,
    profilerActor: String,
    performanceActor: String,
}

pub struct TabActor {
    pub name: String,
    pub title: String,
    pub url: String,
    pub console: String,
    pub inspector: String,
    pub timeline: String,
    pub profiler: String,
    pub performance: String,
}

impl Actor for TabActor {
    fn name(&self) -> String {
        self.name.clone()
    }

    fn handle_message(&self,
                      registry: &ActorRegistry,
                      msg_type: &str,
                      _msg: &json::Object,
                      stream: &mut TcpStream) -> Result<ActorMessageStatus, ()> {
        Ok(match msg_type {
            "reconfigure" => {
                stream.write_json_packet(&ReconfigureReply { from: self.name() });
                ActorMessageStatus::Processed
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
                let console_actor = registry.find::<ConsoleActor>(&self.console);
                console_actor.streams.borrow_mut().push(stream.try_clone().unwrap());
                stream.write_json_packet(&msg);
                console_actor.script_chan.send(
                    WantsLiveNotifications(console_actor.pipeline, true)).unwrap();
                ActorMessageStatus::Processed
            }

            //FIXME: The current implementation won't work for multiple connections. Need to ensure 105
            //       that the correct stream is removed.
            "detach" => {
                let msg = TabDetachedReply {
                    from: self.name(),
                    __type__: "detached".to_string(),
                };
                let console_actor = registry.find::<ConsoleActor>(&self.console);
                console_actor.streams.borrow_mut().pop();
                stream.write_json_packet(&msg);
                console_actor.script_chan.send(
                    WantsLiveNotifications(console_actor.pipeline, false)).unwrap();
                ActorMessageStatus::Processed
            }

            "listFrames" => {
                let msg = ListFramesReply {
                    from: self.name(),
                    frames: vec!(),
                };
                stream.write_json_packet(&msg);
                ActorMessageStatus::Processed
            }

            _ => ActorMessageStatus::Ignored
        })
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
            timelineActor: self.timeline.clone(),
            profilerActor: self.profiler.clone(),
            performanceActor: self.performance.clone(),
        }
    }
}

/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Liberally derived from the [Firefox JS implementation]
//! (http://mxr.mozilla.org/mozilla-central/source/toolkit/devtools/server/actors/webbrowser.js).
//! Connection point for remote devtools that wish to investigate a particular Browsing Context's contents.
//! Supports dynamic attaching and detaching which control notifications of navigation, etc.

use crate::actor::{Actor, ActorMessageStatus, ActorRegistry};
use crate::actors::console::ConsoleActor;
use crate::protocol::JsonPacketStream;
use devtools_traits::DevtoolScriptControlMsg::{self, WantsLiveNotifications};
use serde_json::{Map, Value};
use std::net::TcpStream;

#[derive(Serialize)]
struct BrowsingContextTraits;

#[derive(Serialize)]
struct BrowsingContextAttachedReply {
    from: String,
    #[serde(rename = "type")]
    type_: String,
    threadActor: String,
    cacheDisabled: bool,
    javascriptEnabled: bool,
    traits: BrowsingContextTraits,
}

#[derive(Serialize)]
struct BrowsingContextDetachedReply {
    from: String,
    #[serde(rename = "type")]
    type_: String,
}

#[derive(Serialize)]
struct ReconfigureReply {
    from: String,
}

#[derive(Serialize)]
struct ListFramesReply {
    from: String,
    frames: Vec<FrameMsg>,
}

#[derive(Serialize)]
struct FrameMsg {
    id: u32,
    url: String,
    title: String,
    parentID: u32,
}

#[derive(Serialize)]
struct ListWorkersReply {
    from: String,
    workers: Vec<WorkerMsg>,
}

#[derive(Serialize)]
struct WorkerMsg {
    id: u32,
}

#[derive(Serialize)]
pub struct BrowsingContextActorMsg {
    actor: String,
    title: String,
    url: String,
    outerWindowID: u32,
    consoleActor: String,
    emulationActor: String,
    inspectorActor: String,
    timelineActor: String,
    profilerActor: String,
    performanceActor: String,
    styleSheetsActor: String,
}

pub struct BrowsingContextActor {
    pub name: String,
    pub title: String,
    pub url: String,
    pub console: String,
    pub emulation: String,
    pub inspector: String,
    pub timeline: String,
    pub profiler: String,
    pub performance: String,
    pub styleSheets: String,
    pub thread: String,
}

impl Actor for BrowsingContextActor {
    fn name(&self) -> String {
        self.name.clone()
    }

    fn handle_message(
        &self,
        registry: &ActorRegistry,
        msg_type: &str,
        msg: &Map<String, Value>,
        stream: &mut TcpStream,
    ) -> Result<ActorMessageStatus, ()> {
        Ok(match msg_type {
            "reconfigure" => {
                if let Some(options) = msg.get("options").and_then(|o| o.as_object()) {
                    if let Some(val) = options.get("performReload") {
                        if val.as_bool().unwrap_or(false) {
                            let console_actor = registry.find::<ConsoleActor>(&self.console);
                            let _ = console_actor
                                .script_chan
                                .send(DevtoolScriptControlMsg::Reload(console_actor.pipeline));
                        }
                    }
                }
                stream.write_json_packet(&ReconfigureReply { from: self.name() });
                ActorMessageStatus::Processed
            },

            // https://docs.firefox-dev.tools/backend/protocol.html#listing-browser-tabs
            // (see "To attach to a _targetActor_")
            "attach" => {
                let msg = BrowsingContextAttachedReply {
                    from: self.name(),
                    type_: "targetAttached".to_owned(),
                    threadActor: self.thread.clone(),
                    cacheDisabled: false,
                    javascriptEnabled: true,
                    traits: BrowsingContextTraits,
                };
                let console_actor = registry.find::<ConsoleActor>(&self.console);
                console_actor
                    .streams
                    .borrow_mut()
                    .push(stream.try_clone().unwrap());
                stream.write_json_packet(&msg);
                console_actor
                    .script_chan
                    .send(WantsLiveNotifications(console_actor.pipeline, true))
                    .unwrap();
                ActorMessageStatus::Processed
            },

            //FIXME: The current implementation won't work for multiple connections. Need to ensure 105
            //       that the correct stream is removed.
            "detach" => {
                let msg = BrowsingContextDetachedReply {
                    from: self.name(),
                    type_: "detached".to_owned(),
                };
                let console_actor = registry.find::<ConsoleActor>(&self.console);
                console_actor.streams.borrow_mut().pop();
                stream.write_json_packet(&msg);
                console_actor
                    .script_chan
                    .send(WantsLiveNotifications(console_actor.pipeline, false))
                    .unwrap();
                ActorMessageStatus::Processed
            },

            "listFrames" => {
                let msg = ListFramesReply {
                    from: self.name(),
                    frames: vec![],
                };
                stream.write_json_packet(&msg);
                ActorMessageStatus::Processed
            },

            "listWorkers" => {
                let msg = ListWorkersReply {
                    from: self.name(),
                    workers: vec![],
                };
                stream.write_json_packet(&msg);
                ActorMessageStatus::Processed
            },

            _ => ActorMessageStatus::Ignored,
        })
    }
}

impl BrowsingContextActor {
    pub fn encodable(&self) -> BrowsingContextActorMsg {
        BrowsingContextActorMsg {
            actor: self.name(),
            title: self.title.clone(),
            url: self.url.clone(),
            outerWindowID: 0, //FIXME: this should probably be the pipeline id
            consoleActor: self.console.clone(),
            emulationActor: self.emulation.clone(),
            inspectorActor: self.inspector.clone(),
            timelineActor: self.timeline.clone(),
            profilerActor: self.profiler.clone(),
            performanceActor: self.performance.clone(),
            styleSheetsActor: self.styleSheets.clone(),
        }
    }
}

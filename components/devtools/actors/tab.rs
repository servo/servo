/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::net::TcpStream;

use serde::Serialize;
use serde_json::{Map, Value};

use crate::actor::{Actor, ActorMessageStatus, ActorRegistry};
use crate::actors::browsing_context::{BrowsingContextActor, BrowsingContextActorMsg};
use crate::actors::root::{DescriptorTraits, RootActor};
use crate::protocol::JsonPacketStream;
use crate::StreamId;

// https://searchfox.org/mozilla-central/source/devtools/server/actors/descriptors/tab.js
#[derive(Serialize)]
pub struct TabDescriptorActorMsg {
    actor: String,
    browserId: u32,
    browsingContextId: u32,
    isZombieTab: bool,
    outerWindowID: u32,
    selected: bool,
    title: String,
    traits: DescriptorTraits,
    url: String,
}

impl TabDescriptorActorMsg {
    pub fn id(&self) -> u32 {
        self.browserId
    }
}

#[derive(Serialize)]
struct GetTargetReply {
    from: String,
    frame: BrowsingContextActorMsg,
}

#[derive(Serialize)]
struct GetFaviconReply {
    from: String,
    favicon: String,
}

pub struct TabDescriptorActor {
    name: String,
    browsing_context_actor: String,
}

impl Actor for TabDescriptorActor {
    fn name(&self) -> String {
        self.name.clone()
    }

    fn handle_message(
        &self,
        registry: &ActorRegistry,
        msg_type: &str,
        _msg: &Map<String, Value>,
        stream: &mut TcpStream,
        _id: StreamId,
    ) -> Result<ActorMessageStatus, ()> {
        Ok(match msg_type {
            "getTarget" => {
                let frame = registry
                    .find::<BrowsingContextActor>(&self.browsing_context_actor)
                    .encodable();
                let _ = stream.write_json_packet(&GetTargetReply {
                    from: self.name(),
                    frame,
                });
                ActorMessageStatus::Processed
            },
            "getFavicon" => {
                // TODO: Return a favicon when available
                let _ = stream.write_json_packet(&GetFaviconReply {
                    from: self.name(),
                    favicon: String::new(),
                });
                ActorMessageStatus::Processed
            },
            // TODO: Unexpected message getWatcher when inspecting tab (create watcher actor)
            _ => ActorMessageStatus::Ignored,
        })
    }
}

impl TabDescriptorActor {
    pub(crate) fn new(
        actors: &mut ActorRegistry,
        browsing_context_actor: String,
    ) -> TabDescriptorActor {
        let name = actors.new_name("tabDescription");
        let root = actors.find_mut::<RootActor>("root");
        root.tabs.push(name.clone());
        TabDescriptorActor {
            name,
            browsing_context_actor,
        }
    }

    pub fn encodable(&self, registry: &ActorRegistry, selected: bool) -> TabDescriptorActorMsg {
        let ctx_actor = registry.find::<BrowsingContextActor>(&self.browsing_context_actor);

        let title = ctx_actor.title.borrow().clone();
        let url = ctx_actor.url.borrow().clone();

        TabDescriptorActorMsg {
            actor: self.name(),
            browsingContextId: ctx_actor.browsing_context_id.index.0.get(),
            browserId: ctx_actor.active_pipeline.get().index.0.get(),
            isZombieTab: false,
            outerWindowID: ctx_actor.active_pipeline.get().index.0.get(),
            selected,
            title,
            traits: DescriptorTraits {
                watcher: true,
                supportsReloadDescriptor: false,
            },
            url,
        }
    }
}

/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::actor::{Actor, ActorMessageStatus, ActorRegistry};
use crate::actors::browsing_context::{BrowsingContextActor, BrowsingContextActorMsg};
use crate::actors::root::RootActor;
use crate::protocol::JsonPacketStream;
use crate::StreamId;
use serde_json::{Map, Value};
use std::net::TcpStream;

#[derive(Serialize)]
pub struct TabDescriptorTraits {
    getFavicon: bool,
    hasTabInfo: bool,
    watcher: bool,
}

#[derive(Serialize)]
pub struct TabDescriptorActorMsg {
    actor: String,
    title: String,
    url: String,
    outerWindowID: u32,
    browsingContextId: u32,
    traits: TabDescriptorTraits,
}

#[derive(Serialize)]
struct GetTargetReply {
    from: String,
    frame: BrowsingContextActorMsg,
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
            name: name,
            browsing_context_actor,
        }
    }

    pub fn encodable(&self, registry: &ActorRegistry) -> TabDescriptorActorMsg {
        let ctx_actor = registry.find::<BrowsingContextActor>(&self.browsing_context_actor);

        let title = ctx_actor.title.borrow().clone();
        let url = ctx_actor.url.borrow().clone();

        TabDescriptorActorMsg {
            title,
            url,
            actor: self.name(),
            browsingContextId: ctx_actor.browsing_context_id.index.0.get(),
            outerWindowID: ctx_actor.active_pipeline.get().index.0.get(),
            traits: TabDescriptorTraits {
                getFavicon: false,
                hasTabInfo: true,
                watcher: false,
            },
        }
    }
}

/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Descriptor actor that represents a web view. It can link a tab to the corresponding watcher
//! actor to enable inspection.
//!
//! Liberally derived from the [Firefox JS implementation].
//!
//! [Firefox JS implementation]: https://searchfox.org/mozilla-central/source/devtools/server/actors/descriptors/tab.js

use std::net::TcpStream;

use serde::Serialize;
use serde_json::{Map, Value};

use crate::StreamId;
use crate::actor::{Actor, ActorMessageStatus, ActorRegistry};
use crate::actors::browsing_context::{BrowsingContextActor, BrowsingContextActorMsg};
use crate::actors::root::{DescriptorTraits, RootActor};
use crate::actors::watcher::{WatcherActor, WatcherActorMsg};
use crate::protocol::JsonPacketStream;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TabDescriptorActorMsg {
    actor: String,
    #[serde(rename = "browserId")]
    webview_id: u32,
    #[serde(rename = "browsingContextID")]
    browsing_context_id: u32,
    is_zombie_tab: bool,
    #[serde(rename = "outerWindowID")]
    outer_window_id: u32,
    selected: bool,
    title: String,
    traits: DescriptorTraits,
    url: String,
}

impl TabDescriptorActorMsg {
    pub fn webview_id(&self) -> u32 {
        self.webview_id
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

#[derive(Serialize)]
struct GetWatcherReply {
    from: String,
    #[serde(flatten)]
    watcher: WatcherActorMsg,
}

pub struct TabDescriptorActor {
    name: String,
    browsing_context_actor: String,
    is_top_level_global: bool,
}

impl Actor for TabDescriptorActor {
    fn name(&self) -> String {
        self.name.clone()
    }

    /// The tab actor can handle the following messages:
    ///
    /// - `getTarget`: Returns the surrounding `BrowsingContextActor`.
    ///
    /// - `getFavicon`: Should return the tab favicon, but it is not yet supported.
    ///
    /// - `getWatcher`: Returns a `WatcherActor` linked to the tab's `BrowsingContext`. It is used
    ///   to describe the debugging capabilities of this tab.
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
            "getWatcher" => {
                let ctx_actor = registry.find::<BrowsingContextActor>(&self.browsing_context_actor);
                let watcher = registry.find::<WatcherActor>(&ctx_actor.watcher);
                let _ = stream.write_json_packet(&GetWatcherReply {
                    from: self.name(),
                    watcher: watcher.encodable(),
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
        is_top_level_global: bool,
    ) -> TabDescriptorActor {
        let name = actors.new_name("tab-description");
        let root = actors.find_mut::<RootActor>("root");
        root.tabs.push(name.clone());
        TabDescriptorActor {
            name,
            browsing_context_actor,
            is_top_level_global,
        }
    }

    pub fn encodable(&self, registry: &ActorRegistry, selected: bool) -> TabDescriptorActorMsg {
        let ctx_actor = registry.find::<BrowsingContextActor>(&self.browsing_context_actor);
        let webview_id = ctx_actor.webview_id.0.index.0.get();
        let browsing_context_id = ctx_actor.browsing_context_id.index.0.get();
        let outer_window_id = ctx_actor.active_pipeline.get().index.0.get();
        let title = ctx_actor.title.borrow().clone();
        let url = ctx_actor.url.borrow().clone();

        TabDescriptorActorMsg {
            actor: self.name(),
            webview_id,
            browsing_context_id,
            is_zombie_tab: false,
            outer_window_id,
            selected,
            title,
            traits: DescriptorTraits {
                watcher: true,
                supports_reload_descriptor: true,
            },
            url,
        }
    }

    pub(crate) fn is_top_level_global(&self) -> bool {
        self.is_top_level_global
    }
}

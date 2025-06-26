/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Descriptor actor that represents a web view. It can link a tab to the corresponding watcher
//! actor to enable inspection.
//!
//! Liberally derived from the [Firefox JS implementation].
//!
//! [Firefox JS implementation]: https://searchfox.org/mozilla-central/source/devtools/server/actors/descriptors/tab.js

use serde::Serialize;
use serde_json::{Map, Value};

use crate::StreamId;
use crate::actor::{Actor, ActorError, ActorRegistry};
use crate::actors::browsing_context::{BrowsingContextActor, BrowsingContextActorMsg};
use crate::actors::root::{DescriptorTraits, RootActor};
use crate::actors::watcher::{WatcherActor, WatcherActorMsg};
use crate::protocol::ClientRequest;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TabDescriptorActorMsg {
    actor: String,
    /// This correspond to webview_id
    #[serde(rename = "browserId")]
    browser_id: u32,
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
    pub fn browser_id(&self) -> u32 {
        self.browser_id
    }

    pub fn actor(&self) -> String {
        self.actor.clone()
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
        request: ClientRequest,
        registry: &ActorRegistry,
        msg_type: &str,
        _msg: &Map<String, Value>,
        _id: StreamId,
    ) -> Result<(), ActorError> {
        match msg_type {
            "getTarget" => {
                let frame = registry
                    .find::<BrowsingContextActor>(&self.browsing_context_actor)
                    .encodable();
                request.reply_final(&GetTargetReply {
                    from: self.name(),
                    frame,
                })?
            },
            "getFavicon" => {
                // TODO: Return a favicon when available
                request.reply_final(&GetFaviconReply {
                    from: self.name(),
                    favicon: String::new(),
                })?
            },
            "getWatcher" => {
                let ctx_actor = registry.find::<BrowsingContextActor>(&self.browsing_context_actor);
                let watcher = registry.find::<WatcherActor>(&ctx_actor.watcher);
                request.reply_final(&GetWatcherReply {
                    from: self.name(),
                    watcher: watcher.encodable(),
                })?
            },
            _ => return Err(ActorError::UnrecognizedPacketType),
        };
        Ok(())
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
        let title = ctx_actor.title.borrow().clone();
        let url = ctx_actor.url.borrow().clone();

        TabDescriptorActorMsg {
            actor: self.name(),
            browser_id: ctx_actor.browser_id.value(),
            browsing_context_id: ctx_actor.browsing_context_id.value(),
            is_zombie_tab: false,
            outer_window_id: ctx_actor.active_outer_window_id.get().value(),
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

    #[allow(dead_code)]
    pub fn browsing_context(&self) -> String {
        self.browsing_context_actor.clone()
    }
}

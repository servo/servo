/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Liberally derived from the [Firefox JS implementation](http://mxr.mozilla.org/mozilla-central/source/toolkit/devtools/server/actors/inspector.js).

use malloc_size_of_derive::MallocSizeOf;
use serde::Serialize;
use serde_json::{self, Map, Value};

use crate::actor::{Actor, ActorError, ActorRegistry};
use crate::actors::inspector::highlighter::HighlighterActor;
use crate::actors::inspector::page_style::{PageStyleActor, PageStyleMsg};
use crate::actors::inspector::walker::{WalkerActor, WalkerMsg};
use crate::protocol::ClientRequest;
use crate::{ActorMsg, StreamId};

pub mod accessibility;
pub mod css_properties;
pub mod highlighter;
pub mod layout;
pub mod node;
pub mod page_style;
pub mod style_rule;
pub mod walker;

#[derive(Serialize)]
struct GetHighlighterReply {
    from: String,
    highlighter: ActorMsg,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct GetPageStyleReply {
    from: String,
    page_style: PageStyleMsg,
}

#[derive(Serialize)]
struct GetWalkerReply {
    from: String,
    walker: WalkerMsg,
}

#[derive(Serialize)]
struct SupportsHighlightersReply {
    from: String,
    value: bool,
}

#[derive(MallocSizeOf)]
pub(crate) struct InspectorActor {
    name: String,
    highlighter_name: String,
    page_style_name: String,
    pub(crate) walker_name: String,
}

impl Actor for InspectorActor {
    fn name(&self) -> String {
        self.name.clone()
    }

    fn handle_message(
        &self,
        request: ClientRequest,
        registry: &ActorRegistry,
        msg_type: &str,
        _msg: &Map<String, Value>,
        _id: StreamId,
    ) -> Result<(), ActorError> {
        match msg_type {
            "getPageStyle" => {
                let msg = GetPageStyleReply {
                    from: self.name(),
                    page_style: registry.encode::<PageStyleActor, _>(&self.page_style_name),
                };
                request.reply_final(&msg)?
            },

            "getHighlighterByType" => {
                let msg = GetHighlighterReply {
                    from: self.name(),
                    highlighter: registry.encode::<HighlighterActor, _>(&self.highlighter_name),
                };
                request.reply_final(&msg)?
            },

            "getWalker" => {
                let msg = GetWalkerReply {
                    from: self.name(),
                    walker: registry.encode::<WalkerActor, _>(&self.walker_name),
                };
                request.reply_final(&msg)?
            },

            "supportsHighlighters" => {
                let msg = SupportsHighlightersReply {
                    from: self.name(),
                    value: true,
                };
                request.reply_final(&msg)?
            },

            _ => return Err(ActorError::UnrecognizedPacketType),
        };
        Ok(())
    }
}

impl InspectorActor {
    pub fn register(registry: &ActorRegistry, browsing_context_name: String) -> String {
        let highlighter_name = HighlighterActor::register(registry, browsing_context_name.clone());

        let page_style_name = PageStyleActor::register(registry);

        let walker_name = WalkerActor::register(registry, browsing_context_name);

        let inspector_actor = Self {
            name: registry.new_name::<InspectorActor>(),
            highlighter_name,
            page_style_name,
            walker_name,
        };
        let inspector_name = inspector_actor.name();

        registry.register(inspector_actor);

        inspector_name
    }
}

/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Liberally derived from the [Firefox JS implementation](http://mxr.mozilla.org/mozilla-central/source/toolkit/devtools/server/actors/inspector.js).

use std::cell::RefCell;

use base::generic_channel::GenericSender;
use base::id::PipelineId;
use devtools_traits::DevtoolScriptControlMsg;
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

pub struct InspectorActor {
    pub name: String,
    pub highlighter: String,
    pub page_style: String,
    pub walker: String,
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
                    page_style: registry.encode::<PageStyleActor, _>(&self.page_style),
                };
                request.reply_final(&msg)?
            },

            "getHighlighterByType" => {
                let msg = GetHighlighterReply {
                    from: self.name(),
                    highlighter: registry.encode::<HighlighterActor, _>(&self.highlighter),
                };
                request.reply_final(&msg)?
            },

            "getWalker" => {
                let msg = GetWalkerReply {
                    from: self.name(),
                    walker: registry.encode::<WalkerActor, _>(&self.walker),
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
    // TODO: Passing the pipeline id here isn't correct. We should query the browsing
    // context for the active pipeline, otherwise reloading or navigating will break the inspector.
    pub fn register(
        registry: &mut ActorRegistry,
        pipeline: PipelineId,
        script_chan: GenericSender<DevtoolScriptControlMsg>,
    ) -> String {
        let highlighter = HighlighterActor {
            name: registry.new_name::<HighlighterActor>(),
            script_sender: script_chan.clone(),
            pipeline,
        };

        let page_style = PageStyleActor {
            name: registry.new_name::<PageStyleActor>(),
            script_chan: script_chan.clone(),
            pipeline,
        };

        let walker = WalkerActor {
            name: registry.new_name::<WalkerActor>(),
            mutations: RefCell::new(vec![]),
            script_chan,
            pipeline,
        };

        let actor = Self {
            name: registry.new_name::<InspectorActor>(),
            highlighter: highlighter.name(),
            page_style: page_style.name(),
            walker: walker.name(),
        };
        let name = actor.name();

        registry.register(highlighter);
        registry.register(page_style);
        registry.register(walker);
        registry.register(actor);

        name
    }
}

/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Liberally derived from the [Firefox JS implementation](http://mxr.mozilla.org/mozilla-central/source/toolkit/devtools/server/actors/inspector.js).

use std::cell::RefCell;
use std::collections::HashMap;

use devtools_traits::DevtoolScriptControlMsg;
use devtools_traits::DevtoolScriptControlMsg::GetRootNode;
use ipc_channel::ipc::{self, IpcSender};
use serde::Serialize;
use serde_json::{self, Map, Value};

use crate::StreamId;
use crate::actor::{Actor, ActorError, ActorRegistry};
use crate::actors::browsing_context::BrowsingContextActor;
use crate::actors::inspector::highlighter::{HighlighterActor, HighlighterMsg};
use crate::actors::inspector::node::NodeInfoToProtocol;
use crate::actors::inspector::page_style::{PageStyleActor, PageStyleMsg};
use crate::actors::inspector::walker::{WalkerActor, WalkerMsg};
use crate::protocol::ClientRequest;

pub mod accessibility;
pub mod css_properties;
pub mod highlighter;
pub mod layout;
pub mod node;
pub mod page_style;
pub mod style_rule;
pub mod walker;

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

#[derive(Serialize)]
struct GetHighlighterReply {
    from: String,
    highlighter: HighlighterMsg,
}

pub struct InspectorActor {
    pub name: String,
    pub walker: RefCell<Option<String>>,
    pub page_style: RefCell<Option<String>>,
    pub highlighter: RefCell<Option<String>>,
    pub script_chan: IpcSender<DevtoolScriptControlMsg>,
    pub browsing_context: String,
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
        let browsing_context = registry.find::<BrowsingContextActor>(&self.browsing_context);
        let pipeline = browsing_context.active_pipeline_id.get();
        match msg_type {
            "getWalker" => {
                let (tx, rx) = ipc::channel().unwrap();
                self.script_chan.send(GetRootNode(pipeline, tx)).unwrap();
                let root_info = rx.recv().unwrap().ok_or(ActorError::Internal)?;

                let name = self
                    .walker
                    .borrow()
                    .clone()
                    .unwrap_or_else(|| registry.new_name("walker"));

                let root = root_info.encode(
                    registry,
                    false,
                    self.script_chan.clone(),
                    pipeline,
                    name.clone(),
                );

                if self.walker.borrow().is_none() {
                    let walker = WalkerActor {
                        name,
                        script_chan: self.script_chan.clone(),
                        pipeline,
                        root_node: root.clone(),
                        mutations: RefCell::new(vec![]),
                    };
                    let mut walker_name = self.walker.borrow_mut();
                    *walker_name = Some(walker.name());
                    registry.register_later(Box::new(walker));
                }

                let msg = GetWalkerReply {
                    from: self.name(),
                    walker: WalkerMsg {
                        actor: self.walker.borrow().clone().unwrap(),
                        root,
                    },
                };
                request.reply_final(&msg)?
            },

            "getPageStyle" => {
                if self.page_style.borrow().is_none() {
                    let style = PageStyleActor {
                        name: registry.new_name("page-style"),
                        script_chan: self.script_chan.clone(),
                        pipeline,
                    };
                    let mut page_style = self.page_style.borrow_mut();
                    *page_style = Some(style.name());
                    registry.register_later(Box::new(style));
                }

                let msg = GetPageStyleReply {
                    from: self.name(),
                    page_style: PageStyleMsg {
                        actor: self.page_style.borrow().clone().unwrap(),
                        traits: HashMap::from([
                            ("fontStretchLevel4".into(), true),
                            ("fontStyleLevel4".into(), true),
                            ("fontVariations".into(), true),
                            ("fontWeightLevel4".into(), true),
                        ]),
                    },
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

            "getHighlighterByType" => {
                if self.highlighter.borrow().is_none() {
                    let highlighter_actor = HighlighterActor {
                        name: registry.new_name("highlighter"),
                        pipeline,
                        script_sender: self.script_chan.clone(),
                    };
                    let mut highlighter = self.highlighter.borrow_mut();
                    *highlighter = Some(highlighter_actor.name());
                    registry.register_later(Box::new(highlighter_actor));
                }

                let msg = GetHighlighterReply {
                    from: self.name(),
                    highlighter: HighlighterMsg {
                        actor: self.highlighter.borrow().clone().unwrap(),
                    },
                };
                request.reply_final(&msg)?
            },
            _ => return Err(ActorError::UnrecognizedPacketType),
        };
        Ok(())
    }
}

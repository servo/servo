/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Liberally derived from the [Firefox JS implementation](http://mxr.mozilla.org/mozilla-central/source/toolkit/devtools/server/actors/inspector.js).

#![allow(non_snake_case)] // NOTE: To be removed on the inspector specific pr

use std::cell::RefCell;
use std::net::TcpStream;

use base::id::PipelineId;
use devtools_traits::DevtoolScriptControlMsg::{
    GetChildren, GetDocumentElement, GetLayout, GetRootNode, ModifyAttribute,
};
use devtools_traits::{ComputedNodeLayout, DevtoolScriptControlMsg, NodeInfo};
use ipc_channel::ipc::{self, IpcSender};
use serde::Serialize;
use serde_json::{self, Map, Value};

use crate::actor::{Actor, ActorMessageStatus, ActorRegistry};
use crate::actors::browsing_context::BrowsingContextActor;
use crate::protocol::JsonPacketStream;
use crate::StreamId;

pub struct InspectorActor {
    pub name: String,
    pub walker: RefCell<Option<String>>,
    pub page_style: RefCell<Option<String>>,
    pub highlighter: RefCell<Option<String>>,
    pub script_chan: IpcSender<DevtoolScriptControlMsg>,
    pub browsing_context: String,
}

#[derive(Serialize)]
struct GetHighlighterReply {
    highligter: HighlighterMsg, // sic.
    from: String,
}

#[derive(Serialize)]
struct HighlighterMsg {
    actor: String,
}

struct HighlighterActor {
    name: String,
}

pub struct NodeActor {
    pub name: String,
    script_chan: IpcSender<DevtoolScriptControlMsg>,
    pipeline: PipelineId,
}

#[derive(Serialize)]
struct ShowBoxModelReply {
    from: String,
}

#[derive(Serialize)]
struct HideBoxModelReply {
    from: String,
}

impl Actor for HighlighterActor {
    fn name(&self) -> String {
        self.name.clone()
    }

    fn handle_message(
        &self,
        _registry: &ActorRegistry,
        msg_type: &str,
        _msg: &Map<String, Value>,
        stream: &mut TcpStream,
        _id: StreamId,
    ) -> Result<ActorMessageStatus, ()> {
        Ok(match msg_type {
            "showBoxModel" => {
                let msg = ShowBoxModelReply { from: self.name() };
                let _ = stream.write_json_packet(&msg);
                ActorMessageStatus::Processed
            },

            "hideBoxModel" => {
                let msg = HideBoxModelReply { from: self.name() };
                let _ = stream.write_json_packet(&msg);
                ActorMessageStatus::Processed
            },

            _ => ActorMessageStatus::Ignored,
        })
    }
}

#[derive(Serialize)]
struct ModifyAttributeReply {
    from: String,
}

impl Actor for NodeActor {
    fn name(&self) -> String {
        self.name.clone()
    }

    fn handle_message(
        &self,
        registry: &ActorRegistry,
        msg_type: &str,
        msg: &Map<String, Value>,
        stream: &mut TcpStream,
        _id: StreamId,
    ) -> Result<ActorMessageStatus, ()> {
        Ok(match msg_type {
            "modifyAttributes" => {
                let target = msg.get("to").unwrap().as_str().unwrap();
                let mods = msg.get("modifications").unwrap().as_array().unwrap();
                let modifications = mods
                    .iter()
                    .map(|json_mod| {
                        serde_json::from_str(&serde_json::to_string(json_mod).unwrap()).unwrap()
                    })
                    .collect();

                self.script_chan
                    .send(ModifyAttribute(
                        self.pipeline,
                        registry.actor_to_script(target.to_owned()),
                        modifications,
                    ))
                    .unwrap();
                let reply = ModifyAttributeReply { from: self.name() };
                let _ = stream.write_json_packet(&reply);
                ActorMessageStatus::Processed
            },

            _ => ActorMessageStatus::Ignored,
        })
    }
}

#[derive(Serialize)]
struct GetWalkerReply {
    from: String,
    walker: WalkerMsg,
}

#[derive(Serialize)]
struct WalkerMsg {
    actor: String,
    root: NodeActorMsg,
}

#[derive(Serialize)]
struct AttrMsg {
    namespace: String,
    name: String,
    value: String,
}

#[derive(Serialize)]
struct NodeActorMsg {
    actor: String,
    baseURI: String,
    parent: String,
    nodeType: u16,
    namespaceURI: String,
    nodeName: String,
    numChildren: usize,

    name: String,
    publicId: String,
    systemId: String,

    attrs: Vec<AttrMsg>,

    pseudoClassLocks: Vec<String>,

    isDisplayed: bool,

    hasEventListeners: bool,

    isDocumentElement: bool,

    shortValue: String,
    incompleteValue: bool,
}

trait NodeInfoToProtocol {
    fn encode(
        self,
        actors: &ActorRegistry,
        display: bool,
        script_chan: IpcSender<DevtoolScriptControlMsg>,
        pipeline: PipelineId,
    ) -> NodeActorMsg;
}

impl NodeInfoToProtocol for NodeInfo {
    fn encode(
        self,
        actors: &ActorRegistry,
        display: bool,
        script_chan: IpcSender<DevtoolScriptControlMsg>,
        pipeline: PipelineId,
    ) -> NodeActorMsg {
        let actor_name = if !actors.script_actor_registered(self.unique_id.clone()) {
            let name = actors.new_name("node");
            let node_actor = NodeActor {
                name: name.clone(),
                script_chan,
                pipeline,
            };
            actors.register_script_actor(self.unique_id, name.clone());
            actors.register_later(Box::new(node_actor));
            name
        } else {
            actors.script_to_actor(self.unique_id)
        };

        NodeActorMsg {
            actor: actor_name,
            baseURI: self.base_uri,
            parent: actors.script_to_actor(self.parent.clone()),
            nodeType: self.node_type,
            namespaceURI: self.namespace_uri,
            nodeName: self.node_name,
            numChildren: self.num_children,

            name: self.name,
            publicId: self.public_id,
            systemId: self.system_id,

            attrs: self
                .attrs
                .into_iter()
                .map(|attr| AttrMsg {
                    namespace: attr.namespace,
                    name: attr.name,
                    value: attr.value,
                })
                .collect(),

            pseudoClassLocks: vec![], //TODO get this data from script

            isDisplayed: display,

            hasEventListeners: false, //TODO get this data from script

            isDocumentElement: self.is_document_element,

            shortValue: self.short_value,
            incompleteValue: self.incomplete_value,
        }
    }
}

struct WalkerActor {
    name: String,
    script_chan: IpcSender<DevtoolScriptControlMsg>,
    pipeline: PipelineId,
}

#[derive(Serialize)]
struct QuerySelectorReply {
    from: String,
}

#[derive(Serialize)]
struct DocumentElementReply {
    from: String,
    node: NodeActorMsg,
}

#[derive(Serialize)]
struct ClearPseudoclassesReply {
    from: String,
}

#[derive(Serialize)]
struct ChildrenReply {
    hasFirst: bool,
    hasLast: bool,
    nodes: Vec<NodeActorMsg>,
    from: String,
}

impl Actor for WalkerActor {
    fn name(&self) -> String {
        self.name.clone()
    }

    fn handle_message(
        &self,
        registry: &ActorRegistry,
        msg_type: &str,
        msg: &Map<String, Value>,
        stream: &mut TcpStream,
        _id: StreamId,
    ) -> Result<ActorMessageStatus, ()> {
        Ok(match msg_type {
            "querySelector" => {
                let msg = QuerySelectorReply { from: self.name() };
                let _ = stream.write_json_packet(&msg);
                ActorMessageStatus::Processed
            },

            "documentElement" => {
                let (tx, rx) = ipc::channel().unwrap();
                self.script_chan
                    .send(GetDocumentElement(self.pipeline, tx))
                    .unwrap();
                let doc_elem_info = rx.recv().unwrap().ok_or(())?;
                let node =
                    doc_elem_info.encode(registry, true, self.script_chan.clone(), self.pipeline);

                let msg = DocumentElementReply {
                    from: self.name(),
                    node,
                };
                let _ = stream.write_json_packet(&msg);
                ActorMessageStatus::Processed
            },

            "clearPseudoClassLocks" => {
                let msg = ClearPseudoclassesReply { from: self.name() };
                let _ = stream.write_json_packet(&msg);
                ActorMessageStatus::Processed
            },

            "children" => {
                let target = msg.get("node").unwrap().as_str().unwrap();
                let (tx, rx) = ipc::channel().unwrap();
                self.script_chan
                    .send(GetChildren(
                        self.pipeline,
                        registry.actor_to_script(target.to_owned()),
                        tx,
                    ))
                    .unwrap();
                let children = rx.recv().unwrap().ok_or(())?;

                let msg = ChildrenReply {
                    hasFirst: true,
                    hasLast: true,
                    nodes: children
                        .into_iter()
                        .map(|child| {
                            child.encode(registry, true, self.script_chan.clone(), self.pipeline)
                        })
                        .collect(),
                    from: self.name(),
                };
                let _ = stream.write_json_packet(&msg);
                ActorMessageStatus::Processed
            },

            _ => ActorMessageStatus::Ignored,
        })
    }
}

#[derive(Serialize)]
struct GetPageStyleReply {
    from: String,
    pageStyle: PageStyleMsg,
}

#[derive(Serialize)]
struct PageStyleMsg {
    actor: String,
}

struct PageStyleActor {
    name: String,
    script_chan: IpcSender<DevtoolScriptControlMsg>,
    pipeline: PipelineId,
}

#[derive(Serialize)]
struct GetAppliedReply {
    entries: Vec<AppliedEntry>,
    rules: Vec<AppliedRule>,
    sheets: Vec<AppliedSheet>,
    from: String,
}

#[derive(Serialize)]
struct GetComputedReply {
    computed: Vec<u32>, //XXX all css props
    from: String,
}

#[derive(Serialize)]
struct AppliedEntry {
    rule: String,
    pseudoElement: Value,
    isSystem: bool,
    matchedSelectors: Vec<String>,
}

#[derive(Serialize)]
struct AppliedRule {
    actor: String,
    #[serde(rename = "type")]
    type_: String,
    href: String,
    cssText: String,
    line: u32,
    column: u32,
    parentStyleSheet: String,
}

#[derive(Serialize)]
struct AppliedSheet {
    actor: String,
    href: String,
    nodeHref: String,
    disabled: bool,
    title: String,
    system: bool,
    styleSheetIndex: isize,
    ruleCount: usize,
}

#[derive(Serialize)]
struct GetLayoutReply {
    from: String,

    display: String,
    position: String,
    #[serde(rename = "z-index")]
    zIndex: String,
    #[serde(rename = "box-sizing")]
    boxSizing: String,

    // Would be nice to use a proper struct, blocked by
    // https://github.com/serde-rs/serde/issues/43
    autoMargins: serde_json::value::Value,
    #[serde(rename = "margin-top")]
    marginTop: String,
    #[serde(rename = "margin-right")]
    marginRight: String,
    #[serde(rename = "margin-bottom")]
    marginBottom: String,
    #[serde(rename = "margin-left")]
    marginLeft: String,

    #[serde(rename = "border-top-width")]
    borderTopWidth: String,
    #[serde(rename = "border-right-width")]
    borderRightWidth: String,
    #[serde(rename = "border-bottom-width")]
    borderBottomWidth: String,
    #[serde(rename = "border-left-width")]
    borderLeftWidth: String,

    #[serde(rename = "padding-top")]
    paddingTop: String,
    #[serde(rename = "padding-right")]
    paddingRight: String,
    #[serde(rename = "padding-bottom")]
    paddingBottom: String,
    #[serde(rename = "padding-left")]
    paddingLeft: String,

    width: f32,
    height: f32,
}

impl Actor for PageStyleActor {
    fn name(&self) -> String {
        self.name.clone()
    }

    fn handle_message(
        &self,
        registry: &ActorRegistry,
        msg_type: &str,
        msg: &Map<String, Value>,
        stream: &mut TcpStream,
        _id: StreamId,
    ) -> Result<ActorMessageStatus, ()> {
        Ok(match msg_type {
            "getApplied" => {
                //TODO: query script for relevant applied styles to node (msg.node)
                let msg = GetAppliedReply {
                    entries: vec![],
                    rules: vec![],
                    sheets: vec![],
                    from: self.name(),
                };
                let _ = stream.write_json_packet(&msg);
                ActorMessageStatus::Processed
            },

            "getComputed" => {
                //TODO: query script for relevant computed styles on node (msg.node)
                let msg = GetComputedReply {
                    computed: vec![],
                    from: self.name(),
                };
                let _ = stream.write_json_packet(&msg);
                ActorMessageStatus::Processed
            },

            //TODO: query script for box layout properties of node (msg.node)
            "getLayout" => {
                let target = msg.get("node").unwrap().as_str().unwrap();
                let (tx, rx) = ipc::channel().unwrap();
                self.script_chan
                    .send(GetLayout(
                        self.pipeline,
                        registry.actor_to_script(target.to_owned()),
                        tx,
                    ))
                    .unwrap();
                let ComputedNodeLayout {
                    display,
                    position,
                    z_index: zIndex,
                    box_sizing: boxSizing,
                    auto_margins: autoMargins,
                    margin_top: marginTop,
                    margin_right: marginRight,
                    margin_bottom: marginBottom,
                    margin_left: marginLeft,
                    border_top_width: borderTopWidth,
                    border_right_width: borderRightWidth,
                    border_bottom_width: borderBottomWidth,
                    border_left_width: borderLeftWidth,
                    padding_top: paddingTop,
                    padding_right: paddingRight,
                    padding_bottom: paddingBottom,
                    padding_left: paddingLeft,
                    width,
                    height,
                } = rx.recv().unwrap().ok_or(())?;

                let auto_margins = msg
                    .get("autoMargins")
                    .and_then(Value::as_bool)
                    .unwrap_or(false);

                // http://mxr.mozilla.org/mozilla-central/source/toolkit/devtools/server/actors/styles.js
                let msg = GetLayoutReply {
                    from: self.name(),
                    display,
                    position,
                    zIndex,
                    boxSizing,
                    autoMargins: if auto_margins {
                        let mut m = Map::new();
                        let auto = serde_json::value::Value::String("auto".to_owned());
                        if autoMargins.top {
                            m.insert("top".to_owned(), auto.clone());
                        }
                        if autoMargins.right {
                            m.insert("right".to_owned(), auto.clone());
                        }
                        if autoMargins.bottom {
                            m.insert("bottom".to_owned(), auto.clone());
                        }
                        if autoMargins.left {
                            m.insert("left".to_owned(), auto);
                        }
                        serde_json::value::Value::Object(m)
                    } else {
                        serde_json::value::Value::Null
                    },
                    marginTop,
                    marginRight,
                    marginBottom,
                    marginLeft,
                    borderTopWidth,
                    borderRightWidth,
                    borderBottomWidth,
                    borderLeftWidth,
                    paddingTop,
                    paddingRight,
                    paddingBottom,
                    paddingLeft,
                    width,
                    height,
                };
                let msg = serde_json::to_string(&msg).unwrap();
                let msg = serde_json::from_str::<Value>(&msg).unwrap();
                let _ = stream.write_json_packet(&msg);
                ActorMessageStatus::Processed
            },

            _ => ActorMessageStatus::Ignored,
        })
    }
}

impl Actor for InspectorActor {
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
        let browsing_context = registry.find::<BrowsingContextActor>(&self.browsing_context);
        let pipeline = browsing_context.active_pipeline.get();
        Ok(match msg_type {
            "getWalker" => {
                if self.walker.borrow().is_none() {
                    let walker = WalkerActor {
                        name: registry.new_name("walker"),
                        script_chan: self.script_chan.clone(),
                        pipeline,
                    };
                    let mut walker_name = self.walker.borrow_mut();
                    *walker_name = Some(walker.name());
                    registry.register_later(Box::new(walker));
                }

                let (tx, rx) = ipc::channel().unwrap();
                self.script_chan.send(GetRootNode(pipeline, tx)).unwrap();
                let root_info = rx.recv().unwrap().ok_or(())?;

                let node = root_info.encode(registry, false, self.script_chan.clone(), pipeline);

                let msg = GetWalkerReply {
                    from: self.name(),
                    walker: WalkerMsg {
                        actor: self.walker.borrow().clone().unwrap(),
                        root: node,
                    },
                };
                let _ = stream.write_json_packet(&msg);
                ActorMessageStatus::Processed
            },

            "getPageStyle" => {
                if self.page_style.borrow().is_none() {
                    let style = PageStyleActor {
                        name: registry.new_name("pageStyle"),
                        script_chan: self.script_chan.clone(),
                        pipeline,
                    };
                    let mut pageStyle = self.page_style.borrow_mut();
                    *pageStyle = Some(style.name());
                    registry.register_later(Box::new(style));
                }

                let msg = GetPageStyleReply {
                    from: self.name(),
                    pageStyle: PageStyleMsg {
                        actor: self.page_style.borrow().clone().unwrap(),
                    },
                };
                let _ = stream.write_json_packet(&msg);
                ActorMessageStatus::Processed
            },

            //TODO: this is an old message; try adding highlightable to the root traits instead
            //      and support getHighlighter instead
            //"highlight" => {}
            "getHighlighter" => {
                if self.highlighter.borrow().is_none() {
                    let highlighter_actor = HighlighterActor {
                        name: registry.new_name("highlighter"),
                    };
                    let mut highlighter = self.highlighter.borrow_mut();
                    *highlighter = Some(highlighter_actor.name());
                    registry.register_later(Box::new(highlighter_actor));
                }

                let msg = GetHighlighterReply {
                    from: self.name(),
                    highligter: HighlighterMsg {
                        actor: self.highlighter.borrow().clone().unwrap(),
                    },
                };
                let _ = stream.write_json_packet(&msg);
                ActorMessageStatus::Processed
            },

            _ => ActorMessageStatus::Ignored,
        })
    }
}

/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

/// Liberally derived from the [Firefox JS implementation](http://mxr.mozilla.org/mozilla-central/source/toolkit/devtools/server/actors/inspector.js).

use devtools_traits::{DevtoolScriptControlMsg, NodeInfo};
use devtools_traits::DevtoolScriptControlMsg::{GetRootNode, GetDocumentElement, GetChildren};
use devtools_traits::DevtoolScriptControlMsg::{GetLayout, ModifyAttribute};

use actor::{Actor, ActorRegistry};
use protocol::JsonPacketStream;

use collections::BTreeMap;
use msg::constellation_msg::PipelineId;
use serialize::json::{self, Json, ToJson};
use std::cell::RefCell;
use std::old_io::TcpStream;
use std::sync::mpsc::{channel, Sender};
use std::num::Float;

pub struct InspectorActor {
    pub name: String,
    pub walker: RefCell<Option<String>>,
    pub pageStyle: RefCell<Option<String>>,
    pub highlighter: RefCell<Option<String>>,
    pub script_chan: Sender<DevtoolScriptControlMsg>,
    pub pipeline: PipelineId,
}

#[derive(RustcEncodable)]
struct GetHighlighterReply {
    highligter: HighlighterMsg, // sic.
    from: String,
}

#[derive(RustcEncodable)]
struct HighlighterMsg {
    actor: String,
}

struct HighlighterActor {
    name: String,
}

pub struct NodeActor {
    pub name: String,
    script_chan: Sender<DevtoolScriptControlMsg>,
    pipeline: PipelineId,
}

#[derive(RustcEncodable)]
struct ShowBoxModelReply {
    from: String,
}

#[derive(RustcEncodable)]
struct HideBoxModelReply {
    from: String,
}

impl Actor for HighlighterActor {
    fn name(&self) -> String {
        self.name.clone()
    }

    fn handle_message(&self,
                      _registry: &ActorRegistry,
                      msg_type: &String,
                      _msg: &json::Object,
                      stream: &mut TcpStream) -> Result<bool, ()> {
        Ok(match msg_type.as_slice() {
            "showBoxModel" => {
                let msg = ShowBoxModelReply {
                    from: self.name(),
                };
                stream.write_json_packet(&msg);
                true
            }

            "hideBoxModel" => {
                let msg = HideBoxModelReply {
                    from: self.name(),
                };
                stream.write_json_packet(&msg);
                true
            }

            _ => false,
        })
    }
}

#[derive(RustcEncodable)]
struct ModifyAttributeReply{
    from: String,
}

impl Actor for NodeActor {
    fn name(&self) -> String {
        self.name.clone()
    }

    fn handle_message(&self,
                      registry: &ActorRegistry,
                      msg_type: &String,
                      msg: &json::Object,
                      stream: &mut TcpStream) -> Result<bool, ()> {
        Ok(match msg_type.as_slice() {
            "modifyAttributes" => {
                let target = msg.get(&"to".to_string()).unwrap().as_string().unwrap();
                let mods = msg.get(&"modifications".to_string()).unwrap().as_array().unwrap();
                let modifications = mods.iter().map(|json_mod| {
                    json::decode(json_mod.to_string().as_slice()).unwrap()
                }).collect();

                self.script_chan.send(ModifyAttribute(self.pipeline,
                                                      registry.actor_to_script(target.to_string()),
                                                      modifications))
                                .unwrap();
                let reply = ModifyAttributeReply{
                    from: self.name(),
                };
                stream.write_json_packet(&reply);
                true
            }

            _ => false,
        })
    }
}

#[derive(RustcEncodable)]
struct GetWalkerReply {
    from: String,
    walker: WalkerMsg,
}

#[derive(RustcEncodable)]
struct WalkerMsg {
    actor: String,
    root: NodeActorMsg,
}

#[derive(RustcEncodable)]
struct AttrMsg {
    namespace: String,
    name: String,
    value: String,
}

#[derive(RustcEncodable)]
struct NodeActorMsg {
    actor: String,
    baseURI: String,
    parent: String,
    nodeType: uint,
    namespaceURI: String,
    nodeName: String,
    numChildren: uint,

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
    fn encode(self,
              actors: &ActorRegistry,
              display: bool,
              script_chan: Sender<DevtoolScriptControlMsg>,
              pipeline: PipelineId) -> NodeActorMsg;
}

impl NodeInfoToProtocol for NodeInfo {
    fn encode(self,
              actors: &ActorRegistry,
              display: bool,
              script_chan: Sender<DevtoolScriptControlMsg>,
              pipeline: PipelineId) -> NodeActorMsg {
        let actor_name = if !actors.script_actor_registered(self.uniqueId.clone()) {
            let name = actors.new_name("node");
            let node_actor = NodeActor {
                name: name.clone(),
                script_chan: script_chan,
                pipeline: pipeline.clone(),
            };
            actors.register_script_actor(self.uniqueId, name.clone());
            actors.register_later(box node_actor);
            name
        } else {
            actors.script_to_actor(self.uniqueId)
        };

        NodeActorMsg {
            actor: actor_name,
            baseURI: self.baseURI,
            parent: actors.script_to_actor(self.parent.clone()),
            nodeType: self.nodeType,
            namespaceURI: self.namespaceURI,
            nodeName: self.nodeName,
            numChildren: self.numChildren,

            name: self.name,
            publicId: self.publicId,
            systemId: self.systemId,

            attrs: self.attrs.into_iter().map(|attr| {
                AttrMsg {
                    namespace: attr.namespace,
                    name: attr.name,
                    value: attr.value,
                }
            }).collect(),

            pseudoClassLocks: vec!(), //TODO get this data from script

            isDisplayed: display,

            hasEventListeners: false, //TODO get this data from script

            isDocumentElement: self.isDocumentElement,

            shortValue: self.shortValue,
            incompleteValue: self.incompleteValue,
        }
    }
}

struct WalkerActor {
    name: String,
    script_chan: Sender<DevtoolScriptControlMsg>,
    pipeline: PipelineId,
}

#[derive(RustcEncodable)]
struct QuerySelectorReply {
    from: String,
}

#[derive(RustcEncodable)]
struct DocumentElementReply {
    from: String,
    node: NodeActorMsg,
}

#[derive(RustcEncodable)]
struct ClearPseudoclassesReply {
    from: String,
}

#[derive(RustcEncodable)]
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

    fn handle_message(&self,
                      registry: &ActorRegistry,
                      msg_type: &String,
                      msg: &json::Object,
                      stream: &mut TcpStream) -> Result<bool, ()> {
        Ok(match msg_type.as_slice() {
            "querySelector" => {
                let msg = QuerySelectorReply {
                    from: self.name(),
                };
                stream.write_json_packet(&msg);
                true
            }

            "documentElement" => {
                let (tx, rx) = channel();
                self.script_chan.send(GetDocumentElement(self.pipeline, tx)).unwrap();
                let doc_elem_info = rx.recv().unwrap();
                let node = doc_elem_info.encode(registry, true, self.script_chan.clone(), self.pipeline);

                let msg = DocumentElementReply {
                    from: self.name(),
                    node: node,
                };
                stream.write_json_packet(&msg);
                true
            }

            "clearPseudoClassLocks" => {
                let msg = ClearPseudoclassesReply {
                    from: self.name(),
                };
                stream.write_json_packet(&msg);
                true
            }

            "children" => {
                let target = msg.get(&"node".to_string()).unwrap().as_string().unwrap();
                let (tx, rx) = channel();
                self.script_chan.send(GetChildren(self.pipeline,
                                                  registry.actor_to_script(target.to_string()),
                                                  tx))
                                .unwrap();
                let children = rx.recv().unwrap();

                let msg = ChildrenReply {
                    hasFirst: true,
                    hasLast: true,
                    nodes: children.into_iter().map(|child| {
                        child.encode(registry, true, self.script_chan.clone(), self.pipeline)
                    }).collect(),
                    from: self.name(),
                };
                stream.write_json_packet(&msg);
                true
            }

            _ => false,
        })
    }
}

#[derive(RustcEncodable)]
struct GetPageStyleReply {
    from: String,
    pageStyle: PageStyleMsg,
}

#[derive(RustcEncodable)]
struct PageStyleMsg {
    actor: String,
}

struct PageStyleActor {
    name: String,
    script_chan: Sender<DevtoolScriptControlMsg>,
    pipeline: PipelineId,
}

#[derive(RustcEncodable)]
struct GetAppliedReply {
    entries: Vec<AppliedEntry>,
    rules: Vec<AppliedRule>,
    sheets: Vec<AppliedSheet>,
    from: String,
}

#[derive(RustcEncodable)]
struct GetComputedReply {
    computed: Vec<uint>, //XXX all css props
    from: String,
}

#[derive(RustcEncodable)]
struct AppliedEntry {
    rule: String,
    pseudoElement: Json,
    isSystem: bool,
    matchedSelectors: Vec<String>,
}

#[derive(RustcEncodable)]
struct AppliedRule {
    actor: String,
    __type__: uint,
    href: String,
    cssText: String,
    line: uint,
    column: uint,
    parentStyleSheet: String,
}

#[derive(RustcEncodable)]
struct AppliedSheet {
    actor: String,
    href: String,
    nodeHref: String,
    disabled: bool,
    title: String,
    system: bool,
    styleSheetIndex: int,
    ruleCount: uint,
}

#[derive(RustcEncodable)]
struct GetLayoutReply {
    width: int,
    height: int,
    autoMargins: Json,
    from: String,
}

#[derive(RustcEncodable)]
#[allow(dead_code)]
struct AutoMargins {
    top: String,
    bottom: String,
    left: String,
    right: String,
}

impl Actor for PageStyleActor {
    fn name(&self) -> String {
        self.name.clone()
    }

    fn handle_message(&self,
                      registry: &ActorRegistry,
                      msg_type: &String,
                      msg: &json::Object,
                      stream: &mut TcpStream) -> Result<bool, ()> {
        Ok(match msg_type.as_slice() {
            "getApplied" => {
                //TODO: query script for relevant applied styles to node (msg.node)
                let msg = GetAppliedReply {
                    entries: vec!(),
                    rules: vec!(),
                    sheets: vec!(),
                    from: self.name(),
                };
                stream.write_json_packet(&msg);
                true
            }

            "getComputed" => {
                //TODO: query script for relevant computed styles on node (msg.node)
                let msg = GetComputedReply {
                    computed: vec!(),
                    from: self.name(),
                };
                stream.write_json_packet(&msg);
                true
            }

            //TODO: query script for box layout properties of node (msg.node)
            "getLayout" => {
                let target = msg.get(&"node".to_string()).unwrap().as_string().unwrap();
                let (tx, rx) = channel();
                self.script_chan.send(GetLayout(self.pipeline,
                                      registry.actor_to_script(target.to_string()),
                                      tx))
                                .unwrap();
                let (width, height) = rx.recv().unwrap();

                let auto_margins = msg.get(&"autoMargins".to_string()).unwrap().as_boolean().unwrap();

                //TODO: the remaining layout properties (margin, border, padding, position)
                //      as specified in getLayout in http://mxr.mozilla.org/mozilla-central/source/toolkit/devtools/server/actors/styles.js
                let msg = GetLayoutReply {
                    width: width.round() as int,
                    height: height.round() as int,
                    autoMargins: if auto_margins {
                        //TODO: real values like processMargins in http://mxr.mozilla.org/mozilla-central/source/toolkit/devtools/server/actors/styles.js
                        let mut m = BTreeMap::new();
                        m.insert("top".to_string(), "auto".to_string().to_json());
                        m.insert("bottom".to_string(), "auto".to_string().to_json());
                        m.insert("left".to_string(), "auto".to_string().to_json());
                        m.insert("right".to_string(), "auto".to_string().to_json());
                        Json::Object(m)
                    } else {
                        Json::Null
                    },
                    from: self.name(),
                };
                stream.write_json_packet(&msg);
                true
            }

            _ => false,
        })
    }
}

impl Actor for InspectorActor {
    fn name(&self) -> String {
        self.name.clone()
    }

    fn handle_message(&self,
                      registry: &ActorRegistry,
                      msg_type: &String,
                      _msg: &json::Object,
                      stream: &mut TcpStream) -> Result<bool, ()> {
        Ok(match msg_type.as_slice() {
            "getWalker" => {
                if self.walker.borrow().is_none() {
                    let walker = WalkerActor {
                        name: registry.new_name("walker"),
                        script_chan: self.script_chan.clone(),
                        pipeline: self.pipeline,
                    };
                    let mut walker_name = self.walker.borrow_mut();
                    *walker_name = Some(walker.name());
                    registry.register_later(box walker);
                }

                let (tx, rx) = channel();
                self.script_chan.send(GetRootNode(self.pipeline, tx)).unwrap();
                let root_info = rx.recv().unwrap();

                let node = root_info.encode(registry, false, self.script_chan.clone(), self.pipeline);

                let msg = GetWalkerReply {
                    from: self.name(),
                    walker: WalkerMsg {
                        actor: self.walker.borrow().clone().unwrap(),
                        root: node,
                    }
                };
                stream.write_json_packet(&msg);
                true
            }

            "getPageStyle" => {
                if self.pageStyle.borrow().is_none() {
                    let style = PageStyleActor {
                        name: registry.new_name("pageStyle"),
                        script_chan: self.script_chan.clone(),
                        pipeline: self.pipeline,
                    };
                    let mut pageStyle = self.pageStyle.borrow_mut();
                    *pageStyle = Some(style.name());
                    registry.register_later(box style);
                }

                let msg = GetPageStyleReply {
                    from: self.name(),
                    pageStyle: PageStyleMsg {
                        actor: self.pageStyle.borrow().clone().unwrap(),
                    },
                };
                stream.write_json_packet(&msg);
                true
            }

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
                    registry.register_later(box highlighter_actor);
                }

                let msg = GetHighlighterReply {
                    from: self.name(),
                    highligter: HighlighterMsg {
                        actor: self.highlighter.borrow().clone().unwrap(),
                    },
                };
                stream.write_json_packet(&msg);
                true
            }

            _ => false,
        })
    }
}

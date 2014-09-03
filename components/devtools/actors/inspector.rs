/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

/// Liberally derived from the [Firefox JS implementation](http://mxr.mozilla.org/mozilla-central/source/toolkit/devtools/server/actors/inspector.js).

use actor::{Actor, ActorRegistry};
use protocol::JsonPacketSender;

use serialize::json;
use std::cell::RefCell;
use std::io::TcpStream;

pub struct InspectorActor {
    pub name: String,
    pub walker: RefCell<Option<String>>,
    pub pageStyle: RefCell<Option<String>>,
    pub highlighter: RefCell<Option<String>>,
}

#[deriving(Encodable)]
struct GetHighlighterReply {
    highligter: HighlighterMsg, // sic.
    from: String,
}

#[deriving(Encodable)]
struct HighlighterMsg {
    actor: String,
}

struct HighlighterActor {
    name: String,
}

#[deriving(Encodable)]
struct ShowBoxModelReply {
    from: String,
}

#[deriving(Encodable)]
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
                      stream: &mut TcpStream) -> bool {
        match msg_type.as_slice() {
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
        }
    }
}

#[deriving(Encodable)]
struct GetWalkerReply {
    from: String,
    walker: WalkerMsg,
}

#[deriving(Encodable)]
struct WalkerMsg {
    actor: String,
    root: NodeActorMsg,
}

#[deriving(Encodable)]
struct AttrMsg {
    namespace: String,
    name: String,
    value: String,
}

#[deriving(Encodable)]
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

struct WalkerActor {
    name: String,
}

#[deriving(Encodable)]
struct QuerySelectorReply {
    from: String,
}

#[deriving(Encodable)]
struct DocumentElementReply {
    from: String,
    node: NodeActorMsg,
}

#[deriving(Encodable)]
struct ClearPseudoclassesReply {
    from: String,
}

#[deriving(Encodable)]
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
                      _registry: &ActorRegistry,
                      msg_type: &String,
                      _msg: &json::Object,
                      stream: &mut TcpStream) -> bool {
        match msg_type.as_slice() {
            "querySelector" => {
                let msg = QuerySelectorReply {
                    from: self.name(),
                };
                stream.write_json_packet(&msg);
                true
            }

            "documentElement" => {
                let msg = DocumentElementReply {
                    from: self.name(),
                    node: NodeActorMsg {
                        actor: "node0".to_string(),
                        baseURI: "".to_string(),
                        parent: "".to_string(),
                        nodeType: 1, //ELEMENT_NODE
                        namespaceURI: "".to_string(),
                        nodeName: "html".to_string(),
                        numChildren: 0,

                        name: "".to_string(),
                        publicId: "".to_string(),
                        systemId: "".to_string(),

                        attrs: vec!(AttrMsg {
                            namespace: "".to_string(),
                            name: "manifest".to_string(),
                            value: "foo.manifest".to_string(),
                        }),

                        pseudoClassLocks: vec!(),

                        isDisplayed: true,

                        hasEventListeners: false,

                        isDocumentElement: true,

                        shortValue: "".to_string(),
                        incompleteValue: false,
                    }
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
                let msg = ChildrenReply {
                    hasFirst: true,
                    hasLast: true,
                    nodes: vec!(),
                    from: self.name(),
                };
                stream.write_json_packet(&msg);
                true
            }

            _ => false,
        }
    }
}

struct NodeActor {
    name: String,
}

impl Actor for NodeActor {
    fn name(&self) -> String {
        self.name.clone()
    }

    fn handle_message(&self,
                      _registry: &ActorRegistry,
                      msg_type: &String,
                      _msg: &json::Object,
                      _stream: &mut TcpStream) -> bool {
        match msg_type.as_slice() {
            _ => false,
        }
    }
}

#[deriving(Encodable)]
struct GetPageStyleReply {
    from: String,
    pageStyle: PageStyleMsg,
}

#[deriving(Encodable)]
struct PageStyleMsg {
    actor: String,
}

struct PageStyleActor {
    name: String,
}

#[deriving(Encodable)]
struct GetAppliedReply {
    entries: Vec<AppliedEntry>,
    rules: Vec<AppliedRule>,
    sheets: Vec<AppliedSheet>,
    from: String,
}

#[deriving(Encodable)]
struct GetComputedReply {
    computed: Vec<uint>, //XXX all css props
    from: String,
}

#[deriving(Encodable)]
struct AppliedEntry {
    rule: String,
    pseudoElement: json::Json,
    isSystem: bool,
    matchedSelectors: Vec<String>,
}

#[deriving(Encodable)]
struct AppliedRule {
    actor: String,
    __type__: uint,
    href: String,
    cssText: String,
    line: uint,
    column: uint,
    parentStyleSheet: String,
}

#[deriving(Encodable)]
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

impl Actor for PageStyleActor {
    fn name(&self) -> String {
        self.name.clone()
    }

    fn handle_message(&self,
                      _registry: &ActorRegistry,
                      msg_type: &String,
                      _msg: &json::Object,
                      stream: &mut TcpStream) -> bool {
        match msg_type.as_slice() {
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
            //"getLayout" => {}

            _ => false,
        }
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
                      stream: &mut TcpStream) -> bool {
        match msg_type.as_slice() {
            "getWalker" => {
                if self.walker.borrow().is_none() {
                    let walker = WalkerActor {
                        name: registry.new_name("walker"),
                    };
                    let mut walker_name = self.walker.borrow_mut();
                    *walker_name = Some(walker.name());
                    registry.register_later(box walker);
                }

                let node = NodeActor {
                    name: registry.new_name("node"),
                };
                let node_actor_name = node.name();
                registry.register_later(box node);

                //TODO: query script for actual root node
                //TODO: extra node actor creation
                let node = NodeActorMsg {
                    actor: node_actor_name,
                    baseURI: "".to_string(),
                    parent: "".to_string(),
                    nodeType: 1, //ELEMENT_NODE
                    namespaceURI: "".to_string(),
                    nodeName: "html".to_string(),
                    numChildren: 1,

                    name: "".to_string(),
                    publicId: "".to_string(),
                    systemId: "".to_string(),

                    attrs: vec!(AttrMsg {
                        namespace: "".to_string(),
                        name: "manifest".to_string(),
                        value: "foo.manifest".to_string(),
                    }),

                    pseudoClassLocks: vec!(),

                    isDisplayed: true,

                    hasEventListeners: false,

                    isDocumentElement: true,

                    shortValue: "".to_string(),
                    incompleteValue: false,
                };

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
        }
    }
}

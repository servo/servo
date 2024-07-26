/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! The walker actor is responsible for traversing the DOM tree in various ways to create new nodes

use std::net::TcpStream;

use base::id::PipelineId;
use devtools_traits::DevtoolScriptControlMsg;
use devtools_traits::DevtoolScriptControlMsg::{GetChildren, GetDocumentElement};
use ipc_channel::ipc::{self, IpcSender};
use serde::Serialize;
use serde_json::{self, Map, Value};

use crate::actor::{Actor, ActorMessageStatus, ActorRegistry};
use crate::actors::inspector::layout::{LayoutInspectorActor, LayoutInspectorActorMsg};
use crate::actors::inspector::node::{NodeActorMsg, NodeInfoToProtocol};
use crate::protocol::JsonPacketStream;
use crate::{EmptyReplyMsg, StreamId};

#[derive(Serialize)]
pub struct WalkerMsg {
    pub actor: String,
    pub root: NodeActorMsg,
}

pub struct WalkerActor {
    pub name: String,
    pub script_chan: IpcSender<DevtoolScriptControlMsg>,
    pub pipeline: PipelineId,
    pub root_node: NodeActorMsg,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct QuerySelectorReply {
    from: String,
    node: NodeActorMsg,
    new_parents: Vec<NodeActorMsg>,
}

#[derive(Serialize)]
struct DocumentElementReply {
    from: String,
    node: NodeActorMsg,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ChildrenReply {
    has_first: bool,
    has_last: bool,
    nodes: Vec<NodeActorMsg>,
    from: String,
}

#[derive(Serialize)]
struct GetLayoutInspectorReply {
    actor: LayoutInspectorActorMsg,
    from: String,
}

#[derive(Serialize)]
struct WatchRootNodeReply {
    #[serde(rename = "type")]
    type_: String,
    from: String,
    node: NodeActorMsg,
}

#[derive(Serialize)]
struct GetOffsetParentReply {
    from: String,
    node: Option<()>,
}

impl Actor for WalkerActor {
    fn name(&self) -> String {
        self.name.clone()
    }

    /// The walker actor can handle the following messages:
    ///
    /// - `children`: Returns a list of children nodes of the specified node
    ///
    /// - `clearPseudoClassLocks`: Placeholder
    ///
    /// - `documentElement`: Returns the base document element node
    ///
    /// - `getLayoutInspector`: Returns the Layout inspector actor, placeholder
    ///
    /// - `getOffsetParent`: Placeholder
    ///
    /// - `querySelector`: Recursively looks for the specified selector in the tree, reutrning the
    /// node and its ascendents
    fn handle_message(
        &self,
        registry: &ActorRegistry,
        msg_type: &str,
        msg: &Map<String, Value>,
        stream: &mut TcpStream,
        _id: StreamId,
    ) -> Result<ActorMessageStatus, ()> {
        Ok(match msg_type {
            "children" => {
                let target = msg.get("node").ok_or(())?.as_str().ok_or(())?;
                let (tx, rx) = ipc::channel().map_err(|_| ())?;
                self.script_chan
                    .send(GetChildren(
                        self.pipeline,
                        registry.actor_to_script(target.into()),
                        tx,
                    ))
                    .map_err(|_| ())?;
                let children = rx.recv().map_err(|_| ())?.ok_or(())?;

                let msg = ChildrenReply {
                    has_first: true,
                    has_last: true,
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
            "clearPseudoClassLocks" => {
                let msg = EmptyReplyMsg { from: self.name() };
                let _ = stream.write_json_packet(&msg);
                ActorMessageStatus::Processed
            },
            "documentElement" => {
                let (tx, rx) = ipc::channel().map_err(|_| ())?;
                self.script_chan
                    .send(GetDocumentElement(self.pipeline, tx))
                    .map_err(|_| ())?;
                let doc_elem_info = rx.recv().map_err(|_| ())?.ok_or(())?;
                let node =
                    doc_elem_info.encode(registry, true, self.script_chan.clone(), self.pipeline);

                let msg = DocumentElementReply {
                    from: self.name(),
                    node,
                };
                let _ = stream.write_json_packet(&msg);
                ActorMessageStatus::Processed
            },
            "getLayoutInspector" => {
                // TODO: Create actual layout inspector actor
                let layout = LayoutInspectorActor::new(registry.new_name("layout"));
                let actor = layout.encodable();
                registry.register_later(Box::new(layout));

                let msg = GetLayoutInspectorReply {
                    from: self.name(),
                    actor,
                };
                let _ = stream.write_json_packet(&msg);
                ActorMessageStatus::Processed
            },
            "getOffsetParent" => {
                let msg = GetOffsetParentReply {
                    from: self.name(),
                    node: None,
                };
                let _ = stream.write_json_packet(&msg);
                ActorMessageStatus::Processed
            },
            "querySelector" => {
                let selector = msg.get("selector").ok_or(())?.as_str().ok_or(())?;
                let node = msg.get("node").ok_or(())?.as_str().ok_or(())?;
                let mut hierarchy = find_child(
                    &self.script_chan,
                    self.pipeline,
                    registry,
                    selector,
                    node,
                    vec![],
                )
                .map_err(|_| ())?;
                hierarchy.reverse();
                let node = hierarchy.pop().ok_or(())?;

                let msg = QuerySelectorReply {
                    from: self.name(),
                    node,
                    new_parents: hierarchy,
                };
                let _ = stream.write_json_packet(&msg);
                ActorMessageStatus::Processed
            },
            "watchRootNode" => {
                let msg = WatchRootNodeReply {
                    type_: "root-available".into(),
                    from: self.name(),
                    node: self.root_node.clone(),
                };
                let _ = stream.write_json_packet(&msg);

                let msg = EmptyReplyMsg { from: self.name() };
                let _ = stream.write_json_packet(&msg);
                ActorMessageStatus::Processed
            },
            _ => ActorMessageStatus::Ignored,
        })
    }
}

/// Recursively searches for a child with the specified selector
/// If it is found, returns a list with the child and all of its ancestors.
fn find_child(
    script_chan: &IpcSender<DevtoolScriptControlMsg>,
    pipeline: PipelineId,
    registry: &ActorRegistry,
    selector: &str,
    node: &str,
    mut hierarchy: Vec<NodeActorMsg>,
) -> Result<Vec<NodeActorMsg>, Vec<NodeActorMsg>> {
    let (tx, rx) = ipc::channel().unwrap();
    script_chan
        .send(GetChildren(
            pipeline,
            registry.actor_to_script(node.into()),
            tx,
        ))
        .unwrap();
    let children = rx.recv().unwrap().ok_or(vec![])?;

    for child in children {
        let msg = child.encode(registry, true, script_chan.clone(), pipeline);
        if msg.display_name == selector {
            hierarchy.push(msg);
            return Ok(hierarchy);
        };

        if msg.num_children == 0 {
            continue;
        }

        match find_child(
            script_chan,
            pipeline,
            registry,
            selector,
            &msg.actor,
            hierarchy,
        ) {
            Ok(mut hierarchy) => {
                hierarchy.push(msg);
                return Ok(hierarchy);
            },
            Err(e) => {
                hierarchy = e;
            },
        }
    }
    Err(hierarchy)
}

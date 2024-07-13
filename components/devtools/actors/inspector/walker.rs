/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

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
struct ClearPseudoclassesReply {
    from: String,
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
                let selector = msg.get("selector").unwrap().as_str().unwrap();

                let node = msg.get("node").unwrap().as_str().unwrap();
                let mut hierarchy = vec![];
                find_child(
                    &self.script_chan,
                    self.pipeline,
                    registry,
                    selector,
                    node,
                    &mut hierarchy,
                )
                .ok_or(())?;

                hierarchy.reverse();
                let node = hierarchy.pop().unwrap();

                let msg = QuerySelectorReply {
                    from: self.name(),
                    node,
                    new_parents: hierarchy,
                };
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

            "watchRootNode" => {
                let _ = stream.write_json_packet(&WatchRootNodeReply {
                    type_: "root-available".into(),
                    from: self.name(),
                    node: self.root_node.clone(),
                });

                let _ = stream.write_json_packet(&EmptyReplyMsg { from: self.name() });

                ActorMessageStatus::Processed
            },

            "getLayoutInspector" => {
                // TODO: Save layout
                let layout = LayoutInspectorActor::new(registry.new_name("layout"));
                let actor = layout.encodable();
                registry.register_later(Box::new(layout));

                let _ = stream.write_json_packet(&GetLayoutInspectorReply {
                    from: self.name(),
                    actor,
                });

                ActorMessageStatus::Processed
            },

            _ => ActorMessageStatus::Ignored,
        })
    }
}

fn find_child(
    script_chan: &IpcSender<DevtoolScriptControlMsg>,
    pipeline: PipelineId,
    registry: &ActorRegistry,
    selector: &str,
    node: &str,
    hierarchy: &mut Vec<NodeActorMsg>,
) -> Option<()> {
    let (tx, rx) = ipc::channel().unwrap();
    script_chan
        .send(GetChildren(
            pipeline,
            registry.actor_to_script(node.into()),
            tx,
        ))
        .unwrap();
    let children = rx.recv().unwrap()?;

    for child in children {
        let msg = child.encode(registry, true, script_chan.clone(), pipeline);
        if msg.display_name == selector ||
            (msg.num_children > 0 &&
                find_child(
                    script_chan,
                    pipeline,
                    registry,
                    selector,
                    &msg.actor,
                    hierarchy,
                )
                .is_some())
        {
            hierarchy.push(msg);
            return Some(());
        };
    }
    None
}

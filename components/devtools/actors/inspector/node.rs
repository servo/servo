/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! This actor represents one DOM node. It is created by the Walker actor when it is traversing the
//! document tree.

use std::collections::HashMap;
use std::net::TcpStream;

use base::id::PipelineId;
use devtools_traits::DevtoolScriptControlMsg::{GetDocumentElement, ModifyAttribute};
use devtools_traits::{DevtoolScriptControlMsg, NodeInfo};
use ipc_channel::ipc::{self, IpcSender};
use serde::Serialize;
use serde_json::{self, Map, Value};

use crate::actor::{Actor, ActorMessageStatus, ActorRegistry};
use crate::protocol::JsonPacketStream;
use crate::{EmptyReplyMsg, StreamId};

#[derive(Serialize)]
struct GetUniqueSelectorReply {
    from: String,
    value: String,
}

#[derive(Clone, Serialize)]
struct AttrMsg {
    name: String,
    value: String,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NodeActorMsg {
    pub actor: String,
    #[serde(rename = "baseURI")]
    base_uri: String,
    causes_overflow: bool,
    container_type: Option<()>,
    pub display_name: String,
    display_type: Option<String>,
    is_after_pseudo_element: bool,
    is_anonymous: bool,
    is_before_pseudo_element: bool,
    is_direct_shadow_host_child: Option<bool>,
    is_displayed: bool,
    #[serde(rename = "isInHTMLDocument")]
    is_in_html_document: Option<bool>,
    is_marker_pseudo_element: bool,
    is_native_anonymous: bool,
    is_scrollable: bool,
    is_shadow_root: bool,
    is_top_level_document: bool,
    node_name: String,
    node_type: u16,
    node_value: Option<()>,
    pub num_children: usize,
    #[serde(skip_serializing_if = "String::is_empty")]
    parent: String,
    shadow_root_mode: Option<()>,
    traits: HashMap<String, ()>,
    attrs: Vec<AttrMsg>,
}

pub struct NodeActor {
    name: String,
    script_chan: IpcSender<DevtoolScriptControlMsg>,
    pipeline: PipelineId,
}

impl Actor for NodeActor {
    fn name(&self) -> String {
        self.name.clone()
    }

    /// The node actor can handle the following messages:
    ///
    /// - `modifyAttributes`: Asks the script to change a value in the attribute of the
    /// corresponding node
    ///
    /// - `getUniqueSelector`: Returns the display name of this node
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
                let target = msg.get("to").ok_or(())?.as_str().ok_or(())?;
                let mods = msg.get("modifications").ok_or(())?.as_array().ok_or(())?;
                let modifications = mods
                    .iter()
                    .filter_map(|json_mod| {
                        serde_json::from_str(&serde_json::to_string(json_mod).ok()?).ok()
                    })
                    .collect();
                self.script_chan
                    .send(ModifyAttribute(
                        self.pipeline,
                        registry.actor_to_script(target.to_owned()),
                        modifications,
                    ))
                    .map_err(|_| ())?;

                let reply = EmptyReplyMsg { from: self.name() };
                let _ = stream.write_json_packet(&reply);
                ActorMessageStatus::Processed
            },

            "getUniqueSelector" => {
                let (tx, rx) = ipc::channel().unwrap();
                self.script_chan
                    .send(GetDocumentElement(self.pipeline, tx))
                    .unwrap();
                let doc_elem_info = rx.recv().map_err(|_| ())?.ok_or(())?;
                let node =
                    doc_elem_info.encode(registry, true, self.script_chan.clone(), self.pipeline);

                let msg = GetUniqueSelectorReply {
                    from: self.name(),
                    value: node.display_name,
                };
                let _ = stream.write_json_packet(&msg);
                ActorMessageStatus::Processed
            },

            _ => ActorMessageStatus::Ignored,
        })
    }
}

pub trait NodeInfoToProtocol {
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
        let actor = if !actors.script_actor_registered(self.unique_id.clone()) {
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
            actor,
            base_uri: self.base_uri,
            causes_overflow: false,
            container_type: None,
            display_name: self.node_name.clone().to_lowercase(),
            display_type: Some("block".into()),
            is_after_pseudo_element: false,
            is_anonymous: false,
            is_before_pseudo_element: false,
            is_direct_shadow_host_child: None,
            is_displayed: display,
            is_in_html_document: Some(true),
            is_marker_pseudo_element: false,
            is_native_anonymous: false,
            is_scrollable: false,
            is_shadow_root: false,
            is_top_level_document: self.is_top_level_document,
            node_name: self.node_name,
            node_type: self.node_type,
            node_value: None,
            num_children: self.num_children,
            parent: actors.script_to_actor(self.parent.clone()),
            shadow_root_mode: None,
            traits: HashMap::new(),
            attrs: self
                .attrs
                .into_iter()
                .map(|attr| AttrMsg {
                    name: attr.name,
                    value: attr.value,
                })
                .collect(),
        }
    }
}

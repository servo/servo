/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::net::TcpStream;

use base::id::PipelineId;
use devtools_traits::DevtoolScriptControlMsg::ModifyAttribute;
use devtools_traits::{DevtoolScriptControlMsg, NodeInfo};
use ipc_channel::ipc::IpcSender;
use serde::Serialize;
use serde_json::{self, Map, Value};

use crate::actor::{Actor, ActorMessageStatus, ActorRegistry};
use crate::protocol::JsonPacketStream;
use crate::StreamId;

#[derive(Serialize)]
struct ModifyAttributeReply {
    from: String,
}

#[derive(Clone, Serialize)]
struct AttrMsg {
    namespace: String,
    name: String,
    value: String,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NodeActorMsg {
    actor: String,
    #[serde(rename = "baseURI")]
    base_uri: String,
    parent: String,
    node_type: u16,
    #[serde(rename = "namespaceURI")]
    namespace_uri: String,
    node_name: String,
    num_children: usize,
    name: String,
    public_id: String,
    system_id: String,
    attrs: Vec<AttrMsg>,
    pseudo_class_locks: Vec<String>,
    is_displayed: bool,
    has_event_listeners: bool,
    is_document_element: bool,
    short_value: String,
    incomplete_value: bool,
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
            base_uri: self.base_uri,
            parent: actors.script_to_actor(self.parent.clone()),
            node_type: self.node_type,
            namespace_uri: self.namespace_uri,
            node_name: self.node_name,
            num_children: self.num_children,
            name: self.name,
            public_id: self.public_id,
            system_id: self.system_id,
            attrs: self
                .attrs
                .into_iter()
                .map(|attr| AttrMsg {
                    namespace: attr.namespace,
                    name: attr.name,
                    value: attr.value,
                })
                .collect(),
            pseudo_class_locks: vec![], //TODO: get this data from script
            is_displayed: display,
            has_event_listeners: false, //TODO: get this data from script
            is_document_element: self.is_document_element,
            short_value: self.short_value,
            incomplete_value: self.incomplete_value,
        }
    }
}

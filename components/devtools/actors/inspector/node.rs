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

#[derive(Serialize, Clone)]
struct AttrMsg {
    namespace: String,
    name: String,
    value: String,
}

#[derive(Serialize, Clone)]
pub struct NodeActorMsg {
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

pub struct NodeActor {
    pub name: String,
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
        let actor_name = if !actors.script_actor_registered(self.uniqueId.clone()) {
            let name = actors.new_name("node");
            let node_actor = NodeActor {
                name: name.clone(),
                script_chan,
                pipeline,
            };
            actors.register_script_actor(self.uniqueId, name.clone());
            actors.register_later(Box::new(node_actor));
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

            isDocumentElement: self.isDocumentElement,

            shortValue: self.shortValue,
            incompleteValue: self.incompleteValue,
        }
    }
}

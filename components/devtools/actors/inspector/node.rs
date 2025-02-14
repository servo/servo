/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! This actor represents one DOM node. It is created by the Walker actor when it is traversing the
//! document tree.

use std::cell::RefCell;
use std::collections::HashMap;
use std::net::TcpStream;

use base::id::PipelineId;
use devtools_traits::DevtoolScriptControlMsg::{GetChildren, GetDocumentElement, ModifyAttribute};
use devtools_traits::{DevtoolScriptControlMsg, NodeInfo, ShadowRootMode};
use ipc_channel::ipc::{self, IpcSender};
use serde::Serialize;
use serde_json::{self, Map, Value};

use crate::actor::{Actor, ActorMessageStatus, ActorRegistry};
use crate::actors::inspector::walker::WalkerActor;
use crate::protocol::JsonPacketStream;
use crate::{EmptyReplyMsg, StreamId};

/// Text node type constant. This is defined again to avoid depending on `script`, where it is defined originally.
/// See `script::dom::bindings::codegen::Bindings::NodeBinding::NodeConstants`.
const TEXT_NODE: u16 = 3;

/// The maximum length of a text node for it to appear as an inline child in the inspector.
const MAX_INLINE_LENGTH: usize = 50;

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

    /// The ID of the shadow host of this node, if it is
    /// a shadow root
    host: Option<String>,
    #[serde(rename = "baseURI")]
    base_uri: String,
    causes_overflow: bool,
    container_type: Option<()>,
    pub display_name: String,
    display_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    inline_text_child: Option<Box<NodeActorMsg>>,
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
    is_shadow_host: bool,
    is_shadow_root: bool,
    is_top_level_document: bool,
    node_name: String,
    node_type: u16,
    node_value: Option<String>,
    pub num_children: usize,
    #[serde(skip_serializing_if = "String::is_empty")]
    parent: String,
    shadow_root_mode: Option<String>,
    traits: HashMap<String, ()>,
    attrs: Vec<AttrMsg>,
}

pub struct NodeActor {
    name: String,
    pub script_chan: IpcSender<DevtoolScriptControlMsg>,
    pub pipeline: PipelineId,
    pub walker: String,
    pub style_rules: RefCell<HashMap<(String, usize), String>>,
}

impl Actor for NodeActor {
    fn name(&self) -> String {
        self.name.clone()
    }

    /// The node actor can handle the following messages:
    ///
    /// - `modifyAttributes`: Asks the script to change a value in the attribute of the
    ///   corresponding node
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
                let mods = msg.get("modifications").ok_or(())?.as_array().ok_or(())?;
                let modifications: Vec<_> = mods
                    .iter()
                    .filter_map(|json_mod| {
                        serde_json::from_str(&serde_json::to_string(json_mod).ok()?).ok()
                    })
                    .collect();

                let walker = registry.find::<WalkerActor>(&self.walker);
                walker.new_mutations(stream, &self.name, &modifications);

                self.script_chan
                    .send(ModifyAttribute(
                        self.pipeline,
                        registry.actor_to_script(self.name()),
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
                let node = doc_elem_info.encode(
                    registry,
                    true,
                    self.script_chan.clone(),
                    self.pipeline,
                    self.walker.clone(),
                );

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
        walker: String,
    ) -> NodeActorMsg;
}

impl NodeInfoToProtocol for NodeInfo {
    fn encode(
        self,
        actors: &ActorRegistry,
        display: bool,
        script_chan: IpcSender<DevtoolScriptControlMsg>,
        pipeline: PipelineId,
        walker: String,
    ) -> NodeActorMsg {
        let get_or_register_node_actor = |id: &str| {
            if !actors.script_actor_registered(id.to_string()) {
                let name = actors.new_name("node");
                actors.register_script_actor(id.to_string(), name.clone());

                let node_actor = NodeActor {
                    name: name.clone(),
                    script_chan: script_chan.clone(),
                    pipeline,
                    walker: walker.clone(),
                    style_rules: RefCell::new(HashMap::new()),
                };
                actors.register_later(Box::new(node_actor));
                name
            } else {
                actors.script_to_actor(id.to_string())
            }
        };

        let actor = get_or_register_node_actor(&self.unique_id);
        let host = self
            .host
            .as_ref()
            .map(|host_id| get_or_register_node_actor(host_id));

        let name = actors.actor_to_script(actor.clone());

        // If a node only has a single text node as a child whith a small enough text,
        // return it with this node as an `inlineTextChild`.
        let inline_text_child = (|| {
            // TODO: Also return if this node is a flex element.
            if self.num_children != 1 || self.node_name == "SLOT" {
                return None;
            }

            let (tx, rx) = ipc::channel().ok()?;
            script_chan
                .send(GetChildren(pipeline, name.clone(), tx))
                .unwrap();
            let mut children = rx.recv().ok()??;

            let child = children.pop()?;
            let msg = child.encode(actors, true, script_chan.clone(), pipeline, walker);

            // If the node child is not a text node, do not represent it inline.
            if msg.node_type != TEXT_NODE {
                return None;
            }

            // If the text node child is too big, do not represent it inline.
            if msg.node_value.clone().unwrap_or_default().len() > MAX_INLINE_LENGTH {
                return None;
            }

            Some(Box::new(msg))
        })();

        NodeActorMsg {
            actor,
            host,
            base_uri: self.base_uri,
            causes_overflow: false,
            container_type: None,
            display_name: self.node_name.clone().to_lowercase(),
            display_type: Some("block".into()),
            inline_text_child,
            is_after_pseudo_element: false,
            is_anonymous: false,
            is_before_pseudo_element: false,
            is_direct_shadow_host_child: None,
            is_displayed: display,
            is_in_html_document: Some(true),
            is_marker_pseudo_element: false,
            is_native_anonymous: false,
            is_scrollable: false,
            is_shadow_host: self.is_shadow_host,
            is_shadow_root: self.shadow_root_mode.is_some(),
            is_top_level_document: self.is_top_level_document,
            node_name: self.node_name,
            node_type: self.node_type,
            node_value: self.node_value,
            num_children: self.num_children,
            parent: actors.script_to_actor(self.parent.clone()),
            shadow_root_mode: self
                .shadow_root_mode
                .as_ref()
                .map(ShadowRootMode::to_string),
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

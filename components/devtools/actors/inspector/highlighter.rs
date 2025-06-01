/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Handles highlighting selected DOM nodes in the inspector. At the moment it only replies and
//! changes nothing on Servo's side.

use std::net::TcpStream;

use base::id::PipelineId;
use devtools_traits::DevtoolScriptControlMsg;
use ipc_channel::ipc::IpcSender;
use serde::Serialize;
use serde_json::{self, Map, Value};

use crate::actor::{Actor, ActorMessageStatus, ActorRegistry};
use crate::protocol::JsonPacketStream;
use crate::{EmptyReplyMsg, StreamId};

#[derive(Serialize)]
pub struct HighlighterMsg {
    pub actor: String,
}

pub struct HighlighterActor {
    pub name: String,
    pub script_sender: IpcSender<DevtoolScriptControlMsg>,
    pub pipeline: PipelineId,
}

#[derive(Serialize)]
struct ShowReply {
    from: String,
    value: bool,
}

impl Actor for HighlighterActor {
    fn name(&self) -> String {
        self.name.clone()
    }

    /// The highligher actor can handle the following messages:
    ///
    /// - `show`: Enables highlighting for the selected node
    ///
    /// - `hide`: Disables highlighting for the selected node
    fn handle_message(
        &self,
        registry: &ActorRegistry,
        msg_type: &str,
        msg: &Map<String, Value>,
        stream: &mut TcpStream,
        _id: StreamId,
    ) -> Result<ActorMessageStatus, ()> {
        Ok(match msg_type {
            "show" => {
                let Some(node_actor) = msg.get("node") else {
                    // TODO: send missing parameter error
                    return Ok(ActorMessageStatus::Ignored);
                };

                let Some(node_actor_name) = node_actor.as_str() else {
                    // TODO: send invalid parameter error
                    return Ok(ActorMessageStatus::Ignored);
                };

                if node_actor_name.starts_with("inspector") {
                    // TODO: For some reason, the client initially asks us to highlight
                    // the inspector? Investigate what this is supposed to mean.
                    let msg = ShowReply {
                        from: self.name(),
                        value: false,
                    };
                    let _ = stream.write_json_packet(&msg);
                    return Ok(ActorMessageStatus::Processed);
                }

                self.instruct_script_thread_to_highlight_node(
                    Some(node_actor_name.to_owned()),
                    registry,
                );
                let msg = ShowReply {
                    from: self.name(),
                    value: true,
                };
                let _ = stream.write_json_packet(&msg);
                ActorMessageStatus::Processed
            },

            "hide" => {
                self.instruct_script_thread_to_highlight_node(None, registry);

                let msg = EmptyReplyMsg { from: self.name() };
                let _ = stream.write_json_packet(&msg);
                ActorMessageStatus::Processed
            },

            _ => ActorMessageStatus::Ignored,
        })
    }
}

impl HighlighterActor {
    fn instruct_script_thread_to_highlight_node(
        &self,
        node_actor: Option<String>,
        registry: &ActorRegistry,
    ) {
        let node_id = node_actor.map(|node_actor| registry.actor_to_script(node_actor));
        self.script_sender
            .send(DevtoolScriptControlMsg::HighlightDomNode(
                self.pipeline,
                node_id,
            ))
            .unwrap();
    }
}

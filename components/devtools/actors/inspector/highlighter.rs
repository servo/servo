/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Handles highlighting selected DOM nodes in the inspector. At the moment it only replies and
//! changes nothing on Servo's side.

use devtools_traits::DevtoolScriptControlMsg;
use malloc_size_of_derive::MallocSizeOf;
use serde::Serialize;
use serde_json::{self, Map, Value};

use crate::actor::{Actor, ActorEncode, ActorError, ActorRegistry};
use crate::actors::browsing_context::BrowsingContextActor;
use crate::actors::inspector::InspectorActor;
use crate::protocol::ClientRequest;
use crate::{ActorMsg, EmptyReplyMsg, StreamId};

#[derive(MallocSizeOf)]
pub(crate) struct HighlighterActor {
    pub name: String,
    pub browsing_context_name: String,
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
    ///
    /// - `finalize`: Performs cleanup for this actor; currently a no-op
    fn handle_message(
        &self,
        request: ClientRequest,
        registry: &ActorRegistry,
        msg_type: &str,
        msg: &Map<String, Value>,
        _id: StreamId,
    ) -> Result<(), ActorError> {
        match msg_type {
            "show" => {
                let Some(node_actor) = msg.get("node") else {
                    return Err(ActorError::MissingParameter);
                };

                let Some(node_actor_name) = node_actor.as_str() else {
                    return Err(ActorError::BadParameterType);
                };

                if node_actor_name.starts_with(ActorRegistry::base_name::<InspectorActor>()) {
                    // TODO: For some reason, the client initially asks us to highlight
                    // the inspector? Investigate what this is supposed to mean.
                    let msg = ShowReply {
                        from: self.name(),
                        value: false,
                    };
                    return request.reply_final(&msg);
                }

                self.instruct_script_thread_to_highlight_node(
                    Some(node_actor_name.to_owned()),
                    registry,
                );
                let msg = ShowReply {
                    from: self.name(),
                    value: true,
                };
                request.reply_final(&msg)?
            },

            "hide" => {
                self.instruct_script_thread_to_highlight_node(None, registry);

                let msg = EmptyReplyMsg { from: self.name() };
                request.reply_final(&msg)?
            },

            "finalize" => {
                request.mark_handled();
            },

            _ => return Err(ActorError::UnrecognizedPacketType),
        };
        Ok(())
    }
}

impl HighlighterActor {
    pub fn register(registry: &ActorRegistry, browsing_context_name: String) -> String {
        let name = registry.new_name::<Self>();
        let actor = Self {
            name: name.clone(),
            browsing_context_name,
        };
        registry.register::<Self>(actor);
        name
    }

    fn instruct_script_thread_to_highlight_node(
        &self,
        node_name: Option<String>,
        registry: &ActorRegistry,
    ) {
        let node_id = node_name.map(|node_name| registry.actor_to_script(node_name));
        let browsing_context_actor =
            registry.find::<BrowsingContextActor>(&self.browsing_context_name);
        browsing_context_actor
            .script_chan()
            .send(DevtoolScriptControlMsg::HighlightDomNode(
                browsing_context_actor.pipeline_id(),
                node_id,
            ))
            .unwrap();
    }
}

impl ActorEncode<ActorMsg> for HighlighterActor {
    fn encode(&self, _: &ActorRegistry) -> ActorMsg {
        ActorMsg { actor: self.name() }
    }
}

/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! The walker actor is responsible for traversing the DOM tree in various ways to create new nodes

use std::sync::Arc;

use atomic_refcell::AtomicRefCell;
use devtools_traits::DevtoolScriptControlMsg::{
    GetChildren, GetDocumentElement, GetInnerHTML, GetOuterHTML, GetRootNode,
};
use devtools_traits::DomMutation;
use malloc_size_of_derive::MallocSizeOf;
use serde::Serialize;
use serde_json::{self, Map, Value};
use servo_base::generic_channel;

use crate::actor::{
    Actor, ActorEncode, ActorError, ActorRegistry, DowncastableActorArc, new_actor_name,
};
use crate::actors::browsing_context::BrowsingContextActor;
use crate::actors::inspector::layout::LayoutInspectorActor;
use crate::actors::inspector::node::{NodeActor, NodeActorMsg};
use crate::actors::long_string::{LongStringActor, LongStringObj};
use crate::protocol::{ClientRequest, DevtoolsConnection, JsonPacketStream};
use crate::{ActorMsg, EmptyReplyMsg, StreamId};

#[derive(Serialize)]
pub(crate) struct WalkerMsg {
    actor: String,
    root: NodeActorMsg,
}

#[derive(MallocSizeOf)]
pub(crate) struct WalkerActor {
    pub name: String,
    pub mutations: AtomicRefCell<Vec<DomMutation>>,
    /// Name of the [`BrowsingContextActor`] that owns this walker.
    pub browsing_context_name: String,
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
    from: String,
    actor: ActorMsg,
}

#[derive(Serialize)]
struct WatchRootNodeNotification {
    #[serde(rename = "type")]
    type_: String,
    from: String,
    node: NodeActorMsg,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct MutationMsg {
    #[serde(flatten)]
    variant: MutationVariant,
    #[serde(rename = "type")]
    type_: String,
    target: String,
}

#[derive(Serialize)]
#[serde(untagged)]
enum MutationVariant {
    AttributeModified {
        #[serde(rename = "attributeName")]
        attribute_name: String,
        #[serde(rename = "newValue")]
        new_value: Option<String>,
    },
}

#[derive(Serialize)]
struct GetMutationsReply {
    from: String,
    mutations: Vec<MutationMsg>,
}

#[derive(Serialize)]
struct GetOffsetParentReply {
    from: String,
    node: Option<()>,
}

#[derive(Serialize)]
struct NewMutationsNotification {
    from: String,
    #[serde(rename = "type")]
    type_: String,
}

#[derive(Serialize)]
struct GetInnerOrOuterHTMLReply {
    from: String,
    value: LongStringObj,
}

impl Actor for WalkerActor {
    fn name(&self) -> &str {
        &self.name
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
    /// - `getMutations`: Returns the list of attribute changes since it was last called
    ///
    /// - `getOffsetParent`: Placeholder
    ///
    /// - `querySelector`: Recursively looks for the specified selector in the tree, reutrning the
    ///   node and its ascendents
    ///
    /// - `outerHTML`: Return outer html element or text from specified node
    ///
    /// - `innerHTML`: Return inner html element or text from specified node
    fn handle_message(
        &self,
        mut request: ClientRequest,
        registry: &ActorRegistry,
        msg_type: &str,
        msg: &Map<String, Value>,
        _id: StreamId,
    ) -> Result<(), ActorError> {
        let browsing_context_actor = self.browsing_context_actor(registry);
        match msg_type {
            "children" => {
                let target = msg
                    .get("node")
                    .ok_or(ActorError::MissingParameter)?
                    .as_str()
                    .ok_or(ActorError::BadParameterType)?;
                let Some((tx, rx)) = generic_channel::channel() else {
                    return Err(ActorError::Internal);
                };
                browsing_context_actor
                    .script_chan()
                    .send(GetChildren(
                        browsing_context_actor.pipeline_id(),
                        registry.actor_to_script(target.into()),
                        tx,
                    ))
                    .map_err(|_| ActorError::Internal)?;
                let children = rx
                    .recv()
                    .map_err(|_| ActorError::Internal)?
                    .ok_or(ActorError::Internal)?;

                let msg = ChildrenReply {
                    has_first: true,
                    has_last: true,
                    nodes: children
                        .into_iter()
                        .map(|child| {
                            let node_actor =
                                NodeActor::register_or_update(registry, &self.name, child);
                            node_actor.encode(registry)
                        })
                        .collect(),
                    from: self.name().into(),
                };
                request.reply_final(&msg)?
            },
            "clearPseudoClassLocks" => {
                let msg = EmptyReplyMsg {
                    from: self.name().into(),
                };
                request.reply_final(&msg)?
            },
            "documentElement" => {
                let Some((tx, rx)) = generic_channel::channel() else {
                    return Err(ActorError::Internal);
                };
                browsing_context_actor
                    .script_chan()
                    .send(GetDocumentElement(browsing_context_actor.pipeline_id(), tx))
                    .map_err(|_| ActorError::Internal)?;
                let node_info = rx
                    .recv()
                    .map_err(|_| ActorError::Internal)?
                    .ok_or(ActorError::Internal)?;

                let node_actor = NodeActor::register_or_update(registry, &self.name, node_info);
                let node = node_actor.encode(registry);

                let msg = DocumentElementReply {
                    from: self.name().into(),
                    node,
                };
                request.reply_final(&msg)?
            },
            "getLayoutInspector" => {
                let layout_inspector_actor = LayoutInspectorActor::register(registry);
                let msg = GetLayoutInspectorReply {
                    from: self.name().into(),
                    actor: layout_inspector_actor.encode(registry),
                };
                request.reply_final(&msg)?
            },
            "getMutations" => self.handle_get_mutations(request, registry)?,
            "getOffsetParent" => {
                let msg = GetOffsetParentReply {
                    from: self.name().into(),
                    node: None,
                };
                request.reply_final(&msg)?
            },
            "querySelector" => {
                let selector = msg
                    .get("selector")
                    .ok_or(ActorError::MissingParameter)?
                    .as_str()
                    .ok_or(ActorError::BadParameterType)?;
                let node_name = msg
                    .get("node")
                    .ok_or(ActorError::MissingParameter)?
                    .as_str()
                    .ok_or(ActorError::BadParameterType)?;
                let mut hierarchy = find_child(&self.name, registry, node_name, vec![], |msg| {
                    msg.display_name == selector
                })
                .map_err(|_| ActorError::Internal)?;
                hierarchy.reverse();
                let node = hierarchy.pop().ok_or(ActorError::Internal)?;

                let msg = QuerySelectorReply {
                    from: self.name().into(),
                    node,
                    new_parents: hierarchy,
                };
                request.reply_final(&msg)?
            },
            "watchRootNode" => {
                let msg = WatchRootNodeNotification {
                    type_: "root-available".into(),
                    from: self.name().into(),
                    node: self.root(registry)?,
                };
                let _ = request.write_json_packet(&msg);

                let msg = EmptyReplyMsg {
                    from: self.name().into(),
                };
                request.reply_final(&msg)?
            },
            "outerHTML" => {
                let target = msg
                    .get("node")
                    .ok_or(ActorError::MissingParameter)?
                    .as_str()
                    .ok_or(ActorError::BadParameterType)?;

                let Some((tx, rx)) = generic_channel::channel() else {
                    return Err(ActorError::Internal);
                };
                browsing_context_actor
                    .script_chan()
                    .send(GetOuterHTML(
                        browsing_context_actor.pipeline_id(),
                        registry.actor_to_script(target.into()),
                        tx,
                    ))
                    .map_err(|_| ActorError::Internal)?;

                let html_text = rx
                    .recv()
                    .map_err(|_| ActorError::Internal)?
                    .ok_or(ActorError::Internal)?;

                let long_string_actor = LongStringActor::register(registry, html_text);

                let msg = GetInnerOrOuterHTMLReply {
                    from: self.name().into(),
                    value: long_string_actor.long_string_obj(),
                };
                request.reply_final(&msg)?;
            },
            "innerHTML" => {
                let target = msg
                    .get("node")
                    .ok_or(ActorError::MissingParameter)?
                    .as_str()
                    .ok_or(ActorError::BadParameterType)?;

                let Some((tx, rx)) = generic_channel::channel() else {
                    return Err(ActorError::Internal);
                };
                browsing_context_actor
                    .script_chan()
                    .send(GetInnerHTML(
                        browsing_context_actor.pipeline_id(),
                        registry.actor_to_script(target.into()),
                        tx,
                    ))
                    .map_err(|_| ActorError::Internal)?;

                let html_text = rx
                    .recv()
                    .map_err(|_| ActorError::Internal)?
                    .ok_or(ActorError::Internal)?;

                let long_string_actor = LongStringActor::register(registry, html_text);

                let msg = GetInnerOrOuterHTMLReply {
                    from: self.name().into(),
                    value: long_string_actor.long_string_obj(),
                };
                request.reply_final(&msg)?;
            },
            _ => return Err(ActorError::UnrecognizedPacketType),
        };
        Ok(())
    }
}

impl WalkerActor {
    pub fn register(registry: &ActorRegistry, browsing_context_name: String) -> Arc<Self> {
        let name = new_actor_name::<WalkerActor>();
        let actor = WalkerActor {
            name,
            mutations: AtomicRefCell::new(vec![]),
            browsing_context_name,
        };
        registry.register::<Self>(actor)
    }

    pub(crate) fn browsing_context_actor(
        &self,
        registry: &ActorRegistry,
    ) -> DowncastableActorArc<BrowsingContextActor> {
        registry.find::<BrowsingContextActor>(&self.browsing_context_name)
    }

    pub(crate) fn root(&self, registry: &ActorRegistry) -> Result<NodeActorMsg, ActorError> {
        let browsing_context_actor = self.browsing_context_actor(registry);
        let pipeline = browsing_context_actor.pipeline_id();
        let (tx, rx) = generic_channel::channel().ok_or(ActorError::Internal)?;
        browsing_context_actor
            .script_chan()
            .send(GetRootNode(pipeline, tx))
            .map_err(|_| ActorError::Internal)?;
        let node_info = rx
            .recv()
            .map_err(|_| ActorError::Internal)?
            .ok_or(ActorError::Internal)?;

        let node_actor = NodeActor::register_or_update(registry, &self.name, node_info);
        Ok(node_actor.encode(registry))
    }

    pub(crate) fn handle_dom_mutation(
        &self,
        dom_mutation: DomMutation,
        stream: &mut DevtoolsConnection,
    ) -> Result<(), ActorError> {
        let mut pending_mutations = self.mutations.borrow_mut();

        // Discard all previous modifications to that same attribute
        // which we didn't tell the devtools client about yet.
        let DomMutation::AttributeModified {
            node,
            attribute_name,
            ..
        } = &dom_mutation;
        pending_mutations.retain(|pending_mutation| match pending_mutation {
            DomMutation::AttributeModified {
                node: old_node,
                attribute_name: old_attribute_name,
                ..
            } => old_node != node || old_attribute_name != attribute_name,
        });

        pending_mutations.push(dom_mutation);

        stream.write_json_packet(&NewMutationsNotification {
            from: self.name().into(),
            type_: "newMutations".into(),
        })
    }

    /// Handle the `getMutations` message from a devtools client.
    fn handle_get_mutations(
        &self,
        request: ClientRequest,
        registry: &ActorRegistry,
    ) -> Result<(), ActorError> {
        let msg = GetMutationsReply {
            from: self.name().into(),
            mutations: self
                .mutations
                .borrow_mut()
                .drain(..)
                .map(|mutation| match mutation {
                    DomMutation::AttributeModified {
                        node,
                        attribute_name,
                        new_value,
                    } => MutationMsg {
                        variant: MutationVariant::AttributeModified {
                            attribute_name,
                            new_value,
                        },
                        target: registry.script_to_actor(&node),
                        type_: "attributes".to_owned(),
                    },
                })
                .collect(),
        };

        request.reply_final(&msg)
    }
}

/// Recursively searches for a child with the specified selector
/// If it is found, returns a list with the child and all of its ancestors.
/// TODO: Investigate how to cache this to some extent.
pub fn find_child(
    walker_name: &str,
    registry: &ActorRegistry,
    node_name: &str,
    mut hierarchy: Vec<NodeActorMsg>,
    compare_fn: impl Fn(&NodeActorMsg) -> bool + Clone,
) -> Result<Vec<NodeActorMsg>, Vec<NodeActorMsg>> {
    let walker = registry.find::<WalkerActor>(walker_name);
    let browsing_context = walker.browsing_context_actor(registry);
    let script_chan = browsing_context.script_chan();
    let pipeline = browsing_context.pipeline_id();

    let (tx, rx) = generic_channel::channel().unwrap();
    script_chan
        .send(GetChildren(
            pipeline,
            registry.actor_to_script(node_name.into()),
            tx,
        ))
        .unwrap();
    let children = rx.recv().unwrap().ok_or(vec![])?;

    for child in children {
        let node_actor = NodeActor::register_or_update(registry, walker_name, child);
        let msg = node_actor.encode(registry);

        if compare_fn(&msg) {
            hierarchy.push(msg);
            return Ok(hierarchy);
        };

        if msg.num_children == 0 {
            continue;
        }

        match find_child(
            walker_name,
            registry,
            &msg.actor,
            hierarchy,
            compare_fn.clone(),
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

impl ActorEncode<WalkerMsg> for WalkerActor {
    fn encode(&self, registry: &ActorRegistry) -> WalkerMsg {
        WalkerMsg {
            actor: self.name().into(),
            root: self.root(registry).unwrap(),
        }
    }
}

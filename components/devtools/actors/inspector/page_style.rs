/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! The page style actor is responsible of informing the DevTools client of the different style
//! properties applied, including the attributes and layout of each element.

use std::collections::HashMap;
use std::collections::hash_map::Entry;
use std::iter::once;

use base::id::PipelineId;
use devtools_traits::DevtoolScriptControlMsg::{GetLayout, GetSelectors};
use devtools_traits::{ComputedNodeLayout, DevtoolScriptControlMsg};
use ipc_channel::ipc::{self, IpcSender};
use serde::Serialize;
use serde_json::{self, Map, Value};

use crate::StreamId;
use crate::actor::{Actor, ActorError, ActorRegistry};
use crate::actors::inspector::node::NodeActor;
use crate::actors::inspector::style_rule::{AppliedRule, ComputedDeclaration, StyleRuleActor};
use crate::actors::inspector::walker::{WalkerActor, find_child};
use crate::protocol::ClientRequest;

#[derive(Serialize)]
struct GetAppliedReply {
    entries: Vec<AppliedEntry>,
    from: String,
}

#[derive(Serialize)]
struct GetComputedReply {
    computed: HashMap<String, ComputedDeclaration>,
    from: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct AppliedEntry {
    rule: AppliedRule,
    pseudo_element: Option<()>,
    is_system: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    inherited: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "kebab-case")]
struct GetLayoutReply {
    from: String,

    display: String,
    position: String,
    z_index: String,
    box_sizing: String,

    // Would be nice to use a proper struct, blocked by
    // https://github.com/serde-rs/serde/issues/43
    auto_margins: serde_json::value::Value,
    margin_top: String,
    margin_right: String,
    margin_bottom: String,
    margin_left: String,

    border_top_width: String,
    border_right_width: String,
    border_bottom_width: String,
    border_left_width: String,

    padding_top: String,
    padding_right: String,
    padding_bottom: String,
    padding_left: String,

    width: f32,
    height: f32,
}

#[derive(Serialize)]
pub struct IsPositionEditableReply {
    pub from: String,
    pub value: bool,
}

#[derive(Serialize)]
pub struct PageStyleMsg {
    pub actor: String,
    pub traits: HashMap<String, bool>,
}

pub struct PageStyleActor {
    pub name: String,
    pub script_chan: IpcSender<DevtoolScriptControlMsg>,
    pub pipeline: PipelineId,
}

impl Actor for PageStyleActor {
    fn name(&self) -> String {
        self.name.clone()
    }

    /// The page style actor can handle the following messages:
    ///
    /// - `getApplied`: Returns the applied styles for a node, they represent the explicit css
    ///   rules set for them, both in the style attribute and in stylesheets.
    ///
    /// - `getComputed`: Returns the computed styles for a node, these include all of the supported
    ///   css properties calculated values.
    ///
    /// - `getLayout`: Returns the box layout properties for a node.
    ///
    /// - `isPositionEditable`: Informs whether you can change a style property in the inspector.
    fn handle_message(
        &self,
        request: ClientRequest,
        registry: &ActorRegistry,
        msg_type: &str,
        msg: &Map<String, Value>,
        _id: StreamId,
    ) -> Result<(), ActorError> {
        match msg_type {
            "getApplied" => self.get_applied(request, msg, registry),
            "getComputed" => self.get_computed(request, msg, registry),
            "getLayout" => self.get_layout(request, msg, registry),
            "isPositionEditable" => self.is_position_editable(request),
            _ => Err(ActorError::UnrecognizedPacketType),
        }
    }
}

impl PageStyleActor {
    fn get_applied(
        &self,
        request: ClientRequest,
        msg: &Map<String, Value>,
        registry: &ActorRegistry,
    ) -> Result<(), ActorError> {
        let target = msg
            .get("node")
            .ok_or(ActorError::MissingParameter)?
            .as_str()
            .ok_or(ActorError::BadParameterType)?;
        let node = registry.find::<NodeActor>(target);
        let walker = registry.find::<WalkerActor>(&node.walker);
        let entries: Vec<_> = find_child(
            &node.script_chan,
            node.pipeline,
            target,
            registry,
            &walker.root_node.actor,
            vec![],
            |msg| msg.actor == target,
        )
        .unwrap_or_default()
        .into_iter()
        .flat_map(|node| {
            let inherited = (node.actor != target).then(|| node.actor.clone());
            let node_actor = registry.find::<NodeActor>(&node.actor);

            // Get the css selectors that match this node present in the currently active stylesheets.
            let selectors = (|| {
                let (selectors_sender, selector_receiver) = ipc::channel().ok()?;
                walker
                    .script_chan
                    .send(GetSelectors(
                        walker.pipeline,
                        registry.actor_to_script(node.actor.clone()),
                        selectors_sender,
                    ))
                    .ok()?;
                selector_receiver.recv().ok()?
            })()
            .unwrap_or_default();

            // For each selector (plus an empty one that represents the style attribute)
            // get all of the rules associated with it.
            let entries =
                once(("".into(), usize::MAX))
                    .chain(selectors)
                    .filter_map(move |selector| {
                        let rule = match node_actor.style_rules.borrow_mut().entry(selector) {
                            Entry::Vacant(e) => {
                                let name = registry.new_name("style-rule");
                                let actor = StyleRuleActor::new(
                                    name.clone(),
                                    node_actor.name(),
                                    (!e.key().0.is_empty()).then_some(e.key().clone()),
                                );
                                let rule = actor.applied(registry)?;

                                registry.register_later(Box::new(actor));
                                e.insert(name);
                                rule
                            },
                            Entry::Occupied(e) => {
                                let actor = registry.find::<StyleRuleActor>(e.get());
                                actor.applied(registry)?
                            },
                        };
                        if inherited.is_some() && rule.declarations.is_empty() {
                            return None;
                        }

                        Some(AppliedEntry {
                            rule,
                            // TODO: Handle pseudo elements
                            pseudo_element: None,
                            is_system: false,
                            inherited: inherited.clone(),
                        })
                    });
            entries
        })
        .collect();
        let msg = GetAppliedReply {
            entries,
            from: self.name(),
        };
        request.reply_final(&msg)
    }

    fn get_computed(
        &self,
        request: ClientRequest,
        msg: &Map<String, Value>,
        registry: &ActorRegistry,
    ) -> Result<(), ActorError> {
        let target = msg
            .get("node")
            .ok_or(ActorError::MissingParameter)?
            .as_str()
            .ok_or(ActorError::BadParameterType)?;
        let node_actor = registry.find::<NodeActor>(target);
        let computed = (|| match node_actor
            .style_rules
            .borrow_mut()
            .entry(("".into(), usize::MAX))
        {
            Entry::Vacant(e) => {
                let name = registry.new_name("style-rule");
                let actor = StyleRuleActor::new(name.clone(), target.into(), None);
                let computed = actor.computed(registry)?;
                registry.register_later(Box::new(actor));
                e.insert(name);
                Some(computed)
            },
            Entry::Occupied(e) => {
                let actor = registry.find::<StyleRuleActor>(e.get());
                Some(actor.computed(registry)?)
            },
        })()
        .unwrap_or_default();
        let msg = GetComputedReply {
            computed,
            from: self.name(),
        };
        request.reply_final(&msg)
    }

    fn get_layout(
        &self,
        request: ClientRequest,
        msg: &Map<String, Value>,
        registry: &ActorRegistry,
    ) -> Result<(), ActorError> {
        let target = msg
            .get("node")
            .ok_or(ActorError::MissingParameter)?
            .as_str()
            .ok_or(ActorError::BadParameterType)?;
        let (computed_node_sender, computed_node_receiver) =
            ipc::channel().map_err(|_| ActorError::Internal)?;
        self.script_chan
            .send(GetLayout(
                self.pipeline,
                registry.actor_to_script(target.to_owned()),
                computed_node_sender,
            ))
            .unwrap();
        let ComputedNodeLayout {
            display,
            position,
            z_index,
            box_sizing,
            auto_margins,
            margin_top,
            margin_right,
            margin_bottom,
            margin_left,
            border_top_width,
            border_right_width,
            border_bottom_width,
            border_left_width,
            padding_top,
            padding_right,
            padding_bottom,
            padding_left,
            width,
            height,
        } = computed_node_receiver
            .recv()
            .map_err(|_| ActorError::Internal)?
            .ok_or(ActorError::Internal)?;
        let msg_auto_margins = msg
            .get("autoMargins")
            .and_then(Value::as_bool)
            .unwrap_or(false);
        let msg = GetLayoutReply {
            from: self.name(),
            display,
            position,
            z_index,
            box_sizing,
            auto_margins: if msg_auto_margins {
                let mut m = Map::new();
                let auto = serde_json::value::Value::String("auto".to_owned());
                if auto_margins.top {
                    m.insert("top".to_owned(), auto.clone());
                }
                if auto_margins.right {
                    m.insert("right".to_owned(), auto.clone());
                }
                if auto_margins.bottom {
                    m.insert("bottom".to_owned(), auto.clone());
                }
                if auto_margins.left {
                    m.insert("left".to_owned(), auto);
                }
                serde_json::value::Value::Object(m)
            } else {
                serde_json::value::Value::Null
            },
            margin_top,
            margin_right,
            margin_bottom,
            margin_left,
            border_top_width,
            border_right_width,
            border_bottom_width,
            border_left_width,
            padding_top,
            padding_right,
            padding_bottom,
            padding_left,
            width,
            height,
        };
        let msg = serde_json::to_string(&msg).map_err(|_| ActorError::Internal)?;
        let msg = serde_json::from_str::<Value>(&msg).map_err(|_| ActorError::Internal)?;
        request.reply_final(&msg)
    }

    fn is_position_editable(&self, request: ClientRequest) -> Result<(), ActorError> {
        let msg = IsPositionEditableReply {
            from: self.name(),
            value: false,
        };
        request.reply_final(&msg)
    }
}

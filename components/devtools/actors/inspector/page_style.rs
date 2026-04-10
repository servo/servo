/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! The page style actor is responsible of informing the DevTools client of the different style
//! properties applied, including the attributes and layout of each element.

use std::collections::HashMap;
use std::iter::once;

use devtools_traits::DevtoolScriptControlMsg::{GetLayout, GetSelectors};
use devtools_traits::{AutoMargins, ComputedNodeLayout, MatchedRule};
use malloc_size_of_derive::MallocSizeOf;
use serde::Serialize;
use serde_json::{self, Map, Value};
use servo_base::generic_channel::{self};

use crate::StreamId;
use crate::actor::{Actor, ActorEncode, ActorError, ActorRegistry};
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
struct DevtoolsAutoMargins {
    #[serde(skip_serializing_if = "Option::is_none")]
    top: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    right: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    bottom: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    left: Option<String>,
}

impl From<AutoMargins> for DevtoolsAutoMargins {
    fn from(auto_margins: AutoMargins) -> Self {
        const AUTO: &str = "auto";
        Self {
            top: auto_margins.top.then_some(AUTO.into()),
            right: auto_margins.right.then_some(AUTO.into()),
            bottom: auto_margins.bottom.then_some(AUTO.into()),
            left: auto_margins.left.then_some(AUTO.into()),
        }
    }
}

#[derive(Serialize)]
struct GetLayoutReply {
    from: String,
    #[serde(flatten)]
    layout: ComputedNodeLayout,
    #[serde(rename = "autoMargins")]
    auto_margins: DevtoolsAutoMargins,
}

#[derive(Serialize)]
pub(crate) struct IsPositionEditableReply {
    from: String,
    value: bool,
}

#[derive(Serialize)]
pub(crate) struct PageStyleMsg {
    pub actor: String,
    pub traits: HashMap<String, bool>,
}

#[derive(MallocSizeOf)]
pub(crate) struct PageStyleActor {
    pub name: String,
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
    pub fn register(registry: &ActorRegistry) -> String {
        let name = registry.new_name::<Self>();
        let actor = Self { name: name.clone() };
        registry.register::<Self>(actor);
        name
    }

    fn get_applied(
        &self,
        request: ClientRequest,
        msg: &Map<String, Value>,
        registry: &ActorRegistry,
    ) -> Result<(), ActorError> {
        let node_name = msg
            .get("node")
            .ok_or(ActorError::MissingParameter)?
            .as_str()
            .ok_or(ActorError::BadParameterType)?;
        let node_actor = registry.find::<NodeActor>(node_name);
        let walker = registry.find::<WalkerActor>(&node_actor.walker);
        let browsing_context_actor = walker.browsing_context_actor(registry);
        let entries: Vec<_> = find_child(
            &node_actor.script_chan,
            node_actor.pipeline,
            node_name,
            registry,
            &walker.root(registry)?.actor,
            vec![],
            |msg| msg.actor == node_name,
        )
        .unwrap_or_default()
        .into_iter()
        .flat_map(|node| {
            let inherited = (node.actor != node_name).then(|| node.actor.clone());
            let node_actor = registry.find::<NodeActor>(&node.actor);

            // Get the css selectors that match this node present in the currently active stylesheets.
            let selectors = (|| {
                let (selectors_sender, selector_receiver) = generic_channel::channel()?;
                browsing_context_actor
                    .script_chan()
                    .send(GetSelectors(
                        browsing_context_actor.pipeline_id(),
                        registry.actor_to_script(node.actor.clone()),
                        selectors_sender,
                    ))
                    .ok()?;
                selector_receiver.recv().ok()?
            })()
            .unwrap_or_default();

            // For each selector (plus an empty one that represents the style attribute)
            // get all of the rules associated with it.

            let style_attribute_rule = MatchedRule {
                selector: "".into(),
                stylesheet_index: usize::MAX,
                block_id: 0,
                ancestor_data: vec![],
            };

            once(style_attribute_rule)
                .chain(selectors)
                .filter_map(move |matched_rule| {
                    let style_rule_name = node_actor
                        .style_rules
                        .borrow_mut()
                        .entry(matched_rule.clone())
                        .or_insert_with(|| {
                            StyleRuleActor::register(
                                registry,
                                node_actor.name(),
                                (matched_rule.stylesheet_index != usize::MAX)
                                    .then_some(matched_rule.clone()),
                            )
                        })
                        .clone();

                    let rule = registry
                        .find::<StyleRuleActor>(&style_rule_name)
                        .applied(registry)?;
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
                })
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
        let node_name = msg
            .get("node")
            .ok_or(ActorError::MissingParameter)?
            .as_str()
            .ok_or(ActorError::BadParameterType)?;
        let node_actor = registry.find::<NodeActor>(node_name);
        let style_attribute_rule = devtools_traits::MatchedRule {
            selector: "".into(),
            stylesheet_index: usize::MAX,
            block_id: 0,
            ancestor_data: vec![],
        };

        let style_rule_name = node_actor
            .style_rules
            .borrow_mut()
            .entry(style_attribute_rule)
            .or_insert_with(|| StyleRuleActor::register(registry, node_name.into(), None))
            .clone();
        let computed = registry
            .find::<StyleRuleActor>(&style_rule_name)
            .computed(registry)
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
        let node_name = msg
            .get("node")
            .ok_or(ActorError::MissingParameter)?
            .as_str()
            .ok_or(ActorError::BadParameterType)?;
        let node_actor = registry.find::<NodeActor>(node_name);
        let walker = registry.find::<WalkerActor>(&node_actor.walker);
        let browsing_context_actor = walker.browsing_context_actor(registry);
        let (tx, rx) = generic_channel::channel().ok_or(ActorError::Internal)?;
        browsing_context_actor
            .script_chan()
            .send(GetLayout(
                browsing_context_actor.pipeline_id(),
                registry.actor_to_script(node_name.to_owned()),
                tx,
            ))
            .map_err(|_| ActorError::Internal)?;
        let (layout, auto_margins) = rx
            .recv()
            .map_err(|_| ActorError::Internal)?
            .ok_or(ActorError::Internal)?;
        request.reply_final(&GetLayoutReply {
            from: self.name(),
            layout,
            auto_margins: auto_margins.into(),
        })
    }

    fn is_position_editable(&self, request: ClientRequest) -> Result<(), ActorError> {
        let msg = IsPositionEditableReply {
            from: self.name(),
            value: false,
        };
        request.reply_final(&msg)
    }
}

impl ActorEncode<PageStyleMsg> for PageStyleActor {
    fn encode(&self, _: &ActorRegistry) -> PageStyleMsg {
        PageStyleMsg {
            actor: self.name(),
            traits: HashMap::from([
                ("fontStretchLevel4".into(), true),
                ("fontStyleLevel4".into(), true),
                ("fontVariations".into(), true),
                ("fontWeightLevel4".into(), true),
            ]),
        }
    }
}

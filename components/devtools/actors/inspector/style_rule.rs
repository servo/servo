/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Liberally derived from <https://searchfox.org/mozilla-central/source/devtools/server/actors/thread-configuration.js>
//! This actor represents one css rule group from a node, allowing the inspector to view it and change it.
//! A group is either the html style attribute or one selector from one stylesheet.

use std::collections::HashMap;

use devtools_traits::DevtoolScriptControlMsg::{
    GetAttributeStyle, GetComputedStyle, GetDocumentElement, GetStylesheetStyle, ModifyRule,
};
use ipc_channel::ipc;
use serde::Serialize;
use serde_json::{Map, Value};

use crate::StreamId;
use crate::actor::{Actor, ActorError, ActorRegistry};
use crate::actors::inspector::node::NodeActor;
use crate::actors::inspector::walker::WalkerActor;
use crate::protocol::ClientRequest;

const ELEMENT_STYLE_TYPE: u32 = 100;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppliedRule {
    actor: String,
    ancestor_data: Vec<()>,
    authored_text: String,
    css_text: String,
    pub declarations: Vec<AppliedDeclaration>,
    href: String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    selectors: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    selectors_specificity: Vec<u32>,
    #[serde(rename = "type")]
    type_: u32,
    traits: StyleRuleActorTraits,
}

#[derive(Serialize)]
pub struct IsUsed {
    pub used: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppliedDeclaration {
    colon_offsets: Vec<i32>,
    is_name_valid: bool,
    is_used: IsUsed,
    is_valid: bool,
    name: String,
    offsets: Vec<i32>,
    priority: String,
    terminator: String,
    value: String,
}

#[derive(Serialize)]
pub struct ComputedDeclaration {
    matched: bool,
    value: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StyleRuleActorTraits {
    pub can_set_rule_text: bool,
}

#[derive(Serialize)]
pub struct StyleRuleActorMsg {
    from: String,
    rule: Option<AppliedRule>,
}

pub struct StyleRuleActor {
    name: String,
    node: String,
    selector: Option<(String, usize)>,
}

impl Actor for StyleRuleActor {
    fn name(&self) -> String {
        self.name.clone()
    }

    /// The style rule configuration actor can handle the following messages:
    ///
    /// - `setRuleText`: Applies a set of modifications to the css rules that this actor manages.
    ///   There is also `modifyProperties`, which has a slightly different API to do the same, but
    ///   this is preferred. Which one the devtools client sends is decided by the `traits` defined
    ///   when returning the list of rules.
    fn handle_message(
        &self,
        request: ClientRequest,
        registry: &ActorRegistry,
        msg_type: &str,
        msg: &Map<String, Value>,
        _id: StreamId,
    ) -> Result<(), ActorError> {
        match msg_type {
            "setRuleText" => {
                // Parse the modifications sent from the client
                let mods = msg
                    .get("modifications")
                    .ok_or(ActorError::MissingParameter)?
                    .as_array()
                    .ok_or(ActorError::BadParameterType)?;
                let modifications: Vec<_> = mods
                    .iter()
                    .filter_map(|json_mod| {
                        serde_json::from_str(&serde_json::to_string(json_mod).ok()?).ok()
                    })
                    .collect();

                // Query the rule modification
                let node = registry.find::<NodeActor>(&self.node);
                let walker = registry.find::<WalkerActor>(&node.walker);
                walker
                    .script_chan
                    .send(ModifyRule(
                        walker.pipeline,
                        registry.actor_to_script(self.node.clone()),
                        modifications,
                    ))
                    .map_err(|_| ActorError::Internal)?;

                request.reply_final(&self.encodable(registry))?
            },
            _ => return Err(ActorError::UnrecognizedPacketType),
        };
        Ok(())
    }
}

impl StyleRuleActor {
    pub fn new(name: String, node: String, selector: Option<(String, usize)>) -> Self {
        Self {
            name,
            node,
            selector,
        }
    }

    pub fn applied(&self, registry: &ActorRegistry) -> Option<AppliedRule> {
        let node = registry.find::<NodeActor>(&self.node);
        let walker = registry.find::<WalkerActor>(&node.walker);

        let (document_sender, document_receiver) = ipc::channel().ok()?;
        walker
            .script_chan
            .send(GetDocumentElement(walker.pipeline, document_sender))
            .ok()?;
        let node = document_receiver.recv().ok()??;

        // Gets the style definitions. If there is a selector, query the relevant stylesheet, if
        // not, this represents the style attribute.
        let (style_sender, style_receiver) = ipc::channel().ok()?;
        let req = match &self.selector {
            Some(selector) => {
                let (selector, stylesheet) = selector.clone();
                GetStylesheetStyle(
                    walker.pipeline,
                    registry.actor_to_script(self.node.clone()),
                    selector,
                    stylesheet,
                    style_sender,
                )
            },
            None => GetAttributeStyle(
                walker.pipeline,
                registry.actor_to_script(self.node.clone()),
                style_sender,
            ),
        };
        walker.script_chan.send(req).ok()?;
        let style = style_receiver.recv().ok()??;

        Some(AppliedRule {
            actor: self.name(),
            ancestor_data: vec![], // TODO: Fill with hierarchy
            authored_text: "".into(),
            css_text: "".into(), // TODO: Specify the css text
            declarations: style
                .into_iter()
                .map(|decl| {
                    AppliedDeclaration {
                        colon_offsets: vec![],
                        is_name_valid: true,
                        is_used: IsUsed { used: true },
                        is_valid: true,
                        name: decl.name,
                        offsets: vec![], // TODO: Get the source of the declaration
                        priority: decl.priority,
                        terminator: "".into(),
                        value: decl.value,
                    }
                })
                .collect(),
            href: node.base_uri.clone(),
            selectors: self.selector.iter().map(|(s, _)| s).cloned().collect(),
            selectors_specificity: self.selector.iter().map(|_| 1).collect(),
            type_: ELEMENT_STYLE_TYPE,
            traits: StyleRuleActorTraits {
                can_set_rule_text: true,
            },
        })
    }

    pub fn computed(
        &self,
        registry: &ActorRegistry,
    ) -> Option<HashMap<String, ComputedDeclaration>> {
        let node = registry.find::<NodeActor>(&self.node);
        let walker = registry.find::<WalkerActor>(&node.walker);

        let (style_sender, style_receiver) = ipc::channel().ok()?;
        walker
            .script_chan
            .send(GetComputedStyle(
                walker.pipeline,
                registry.actor_to_script(self.node.clone()),
                style_sender,
            ))
            .ok()?;
        let style = style_receiver.recv().ok()??;

        Some(
            style
                .into_iter()
                .map(|s| {
                    (
                        s.name,
                        ComputedDeclaration {
                            matched: true,
                            value: s.value,
                        },
                    )
                })
                .collect(),
        )
    }

    pub fn encodable(&self, registry: &ActorRegistry) -> StyleRuleActorMsg {
        StyleRuleActorMsg {
            from: self.name(),
            rule: self.applied(registry),
        }
    }
}

/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Liberally derived from <https://searchfox.org/mozilla-central/source/devtools/server/actors/thread-configuration.js>
//! This actor represents one css rule from a node, allowing the inspector to view it and change it.

use std::net::TcpStream;

use devtools_traits::DevtoolScriptControlMsg::{GetAppliedStyle, GetDocumentElement, ModifyRule};
use ipc_channel::ipc;
use serde::Serialize;
use serde_json::{Map, Value};

use crate::actor::{Actor, ActorMessageStatus, ActorRegistry};
use crate::actors::inspector::node::NodeActor;
use crate::actors::inspector::walker::WalkerActor;
use crate::protocol::JsonPacketStream;
use crate::StreamId;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppliedRule {
    actor: String,
    ancestor_data: Vec<()>,
    authored_text: String,
    css_text: String,
    declarations: Vec<AppliedDeclaration>,
    href: String,
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
}

impl Actor for StyleRuleActor {
    fn name(&self) -> String {
        self.name.clone()
    }

    /// The style rule configuration actor can handle the following messages:
    ///
    /// -
    fn handle_message(
        &self,
        registry: &ActorRegistry,
        msg_type: &str,
        msg: &Map<String, Value>,
        stream: &mut TcpStream,
        _id: StreamId,
    ) -> Result<ActorMessageStatus, ()> {
        Ok(match msg_type {
            "modifyProperties" | "setRuleText" => {
                let mods = msg.get("modifications").ok_or(())?.as_array().ok_or(())?;
                // TODO: Check this
                let modifications: Vec<_> = mods
                    .iter()
                    .filter_map(|json_mod| {
                        serde_json::from_str(&serde_json::to_string(json_mod).ok()?).ok()
                    })
                    .collect();

                log::info!("{:?}", modifications);

                let node = registry.find::<NodeActor>(&self.node);
                let walker = registry.find::<WalkerActor>(&node.walker);
                // TODO: Is this necessary?
                // walker.new_mutations(stream, &self.node, &modifications);

                walker
                    .script_chan
                    .send(ModifyRule(
                        walker.pipeline,
                        registry.actor_to_script(self.node.clone()),
                        modifications,
                    ))
                    .map_err(|_| ())?;

                let _ = stream.write_json_packet(&self.encodable(registry));
                ActorMessageStatus::Processed
            },
            _ => ActorMessageStatus::Ignored,
        })
    }
}

impl StyleRuleActor {
    pub fn new(name: String, node: String) -> Self {
        Self { name, node }
    }

    pub fn rule(&self, registry: &ActorRegistry) -> Option<AppliedRule> {
        let node = registry.find::<NodeActor>(&self.node);
        let walker = registry.find::<WalkerActor>(&node.walker);

        let (tx, rx) = ipc::channel().ok()?;
        walker
            .script_chan
            .send(GetDocumentElement(walker.pipeline, tx))
            .unwrap();
        let node = rx.recv().ok()??;

        let (tx, rx) = ipc::channel().ok()?;
        walker
            .script_chan
            .send(GetAppliedStyle(
                walker.pipeline,
                registry.actor_to_script(self.node.clone()),
                tx,
            ))
            .unwrap();
        let style = rx.recv().ok()??;

        // TODO: Fill with real values
        Some(AppliedRule {
            actor: self.name(),
            ancestor_data: vec![],
            authored_text: "TODO".into(),
            css_text: "TODO".into(),
            declarations: style
                .into_iter()
                .filter_map(|decl| {
                    Some(AppliedDeclaration {
                        colon_offsets: vec![],
                        is_name_valid: true,
                        is_used: IsUsed { used: true },
                        is_valid: true,
                        name: decl.name,
                        offsets: vec![],
                        priority: decl.priority,
                        terminator: "".into(),
                        value: decl.value,
                    })
                })
                .collect(),
            href: node.base_uri.clone(),
            type_: 100, // Element style type, extract to constant
            traits: StyleRuleActorTraits {
                can_set_rule_text: true,
            },
        })
    }

    pub fn encodable(&self, registry: &ActorRegistry) -> StyleRuleActorMsg {
        StyleRuleActorMsg {
            from: self.name(),
            rule: self.rule(registry),
        }
    }
}

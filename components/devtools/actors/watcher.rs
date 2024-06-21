/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::collections::HashMap;
use std::net::TcpStream;

use serde::Serialize;
use serde_json::{Map, Value};

use crate::actor::{Actor, ActorMessageStatus, ActorRegistry};
use crate::actors::browsing_context::{BrowsingContextActor, BrowsingContextActorMsg};
use crate::actors::configuration::{
    TargetConfigurationActor, TargetConfigurationActorMsg, ThreadConfigurationActor,
    ThreadConfigurationActorMsg,
};
use crate::protocol::JsonPacketStream;
use crate::{EmptyReplyMsg, StreamId};

#[derive(Serialize)]
pub enum SessionContextType {
    BrowserElement,
    _ContextProcess,
    _WebExtension,
    _Worker,
    _All,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionContext {
    is_server_target_switching_enabled: bool,
    supported_targets: HashMap<&'static str, bool>,
    supported_resources: HashMap<&'static str, bool>,
    context_type: SessionContextType,
}

impl SessionContext {
    pub fn new(context_type: SessionContextType) -> Self {
        Self {
            is_server_target_switching_enabled: false,
            supported_targets: HashMap::from([
                ("frame", true),
                ("process", false),
                ("worker", false),
                ("service_worker", false),
                ("shared_worker", false),
            ]),
            supported_resources: HashMap::from([
                ("console-message", false),
                ("css-change", false),
                ("css-message", false),
                ("css-registered-properties", false),
                ("document-event", true),
                ("Cache", false),
                ("cookies", false),
                ("error-message", false),
                ("extension-storage", false),
                ("indexed-db", false),
                ("local-storage", false),
                ("session-storage", false),
                ("platform-message", false),
                ("network-event", false),
                ("network-event-stacktrace", false),
                ("reflow", false),
                ("stylesheet", false),
                ("source", false),
                ("thread-state", false),
                ("server-sent-event", false),
                ("websocket", false),
                ("jstracer-trace", false),
                ("jstracer-state", false),
                ("last-private-context-exit", false),
            ]),
            context_type,
        }
    }
}

#[derive(Serialize)]
struct WatchTargetsReply {
    from: String,
    #[serde(rename = "type")]
    type_: String,
    target: BrowsingContextActorMsg,
}

#[derive(Serialize)]
struct GetTargetConfigurationActorReply {
    from: String,
    configuration: TargetConfigurationActorMsg,
}

#[derive(Serialize)]
struct GetThreadConfigurationActorReply {
    from: String,
    configuration: ThreadConfigurationActorMsg,
}

#[derive(Serialize)]
struct WatcherTraits {
    resources: HashMap<&'static str, bool>,
    #[serde(flatten)]
    targets: HashMap<&'static str, bool>,
}

#[derive(Serialize)]
pub struct WatcherActorMsg {
    actor: String,
    traits: WatcherTraits,
}

pub struct WatcherActor {
    name: String,
    browsing_context_actor: String,
    session_context: SessionContext,
}

impl Actor for WatcherActor {
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
            "watchTargets" => {
                let target = registry
                    .find::<BrowsingContextActor>(&self.browsing_context_actor)
                    .encodable();
                let _ = stream.write_json_packet(&WatchTargetsReply {
                    from: self.name(),
                    type_: "target-available-form".into(),
                    target,
                });

                let target = registry.find::<BrowsingContextActor>(&self.browsing_context_actor);
                target.frame_update(stream);

                // TODO: Document this type of reply after sending event messages
                let _ = stream.write_json_packet(&EmptyReplyMsg { from: self.name() });

                ActorMessageStatus::Processed
            },
            "watchResources" => {
                let Some(resource_types) = msg.get("resourceTypes") else {
                    return Ok(ActorMessageStatus::Ignored);
                };

                let Some(resource_types) = resource_types.as_array() else {
                    return Ok(ActorMessageStatus::Ignored);
                };

                for resource in resource_types {
                    let Some(resource) = resource.as_str() else {
                        continue;
                    };

                    let target =
                        registry.find::<BrowsingContextActor>(&self.browsing_context_actor);

                    match resource {
                        "document-event" => {
                            target.document_event(stream);
                        },
                        _ => {},
                    }

                    let _ = stream.write_json_packet(&EmptyReplyMsg { from: self.name() });
                }

                ActorMessageStatus::Processed
            },
            "getTargetConfigurationActor" => {
                let target = registry.find::<BrowsingContextActor>(&self.browsing_context_actor);
                let target_configuration =
                    registry.find::<TargetConfigurationActor>(&target.target_configuration);

                let _ = stream.write_json_packet(&GetTargetConfigurationActorReply {
                    from: self.name(),
                    configuration: target_configuration.encodable(),
                });

                ActorMessageStatus::Processed
            },
            "getThreadConfigurationActor" => {
                let target = registry.find::<BrowsingContextActor>(&self.browsing_context_actor);
                let thread_configuration =
                    registry.find::<ThreadConfigurationActor>(&target.thread_configuration);

                let _ = stream.write_json_packet(&GetThreadConfigurationActorReply {
                    from: self.name(),
                    configuration: thread_configuration.encodable(),
                });

                ActorMessageStatus::Processed
            },
            _ => ActorMessageStatus::Ignored,
        })
    }
}

impl WatcherActor {
    pub fn new(
        name: String,
        browsing_context_actor: String,
        session_context: SessionContext,
    ) -> Self {
        Self {
            name,
            browsing_context_actor,
            session_context,
        }
    }

    pub fn encodable(&self) -> WatcherActorMsg {
        WatcherActorMsg {
            actor: self.name(),
            traits: WatcherTraits {
                resources: self.session_context.supported_resources.clone(),
                targets: self.session_context.supported_targets.clone(),
            },
        }
    }
}

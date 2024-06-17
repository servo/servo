/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::collections::HashMap;
use std::net::TcpStream;

use serde::Serialize;
use serde_json::{Map, Value};

use crate::actor::{Actor, ActorMessageStatus, ActorRegistry};
use crate::actors::browsing_context::{BrowsingContextActor, BrowsingContextActorMsg};
use crate::protocol::JsonPacketStream;
use crate::StreamId;

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

// TODO: This is not actually going through
// It hangs indefinitely and I get this error:
// Exception while opening the toolbox Error: Connection closed, pending request to watcher8, type watchTargets failed
// I suspect it has to do with the thread actor that I am now passing
#[derive(Serialize)]
struct WatchTargetsReply {
    from: String,
    #[serde(rename = "type")]
    type_: String,
    target: BrowsingContextActorMsg,
}

#[derive(Serialize)]
struct WatchResourcesReply {
    from: String,
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

                    // Let the browsing context reply with the resource availability status
                    let target =
                        registry.find::<BrowsingContextActor>(&self.browsing_context_actor);
                    target.resource_available(resource, stream);

                    // This just responds with and acknowledgement
                    let _ = stream.write_json_packet(&WatchResourcesReply { from: self.name() });
                }

                ActorMessageStatus::Processed
            },
            _ => ActorMessageStatus::Ignored,
        })
    }
}

// TODO: Improve the JSON response builder (with from, different forms, etc...)

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

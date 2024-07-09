/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Liberally derived from the [Firefox JS implementation]
//! (https://searchfox.org/mozilla-central/source/devtools/server/actors/watcher.js).
//! The watcher is the main entry point when debugging an element. Right now only web views are supported.
//! It talks to the devtools remote and lists the capabilities of the inspected target, and it serves
//! as a bridge for messages between actors.

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

/// Describes the debugged context. It informs the server of which objects can be debugged.
/// <https://searchfox.org/mozilla-central/source/devtools/server/actors/watcher/session-context.js>
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
            // Right now we only support debugging web views (frames)
            supported_targets: HashMap::from([
                ("frame", true),
                ("process", false),
                ("worker", false),
                ("service_worker", false),
                ("shared_worker", false),
            ]),
            // At the moment we are blocking most resources to avoid errors
            // Support for them will be enabled gradually once the corresponding actors start
            // working propperly
            supported_resources: HashMap::from([
                ("console-message", true),
                ("css-change", false),
                ("css-message", false),
                ("css-registered-properties", false),
                ("document-event", true),
                ("Cache", false),
                ("cookies", false),
                ("error-message", true),
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
pub enum SessionContextType {
    BrowserElement,
    _ContextProcess,
    _WebExtension,
    _Worker,
    _All,
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

    /// The watcher actor can handle the following messages:
    ///
    /// - `watchTargets`: Returns a list of objects to debug. Since we only support web views, it
    /// returns the associated `BrowsingContextActor`. Every target sent creates a
    /// `target-available-form` event.
    ///
    /// - `watchResources`: Start watching certain resource types. This sends
    /// `resource-available-form` events.
    ///
    /// - `getTargetConfigurationActor`: Returns the configuration actor for a specific target, so
    /// that the server can update its settings.
    ///
    /// - `getThreadConfigurationActor`: The same but with the configuration actor for the thread
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

                // Messages that contain a `type` field are used to send event callbacks, but they
                // don't count as a reply. Since every message needs to be responded, we send an
                // extra empty packet to the devtools host to inform that we successfully received
                // and processed the message so that it can continue
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

                    if resource == "document-event" {
                        target.document_event(stream);
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

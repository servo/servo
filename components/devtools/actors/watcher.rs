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

use log::warn;
use serde::Serialize;
use serde_json::{Map, Value};

use self::network_parent::{NetworkParentActor, NetworkParentActorMsg};
use crate::actor::{Actor, ActorMessageStatus, ActorRegistry};
use crate::actors::browsing_context::{BrowsingContextActor, BrowsingContextActorMsg};
use crate::actors::watcher::target_configuration::{
    TargetConfigurationActor, TargetConfigurationActorMsg,
};
use crate::actors::watcher::thread_configuration::{
    ThreadConfigurationActor, ThreadConfigurationActorMsg,
};
use crate::protocol::JsonPacketStream;
use crate::{EmptyReplyMsg, StreamId};

pub mod network_parent;
pub mod target_configuration;
pub mod thread_configuration;

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
                ("document-event", false),
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
struct GetParentBrowsingContextIDReply {
    from: String,
    #[serde(rename = "browsingContextID")]
    browsing_context_id: u32,
}

#[derive(Serialize)]
struct GetNetworkParentActorReply {
    from: String,
    network: NetworkParentActorMsg,
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
    network_parent: String,
    target_configuration: String,
    thread_configuration: String,
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
    /// - `getNetworkParentActor`: Returns the network parent actor. It doesn't seem to do much at
    /// the moment.
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
        let target = registry.find::<BrowsingContextActor>(&self.browsing_context_actor);
        Ok(match msg_type {
            "watchTargets" => {
                let msg = WatchTargetsReply {
                    from: self.name(),
                    type_: "target-available-form".into(),
                    target: target.encodable(),
                };
                let _ = stream.write_json_packet(&msg);

                target.frame_update(stream);

                // Messages that contain a `type` field are used to send event callbacks, but they
                // don't count as a reply. Since every message needs to be responded, we send an
                // extra empty packet to the devtools host to inform that we successfully received
                // and processed the message so that it can continue
                let msg = EmptyReplyMsg { from: self.name() };
                let _ = stream.write_json_packet(&msg);
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
                    match resource {
                        "document-event" => {
                            target.document_event(stream);
                        },
                        "console-message" | "error-message" => {},
                        _ => warn!("resource {} not handled yet", resource),
                    }

                    let msg = EmptyReplyMsg { from: self.name() };
                    let _ = stream.write_json_packet(&msg);
                }
                ActorMessageStatus::Processed
            },
            "getParentBrowsingContextID" => {
                let browsing_context_id = target.browsing_context_id.index.0.get();
                let msg = GetParentBrowsingContextIDReply {
                    from: self.name(),
                    browsing_context_id,
                };
                let _ = stream.write_json_packet(&msg);
                ActorMessageStatus::Processed
            },
            "getNetworkParentActor" => {
                let network_parent = registry.find::<NetworkParentActor>(&self.network_parent);
                let msg = GetNetworkParentActorReply {
                    from: self.name(),
                    network: network_parent.encodable(),
                };
                let _ = stream.write_json_packet(&msg);
                ActorMessageStatus::Processed
            },
            "getTargetConfigurationActor" => {
                let target_configuration =
                    registry.find::<TargetConfigurationActor>(&self.target_configuration);
                let msg = GetTargetConfigurationActorReply {
                    from: self.name(),
                    configuration: target_configuration.encodable(),
                };
                let _ = stream.write_json_packet(&msg);
                ActorMessageStatus::Processed
            },
            "getThreadConfigurationActor" => {
                let thread_configuration =
                    registry.find::<ThreadConfigurationActor>(&self.thread_configuration);
                let msg = GetThreadConfigurationActorReply {
                    from: self.name(),
                    configuration: thread_configuration.encodable(),
                };
                let _ = stream.write_json_packet(&msg);
                ActorMessageStatus::Processed
            },
            _ => ActorMessageStatus::Ignored,
        })
    }
}

impl WatcherActor {
    pub fn new(
        actors: &mut ActorRegistry,
        browsing_context_actor: String,
        session_context: SessionContext,
    ) -> Self {
        let network_parent = NetworkParentActor::new(actors.new_name("network-parent"));
        let target_configuration =
            TargetConfigurationActor::new(actors.new_name("target-configuration"));
        let thread_configuration =
            ThreadConfigurationActor::new(actors.new_name("thread-configuration"));

        let watcher = Self {
            name: actors.new_name("watcher"),
            browsing_context_actor,
            network_parent: network_parent.name(),
            target_configuration: target_configuration.name(),
            thread_configuration: thread_configuration.name(),
            session_context,
        };

        actors.register(Box::new(network_parent));
        actors.register(Box::new(target_configuration));
        actors.register(Box::new(thread_configuration));

        watcher
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

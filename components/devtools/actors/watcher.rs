/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! The watcher is the main entry point when debugging an element. Right now only web views are supported.
//! It talks to the devtools remote and lists the capabilities of the inspected target, and it serves
//! as a bridge for messages between actors.
//!
//! Liberally derived from the [Firefox JS implementation].
//!
//! [Firefox JS implementation]: https://searchfox.org/mozilla-central/source/devtools/server/actors/descriptors/watcher.js

use std::collections::HashMap;

use devtools_traits::get_time_stamp;
use log::warn;
use malloc_size_of_derive::MallocSizeOf;
use serde::Serialize;
use serde_json::{Map, Value};
use servo_base::id::BrowsingContextId;
use servo_url::ServoUrl;

use self::network_parent::NetworkParentActor;
use super::breakpoint::BreakpointListActor;
use super::thread::ThreadActor;
use super::worker::WorkerTargetActorMsg;
use crate::actor::{Actor, ActorEncode, ActorError, ActorRegistry};
use crate::actors::blackboxing::BlackboxingActor;
use crate::actors::browsing_context::{BrowsingContextActor, BrowsingContextActorMsg};
use crate::actors::console::ConsoleActor;
use crate::actors::root::RootActor;
use crate::actors::watcher::target_configuration::{
    TargetConfigurationActor, TargetConfigurationActorMsg,
};
use crate::actors::watcher::thread_configuration::ThreadConfigurationActor;
use crate::protocol::{ClientRequest, DevtoolsConnection, JsonPacketStream};
use crate::resource::{ResourceArrayType, ResourceAvailable};
use crate::{ActorMsg, EmptyReplyMsg, IdMap, StreamId, WorkerTargetActor};

pub mod network_parent;
pub mod target_configuration;
pub mod thread_configuration;

/// Describes the debugged context. It informs the server of which objects can be debugged.
/// <https://searchfox.org/mozilla-central/source/devtools/server/actors/watcher/session-context.js>
#[derive(Serialize, MallocSizeOf)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SessionContext {
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
                ("worker", true),
                ("service_worker", true),
                ("shared_worker", false),
            ]),
            // At the moment, we are blocking most resources to avoid errors
            // Support for them will be enabled gradually once the corresponding actors start
            // working properly
            supported_resources: HashMap::from([
                ("console-message", true),
                ("css-change", true),
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
                ("platform-message", true),
                ("network-event", true),
                ("network-event-stacktrace", false),
                ("reflow", true),
                ("stylesheet", false),
                ("source", true),
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

#[derive(Serialize, MallocSizeOf)]
pub enum SessionContextType {
    BrowserElement,
    _ContextProcess,
    _WebExtension,
    _Worker,
    _All,
}

#[derive(Serialize)]
#[serde(untagged)]
enum TargetActorMsg {
    BrowsingContext(BrowsingContextActorMsg),
    Worker(WorkerTargetActorMsg),
}

#[derive(Serialize)]
struct WatchTargetsReply {
    from: String,
    #[serde(rename = "type")]
    type_: String,
    target: TargetActorMsg,
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
    network: ActorMsg,
}

#[derive(Serialize)]
struct GetTargetConfigurationActorReply {
    from: String,
    configuration: TargetConfigurationActorMsg,
}

#[derive(Serialize)]
struct GetThreadConfigurationActorReply {
    from: String,
    configuration: ActorMsg,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct GetBlackboxingActorReply {
    from: String,
    blackboxing: ActorMsg,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct GetBreakpointListActorReply {
    from: String,
    breakpoint_list: ActorMsg,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct DocumentEvent {
    #[serde(rename = "hasNativeConsoleAPI")]
    has_native_console_api: Option<bool>,
    name: String,
    #[serde(rename = "newURI")]
    new_uri: Option<String>,
    time: u64,
    title: Option<String>,
    url: Option<String>,
}

#[derive(Serialize)]
struct WatcherTraits {
    resources: HashMap<&'static str, bool>,
    #[serde(flatten)]
    targets: HashMap<&'static str, bool>,
}

#[derive(Serialize)]
pub(crate) struct WatcherActorMsg {
    actor: String,
    traits: WatcherTraits,
}

#[derive(MallocSizeOf)]
pub(crate) struct WatcherActor {
    name: String,
    pub browsing_context_name: String,
    network_parent_name: String,
    target_configuration_name: String,
    thread_configuration_name: String,
    breakpoint_list_name: String,
    blackboxing_name: String,
    session_context: SessionContext,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct WillNavigateMessage {
    #[serde(rename = "browsingContextID")]
    browsing_context_id: u32,
    inner_window_id: u32,
    name: String,
    time: u64,
    is_frame_switching: bool,
    #[serde(rename = "newURI")]
    new_uri: ServoUrl,
}

impl Actor for WatcherActor {
    fn name(&self) -> String {
        self.name.clone()
    }

    /// The watcher actor can handle the following messages:
    ///
    /// - `watchTargets`: Returns a list of objects to debug. Since we only support web views, it
    ///   returns the associated `BrowsingContextActor`. Every target sent creates a
    ///   `target-available-form` event.
    ///
    /// - `unwatchTargets`: Stop watching a set of targets.
    ///   This is currently a no-op because `watchTargets` only returns a point-in-time snapshot.
    ///
    /// - `watchResources`: Start watching certain resource types. This sends
    ///   `resources-available-array` events.
    ///
    /// - `unwatchResources`: Stop watching a set of resources.
    ///   This is currently a no-op because `watchResources` only returns a point-in-time snapshot.
    ///
    /// - `getNetworkParentActor`: Returns the network parent actor. It doesn't seem to do much at
    ///   the moment.
    ///
    /// - `getTargetConfigurationActor`: Returns the configuration actor for a specific target, so
    ///   that the server can update its settings.
    ///
    /// - `getThreadConfigurationActor`: The same but with the configuration actor for the thread
    fn handle_message(
        &self,
        mut request: ClientRequest,
        registry: &ActorRegistry,
        msg_type: &str,
        msg: &Map<String, Value>,
        _id: StreamId,
    ) -> Result<(), ActorError> {
        let browsing_context_actor =
            registry.find::<BrowsingContextActor>(&self.browsing_context_name);
        let root_actor = registry.find::<RootActor>("root");
        match msg_type {
            "watchTargets" => {
                // As per logs we either get targetType as "frame" or "worker"
                let target_type = msg
                    .get("targetType")
                    .and_then(Value::as_str)
                    .unwrap_or("frame"); // default to "frame"

                if target_type == "frame" {
                    let msg = WatchTargetsReply {
                        from: self.name(),
                        type_: "target-available-form".into(),
                        target: TargetActorMsg::BrowsingContext(
                            browsing_context_actor.encode(registry),
                        ),
                    };
                    let _ = request.write_json_packet(&msg);

                    browsing_context_actor.frame_update(&mut request);
                } else if target_type == "worker" {
                    for worker_name in &*root_actor.workers.borrow() {
                        let worker_msg = WatchTargetsReply {
                            from: self.name(),
                            type_: "target-available-form".into(),
                            target: TargetActorMsg::Worker(
                                registry.encode::<WorkerTargetActor, _>(worker_name),
                            ),
                        };
                        let _ = request.write_json_packet(&worker_msg);
                    }
                } else if target_type == "service_worker" {
                    for worker_name in &*root_actor.service_workers.borrow() {
                        let worker_msg = WatchTargetsReply {
                            from: self.name(),
                            type_: "target-available-form".into(),
                            target: TargetActorMsg::Worker(
                                registry.encode::<WorkerTargetActor, _>(worker_name),
                            ),
                        };
                        let _ = request.write_json_packet(&worker_msg);
                    }
                } else {
                    warn!("Unexpected target_type: {}", target_type);
                }

                // Messages that contain a `type` field are used to send event callbacks, but they
                // don't count as a reply. Since every message needs to be responded, we send an
                // extra empty packet to the devtools host to inform that we successfully received
                // and processed the message so that it can continue
                let msg = EmptyReplyMsg { from: self.name() };
                request.reply_final(&msg)?
            },
            "unwatchTargets" => {
                // "unwatchTargets" messages are one-way and expect no reply.
                request.mark_handled();
            },
            "watchResources" => {
                let Some(resource_types) = msg.get("resourceTypes") else {
                    return Err(ActorError::MissingParameter);
                };
                let Some(resource_types) = resource_types.as_array() else {
                    return Err(ActorError::BadParameterType);
                };

                for resource in resource_types {
                    let Some(resource) = resource.as_str() else {
                        continue;
                    };
                    match resource {
                        "document-event" => {
                            // TODO: This is a hacky way of sending the 3 messages
                            //       Figure out if there needs work to be done here, ensure the page is loaded
                            for &name in ["dom-loading", "dom-interactive", "dom-complete"].iter() {
                                let event = DocumentEvent {
                                    has_native_console_api: None,
                                    name: name.into(),
                                    new_uri: None,
                                    time: get_time_stamp(),
                                    title: Some(browsing_context_actor.title.borrow().clone()),
                                    url: Some(browsing_context_actor.url.borrow().clone()),
                                };
                                browsing_context_actor.resource_array(
                                    event,
                                    resource.into(),
                                    ResourceArrayType::Available,
                                    &mut request,
                                );
                            }
                        },
                        "source" => {
                            let thread_actor =
                                registry.find::<ThreadActor>(&browsing_context_actor.thread_name);
                            browsing_context_actor.resources_array(
                                thread_actor.source_manager.source_forms(registry),
                                resource.into(),
                                ResourceArrayType::Available,
                                &mut request,
                            );

                            for worker_name in &*root_actor.workers.borrow() {
                                let worker_actor = registry.find::<WorkerTargetActor>(worker_name);
                                let thread_actor =
                                    registry.find::<ThreadActor>(&worker_actor.thread_name);

                                worker_actor.resources_array(
                                    thread_actor.source_manager.source_forms(registry),
                                    resource.into(),
                                    ResourceArrayType::Available,
                                    &mut request,
                                );
                            }
                        },
                        "console-message" | "error-message" => {
                            let console_actor =
                                registry.find::<ConsoleActor>(&browsing_context_actor.console_name);
                            console_actor.received_first_message_from_client();
                            browsing_context_actor.resources_array(
                                console_actor.get_cached_messages(registry, resource),
                                resource.into(),
                                ResourceArrayType::Available,
                                &mut request,
                            );

                            for worker_name in &*root_actor.workers.borrow() {
                                let worker_actor = registry.find::<WorkerTargetActor>(worker_name);
                                let console_actor =
                                    registry.find::<ConsoleActor>(&worker_actor.console_name);

                                worker_actor.resources_array(
                                    console_actor.get_cached_messages(registry, resource),
                                    resource.into(),
                                    ResourceArrayType::Available,
                                    &mut request,
                                );
                            }
                        },
                        "network-event" => {},
                        _ => warn!("resource {} not handled yet", resource),
                    }
                }
                let msg = EmptyReplyMsg { from: self.name() };
                request.reply_final(&msg)?
            },
            "unwatchResources" => {
                // "unwatchResources" messages are one-way and expect no reply.
                request.mark_handled();
            },
            "getParentBrowsingContextID" => {
                let msg = GetParentBrowsingContextIDReply {
                    from: self.name(),
                    browsing_context_id: browsing_context_actor.browsing_context_id.value(),
                };
                request.reply_final(&msg)?
            },
            "getNetworkParentActor" => {
                let msg = GetNetworkParentActorReply {
                    from: self.name(),
                    network: registry.encode::<NetworkParentActor, _>(&self.network_parent_name),
                };
                request.reply_final(&msg)?
            },
            "getTargetConfigurationActor" => {
                let msg = GetTargetConfigurationActorReply {
                    from: self.name(),
                    configuration: registry
                        .encode::<TargetConfigurationActor, _>(&self.target_configuration_name),
                };
                request.reply_final(&msg)?
            },
            "getThreadConfigurationActor" => {
                let msg = GetThreadConfigurationActorReply {
                    from: self.name(),
                    configuration: registry
                        .encode::<ThreadConfigurationActor, _>(&self.thread_configuration_name),
                };
                request.reply_final(&msg)?
            },
            "getBreakpointListActor" => {
                let msg = GetBreakpointListActorReply {
                    from: self.name(),
                    breakpoint_list: registry
                        .encode::<BreakpointListActor, _>(&self.breakpoint_list_name),
                };
                request.reply_final(&msg)?
            },
            "getBlackboxingActor" => {
                let msg = GetBlackboxingActorReply {
                    from: self.name(),
                    blackboxing: registry.encode::<BlackboxingActor, _>(&self.blackboxing_name),
                };
                request.reply_final(&msg)?
            },
            _ => return Err(ActorError::UnrecognizedPacketType),
        };
        Ok(())
    }
}

impl ResourceAvailable for WatcherActor {
    fn actor_name(&self) -> String {
        self.name.clone()
    }
}

impl WatcherActor {
    pub fn register(
        registry: &ActorRegistry,
        browsing_context_name: String,
        session_context: SessionContext,
    ) -> String {
        let network_parent_name = NetworkParentActor::register(registry);
        let target_configuration_name = TargetConfigurationActor::register(registry);
        let thread_configuration_name = ThreadConfigurationActor::register(registry);
        let breakpoint_list_name =
            BreakpointListActor::register(registry, browsing_context_name.clone());
        let blackboxing_name = BlackboxingActor::register(registry);

        let name = registry.new_name::<Self>();
        let actor = Self {
            name: name.clone(),
            browsing_context_name,
            network_parent_name,
            target_configuration_name,
            thread_configuration_name,
            breakpoint_list_name,
            blackboxing_name,
            session_context,
        };

        registry.register::<Self>(actor);

        name
    }

    pub fn emit_will_navigate<'a>(
        &self,
        browsing_context_id: BrowsingContextId,
        url: ServoUrl,
        connections: impl Iterator<Item = &'a mut DevtoolsConnection>,
        id_map: &mut IdMap,
    ) {
        let msg = WillNavigateMessage {
            browsing_context_id: id_map.browsing_context_id(browsing_context_id).value(),
            inner_window_id: 0, // TODO: set this to the correct value
            name: "will-navigate".to_string(),
            time: get_time_stamp(),
            is_frame_switching: false, // TODO: Implement frame switching
            new_uri: url,
        };

        for stream in connections {
            self.resource_array(
                msg.clone(),
                "document-event".to_string(),
                ResourceArrayType::Available,
                stream,
            );
        }
    }
}

impl ActorEncode<WatcherActorMsg> for WatcherActor {
    fn encode(&self, _: &ActorRegistry) -> WatcherActorMsg {
        WatcherActorMsg {
            actor: self.name(),
            traits: WatcherTraits {
                resources: self.session_context.supported_resources.clone(),
                targets: self.session_context.supported_targets.clone(),
            },
        }
    }
}

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
                ("process", true),
                ("worker", true),
                ("service_worker", true),
                ("shared_worker", true),
            ]),
            supported_resources: HashMap::from([
                ("console-message", true),
                ("css-change", true),
                ("css-message", true),
                ("css-registered-properties", true),
                ("document-event", true),
                ("Cache", true),
                ("cookies", true),
                ("error-message", true),
                ("extension-storage", true),
                ("indexed-db", true),
                ("local-storage", true),
                ("session-storage", true),
                ("platform-message", true),
                ("network-event", true),
                ("network-event-stacktrace", true),
                ("reflow", true),
                ("stylesheet", true),
                ("source", true),
                ("thread-state", true),
                ("server-sent-event", true),
                ("websocket", true),
                ("jstracer-trace", true),
                ("jstracer-state", true),
                ("last-private-context-exit", true),
            ]),
            context_type,
        }
    }
}

#[derive(Serialize)]
struct WatchTargetsReply {
    from: String,
    target: BrowsingContextActorMsg,
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
        _msg: &Map<String, Value>,
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
                    target,
                });
                ActorMessageStatus::Processed
            },
            // TODO: Handle watchResources (!!!)
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

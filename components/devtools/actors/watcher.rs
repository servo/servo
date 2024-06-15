/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#![allow(dead_code)] // TODO: Remove this

use std::collections::HashMap;
use std::net::TcpStream;

use serde::Serialize;
use serde_json::{Map, Value};

use crate::actor::{Actor, ActorMessageStatus, ActorRegistry};
use crate::StreamId;

#[derive(Serialize)]
pub enum SessionContextType {
    BrowserElement,
    ContextProcess,
    WebExtension,
    Worker,
    All,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionContext {
    is_server_target_switching_enabled: bool,
    supported_targets: HashMap<&'static str, bool>,
    supported_resources: HashMap<&'static str, bool>,
    r#type: SessionContextType,
}

impl SessionContext {
    pub fn new(r#type: SessionContextType) -> Self {
        Self {
            is_server_target_switching_enabled: false,
            supported_targets: HashMap::new(), // TODO: Fill this
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
            r#type,
        }
    }
}

/*
{"actor":"server1.conn0.watcher42",
 "traits":{"frame":true,"process":true,"worker":true,"service_worker":true,"resources":{"console-message":true,"css-change":true,"css-message":true,"css-registered-properties":true,"document-event":true,"Cache":true,"cookies":true,"error-message":true,"extension-storage":true,"indexed-db":true,"local-storage":true,"session-storage":true,"platform-message":true,"network-event":true,"network-event-stacktrace":true,"reflow":true,"stylesheet":true,"source":true,"thread-state":true,"server-sent-event":true,"websocket":true,"jstracer-trace":true,"jstracer-state":true,"last-private-context-exit":true}},
 "from":"server1.conn0.tabDescriptor11"}
 */

#[derive(Serialize)]
pub struct WatcherTraits {
    resources: HashMap<&'static str, bool>,
    frame: bool,
    process: bool,
    worker: bool,
    service_worker: bool,
    // shared_worker: bool,
}

pub struct WatcherActor {
    name: String,
    session_context: SessionContext,
}

impl WatcherActor {
    pub fn new(name: String, session_context: SessionContext) -> Self {
        Self {
            name,
            session_context,
        }
    }

    // TODO: Change this for encodable and handle message here
    pub fn traits(&self) -> WatcherTraits {
        WatcherTraits {
            resources: self.session_context.supported_resources.clone(),
            frame: true,
            process: true,
            worker: true,
            service_worker: true,
        }
    }
}

impl Actor for WatcherActor {
    fn name(&self) -> String {
        self.name.clone()
    }

    fn handle_message(
        &self,
        _registry: &ActorRegistry,
        msg_type: &str,
        _msg: &Map<String, Value>,
        _stream: &mut TcpStream,
        _id: StreamId,
    ) -> Result<ActorMessageStatus, ()> {
        Ok(match msg_type {
            _ => ActorMessageStatus::Ignored,
        })
    }
}

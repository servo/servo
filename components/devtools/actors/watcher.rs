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
    BrowserElement(u32),
    ContextProcess,
    WebExtension,
    Worker,
    All,
}

/*TARGETS {
  [Targets.TYPES.FRAME]: true,
  [Targets.TYPES.PROCESS]: true,
  [Targets.TYPES.WORKER]: true,
  [Targets.TYPES.SERVICE_WORKER]:
    type == SESSION_TYPES.BROWSER_ELEMENT || type == SESSION_TYPES.ALL,
  [Targets.TYPES.SHARED_WORKER]: type == SESSION_TYPES.ALL,
};*/

/*RESOURCES {
  [Resources.TYPES.CONSOLE_MESSAGE]: true,
  [Resources.TYPES.CSS_CHANGE]: isTabOrWebExtensionToolbox,
  [Resources.TYPES.CSS_MESSAGE]: true,
  [Resources.TYPES.CSS_REGISTERED_PROPERTIES]: true,
  [Resources.TYPES.DOCUMENT_EVENT]: true,
  [Resources.TYPES.CACHE_STORAGE]: true,
  [Resources.TYPES.COOKIE]: true,
  [Resources.TYPES.ERROR_MESSAGE]: true,
  [Resources.TYPES.EXTENSION_STORAGE]: true,
  [Resources.TYPES.INDEXED_DB]: true,
  [Resources.TYPES.LOCAL_STORAGE]: true,
  [Resources.TYPES.SESSION_STORAGE]: true,
  [Resources.TYPES.PLATFORM_MESSAGE]: true,
  [Resources.TYPES.NETWORK_EVENT]: true,
  [Resources.TYPES.NETWORK_EVENT_STACKTRACE]: true,
  [Resources.TYPES.REFLOW]: true,
  [Resources.TYPES.STYLESHEET]: true,
  [Resources.TYPES.SOURCE]: true,
  [Resources.TYPES.THREAD_STATE]: true,
  [Resources.TYPES.SERVER_SENT_EVENT]: true,
  [Resources.TYPES.WEBSOCKET]: true,
  [Resources.TYPES.JSTRACER_TRACE]: true,
  [Resources.TYPES.JSTRACER_STATE]: true,
  [Resources.TYPES.LAST_PRIVATE_CONTEXT_EXIT]: true,
};*/

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionContext {
    isServerTargetSwitchingEnabled: bool,
    supportedTargets: HashMap<&'static str, bool>,
    supportedResources: HashMap<&'static str, bool>,
    r#type: SessionContextType,
}

impl SessionContext {
    pub fn new(r#type: SessionContextType) -> Self {
        Self {
            isServerTargetSwitchingEnabled: false,
            supportedTargets: HashMap::new(), // TODO: Fill this
            supportedResources: HashMap::from([
                ("Resources.TYPES.CONSOLE_MESSAGE", true),
                ("Resources.TYPES.CSS_CHANGE", true),
            ]),
            r#type,
        }
    }
}

#[derive(Serialize)]
struct WatcherTraits {
    resources: HashMap<&'static str, bool>,
    #[serde(rename = "Targets.TYPES.FRAME")]
    frame: bool,
    #[serde(rename = "Targets.TYPES.PROCESS")]
    process: bool,
    #[serde(rename = "Targets.TYPES.WORKER")]
    worker: bool,
    #[serde(rename = "Targets.TYPES.SERVICE_WORKER")]
    service_worker: bool,
    #[serde(rename = "Targets.TYPES.SHARED_WORKER")]
    shared_worker: bool,
}

#[derive(Serialize)]
pub struct WatcherActorMsg {
    actor: String,
    traits: WatcherTraits,
}

pub struct WatcherActor {
    name: String,
    sessionContext: SessionContext,
}

impl WatcherActor {
    pub fn new(name: String, sessionContext: SessionContext) -> Self {
        Self {
            name,
            sessionContext,
        }
    }

    pub fn encodable(&self) -> WatcherActorMsg {
        WatcherActorMsg {
            actor: self.name(),
            traits: WatcherTraits {
                resources: self.sessionContext.supportedResources.clone(),
                frame: true,
                process: true,
                worker: true,
                service_worker: true,
                shared_worker: true,
            },
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

/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Liberally derived from <https://searchfox.org/mozilla-central/source/devtools/server/actors/target-configuration.js>
//! This actor manages the configuration flags that the devtools host can apply to the targets.

use std::collections::HashMap;
use std::net::TcpStream;

use serde::Serialize;
use serde_json::{Map, Value};

use crate::actor::{Actor, ActorMessageStatus, ActorRegistry};
use crate::protocol::JsonPacketStream;
use crate::{EmptyReplyMsg, StreamId};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TargetConfigurationTraits {
    supported_options: HashMap<&'static str, bool>,
}

#[derive(Serialize)]
pub struct TargetConfigurationActorMsg {
    actor: String,
    configuration: HashMap<&'static str, bool>,
    traits: TargetConfigurationTraits,
}

pub struct TargetConfigurationActor {
    name: String,
    configuration: HashMap<&'static str, bool>,
    supported_options: HashMap<&'static str, bool>,
}

impl Actor for TargetConfigurationActor {
    fn name(&self) -> String {
        self.name.clone()
    }

    /// The target configuration actor can handle the following messages:
    ///
    /// - `updateConfiguration`: Receives new configuration flags from the devtools host.
    fn handle_message(
        &self,
        _registry: &ActorRegistry,
        msg_type: &str,
        _msg: &Map<String, Value>,
        stream: &mut TcpStream,
        _id: StreamId,
    ) -> Result<ActorMessageStatus, ()> {
        Ok(match msg_type {
            "updateConfiguration" => {
                // TODO: Actually update configuration
                let msg = EmptyReplyMsg { from: self.name() };
                let _ = stream.write_json_packet(&msg);
                ActorMessageStatus::Processed
            },
            _ => ActorMessageStatus::Ignored,
        })
    }
}

impl TargetConfigurationActor {
    pub fn new(name: String) -> Self {
        Self {
            name,
            configuration: HashMap::new(),
            supported_options: HashMap::from([
                ("cacheDisabled", false),
                ("colorSchemeSimulation", false),
                ("customFormatters", false),
                ("customUserAgent", false),
                ("javascriptEnabled", false),
                ("overrideDPPX", false),
                ("printSimulationEnabled", false),
                ("rdmPaneMaxTouchPoints", false),
                ("rdmPaneOrientation", false),
                ("recordAllocations", false),
                ("reloadOnTouchSimulationToggle", false),
                ("restoreFocus", false),
                ("serviceWorkersTestingEnabled", false),
                ("setTabOffline", false),
                ("touchEventsOverride", false),
                ("tracerOptions", false),
                ("useSimpleHighlightersForReducedMotion", false),
            ]),
        }
    }

    pub fn encodable(&self) -> TargetConfigurationActorMsg {
        TargetConfigurationActorMsg {
            actor: self.name(),
            configuration: self.configuration.clone(),
            traits: TargetConfigurationTraits {
                supported_options: self.supported_options.clone(),
            },
        }
    }
}

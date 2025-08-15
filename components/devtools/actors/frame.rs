/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use serde::Serialize;
use serde_json::{Map, Value};

use crate::actors::object::ObjectActor;
use crate::StreamId;
use crate::actor::{Actor, ActorError, ActorRegistry};
use crate::protocol::ClientRequest;

use super::source::SourceActor;

#[derive(Clone, Debug)]
pub struct FrameActor {
    pub name: String,
    pub arguments: Vec<Value>,
    pub async_cause: Option<String>,
    pub display_name: String,
    pub oldest: bool,
    pub state: String,
    pub this_object: String,
    pub type_: String,
    pub where_: WhereInfo,
}

#[derive(Clone, Debug, Serialize)]
pub struct WhereInfo {
    pub actor: String,
    pub line: u32,
    pub column: u32,
}

#[derive(Serialize)]
struct FrameEnvironmentReply {
    from: String,
    bindings: FrameBindings,
}

#[derive(Serialize)]
struct FrameBindings {
    arguments: Vec<Value>,
    variables: Map<String, Value>,
}

// https://searchfox.org/mozilla-central/source/devtools/client/fronts/frame.js#1
impl FrameActor {
    pub fn new(
        name: String,
        source_actor: String,
        line: u32,
        column: u32,
        this_object: String,
    ) -> FrameActor {
        FrameActor {
            name,
            arguments: vec![],
            async_cause: None,
            display_name: "tick".to_owned(), // check what other values it can have
            oldest: true,
            state: "on-stack".to_owned(), // what is the correct value?
            this_object,
            type_: "call".to_owned(),
            where_: WhereInfo {
                actor: source_actor,
                line,
                column,
            },
        }
    }

    pub fn register(
        registry: &ActorRegistry,
        source_actor: &SourceActor,
        line: u32,
        column: u32,
    ) -> String {
        let name = registry.new_name("frame");

        // expand obj actor to have rest of the fields
        let this_object = ObjectActor::register(registry, "window".to_owned());

        let actor = FrameActor::new(
            name.clone(),
            source_actor.name.clone(),
            line,
            column,
            this_object, // we need to pass actual obj actor data here
        );

        registry.register_later(Box::new(actor));
        name
    }
}

impl Actor for FrameActor {
    fn name(&self) -> String {
        self.name.clone()
    }

    fn handle_message(
        &self,
        request: ClientRequest,
        _registry: &ActorRegistry,
        msg_type: &str,
        _msg: &Map<String, Value>,
        _id: StreamId,
    ) -> Result<(), ActorError> {
        match msg_type {
            // {"_to": "server1.conn0.watcher34.process7//frame21", "message": {"to": "server1.conn0.watcher34.process7//frame21", "type": "getEnvironment"}}
            "getEnvironment" => {
                let reply = FrameEnvironmentReply {
                    from: self.name(),
                    bindings: FrameBindings {
                        arguments: vec![],
                        variables: Map::new(),
                    },
                };
                request.reply_final(&reply)?
            },
            _ => return Err(ActorError::UnrecognizedPacketType),
        };
        Ok(())
    }
}

/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Liberally derived from the [Firefox JS implementation](http://mxr.mozilla.org/mozilla-central/source/toolkit/devtools/server/actors/webconsole.js).
//! Mediates interaction between the remote web console and equivalent functionality (object
//! inspection, JS evaluation, autocompletion) in Servo.

use std::collections::HashMap;
use std::net::TcpStream;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use atomic_refcell::AtomicRefCell;
use base::generic_channel::{self, GenericSender};
use base::id::TEST_PIPELINE_ID;
use devtools_traits::EvaluateJSReply::{
    ActorValue, BooleanValue, NullValue, NumberValue, StringValue, VoidValue,
};
use devtools_traits::{ConsoleResource, DevtoolScriptControlMsg};
use serde::Serialize;
use serde_json::{self, Map, Number, Value};
use uuid::Uuid;

use crate::actor::{Actor, ActorError, ActorRegistry};
use crate::actors::browsing_context::BrowsingContextActor;
use crate::actors::object::ObjectActor;
use crate::actors::worker::WorkerActor;
use crate::protocol::{ClientRequest, JsonPacketStream};
use crate::resource::{ResourceArrayType, ResourceAvailable};
use crate::{EmptyReplyMsg, StreamId, UniqueId};

#[derive(Serialize)]
pub struct ConsoleClearMessage {
    pub level: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct AutocompleteReply {
    from: String,
    matches: Vec<String>,
    match_prop: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct EvaluateJSReply {
    from: String,
    input: String,
    result: Value,
    timestamp: u64,
    exception: Value,
    exception_message: Value,
    helper_result: Value,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct EvaluateJSEvent {
    from: String,
    #[serde(rename = "type")]
    type_: String,
    input: String,
    result: Value,
    timestamp: u64,
    #[serde(rename = "resultID")]
    result_id: String,
    exception: Value,
    exception_message: Value,
    helper_result: Value,
}

#[derive(Serialize)]
struct EvaluateJSAsyncReply {
    from: String,
    #[serde(rename = "resultID")]
    result_id: String,
}

#[derive(Serialize)]
struct SetPreferencesReply {
    from: String,
    updated: Vec<String>,
}

pub(crate) enum Root {
    BrowsingContext(String),
    DedicatedWorker(String),
}

pub(crate) struct ConsoleActor {
    pub name: String,
    pub root: Root,
    pub cached_events: AtomicRefCell<HashMap<UniqueId, Vec<ConsoleResource>>>,
    pub only_cache: AtomicBool,
}

impl ConsoleActor {
    fn script_chan(&self, registry: &ActorRegistry) -> GenericSender<DevtoolScriptControlMsg> {
        match &self.root {
            Root::BrowsingContext(browsing_context) => registry
                .find::<BrowsingContextActor>(browsing_context)
                .script_chan
                .clone(),
            Root::DedicatedWorker(worker) => {
                registry.find::<WorkerActor>(worker).script_chan.clone()
            },
        }
    }

    fn current_unique_id(&self, registry: &ActorRegistry) -> UniqueId {
        match &self.root {
            Root::BrowsingContext(browsing_context) => UniqueId::Pipeline(
                registry
                    .find::<BrowsingContextActor>(browsing_context)
                    .pipeline_id(),
            ),
            Root::DedicatedWorker(worker) => {
                UniqueId::Worker(registry.find::<WorkerActor>(worker).worker_id)
            },
        }
    }

    fn evaluate_js(
        &self,
        registry: &ActorRegistry,
        msg: &Map<String, Value>,
    ) -> Result<EvaluateJSReply, ()> {
        let input = msg.get("text").unwrap().as_str().unwrap().to_owned();
        let (chan, port) = generic_channel::channel().unwrap();
        // FIXME: Redesign messages so we don't have to fake pipeline ids when
        //        communicating with workers.
        let pipeline = match self.current_unique_id(registry) {
            UniqueId::Pipeline(p) => p,
            UniqueId::Worker(_) => TEST_PIPELINE_ID,
        };
        self.script_chan(registry)
            .send(DevtoolScriptControlMsg::EvaluateJS(
                pipeline,
                input.clone(),
                chan,
            ))
            .unwrap();

        // TODO: Extract conversion into protocol module or some other useful place
        let result = match port.recv().map_err(|_| ())? {
            VoidValue => {
                let mut m = Map::new();
                m.insert("type".to_owned(), Value::String("undefined".to_owned()));
                Value::Object(m)
            },
            NullValue => {
                let mut m = Map::new();
                m.insert("type".to_owned(), Value::String("null".to_owned()));
                Value::Object(m)
            },
            BooleanValue(val) => Value::Bool(val),
            NumberValue(val) => {
                if val.is_nan() {
                    let mut m = Map::new();
                    m.insert("type".to_owned(), Value::String("NaN".to_owned()));
                    Value::Object(m)
                } else if val.is_infinite() {
                    let mut m = Map::new();
                    if val < 0. {
                        m.insert("type".to_owned(), Value::String("-Infinity".to_owned()));
                    } else {
                        m.insert("type".to_owned(), Value::String("Infinity".to_owned()));
                    }
                    Value::Object(m)
                } else if val == 0. && val.is_sign_negative() {
                    let mut m = Map::new();
                    m.insert("type".to_owned(), Value::String("-0".to_owned()));
                    Value::Object(m)
                } else {
                    Value::Number(Number::from_f64(val).unwrap())
                }
            },
            StringValue(s) => Value::String(s),
            ActorValue { class, uuid } => {
                // TODO: Make initial ActorValue message include these properties?
                let mut m = Map::new();
                let actor = ObjectActor::register(registry, uuid);

                m.insert("type".to_owned(), Value::String("object".to_owned()));
                m.insert("class".to_owned(), Value::String(class));
                m.insert("actor".to_owned(), Value::String(actor));
                m.insert("extensible".to_owned(), Value::Bool(true));
                m.insert("frozen".to_owned(), Value::Bool(false));
                m.insert("sealed".to_owned(), Value::Bool(false));
                Value::Object(m)
            },
        };

        // TODO: Catch and return exception values from JS evaluation
        let reply = EvaluateJSReply {
            from: self.name(),
            input,
            result,
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64,
            exception: Value::Null,
            exception_message: Value::Null,
            helper_result: Value::Null,
        };
        std::result::Result::Ok(reply)
    }

    pub(crate) fn handle_console_resource(
        &self,
        resource: ConsoleResource,
        id: UniqueId,
        registry: &ActorRegistry,
        stream: &mut TcpStream,
    ) {
        self.cached_events
            .borrow_mut()
            .entry(id.clone())
            .or_default()
            .push(resource.clone());
        if self.only_cache.load(Ordering::Relaxed) {
            return;
        }
        let resource_type = resource.resource_type();
        if id == self.current_unique_id(registry) {
            if let Root::BrowsingContext(bc) = &self.root {
                registry.find::<BrowsingContextActor>(bc).resource_array(
                    resource,
                    resource_type,
                    ResourceArrayType::Available,
                    stream,
                )
            };
        }
    }

    pub(crate) fn send_clear_message(
        &self,
        id: UniqueId,
        registry: &ActorRegistry,
        stream: &mut TcpStream,
    ) {
        if id == self.current_unique_id(registry) {
            if let Root::BrowsingContext(bc) = &self.root {
                registry.find::<BrowsingContextActor>(bc).resource_array(
                    ConsoleClearMessage {
                        level: "clear".to_owned(),
                    },
                    "console-message".into(),
                    ResourceArrayType::Available,
                    stream,
                )
            };
        }
    }

    pub(crate) fn get_cached_messages(
        &self,
        registry: &ActorRegistry,
        resource: &str,
    ) -> Vec<ConsoleResource> {
        let id = self.current_unique_id(registry);
        let cached_events = self.cached_events.borrow();
        let Some(events) = cached_events.get(&id) else {
            return vec![];
        };
        events
            .iter()
            .filter(|event| event.resource_type() == resource)
            .cloned()
            .collect()
    }
}

impl Actor for ConsoleActor {
    fn name(&self) -> String {
        self.name.clone()
    }

    fn handle_message(
        &self,
        request: ClientRequest,
        registry: &ActorRegistry,
        msg_type: &str,
        msg: &Map<String, Value>,
        _id: StreamId,
    ) -> Result<(), ActorError> {
        match msg_type {
            "clearMessagesCacheAsync" => {
                self.cached_events
                    .borrow_mut()
                    .remove(&self.current_unique_id(registry));
                let msg = EmptyReplyMsg { from: self.name() };
                request.reply_final(&msg)?
            },

            // TODO: implement autocompletion like onAutocomplete in
            //      http://mxr.mozilla.org/mozilla-central/source/toolkit/devtools/server/actors/webconsole.js
            "autocomplete" => {
                let msg = AutocompleteReply {
                    from: self.name(),
                    matches: vec![],
                    match_prop: "".to_owned(),
                };
                request.reply_final(&msg)?
            },

            "evaluateJS" => {
                let msg = self.evaluate_js(registry, msg);
                request.reply_final(&msg)?
            },

            "evaluateJSAsync" => {
                let result_id = Uuid::new_v4().to_string();
                let early_reply = EvaluateJSAsyncReply {
                    from: self.name(),
                    result_id: result_id.clone(),
                };
                // Emit an eager reply so that the client starts listening
                // for an async event with the resultID
                let stream = request.reply(&early_reply)?;

                if msg.get("eager").and_then(|v| v.as_bool()).unwrap_or(false) {
                    // We don't support the side-effect free evaluation that eager evaluation
                    // really needs.
                    return Ok(());
                }

                let reply = self.evaluate_js(registry, msg).unwrap();
                let msg = EvaluateJSEvent {
                    from: self.name(),
                    type_: "evaluationResult".to_owned(),
                    input: reply.input,
                    result: reply.result,
                    timestamp: reply.timestamp,
                    result_id,
                    exception: reply.exception,
                    exception_message: reply.exception_message,
                    helper_result: reply.helper_result,
                };
                // Send the data from evaluateJS along with a resultID
                stream.write_json_packet(&msg)?
            },

            "setPreferences" => {
                let msg = SetPreferencesReply {
                    from: self.name(),
                    updated: vec![],
                };
                request.reply_final(&msg)?
            },

            // NOTE: Do not handle `startListeners`, it is a legacy API.
            // Instead, enable the resource in `WatcherActor::supported_resources`
            // and handle the messages there.
            _ => return Err(ActorError::UnrecognizedPacketType),
        };
        Ok(())
    }
}

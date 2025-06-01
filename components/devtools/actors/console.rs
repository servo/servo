/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Liberally derived from the [Firefox JS implementation](http://mxr.mozilla.org/mozilla-central/source/toolkit/devtools/server/actors/webconsole.js).
//! Mediates interaction between the remote web console and equivalent functionality (object
//! inspection, JS evaluation, autocompletion) in Servo.

use std::cell::RefCell;
use std::collections::HashMap;
use std::net::TcpStream;
use std::time::{SystemTime, UNIX_EPOCH};

use base::id::TEST_PIPELINE_ID;
use devtools_traits::EvaluateJSReply::{
    ActorValue, BooleanValue, NullValue, NumberValue, StringValue, VoidValue,
};
use devtools_traits::{
    CachedConsoleMessage, CachedConsoleMessageTypes, ConsoleLog, ConsoleMessage,
    DevtoolScriptControlMsg, PageError,
};
use ipc_channel::ipc::{self, IpcSender};
use log::debug;
use serde::Serialize;
use serde_json::{self, Map, Number, Value};
use uuid::Uuid;

use crate::actor::{Actor, ActorMessageStatus, ActorRegistry};
use crate::actors::browsing_context::BrowsingContextActor;
use crate::actors::object::ObjectActor;
use crate::actors::worker::WorkerActor;
use crate::protocol::JsonPacketStream;
use crate::resource::ResourceAvailable;
use crate::{StreamId, UniqueId};

trait EncodableConsoleMessage {
    fn encode(&self) -> serde_json::Result<String>;
}

impl EncodableConsoleMessage for CachedConsoleMessage {
    fn encode(&self) -> serde_json::Result<String> {
        match *self {
            CachedConsoleMessage::PageError(ref a) => serde_json::to_string(a),
            CachedConsoleMessage::ConsoleLog(ref a) => serde_json::to_string(a),
        }
    }
}

#[derive(Serialize)]
struct StartedListenersTraits;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct StartedListenersReply {
    from: String,
    native_console_api: bool,
    started_listeners: Vec<String>,
    traits: StartedListenersTraits,
}

#[derive(Serialize)]
struct GetCachedMessagesReply {
    from: String,
    messages: Vec<Map<String, Value>>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct StopListenersReply {
    from: String,
    stopped_listeners: Vec<String>,
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

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct PageErrorWrapper {
    page_error: PageError,
}

pub(crate) enum Root {
    BrowsingContext(String),
    DedicatedWorker(String),
}

pub(crate) struct ConsoleActor {
    pub name: String,
    pub root: Root,
    pub cached_events: RefCell<HashMap<UniqueId, Vec<CachedConsoleMessage>>>,
}

impl ConsoleActor {
    fn script_chan<'a>(
        &self,
        registry: &'a ActorRegistry,
    ) -> &'a IpcSender<DevtoolScriptControlMsg> {
        match &self.root {
            Root::BrowsingContext(bc) => &registry.find::<BrowsingContextActor>(bc).script_chan,
            Root::DedicatedWorker(worker) => &registry.find::<WorkerActor>(worker).script_chan,
        }
    }

    fn current_unique_id(&self, registry: &ActorRegistry) -> UniqueId {
        match &self.root {
            Root::BrowsingContext(bc) => UniqueId::Pipeline(
                registry
                    .find::<BrowsingContextActor>(bc)
                    .active_pipeline_id
                    .get(),
            ),
            Root::DedicatedWorker(w) => UniqueId::Worker(registry.find::<WorkerActor>(w).worker_id),
        }
    }

    fn evaluate_js(
        &self,
        registry: &ActorRegistry,
        msg: &Map<String, Value>,
    ) -> Result<EvaluateJSReply, ()> {
        let input = msg.get("text").unwrap().as_str().unwrap().to_owned();
        let (chan, port) = ipc::channel().unwrap();
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

    pub(crate) fn handle_page_error(
        &self,
        page_error: PageError,
        id: UniqueId,
        registry: &ActorRegistry,
        stream: &mut TcpStream,
    ) {
        self.cached_events
            .borrow_mut()
            .entry(id.clone())
            .or_default()
            .push(CachedConsoleMessage::PageError(page_error.clone()));
        if id == self.current_unique_id(registry) {
            if let Root::BrowsingContext(bc) = &self.root {
                registry
                    .find::<BrowsingContextActor>(bc)
                    .resource_available(
                        PageErrorWrapper { page_error },
                        "error-message".into(),
                        stream,
                    )
            };
        }
    }

    pub(crate) fn handle_console_api(
        &self,
        console_message: ConsoleMessage,
        id: UniqueId,
        registry: &ActorRegistry,
        stream: &mut TcpStream,
    ) {
        let log_message: ConsoleLog = console_message.into();
        self.cached_events
            .borrow_mut()
            .entry(id.clone())
            .or_default()
            .push(CachedConsoleMessage::ConsoleLog(log_message.clone()));
        if id == self.current_unique_id(registry) {
            if let Root::BrowsingContext(bc) = &self.root {
                registry
                    .find::<BrowsingContextActor>(bc)
                    .resource_available(log_message, "console-message".into(), stream)
            };
        }
    }
}

impl Actor for ConsoleActor {
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
            "clearMessagesCache" => {
                self.cached_events
                    .borrow_mut()
                    .remove(&self.current_unique_id(registry));
                ActorMessageStatus::Processed
            },

            "getCachedMessages" => {
                let str_types = msg
                    .get("messageTypes")
                    .unwrap()
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|json_type| json_type.as_str().unwrap());
                let mut message_types = CachedConsoleMessageTypes::empty();
                for str_type in str_types {
                    match str_type {
                        "PageError" => message_types.insert(CachedConsoleMessageTypes::PAGE_ERROR),
                        "ConsoleAPI" => {
                            message_types.insert(CachedConsoleMessageTypes::CONSOLE_API)
                        },
                        s => debug!("unrecognized message type requested: \"{}\"", s),
                    };
                }
                let mut messages = vec![];
                for event in self
                    .cached_events
                    .borrow()
                    .get(&self.current_unique_id(registry))
                    .unwrap_or(&vec![])
                    .iter()
                {
                    let include = match event {
                        CachedConsoleMessage::PageError(_)
                            if message_types.contains(CachedConsoleMessageTypes::PAGE_ERROR) =>
                        {
                            true
                        },
                        CachedConsoleMessage::ConsoleLog(_)
                            if message_types.contains(CachedConsoleMessageTypes::CONSOLE_API) =>
                        {
                            true
                        },
                        _ => false,
                    };
                    if include {
                        let json_string = event.encode().unwrap();
                        let json = serde_json::from_str::<Value>(&json_string).unwrap();
                        messages.push(json.as_object().unwrap().to_owned())
                    }
                }

                let msg = GetCachedMessagesReply {
                    from: self.name(),
                    messages,
                };
                let _ = stream.write_json_packet(&msg);
                ActorMessageStatus::Processed
            },

            "startListeners" => {
                //TODO: actually implement listener filters that support starting/stopping
                let listeners = msg.get("listeners").unwrap().as_array().unwrap().to_owned();
                let msg = StartedListenersReply {
                    from: self.name(),
                    native_console_api: true,
                    started_listeners: listeners
                        .into_iter()
                        .map(|s| s.as_str().unwrap().to_owned())
                        .collect(),
                    traits: StartedListenersTraits,
                };
                let _ = stream.write_json_packet(&msg);
                ActorMessageStatus::Processed
            },

            "stopListeners" => {
                //TODO: actually implement listener filters that support starting/stopping
                let msg = StopListenersReply {
                    from: self.name(),
                    stopped_listeners: msg
                        .get("listeners")
                        .unwrap()
                        .as_array()
                        .unwrap_or(&vec![])
                        .iter()
                        .map(|listener| listener.as_str().unwrap().to_owned())
                        .collect(),
                };
                let _ = stream.write_json_packet(&msg);
                ActorMessageStatus::Processed
            },

            //TODO: implement autocompletion like onAutocomplete in
            //      http://mxr.mozilla.org/mozilla-central/source/toolkit/devtools/server/actors/webconsole.js
            "autocomplete" => {
                let msg = AutocompleteReply {
                    from: self.name(),
                    matches: vec![],
                    match_prop: "".to_owned(),
                };
                let _ = stream.write_json_packet(&msg);
                ActorMessageStatus::Processed
            },

            "evaluateJS" => {
                let msg = self.evaluate_js(registry, msg);
                let _ = stream.write_json_packet(&msg);
                ActorMessageStatus::Processed
            },

            "evaluateJSAsync" => {
                let result_id = Uuid::new_v4().to_string();
                let early_reply = EvaluateJSAsyncReply {
                    from: self.name(),
                    result_id: result_id.clone(),
                };
                // Emit an eager reply so that the client starts listening
                // for an async event with the resultID
                if stream.write_json_packet(&early_reply).is_err() {
                    return Ok(ActorMessageStatus::Processed);
                }

                if msg.get("eager").and_then(|v| v.as_bool()).unwrap_or(false) {
                    // We don't support the side-effect free evaluation that eager evalaution
                    // really needs.
                    return Ok(ActorMessageStatus::Processed);
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
                let _ = stream.write_json_packet(&msg);
                ActorMessageStatus::Processed
            },

            "setPreferences" => {
                let msg = SetPreferencesReply {
                    from: self.name(),
                    updated: vec![],
                };
                let _ = stream.write_json_packet(&msg);
                ActorMessageStatus::Processed
            },

            _ => ActorMessageStatus::Ignored,
        })
    }
}

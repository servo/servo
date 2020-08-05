/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Liberally derived from the [Firefox JS implementation]
//! (http://mxr.mozilla.org/mozilla-central/source/toolkit/devtools/server/actors/webconsole.js).
//! Mediates interaction between the remote web console and equivalent functionality (object
//! inspection, JS evaluation, autocompletion) in Servo.

use crate::actor::{Actor, ActorMessageStatus, ActorRegistry};
use crate::actors::browsing_context::BrowsingContextActor;
use crate::actors::object::ObjectActor;
use crate::actors::worker::WorkerActor;
use crate::protocol::JsonPacketStream;
use crate::{StreamId, UniqueId};
use devtools_traits::CachedConsoleMessage;
use devtools_traits::ConsoleMessage;
use devtools_traits::EvaluateJSReply::{ActorValue, BooleanValue, StringValue};
use devtools_traits::EvaluateJSReply::{NullValue, NumberValue, VoidValue};
use devtools_traits::{
    CachedConsoleMessageTypes, ConsoleAPI, DevtoolScriptControlMsg, LogLevel, PageError,
};
use ipc_channel::ipc::{self, IpcSender};
use msg::constellation_msg::TEST_PIPELINE_ID;
use serde_json::{self, Map, Number, Value};
use std::cell::RefCell;
use std::collections::HashMap;
use std::net::TcpStream;
use time::precise_time_ns;
use uuid::Uuid;

trait EncodableConsoleMessage {
    fn encode(&self) -> serde_json::Result<String>;
}

impl EncodableConsoleMessage for CachedConsoleMessage {
    fn encode(&self) -> serde_json::Result<String> {
        match *self {
            CachedConsoleMessage::PageError(ref a) => serde_json::to_string(a),
            CachedConsoleMessage::ConsoleAPI(ref a) => serde_json::to_string(a),
        }
    }
}

#[derive(Serialize)]
struct StartedListenersTraits;

#[derive(Serialize)]
struct StartedListenersReply {
    from: String,
    nativeConsoleAPI: bool,
    startedListeners: Vec<String>,
    traits: StartedListenersTraits,
}

#[derive(Serialize)]
struct GetCachedMessagesReply {
    from: String,
    messages: Vec<Map<String, Value>>,
}

#[derive(Serialize)]
struct StopListenersReply {
    from: String,
    stoppedListeners: Vec<String>,
}

#[derive(Serialize)]
struct AutocompleteReply {
    from: String,
    matches: Vec<String>,
    matchProp: String,
}

#[derive(Serialize)]
struct EvaluateJSReply {
    from: String,
    input: String,
    result: Value,
    timestamp: u64,
    exception: Value,
    exceptionMessage: Value,
    helperResult: Value,
}

#[derive(Serialize)]
struct EvaluateJSEvent {
    from: String,
    r#type: String,
    input: String,
    result: Value,
    timestamp: u64,
    resultID: String,
    exception: Value,
    exceptionMessage: Value,
    helperResult: Value,
}

#[derive(Serialize)]
struct EvaluateJSAsyncReply {
    from: String,
    resultID: String,
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

    fn streams_mut<'a>(&self, registry: &'a ActorRegistry, cb: impl Fn(&mut TcpStream)) {
        match &self.root {
            Root::BrowsingContext(bc) => registry
                .find::<BrowsingContextActor>(bc)
                .streams
                .borrow_mut()
                .values_mut()
                .for_each(cb),
            Root::DedicatedWorker(worker) => registry
                .find::<WorkerActor>(worker)
                .streams
                .borrow_mut()
                .values_mut()
                .for_each(cb),
        }
    }

    fn current_unique_id(&self, registry: &ActorRegistry) -> UniqueId {
        match &self.root {
            Root::BrowsingContext(bc) => UniqueId::Pipeline(
                registry
                    .find::<BrowsingContextActor>(bc)
                    .active_pipeline
                    .get(),
            ),
            Root::DedicatedWorker(w) => UniqueId::Worker(registry.find::<WorkerActor>(w).id),
        }
    }

    fn evaluateJS(
        &self,
        registry: &ActorRegistry,
        msg: &Map<String, Value>,
    ) -> Result<EvaluateJSReply, ()> {
        let input = msg.get("text").unwrap().as_str().unwrap().to_owned();
        let (chan, port) = ipc::channel().unwrap();
        // FIXME: redesign messages so we don't have to fake pipeline ids when
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

        //TODO: extract conversion into protocol module or some other useful place
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
                //TODO: make initial ActorValue message include these properties?
                let mut m = Map::new();
                let actor = ObjectActor::new(registry, uuid);

                m.insert("type".to_owned(), Value::String("object".to_owned()));
                m.insert("class".to_owned(), Value::String(class));
                m.insert("actor".to_owned(), Value::String(actor));
                m.insert("extensible".to_owned(), Value::Bool(true));
                m.insert("frozen".to_owned(), Value::Bool(false));
                m.insert("sealed".to_owned(), Value::Bool(false));
                Value::Object(m)
            },
        };

        //TODO: catch and return exception values from JS evaluation
        let reply = EvaluateJSReply {
            from: self.name(),
            input: input,
            result: result,
            timestamp: 0,
            exception: Value::Null,
            exceptionMessage: Value::Null,
            helperResult: Value::Null,
        };
        std::result::Result::Ok(reply)
    }

    pub(crate) fn handle_page_error(
        &self,
        page_error: PageError,
        id: UniqueId,
        registry: &ActorRegistry,
    ) {
        self.cached_events
            .borrow_mut()
            .entry(id.clone())
            .or_insert(vec![])
            .push(CachedConsoleMessage::PageError(page_error.clone()));
        if id == self.current_unique_id(registry) {
            let msg = PageErrorMsg {
                from: self.name(),
                type_: "pageError".to_owned(),
                pageError: page_error,
            };
            self.streams_mut(registry, |stream| {
                let _ = stream.write_json_packet(&msg);
            });
        }
    }

    pub(crate) fn handle_console_api(
        &self,
        console_message: ConsoleMessage,
        id: UniqueId,
        registry: &ActorRegistry,
    ) {
        let level = match console_message.logLevel {
            LogLevel::Debug => "debug",
            LogLevel::Info => "info",
            LogLevel::Warn => "warn",
            LogLevel::Error => "error",
            LogLevel::Clear => "clear",
            _ => "log",
        }
        .to_owned();
        self.cached_events
            .borrow_mut()
            .entry(id.clone())
            .or_insert(vec![])
            .push(CachedConsoleMessage::ConsoleAPI(ConsoleAPI {
                type_: "ConsoleAPI".to_owned(),
                level: level.clone(),
                filename: console_message.filename.clone(),
                lineNumber: console_message.lineNumber as u32,
                functionName: "".to_string(), //TODO
                timeStamp: precise_time_ns(),
                private: false,
                arguments: vec![console_message.message.clone()],
            }));
        if id == self.current_unique_id(registry) {
            let msg = ConsoleAPICall {
                from: self.name(),
                type_: "consoleAPICall".to_owned(),
                message: ConsoleMsg {
                    level: level,
                    timeStamp: precise_time_ns(),
                    arguments: vec![console_message.message],
                    filename: console_message.filename,
                    lineNumber: console_message.lineNumber,
                    columnNumber: console_message.columnNumber,
                },
            };
            self.streams_mut(registry, |stream| {
                let _ = stream.write_json_packet(&msg);
            });
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
                    .into_iter()
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
                        CachedConsoleMessage::ConsoleAPI(_)
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
                    messages: messages,
                };
                let _ = stream.write_json_packet(&msg);
                ActorMessageStatus::Processed
            },

            "startListeners" => {
                //TODO: actually implement listener filters that support starting/stopping
                let listeners = msg.get("listeners").unwrap().as_array().unwrap().to_owned();
                let msg = StartedListenersReply {
                    from: self.name(),
                    nativeConsoleAPI: true,
                    startedListeners: listeners
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
                    stoppedListeners: msg
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
                    matchProp: "".to_owned(),
                };
                let _ = stream.write_json_packet(&msg);
                ActorMessageStatus::Processed
            },

            "evaluateJS" => {
                let msg = self.evaluateJS(&registry, &msg);
                let _ = stream.write_json_packet(&msg);
                ActorMessageStatus::Processed
            },

            "evaluateJSAsync" => {
                let resultID = Uuid::new_v4().to_string();
                let early_reply = EvaluateJSAsyncReply {
                    from: self.name(),
                    resultID: resultID.clone(),
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

                let reply = self.evaluateJS(&registry, &msg).unwrap();
                let msg = EvaluateJSEvent {
                    from: self.name(),
                    r#type: "evaluationResult".to_owned(),
                    input: reply.input,
                    result: reply.result,
                    timestamp: reply.timestamp,
                    resultID: resultID,
                    exception: reply.exception,
                    exceptionMessage: reply.exceptionMessage,
                    helperResult: reply.helperResult,
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

#[derive(Serialize)]
struct ConsoleAPICall {
    from: String,
    #[serde(rename = "type")]
    type_: String,
    message: ConsoleMsg,
}

#[derive(Serialize)]
struct ConsoleMsg {
    level: String,
    timeStamp: u64,
    arguments: Vec<String>,
    filename: String,
    lineNumber: usize,
    columnNumber: usize,
}

#[derive(Serialize)]
struct PageErrorMsg {
    from: String,
    #[serde(rename = "type")]
    type_: String,
    pageError: PageError,
}

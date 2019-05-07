/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Liberally derived from the [Firefox JS implementation]
//! (http://mxr.mozilla.org/mozilla-central/source/toolkit/devtools/server/actors/webconsole.js).
//! Mediates interaction between the remote web console and equivalent functionality (object
//! inspection, JS evaluation, autocompletion) in Servo.

use crate::actor::{Actor, ActorMessageStatus, ActorRegistry};
use crate::actors::object::ObjectActor;
use crate::protocol::JsonPacketStream;
use devtools_traits::CachedConsoleMessage;
use devtools_traits::EvaluateJSReply::{ActorValue, BooleanValue, StringValue};
use devtools_traits::EvaluateJSReply::{NullValue, NumberValue, VoidValue};
use devtools_traits::{CachedConsoleMessageTypes, DevtoolScriptControlMsg};
use ipc_channel::ipc::{self, IpcSender};
use msg::constellation_msg::PipelineId;
use serde_json::{self, Map, Number, Value};
use std::cell::RefCell;
use std::net::TcpStream;
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
struct StartedListenersTraits {
    customNetworkRequest: bool,
}

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

pub struct ConsoleActor {
    pub name: String,
    pub pipeline: PipelineId,
    pub script_chan: IpcSender<DevtoolScriptControlMsg>,
    pub streams: RefCell<Vec<TcpStream>>,
}

impl ConsoleActor {
    fn evaluateJS(
        &self,
        registry: &ActorRegistry,
        msg: &Map<String, Value>,
    ) -> Result<EvaluateJSReply, ()> {
        let input = msg.get("text").unwrap().as_str().unwrap().to_owned();
        let (chan, port) = ipc::channel().unwrap();
        self.script_chan
            .send(DevtoolScriptControlMsg::EvaluateJS(
                self.pipeline,
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
    ) -> Result<ActorMessageStatus, ()> {
        Ok(match msg_type {
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
                let (chan, port) = ipc::channel().unwrap();
                self.script_chan
                    .send(DevtoolScriptControlMsg::GetCachedMessages(
                        self.pipeline,
                        message_types,
                        chan,
                    ))
                    .unwrap();
                let messages = port
                    .recv()
                    .map_err(|_| ())?
                    .into_iter()
                    .map(|message| {
                        let json_string = message.encode().unwrap();
                        let json = serde_json::from_str::<Value>(&json_string).unwrap();
                        json.as_object().unwrap().to_owned()
                    })
                    .collect();

                let msg = GetCachedMessagesReply {
                    from: self.name(),
                    messages: messages,
                };
                stream.write_json_packet(&msg);
                ActorMessageStatus::Processed
            },

            "startListeners" => {
                //TODO: actually implement listener filters that support starting/stopping
                let msg = StartedListenersReply {
                    from: self.name(),
                    nativeConsoleAPI: true,
                    startedListeners: vec!["PageError".to_owned(), "ConsoleAPI".to_owned()],
                    traits: StartedListenersTraits {
                        customNetworkRequest: true,
                    },
                };
                stream.write_json_packet(&msg);
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
                stream.write_json_packet(&msg);
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
                stream.write_json_packet(&msg);
                ActorMessageStatus::Processed
            },

            "evaluateJS" => {
                let msg = self.evaluateJS(&registry, &msg);
                stream.write_json_packet(&msg);
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
                stream.write_json_packet(&early_reply);
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
                stream.write_json_packet(&msg);
                ActorMessageStatus::Processed
            },

            "setPreferences" => {
                let msg = SetPreferencesReply {
                    from: self.name(),
                    updated: vec![],
                };
                stream.write_json_packet(&msg);
                ActorMessageStatus::Processed
            },

            _ => ActorMessageStatus::Ignored,
        })
    }
}

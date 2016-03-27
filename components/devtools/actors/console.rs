/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Liberally derived from the [Firefox JS implementation]
//! (http://mxr.mozilla.org/mozilla-central/source/toolkit/devtools/server/actors/webconsole.js).
//! Mediates interaction between the remote web console and equivalent functionality (object
//! inspection, JS evaluation, autocompletion) in Servo.

use actor::{Actor, ActorMessageStatus, ActorRegistry};
use actors::object::ObjectActor;
use devtools_traits::CachedConsoleMessage;
use devtools_traits::EvaluateJSReply::{ActorValue, BooleanValue, StringValue};
use devtools_traits::EvaluateJSReply::{NullValue, NumberValue, VoidValue};
use devtools_traits::{CONSOLE_API, CachedConsoleMessageTypes, DevtoolScriptControlMsg, PAGE_ERROR};
use ipc_channel::ipc::{self, IpcSender};
use msg::constellation_msg::PipelineId;
use protocol::JsonPacketStream;
use rustc_serialize::json::{self, Json, ToJson};
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::net::TcpStream;

trait EncodableConsoleMessage {
    fn encode(&self) -> json::EncodeResult<String>;
}

impl EncodableConsoleMessage for CachedConsoleMessage {
    fn encode(&self) -> json::EncodeResult<String> {
        match *self {
            CachedConsoleMessage::PageError(ref a) => json::encode(a),
            CachedConsoleMessage::ConsoleAPI(ref a) => json::encode(a),
        }
    }
}

#[derive(RustcEncodable)]
struct StartedListenersTraits {
    customNetworkRequest: bool,
}

#[derive(RustcEncodable)]
struct StartedListenersReply {
    from: String,
    nativeConsoleAPI: bool,
    startedListeners: Vec<String>,
    traits: StartedListenersTraits,
}

#[derive(RustcEncodable)]
struct GetCachedMessagesReply {
    from: String,
    messages: Vec<json::Object>,
}

#[derive(RustcEncodable)]
struct StopListenersReply {
    from: String,
    stoppedListeners: Vec<String>,
}

#[derive(RustcEncodable)]
struct AutocompleteReply {
    from: String,
    matches: Vec<String>,
    matchProp: String,
}

#[derive(RustcEncodable)]
struct EvaluateJSReply {
    from: String,
    input: String,
    result: Json,
    timestamp: u64,
    exception: Json,
    exceptionMessage: String,
    helperResult: Json,
}

#[derive(RustcEncodable)]
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

impl Actor for ConsoleActor {
    fn name(&self) -> String {
        self.name.clone()
    }

    fn handle_message(&self,
                      registry: &ActorRegistry,
                      msg_type: &str,
                      msg: &json::Object,
                      stream: &mut TcpStream) -> Result<ActorMessageStatus, ()> {
        Ok(match msg_type {
            "getCachedMessages" => {
                let str_types = msg.get("messageTypes").unwrap().as_array().unwrap().into_iter().map(|json_type| {
                    json_type.as_string().unwrap()
                });
                let mut message_types = CachedConsoleMessageTypes::empty();
                for str_type in str_types {
                    match str_type {
                        "PageError" => message_types.insert(PAGE_ERROR),
                        "ConsoleAPI" => message_types.insert(CONSOLE_API),
                        s => println!("unrecognized message type requested: \"{}\"", s),
                    };
                };
                let (chan, port) = ipc::channel().unwrap();
                self.script_chan.send(DevtoolScriptControlMsg::GetCachedMessages(
                    self.pipeline, message_types, chan)).unwrap();
                let messages = try!(port.recv().map_err(|_| ())).into_iter().map(|message| {
                    let json_string = message.encode().unwrap();
                    let json = Json::from_str(&json_string).unwrap();
                    json.as_object().unwrap().to_owned()
                }).collect();

                let msg = GetCachedMessagesReply {
                    from: self.name(),
                    messages: messages,
                };
                stream.write_json_packet(&msg);
                ActorMessageStatus::Processed
            }

            "startListeners" => {
                //TODO: actually implement listener filters that support starting/stopping
                let msg = StartedListenersReply {
                    from: self.name(),
                    nativeConsoleAPI: true,
                    startedListeners:
                        vec!("PageError".to_owned(), "ConsoleAPI".to_owned()),
                    traits: StartedListenersTraits {
                        customNetworkRequest: true,
                    }
                };
                stream.write_json_packet(&msg);
                ActorMessageStatus::Processed
            }

            "stopListeners" => {
                //TODO: actually implement listener filters that support starting/stopping
                let msg = StopListenersReply {
                    from: self.name(),
                    stoppedListeners: msg.get("listeners")
                                         .unwrap()
                                         .as_array()
                                         .unwrap_or(&vec!())
                                         .iter()
                                         .map(|listener| listener.as_string().unwrap().to_owned())
                                         .collect(),
                };
                stream.write_json_packet(&msg);
                ActorMessageStatus::Processed
            }

            //TODO: implement autocompletion like onAutocomplete in
            //      http://mxr.mozilla.org/mozilla-central/source/toolkit/devtools/server/actors/webconsole.js
            "autocomplete" => {
                let msg = AutocompleteReply {
                    from: self.name(),
                    matches: vec!(),
                    matchProp: "".to_owned(),
                };
                stream.write_json_packet(&msg);
                ActorMessageStatus::Processed
            }

            "evaluateJS" => {
                let input = msg.get("text").unwrap().as_string().unwrap().to_owned();
                let (chan, port) = ipc::channel().unwrap();
                self.script_chan.send(DevtoolScriptControlMsg::EvaluateJS(
                    self.pipeline, input.clone(), chan)).unwrap();

                //TODO: extract conversion into protocol module or some other useful place
                let result = match try!(port.recv().map_err(|_| ())) {
                    VoidValue => {
                        let mut m = BTreeMap::new();
                        m.insert("type".to_owned(), "undefined".to_owned().to_json());
                        Json::Object(m)
                    }
                    NullValue => {
                        let mut m = BTreeMap::new();
                        m.insert("type".to_owned(), "null".to_owned().to_json());
                        Json::Object(m)
                    }
                    BooleanValue(val) => val.to_json(),
                    NumberValue(val) => {
                        if val.is_nan() {
                            let mut m = BTreeMap::new();
                            m.insert("type".to_owned(), "NaN".to_owned().to_json());
                            Json::Object(m)
                        } else if val.is_infinite() {
                            let mut m = BTreeMap::new();
                            if val < 0. {
                                m.insert("type".to_owned(), "-Infinity".to_owned().to_json());
                            } else {
                                m.insert("type".to_owned(), "Infinity".to_owned().to_json());
                            }
                            Json::Object(m)
                        } else if val == 0. && val.is_sign_negative() {
                            let mut m = BTreeMap::new();
                            m.insert("type".to_owned(), "-0".to_owned().to_json());
                            Json::Object(m)
                        } else {
                            val.to_json()
                        }
                    }
                    StringValue(s) => s.to_json(),
                    ActorValue { class, uuid } => {
                        //TODO: make initial ActorValue message include these properties?
                        let mut m = BTreeMap::new();
                        let actor = ObjectActor::new(registry, uuid);

                        m.insert("type".to_owned(), "object".to_owned().to_json());
                        m.insert("class".to_owned(), class.to_json());
                        m.insert("actor".to_owned(), actor.to_json());
                        m.insert("extensible".to_owned(), true.to_json());
                        m.insert("frozen".to_owned(), false.to_json());
                        m.insert("sealed".to_owned(), false.to_json());
                        Json::Object(m)
                    }
                };

                //TODO: catch and return exception values from JS evaluation
                let msg = EvaluateJSReply {
                    from: self.name(),
                    input: input,
                    result: result,
                    timestamp: 0,
                    exception: Json::Object(BTreeMap::new()),
                    exceptionMessage: "".to_owned(),
                    helperResult: Json::Object(BTreeMap::new()),
                };
                stream.write_json_packet(&msg);
                ActorMessageStatus::Processed
            }

            "setPreferences" => {
                let msg = SetPreferencesReply {
                    from: self.name(),
                    updated: vec![],
                };
                stream.write_json_packet(&msg);
                ActorMessageStatus::Processed
            }

            _ => ActorMessageStatus::Ignored
        })
    }
}

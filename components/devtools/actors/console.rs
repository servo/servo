/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

/// Liberally derived from the [Firefox JS implementation](http://mxr.mozilla.org/mozilla-central/source/toolkit/devtools/server/actors/webconsole.js).
/// Mediates interaction between the remote web console and equivalent functionality (object
/// inspection, JS evaluation, autocompletion) in Servo.

use actor::{Actor, ActorRegistry};
use protocol::JsonPacketStream;

use devtools_traits::EvaluateJSReply::{NullValue, VoidValue, NumberValue};
use devtools_traits::EvaluateJSReply::{StringValue, BooleanValue, ActorValue};
use devtools_traits::DevtoolScriptControlMsg;
use msg::constellation_msg::PipelineId;

use collections::BTreeMap;
use core::cell::RefCell;
use serialize::json::{self, Json, ToJson};
use std::old_io::TcpStream;
use std::num::Float;
use std::sync::mpsc::{channel, Sender};

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
#[allow(dead_code)]
struct ConsoleAPIMessage {
    _type: String, //FIXME: should this be __type__ instead?
}

#[derive(RustcEncodable)]
#[allow(dead_code)]
struct PageErrorMessage {
    _type: String, //FIXME: should this be __type__ instead?
    errorMessage: String,
    sourceName: String,
    lineText: String,
    lineNumber: uint,
    columnNumber: uint,
    category: String,
    timeStamp: uint,
    warning: bool,
    error: bool,
    exception: bool,
    strict: bool,
    private: bool,
}

#[derive(RustcEncodable)]
#[allow(dead_code)]
struct LogMessage {
    _type: String, //FIXME: should this be __type__ instead?
    timeStamp: uint,
    message: String,
}

#[derive(RustcEncodable)]
#[allow(dead_code)]
enum ConsoleMessageType {
    ConsoleAPIType(ConsoleAPIMessage),
    PageErrorType(PageErrorMessage),
    LogMessageType(LogMessage),
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
    timestamp: uint,
    exception: Json,
    exceptionMessage: String,
    helperResult: Json,
}

pub struct ConsoleActor {
    pub name: String,
    pub pipeline: PipelineId,
    pub script_chan: Sender<DevtoolScriptControlMsg>,
    pub streams: RefCell<Vec<TcpStream>>,
}

impl Actor for ConsoleActor {
    fn name(&self) -> String {
        self.name.clone()
    }

    fn handle_message(&self,
                      _registry: &ActorRegistry,
                      msg_type: &String,
                      msg: &json::Object,
                      stream: &mut TcpStream) -> Result<bool, ()> {
        Ok(match msg_type.as_slice() {
            "getCachedMessages" => {
                let types = msg.get(&"messageTypes".to_string()).unwrap().as_array().unwrap();
                let /*mut*/ messages = vec!();
                for msg_type in types.iter() {
                    let msg_type = msg_type.as_string().unwrap();
                    match msg_type.as_slice() {
                        "ConsoleAPI" => {
                            //TODO: figure out all consoleapi properties from FFOX source
                        }

                        "PageError" => {
                            //TODO: make script error reporter pass all reported errors
                            //      to devtools and cache them for returning here.

                            /*let message = PageErrorMessage {
                                _type: msg_type.to_string(),
                                sourceName: "".to_string(),
                                lineText: "".to_string(),
                                lineNumber: 0,
                                columnNumber: 0,
                                category: "".to_string(),
                                warning: false,
                                error: true,
                                exception: false,
                                strict: false,
                                private: false,
                                timeStamp: 0,
                                errorMessage: "page error test".to_string(),
                            };
                            messages.push(json::from_str(json::encode(&message).as_slice()).unwrap().as_object().unwrap().clone());*/
                        }

                        "LogMessage" => {
                            //TODO: figure out when LogMessage is necessary
                            /*let message = LogMessage {
                                _type: msg_type.to_string(),
                                timeStamp: 0,
                                message: "log message test".to_string(),
                            };
                            messages.push(json::from_str(json::encode(&message).as_slice()).unwrap().as_object().unwrap().clone());*/
                        }

                        s => println!("unrecognized message type requested: \"{}\"", s),
                    }
                }

                let msg = GetCachedMessagesReply {
                    from: self.name(),
                    messages: messages,
                };
                stream.write_json_packet(&msg);
                true
            }

            "startListeners" => {
                //TODO: actually implement listener filters that support starting/stopping
                let msg = StartedListenersReply {
                    from: self.name(),
                    nativeConsoleAPI: true,
                    startedListeners:
                        vec!("PageError".to_string(), "ConsoleAPI".to_string()),
                    traits: StartedListenersTraits {
                        customNetworkRequest: true,
                    }
                };
                stream.write_json_packet(&msg);
                true
            }

            "stopListeners" => {
                //TODO: actually implement listener filters that support starting/stopping
                let msg = StopListenersReply {
                    from: self.name(),
                    stoppedListeners: msg.get(&"listeners".to_string())
                                         .unwrap()
                                         .as_array()
                                         .unwrap_or(&vec!())
                                         .iter()
                                         .map(|listener| listener.as_string().unwrap().to_string())
                                         .collect(),
                };
                stream.write_json_packet(&msg);
                true
            }

            //TODO: implement autocompletion like onAutocomplete in
            //      http://mxr.mozilla.org/mozilla-central/source/toolkit/devtools/server/actors/webconsole.js
            "autocomplete" => {
                let msg = AutocompleteReply {
                    from: self.name(),
                    matches: vec!(),
                    matchProp: "".to_string(),
                };
                stream.write_json_packet(&msg);
                true
            }

            "evaluateJS" => {
                let input = msg.get(&"text".to_string()).unwrap().as_string().unwrap().to_string();
                let (chan, port) = channel();
                self.script_chan.send(DevtoolScriptControlMsg::EvaluateJS(
                    self.pipeline, input.clone(), chan)).unwrap();

                //TODO: extract conversion into protocol module or some other useful place
                let result = match try!(port.recv().map_err(|_| ())) {
                    VoidValue => {
                        let mut m = BTreeMap::new();
                        m.insert("type".to_string(), "undefined".to_string().to_json());
                        Json::Object(m)
                    }
                    NullValue => {
                        let mut m = BTreeMap::new();
                        m.insert("type".to_string(), "null".to_string().to_json());
                        Json::Object(m)
                    }
                    BooleanValue(val) => val.to_json(),
                    NumberValue(val) => {
                        if val.is_nan() {
                            let mut m = BTreeMap::new();
                            m.insert("type".to_string(), "NaN".to_string().to_json());
                            Json::Object(m)
                        } else if val.is_infinite() {
                            let mut m = BTreeMap::new();
                            if val < 0. {
                                m.insert("type".to_string(), "-Infinity".to_string().to_json());
                            } else {
                                m.insert("type".to_string(), "Infinity".to_string().to_json());
                            }
                            Json::Object(m)
                        } else if val == Float::neg_zero() {
                            let mut m = BTreeMap::new();
                            m.insert("type".to_string(), "-0".to_string().to_json());
                            Json::Object(m)
                        } else {
                            val.to_json()
                        }
                    }
                    StringValue(s) => s.to_json(),
                    ActorValue(s) => {
                        //TODO: make initial ActorValue message include these properties.
                        let mut m = BTreeMap::new();
                        m.insert("type".to_string(), "object".to_string().to_json());
                        m.insert("class".to_string(), "???".to_string().to_json());
                        m.insert("actor".to_string(), s.to_json());
                        m.insert("extensible".to_string(), true.to_json());
                        m.insert("frozen".to_string(), false.to_json());
                        m.insert("sealed".to_string(), false.to_json());
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
                    exceptionMessage: "".to_string(),
                    helperResult: Json::Object(BTreeMap::new()),
                };
                stream.write_json_packet(&msg);
                true
            }

            _ => false
        })
    }
}

/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#![crate_name = "devtools"]
#![crate_type = "rlib"]

#![comment = "The Servo Parallel Browser Project"]
#![license = "MPL"]

#![feature(phase)]

#![feature(phase)]
#[phase(plugin, link)]
extern crate log;

extern crate collections;
extern crate core;
extern crate devtools_traits;
extern crate debug;
extern crate std;
extern crate serialize;
extern crate sync;

use devtools_traits::{ServerExitMsg, DevtoolsControlMsg, NewGlobal};

use collections::TreeMap;
use std::any::{Any, AnyRefExt};
use std::collections::hashmap::HashMap;
use std::comm;
use std::comm::{Disconnected, Empty};
use std::io::{TcpListener, TcpStream};
use std::io::{Acceptor, Listener, EndOfFile, IoError, TimedOut};
use std::num;
use std::task::TaskBuilder;
use serialize::{json, Encodable};
use sync::{Arc, Mutex};

#[deriving(Encodable)]
struct ActorTraits {
    sources: bool
}

#[deriving(Encodable)]
struct RootActorMsg {
    from: String,
    applicationType: String,
    traits: ActorTraits,
}

struct RootActor {
    tabs: Vec<String>,
}

#[deriving(Encodable)]
struct ErrorReply {
    from: String,
    error: String,
    message: String,
}

#[deriving(Encodable)]
struct TabActorMsg {
    actor: String,
    title: String,
    url: String,
    outerWindowID: uint,
    consoleActor: String,
}

struct TabActor {
    name: String,
    title: String,
    url: String,
}

#[deriving(Encodable)]
struct ListTabsReply {
    from: String,
    selected: uint,
    tabs: Vec<TabActorMsg>,
}

#[deriving(Encodable)]
struct TabTraits {
    reconfigure: bool,
}

#[deriving(Encodable)]
struct TabAttachedReply {
    from: String,
    __type__: String,
    threadActor: String,
    cacheEnabled: bool,
    javascriptEnabled: bool,
    traits: TabTraits,
}

#[deriving(Encodable)]
struct TabDetachedReply {
    from: String,
    __type__: String,
}

#[deriving(Encodable)]
struct StartedListenersReply {
    from: String,
    nativeConsoleAPI: bool,
    startedListeners: Vec<String>,
}

#[deriving(Encodable)]
struct ConsoleAPIMessage {
    _type: String,
}

#[deriving(Encodable)]
struct PageErrorMessage {
    _type: String,
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

#[deriving(Encodable)]
struct LogMessage {
    _type: String,
    timeStamp: uint,
    message: String,
}

#[deriving(Encodable)]
enum ConsoleMessageType {
    ConsoleAPIType(ConsoleAPIMessage),
    PageErrorType(PageErrorMessage),
    LogMessageType(LogMessage),
}

#[deriving(Encodable)]
struct GetCachedMessagesReply {
    from: String,
    messages: Vec<json::Object>,
}

#[deriving(Encodable)]
struct StopListenersReply {
    from: String,
    stoppedListeners: Vec<String>,
}

#[deriving(Encodable)]
struct AutocompleteReply {
    from: String,
    matches: Vec<String>,
    matchProp: String,
}

#[deriving(Encodable)]
struct EvaluateJSReply {
    from: String,
    input: String,
    result: json::Json,
    timestamp: uint,
    exception: json::Json,
    exceptionMessage: String,
    helperResult: json::Json,
}

struct ActorRegistry {
    actors: HashMap<String, Box<Actor+Send+Sized>>,
}

impl ActorRegistry {
    fn new() -> ActorRegistry {
        ActorRegistry {
            actors: HashMap::new(),
        }
    }

    fn register(&mut self, actor: Box<Actor+Send+Sized>) {
        self.actors.insert(actor.name().to_string(), actor);
    }

    fn find<'a, T: 'static>(&'a self, name: &str) -> &'a T {
        let actor: &Actor+Send+Sized = *self.actors.find(&name.to_string()).unwrap();
        (actor as &Any).downcast_ref::<T>().unwrap()
    }

    fn handle_message(&self, msg: &json::Object, stream: &mut TcpStream) {
        let to = msg.find(&"to".to_string()).unwrap().as_string().unwrap();
        match self.actors.find(&to.to_string()) {
            None => println!("message received for unknown actor \"{:s}\"", to),
            Some(actor) => {
                let msg_type = msg.find(&"type".to_string()).unwrap()
                    .as_string().unwrap();
                if !actor.handle_message(self, &msg_type.to_string(), msg, stream) {
                    println!("unexpected message type \"{:s}\" found for actor \"{:s}\"",
                             msg_type, to);
                }
            }
        }
    }
}

trait Actor: Any {
    fn handle_message(&self,
                      registry: &ActorRegistry,
                      msg_type: &String,
                      msg: &json::Object,
                      stream: &mut TcpStream) -> bool;
    fn name(&self) -> String;
}

impl Actor for RootActor {
    fn name(&self) -> String {
        "root".to_string()
    }

    fn handle_message(&self,
                      registry: &ActorRegistry,
                      msg_type: &String,
                      _msg: &json::Object,
                      stream: &mut TcpStream) -> bool {
        match msg_type.as_slice() {
            "listAddons" => {
                let actor = ErrorReply {
                    from: "root".to_string(),
                    error: "noAddons".to_string(),
                    message: "This root actor has no browser addons.".to_string(),
                };
                stream.write_json_packet(&actor);
                true
            }
            "listTabs" => {
                let actor = ListTabsReply {
                    from: "root".to_string(),
                    selected: 0,
                    tabs: self.tabs.iter().map(|tab| {
                        registry.find::<TabActor>(tab.as_slice()).encodable()
                    }).collect()
                };
                stream.write_json_packet(&actor);
                true
            }
            _ => false
        }
    }
}

impl RootActor {
    fn encodable(&self) -> RootActorMsg {
        RootActorMsg {
            from: "root".to_string(),
            applicationType: "browser".to_string(),
            traits: ActorTraits {
                sources: true,
            },
        }
    }
}

impl Actor for TabActor {
    fn name(&self) -> String {
        self.name.clone()
    }

    fn handle_message(&self,
                      _registry: &ActorRegistry,
                      msg_type: &String,
                      msg: &json::Object,
                      stream: &mut TcpStream) -> bool {
        match msg_type.as_slice() {
            "attach" => {
                let msg = TabAttachedReply {
                    from: self.name(),
                    __type__: "tabAttached".to_string(),
                    threadActor: self.name(),
                    cacheEnabled: false,
                    javascriptEnabled: true,
                    traits: TabTraits {
                        reconfigure: true,
                    },
                };
                stream.write_json_packet(&msg);
                true
            }
            "detach" => {
                let msg = TabDetachedReply {
                    from: self.name(),
                    __type__: "detached".to_string(),
                };
                stream.write_json_packet(&msg);
                true
            }
            "startListeners" => {
                let msg = StartedListenersReply {
                    from: self.name(),
                    nativeConsoleAPI: true,
                    startedListeners:
                        vec!("PageError".to_string(), "ConsoleAPI".to_string()),
                };
                stream.write_json_packet(&msg);
                true
            }
            "getCachedMessages" => {
                let types = msg.find(&"messageTypes".to_string()).unwrap().as_list().unwrap();
                let mut messages = vec!();
                for msg_type in types.iter() {
                    let msg_type = msg_type.as_string().unwrap();
                    match msg_type.as_slice() {
                        "ConsoleAPI" => {
                            //XXX need more info about consoleapi properties
                        }
                        "PageError" => {
                            let message = PageErrorMessage {
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
                            messages.push(json::from_str(json::encode(&message).as_slice()).unwrap().as_object().unwrap().clone());
                        }
                        "LogMessage" => {
                            let message = LogMessage {
                                _type: msg_type.to_string(),
                                timeStamp: 0,
                                message: "log message test".to_string(),
                            };
                            messages.push(json::from_str(json::encode(&message).as_slice()).unwrap().as_object().unwrap().clone());
                        }
                        s => println!("unrecognized message type requested: \"{:s}\"", s),
                    }
                }
                let msg = GetCachedMessagesReply {
                    from: self.name(),
                    messages: messages,
                };
                stream.write_json_packet(&msg);
                true
            }
            "stopListeners" => {
                let msg = StopListenersReply {
                    from: self.name(),
                    stoppedListeners: msg.find(&"listeners".to_string())
                                         .unwrap()
                                         .as_list()
                                         .unwrap_or(&vec!())
                                         .iter()
                                         .map(|listener| listener.as_string().unwrap().to_string())
                                         .collect(),
                };
                stream.write_json_packet(&msg);
                true
            }
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
                let msg = EvaluateJSReply {
                    from: self.name(),
                    input: msg.find(&"text".to_string()).unwrap().as_string().unwrap().to_string(),
                    result: json::Object(TreeMap::new()),
                    timestamp: 0,
                    exception: json::Object(TreeMap::new()),
                    exceptionMessage: "".to_string(),
                    helperResult: json::Object(TreeMap::new()),
                };
                stream.write_json_packet(&msg);
                true
            }
            _ => false
        }
    }
}

impl TabActor {
    fn encodable(&self) -> TabActorMsg {
        TabActorMsg {
            actor: self.name(),
            title: self.title.clone(),
            url: self.url.clone(),
            outerWindowID: 0,
            consoleActor: self.name(),
        }
    }
}

trait JsonPacketSender {
    fn write_json_packet<'a, T: Encodable<json::Encoder<'a>,IoError>>(&mut self, obj: &T);
}

impl JsonPacketSender for TcpStream {
    fn write_json_packet<'a, T: Encodable<json::Encoder<'a>,IoError>>(&mut self, obj: &T) {
        let s = json::encode(obj).replace("__type__", "type");
        println!("<- {:s}", s);
        self.write_str(s.len().to_string().as_slice()).unwrap();
        self.write_u8(':' as u8).unwrap();
        self.write_str(s.as_slice()).unwrap();
    }
}

pub fn start_server() -> Sender<DevtoolsControlMsg> {
    let (chan, port) = comm::channel();
    TaskBuilder::new().named("devtools").spawn(proc() {
        run_server(port)
    });
    chan
}

static POLL_TIMEOUT: u64 = 300;

fn run_server(port: Receiver<DevtoolsControlMsg>) {
    let listener = TcpListener::bind("127.0.0.1", 6000);

    // bind the listener to the specified address
    let mut acceptor = listener.listen().unwrap();
    acceptor.set_timeout(Some(POLL_TIMEOUT));

    let mut registry = ActorRegistry::new();

    let tab = box TabActor {
        name: "tab1".to_string(),
        title: "Performing Layout".to_string(),
        url: "about-mozilla.html".to_string(),
    };

    let root = box RootActor {
        tabs: vec!(tab.name().to_string()),
    };

    registry.register(tab);
    registry.register(root);

    let actors = Arc::new(Mutex::new(registry));

    fn handle_client(actors: Arc<Mutex<ActorRegistry>>, mut stream: TcpStream) {
        println!("connection established to {:?}", stream.peer_name().unwrap());

        {
            let mut actors = actors.lock();
            let msg = actors.find::<RootActor>("root").encodable();
            stream.write_json_packet(&msg);
        }

        'outer: loop {
            let mut buffer = vec!();
            loop {
                let colon = ':' as u8;
                match stream.read_byte() {
                    Ok(c) if c != colon => buffer.push(c as u8),
                    Ok(_) => {
                        let packet_len_str = String::from_utf8(buffer).unwrap();
                        let packet_len = num::from_str_radix(packet_len_str.as_slice(), 10).unwrap();
                        let packet_buf = stream.read_exact(packet_len).unwrap();
                        let packet = String::from_utf8(packet_buf).unwrap();
                        println!("{:s}", packet);
                        let json_packet = json::from_str(packet.as_slice()).unwrap();
                        actors.lock().handle_message(json_packet.as_object().unwrap(),
                                                     &mut stream);
                        break;
                    }
                    Err(ref e) if e.kind == EndOfFile => {
                        println!("\nEOF");
                        break 'outer;
                    },
                    _ => {
                        println!("\nconnection error");
                        break 'outer;
                    }
                }
            }
        }
    }

    // accept connections and process them, spawning a new tasks for each one
    for stream in acceptor.incoming() {
        match stream {
            Err(ref e) if e.kind == TimedOut => {
                match port.try_recv() {
                    Ok(ServerExitMsg) | Err(Disconnected) => break,
                    Ok(NewGlobal(_)) => { /*TODO*/ },
                    Err(Empty) => acceptor.set_timeout(Some(POLL_TIMEOUT)),
                }
            }
            Err(_e) => { /* connection failed */ }
            Ok(stream) => {
                let actors = actors.clone();
                spawn(proc() {
                    // connection succeeded
                    handle_client(actors, stream.clone())
                })
            }
        }
    }
}

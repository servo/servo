/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! An actor-based remote devtools server implementation. Only tested with
//! nightly Firefox versions at time of writing. Largely based on
//! reverse-engineering of Firefox chrome devtool logs and reading of
//! [code](http://mxr.mozilla.org/mozilla-central/source/toolkit/devtools/server/).

#![crate_name = "devtools"]
#![crate_type = "rlib"]

#![feature(int_uint, box_syntax, io, core, rustc_private)]
#![feature(collections, std_misc)]

#![allow(non_snake_case)]
#![allow(missing_copy_implementations)]

#[macro_use]
extern crate log;

extern crate collections;
extern crate core;
extern crate devtools_traits;
extern crate "serialize" as rustc_serialize;
extern crate serialize;
extern crate msg;
extern crate time;
extern crate util;

use actor::{Actor, ActorRegistry};
use actors::console::ConsoleActor;
use actors::inspector::InspectorActor;
use actors::root::RootActor;
use actors::tab::TabActor;
use protocol::JsonPacketStream;

use devtools_traits::{ConsoleMessage, DevtoolsControlMsg};
use devtools_traits::{DevtoolsPageInfo, DevtoolScriptControlMsg};
use msg::constellation_msg::PipelineId;
use util::task::spawn_named;

use std::borrow::ToOwned;
use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::mpsc::TryRecvError::{Disconnected, Empty};
use std::old_io::{TcpListener, TcpStream};
use std::old_io::{Acceptor, Listener, TimedOut};
use std::sync::{Arc, Mutex};
use time::precise_time_ns;

mod actor;
/// Corresponds to http://mxr.mozilla.org/mozilla-central/source/toolkit/devtools/server/actors/
mod actors {
    pub mod console;
    pub mod inspector;
    pub mod root;
    pub mod tab;
}
mod protocol;

#[derive(RustcEncodable)]
struct ConsoleAPICall {
    from: String,
    __type__: String,
    message: ConsoleMsg,
}

#[derive(RustcEncodable)]
struct ConsoleMsg {
    logLevel: u32,
    timestamp: u64,
    message: String,
}

/// Spin up a devtools server that listens for connections on the specified port.
pub fn start_server(port: u16) -> Sender<DevtoolsControlMsg> {
    let (sender, receiver) = channel();
    spawn_named("Devtools".to_owned(), move || {
        run_server(receiver, port)
    });
    sender
}

static POLL_TIMEOUT: u64 = 300;

fn run_server(receiver: Receiver<DevtoolsControlMsg>, port: u16) {
    let listener = TcpListener::bind(format!("{}:{}", "127.0.0.1", port).as_slice());

    // bind the listener to the specified address
    let mut acceptor = listener.listen().unwrap();
    acceptor.set_timeout(Some(POLL_TIMEOUT));

    let mut registry = ActorRegistry::new();

    let root = box RootActor {
        tabs: vec!(),
    };

    registry.register(root);
    registry.find::<RootActor>("root");

    let actors = Arc::new(Mutex::new(registry));

    let mut accepted_connections: Vec<TcpStream> = Vec::new();

    let mut actor_pipelines: HashMap<PipelineId, String> = HashMap::new();

    /// Process the input from a single devtools client until EOF.
    fn handle_client(actors: Arc<Mutex<ActorRegistry>>, mut stream: TcpStream) {
        println!("connection established to {}", stream.peer_name().unwrap());
        {
            let actors = actors.lock().unwrap();
            let msg = actors.find::<RootActor>("root").encodable();
            stream.write_json_packet(&msg);
        }

        'outer: loop {
            match stream.read_json_packet() {
                Ok(json_packet) => {
                    let mut actors = actors.lock().unwrap();
                    match actors.handle_message(json_packet.as_object().unwrap(),
                                                &mut stream) {
                        Ok(()) => {},
                        Err(()) => {
                            println!("error: devtools actor stopped responding");
                            let _ = stream.close_read();
                            let _ = stream.close_write();
                            break 'outer
                        }
                    }
                }
                Err(e) => {
                    println!("error: {}", e.desc);
                    break 'outer
                }
            }
        }
    }

    // We need separate actor representations for each script global that exists;
    // clients can theoretically connect to multiple globals simultaneously.
    // TODO: move this into the root or tab modules?
    fn handle_new_global(actors: Arc<Mutex<ActorRegistry>>,
                         pipeline: PipelineId,
                         sender: Sender<DevtoolScriptControlMsg>,
                         actor_pipelines: &mut HashMap<PipelineId, String>,
                         page_info: DevtoolsPageInfo) {
        let mut actors = actors.lock().unwrap();

        //TODO: move all this actor creation into a constructor method on TabActor
        let (tab, console, inspector) = {
            let console = ConsoleActor {
                name: actors.new_name("console"),
                script_chan: sender.clone(),
                pipeline: pipeline,
                streams: RefCell::new(Vec::new()),
            };
            let inspector = InspectorActor {
                name: actors.new_name("inspector"),
                walker: RefCell::new(None),
                pageStyle: RefCell::new(None),
                highlighter: RefCell::new(None),
                script_chan: sender,
                pipeline: pipeline,
            };

            let DevtoolsPageInfo { title, url } = page_info;
            let tab = TabActor {
                name: actors.new_name("tab"),
                title: title,
                url: url.serialize(),
                console: console.name(),
                inspector: inspector.name(),
            };

            let root = actors.find_mut::<RootActor>("root");
            root.tabs.push(tab.name.clone());
            (tab, console, inspector)
        };

        actor_pipelines.insert(pipeline, tab.name.clone());
        actors.register(box tab);
        actors.register(box console);
        actors.register(box inspector);
    }

    fn handle_console_message(actors: Arc<Mutex<ActorRegistry>>,
                              id: PipelineId,
                              console_message: ConsoleMessage,
                              actor_pipelines: &HashMap<PipelineId, String>) {
        let console_actor_name = find_console_actor(actors.clone(), id, actor_pipelines);
        let actors = actors.lock().unwrap();
        let console_actor = actors.find::<ConsoleActor>(console_actor_name.as_slice());
        match console_message {
            ConsoleMessage::LogMessage(message) => {
                let msg = ConsoleAPICall {
                    from: console_actor.name.clone(),
                    __type__: "consoleAPICall".to_string(),
                    message: ConsoleMsg {
                        logLevel: 0,
                        timestamp: precise_time_ns(),
                        message: message,
                    },
                };
                for stream in console_actor.streams.borrow_mut().iter_mut() {
                    stream.write_json_packet(&msg);
                }
            }
        }
    }

    fn find_console_actor(actors: Arc<Mutex<ActorRegistry>>,
                          id: PipelineId,
                          actor_pipelines: &HashMap<PipelineId, String>) -> String {
        let actors = actors.lock().unwrap();
        let ref tab_actor_name = (*actor_pipelines)[id];
        let tab_actor = actors.find::<TabActor>(tab_actor_name.as_slice());
        let console_actor_name = tab_actor.console.clone();
        return console_actor_name;
    }

    //TODO: figure out some system that allows us to watch for new connections,
    //      shut down existing ones at arbitrary times, and also watch for messages
    //      from multiple script tasks simultaneously. Polling for new connections
    //      for 300ms and then checking the receiver is not a good compromise
    //      (and makes Servo hang on exit if there's an open connection, no less).
    // accept connections and process them, spawning a new tasks for each one
    loop {
        match acceptor.accept() {
            Err(ref e) if e.kind == TimedOut => {
                match receiver.try_recv() {
                    Ok(DevtoolsControlMsg::ServerExitMsg) | Err(Disconnected) => break,
                    Ok(DevtoolsControlMsg::NewGlobal(id, sender, pageinfo)) =>
                        handle_new_global(actors.clone(), id,sender, &mut actor_pipelines,
                                          pageinfo),
                    Ok(DevtoolsControlMsg::SendConsoleMessage(id, console_message)) =>
                        handle_console_message(actors.clone(), id, console_message,
                                               &actor_pipelines),
                    Err(Empty) => acceptor.set_timeout(Some(POLL_TIMEOUT)),
                }
            }
            Err(_e) => { /* connection failed */ }
            Ok(stream) => {
                let actors = actors.clone();
                accepted_connections.push(stream.clone());
                spawn_named("DevtoolsClientHandler".to_owned(), move || {
                    // connection succeeded
                    handle_client(actors, stream.clone())
                })
            }
        }
    }

    for connection in accepted_connections.iter_mut() {
        let _read = connection.close_read();
        let _write = connection.close_write();
    }
}

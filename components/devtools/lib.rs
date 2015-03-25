/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! An actor-based remote devtools server implementation. Only tested with
//! nightly Firefox versions at time of writing. Largely based on
//! reverse-engineering of Firefox chrome devtool logs and reading of
//! [code](http://mxr.mozilla.org/mozilla-central/source/toolkit/devtools/server/).

#![crate_name = "devtools"]
#![crate_type = "rlib"]

#![feature(int_uint, box_syntax, core, rustc_private)]
#![feature(collections, std_misc)]
#![feature(io)]
#![feature(net)]

#![allow(non_snake_case)]

#[macro_use]
extern crate log;

extern crate collections;
extern crate core;
extern crate devtools_traits;
extern crate "rustc-serialize" as rustc_serialize;
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
use std::sync::mpsc::{channel, Receiver, Sender, RecvError};
use std::net::{TcpListener, TcpStream, Shutdown};
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
    level: String,
    timeStamp: u64,
    arguments: Vec<String>,
    filename: String,
    lineNumber: u32,
    columnNumber: u32,
}

/// Spin up a devtools server that listens for connections on the specified port.
pub fn start_server(port: u16) -> Sender<DevtoolsControlMsg> {
    let (sender, receiver) = channel();
    {
        let sender = sender.clone();
        spawn_named("Devtools".to_owned(), move || {
            run_server(sender, receiver, port)
        });
    }
    sender
}

fn run_server(sender: Sender<DevtoolsControlMsg>,
              receiver: Receiver<DevtoolsControlMsg>,
              port: u16) {
    let listener = TcpListener::bind(&("127.0.0.1", port)).unwrap();

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
        println!("connection established to {}", stream.peer_addr().unwrap());
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
                            let _ = stream.shutdown(Shutdown::Both);
                            break 'outer
                        }
                    }
                }
                Err(e) => {
                    println!("error: {}", e.description());
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
                         scriptSender: Sender<DevtoolScriptControlMsg>,
                         actor_pipelines: &mut HashMap<PipelineId, String>,
                         page_info: DevtoolsPageInfo) {
        let mut actors = actors.lock().unwrap();

        //TODO: move all this actor creation into a constructor method on TabActor
        let (tab, console, inspector) = {
            let console = ConsoleActor {
                name: actors.new_name("console"),
                script_chan: scriptSender.clone(),
                pipeline: pipeline,
                streams: RefCell::new(Vec::new()),
            };
            let inspector = InspectorActor {
                name: actors.new_name("inspector"),
                walker: RefCell::new(None),
                pageStyle: RefCell::new(None),
                highlighter: RefCell::new(None),
                script_chan: scriptSender,
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
        let console_actor = actors.find::<ConsoleActor>(&console_actor_name);
        match console_message {
            ConsoleMessage::LogMessage(message, filename, lineNumber, columnNumber) => {
                let msg = ConsoleAPICall {
                    from: console_actor.name.clone(),
                    __type__: "consoleAPICall".to_string(),
                    message: ConsoleMsg {
                        level: "log".to_string(),
                        timeStamp: precise_time_ns(),
                        arguments: vec!(message),
                        filename: filename,
                        lineNumber: lineNumber,
                        columnNumber: columnNumber,
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
        let tab_actor = actors.find::<TabActor>(tab_actor_name);
        let console_actor_name = tab_actor.console.clone();
        return console_actor_name;
    }

    spawn_named("DevtoolsClientAcceptor".to_owned(), move || {
        // accept connections and process them, spawning a new task for each one
        for stream in listener.incoming() {
            // connection succeeded
            sender.send(DevtoolsControlMsg::AddClient(stream.unwrap())).unwrap();
        }
    });

    loop {
        match receiver.recv() {
            Ok(DevtoolsControlMsg::AddClient(stream)) => {
                let actors = actors.clone();
                accepted_connections.push(stream.try_clone().unwrap());
                spawn_named("DevtoolsClientHandler".to_owned(), move || {
                    handle_client(actors, stream.try_clone().unwrap())
                })
            }
            Ok(DevtoolsControlMsg::ServerExitMsg) | Err(RecvError) => break,
            Ok(DevtoolsControlMsg::NewGlobal(id, scriptSender, pageinfo)) =>
                handle_new_global(actors.clone(), id, scriptSender, &mut actor_pipelines,
                                  pageinfo),
            Ok(DevtoolsControlMsg::SendConsoleMessage(id, console_message)) =>
                handle_console_message(actors.clone(), id, console_message,
                                       &actor_pipelines),
        }
    }

    for connection in accepted_connections.iter_mut() {
        let _ = connection.shutdown(Shutdown::Both);
    }
}

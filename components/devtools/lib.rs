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

/// An actor-based remote devtools server implementation. Only tested with nightly Firefox
/// versions at time of writing. Largely based on reverse-engineering of Firefox chrome
/// devtool logs and reading of [code](http://mxr.mozilla.org/mozilla-central/source/toolkit/devtools/server/).

extern crate collections;
extern crate core;
extern crate devtools_traits;
extern crate debug;
extern crate serialize;
extern crate sync;
extern crate "msg" as servo_msg;

use actor::{Actor, ActorRegistry};
use actors::console::ConsoleActor;
use actors::inspector::InspectorActor;
use actors::root::RootActor;
use actors::tab::TabActor;
use protocol::JsonPacketSender;

use devtools_traits::{ServerExitMsg, DevtoolsControlMsg, NewGlobal, DevtoolScriptControlMsg};
use servo_msg::constellation_msg::PipelineId;

use std::cell::RefCell;
use std::comm;
use std::comm::{Disconnected, Empty};
use std::io::{TcpListener, TcpStream};
use std::io::{Acceptor, Listener, EndOfFile, TimedOut};
use std::num;
use std::task::TaskBuilder;
use serialize::json;
use sync::{Arc, Mutex};

mod actor;
/// Corresponds to http://mxr.mozilla.org/mozilla-central/source/toolkit/devtools/server/actors/
mod actors {
    pub mod console;
    pub mod inspector;
    pub mod root;
    pub mod tab;
}
mod protocol;

/// Spin up a devtools server that listens for connections. Defaults to port 6000.
/// TODO: allow specifying a port
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

    let root = box RootActor {
        tabs: vec!(),
    };

    registry.register(root);
    registry.find::<RootActor>("root");

    let actors = Arc::new(Mutex::new(registry));

    /// Process the input from a single devtools client until EOF.
    fn handle_client(actors: Arc<Mutex<ActorRegistry>>, mut stream: TcpStream) {
        println!("connection established to {:?}", stream.peer_name().unwrap());

        {
            let mut actors = actors.lock();
            let msg = actors.find::<RootActor>("root").encodable();
            stream.write_json_packet(&msg);
        }

        // https://wiki.mozilla.org/Remote_Debugging_Protocol_Stream_Transport
        // In short, each JSON packet is [ascii length]:[JSON data of given length]
        // TODO: this really belongs in the protocol module.
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

    // We need separate actor representations for each script global that exists;
    // clients can theoretically connect to multiple globals simultaneously.
    // TODO: move this into the root or tab modules?
    fn handle_new_global(actors: Arc<Mutex<ActorRegistry>>,
                         pipeline: PipelineId,
                         sender: Sender<DevtoolScriptControlMsg>) {
        let mut actors = actors.lock();

        //TODO: move all this actor creation into a constructor method on TabActor
        let (tab, console, inspector) = {
            let console = ConsoleActor {
                name: actors.new_name("console"),
                script_chan: sender.clone(),
                pipeline: pipeline,
            };
            let inspector = InspectorActor {
                name: actors.new_name("inspector"),
                walker: RefCell::new(None),
                pageStyle: RefCell::new(None),
                highlighter: RefCell::new(None),
                script_chan: sender,
                pipeline: pipeline,
            };
            //TODO: send along the current page title and URL
            let tab = TabActor {
                name: actors.new_name("tab"),
                title: "".to_string(),
                url: "about:blank".to_string(),
                console: console.name(),
                inspector: inspector.name(),
            };

            let root = actors.find_mut::<RootActor>("root");
            root.tabs.push(tab.name.clone());
            (tab, console, inspector)
        };

        actors.register(box tab);
        actors.register(box console);
        actors.register(box inspector);
    }

    //TODO: figure out some system that allows us to watch for new connections,
    //      shut down existing ones at arbitrary times, and also watch for messages
    //      from multiple script tasks simultaneously. Polling for new connections
    //      for 300ms and then checking the receiver is not a good compromise
    //      (and makes Servo hang on exit if there's an open connection, no less).

    //TODO: make constellation send ServerExitMsg on shutdown.

    // accept connections and process them, spawning a new tasks for each one
    loop {
        match acceptor.accept() {
            Err(ref e) if e.kind == TimedOut => {
                match port.try_recv() {
                    Ok(ServerExitMsg) | Err(Disconnected) => break,
                    Ok(NewGlobal(id, sender)) => handle_new_global(actors.clone(), id, sender),
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

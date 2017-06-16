/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! An actor-based remote devtools server implementation. Only tested with
//! nightly Firefox versions at time of writing. Largely based on
//! reverse-engineering of Firefox chrome devtool logs and reading of
//! [code](http://mxr.mozilla.org/mozilla-central/source/toolkit/devtools/server/).

#![crate_name = "devtools"]
#![crate_type = "rlib"]

#![allow(non_snake_case)]
#![deny(unsafe_code)]
#![feature(box_syntax)]

extern crate devtools_traits;
extern crate hyper;
extern crate ipc_channel;
#[macro_use]
extern crate log;
extern crate msg;
#[macro_use] extern crate serde;
extern crate serde_json;
extern crate time;

use actor::{Actor, ActorRegistry};
use actors::console::ConsoleActor;
use actors::framerate::FramerateActor;
use actors::inspector::InspectorActor;
use actors::network_event::{EventActor, NetworkEventActor, ResponseStartMsg};
use actors::performance::PerformanceActor;
use actors::profiler::ProfilerActor;
use actors::root::RootActor;
use actors::tab::TabActor;
use actors::thread::ThreadActor;
use actors::timeline::TimelineActor;
use actors::worker::WorkerActor;
use devtools_traits::{ChromeToDevtoolsControlMsg, ConsoleMessage, DevtoolsControlMsg};
use devtools_traits::{DevtoolScriptControlMsg, DevtoolsPageInfo, LogLevel, NetworkEvent};
use devtools_traits::{ScriptToDevtoolsControlMsg, WorkerId};
use ipc_channel::ipc::IpcSender;
use msg::constellation_msg::PipelineId;
use protocol::JsonPacketStream;
use std::borrow::ToOwned;
use std::cell::RefCell;
use std::collections::HashMap;
use std::collections::hash_map::Entry::{Occupied, Vacant};
use std::net::{Shutdown, TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::sync::mpsc::{Receiver, Sender, channel};
use std::thread;
use time::precise_time_ns;

mod actor;
/// Corresponds to http://mxr.mozilla.org/mozilla-central/source/toolkit/devtools/server/actors/
mod actors {
    pub mod console;
    pub mod framerate;
    pub mod inspector;
    pub mod memory;
    pub mod network_event;
    pub mod object;
    pub mod performance;
    pub mod profiler;
    pub mod root;
    pub mod tab;
    pub mod thread;
    pub mod timeline;
    pub mod worker;
}
mod protocol;

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
struct NetworkEventMsg {
    from: String,
    #[serde(rename = "type")]
    type_: String,
    eventActor: EventActor,
}

#[derive(Serialize)]
struct NetworkEventUpdateMsg {
    from: String,
    #[serde(rename = "type")]
    type_: String,
    updateType: String,
}

#[derive(Serialize)]
struct EventTimingsUpdateMsg {
    totalTime: u64,
}

#[derive(Serialize)]
struct SecurityInfoUpdateMsg {
    state: String,
}

#[derive(Serialize)]
struct ResponseStartUpdateMsg {
    from: String,
    #[serde(rename = "type")]
    type_: String,
    updateType: String,
    response: ResponseStartMsg,
}

/// Spin up a devtools server that listens for connections on the specified port.
pub fn start_server(port: u16) -> Sender<DevtoolsControlMsg> {
    let (sender, receiver) = channel();
    {
        let sender = sender.clone();
        thread::Builder::new().name("Devtools".to_owned()).spawn(move || {
            run_server(sender, receiver, port)
        }).expect("Thread spawning failed");
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

    let actors = registry.create_shareable();

    let mut accepted_connections: Vec<TcpStream> = Vec::new();

    let mut actor_pipelines: HashMap<PipelineId, String> = HashMap::new();
    let mut actor_requests: HashMap<String, String> = HashMap::new();

    let mut actor_workers: HashMap<(PipelineId, WorkerId), String> = HashMap::new();


    /// Process the input from a single devtools client until EOF.
    fn handle_client(actors: Arc<Mutex<ActorRegistry>>, mut stream: TcpStream) {
        debug!("connection established to {}", stream.peer_addr().unwrap());
        {
            let actors = actors.lock().unwrap();
            let msg = actors.find::<RootActor>("root").encodable();
            stream.write_json_packet(&msg);
        }

        'outer: loop {
            match stream.read_json_packet() {
                Ok(Some(json_packet)) => {
                    if let Err(()) = actors.lock().unwrap().handle_message(json_packet.as_object().unwrap(),
                                                                           &mut stream) {
                        debug!("error: devtools actor stopped responding");
                        let _ = stream.shutdown(Shutdown::Both);
                        break 'outer
                    }
                }
                Ok(None) => {
                    debug!("error: EOF");
                    break 'outer
                }
                Err(err_msg) => {
                    debug!("error: {}", err_msg);
                    break 'outer
                }
            }
        }
    }

    fn handle_framerate_tick(actors: Arc<Mutex<ActorRegistry>>, actor_name: String, tick: f64) {
        let mut actors = actors.lock().unwrap();
        let framerate_actor = actors.find_mut::<FramerateActor>(&actor_name);
        framerate_actor.add_tick(tick);
    }

    // We need separate actor representations for each script global that exists;
    // clients can theoretically connect to multiple globals simultaneously.
    // TODO: move this into the root or tab modules?
    fn handle_new_global(actors: Arc<Mutex<ActorRegistry>>,
                         ids: (PipelineId, Option<WorkerId>),
                         script_sender: IpcSender<DevtoolScriptControlMsg>,
                         actor_pipelines: &mut HashMap<PipelineId, String>,
                         actor_workers: &mut HashMap<(PipelineId, WorkerId), String>,
                         page_info: DevtoolsPageInfo) {
        let mut actors = actors.lock().unwrap();

        let (pipeline, worker_id) = ids;

        //TODO: move all this actor creation into a constructor method on TabActor
        let (tab, console, inspector, timeline, profiler, performance, thread) = {
            let console = ConsoleActor {
                name: actors.new_name("console"),
                script_chan: script_sender.clone(),
                pipeline: pipeline,
                streams: RefCell::new(Vec::new()),
            };
            let inspector = InspectorActor {
                name: actors.new_name("inspector"),
                walker: RefCell::new(None),
                pageStyle: RefCell::new(None),
                highlighter: RefCell::new(None),
                script_chan: script_sender.clone(),
                pipeline: pipeline,
            };

            let timeline = TimelineActor::new(actors.new_name("timeline"),
                                              pipeline,
                                              script_sender);

            let profiler = ProfilerActor::new(actors.new_name("profiler"));
            let performance = PerformanceActor::new(actors.new_name("performance"));

            let thread = ThreadActor::new(actors.new_name("context"));

            let DevtoolsPageInfo { title, url } = page_info;
            let tab = TabActor {
                name: actors.new_name("tab"),
                title: String::from(title),
                url: url.into_string(),
                console: console.name(),
                inspector: inspector.name(),
                timeline: timeline.name(),
                profiler: profiler.name(),
                performance: performance.name(),
                thread: thread.name(),
            };

            let root = actors.find_mut::<RootActor>("root");
            root.tabs.push(tab.name.clone());

            (tab, console, inspector, timeline, profiler, performance, thread)
        };

        if let Some(id) = worker_id {
            let worker = WorkerActor {
                name: actors.new_name("worker"),
                console: console.name(),
                id: id,
            };
            actor_workers.insert((pipeline, id), worker.name.clone());
            actors.register(box worker);
        }

        actor_pipelines.insert(pipeline, tab.name.clone());
        actors.register(box tab);
        actors.register(box console);
        actors.register(box inspector);
        actors.register(box timeline);
        actors.register(box profiler);
        actors.register(box performance);
        actors.register(box thread);
    }

    fn handle_console_message(actors: Arc<Mutex<ActorRegistry>>,
                              id: PipelineId,
                              worker_id: Option<WorkerId>,
                              console_message: ConsoleMessage,
                              actor_pipelines: &HashMap<PipelineId, String>,
                              actor_workers: &HashMap<(PipelineId, WorkerId), String>) {
        let console_actor_name = match find_console_actor(actors.clone(), id, worker_id, actor_workers,
                                                          actor_pipelines) {
            Some(name) => name,
            None => return,
        };
        let actors = actors.lock().unwrap();
        let console_actor = actors.find::<ConsoleActor>(&console_actor_name);
        let msg = ConsoleAPICall {
            from: console_actor.name.clone(),
            type_: "consoleAPICall".to_owned(),
            message: ConsoleMsg {
                level: match console_message.logLevel {
                    LogLevel::Debug => "debug",
                    LogLevel::Info => "info",
                    LogLevel::Warn => "warn",
                    LogLevel::Error => "error",
                    _ => "log"
                }.to_owned(),
                timeStamp: precise_time_ns(),
                arguments: vec!(console_message.message),
                filename: console_message.filename,
                lineNumber: console_message.lineNumber,
                columnNumber: console_message.columnNumber,
            },
        };
        for mut stream in &mut *console_actor.streams.borrow_mut() {
            stream.write_json_packet(&msg);
        }
    }

    fn find_console_actor(actors: Arc<Mutex<ActorRegistry>>,
                          id: PipelineId,
                          worker_id: Option<WorkerId>,
                          actor_workers: &HashMap<(PipelineId, WorkerId), String>,
                          actor_pipelines: &HashMap<PipelineId, String>) -> Option<String> {
        let actors = actors.lock().unwrap();
        if let Some(worker_id) = worker_id {
            let actor_name = match (*actor_workers).get(&(id, worker_id)) {
                Some(name) => name,
                None => return None,
            };
            Some(actors.find::<WorkerActor>(actor_name).console.clone())
        } else {
            let actor_name = match (*actor_pipelines).get(&id) {
                Some(name) => name,
                None => return None,
            };
            Some(actors.find::<TabActor>(actor_name).console.clone())
        }
    }

    fn handle_network_event(actors: Arc<Mutex<ActorRegistry>>,
                            mut connections: Vec<TcpStream>,
                            actor_pipelines: &HashMap<PipelineId, String>,
                            actor_requests: &mut HashMap<String, String>,
                            actor_workers: &HashMap<(PipelineId, WorkerId), String>,
                            pipeline_id: PipelineId,
                            request_id: String,
                            network_event: NetworkEvent) {
        let console_actor_name = match find_console_actor(actors.clone(), pipeline_id, None,
                                                          actor_workers, actor_pipelines) {
            Some(name) => name,
            None => return,
        };
        let netevent_actor_name = find_network_event_actor(actors.clone(), actor_requests, request_id.clone());
        let mut actors = actors.lock().unwrap();
        let actor = actors.find_mut::<NetworkEventActor>(&netevent_actor_name);

        match network_event {
            NetworkEvent::HttpRequest(httprequest) => {
                //Store the request information in the actor
                actor.add_request(httprequest);

                //Send a networkEvent message to the client
                let msg = NetworkEventMsg {
                    from: console_actor_name,
                    type_: "networkEvent".to_owned(),
                    eventActor: actor.event_actor(),
                };
                for stream in &mut connections {
                    stream.write_json_packet(&msg);
                }

            }
            NetworkEvent::HttpResponse(httpresponse) => {
                //Store the response information in the actor
                actor.add_response(httpresponse);

                let msg = NetworkEventUpdateMsg {
                    from: netevent_actor_name.clone(),
                    type_: "networkEventUpdate".to_owned(),
                    updateType: "requestHeaders".to_owned(),
                };
                for stream in &mut connections {
                    stream.write_merged_json_packet(&msg, &actor.request_headers());
                }

                let msg = NetworkEventUpdateMsg {
                    from: netevent_actor_name.clone(),
                    type_: "networkEventUpdate".to_owned(),
                    updateType: "requestCookies".to_owned(),
                };
                for stream in &mut connections {
                    stream.write_merged_json_packet(&msg, &actor.request_cookies());
                }

                //Send a networkEventUpdate (responseStart) to the client
                let msg = ResponseStartUpdateMsg {
                    from: netevent_actor_name.clone(),
                    type_: "networkEventUpdate".to_owned(),
                    updateType: "responseStart".to_owned(),
                    response: actor.response_start()
                };

                for stream in &mut connections {
                    stream.write_json_packet(&msg);
                }
                let msg = NetworkEventUpdateMsg {
                    from: netevent_actor_name.clone(),
                    type_: "networkEventUpdate".to_owned(),
                    updateType: "eventTimings".to_owned(),
                };
                let extra = EventTimingsUpdateMsg {
                    totalTime: actor.total_time(),
                };
                for stream in &mut connections {
                    stream.write_merged_json_packet(&msg, &extra);
                }

                let msg = NetworkEventUpdateMsg {
                    from: netevent_actor_name.clone(),
                    type_: "networkEventUpdate".to_owned(),
                    updateType: "securityInfo".to_owned(),
                };
                let extra = SecurityInfoUpdateMsg {
                    state: "insecure".to_owned(),
                };
                for stream in &mut connections {
                    stream.write_merged_json_packet(&msg, &extra);
                }

                let msg = NetworkEventUpdateMsg {
                    from: netevent_actor_name.clone(),
                    type_: "networkEventUpdate".to_owned(),
                    updateType: "responseContent".to_owned(),
                };
                for stream in &mut connections {
                    stream.write_merged_json_packet(&msg, &actor.response_content());
                }

                let msg = NetworkEventUpdateMsg {
                    from: netevent_actor_name.clone(),
                    type_: "networkEventUpdate".to_owned(),
                    updateType: "responseCookies".to_owned(),
                };
                for stream in &mut connections {
                    stream.write_merged_json_packet(&msg, &actor.response_cookies());
                }

                let msg = NetworkEventUpdateMsg {
                    from: netevent_actor_name.clone(),
                    type_: "networkEventUpdate".to_owned(),
                    updateType: "responseHeaders".to_owned(),
                };
                for stream in &mut connections {
                    stream.write_merged_json_packet(&msg, &actor.response_headers());
                }
            }
        }
    }

    // Find the name of NetworkEventActor corresponding to request_id
    // Create a new one if it does not exist, add it to the actor_requests hashmap
    fn find_network_event_actor(actors: Arc<Mutex<ActorRegistry>>,
                                actor_requests: &mut HashMap<String, String>,
                                request_id: String) -> String {
        let mut actors = actors.lock().unwrap();
        match (*actor_requests).entry(request_id) {
            Occupied(name) => {
                //TODO: Delete from map like Firefox does?
                name.into_mut().clone()
            }
            Vacant(entry) => {
                let actor_name = actors.new_name("netevent");
                let actor = NetworkEventActor::new(actor_name.clone());
                entry.insert(actor_name.clone());
                actors.register(box actor);
                actor_name
            }
        }
    }

    let sender_clone = sender.clone();
    thread::Builder::new().name("DevtoolsClientAcceptor".to_owned()).spawn(move || {
        // accept connections and process them, spawning a new thread for each one
        for stream in listener.incoming() {
            // connection succeeded
            sender_clone.send(DevtoolsControlMsg::FromChrome(
                    ChromeToDevtoolsControlMsg::AddClient(stream.unwrap()))).unwrap();
        }
    }).expect("Thread spawning failed");

    while let Ok(msg) = receiver.recv() {
        match msg {
            DevtoolsControlMsg::FromChrome(ChromeToDevtoolsControlMsg::AddClient(stream)) => {
                let actors = actors.clone();
                accepted_connections.push(stream.try_clone().unwrap());
                thread::Builder::new().name("DevtoolsClientHandler".to_owned()).spawn(move || {
                    handle_client(actors, stream.try_clone().unwrap())
                }).expect("Thread spawning failed");
            }
            DevtoolsControlMsg::FromScript(ScriptToDevtoolsControlMsg::FramerateTick(
                        actor_name, tick)) =>
                handle_framerate_tick(actors.clone(), actor_name, tick),
            DevtoolsControlMsg::FromScript(ScriptToDevtoolsControlMsg::NewGlobal(
                        ids, script_sender, pageinfo)) =>
                handle_new_global(actors.clone(), ids, script_sender, &mut actor_pipelines,
                                  &mut actor_workers, pageinfo),
            DevtoolsControlMsg::FromScript(ScriptToDevtoolsControlMsg::ConsoleAPI(
                        id,
                        console_message,
                        worker_id)) =>
                handle_console_message(actors.clone(), id, worker_id, console_message,
                                       &actor_pipelines, &actor_workers),
            DevtoolsControlMsg::FromScript(ScriptToDevtoolsControlMsg::ReportCSSError(
                        id,
                        css_error)) => {
                let console_message =  ConsoleMessage {
                    message: css_error.msg,
                    logLevel: LogLevel::Warn,
                    filename: css_error.filename,
                    lineNumber: css_error.line as usize,
                    columnNumber: css_error.column as usize,
                };
                handle_console_message(actors.clone(), id, None, console_message,
                                       &actor_pipelines, &actor_workers)
            },
            DevtoolsControlMsg::FromChrome(ChromeToDevtoolsControlMsg::NetworkEvent(
                        request_id, network_event)) => {
                // copy the accepted_connections vector
                let mut connections = Vec::<TcpStream>::new();
                for stream in &accepted_connections {
                    connections.push(stream.try_clone().unwrap());
                }

                let pipeline_id = match network_event {
                    NetworkEvent::HttpResponse(ref response) => response.pipeline_id,
                    NetworkEvent::HttpRequest(ref request) => request.pipeline_id,
                };
                handle_network_event(actors.clone(), connections, &actor_pipelines, &mut actor_requests,
                                     &actor_workers, pipeline_id, request_id, network_event);
            },
            DevtoolsControlMsg::FromChrome(ChromeToDevtoolsControlMsg::ServerExitMsg) => break
        }
    }
    for connection in &mut accepted_connections {
        let _ = connection.shutdown(Shutdown::Both);
    }
}

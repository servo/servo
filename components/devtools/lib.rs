/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! An actor-based remote devtools server implementation. Only tested with
//! nightly Firefox versions at time of writing. Largely based on
//! reverse-engineering of Firefox chrome devtool logs and reading of
//! [code](http://mxr.mozilla.org/mozilla-central/source/toolkit/devtools/server/).

#![crate_name = "devtools"]
#![crate_type = "rlib"]
#![allow(non_snake_case)]
#![deny(unsafe_code)]

#[macro_use]
extern crate log;
#[macro_use]
extern crate serde;

use crate::actor::{Actor, ActorRegistry};
use crate::actors::browsing_context::BrowsingContextActor;
use crate::actors::console::ConsoleActor;
use crate::actors::device::DeviceActor;
use crate::actors::framerate::FramerateActor;
use crate::actors::network_event::{EventActor, NetworkEventActor, ResponseStartMsg};
use crate::actors::performance::PerformanceActor;
use crate::actors::preference::PreferenceActor;
use crate::actors::process::ProcessActor;
use crate::actors::root::RootActor;
use crate::actors::worker::WorkerActor;
use crate::protocol::JsonPacketStream;
use crossbeam_channel::{unbounded, Receiver, Sender};
use devtools_traits::{ChromeToDevtoolsControlMsg, ConsoleMessage, DevtoolsControlMsg};
use devtools_traits::{
    DevtoolScriptControlMsg, DevtoolsPageInfo, LogLevel, NavigationState, NetworkEvent,
};
use devtools_traits::{PageError, ScriptToDevtoolsControlMsg, WorkerId};
use embedder_traits::{EmbedderMsg, EmbedderProxy, PromptDefinition, PromptOrigin, PromptResult};
use ipc_channel::ipc::{self, IpcSender};
use msg::constellation_msg::{BrowsingContextId, PipelineId};
use std::borrow::ToOwned;
use std::collections::hash_map::Entry::{Occupied, Vacant};
use std::collections::HashMap;
use std::net::{Shutdown, TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread;

mod actor;
/// Corresponds to http://mxr.mozilla.org/mozilla-central/source/toolkit/devtools/server/actors/
mod actors {
    pub mod browsing_context;
    pub mod console;
    pub mod device;
    pub mod emulation;
    pub mod framerate;
    pub mod inspector;
    pub mod memory;
    pub mod network_event;
    pub mod object;
    pub mod performance;
    pub mod preference;
    pub mod process;
    pub mod profiler;
    pub mod root;
    pub mod stylesheets;
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
struct PageErrorMsg {
    from: String,
    #[serde(rename = "type")]
    type_: String,
    pageError: PageError,
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
pub fn start_server(port: u16, embedder: EmbedderProxy) -> Sender<DevtoolsControlMsg> {
    let (sender, receiver) = unbounded();
    {
        let sender = sender.clone();
        thread::Builder::new()
            .name("Devtools".to_owned())
            .spawn(move || run_server(sender, receiver, port, embedder))
            .expect("Thread spawning failed");
    }
    sender
}

fn run_server(
    sender: Sender<DevtoolsControlMsg>,
    receiver: Receiver<DevtoolsControlMsg>,
    port: u16,
    embedder: EmbedderProxy,
) {
    let bound = TcpListener::bind(&("0.0.0.0", port)).ok().and_then(|l| {
        l.local_addr()
            .map(|addr| addr.port())
            .ok()
            .map(|port| (l, port))
    });

    let port = bound.as_ref().map(|(_, port)| *port).ok_or(());
    embedder.send((None, EmbedderMsg::OnDevtoolsStarted(port)));

    let listener = match bound {
        Some((l, _)) => l,
        None => return,
    };

    let mut registry = ActorRegistry::new();

    let performance = PerformanceActor::new(registry.new_name("performance"));

    let device = DeviceActor::new(registry.new_name("device"));

    let preference = PreferenceActor::new(registry.new_name("preference"));

    let process = ProcessActor::new(registry.new_name("process"));

    let root = Box::new(RootActor {
        tabs: vec![],
        device: device.name(),
        performance: performance.name(),
        preference: preference.name(),
        process: process.name(),
    });

    registry.register(root);
    registry.register(Box::new(performance));
    registry.register(Box::new(device));
    registry.register(Box::new(preference));
    registry.register(Box::new(process));
    registry.find::<RootActor>("root");

    let actors = registry.create_shareable();

    let mut accepted_connections: Vec<TcpStream> = Vec::new();

    let mut browsing_contexts: HashMap<BrowsingContextId, String> = HashMap::new();
    let mut pipelines: HashMap<PipelineId, BrowsingContextId> = HashMap::new();
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
                    if let Err(()) = actors
                        .lock()
                        .unwrap()
                        .handle_message(json_packet.as_object().unwrap(), &mut stream)
                    {
                        debug!("error: devtools actor stopped responding");
                        let _ = stream.shutdown(Shutdown::Both);
                        break 'outer;
                    }
                },
                Ok(None) => {
                    debug!("error: EOF");
                    break 'outer;
                },
                Err(err_msg) => {
                    debug!("error: {}", err_msg);
                    break 'outer;
                },
            }
        }
    }

    fn handle_framerate_tick(actors: Arc<Mutex<ActorRegistry>>, actor_name: String, tick: f64) {
        let mut actors = actors.lock().unwrap();
        let framerate_actor = actors.find_mut::<FramerateActor>(&actor_name);
        framerate_actor.add_tick(tick);
    }

    fn handle_navigate(
        actors: Arc<Mutex<ActorRegistry>>,
        browsing_contexts: &HashMap<BrowsingContextId, String>,
        browsing_context: BrowsingContextId,
        state: NavigationState,
    ) {
        let actor_name = browsing_contexts.get(&browsing_context).unwrap();
        actors
            .lock()
            .unwrap()
            .find::<BrowsingContextActor>(actor_name)
            .navigate(state);
    }

    // We need separate actor representations for each script global that exists;
    // clients can theoretically connect to multiple globals simultaneously.
    // TODO: move this into the root or target modules?
    fn handle_new_global(
        actors: Arc<Mutex<ActorRegistry>>,
        ids: (Option<BrowsingContextId>, PipelineId, Option<WorkerId>),
        script_sender: IpcSender<DevtoolScriptControlMsg>,
        browsing_contexts: &mut HashMap<BrowsingContextId, String>,
        pipelines: &mut HashMap<PipelineId, BrowsingContextId>,
        actor_workers: &mut HashMap<(PipelineId, WorkerId), String>,
        page_info: DevtoolsPageInfo,
    ) {
        let mut actors = actors.lock().unwrap();

        let (browsing_context, pipeline, worker_id) = ids;

        let console_name = actors.new_name("console");

        let browsing_context_name = if let Some(browsing_context) = browsing_context {
            pipelines.insert(pipeline, browsing_context);
            if let Some(actor) = browsing_contexts.get(&browsing_context) {
                actor.to_owned()
            } else {
                let browsing_context_actor = BrowsingContextActor::new(
                    console_name.clone(),
                    browsing_context,
                    page_info,
                    pipeline,
                    script_sender.clone(),
                    &mut *actors,
                );
                let name = browsing_context_actor.name();
                browsing_contexts.insert(browsing_context, name.clone());
                actors.register(Box::new(browsing_context_actor));
                name
            }
        } else {
            "".to_owned()
        };

        // XXXjdm this new actor is useless if it's not a new worker global
        let console = ConsoleActor {
            name: console_name,
            cached_events: Default::default(),
            browsing_context: browsing_context_name,
        };

        if let Some(id) = worker_id {
            let worker = WorkerActor {
                name: actors.new_name("worker"),
                console: console.name(),
                id: id,
            };
            let root = actors.find_mut::<RootActor>("root");
            root.tabs.push(worker.name.clone());

            actor_workers.insert((pipeline, id), worker.name.clone());
            actors.register(Box::new(worker));
        }

        actors.register(Box::new(console));
    }

    fn handle_page_error(
        actors: Arc<Mutex<ActorRegistry>>,
        id: PipelineId,
        page_error: PageError,
        browsing_contexts: &HashMap<BrowsingContextId, String>,
        pipelines: &HashMap<PipelineId, BrowsingContextId>,
    ) {
        let console_actor_name = match find_console_actor(
            actors.clone(),
            id,
            None,
            &HashMap::new(),
            browsing_contexts,
            pipelines,
        ) {
            Some(name) => name,
            None => return,
        };
        let actors = actors.lock().unwrap();
        let console_actor = actors.find::<ConsoleActor>(&console_actor_name);
        let browsing_context_actor =
            actors.find::<BrowsingContextActor>(&console_actor.browsing_context);
        console_actor.handle_page_error(page_error, id, &browsing_context_actor);
    }

    fn handle_console_message(
        actors: Arc<Mutex<ActorRegistry>>,
        id: PipelineId,
        worker_id: Option<WorkerId>,
        console_message: ConsoleMessage,
        browsing_contexts: &HashMap<BrowsingContextId, String>,
        actor_workers: &HashMap<(PipelineId, WorkerId), String>,
        pipelines: &HashMap<PipelineId, BrowsingContextId>,
    ) {
        let console_actor_name = match find_console_actor(
            actors.clone(),
            id,
            worker_id,
            actor_workers,
            browsing_contexts,
            pipelines,
        ) {
            Some(name) => name,
            None => return,
        };
        let actors = actors.lock().unwrap();
        let console_actor = actors.find::<ConsoleActor>(&console_actor_name);
        let browsing_context_actor =
            actors.find::<BrowsingContextActor>(&console_actor.browsing_context);
        console_actor.handle_console_api(console_message, id, &browsing_context_actor);
    }

    fn find_console_actor(
        actors: Arc<Mutex<ActorRegistry>>,
        pipeline: PipelineId,
        worker_id: Option<WorkerId>,
        actor_workers: &HashMap<(PipelineId, WorkerId), String>,
        browsing_contexts: &HashMap<BrowsingContextId, String>,
        pipelines: &HashMap<PipelineId, BrowsingContextId>,
    ) -> Option<String> {
        let actors = actors.lock().unwrap();
        if let Some(worker_id) = worker_id {
            let actor_name = (*actor_workers).get(&(pipeline, worker_id))?;
            Some(actors.find::<WorkerActor>(actor_name).console.clone())
        } else {
            let id = pipelines.get(&pipeline)?;
            let actor_name = browsing_contexts.get(id)?;
            Some(
                actors
                    .find::<BrowsingContextActor>(actor_name)
                    .console
                    .clone(),
            )
        }
    }

    fn handle_network_event(
        actors: Arc<Mutex<ActorRegistry>>,
        mut connections: Vec<TcpStream>,
        browsing_contexts: &HashMap<BrowsingContextId, String>,
        actor_requests: &mut HashMap<String, String>,
        actor_workers: &HashMap<(PipelineId, WorkerId), String>,
        pipelines: &HashMap<PipelineId, BrowsingContextId>,
        pipeline_id: PipelineId,
        request_id: String,
        network_event: NetworkEvent,
    ) {
        let console_actor_name = match find_console_actor(
            actors.clone(),
            pipeline_id,
            None,
            actor_workers,
            browsing_contexts,
            pipelines,
        ) {
            Some(name) => name,
            None => return,
        };
        let netevent_actor_name =
            find_network_event_actor(actors.clone(), actor_requests, request_id);
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
            },
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
                    response: actor.response_start(),
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
                    from: netevent_actor_name,
                    type_: "networkEventUpdate".to_owned(),
                    updateType: "responseHeaders".to_owned(),
                };
                for stream in &mut connections {
                    stream.write_merged_json_packet(&msg, &actor.response_headers());
                }
            },
        }
    }

    // Find the name of NetworkEventActor corresponding to request_id
    // Create a new one if it does not exist, add it to the actor_requests hashmap
    fn find_network_event_actor(
        actors: Arc<Mutex<ActorRegistry>>,
        actor_requests: &mut HashMap<String, String>,
        request_id: String,
    ) -> String {
        let mut actors = actors.lock().unwrap();
        match (*actor_requests).entry(request_id) {
            Occupied(name) => {
                //TODO: Delete from map like Firefox does?
                name.into_mut().clone()
            },
            Vacant(entry) => {
                let actor_name = actors.new_name("netevent");
                let actor = NetworkEventActor::new(actor_name.clone());
                entry.insert(actor_name.clone());
                actors.register(Box::new(actor));
                actor_name
            },
        }
    }

    thread::Builder::new()
        .name("DevtoolsClientAcceptor".to_owned())
        .spawn(move || {
            // accept connections and process them, spawning a new thread for each one
            for stream in listener.incoming() {
                // Prompt user for permission
                let (embedder_sender, receiver) =
                    ipc::channel().expect("Failed to create IPC channel!");
                let message = "Accept incoming devtools connection?".to_owned();
                let prompt = PromptDefinition::YesNo(message, embedder_sender);
                let msg = EmbedderMsg::Prompt(prompt, PromptOrigin::Trusted);
                embedder.send((None, msg));
                if receiver.recv().unwrap() != PromptResult::Primary {
                    continue;
                }
                // connection succeeded and accepted
                sender
                    .send(DevtoolsControlMsg::FromChrome(
                        ChromeToDevtoolsControlMsg::AddClient(stream.unwrap()),
                    ))
                    .unwrap();
            }
        })
        .expect("Thread spawning failed");

    while let Ok(msg) = receiver.recv() {
        match msg {
            DevtoolsControlMsg::FromChrome(ChromeToDevtoolsControlMsg::AddClient(stream)) => {
                let actors = actors.clone();
                accepted_connections.push(stream.try_clone().unwrap());
                thread::Builder::new()
                    .name("DevtoolsClientHandler".to_owned())
                    .spawn(move || handle_client(actors, stream.try_clone().unwrap()))
                    .expect("Thread spawning failed");
            },
            DevtoolsControlMsg::FromScript(ScriptToDevtoolsControlMsg::FramerateTick(
                actor_name,
                tick,
            )) => handle_framerate_tick(actors.clone(), actor_name, tick),
            DevtoolsControlMsg::FromScript(ScriptToDevtoolsControlMsg::NewGlobal(
                ids,
                script_sender,
                pageinfo,
            )) => handle_new_global(
                actors.clone(),
                ids,
                script_sender,
                &mut browsing_contexts,
                &mut pipelines,
                &mut actor_workers,
                pageinfo,
            ),
            DevtoolsControlMsg::FromScript(ScriptToDevtoolsControlMsg::Navigate(
                browsing_context,
                state,
            )) => handle_navigate(actors.clone(), &browsing_contexts, browsing_context, state),
            DevtoolsControlMsg::FromScript(ScriptToDevtoolsControlMsg::ConsoleAPI(
                id,
                console_message,
                worker_id,
            )) => handle_console_message(
                actors.clone(),
                id,
                worker_id,
                console_message,
                &browsing_contexts,
                &actor_workers,
                &pipelines,
            ),
            DevtoolsControlMsg::FromScript(ScriptToDevtoolsControlMsg::ReportPageError(
                id,
                page_error,
            )) => handle_page_error(
                actors.clone(),
                id,
                page_error,
                &browsing_contexts,
                &pipelines,
            ),
            DevtoolsControlMsg::FromScript(ScriptToDevtoolsControlMsg::ReportCSSError(
                id,
                css_error,
            )) => {
                let console_message = ConsoleMessage {
                    message: css_error.msg,
                    logLevel: LogLevel::Warn,
                    filename: css_error.filename,
                    lineNumber: css_error.line as usize,
                    columnNumber: css_error.column as usize,
                };
                handle_console_message(
                    actors.clone(),
                    id,
                    None,
                    console_message,
                    &browsing_contexts,
                    &actor_workers,
                    &pipelines,
                )
            },
            DevtoolsControlMsg::FromChrome(ChromeToDevtoolsControlMsg::NetworkEvent(
                request_id,
                network_event,
            )) => {
                // copy the accepted_connections vector
                let mut connections = Vec::<TcpStream>::new();
                for stream in &accepted_connections {
                    connections.push(stream.try_clone().unwrap());
                }

                let pipeline_id = match network_event {
                    NetworkEvent::HttpResponse(ref response) => response.pipeline_id,
                    NetworkEvent::HttpRequest(ref request) => request.pipeline_id,
                };
                handle_network_event(
                    actors.clone(),
                    connections,
                    &browsing_contexts,
                    &mut actor_requests,
                    &actor_workers,
                    &pipelines,
                    pipeline_id,
                    request_id,
                    network_event,
                );
            },
            DevtoolsControlMsg::FromChrome(ChromeToDevtoolsControlMsg::ServerExitMsg) => break,
        }
    }
    for connection in &mut accepted_connections {
        let _ = connection.shutdown(Shutdown::Both);
    }
}

/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! An actor-based remote devtools server implementation. Only tested with
//! nightly Firefox versions at time of writing. Largely based on
//! reverse-engineering of Firefox chrome devtool logs and reading of
//! [code](https://searchfox.org/mozilla-central/source/devtools/server).

#![crate_name = "devtools"]
#![crate_type = "rlib"]
#![deny(unsafe_code)]

use std::borrow::ToOwned;
use std::collections::hash_map::Entry::{Occupied, Vacant};
use std::collections::HashMap;
use std::io::Read;
use std::net::{Shutdown, TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread;

use base::id::{BrowsingContextId, PipelineId};
use crossbeam_channel::{unbounded, Receiver, Sender};
use devtools_traits::{
    ChromeToDevtoolsControlMsg, ConsoleMessage, DevtoolScriptControlMsg, DevtoolsControlMsg,
    DevtoolsPageInfo, LogLevel, NavigationState, NetworkEvent, PageError,
    ScriptToDevtoolsControlMsg, WorkerId,
};
use embedder_traits::{EmbedderMsg, EmbedderProxy, PromptDefinition, PromptOrigin, PromptResult};
use ipc_channel::ipc::{self, IpcSender};
use log::{debug, warn};
use serde::Serialize;
use servo_rand::RngCore;

use crate::actor::{Actor, ActorRegistry};
use crate::actors::browsing_context::BrowsingContextActor;
use crate::actors::console::{ConsoleActor, Root};
use crate::actors::device::DeviceActor;
use crate::actors::framerate::FramerateActor;
use crate::actors::network_event::{EventActor, NetworkEventActor, ResponseStartMsg};
use crate::actors::performance::PerformanceActor;
use crate::actors::preference::PreferenceActor;
use crate::actors::process::ProcessActor;
use crate::actors::root::RootActor;
use crate::actors::thread::ThreadActor;
use crate::actors::worker::{WorkerActor, WorkerType};
use crate::protocol::JsonPacketStream;

mod actor;
/// <https://searchfox.org/mozilla-central/source/devtools/server/actors>
mod actors {
    pub mod browsing_context;
    pub mod configuration;
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
    pub mod tab;
    pub mod thread;
    pub mod timeline;
    pub mod watcher;
    pub mod worker;
}
mod protocol;

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
enum UniqueId {
    Pipeline(PipelineId),
    Worker(WorkerId),
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct NetworkEventMsg {
    from: String,
    #[serde(rename = "type")]
    type_: String,
    event_actor: EventActor,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct NetworkEventUpdateMsg {
    from: String,
    #[serde(rename = "type")]
    type_: String,
    update_type: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct EventTimingsUpdateMsg {
    total_time: u64,
}

#[derive(Serialize)]
struct SecurityInfoUpdateMsg {
    state: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ResponseStartUpdateMsg {
    from: String,
    #[serde(rename = "type")]
    type_: String,
    update_type: String,
    response: ResponseStartMsg,
}

#[derive(Serialize)]
pub struct EmptyReplyMsg {
    pub from: String,
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

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(crate) struct StreamId(u32);

fn run_server(
    sender: Sender<DevtoolsControlMsg>,
    receiver: Receiver<DevtoolsControlMsg>,
    port: u16,
    embedder: EmbedderProxy,
) {
    let bound = TcpListener::bind(("0.0.0.0", port)).ok().and_then(|l| {
        l.local_addr()
            .map(|addr| addr.port())
            .ok()
            .map(|port| (l, port))
    });

    // A token shared with the embedder to bypass permission prompt.
    let token = format!("{:X}", servo_rand::ServoRng::default().next_u32());

    let port = bound.as_ref().map(|(_, port)| *port).ok_or(());
    embedder.send((None, EmbedderMsg::OnDevtoolsStarted(port, token.clone())));

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
        workers: vec![],
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

    let mut accepted_connections = HashMap::new();
    let mut browsing_contexts: HashMap<_, String> = HashMap::new();
    let mut pipelines = HashMap::new();
    let mut actor_requests = HashMap::new();
    let mut actor_workers = HashMap::new();

    /// Process the input from a single devtools client until EOF.
    fn handle_client(actors: Arc<Mutex<ActorRegistry>>, mut stream: TcpStream, id: StreamId) {
        debug!("connection established to {}", stream.peer_addr().unwrap());
        {
            let actors = actors.lock().unwrap();
            let msg = actors.find::<RootActor>("root").encodable();
            if let Err(e) = stream.write_json_packet(&msg) {
                warn!("Error writing response: {:?}", e);
                return;
            }
        }

        'outer: loop {
            match stream.read_json_packet() {
                Ok(Some(json_packet)) => {
                    if let Err(()) = actors.lock().unwrap().handle_message(
                        json_packet.as_object().unwrap(),
                        &mut stream,
                        id,
                    ) {
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

        actors.lock().unwrap().cleanup(id);
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

    fn handle_title_changed(
        actors: Arc<Mutex<ActorRegistry>>,
        pipelines: &HashMap<PipelineId, BrowsingContextId>,
        browsing_contexts: &HashMap<BrowsingContextId, String>,
        pipeline: PipelineId,
        title: String,
    ) {
        let bc = match pipelines.get(&pipeline) {
            Some(bc) => bc,
            None => return,
        };
        let name = match browsing_contexts.get(bc) {
            Some(name) => name,
            None => return,
        };
        let actors = actors.lock().unwrap();
        let browsing_context = actors.find::<BrowsingContextActor>(name);
        browsing_context.title_changed(pipeline, title);
    }

    // We need separate actor representations for each script global that exists;
    // clients can theoretically connect to multiple globals simultaneously.
    // TODO: move this into the root or target modules?
    fn handle_new_global(
        actors: Arc<Mutex<ActorRegistry>>,
        ids: (BrowsingContextId, PipelineId, Option<WorkerId>),
        script_sender: IpcSender<DevtoolScriptControlMsg>,
        browsing_contexts: &mut HashMap<BrowsingContextId, String>,
        pipelines: &mut HashMap<PipelineId, BrowsingContextId>,
        actor_workers: &mut HashMap<WorkerId, String>,
        page_info: DevtoolsPageInfo,
        connections: &HashMap<StreamId, TcpStream>,
    ) {
        let mut actors = actors.lock().unwrap();

        let (browsing_context, pipeline, worker_id) = ids;

        let console_name = actors.new_name("console");

        let parent_actor = if let Some(id) = worker_id {
            assert!(pipelines.get(&pipeline).is_some());
            assert!(browsing_contexts.get(&browsing_context).is_some());

            let thread = ThreadActor::new(actors.new_name("context"));
            let thread_name = thread.name();
            actors.register(Box::new(thread));

            let worker_name = actors.new_name("worker");
            let worker = WorkerActor {
                name: worker_name.clone(),
                console: console_name.clone(),
                thread: thread_name,
                id,
                url: page_info.url.clone(),
                type_: WorkerType::Dedicated,
                script_chan: script_sender,
                streams: Default::default(),
            };
            let root = actors.find_mut::<RootActor>("root");
            root.workers.push(worker.name.clone());

            actor_workers.insert(id, worker_name.clone());
            actors.register(Box::new(worker));

            Root::DedicatedWorker(worker_name)
        } else {
            pipelines.insert(pipeline, browsing_context);
            let name = browsing_contexts
                .entry(browsing_context)
                .or_insert_with(|| {
                    let browsing_context_actor = BrowsingContextActor::new(
                        console_name.clone(),
                        browsing_context,
                        page_info,
                        pipeline,
                        script_sender,
                        &mut actors,
                    );
                    let name = browsing_context_actor.name();
                    actors.register(Box::new(browsing_context_actor));
                    name
                });

            // Add existing streams to the new browsing context
            let browsing_context = actors.find::<BrowsingContextActor>(name);
            let mut streams = browsing_context.streams.borrow_mut();
            for (id, stream) in connections {
                streams.insert(*id, stream.try_clone().unwrap());
            }

            Root::BrowsingContext(name.clone())
        };

        let console = ConsoleActor {
            name: console_name,
            cached_events: Default::default(),
            root: parent_actor,
        };

        actors.register(Box::new(console));
    }

    fn handle_page_error(
        actors: Arc<Mutex<ActorRegistry>>,
        id: PipelineId,
        worker_id: Option<WorkerId>,
        page_error: PageError,
        browsing_contexts: &HashMap<BrowsingContextId, String>,
        actor_workers: &HashMap<WorkerId, String>,
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
        let id = worker_id.map_or(UniqueId::Pipeline(id), UniqueId::Worker);
        console_actor.handle_page_error(page_error, id, &actors);
    }

    fn handle_console_message(
        actors: Arc<Mutex<ActorRegistry>>,
        id: PipelineId,
        worker_id: Option<WorkerId>,
        console_message: ConsoleMessage,
        browsing_contexts: &HashMap<BrowsingContextId, String>,
        actor_workers: &HashMap<WorkerId, String>,
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
        let id = worker_id.map_or(UniqueId::Pipeline(id), UniqueId::Worker);
        console_actor.handle_console_api(console_message, id, &actors);
    }

    fn find_console_actor(
        actors: Arc<Mutex<ActorRegistry>>,
        pipeline: PipelineId,
        worker_id: Option<WorkerId>,
        actor_workers: &HashMap<WorkerId, String>,
        browsing_contexts: &HashMap<BrowsingContextId, String>,
        pipelines: &HashMap<PipelineId, BrowsingContextId>,
    ) -> Option<String> {
        let actors = actors.lock().unwrap();
        if let Some(worker_id) = worker_id {
            let actor_name = actor_workers.get(&worker_id)?;
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

    #[allow(clippy::too_many_arguments)]
    fn handle_network_event(
        actors: Arc<Mutex<ActorRegistry>>,
        mut connections: Vec<TcpStream>,
        browsing_contexts: &HashMap<BrowsingContextId, String>,
        actor_requests: &mut HashMap<String, String>,
        actor_workers: &HashMap<WorkerId, String>,
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
                    event_actor: actor.event_actor(),
                };
                for stream in &mut connections {
                    let _ = stream.write_json_packet(&msg);
                }
            },
            NetworkEvent::HttpResponse(httpresponse) => {
                //Store the response information in the actor
                actor.add_response(httpresponse);

                let msg = NetworkEventUpdateMsg {
                    from: netevent_actor_name.clone(),
                    type_: "networkEventUpdate".to_owned(),
                    update_type: "requestHeaders".to_owned(),
                };
                for stream in &mut connections {
                    let _ = stream.write_merged_json_packet(&msg, &actor.request_headers());
                }

                let msg = NetworkEventUpdateMsg {
                    from: netevent_actor_name.clone(),
                    type_: "networkEventUpdate".to_owned(),
                    update_type: "requestCookies".to_owned(),
                };
                for stream in &mut connections {
                    let _ = stream.write_merged_json_packet(&msg, &actor.request_cookies());
                }

                //Send a networkEventUpdate (responseStart) to the client
                let msg = ResponseStartUpdateMsg {
                    from: netevent_actor_name.clone(),
                    type_: "networkEventUpdate".to_owned(),
                    update_type: "responseStart".to_owned(),
                    response: actor.response_start(),
                };

                for stream in &mut connections {
                    let _ = stream.write_json_packet(&msg);
                }
                let msg = NetworkEventUpdateMsg {
                    from: netevent_actor_name.clone(),
                    type_: "networkEventUpdate".to_owned(),
                    update_type: "eventTimings".to_owned(),
                };
                let extra = EventTimingsUpdateMsg {
                    total_time: actor.total_time(),
                };
                for stream in &mut connections {
                    let _ = stream.write_merged_json_packet(&msg, &extra);
                }

                let msg = NetworkEventUpdateMsg {
                    from: netevent_actor_name.clone(),
                    type_: "networkEventUpdate".to_owned(),
                    update_type: "securityInfo".to_owned(),
                };
                let extra = SecurityInfoUpdateMsg {
                    state: "insecure".to_owned(),
                };
                for stream in &mut connections {
                    let _ = stream.write_merged_json_packet(&msg, &extra);
                }

                let msg = NetworkEventUpdateMsg {
                    from: netevent_actor_name.clone(),
                    type_: "networkEventUpdate".to_owned(),
                    update_type: "responseContent".to_owned(),
                };
                for stream in &mut connections {
                    let _ = stream.write_merged_json_packet(&msg, &actor.response_content());
                }

                let msg = NetworkEventUpdateMsg {
                    from: netevent_actor_name.clone(),
                    type_: "networkEventUpdate".to_owned(),
                    update_type: "responseCookies".to_owned(),
                };
                for stream in &mut connections {
                    let _ = stream.write_merged_json_packet(&msg, &actor.response_cookies());
                }

                let msg = NetworkEventUpdateMsg {
                    from: netevent_actor_name,
                    type_: "networkEventUpdate".to_owned(),
                    update_type: "responseHeaders".to_owned(),
                };
                for stream in &mut connections {
                    let _ = stream.write_merged_json_packet(&msg, &actor.response_headers());
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
        .name("DevtCliAcceptor".to_owned())
        .spawn(move || {
            // accept connections and process them, spawning a new thread for each one
            for stream in listener.incoming() {
                let mut stream = stream.expect("Can't retrieve stream");
                if !allow_devtools_client(&mut stream, &embedder, &token) {
                    continue;
                };
                // connection succeeded and accepted
                sender
                    .send(DevtoolsControlMsg::FromChrome(
                        ChromeToDevtoolsControlMsg::AddClient(stream),
                    ))
                    .unwrap();
            }
        })
        .expect("Thread spawning failed");

    let mut next_id = StreamId(0);
    while let Ok(msg) = receiver.recv() {
        debug!("{:?}", msg);
        match msg {
            DevtoolsControlMsg::FromChrome(ChromeToDevtoolsControlMsg::AddClient(stream)) => {
                let actors = actors.clone();
                let id = next_id;
                next_id = StreamId(id.0 + 1);
                accepted_connections.insert(id, stream.try_clone().unwrap());

                // Inform every browsing context of the new stream
                for name in browsing_contexts.values() {
                    let actors = actors.lock().unwrap();
                    let browsing_context = actors.find::<BrowsingContextActor>(name);
                    let mut streams = browsing_context.streams.borrow_mut();
                    streams.insert(id, stream.try_clone().unwrap());
                }
                thread::Builder::new()
                    .name("DevtoolsClientHandler".to_owned())
                    .spawn(move || handle_client(actors, stream.try_clone().unwrap(), id))
                    .expect("Thread spawning failed");
            },
            DevtoolsControlMsg::FromScript(ScriptToDevtoolsControlMsg::FramerateTick(
                actor_name,
                tick,
            )) => handle_framerate_tick(actors.clone(), actor_name, tick),
            DevtoolsControlMsg::FromScript(ScriptToDevtoolsControlMsg::TitleChanged(
                pipeline,
                title,
            )) => handle_title_changed(
                actors.clone(),
                &pipelines,
                &browsing_contexts,
                pipeline,
                title,
            ),
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
                &accepted_connections,
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
                None,
                page_error,
                &browsing_contexts,
                &actor_workers,
                &pipelines,
            ),
            DevtoolsControlMsg::FromScript(ScriptToDevtoolsControlMsg::ReportCSSError(
                id,
                css_error,
            )) => {
                let console_message = ConsoleMessage {
                    message: css_error.msg,
                    log_level: LogLevel::Warn,
                    filename: css_error.filename,
                    line_number: css_error.line as usize,
                    column_number: css_error.column as usize,
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
                for stream in accepted_connections.values() {
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
    for connection in accepted_connections.values_mut() {
        let _ = connection.shutdown(Shutdown::Both);
    }
}

fn allow_devtools_client(stream: &mut TcpStream, embedder: &EmbedderProxy, token: &str) -> bool {
    // By-pass prompt if we receive a valid token.
    let token = format!("25:{{\"auth_token\":\"{}\"}}", token);
    let mut buf = [0; 28];
    let timeout = std::time::Duration::from_millis(500);
    // This will read but not consume the bytes from the stream.
    stream.set_read_timeout(Some(timeout)).unwrap();
    let peek = stream.peek(&mut buf);
    stream.set_read_timeout(None).unwrap();
    if let Ok(len) = peek {
        if len == buf.len() {
            if let Ok(s) = std::str::from_utf8(&buf) {
                if s == token {
                    // Consume the message as it was relevant to us.
                    let _ = stream.read_exact(&mut buf);
                    return true;
                }
            }
        }
    };

    // No token found. Prompt user
    let (embedder_sender, receiver) = ipc::channel().expect("Failed to create IPC channel!");
    let message = "Accept incoming devtools connection?".to_owned();
    let prompt = PromptDefinition::YesNo(message, embedder_sender);
    let msg = EmbedderMsg::Prompt(prompt, PromptOrigin::Trusted);
    embedder.send((None, msg));
    receiver.recv().unwrap() == PromptResult::Primary
}

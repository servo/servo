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
use std::collections::HashMap;
use std::collections::hash_map::Entry::{Occupied, Vacant};
use std::io::Read;
use std::net::{Shutdown, TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread;

use base::id::{BrowsingContextId, PipelineId, WebViewId};
use crossbeam_channel::{Receiver, Sender, unbounded};
use devtools_traits::{
    ChromeToDevtoolsControlMsg, ConsoleMessage, ConsoleMessageBuilder, DevtoolScriptControlMsg,
    DevtoolsControlMsg, DevtoolsPageInfo, LogLevel, NavigationState, NetworkEvent, PageError,
    ScriptToDevtoolsControlMsg, WorkerId,
};
use embedder_traits::{AllowOrDeny, EmbedderMsg, EmbedderProxy};
use ipc_channel::ipc::{self, IpcSender};
use log::trace;
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
    pub mod console;
    pub mod device;
    pub mod framerate;
    pub mod inspector;
    pub mod memory;
    pub mod network_event;
    pub mod object;
    pub mod performance;
    pub mod preference;
    pub mod process;
    pub mod reflow;
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
            .spawn(move || {
                if let Some(instance) = DevtoolsInstance::create(sender, receiver, port, embedder) {
                    instance.run()
                }
            })
            .expect("Thread spawning failed");
    }
    sender
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(crate) struct StreamId(u32);

struct DevtoolsInstance {
    actors: Arc<Mutex<ActorRegistry>>,
    browsing_contexts: HashMap<BrowsingContextId, String>,
    receiver: Receiver<DevtoolsControlMsg>,
    pipelines: HashMap<PipelineId, BrowsingContextId>,
    actor_workers: HashMap<WorkerId, String>,
    actor_requests: HashMap<String, String>,
    connections: HashMap<StreamId, TcpStream>,
}

impl DevtoolsInstance {
    fn create(
        sender: Sender<DevtoolsControlMsg>,
        receiver: Receiver<DevtoolsControlMsg>,
        port: u16,
        embedder: EmbedderProxy,
    ) -> Option<Self> {
        let bound = TcpListener::bind(("0.0.0.0", port)).ok().and_then(|l| {
            l.local_addr()
                .map(|addr| addr.port())
                .ok()
                .map(|port| (l, port))
        });

        // A token shared with the embedder to bypass permission prompt.
        let port = if bound.is_some() { Ok(port) } else { Err(()) };
        let token = format!("{:X}", servo_rand::ServoRng::default().next_u32());
        embedder.send(EmbedderMsg::OnDevtoolsStarted(port, token.clone()));

        let listener = match bound {
            Some((l, _)) => l,
            None => {
                return None;
            },
        };

        // Create basic actors
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

        let instance = Self {
            actors,
            browsing_contexts: HashMap::new(),
            pipelines: HashMap::new(),
            receiver,
            actor_requests: HashMap::new(),
            actor_workers: HashMap::new(),
            connections: HashMap::new(),
        };

        thread::Builder::new()
            .name("DevtoolsCliAcceptor".to_owned())
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

        Some(instance)
    }

    fn run(mut self) {
        let mut next_id = StreamId(0);
        while let Ok(msg) = self.receiver.recv() {
            trace!("{:?}", msg);
            match msg {
                DevtoolsControlMsg::FromChrome(ChromeToDevtoolsControlMsg::AddClient(stream)) => {
                    let actors = self.actors.clone();
                    let id = next_id;
                    next_id = StreamId(id.0 + 1);
                    self.connections.insert(id, stream.try_clone().unwrap());

                    // Inform every browsing context of the new stream
                    for name in self.browsing_contexts.values() {
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
                )) => self.handle_framerate_tick(actor_name, tick),
                DevtoolsControlMsg::FromScript(ScriptToDevtoolsControlMsg::TitleChanged(
                    pipeline,
                    title,
                )) => self.handle_title_changed(pipeline, title),
                DevtoolsControlMsg::FromScript(ScriptToDevtoolsControlMsg::NewGlobal(
                    ids,
                    script_sender,
                    pageinfo,
                )) => self.handle_new_global(ids, script_sender, pageinfo),
                DevtoolsControlMsg::FromScript(ScriptToDevtoolsControlMsg::Navigate(
                    browsing_context,
                    state,
                )) => self.handle_navigate(browsing_context, state),
                DevtoolsControlMsg::FromScript(ScriptToDevtoolsControlMsg::ConsoleAPI(
                    pipeline_id,
                    console_message,
                    worker_id,
                )) => self.handle_console_message(pipeline_id, worker_id, console_message),
                DevtoolsControlMsg::FromScript(ScriptToDevtoolsControlMsg::ReportPageError(
                    pipeline_id,
                    page_error,
                )) => self.handle_page_error(pipeline_id, None, page_error),
                DevtoolsControlMsg::FromScript(ScriptToDevtoolsControlMsg::ReportCSSError(
                    pipeline_id,
                    css_error,
                )) => {
                    let mut console_message = ConsoleMessageBuilder::new(
                        LogLevel::Warn,
                        css_error.filename,
                        css_error.line,
                        css_error.column,
                    );
                    console_message.add_argument(css_error.msg.into());

                    self.handle_console_message(pipeline_id, None, console_message.finish())
                },
                DevtoolsControlMsg::FromChrome(ChromeToDevtoolsControlMsg::NetworkEvent(
                    request_id,
                    network_event,
                )) => {
                    // copy the connections vector
                    let mut connections = Vec::<TcpStream>::new();
                    for stream in self.connections.values() {
                        connections.push(stream.try_clone().unwrap());
                    }

                    let pipeline_id = match network_event {
                        NetworkEvent::HttpResponse(ref response) => response.pipeline_id,
                        NetworkEvent::HttpRequest(ref request) => request.pipeline_id,
                    };
                    self.handle_network_event(connections, pipeline_id, request_id, network_event);
                },
                DevtoolsControlMsg::FromChrome(ChromeToDevtoolsControlMsg::ServerExitMsg) => break,
            }
        }

        // Shut down all active connections
        for connection in self.connections.values_mut() {
            let _ = connection.shutdown(Shutdown::Both);
        }
    }

    fn handle_framerate_tick(&self, actor_name: String, tick: f64) {
        let mut actors = self.actors.lock().unwrap();
        let framerate_actor = actors.find_mut::<FramerateActor>(&actor_name);
        framerate_actor.add_tick(tick);
    }

    fn handle_navigate(&self, browsing_context_id: BrowsingContextId, state: NavigationState) {
        let actor_name = self.browsing_contexts.get(&browsing_context_id).unwrap();
        self.actors
            .lock()
            .unwrap()
            .find::<BrowsingContextActor>(actor_name)
            .navigate(state);
    }

    // We need separate actor representations for each script global that exists;
    // clients can theoretically connect to multiple globals simultaneously.
    // TODO: move this into the root or target modules?
    fn handle_new_global(
        &mut self,
        ids: (BrowsingContextId, PipelineId, Option<WorkerId>, WebViewId),
        script_sender: IpcSender<DevtoolScriptControlMsg>,
        page_info: DevtoolsPageInfo,
    ) {
        let mut actors = self.actors.lock().unwrap();

        let (browsing_context_id, pipeline_id, worker_id, webview_id) = ids;

        let console_name = actors.new_name("console");

        let parent_actor = if let Some(id) = worker_id {
            assert!(self.pipelines.contains_key(&pipeline_id));
            assert!(self.browsing_contexts.contains_key(&browsing_context_id));

            let thread = ThreadActor::new(actors.new_name("context"));
            let thread_name = thread.name();
            actors.register(Box::new(thread));

            let worker_name = actors.new_name("worker");
            let worker = WorkerActor {
                name: worker_name.clone(),
                console: console_name.clone(),
                thread: thread_name,
                worker_id: id,
                url: page_info.url.clone(),
                type_: WorkerType::Dedicated,
                script_chan: script_sender,
                streams: Default::default(),
            };
            let root = actors.find_mut::<RootActor>("root");
            root.workers.push(worker.name.clone());

            self.actor_workers.insert(id, worker_name.clone());
            actors.register(Box::new(worker));

            Root::DedicatedWorker(worker_name)
        } else {
            self.pipelines.insert(pipeline_id, browsing_context_id);
            let name = self
                .browsing_contexts
                .entry(browsing_context_id)
                .or_insert_with(|| {
                    let browsing_context_actor = BrowsingContextActor::new(
                        console_name.clone(),
                        webview_id,
                        browsing_context_id,
                        page_info,
                        pipeline_id,
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
            for (id, stream) in &self.connections {
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

    fn handle_title_changed(&self, pipeline_id: PipelineId, title: String) {
        let bc = match self.pipelines.get(&pipeline_id) {
            Some(bc) => bc,
            None => return,
        };
        let name = match self.browsing_contexts.get(bc) {
            Some(name) => name,
            None => return,
        };
        let actors = self.actors.lock().unwrap();
        let browsing_context = actors.find::<BrowsingContextActor>(name);
        browsing_context.title_changed(pipeline_id, title);
    }

    fn handle_page_error(
        &self,
        pipeline_id: PipelineId,
        worker_id: Option<WorkerId>,
        page_error: PageError,
    ) {
        let console_actor_name = match self.find_console_actor(pipeline_id, worker_id) {
            Some(name) => name,
            None => return,
        };
        let actors = self.actors.lock().unwrap();
        let console_actor = actors.find::<ConsoleActor>(&console_actor_name);
        let id = worker_id.map_or(UniqueId::Pipeline(pipeline_id), UniqueId::Worker);
        console_actor.handle_page_error(page_error, id, &actors);
    }

    fn handle_console_message(
        &self,
        pipeline_id: PipelineId,
        worker_id: Option<WorkerId>,
        console_message: ConsoleMessage,
    ) {
        let console_actor_name = match self.find_console_actor(pipeline_id, worker_id) {
            Some(name) => name,
            None => return,
        };
        let actors = self.actors.lock().unwrap();
        let console_actor = actors.find::<ConsoleActor>(&console_actor_name);
        let id = worker_id.map_or(UniqueId::Pipeline(pipeline_id), UniqueId::Worker);
        console_actor.handle_console_api(console_message, id, &actors);
    }

    fn find_console_actor(
        &self,
        pipeline_id: PipelineId,
        worker_id: Option<WorkerId>,
    ) -> Option<String> {
        let actors = self.actors.lock().unwrap();
        if let Some(worker_id) = worker_id {
            let actor_name = self.actor_workers.get(&worker_id)?;
            Some(actors.find::<WorkerActor>(actor_name).console.clone())
        } else {
            let id = self.pipelines.get(&pipeline_id)?;
            let actor_name = self.browsing_contexts.get(id)?;
            Some(
                actors
                    .find::<BrowsingContextActor>(actor_name)
                    .console
                    .clone(),
            )
        }
    }

    fn handle_network_event(
        &mut self,
        mut connections: Vec<TcpStream>,
        pipeline_id: PipelineId,
        request_id: String,
        network_event: NetworkEvent,
    ) {
        let console_actor_name = match self.find_console_actor(pipeline_id, None) {
            Some(name) => name,
            None => return,
        };
        let netevent_actor_name = self.find_network_event_actor(request_id);
        let mut actors = self.actors.lock().unwrap();
        let actor = actors.find_mut::<NetworkEventActor>(&netevent_actor_name);

        match network_event {
            NetworkEvent::HttpRequest(httprequest) => {
                // Store the request information in the actor
                actor.add_request(httprequest);

                // Send a networkEvent message to the client
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
                // Store the response information in the actor
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

                // Send a networkEventUpdate (responseStart) to the client
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
                    total_time: actor.total_time().as_millis() as u64,
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
    fn find_network_event_actor(&mut self, request_id: String) -> String {
        let mut actors = self.actors.lock().unwrap();
        match self.actor_requests.entry(request_id) {
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
    let (request_sender, request_receiver) = ipc::channel().expect("Failed to create IPC channel!");
    embedder.send(EmbedderMsg::RequestDevtoolsConnection(request_sender));
    request_receiver.recv().unwrap() == AllowOrDeny::Allow
}

/// Process the input from a single devtools client until EOF.
fn handle_client(actors: Arc<Mutex<ActorRegistry>>, mut stream: TcpStream, stream_id: StreamId) {
    log::info!("Connection established to {}", stream.peer_addr().unwrap());
    let msg = actors.lock().unwrap().find::<RootActor>("root").encodable();
    if let Err(e) = stream.write_json_packet(&msg) {
        log::warn!("Error writing response: {:?}", e);
        return;
    }

    loop {
        match stream.read_json_packet() {
            Ok(Some(json_packet)) => {
                if let Err(()) = actors.lock().unwrap().handle_message(
                    json_packet.as_object().unwrap(),
                    &mut stream,
                    stream_id,
                ) {
                    log::error!("Devtools actor stopped responding");
                    let _ = stream.shutdown(Shutdown::Both);
                    break;
                }
            },
            Ok(None) => {
                log::info!("Devtools connection closed");
                break;
            },
            Err(err_msg) => {
                log::error!("Failed to read message from devtools client: {}", err_msg);
                break;
            },
        }
    }

    actors.lock().unwrap().cleanup(stream_id);
}

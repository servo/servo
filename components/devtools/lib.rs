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
use std::io::Read;
use std::net::{Ipv4Addr, Shutdown, SocketAddr, TcpListener, TcpStream};
use std::str::FromStr;
use std::sync::{Arc, Mutex};
use std::thread;

use base::generic_channel::{self, GenericSender};
use base::id::{BrowsingContextId, PipelineId, WebViewId};
use crossbeam_channel::{Receiver, Sender, unbounded};
use devtools_traits::{
    ChromeToDevtoolsControlMsg, ConsoleLogLevel, ConsoleMessage, ConsoleMessageFields,
    DevtoolScriptControlMsg, DevtoolsControlMsg, DevtoolsPageInfo, DomMutation, FrameInfo,
    NavigationState, NetworkEvent, PauseReason, ScriptToDevtoolsControlMsg, SourceInfo, WorkerId,
    get_time_stamp,
};
use embedder_traits::{AllowOrDeny, EmbedderMsg, EmbedderProxy};
use log::{trace, warn};
use malloc_size_of::MallocSizeOf;
use malloc_size_of_derive::MallocSizeOf;
use profile_traits::path;
use rand::{RngCore, rng};
use resource::{ResourceArrayType, ResourceAvailable};
use rustc_hash::FxHashMap;
use serde::Serialize;
use servo_config::pref;

use crate::actor::{Actor, ActorError, ActorRegistry};
use crate::actors::browsing_context::BrowsingContextActor;
use crate::actors::console::{ConsoleActor, ConsoleResource, DevtoolsConsoleMessage, Root};
use crate::actors::frame::FrameActor;
use crate::actors::framerate::FramerateActor;
use crate::actors::inspector::InspectorActor;
use crate::actors::inspector::walker::WalkerActor;
use crate::actors::network_event::NetworkEventActor;
use crate::actors::pause::PauseActor;
use crate::actors::root::RootActor;
use crate::actors::source::SourceActor;
use crate::actors::thread::{ThreadActor, ThreadInterruptedReply};
use crate::actors::watcher::WatcherActor;
use crate::actors::worker::{WorkerActor, WorkerType};
use crate::id::IdMap;
use crate::network_handler::handle_network_event;
use crate::protocol::JsonPacketStream;

mod actor;
/// <https://searchfox.org/mozilla-central/source/devtools/server/actors>
mod actors {
    pub mod breakpoint;
    pub mod browsing_context;
    pub mod console;
    pub mod device;
    pub mod environment;
    pub mod frame;
    pub mod framerate;
    pub mod inspector;
    pub mod long_string;
    pub mod memory;
    pub mod network_event;
    pub mod object;
    pub mod pause;
    pub mod performance;
    pub mod preference;
    pub mod process;
    pub mod reflow;
    pub mod root;
    pub mod source;
    pub mod stylesheets;
    pub mod tab;
    pub mod thread;
    pub mod timeline;
    pub mod watcher;
    pub mod worker;
}
mod id;
mod network_handler;
mod protocol;
mod resource;
use profile_traits::mem::{
    ProcessReports, ProfilerChan, Report, ReportKind, perform_memory_report,
};

#[derive(Clone, Debug, Eq, Hash, PartialEq, MallocSizeOf)]
enum UniqueId {
    Pipeline(PipelineId),
    Worker(WorkerId),
}

#[derive(Serialize)]
pub(crate) struct EmptyReplyMsg {
    pub from: String,
}

#[derive(Serialize)]
pub(crate) struct ActorMsg {
    pub actor: String,
}

/// Spin up a devtools server that listens for connections on the specified port.
pub fn start_server(
    embedder: EmbedderProxy,
    mem_profiler_chan: ProfilerChan,
) -> Sender<DevtoolsControlMsg> {
    let (sender, receiver) = unbounded();
    {
        let sender = sender.clone();
        let sender2 = sender.clone();
        thread::Builder::new()
            .name("Devtools".to_owned())
            .spawn(move || {
                mem_profiler_chan.run_with_memory_reporting(
                    || {
                        if let Some(instance) = DevtoolsInstance::create(sender, receiver, embedder)
                        {
                            instance.run()
                        }
                    },
                    String::from("devtools-reporter"),
                    sender2,
                    |chan| {
                        DevtoolsControlMsg::FromChrome(
                            ChromeToDevtoolsControlMsg::CollectMemoryReport(chan),
                        )
                    },
                )
            })
            .expect("Thread spawning failed");
    }
    sender
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, MallocSizeOf)]
pub(crate) struct StreamId(u32);

#[derive(MallocSizeOf)]
struct DevtoolsInstance {
    #[conditional_malloc_size_of]
    registry: Arc<ActorRegistry>,
    #[conditional_malloc_size_of]
    id_map: Arc<Mutex<IdMap>>,
    browsing_contexts: FxHashMap<BrowsingContextId, String>,
    /// This is handed to clients so they can notify the devtools instance when
    /// their connection closes.
    sender: Sender<DevtoolsControlMsg>,
    receiver: Receiver<DevtoolsControlMsg>,
    pipelines: FxHashMap<PipelineId, BrowsingContextId>,
    actor_workers: FxHashMap<WorkerId, String>,
    actor_requests: HashMap<String, String>,
    /// A map of active TCP connections to devtools clients.
    ///
    /// Client threads remove their connection from here once they exit.
    #[conditional_malloc_size_of]
    connections: Arc<Mutex<FxHashMap<StreamId, TcpStream>>>,
    next_resource_id: u64,
}

impl DevtoolsInstance {
    fn create(
        sender: Sender<DevtoolsControlMsg>,
        receiver: Receiver<DevtoolsControlMsg>,
        embedder: EmbedderProxy,
    ) -> Option<Self> {
        let address = if pref!(devtools_server_listen_address).is_empty() {
            SocketAddr::new(Ipv4Addr::new(127, 0, 0, 1).into(), 7000)
        } else if let Ok(addr) = SocketAddr::from_str(&pref!(devtools_server_listen_address)) {
            addr
        } else if let Ok(port) = pref!(devtools_server_listen_address).parse() {
            SocketAddr::new(Ipv4Addr::new(127, 0, 0, 1).into(), port)
        } else {
            SocketAddr::new(Ipv4Addr::new(127, 0, 0, 1).into(), 7000)
        };
        println!("Binding devtools to {address}");

        let bound = TcpListener::bind(address).ok().and_then(|l| {
            l.local_addr()
                .map(|addr| addr.port())
                .ok()
                .map(|port| (l, port))
        });

        // A token shared with the embedder to bypass permission prompt.
        let port = if bound.is_some() {
            Ok(address.port())
        } else {
            Err(())
        };
        let token = format!("{:X}", rng().next_u32());
        embedder.send(EmbedderMsg::OnDevtoolsStarted(port, token.clone()));

        let listener = match bound {
            Some((l, _)) => l,
            None => {
                return None;
            },
        };

        // Create basic actors
        let mut registry = ActorRegistry::default();
        RootActor::register(&mut registry);

        let instance = Self {
            registry: Arc::new(registry),
            id_map: Arc::new(Mutex::new(IdMap::default())),
            browsing_contexts: FxHashMap::default(),
            pipelines: FxHashMap::default(),
            sender: sender.clone(),
            receiver,
            actor_requests: HashMap::new(),
            actor_workers: FxHashMap::default(),
            connections: Default::default(),
            next_resource_id: 1,
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
                    let actors = self.registry.clone();
                    let id = next_id;
                    next_id = StreamId(id.0 + 1);

                    let mut connections = self.connections.lock().unwrap();
                    if connections.is_empty() {
                        // We used to have no connection, now we have one.
                        // Therefore, we need updates from script threads.
                        for browsing_context in self.browsing_contexts.values() {
                            let actor =
                                self.registry.find::<BrowsingContextActor>(browsing_context);
                            actor.instruct_script_to_send_live_updates(true);
                        }
                    }
                    connections.insert(id, stream.try_clone().unwrap());

                    let connections_clone = self.connections.clone();
                    let sender_clone = self.sender.clone();
                    thread::Builder::new()
                        .name("DevtoolsClientHandler".to_owned())
                        .spawn(move || {
                            handle_client(
                                actors,
                                stream.try_clone().unwrap(),
                                id,
                                connections_clone,
                                sender_clone,
                            )
                        })
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
                )) => {
                    let console_message =
                        DevtoolsConsoleMessage::new(console_message, &self.registry);
                    self.handle_console_resource(
                        pipeline_id,
                        worker_id,
                        ConsoleResource::ConsoleMessage(console_message),
                    );
                },
                DevtoolsControlMsg::FromScript(ScriptToDevtoolsControlMsg::ClearConsole(
                    pipeline_id,
                    worker_id,
                )) => self.handle_clear_console(pipeline_id, worker_id),
                DevtoolsControlMsg::FromScript(ScriptToDevtoolsControlMsg::CreateSourceActor(
                    script_sender,
                    pipeline_id,
                    source_info,
                )) => self.handle_create_source_actor(script_sender, pipeline_id, source_info),
                DevtoolsControlMsg::FromScript(
                    ScriptToDevtoolsControlMsg::UpdateSourceContent(pipeline_id, source_content),
                ) => self.handle_update_source_content(pipeline_id, source_content),
                DevtoolsControlMsg::FromScript(ScriptToDevtoolsControlMsg::ReportPageError(
                    pipeline_id,
                    page_error,
                )) => self.handle_console_resource(
                    pipeline_id,
                    None,
                    ConsoleResource::PageError(page_error.into()),
                ),
                DevtoolsControlMsg::FromScript(ScriptToDevtoolsControlMsg::ReportCSSError(
                    pipeline_id,
                    css_error,
                )) => {
                    let console_message = ConsoleMessage {
                        fields: ConsoleMessageFields {
                            level: ConsoleLogLevel::Warn,
                            filename: css_error.filename,
                            line_number: css_error.line,
                            column_number: css_error.column,
                            time_stamp: get_time_stamp(),
                        },
                        arguments: vec![css_error.msg.into()],
                        stacktrace: None,
                    };
                    let console_message =
                        DevtoolsConsoleMessage::new(console_message, &self.registry);

                    self.handle_console_resource(
                        pipeline_id,
                        None,
                        ConsoleResource::ConsoleMessage(console_message),
                    )
                },
                DevtoolsControlMsg::FromScript(ScriptToDevtoolsControlMsg::DomMutation(
                    pipeline_id,
                    dom_mutation,
                )) => {
                    self.handle_dom_mutation(pipeline_id, dom_mutation).unwrap();
                },
                DevtoolsControlMsg::FromScript(ScriptToDevtoolsControlMsg::DebuggerPause(
                    pipeline_id,
                    frame_actor_id,
                    pause_reason,
                )) => self.handle_debugger_pause(pipeline_id, frame_actor_id, pause_reason),
                DevtoolsControlMsg::FromScript(ScriptToDevtoolsControlMsg::CreateFrameActor(
                    result_sender,
                    pipeline_id,
                    frame_info,
                )) => self.handle_create_frame_actor(result_sender, pipeline_id, frame_info),
                DevtoolsControlMsg::FromChrome(ChromeToDevtoolsControlMsg::NetworkEvent(
                    request_id,
                    network_event,
                )) => {
                    // copy the connections vector
                    // FIXME: Why do we need to do this? Cloning the connections here is
                    // almost certainly wrong and means that they might shut down without
                    // us noticing.
                    let mut connections = Vec::<TcpStream>::new();
                    for stream in self.connections.lock().unwrap().values() {
                        connections.push(stream.try_clone().unwrap());
                    }
                    self.handle_network_event(connections, request_id, network_event);
                },
                DevtoolsControlMsg::FromChrome(ChromeToDevtoolsControlMsg::ServerExitMsg) => break,
                DevtoolsControlMsg::FromChrome(
                    ChromeToDevtoolsControlMsg::CollectMemoryReport(chan),
                ) => {
                    perform_memory_report(|ops| {
                        let reports = vec![Report {
                            path: path!["devtools"],
                            kind: ReportKind::ExplicitSystemHeapSize,
                            size: self.size_of(ops),
                        }];
                        chan.send(ProcessReports::new(reports));
                    });
                },
                DevtoolsControlMsg::ClientExited => {
                    if self.connections.lock().unwrap().is_empty() {
                        // Tell every browsing context to stop sending us updates, because we have nowhere to
                        // send them to.
                        for browsing_context in self.browsing_contexts.values() {
                            let actor =
                                self.registry.find::<BrowsingContextActor>(browsing_context);
                            actor.instruct_script_to_send_live_updates(false);
                        }
                    }
                },
            }
        }

        // Shut down all active connections
        let mut connections = self.connections.lock().unwrap();
        for connection in connections.values_mut() {
            let _ = connection.shutdown(Shutdown::Both);
        }
        connections.clear();
    }

    fn handle_framerate_tick(&self, actor_name: String, tick: f64) {
        let actors = &self.registry;
        let framerate_actor = actors.find::<FramerateActor>(&actor_name);
        framerate_actor.add_tick(tick);
    }

    fn handle_navigate(&self, browsing_context_id: BrowsingContextId, state: NavigationState) {
        let actor_name = self.browsing_contexts.get(&browsing_context_id).unwrap();
        let actors = &self.registry;
        let actor = actors.find::<BrowsingContextActor>(actor_name);
        let mut id_map = self.id_map.lock().expect("Mutex poisoned");
        let mut connections = self.connections.lock().unwrap();
        if let NavigationState::Start(url) = &state {
            let watcher_actor = actors.find::<WatcherActor>(&actor.watcher);
            watcher_actor.emit_will_navigate(
                browsing_context_id,
                url.clone(),
                &mut connections.values_mut(),
                &mut id_map,
            );
        };

        actor.navigate(state, &mut id_map, connections.values_mut());
    }

    // We need separate actor representations for each script global that exists;
    // clients can theoretically connect to multiple globals simultaneously.
    // TODO: move this into the root or target modules?
    fn handle_new_global(
        &mut self,
        ids: (BrowsingContextId, PipelineId, Option<WorkerId>, WebViewId),
        script_sender: GenericSender<DevtoolScriptControlMsg>,
        page_info: DevtoolsPageInfo,
    ) {
        let actors = &self.registry;

        let (browsing_context_id, pipeline_id, worker_id, webview_id) = ids;
        let id_map = &mut self.id_map.lock().expect("Mutex poisoned");
        let devtools_browser_id = id_map.browser_id(webview_id);
        let devtools_browsing_context_id = id_map.browsing_context_id(browsing_context_id);
        let devtools_outer_window_id = id_map.outer_window_id(pipeline_id);

        let console_name = actors.new_name::<ConsoleActor>();

        let parent_actor = if let Some(id) = worker_id {
            assert!(self.pipelines.contains_key(&pipeline_id));
            assert!(self.browsing_contexts.contains_key(&browsing_context_id));

            let thread = ThreadActor::new(actors.new_name::<ThreadActor>(), script_sender.clone());
            let thread_name = thread.name();
            actors.register(thread);

            let worker_name = actors.new_name::<WorkerActor>();
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
            let root = actors.find::<RootActor>("root");
            root.workers.borrow_mut().push(worker.name.clone());

            self.actor_workers.insert(id, worker_name.clone());
            actors.register(worker);

            Root::DedicatedWorker(worker_name)
        } else {
            self.pipelines.insert(pipeline_id, browsing_context_id);
            let name = self
                .browsing_contexts
                .entry(browsing_context_id)
                .or_insert_with(|| {
                    let browsing_context_actor = BrowsingContextActor::new(
                        console_name.clone(),
                        devtools_browser_id,
                        devtools_browsing_context_id,
                        page_info,
                        pipeline_id,
                        devtools_outer_window_id,
                        script_sender,
                        actors,
                    );
                    let name = browsing_context_actor.name();
                    actors.register(browsing_context_actor);
                    name
                });

            Root::BrowsingContext(name.clone())
        };

        let console = ConsoleActor::new(console_name, parent_actor);

        actors.register(console);
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
        let actors = &self.registry;
        let browsing_context = actors.find::<BrowsingContextActor>(name);
        browsing_context.title_changed(pipeline_id, title);
    }

    fn handle_console_resource(
        &mut self,
        pipeline_id: PipelineId,
        worker_id: Option<WorkerId>,
        resource: ConsoleResource,
    ) {
        let console_actor_name = match self.find_console_actor(pipeline_id, worker_id) {
            Some(name) => name,
            None => return,
        };
        let actors = &self.registry;
        let console_actor = actors.find::<ConsoleActor>(&console_actor_name);
        let id = worker_id.map_or(UniqueId::Pipeline(pipeline_id), UniqueId::Worker);

        for stream in self.connections.lock().unwrap().values_mut() {
            console_actor.handle_console_resource(resource.clone(), id.clone(), actors, stream);
        }
    }

    fn handle_dom_mutation(
        &mut self,
        pipeline_id: PipelineId,
        dom_mutation: DomMutation,
    ) -> Result<(), ActorError> {
        let Some(browsing_context_id) = self.pipelines.get(&pipeline_id) else {
            log::warn!("Devtools received notification for unknown pipeline {pipeline_id}");
            return Err(ActorError::Internal);
        };
        let Some(browsing_context_actor_id) = self.browsing_contexts.get(browsing_context_id)
        else {
            return Err(ActorError::Internal);
        };
        let browsing_context_actor = self
            .registry
            .find::<BrowsingContextActor>(browsing_context_actor_id);
        let inspector_actor = self
            .registry
            .find::<InspectorActor>(&browsing_context_actor.inspector);
        let walker_actor = self.registry.find::<WalkerActor>(&inspector_actor.walker);

        for stream in self.connections.lock().unwrap().values_mut() {
            walker_actor.handle_dom_mutation(dom_mutation.clone(), stream)?;
        }

        Ok(())
    }

    fn handle_clear_console(&mut self, pipeline_id: PipelineId, worker_id: Option<WorkerId>) {
        let console_actor_name = match self.find_console_actor(pipeline_id, worker_id) {
            Some(name) => name,
            None => return,
        };
        let actors = &self.registry;
        let console_actor = actors.find::<ConsoleActor>(&console_actor_name);
        let id = worker_id.map_or(UniqueId::Pipeline(pipeline_id), UniqueId::Worker);

        for stream in self.connections.lock().unwrap().values_mut() {
            console_actor.send_clear_message(id.clone(), actors, stream);
        }
    }

    fn find_console_actor(
        &self,
        pipeline_id: PipelineId,
        worker_id: Option<WorkerId>,
    ) -> Option<String> {
        let actors = &self.registry;
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
        connections: Vec<TcpStream>,
        request_id: String,
        network_event: NetworkEvent,
    ) {
        let browsing_context_id = match &network_event {
            NetworkEvent::HttpRequest(req) => req.browsing_context_id,
            NetworkEvent::HttpRequestUpdate(req) => req.browsing_context_id,
            NetworkEvent::HttpResponse(resp) => resp.browsing_context_id,
            NetworkEvent::SecurityInfo(update) => update.browsing_context_id,
        };

        let Some(browsing_context_actor_name) = self.browsing_contexts.get(&browsing_context_id)
        else {
            return;
        };
        let watcher_name = self
            .registry
            .find::<BrowsingContextActor>(browsing_context_actor_name)
            .watcher
            .clone();

        let netevent_actor_name = match self.actor_requests.get(&request_id) {
            Some(name) => name.clone(),
            None => self.create_network_event_actor(request_id, watcher_name),
        };

        handle_network_event(
            Arc::clone(&self.registry),
            netevent_actor_name,
            connections,
            network_event,
        )
    }

    /// Create a new NetworkEventActor for a given request ID and watcher name.
    fn create_network_event_actor(&mut self, request_id: String, watcher_name: String) -> String {
        let actors = &self.registry;
        let resource_id = self.next_resource_id;
        self.next_resource_id += 1;

        let actor_name = actors.new_name::<NetworkEventActor>();
        let actor = NetworkEventActor::new(actor_name.clone(), resource_id, watcher_name);

        self.actor_requests.insert(request_id, actor_name.clone());
        actors.register(actor);

        actor_name
    }

    fn handle_create_source_actor(
        &mut self,
        script_sender: GenericSender<DevtoolScriptControlMsg>,
        pipeline_id: PipelineId,
        source_info: SourceInfo,
    ) {
        let actors = &self.registry;

        let source_content = source_info
            .content
            .or_else(|| actors.inline_source_content(pipeline_id));
        let source_actor = SourceActor::new_registered(
            actors,
            pipeline_id,
            source_info.url,
            source_content,
            source_info.content_type,
            source_info.spidermonkey_id,
            source_info.introduction_type,
            script_sender,
        );
        let source_form = actors.find::<SourceActor>(&source_actor).source_form();

        if let Some(worker_id) = source_info.worker_id {
            let Some(worker_actor_name) = self.actor_workers.get(&worker_id) else {
                return;
            };

            let thread_actor_name = actors.find::<WorkerActor>(worker_actor_name).thread.clone();
            let thread_actor = actors.find::<ThreadActor>(&thread_actor_name);

            thread_actor.source_manager.add_source(&source_actor);

            let worker_actor = actors.find::<WorkerActor>(worker_actor_name);

            for stream in self.connections.lock().unwrap().values_mut() {
                worker_actor.resource_array(
                    &source_form,
                    "source".into(),
                    ResourceArrayType::Available,
                    stream,
                );
            }
        } else {
            let Some(browsing_context_id) = self.pipelines.get(&pipeline_id) else {
                return;
            };
            let Some(actor_name) = self.browsing_contexts.get(browsing_context_id) else {
                return;
            };

            let thread_actor_name = {
                let browsing_context = actors.find::<BrowsingContextActor>(actor_name);
                browsing_context.thread.clone()
            };

            let thread_actor = actors.find::<ThreadActor>(&thread_actor_name);
            thread_actor.source_manager.add_source(&source_actor);

            // Notify browsing context about the new source
            let browsing_context = actors.find::<BrowsingContextActor>(actor_name);

            for stream in self.connections.lock().unwrap().values_mut() {
                browsing_context.resource_array(
                    &source_form,
                    "source".into(),
                    ResourceArrayType::Available,
                    stream,
                );
            }
        }
    }

    fn handle_update_source_content(&mut self, pipeline_id: PipelineId, source_content: String) {
        let actors = &self.registry;

        for actor_name in actors.source_actor_names_for_pipeline(pipeline_id) {
            let source_actor = actors.find::<SourceActor>(&actor_name);
            let mut content = source_actor.content.borrow_mut();
            if content.is_none() {
                *content = Some(source_content.clone());
            }
        }

        // Store the source content separately for any future source actors that get created *after* we finish parsing
        // the HTML. For example, adding an `import` to an inline module script can delay it until after parsing.
        actors.set_inline_source_content(pipeline_id, source_content);
    }

    fn handle_debugger_pause(
        &mut self,
        pipeline_id: PipelineId,
        frame_actor_id: String,
        pause_reason: PauseReason,
    ) {
        let actors = &self.registry;

        let Some(browsing_context) = self
            .pipelines
            .get(&pipeline_id)
            .and_then(|id| self.browsing_contexts.get(id))
        else {
            return;
        };

        let browsing_context = actors.find::<BrowsingContextActor>(browsing_context);
        let thread = actors.find::<ThreadActor>(&browsing_context.thread);

        let pause = actors.new_name::<PauseActor>();
        actors.register(PauseActor {
            name: pause.clone(),
        });

        let msg = ThreadInterruptedReply {
            from: thread.name(),
            type_: "paused".to_owned(),
            actor: pause,
            frame: actors.encode::<FrameActor, _>(&frame_actor_id),
            why: pause_reason,
        };

        for stream in self.connections.lock().unwrap().values_mut() {
            let _ = stream.write_json_packet(&msg);
        }
    }

    fn handle_create_frame_actor(
        &mut self,
        result_sender: GenericSender<String>,
        pipeline_id: PipelineId,
        frame: FrameInfo,
    ) {
        let actors = &self.registry;

        let Some(browsing_context) = self
            .pipelines
            .get(&pipeline_id)
            .and_then(|id| self.browsing_contexts.get(id))
        else {
            return;
        };

        let browsing_context = actors.find::<BrowsingContextActor>(browsing_context);
        let thread = actors.find::<ThreadActor>(&browsing_context.thread);

        let source = match thread.source_manager.find_source(actors, &frame.url) {
            Some(source) => source.name(),
            None => {
                warn!("No source actor found for URL: {}", frame.url);
                return;
            },
        };

        let frame = FrameActor::register(actors, source, frame);
        thread.frames.borrow_mut().insert(frame.clone());

        let _ = result_sender.send(frame);
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
    let (request_sender, request_receiver) =
        generic_channel::channel().expect("Failed to create IPC channel!");
    embedder.send(EmbedderMsg::RequestDevtoolsConnection(request_sender));
    request_receiver.recv().unwrap() == AllowOrDeny::Allow
}

/// Process the input from a single devtools client until EOF.
fn handle_client(
    actors: Arc<ActorRegistry>,
    mut stream: TcpStream,
    stream_id: StreamId,
    connections: Arc<Mutex<FxHashMap<StreamId, TcpStream>>>,
    sender: Sender<DevtoolsControlMsg>,
) {
    log::info!("Connection established to {}", stream.peer_addr().unwrap());
    let msg = actors.encode::<RootActor, _>("root");
    if let Err(error) = stream.write_json_packet(&msg) {
        warn!("Failed to send initial packet from root actor: {error:?}");
        return;
    }

    loop {
        match stream.read_json_packet() {
            Ok(Some(json_packet)) => {
                if let Err(()) =
                    actors.handle_message(json_packet.as_object().unwrap(), &mut stream, stream_id)
                {
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

    connections.lock().unwrap().remove(&stream_id);
    let _ = sender.send(DevtoolsControlMsg::ClientExited);

    actors.cleanup(stream_id);
}

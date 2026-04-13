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

use crossbeam_channel::{Receiver, Sender, unbounded};
use devtools_traits::{
    ChromeToDevtoolsControlMsg, ConsoleLogLevel, ConsoleMessage, ConsoleMessageFields,
    DebuggerValue, DevtoolScriptControlMsg, DevtoolsControlMsg, DevtoolsPageInfo, DomMutation,
    EnvironmentInfo, FrameInfo, FrameOffset, NavigationState, NetworkEvent, PauseReason,
    ScriptToDevtoolsControlMsg, SourceInfo, WorkerId, get_time_stamp,
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
use serde_json::{Map, Number, Value};
use servo_base::generic_channel::{self, GenericSender};
use servo_base::id::{BrowsingContextId, PipelineId, WebViewId};
use servo_config::pref;

use crate::actor::{Actor, ActorEncode, ActorError, ActorRegistry};
use crate::actors::browsing_context::BrowsingContextActor;
use crate::actors::console::{ConsoleActor, ConsoleResource, DevtoolsConsoleMessage, Root};
use crate::actors::environment::EnvironmentActor;
use crate::actors::frame::FrameActor;
use crate::actors::framerate::FramerateActor;
use crate::actors::inspector::InspectorActor;
use crate::actors::inspector::walker::WalkerActor;
use crate::actors::network_event::NetworkEventActor;
use crate::actors::object::ObjectActor;
use crate::actors::pause::PauseActor;
use crate::actors::root::RootActor;
use crate::actors::source::SourceActor;
use crate::actors::thread::{ThreadActor, ThreadInterruptedReply};
use crate::actors::watcher::WatcherActor;
use crate::actors::worker::{WorkerTargetActor, WorkerType};
use crate::id::IdMap;
use crate::network_handler::handle_network_event;
use crate::protocol::{DevtoolsConnection, JsonPacketStream};

mod actor;
/// <https://searchfox.org/mozilla-central/source/devtools/server/actors>
mod actors {
    pub mod blackboxing;
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
    pub mod property_iterator;
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
    connections: Arc<Mutex<FxHashMap<StreamId, DevtoolsConnection>>>,
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
                    let id = next_id;
                    next_id = StreamId(id.0 + 1);

                    {
                        let connections = self.connections.lock().unwrap();
                        if connections.is_empty() {
                            // We used to have no connection, now we have one.
                            // Therefore, we need updates from script threads.
                            for browsing_context_name in self.browsing_contexts.values() {
                                let browsing_context_actor = self
                                    .registry
                                    .find::<BrowsingContextActor>(browsing_context_name);
                                browsing_context_actor.instruct_script_to_send_live_updates(true);
                            }
                        }
                    }

                    let connection: DevtoolsConnection = stream.into();
                    let registry = self.registry.clone();
                    let connections = self.connections.clone();
                    let sender_clone = self.sender.clone();
                    thread::Builder::new()
                        .name("DevtoolsClientHandler".to_owned())
                        .spawn(move || {
                            handle_client(registry, connection, id, connections, sender_clone)
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
                        arguments: vec![DebuggerValue::StringValue(css_error.msg)],
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
                    frame_offset,
                    pause_reason,
                )) => self.handle_debugger_pause(pipeline_id, frame_offset, pause_reason),
                DevtoolsControlMsg::FromScript(ScriptToDevtoolsControlMsg::CreateFrameActor(
                    result_sender,
                    pipeline_id,
                    frame_info,
                )) => self.handle_create_frame_actor(result_sender, pipeline_id, frame_info),
                DevtoolsControlMsg::FromScript(
                    ScriptToDevtoolsControlMsg::CreateEnvironmentActor(
                        result_sender,
                        environment,
                        parent,
                    ),
                ) => self.handle_create_environment_actor(result_sender, environment, parent),
                DevtoolsControlMsg::FromChrome(ChromeToDevtoolsControlMsg::NetworkEvent(
                    request_id,
                    network_event,
                )) => {
                    // copy the connections vector
                    // FIXME: Why do we need to do this? Cloning the connections here is
                    // almost certainly wrong and means that they might shut down without
                    // us noticing.
                    let mut connections = Vec::<DevtoolsConnection>::new();
                    for connection in self.connections.lock().unwrap().values() {
                        connections.push(connection.clone());
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
                        for browsing_context_name in self.browsing_contexts.values() {
                            let browsing_context_actor = self
                                .registry
                                .find::<BrowsingContextActor>(browsing_context_name);
                            browsing_context_actor.instruct_script_to_send_live_updates(false);
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
        let framerate_actor = self.registry.find::<FramerateActor>(&actor_name);
        framerate_actor.add_tick(tick);
    }

    fn handle_navigate(&self, browsing_context_id: BrowsingContextId, state: NavigationState) {
        let browsing_context_name = self.browsing_contexts.get(&browsing_context_id).unwrap();
        let browsing_context_actor = self
            .registry
            .find::<BrowsingContextActor>(browsing_context_name);
        let mut id_map = self.id_map.lock().unwrap();
        let mut connections = self.connections.lock().unwrap();
        if let NavigationState::Start(url) = &state {
            let watcher_actor = self
                .registry
                .find::<WatcherActor>(&browsing_context_actor.watcher_name);
            watcher_actor.emit_will_navigate(
                browsing_context_id,
                url.clone(),
                &mut connections.values_mut(),
                &mut id_map,
            );
        }

        browsing_context_actor.handle_navigate(state, &mut id_map, connections.values_mut());
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
        let (browsing_context_id, pipeline_id, worker_id, webview_id) = ids;
        let id_map = &mut self.id_map.lock().unwrap();
        let devtools_browser_id = id_map.browser_id(webview_id);
        let devtools_browsing_context_id = id_map.browsing_context_id(browsing_context_id);
        let devtools_outer_window_id = id_map.outer_window_id(pipeline_id);

        let console_name = self.registry.new_name::<ConsoleActor>();

        let parent_actor = if let Some(id) = worker_id {
            let thread_name = ThreadActor::register(&self.registry, script_sender.clone(), None);

            let worker_type = if page_info.is_service_worker {
                WorkerType::Service
            } else {
                WorkerType::Dedicated
            };
            let worker_name = WorkerTargetActor::register(
                &self.registry,
                console_name.clone(),
                thread_name,
                id,
                page_info.url,
                worker_type,
                script_sender,
            );
            let root_actor = self.registry.find::<RootActor>("root");
            if page_info.is_service_worker {
                root_actor
                    .service_workers
                    .borrow_mut()
                    .push(worker_name.clone());
            } else {
                root_actor.workers.borrow_mut().push(worker_name.clone());
            }

            self.actor_workers.insert(id, worker_name.clone());

            Root::DedicatedWorker(worker_name)
        } else {
            self.pipelines.insert(pipeline_id, browsing_context_id);
            let browsing_context_name = self
                .browsing_contexts
                .entry(browsing_context_id)
                .or_insert_with(|| {
                    BrowsingContextActor::register(
                        &self.registry,
                        console_name.clone(),
                        devtools_browser_id,
                        devtools_browsing_context_id,
                        page_info,
                        pipeline_id,
                        devtools_outer_window_id,
                        script_sender.clone(),
                    )
                });
            let browsing_context_actor = self
                .registry
                .find::<BrowsingContextActor>(browsing_context_name);
            browsing_context_actor.handle_new_global(pipeline_id, script_sender);
            Root::BrowsingContext(browsing_context_name.clone())
        };

        ConsoleActor::register(&self.registry, console_name, parent_actor);
    }

    fn handle_title_changed(&self, pipeline_id: PipelineId, title: String) {
        let browsing_context_id = match self.pipelines.get(&pipeline_id) {
            Some(bc) => bc,
            None => return,
        };
        let browsing_context_name = match self.browsing_contexts.get(browsing_context_id) {
            Some(name) => name,
            None => return,
        };
        let browsing_context_actor = self
            .registry
            .find::<BrowsingContextActor>(browsing_context_name);
        browsing_context_actor.title_changed(pipeline_id, title);
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
        let console_actor = self.registry.find::<ConsoleActor>(&console_actor_name);
        let id = worker_id.map_or(UniqueId::Pipeline(pipeline_id), UniqueId::Worker);

        for connection in self.connections.lock().unwrap().values_mut() {
            console_actor.handle_console_resource(
                resource.clone(),
                id.clone(),
                &self.registry,
                connection,
            );
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
        let Some(browsing_context_name) = self.browsing_contexts.get(browsing_context_id) else {
            return Err(ActorError::Internal);
        };
        let browsing_context_actor = self
            .registry
            .find::<BrowsingContextActor>(browsing_context_name);
        let inspector_actor = self
            .registry
            .find::<InspectorActor>(&browsing_context_actor.inspector_name);
        let walker_actor = self
            .registry
            .find::<WalkerActor>(&inspector_actor.walker_name);

        for connection in self.connections.lock().unwrap().values_mut() {
            walker_actor.handle_dom_mutation(dom_mutation.clone(), connection)?;
        }

        Ok(())
    }

    fn handle_clear_console(&mut self, pipeline_id: PipelineId, worker_id: Option<WorkerId>) {
        let console_actor_name = match self.find_console_actor(pipeline_id, worker_id) {
            Some(name) => name,
            None => return,
        };
        let console_actor = self.registry.find::<ConsoleActor>(&console_actor_name);
        let id = worker_id.map_or(UniqueId::Pipeline(pipeline_id), UniqueId::Worker);

        for stream in self.connections.lock().unwrap().values_mut() {
            console_actor.send_clear_message(id.clone(), &self.registry, stream);
        }
    }

    fn find_console_actor(
        &self,
        pipeline_id: PipelineId,
        worker_id: Option<WorkerId>,
    ) -> Option<String> {
        if let Some(worker_id) = worker_id {
            let worker_name = self.actor_workers.get(&worker_id)?;
            Some(
                self.registry
                    .find::<WorkerTargetActor>(worker_name)
                    .console_name
                    .clone(),
            )
        } else {
            let browsing_context_id = self.pipelines.get(&pipeline_id)?;
            let browsing_context_name = self.browsing_contexts.get(browsing_context_id)?;
            Some(
                self.registry
                    .find::<BrowsingContextActor>(browsing_context_name)
                    .console_name
                    .clone(),
            )
        }
    }

    fn handle_network_event(
        &mut self,
        connections: Vec<DevtoolsConnection>,
        request_id: String,
        network_event: NetworkEvent,
    ) {
        let browsing_context_id = match &network_event {
            NetworkEvent::HttpRequest(req) => req.browsing_context_id,
            NetworkEvent::HttpRequestUpdate(req) => req.browsing_context_id,
            NetworkEvent::HttpResponse(resp) => resp.browsing_context_id,
            NetworkEvent::SecurityInfo(update) => update.browsing_context_id,
        };

        let Some(browsing_context_name) = self.browsing_contexts.get(&browsing_context_id) else {
            return;
        };
        let watcher_name = self
            .registry
            .find::<BrowsingContextActor>(browsing_context_name)
            .watcher_name
            .clone();

        let network_event_name = match self.actor_requests.get(&request_id) {
            Some(name) => name.clone(),
            None => self.create_network_event_actor(request_id, watcher_name),
        };

        handle_network_event(
            Arc::clone(&self.registry),
            network_event_name,
            connections,
            network_event,
        )
    }

    /// Create a new NetworkEventActor for a given request ID and watcher name.
    fn create_network_event_actor(&mut self, request_id: String, watcher_name: String) -> String {
        let resource_id = self.next_resource_id;
        self.next_resource_id += 1;

        let network_event_name =
            NetworkEventActor::register(&self.registry, resource_id, watcher_name);

        self.actor_requests
            .insert(request_id, network_event_name.clone());

        network_event_name
    }

    fn handle_create_source_actor(
        &mut self,
        script_sender: GenericSender<DevtoolScriptControlMsg>,
        pipeline_id: PipelineId,
        source_info: SourceInfo,
    ) {
        let source_content = source_info
            .content
            .or_else(|| self.registry.inline_source_content(pipeline_id));
        let source_actor = SourceActor::register(
            &self.registry,
            pipeline_id,
            source_info.url,
            source_content,
            source_info.content_type,
            source_info.spidermonkey_id,
            source_info.introduction_type,
            script_sender,
        );
        let source_form = self
            .registry
            .find::<SourceActor>(&source_actor)
            .source_form();

        if let Some(worker_id) = source_info.worker_id {
            let Some(worker_name) = self.actor_workers.get(&worker_id) else {
                return;
            };

            let thread_actor_name = self
                .registry
                .find::<WorkerTargetActor>(worker_name)
                .thread_name
                .clone();
            let thread_actor = self.registry.find::<ThreadActor>(&thread_actor_name);

            thread_actor.source_manager.add_source(&source_actor);

            let worker_actor = self.registry.find::<WorkerTargetActor>(worker_name);

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
            let Some(browsing_context_name) = self.browsing_contexts.get(browsing_context_id)
            else {
                return;
            };

            let thread_actor_name = {
                let browsing_context_actor = self
                    .registry
                    .find::<BrowsingContextActor>(browsing_context_name);
                browsing_context_actor.thread_name.clone()
            };

            let thread_actor = self.registry.find::<ThreadActor>(&thread_actor_name);
            thread_actor.source_manager.add_source(&source_actor);

            // Notify browsing context about the new source
            let browsing_context_actor = self
                .registry
                .find::<BrowsingContextActor>(browsing_context_name);

            for stream in self.connections.lock().unwrap().values_mut() {
                browsing_context_actor.resource_array(
                    &source_form,
                    "source".into(),
                    ResourceArrayType::Available,
                    stream,
                );
            }
        }
    }

    fn handle_update_source_content(&mut self, pipeline_id: PipelineId, source_content: String) {
        for source_name in self.registry.source_actor_names_for_pipeline(pipeline_id) {
            let source_actor = self.registry.find::<SourceActor>(&source_name);
            let mut content = source_actor.content.borrow_mut();
            if content.is_none() {
                *content = Some(source_content.clone());
            }
        }

        // Store the source content separately for any future source actors that get created *after* we finish parsing
        // the HTML. For example, adding an `import` to an inline module script can delay it until after parsing.
        self.registry
            .set_inline_source_content(pipeline_id, source_content);
    }

    fn handle_debugger_pause(
        &mut self,
        pipeline_id: PipelineId,
        frame_offset: FrameOffset,
        pause_reason: PauseReason,
    ) {
        let Some(browsing_context_name) = self
            .pipelines
            .get(&pipeline_id)
            .and_then(|id| self.browsing_contexts.get(id))
        else {
            return;
        };

        let browsing_context_actor = self
            .registry
            .find::<BrowsingContextActor>(browsing_context_name);
        let thread_actor = self
            .registry
            .find::<ThreadActor>(&browsing_context_actor.thread_name);

        let pause_name = PauseActor::register(&self.registry);

        let frame_actor = self.registry.find::<FrameActor>(&frame_offset.actor);
        frame_actor.set_offset(frame_offset.column, frame_offset.line);

        let msg = ThreadInterruptedReply {
            from: thread_actor.name(),
            type_: "paused".to_owned(),
            actor: pause_name,
            frame: frame_actor.encode(&self.registry),
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
        let Some(browsing_context_name) = self
            .pipelines
            .get(&pipeline_id)
            .and_then(|id| self.browsing_contexts.get(id))
        else {
            return;
        };

        let browsing_context_actor = self
            .registry
            .find::<BrowsingContextActor>(browsing_context_name);
        let thread_actor = self
            .registry
            .find::<ThreadActor>(&browsing_context_actor.thread_name);

        let source_name = match thread_actor
            .source_manager
            .find_source(&self.registry, &frame.url)
        {
            Some(source_actor) => source_actor.name(),
            None => {
                warn!("No source actor found for URL: {}", frame.url);
                return;
            },
        };

        let frame_name = FrameActor::register(&self.registry, source_name, frame);

        let _ = result_sender.send(frame_name);
    }

    fn handle_create_environment_actor(
        &mut self,
        result_sender: GenericSender<String>,
        environment_info: EnvironmentInfo,
        parent: Option<String>,
    ) {
        let environment_name = EnvironmentActor::register(&self.registry, environment_info, parent);
        let _ = result_sender.send(environment_name);
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
    registry: Arc<ActorRegistry>,
    mut stream: DevtoolsConnection,
    stream_id: StreamId,
    connections: Arc<Mutex<FxHashMap<StreamId, DevtoolsConnection>>>,
    sender: Sender<DevtoolsControlMsg>,
) {
    connections
        .lock()
        .unwrap()
        .insert(stream_id, stream.clone());

    log::info!("Connection established to {}", stream.peer_addr().unwrap());
    let msg = registry.encode::<RootActor, _>("root");
    if let Err(error) = stream.write_json_packet(&msg) {
        warn!("Failed to send initial packet from root actor: {error:?}");
        return;
    }

    loop {
        match stream.read_json_packet() {
            Ok(Some(json_packet)) => {
                if let Err(()) = registry.handle_message(
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

    connections.lock().unwrap().remove(&stream_id);
    let _ = sender.send(DevtoolsControlMsg::ClientExited);

    registry.cleanup(stream_id);
}

/// <https://searchfox.org/mozilla-central/source/devtools/server/actors/object/utils.js#148>
pub(crate) fn debugger_value_to_json(registry: &ActorRegistry, value: DebuggerValue) -> Value {
    let mut v = Map::new();
    match value {
        DebuggerValue::VoidValue => {
            v.insert("type".to_owned(), Value::String("undefined".to_owned()));
            Value::Object(v)
        },
        DebuggerValue::NullValue => {
            v.insert("type".to_owned(), Value::String("null".to_owned()));
            Value::Object(v)
        },
        DebuggerValue::BooleanValue(boolean) => Value::Bool(boolean),
        DebuggerValue::NumberValue(val) => {
            if val.is_nan() {
                v.insert("type".to_owned(), Value::String("NaN".to_owned()));
                Value::Object(v)
            } else if val.is_infinite() {
                if val < 0. {
                    v.insert("type".to_owned(), Value::String("-Infinity".to_owned()));
                } else {
                    v.insert("type".to_owned(), Value::String("Infinity".to_owned()));
                }
                Value::Object(v)
            } else if val == 0. && val.is_sign_negative() {
                v.insert("type".to_owned(), Value::String("-0".to_owned()));
                Value::Object(v)
            } else {
                Value::Number(Number::from_f64(val).unwrap())
            }
        },
        DebuggerValue::StringValue(str) => Value::String(str),
        DebuggerValue::ObjectValue {
            uuid,
            class,
            preview,
            ..
        } => {
            let object_name = ObjectActor::register(registry, Some(uuid), class, preview);
            let object_msg = registry.encode::<ObjectActor, _>(&object_name);
            let value = serde_json::to_value(object_msg).unwrap_or_default();
            Value::Object(value.as_object().cloned().unwrap_or_default())
        },
    }
}

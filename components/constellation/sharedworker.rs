/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#[cfg(feature = "webgpu")]
use std::cell::RefCell;
use std::collections::{HashMap, VecDeque};
use std::rc::Rc;
use std::time::Instant;

use crossbeam_channel::{Receiver, Sender, unbounded};
use log::{debug, error, warn};
use net_traits::pub_domains::reg_suffix;
use servo_base::generic_channel::{GenericCallback, SendError};
use servo_base::id::{
    BroadcastChannelRouterId, MessagePortId, MessagePortRouterId, PipelineId, ScriptEventLoopId,
    WebViewId, WorkerId,
};
use servo_constellation_traits::{
    MessagePortImpl, SharedWorkerClientMessage, SharedWorkerCommand, SharedWorkerConnect,
    SharedWorkerHostEvent, SharedWorkerOptions, SharedWorkerSource, SharedWorkerStart,
};
use servo_url::{Host, ImmutableOrigin, ServoUrl};

#[cfg(feature = "webgpu")]
use crate::constellation::BrowsingContextGroup;
use crate::event_loop::EventLoop;

mod connections;
mod lifecycle;

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
struct SharedWorkerKey {
    // A storage key is a tuple consisting of an origin (an origin).
    // This is expected to change; see Client-Side Storage Partitioning.
    // <https://storage.spec.whatwg.org/#storage-key>
    // Include constructor origin in the key so `data:` SharedWorkers are not reused across origins.
    constructor_origin: ImmutableOrigin,
    top_level_site: SharedWorkerSite,
    is_private: bool,
    constructor_url: ServoUrl,
    name: String,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
enum SharedWorkerSite {
    Opaque(ImmutableOrigin),
    Tuple { scheme: String, host: Host },
}

impl SharedWorkerSite {
    /// <https://html.spec.whatwg.org/multipage/#obtain-a-site>
    fn from_origin(origin: ImmutableOrigin) -> Self {
        match origin {
            // 1. If origin is an opaque origin, then return origin.
            ImmutableOrigin::Opaque(_) => Self::Opaque(origin),
            // 3. Return (origin's scheme, origin's host's registrable domain).
            ImmutableOrigin::Tuple(scheme, Host::Domain(domain), _) => Self::Tuple {
                scheme,
                host: Host::Domain(reg_suffix(&domain).to_owned()),
            },
            // 2. If origin's host's registrable domain is null, then return
            // (origin's scheme, origin's host).
            ImmutableOrigin::Tuple(scheme, host, _) => Self::Tuple { scheme, host },
        }
    }
}

pub(crate) struct SharedWorkerOwner {
    pub pipeline_id: PipelineId,
    pub host: Rc<EventLoop>,
    pub is_private: bool,
    #[cfg(feature = "webgpu")]
    pub browsing_context_group: Rc<RefCell<BrowsingContextGroup>>,
    #[cfg(feature = "webgpu")]
    pub url: ServoUrl,
}

#[derive(Clone, Copy, Debug)]
pub(crate) enum OriginQuery {
    Owner,
    TopLevel,
}

#[derive(Debug)]
pub(crate) enum ManagerEvent {
    HostExited(ScriptEventLoopId),
    Origin(MessagePortId, OriginQuery, Option<ImmutableOrigin>),
    Painter(WorkerId, bool),
    Host(WorkerId, Result<SharedWorkerHostEvent, SendError>),
}

pub(crate) enum ManagerAction {
    Start(Rc<EventLoop>, Box<SharedWorkerStart>),
    RetainPainter(WorkerId, WebViewId, GenericCallback<bool>),
    ReleasePainter(WorkerId),
    ClosePort(MessagePortId),
    RemoveMessageRouter(MessagePortRouterId),
    RemoveBroadcastRouter(BroadcastChannelRouterId),
}

struct PendingRequest {
    request: SharedWorkerConnect,
    owner: SharedWorkerOwner,
    top_host: Rc<EventLoop>,
    owner_origin: Option<ImmutableOrigin>,
    top_origin: Option<ImmutableOrigin>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Phase {
    Starting,
    Loading,
    Running,
    Closing,
}

struct Connection {
    owner: PipelineId,
    caller_is_secure: bool,
    enable_port: bool,
    port_id: MessagePortId,
    port: Option<MessagePortImpl>,
    received: bool,
    reply: GenericCallback<SharedWorkerClientMessage>,
}

// A `SharedWorkerGlobalScope` object has associated constructor origin (an origin), constructor URL (a URL record), and credentials (a credentials mode), and extended lifetime (a boolean).
struct Worker {
    key: SharedWorkerKey,
    options: SharedWorkerOptions,
    context: SharedWorkerOwner,
    event_loop_id: ScriptEventLoopId,
    phase: Phase,
    secure_context: Option<bool>,
    commands: Option<GenericCallback<SharedWorkerCommand>>,
    pending_start: Option<SharedWorkerStart>,
    /// <https://html.spec.whatwg.org/multipage/workers.html#concept-WorkerGlobalScope-owner-set>
    /// "A WorkerGlobalScope object has an associated owner set."
    /// SharedWorker is exposed to Window; None denotes a fully active Document,
    /// and Some records when that owner became non-fully active.
    owners: HashMap<PipelineId, Option<Instant>>,
    owner_set_empty_since: Option<Instant>,
    suspended: bool,
    connections: VecDeque<(MessagePortId, Connection)>,
    csp_client: Option<(PipelineId, GenericCallback<SharedWorkerClientMessage>)>,
    globals: HashMap<WorkerId, ImmutableOrigin>,
    message_routers: HashMap<MessagePortRouterId, WorkerId>,
    broadcast_routers: HashMap<BroadcastChannelRouterId, WorkerId>,
    deadline: Option<Instant>,
}

// A user agent has an associated shared worker manager which is the result of starting a new parallel queue.
// Each user agent has a single shared worker manager for simplicity.
pub(crate) struct SharedWorkerManager {
    workers: HashMap<WorkerId, Worker>,
    keys: HashMap<SharedWorkerKey, WorkerId>,
    pending: VecDeque<PendingRequest>,
    sender: Sender<ManagerEvent>,
    receiver: Receiver<ManagerEvent>,
    actions: Vec<ManagerAction>,
    online: bool,
    shutting_down: bool,
}

impl Default for SharedWorkerManager {
    fn default() -> Self {
        let (sender, receiver) = unbounded();
        Self {
            workers: HashMap::new(),
            keys: HashMap::new(),
            pending: VecDeque::new(),
            sender,
            receiver,
            actions: Vec::new(),
            online: true,
            shutting_down: false,
        }
    }
}

impl SharedWorkerManager {
    pub(crate) fn host_exit_callback(
        &self,
        host: ScriptEventLoopId,
    ) -> Result<GenericCallback<()>, SendError> {
        let sender = self.sender.clone();
        GenericCallback::new(move |result| {
            if let Err(error) = result {
                warn!("Invalid script-host exit notification: {error}");
            }
            if sender.send(ManagerEvent::HostExited(host)).is_err() {
                debug!("Constellation stopped before script-host exit");
            }
        })
    }

    pub(crate) fn receiver(&self) -> &Receiver<ManagerEvent> {
        &self.receiver
    }

    pub(crate) fn take_actions(&mut self) -> Vec<ManagerAction> {
        std::mem::take(&mut self.actions)
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.workers.is_empty() && self.pending.is_empty()
    }

    pub(crate) fn next_deadline(&self) -> Option<Instant> {
        self.workers
            .values()
            .filter_map(|worker| worker.deadline)
            .min()
    }

    pub(crate) fn handle_event(&mut self, event: ManagerEvent, now: Instant) {
        match event {
            ManagerEvent::HostExited(host) => self.host_exited(host, now),
            ManagerEvent::Origin(connection, query, origin) => {
                self.resolve_origin(connection, query, origin);
                self.process_pending(now);
            },
            ManagerEvent::Painter(worker_id, retained) => {
                let Some(worker) = self.workers.get_mut(&worker_id) else {
                    if retained {
                        self.actions.push(ManagerAction::ReleasePainter(worker_id));
                    }
                    return;
                };
                if !retained {
                    self.start_failed(worker_id);
                    return;
                }
                let Some(mut start) = worker.pending_start.take() else {
                    warn!("Duplicate SharedWorker painter acknowledgement");
                    return;
                };
                start.creation.init.online = self.online;
                self.actions.push(ManagerAction::Start(
                    worker.context.host.clone(),
                    Box::new(start),
                ));
            },
            ManagerEvent::Host(worker_id, result) => {
                let event = match result {
                    Ok(event) => event,
                    Err(error) => {
                        error!("Invalid SharedWorker host event: {error}");
                        self.stop(worker_id, true);
                        return;
                    },
                };
                let Some(worker) = self.workers.get_mut(&worker_id) else {
                    if let SharedWorkerHostEvent::Started(commands) = event {
                        if let Err(error) = commands.send(SharedWorkerCommand::Terminate) {
                            warn!("Failed to terminate obsolete SharedWorker host: {error}");
                        }
                    }
                    return;
                };
                match event {
                    SharedWorkerHostEvent::Started(commands) => {
                        worker.commands = Some(commands);
                        if worker.phase == Phase::Closing {
                            self.stop(worker_id, false);
                        } else {
                            if let Some(commands) = &worker.commands {
                                if let Err(error) = commands
                                    .send(SharedWorkerCommand::SetSuspended(worker.suspended))
                                {
                                    warn!("Failed to initialize SharedWorker suspension: {error}");
                                    self.stop(worker_id, true);
                                    return;
                                }
                                if let Err(error) = commands
                                    .send(SharedWorkerCommand::SetNetworkOnlineState(self.online))
                                {
                                    warn!(
                                        "Failed to initialize SharedWorker network state: {error}"
                                    );
                                    self.stop(worker_id, true);
                                    return;
                                }
                            }
                            self.flush_connections(worker_id, now);
                        }
                    },
                    SharedWorkerHostEvent::ScopeCreated { secure_context } => {
                        if worker.phase != Phase::Closing {
                            worker.secure_context = Some(secure_context);
                            worker.phase = Phase::Loading;
                            self.flush_connections(worker_id, now);
                        }
                    },
                    SharedWorkerHostEvent::ScriptLoaded => {
                        if worker.phase != Phase::Closing {
                            worker.phase = Phase::Running;
                            self.complete_connections(worker_id);
                        }
                    },
                    SharedWorkerHostEvent::ScriptLoadFailed => self.stop(worker_id, true),
                    SharedWorkerHostEvent::PortReceived(connection_id) => {
                        if let Some((_, connection)) = worker
                            .connections
                            .iter_mut()
                            .find(|(id, _)| *id == connection_id)
                        {
                            connection.received = true;
                        }
                        self.complete_connections(worker_id);
                    },
                    SharedWorkerHostEvent::CspViolations(violations) => {
                        if let Some((_, reply)) = &worker.csp_client {
                            Self::reply(
                                reply,
                                SharedWorkerClientMessage::CspViolations(violations),
                            );
                        }
                    },
                    SharedWorkerHostEvent::Closing => {
                        worker.phase = Phase::Closing;
                        if self.keys.get(&worker.key) == Some(&worker_id) {
                            self.keys.remove(&worker.key);
                        }
                    },
                    SharedWorkerHostEvent::Exited => {
                        let failed = worker.phase != Phase::Closing;
                        self.finish(worker_id, failed);
                    },
                }
            },
        }
    }

    pub(crate) fn register_global(
        &mut self,
        parent: SharedWorkerSource,
        global_id: WorkerId,
        origin: ImmutableOrigin,
    ) {
        let Some(worker) = self.workers.get_mut(&parent.worker_id) else {
            warn!("Registering a global for an unknown SharedWorker");
            return;
        };
        if global_id != parent.worker_id && !worker.globals.contains_key(&parent.global_id) {
            warn!("Registering a SharedWorker descendant with an unknown parent");
            return;
        }
        if worker.globals.contains_key(&global_id) {
            warn!("Duplicate SharedWorker global registration");
            return;
        }
        worker.globals.insert(global_id, origin);
    }

    pub(crate) fn source_origin(&self, source: SharedWorkerSource) -> Option<&ImmutableOrigin> {
        self.workers
            .get(&source.worker_id)?
            .globals
            .get(&source.global_id)
    }

    #[cfg(feature = "webgpu")]
    pub(crate) fn context(&self, worker: WorkerId) -> Option<&SharedWorkerOwner> {
        self.workers.get(&worker).map(|worker| &worker.context)
    }

    pub(crate) fn unregister_global(&mut self, source: SharedWorkerSource) {
        let Some(worker) = self.workers.get_mut(&source.worker_id) else {
            return;
        };
        worker.globals.remove(&source.global_id);
        worker.message_routers.retain(|router, global| {
            if *global != source.global_id {
                return true;
            }
            self.actions
                .push(ManagerAction::RemoveMessageRouter(*router));
            false
        });
        worker.broadcast_routers.retain(|router, global| {
            if *global != source.global_id {
                return true;
            }
            self.actions
                .push(ManagerAction::RemoveBroadcastRouter(*router));
            false
        });
    }

    pub(crate) fn track_message_router(
        &mut self,
        source: SharedWorkerSource,
        router: MessagePortRouterId,
    ) {
        if let Some(worker) = self.workers.get_mut(&source.worker_id) {
            worker.message_routers.insert(router, source.global_id);
        }
    }

    pub(crate) fn untrack_message_router(
        &mut self,
        source: SharedWorkerSource,
        router: MessagePortRouterId,
    ) {
        if let Some(worker) = self.workers.get_mut(&source.worker_id) {
            worker.message_routers.remove(&router);
        }
    }

    pub(crate) fn track_broadcast_router(
        &mut self,
        source: SharedWorkerSource,
        router: BroadcastChannelRouterId,
    ) {
        if let Some(worker) = self.workers.get_mut(&source.worker_id) {
            worker.broadcast_routers.insert(router, source.global_id);
        }
    }

    pub(crate) fn untrack_broadcast_router(
        &mut self,
        source: SharedWorkerSource,
        router: BroadcastChannelRouterId,
    ) {
        if let Some(worker) = self.workers.get_mut(&source.worker_id) {
            worker.broadcast_routers.remove(&router);
        }
    }
}

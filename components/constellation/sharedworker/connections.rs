/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::collections::{HashMap, VecDeque};
use std::rc::Rc;
use std::time::Instant;

use log::{debug, error, warn};
use script_traits::ScriptThreadMessage;
use servo_base::generic_channel::GenericCallback;
use servo_base::id::{MessagePortId, PipelineId, ScriptEventLoopId, WorkerId};
use servo_constellation_traits::{
    SharedWorkerClientMessage, SharedWorkerCommand, SharedWorkerConnect, SharedWorkerSource,
    SharedWorkerStart,
};
use servo_url::ImmutableOrigin;

use super::{
    Connection, ManagerAction, ManagerEvent, OriginQuery, PendingRequest, Phase, SharedWorkerKey,
    SharedWorkerManager, SharedWorkerOwner, SharedWorkerSite, Worker,
};
use crate::event_loop::EventLoop;

impl SharedWorkerManager {
    pub(crate) fn request(
        &mut self,
        request: SharedWorkerConnect,
        owner: SharedWorkerOwner,
        top_host: Rc<EventLoop>,
        top_pipeline: PipelineId,
        now: Instant,
    ) {
        if self.shutting_down {
            self.reject_request(request, "browser shutdown");
            return;
        }
        let connection_id = request.connection_id;
        let owner_host = owner.host.clone();
        let owner_pipeline = owner.pipeline_id;
        self.pending.push_back(PendingRequest {
            request,
            owner,
            top_host: top_host.clone(),
            owner_origin: None,
            top_origin: None,
        });
        self.query_origin(
            connection_id,
            OriginQuery::Owner,
            owner_pipeline,
            &owner_host,
        );
        self.query_origin(
            connection_id,
            OriginQuery::TopLevel,
            top_pipeline,
            &top_host,
        );
        self.process_pending(now);
    }

    fn query_origin(
        &mut self,
        connection: MessagePortId,
        query: OriginQuery,
        pipeline: PipelineId,
        host: &EventLoop,
    ) {
        let sender = self.sender.clone();
        let reply = GenericCallback::new(move |result| {
            let origin = match result {
                Ok(origin) => origin,
                Err(error) => {
                    warn!("Failed to resolve SharedWorker origin: {error}");
                    None
                },
            };
            if sender
                .send(ManagerEvent::Origin(connection, query, origin))
                .is_err()
            {
                debug!("SharedWorker manager stopped while resolving an origin");
            }
        });
        let result = reply.and_then(|reply| {
            host.send(ScriptThreadMessage::GetSharedWorkerOrigin(pipeline, reply))
        });
        if let Err(error) = result {
            warn!("Could not query SharedWorker origin: {error}");
            self.resolve_origin(connection, query, None);
        }
    }

    pub(super) fn resolve_origin(
        &mut self,
        connection: MessagePortId,
        query: OriginQuery,
        origin: Option<ImmutableOrigin>,
    ) {
        let Some(index) = self
            .pending
            .iter()
            .position(|pending| pending.request.connection_id == connection)
        else {
            return;
        };
        let Some(origin) = origin else {
            let pending = self.pending.remove(index).expect("pending request exists");
            self.reject_request(pending.request, "owner or top-level Document disappeared");
            return;
        };
        match query {
            OriginQuery::Owner => self.pending[index].owner_origin = Some(origin),
            OriginQuery::TopLevel => self.pending[index].top_origin = Some(origin),
        }
    }

    pub(super) fn process_pending(&mut self, now: Instant) {
        while let Some(index) = self
            .pending
            .iter()
            .enumerate()
            .find_map(|(index, pending)| {
                if pending.owner_origin.is_none() || pending.top_origin.is_none() {
                    return None;
                }
                let earlier_match = self.pending.iter().take(index).any(|earlier| {
                    earlier.owner.is_private == pending.owner.is_private &&
                        earlier.request.creation.init.origin ==
                            pending.request.creation.init.origin &&
                        earlier.request.creation.url.url() == pending.request.creation.url.url() &&
                        earlier.request.creation.options.name ==
                            pending.request.creation.options.name
                });
                (!earlier_match).then_some(index)
            })
        {
            let pending = self.pending.remove(index).expect("pending request exists");
            let owner_origin = pending.owner_origin.expect("owner origin was resolved");
            let top_origin = pending.top_origin.expect("top-level origin was resolved");
            if owner_origin != pending.request.creation.init.origin {
                self.reject_request(
                    pending.request,
                    "constructor origin does not match its Document",
                );
                continue;
            }
            self.connect(
                pending.request,
                pending.owner,
                SharedWorkerSite::from_origin(top_origin),
                now,
            );
        }
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-sharedworker>
    /// <https://html.spec.whatwg.org/multipage/#shared-worker-manager>
    fn connect(
        &mut self,
        request: SharedWorkerConnect,
        owner: SharedWorkerOwner,
        top_level_site: SharedWorkerSite,
        now: Instant,
    ) {
        let Some(caller_is_secure) = request.creation.init.inherited_secure_context else {
            self.reject_request(request, "missing constructor secure-context state");
            return;
        };
        let key = SharedWorkerKey {
            constructor_origin: request.creation.init.origin.clone(),
            constructor_url: request.creation.url.url(),
            name: request.creation.options.name.clone(),
            top_level_site,
            is_private: owner.is_private,
        };

        // Step 11.1. Let workerGlobalScope be null.
        // Step 11.2. For each scope in the list of all `SharedWorkerGlobalScope` objects:
        // Step 11.2.1. Let workerStorageKey be the result of running obtain a storage key for non-storage purposes given scope's relevant settings object.
        // Step 11.2.2. If all of the following are true:
        // workerStorageKey equals outsideStorageKey;
        // scope's closing flag is false;
        // scope's constructor URL equals urlRecord; and
        // scope's name equals options["name"],
        let matching_worker = self.keys.get(&key).copied();
        if let Some(worker_id) = matching_worker {
            // Step 11.2.2.1. Set workerGlobalScope to scope.
            // Step 11.2.2.2. Break.
            let worker = self
                .workers
                .get_mut(&worker_id)
                .expect("key must identify a worker");
            // TODO Step 11.3. If workerGlobalScope is not null, but the user agent has been configured to disallow communication between the worker represented by the workerGlobalScope and the scripts whose settings object is outsideSettings, then set workerGlobalScope to null.
            // Step 11.4. If workerGlobalScope is not null, and any of the following are true:
            // workerGlobalScope's type is not equal to options["type"];
            // workerGlobalScope's credentials is not equal to options["credentials"]; or
            // workerGlobalScope's extended lifetime is not equal to options["extendedLifetime"],
            if worker.options != request.creation.options {
                // Step 11.4.1. Queue a global task on the DOM manipulation task source given worker's relevant global object to fire an event named error at worker.
                // Step 11.4.2. Abort these steps.
                self.reject_request(request, "worker options do not match");
                return;
            }
            // Step 11.5. If workerGlobalScope is not null:
            // Step 11.5.1. Let insideSettings be workerGlobalScope's relevant settings object.
            // Step 11.5.2. Let workerIsSecureContext be true if insideSettings is a secure context; otherwise, false.
            // Step 11.5.3. If workerIsSecureContext is not callerIsSecureContext:
            if worker
                .secure_context
                .is_some_and(|secure| secure != caller_is_secure)
            {
                // Step 11.5.3.1. Queue a global task on the DOM manipulation task source given worker's relevant global object to fire an event named error at worker.
                // Step 11.5.3.2. Abort these steps.
                self.reject_request(request, "worker secure context does not match");
                return;
            }
            // Step 11.5.4. Associate worker with workerGlobalScope.
            worker.connections.push_back((
                request.connection_id,
                Connection {
                    owner: owner.pipeline_id,
                    caller_is_secure,
                    enable_port: false,
                    port_id: *request.port.message_port_id(),
                    port: Some(request.port),
                    received: false,
                    reply: request.reply,
                },
            ));
            worker.deadline = None;
            self.flush_connections(worker_id, now);
            return;
        }

        // Step 11.6. Otherwise, in parallel, run a worker given worker, urlRecord, outsideSettings, outsidePort, and options.
        let worker_id = request.creation.init.worker_id;
        if self.workers.contains_key(&worker_id) {
            self.reject_request(request, "duplicate worker incarnation");
            return;
        }
        let sender = self.sender.clone();
        let events = GenericCallback::new(move |event| {
            if sender.send(ManagerEvent::Host(worker_id, event)).is_err() {
                debug!("SharedWorker manager stopped before worker notification");
            }
        });
        let sender = self.sender.clone();
        let painter_reply = GenericCallback::new(move |result| {
            let retained = match result {
                Ok(retained) => retained,
                Err(error) => {
                    warn!("SharedWorker painter lease failed: {error}");
                    false
                },
            };
            if sender
                .send(ManagerEvent::Painter(worker_id, retained))
                .is_err()
            {
                debug!("SharedWorker manager stopped before painter acknowledgement");
            }
        });
        let (events, painter_reply) = match (events, painter_reply) {
            (Ok(events), Ok(painter_reply)) => (events, painter_reply),
            (events, painter_reply) => {
                error!(
                    "Could not create SharedWorker manager endpoints: {events:?}, {painter_reply:?}"
                );
                self.reject_request(request, "could not create worker endpoints");
                return;
            },
        };
        let webview_id = request.creation.webview_id;
        let csp_client = Some((owner.pipeline_id, request.reply.clone()));
        let connection = Connection {
            owner: owner.pipeline_id,
            caller_is_secure,
            enable_port: true,
            port_id: *request.port.message_port_id(),
            port: Some(request.port),
            received: false,
            reply: request.reply,
        };
        let mut creation = request.creation;
        creation.init.shared_worker = Some(SharedWorkerSource {
            worker_id,
            global_id: worker_id,
        });
        let event_loop_id = ScriptEventLoopId::new();
        let owner_pipeline_id = owner.pipeline_id;
        let worker = Worker {
            key: key.clone(),
            options: creation.options.clone(),
            context: owner,
            event_loop_id,
            phase: Phase::Starting,
            secure_context: None,
            commands: None,
            pending_start: Some(SharedWorkerStart {
                creation,
                event_loop_id,
                events,
            }),
            owners: HashMap::from([(owner_pipeline_id, None)]),
            owner_set_empty_since: None,
            suspended: true,
            connections: VecDeque::from([(request.connection_id, connection)]),
            csp_client,
            globals: HashMap::new(),
            message_routers: HashMap::new(),
            broadcast_routers: HashMap::new(),
            deadline: None,
        };
        self.workers.insert(worker_id, worker);
        self.keys.insert(key, worker_id);
        self.actions.push(ManagerAction::RetainPainter(
            worker_id,
            webview_id,
            painter_reply,
        ));
    }

    pub(super) fn flush_connections(&mut self, worker_id: WorkerId, now: Instant) {
        let Some(worker) = self.workers.get_mut(&worker_id) else {
            return;
        };
        if worker.phase == Phase::Closing {
            return;
        }
        let (Some(secure), Some(commands)) = (worker.secure_context, &worker.commands) else {
            return;
        };
        let mut rejected = Vec::new();
        let mut failed = false;
        for (id, connection) in &mut worker.connections {
            if connection.caller_is_secure != secure {
                rejected.push(*id);
                continue;
            }
            // Step 11.5.8. Append the relevant owner to add given outsideSettings to workerGlobalScope's owner set.
            worker.owners.entry(connection.owner).or_insert(Some(now));
            worker.owner_set_empty_since = None;
            if let Some(port) = connection.port.take() {
                // Step 11.5.7. Queue a global task on the DOM manipulation task source given workerGlobalScope to fire an event named connect at workerGlobalScope, using MessageEvent, with the data attribute initialized to the empty string, the ports attribute initialized to a new frozen array containing only insidePort, and the source attribute initialized to insidePort.
                // Step 13. If is shared is true, then queue a global task on the DOM
                // manipulation task source given worker global scope to fire an event
                // named connect at worker global scope, using MessageEvent, with the data
                // attribute initialized to the empty string, the ports attribute
                // initialized to a new frozen array containing inside port, and the
                // source attribute initialized to inside port.
                if let Err(error) = commands.send(SharedWorkerCommand::Connect {
                    connection_id: *id,
                    port,
                }) {
                    warn!("Failed to connect to SharedWorker: {error}");
                    failed = true;
                    break;
                }
            }
        }
        worker.connections.retain(|(id, connection)| {
            if rejected.contains(id) {
                Self::reply(&connection.reply, SharedWorkerClientMessage::Error);
                self.actions
                    .push(ManagerAction::ClosePort(connection.port_id));
                return false;
            }
            true
        });
        if failed {
            self.stop(worker_id, true);
        }
    }

    pub(super) fn complete_connections(&mut self, worker_id: WorkerId) {
        let Some(worker) = self.workers.get_mut(&worker_id) else {
            return;
        };
        if worker.phase != Phase::Running {
            return;
        }
        worker.connections.retain(|(_, connection)| {
            if !connection.received {
                return true;
            }
            Self::reply(
                &connection.reply,
                SharedWorkerClientMessage::Connected {
                    enable_port: connection.enable_port,
                },
            );
            false
        });
    }

    pub(super) fn reply(
        reply: &GenericCallback<SharedWorkerClientMessage>,
        message: SharedWorkerClientMessage,
    ) {
        if let Err(error) = reply.send(message) {
            warn!("SharedWorker client reply failed: {error}");
        }
    }

    pub(crate) fn reject_request(&mut self, request: SharedWorkerConnect, reason: &str) {
        warn!("SharedWorker connection rejected: {reason}");
        Self::reply(&request.reply, SharedWorkerClientMessage::Error);
        self.actions
            .push(ManagerAction::ClosePort(*request.port.message_port_id()));
    }
}

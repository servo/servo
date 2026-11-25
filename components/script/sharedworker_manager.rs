/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

use content_security_policy::Violation;
use crossbeam_channel::{Receiver, Sender, select, unbounded};
use fonts::FontContext;
use net_traits::image_cache::ImageCacheFactory;
use net_traits::pub_domains::Site;
use paint_api::CrossProcessPaintApi;
use servo_base::generic_channel::{GenericCallback, RoutedReceiver};
use servo_base::id::{MessagePortId, PipelineId, PipelineNamespace, ScriptEventLoopId, WorkerId};
use servo_config::pref;
use servo_constellation_traits::{
    MessagePortImpl, ScriptToConstellationChan, ScriptToConstellationMessage,
    SharedWorkerClientMessage, SharedWorkerConnect, SharedWorkerManagerMessage,
    SharedWorkerManagerSenders, SharedWorkerOptions,
};
use servo_url::{ImmutableOrigin, ServoUrl};

#[cfg(feature = "webgpu")]
use crate::dom::webgpu::identityhub::IdentityHub;
use crate::event_loop::svg_font::SvgFontResolver;
use crate::sharedworker_host::{SharedWorkerStart, SharedWorkerThread};

// <https://html.spec.whatwg.org/multipage/#between-loads-shared-worker-timeout>
// A typical value for the between-loads shared worker timeout might be 5 seconds.
const BETWEEN_LOADS_TIMEOUT: Duration = Duration::from_secs(5);

pub(crate) enum SharedWorkerEvent {
    ScopeCreated { secure_context: bool },
    ScriptLoaded,
    ScriptLoadFailed,
    PortReceived(MessagePortId),
    CspViolations(Vec<Violation>),
    Closing,
    Exited { panicked: bool },
}

/// <https://html.spec.whatwg.org/multipage/#dom-sharedworker>
#[derive(Clone, Eq, Hash, PartialEq)]
struct WorkerKey {
    // Step 11.3. If workerGlobalScope is not null, but the user agent has been configured to disallow communication between the worker represented by the workerGlobalScope and the scripts whose settings object is outsideSettings, then set workerGlobalScope to null.
    top_level_site: Option<Site>,
    url: ServoUrl,
    name: String,
}

struct Connection {
    owner: PipelineId,
    caller_is_secure: bool,
    enable_port: bool,
    port_id: MessagePortId,
    entangled_port_id: Option<MessagePortId>,
    port: Option<MessagePortImpl>,
    received: bool,
    reply: GenericCallback<SharedWorkerClientMessage>,
    constellation: ScriptToConstellationChan,
}

impl Connection {
    fn close(&self) {
        if let Err(error) = self
            .constellation
            .send(ScriptToConstellationMessage::DisentanglePorts(
                self.port_id,
                self.entangled_port_id,
            ))
        {
            warn!("Failed to close SharedWorker connection: {error}");
        }
    }

    fn reply(&self, message: SharedWorkerClientMessage) {
        if let Err(error) = self.reply.send(message) {
            warn!("Failed to reply to SharedWorker client: {error}");
        }
    }
}

struct Worker {
    key: WorkerKey,
    options: SharedWorkerOptions,
    thread: SharedWorkerThread,
    secure_context: Option<bool>,
    loaded: bool,
    /// <https://html.spec.whatwg.org/multipage/#concept-WorkerGlobalScope-owner-set>
    /// A WorkerGlobalScope object has an associated owner set (a set of Document and WorkerGlobalScope objects).
    owners: HashMap<PipelineId, Option<Instant>>,
    owner_set_empty_since: Option<Instant>,
    connections: VecDeque<(MessagePortId, Connection)>,
    csp_client: Option<(PipelineId, GenericCallback<SharedWorkerClientMessage>)>,
    deadline: Option<Instant>,
}

/// <https://html.spec.whatwg.org/multipage/#shared-worker-manager>
/// A user agent has an associated shared worker manager which is the result of starting a new parallel queue.
pub(crate) struct SharedWorkerManager {
    origin: ImmutableOrigin,
    workers: HashMap<WorkerId, Worker>,
    keys: HashMap<WorkerKey, WorkerId>,
    receiver: RoutedReceiver<SharedWorkerManagerMessage>,
    events: Receiver<(WorkerId, SharedWorkerEvent)>,
    event_sender: Sender<(WorkerId, SharedWorkerEvent)>,
    fully_active: HashMap<PipelineId, bool>,
    has_loading_document: bool,
    online: bool,
    font_context: Arc<FontContext>,
    paint_api: CrossProcessPaintApi,
    image_cache_factory: Arc<dyn ImageCacheFactory>,
}

impl SharedWorkerManager {
    pub(crate) fn create(
        senders: SharedWorkerManagerSenders,
        origin: ImmutableOrigin,
        image_cache_factory: Arc<dyn ImageCacheFactory>,
    ) -> JoinHandle<()> {
        thread::Builder::new()
            .name("SharedWorkerManager".to_owned())
            .spawn(move || {
                let _lifeline = senders.lifeline_sender;
                PipelineNamespace::set_installer_sender(senders.namespace_request_sender);
                PipelineNamespace::auto_install();
                let (event_sender, events) = unbounded();
                let font_context = Arc::new(FontContext::new(
                    Arc::new(senders.system_font_service_sender.to_proxy()),
                    senders.paint_api.clone(),
                    senders.resource_threads,
                ));
                Self {
                    origin,
                    workers: HashMap::new(),
                    keys: HashMap::new(),
                    receiver: senders.receiver.route_preserving_errors(),
                    events,
                    event_sender,
                    fully_active: HashMap::new(),
                    has_loading_document: false,
                    online: true,
                    font_context,
                    paint_api: senders.paint_api,
                    image_cache_factory,
                }
                .run();
            })
            .expect("Failed to start SharedWorker manager")
    }

    fn run(&mut self) {
        loop {
            let timeout = self
                .workers
                .values()
                .filter_map(|worker| worker.deadline)
                .min()
                .map(|deadline| {
                    crossbeam_channel::after(deadline.saturating_duration_since(Instant::now()))
                })
                .unwrap_or_else(crossbeam_channel::never);
            select! {
                recv(self.receiver) -> message => match message {
                    Ok(Ok(SharedWorkerManagerMessage::Connect(request))) => self.connect(*request),
                    Ok(Ok(SharedWorkerManagerMessage::UpdateOwners { fully_active, has_loading_document })) => {
                        self.fully_active = fully_active;
                        self.has_loading_document = has_loading_document;
                    },
                    Ok(Ok(SharedWorkerManagerMessage::RemoveOwner(owner))) => self.remove_owner(owner),
                    Ok(Ok(SharedWorkerManagerMessage::SetNetworkOnlineState(online))) => {
                        self.online = online;
                        for worker in self.workers.values() {
                            worker.thread.set_online(online);
                        }
                    },
                    Ok(Ok(SharedWorkerManagerMessage::Exit)) | Err(_) => break,
                    Ok(Err(error)) => {
                        warn!("Invalid SharedWorker manager message: {error}");
                        break;
                    },
                },
                recv(self.events) -> event => {
                    if let Ok((id, event)) = event {
                        self.handle_event(id, event);
                    }
                },
                recv(timeout) -> _ => {},
            }
            self.update_lifetimes(Instant::now());
        }
        for worker in self.workers.values() {
            worker.thread.terminate();
        }
        let workers: Vec<_> = self.workers.keys().copied().collect();
        for id in workers {
            self.finish(id, true);
        }
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-sharedworker>
    fn connect(&mut self, request: SharedWorkerConnect) {
        let owner = request.creation.init.pipeline_id;
        let caller_is_secure = request.creation.init.inherited_secure_context;
        let origin = request.creation.init.origin.clone();
        let key = WorkerKey {
            top_level_site: request.creation.top_level_site.clone(),
            url: request.creation.url.url(),
            name: request.creation.options.name.clone(),
        };
        let connection_id = request.connection_id;
        let connection = Connection {
            owner,
            caller_is_secure: caller_is_secure.unwrap_or(false),
            enable_port: false,
            port_id: *request.port.message_port_id(),
            entangled_port_id: request.port.entangled_port_id(),
            port: Some(request.port),
            received: false,
            reply: request.reply,
            constellation: ScriptToConstellationChan {
                sender: request.creation.init.script_to_constellation_chan.clone(),
                webview_id: request.creation.webview_id,
                pipeline_id: owner,
            },
        };
        if origin != self.origin || caller_is_secure.is_none() {
            connection.reply(SharedWorkerClientMessage::Error);
            connection.close();
            return;
        }
        // Step 11.2.1. If all of the following are true:
        // scope's constructor storage key equals outsideStorageKey;
        // scope's closing flag is false;
        // scope's constructor URL equals urlRecord; and
        // scope's name equals options["name"],
        let existing = self.keys.get(&key).copied().filter(|id| {
            self.workers
                .get(id)
                .is_some_and(|worker| !worker.thread.is_closing())
        });
        if let Some(id) = existing {
            let worker = self
                .workers
                .get_mut(&id)
                .expect("SharedWorker key must exist");
            // Step 11.4. If workerGlobalScope is not null, and any of the following are true:
            // workerGlobalScope's type is not equal to options["type"];
            // workerGlobalScope's credentials is not equal to options["credentials"]; or
            // workerGlobalScope's extended lifetime is not equal to options["extendedLifetime"],
            if worker.options != request.creation.options {
                // Step 11.4.1. Queue a global task on the DOM manipulation task source given worker's relevant global object to fire an event named error at worker.
                connection.reply(SharedWorkerClientMessage::Error);
                connection.close();
                return;
            }
            // Step 11.5.3. If workerIsSecureContext is not callerIsSecureContext:
            if worker
                .secure_context
                .is_some_and(|secure| secure != connection.caller_is_secure)
            {
                // Step 11.5.3.1. Queue a global task on the DOM manipulation task source given worker's relevant global object to fire an event named error at worker.
                connection.reply(SharedWorkerClientMessage::Error);
                connection.close();
                return;
            }
            worker.connections.push_back((connection_id, connection));
            self.flush_connections(id);
            return;
        }
        let mut creation = request.creation;
        creation.init.online = self.online;
        let id = creation.init.worker_id;
        if self.workers.contains_key(&id) {
            connection.reply(SharedWorkerClientMessage::Error);
            connection.close();
            return;
        }
        let font_context = Arc::new(self.font_context.new_for_worker());
        let image_cache = self.image_cache_factory.create(
            creation.webview_id,
            owner,
            &self.paint_api,
            Arc::new(SvgFontResolver::new(font_context.clone())),
        );
        let options = creation.options.clone();
        let csp_client = Some((owner, connection.reply.clone()));
        // Step 11.6. Otherwise, in parallel, run a worker given worker, urlRecord, outsideSettings, outsidePort, and options.
        let thread = match SharedWorkerThread::spawn(
            SharedWorkerStart {
                creation,
                event_loop_id: ScriptEventLoopId::new(),
                events: self.event_sender.clone(),
            },
            image_cache,
            font_context,
            #[cfg(feature = "webgpu")]
            Arc::new(IdentityHub::default()),
        ) {
            Ok(thread) => thread,
            Err(error) => {
                warn!("Failed to start SharedWorker: {error}");
                connection.reply(SharedWorkerClientMessage::Error);
                connection.close();
                return;
            },
        };
        self.keys.insert(key.clone(), id);
        self.workers.insert(
            id,
            Worker {
                key,
                options,
                thread,
                secure_context: None,
                loaded: false,
                owners: HashMap::from([(owner, None)]),
                owner_set_empty_since: None,
                connections: VecDeque::from([(
                    connection_id,
                    Connection {
                        enable_port: true,
                        ..connection
                    },
                )]),
                csp_client,
                deadline: None,
            },
        );
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-sharedworker>
    fn flush_connections(&mut self, id: WorkerId) {
        let Some(worker) = self.workers.get_mut(&id) else {
            return;
        };
        let Some(secure) = worker.secure_context else {
            return;
        };
        let mut failed = false;
        worker
            .connections
            .retain_mut(|(connection_id, connection)| {
                if secure != connection.caller_is_secure {
                    connection.reply(SharedWorkerClientMessage::Error);
                    connection.close();
                    return false;
                }
                // Step 11.5.8. Append the relevant owner to add given outsideSettings to workerGlobalScope's owner set.
                worker.owners.entry(connection.owner).or_insert(None);
                worker.owner_set_empty_since = None;
                if let Some(port) = connection.port.take() {
                    failed |= !worker.thread.connect(*connection_id, port);
                }
                true
            });
        if failed {
            self.finish(id, true);
        }
    }

    fn handle_event(&mut self, id: WorkerId, event: SharedWorkerEvent) {
        let Some(worker) = self.workers.get_mut(&id) else {
            return;
        };
        match event {
            SharedWorkerEvent::ScopeCreated { secure_context } => {
                worker.secure_context = Some(secure_context);
                self.flush_connections(id);
            },
            SharedWorkerEvent::ScriptLoaded => worker.loaded = true,
            SharedWorkerEvent::PortReceived(connection_id) => {
                if let Some((_, connection)) = worker
                    .connections
                    .iter_mut()
                    .find(|(other, _)| *other == connection_id)
                {
                    connection.received = true;
                }
            },
            SharedWorkerEvent::ScriptLoadFailed => self.finish(id, true),
            SharedWorkerEvent::CspViolations(violations) => {
                if let Some((_, reply)) = &worker.csp_client {
                    let _ = reply.send(SharedWorkerClientMessage::CspViolations(violations));
                }
            },
            SharedWorkerEvent::Closing => {
                if self.keys.get(&worker.key) == Some(&id) {
                    self.keys.remove(&worker.key);
                }
            },
            SharedWorkerEvent::Exited { panicked } => self.finish(id, panicked),
        }
        if let Some(worker) = self.workers.get_mut(&id) &&
            worker.loaded
        {
            worker.connections.retain(|(_, connection)| {
                if !connection.received {
                    return true;
                }
                connection.reply(SharedWorkerClientMessage::Connected {
                    enable_port: connection.enable_port,
                });
                false
            });
        }
    }

    fn finish(&mut self, id: WorkerId, failed: bool) {
        let Some(worker) = self.workers.remove(&id) else {
            return;
        };
        if self.keys.get(&worker.key) == Some(&id) {
            self.keys.remove(&worker.key);
        }
        for (_, connection) in &worker.connections {
            if failed {
                connection.reply(SharedWorkerClientMessage::Error);
            }
            if !connection.received {
                connection.close();
            }
        }
    }

    /// <https://html.spec.whatwg.org/multipage/#destroy-a-document>
    fn remove_owner(&mut self, owner: PipelineId) {
        self.fully_active.remove(&owner);
        for worker in self.workers.values_mut() {
            // Step 8. Remove document from the owner set of each WorkerGlobalScope object whose set contains document.
            worker.owners.remove(&owner);
            worker.connections.retain(|(_, connection)| {
                if connection.owner != owner {
                    return true;
                }
                if !connection.received {
                    connection.close();
                }
                false
            });
            if worker
                .csp_client
                .as_ref()
                .is_some_and(|(pipeline, _)| *pipeline == owner)
            {
                worker.csp_client = None;
            }
        }
    }

    /// <https://html.spec.whatwg.org/multipage/#run-a-worker>
    fn update_lifetimes(&mut self, now: Instant) {
        let extended_timeout = Duration::from_secs(pref!(dom_serviceworker_timeout_seconds));
        let mut stop = Vec::new();
        for (id, worker) in &mut self.workers {
            for (owner, inactive_since) in &mut worker.owners {
                if self.fully_active.get(owner) == Some(&true) {
                    *inactive_since = None;
                } else {
                    inactive_since.get_or_insert(now);
                }
            }
            if worker.owners.is_empty() {
                worker.owner_set_empty_since.get_or_insert(now);
            } else {
                worker.owner_set_empty_since = None;
            }
            let extended_deadline = worker.extended_lifetime_deadline(now, extended_timeout);
            let actively_needed =
                extended_deadline.is_some() || worker.owners.values().any(Option::is_none);
            let between_loads_deadline = worker
                .owner_set_empty_since
                .map(|since| since + BETWEEN_LOADS_TIMEOUT)
                .filter(|deadline| self.has_loading_document && now < *deadline);
            let permissible = !worker.owners.is_empty() ||
                extended_deadline.is_some() ||
                between_loads_deadline.is_some();
            worker.deadline = extended_deadline.or(between_loads_deadline);
            if (!permissible && (worker.secure_context.is_some() || worker.connections.is_empty())) ||
                (worker.thread.is_closing() && !actively_needed)
            {
                stop.push(*id);
                continue;
            }
            // Step 8 of onComplete of <https://html.spec.whatwg.org/multipage/#run-a-worker>
            // Suspending workers: Start monitoring worker global scope, such that whenever worker global scope's closing flag is false and it is suspendable, the user agent suspends execution of script in worker global scope until such time as either the closing flag switches to true or worker global scope stops being suspendable.
            worker.thread.set_suspended(
                worker.secure_context.is_none() || (!actively_needed && permissible),
            );
        }
        for id in stop {
            self.finish(id, false);
        }
    }
}

impl Worker {
    /// <https://html.spec.whatwg.org/multipage/#in-extended-lifetime-worker>
    fn extended_lifetime_deadline(&self, now: Instant, timeout: Duration) -> Option<Instant> {
        // Step 2. If global's extended lifetime is false, then return false.
        if !self.options.extended_lifetime {
            return None;
        }
        // Step 3. If global's owner set is empty, but has been empty for less than the user agent's extended lifetime shared worker timeout, then return true.
        if self.owners.is_empty() {
            return self
                .owner_set_empty_since
                .map(|since| since + timeout)
                .filter(|deadline| now < *deadline);
        }
        // Step 4.1. If owner is a Document that is fully active, then return false.
        if self.owners.values().any(Option::is_none) {
            return None;
        }
        // Step 5.1. If owner is a Document and the amount of time that owner has been non-fully active is less than the user agent's extended lifetime shared worker timeout, then return true.
        self.owners
            .values()
            .filter_map(|since| since.map(|since| since + timeout))
            .filter(|deadline| now < *deadline)
            .max()
    }
}

impl Drop for SharedWorkerManager {
    fn drop(&mut self) {
        for worker in self.workers.values() {
            worker.thread.terminate();
        }
        self.workers.clear();
    }
}

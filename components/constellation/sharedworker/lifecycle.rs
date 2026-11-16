/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::collections::HashMap;
use std::time::{Duration, Instant};

use log::warn;
use script_traits::ScriptThreadMessage;
use servo_base::id::{PipelineId, ScriptEventLoopId, WorkerId};
use servo_config::pref;
use servo_constellation_traits::{SharedWorkerClientMessage, SharedWorkerCommand};

use super::{ManagerAction, Phase, SharedWorkerManager, Worker};

// <https://html.spec.whatwg.org/multipage/workers.html#between-loads-shared-worker-timeout>
// "A typical value for the between-loads shared worker timeout might be 5 seconds."
const BETWEEN_LOADS_TIMEOUT: Duration = Duration::from_secs(5);

impl Worker {
    /// <https://html.spec.whatwg.org/multipage/workers.html#in-extended-lifetime-worker>
    fn extended_lifetime_deadline(&self, now: Instant, timeout: Duration) -> Option<Instant> {
        // Step 2. If global's extended lifetime is false, then return false.
        if !self.options.extended_lifetime {
            return None;
        }

        // Step 3. If global's owner set is empty, but has been empty for less than
        // the user agent's extended lifetime shared worker timeout, then return true.
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

        // Step 5.1. If owner is a Document and the amount of time that owner has
        // been non-fully active is less than the user agent's extended lifetime
        // shared worker timeout, then return true.
        // Step 6. Return false.
        self.owners
            .values()
            .filter_map(|since| since.map(|since| since + timeout))
            .filter(|deadline| now < *deadline)
            .max()
    }

    /// <https://html.spec.whatwg.org/multipage/workers.html#active-needed-worker>
    fn actively_needed(&self, in_extended_lifetime: bool) -> bool {
        // Step 1. If global is in extended lifetime, then return true.
        // Step 2.1. If owner is a Document, and owner is fully active, then return true.
        // Step 3. Return false.
        in_extended_lifetime || self.owners.values().any(Option::is_none)
    }

    /// <https://html.spec.whatwg.org/multipage/workers.html#permissible-worker>
    fn permissible(
        &self,
        now: Instant,
        in_extended_lifetime: bool,
        has_loading_document: bool,
    ) -> bool {
        // Step 1. If global's owner set is not empty, then return true.
        // Step 2. If global is in extended lifetime, then return true.
        if !self.owners.is_empty() || in_extended_lifetime {
            return true;
        }

        // Step 3. If all of the following are true:
        // - global is a SharedWorkerGlobalScope;
        // - global's owner set has been empty for no more than the user agent's
        //   between-loads shared worker timeout; and
        // - the user agent has a navigable whose active document is not completely loaded,
        // then return true.
        // Step 4. Return false.
        has_loading_document &&
            self.owner_set_empty_since
                .is_some_and(|since| now < since + BETWEEN_LOADS_TIMEOUT)
    }
}

impl SharedWorkerManager {
    pub(crate) fn start_failed(&mut self, worker_id: WorkerId) {
        self.finish(worker_id, true);
    }

    pub(super) fn stop(&mut self, worker_id: WorkerId, failed: bool) {
        let Some(worker) = self.workers.get_mut(&worker_id) else {
            return;
        };
        worker.phase = Phase::Closing;
        worker.deadline = None;
        if self.keys.get(&worker.key) == Some(&worker_id) {
            self.keys.remove(&worker.key);
        }
        for (_, connection) in worker.connections.drain(..) {
            if failed {
                Self::reply(&connection.reply, SharedWorkerClientMessage::Error);
            }
            if !connection.received {
                self.actions
                    .push(ManagerAction::ClosePort(connection.port_id));
            }
        }
        worker.csp_client = None;
        if worker.pending_start.is_some() {
            self.finish(worker_id, false);
            return;
        }
        let result = match &worker.commands {
            Some(commands) => commands.send(SharedWorkerCommand::Terminate),
            None => worker
                .context
                .host
                .send(ScriptThreadMessage::StopSharedWorker(worker_id)),
        };
        if let Err(error) = result {
            warn!("SharedWorker host is no longer reachable: {error}");
            self.finish(worker_id, true);
        }
    }

    pub(super) fn finish(&mut self, worker_id: WorkerId, failed: bool) {
        let Some(worker) = self.workers.remove(&worker_id) else {
            return;
        };
        if self.keys.get(&worker.key) == Some(&worker_id) {
            self.keys.remove(&worker.key);
        }
        for (_, connection) in worker.connections {
            if failed {
                Self::reply(&connection.reply, SharedWorkerClientMessage::Error);
            }
            if !connection.received {
                self.actions
                    .push(ManagerAction::ClosePort(connection.port_id));
            }
        }
        self.actions.extend(
            worker
                .message_routers
                .into_keys()
                .map(ManagerAction::RemoveMessageRouter),
        );
        self.actions.extend(
            worker
                .broadcast_routers
                .into_keys()
                .map(ManagerAction::RemoveBroadcastRouter),
        );
        self.actions.push(ManagerAction::ReleasePainter(worker_id));
    }

    pub(crate) fn owner_pipeline_ids(&self) -> impl Iterator<Item = PipelineId> + '_ {
        self.workers
            .values()
            .flat_map(|worker| worker.owners.keys().copied())
    }

    /// <https://html.spec.whatwg.org/multipage/workers.html#run-a-worker>
    pub(crate) fn update_lifetimes(
        &mut self,
        now: Instant,
        fully_active: &HashMap<PipelineId, bool>,
        has_loading_document: bool,
    ) {
        // <https://html.spec.whatwg.org/multipage/workers.html#extended-lifetime-shared-worker-timeout>
        // "User agents should choose a value that is equal to the longest lifetime
        // that a service worker can stay alive without any clients, and must not
        // choose a value that is longer than that amount of time."
        let extended_timeout = Duration::from_secs(pref!(dom_serviceworker_timeout_seconds));
        let mut stop = Vec::new();
        for (id, worker) in &mut self.workers {
            for (owner, inactive_since) in &mut worker.owners {
                if fully_active.get(owner) == Some(&true) {
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
            let actively_needed = worker.actively_needed(extended_deadline.is_some());
            let permissible =
                worker.permissible(now, extended_deadline.is_some(), has_loading_document);
            worker.deadline = extended_deadline.or_else(|| {
                worker
                    .owner_set_empty_since
                    .map(|since| since + BETWEEN_LOADS_TIMEOUT)
                    .filter(|deadline| has_loading_document && now < *deadline)
            });

            // <https://html.spec.whatwg.org/multipage/workers.html#terminate-a-worker>
            // "User agents may invoke the terminate a worker algorithm when a
            // worker stops being actively needed and the worker continues
            // executing even after its closing flag was set to true."
            if worker.phase == Phase::Closing {
                if !actively_needed {
                    stop.push(*id);
                }
                continue;
            }

            // Closing orphan workers: "no sooner than it stops being protected,
            // and no later than it stops being permissible, worker global scope's
            // closing flag is set to true."
            // <https://html.spec.whatwg.org/multipage/workers.html#protected-worker>
            // Step 1. If global is not actively needed, then return false.
            // Step 2. If global is in extended lifetime, then return true.
            // Step 3. If global is a SharedWorkerGlobalScope, then return true.
            if !permissible && (worker.secure_context.is_some() || worker.connections.is_empty()) {
                stop.push(*id);
                continue;
            }

            // <https://html.spec.whatwg.org/multipage/workers.html#suspendable-worker>
            // Step 1. If global is actively needed, then return false.
            // Step 2. If global is permissible, then return true.
            // Step 3. Return false.
            // Finish scope initialization and validate pending connections before
            // allowing its script to run.
            let suspended = worker.secure_context.is_none() || (!actively_needed && permissible);
            if worker.suspended != suspended {
                worker.suspended = suspended;
                if let Some(commands) = &worker.commands &&
                    let Err(error) = commands.send(SharedWorkerCommand::SetSuspended(suspended))
                {
                    warn!("Failed to update SharedWorker suspension: {error}");
                    stop.push(*id);
                }
            }
        }
        for id in stop {
            self.stop(id, false);
        }
    }

    /// <https://html.spec.whatwg.org/multipage/document-lifecycle.html#destroy-a-document>
    pub(crate) fn remove_owner(&mut self, pipeline: PipelineId, now: Instant) {
        let mut removed = Vec::new();
        self.pending.retain(|pending| {
            if pending.owner.pipeline_id != pipeline {
                return true;
            }
            removed.push(*pending.request.port.message_port_id());
            false
        });
        self.actions
            .extend(removed.into_iter().map(ManagerAction::ClosePort));
        for worker in self.workers.values_mut() {
            // "Remove document from the owner set of each WorkerGlobalScope
            // object whose set contains document."
            worker.owners.remove(&pipeline);
            if worker
                .csp_client
                .as_ref()
                .is_some_and(|(owner, _)| *owner == pipeline)
            {
                worker.csp_client = None;
            }
            worker.connections.retain(|(_, connection)| {
                if connection.owner != pipeline {
                    return true;
                }
                if !connection.received {
                    self.actions
                        .push(ManagerAction::ClosePort(connection.port_id));
                }
                false
            });
            if worker.owners.is_empty() {
                worker.owner_set_empty_since.get_or_insert(now);
            }
        }
        self.process_pending(now);
    }

    pub(crate) fn shutdown(&mut self) {
        self.shutting_down = true;
        while let Some(pending) = self.pending.pop_front() {
            self.reject_request(pending.request, "browser shutdown");
        }
        let ids: Vec<_> = self.workers.keys().copied().collect();
        for id in ids {
            self.stop(id, true);
        }
    }

    pub(crate) fn set_online(&mut self, online: bool) {
        self.online = online;
        let mut failed = Vec::new();
        for (id, worker) in &self.workers {
            if worker.phase == Phase::Closing {
                continue;
            }
            if let Some(commands) = &worker.commands {
                if let Err(error) =
                    commands.send(SharedWorkerCommand::SetNetworkOnlineState(online))
                {
                    warn!("Failed to send SharedWorker network state: {error}");
                    failed.push(*id);
                }
            }
        }
        for id in failed {
            self.finish(id, true);
        }
    }

    pub(crate) fn host_exited(&mut self, host: ScriptEventLoopId, now: Instant) {
        let ids: Vec<_> = self
            .workers
            .iter()
            .filter(|(_, worker)| worker.context.host.id() == host)
            .map(|(id, _)| *id)
            .collect();
        for id in ids {
            self.finish(id, true);
        }
        let mut requests = Vec::new();
        let pending = std::mem::take(&mut self.pending);
        for request in pending {
            if request.owner.host.id() == host || request.top_host.id() == host {
                requests.push(request.request);
            } else {
                self.pending.push_back(request);
            }
        }
        for request in requests {
            self.reject_request(request, "script host exited");
        }
        self.process_pending(now);
    }

    pub(crate) fn worker_panicked(&mut self, event_loop: ScriptEventLoopId) -> bool {
        let id = self
            .workers
            .iter()
            .find(|(_, worker)| worker.event_loop_id == event_loop)
            .map(|(id, _)| *id);
        if let Some(id) = id {
            self.stop(id, true);
            return true;
        }
        false
    }

    pub(crate) fn host_panicked(&mut self, event_loop: ScriptEventLoopId) {
        let ids: Vec<_> = self
            .workers
            .iter()
            .filter(|(_, worker)| worker.context.host.id() == event_loop)
            .map(|(id, _)| *id)
            .collect();
        for id in ids {
            self.stop(id, true);
        }
    }
}

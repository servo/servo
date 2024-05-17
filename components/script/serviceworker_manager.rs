/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! The service worker manager persists the descriptor of any registered service workers.
//! It also stores an active workers map, which holds descriptors of running service workers.
//! If an active service worker timeouts, then it removes the descriptor entry from its
//! active_workers map

use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread::{self, JoinHandle};

use base::id::{PipelineNamespace, ServiceWorkerId, ServiceWorkerRegistrationId};
use crossbeam_channel::{select, unbounded, Receiver, RecvError, Sender};
use ipc_channel::ipc::{self, IpcSender};
use ipc_channel::router::ROUTER;
use net_traits::{CoreResourceMsg, CustomResponseMediator};
use script_traits::{
    DOMMessage, Job, JobError, JobResult, JobResultValue, JobType, SWManagerMsg, SWManagerSenders,
    ScopeThings, ServiceWorkerManagerFactory, ServiceWorkerMsg,
};
use servo_config::pref;
use servo_url::{ImmutableOrigin, ServoUrl};

use crate::dom::abstractworker::WorkerScriptMsg;
use crate::dom::serviceworkerglobalscope::{
    ServiceWorkerControlMsg, ServiceWorkerGlobalScope, ServiceWorkerScriptMsg,
};
use crate::dom::serviceworkerregistration::longest_prefix_match;
use crate::script_runtime::ContextForRequestInterrupt;

enum Message {
    FromResource(CustomResponseMediator),
    FromConstellation(ServiceWorkerMsg),
}

/// <https://w3c.github.io/ServiceWorker/#dfn-service-worker>
#[derive(Clone)]
struct ServiceWorker {
    /// A unique identifer.
    pub id: ServiceWorkerId,
    /// <https://w3c.github.io/ServiceWorker/#dfn-script-url>
    pub script_url: ServoUrl,
    /// A sender to the running service worker scope.
    pub sender: Sender<ServiceWorkerScriptMsg>,
}

impl ServiceWorker {
    fn new(
        script_url: ServoUrl,
        sender: Sender<ServiceWorkerScriptMsg>,
        id: ServiceWorkerId,
    ) -> ServiceWorker {
        ServiceWorker {
            id,
            script_url,
            sender,
        }
    }

    /// Forward a DOM message to the running service worker scope.
    fn forward_dom_message(&self, msg: DOMMessage) {
        let DOMMessage { origin, data } = msg;
        let _ = self.sender.send(ServiceWorkerScriptMsg::CommonWorker(
            WorkerScriptMsg::DOMMessage { origin, data },
        ));
    }

    /// Send a message to the running service worker scope.
    fn send_message(&self, msg: ServiceWorkerScriptMsg) {
        let _ = self.sender.send(msg);
    }
}

/// When updating a registration, which worker are we targetting?
#[allow(dead_code)]
enum RegistrationUpdateTarget {
    Installing,
    Waiting,
    Active,
}

impl Drop for ServiceWorkerRegistration {
    /// <https://html.spec.whatwg.org/multipage/#terminate-a-worker>
    fn drop(&mut self) {
        // Drop the channel to signal shutdown.
        if self
            .control_sender
            .take()
            .expect("No control sender to worker thread.")
            .send(ServiceWorkerControlMsg::Exit)
            .is_err()
        {
            warn!("Failed to send exit message to service worker scope.");
        }

        self.closing
            .take()
            .expect("No close flag for worker")
            .store(true, Ordering::SeqCst);
        self.context
            .take()
            .expect("No context to request interrupt.")
            .request_interrupt();

        // TODO: Step 1, 2 and 3.
        if self
            .join_handle
            .take()
            .expect("No handle to join on worker.")
            .join()
            .is_err()
        {
            warn!("Failed to join on service worker thread.");
        }
    }
}

/// <https://w3c.github.io/ServiceWorker/#service-worker-registration-concept>
struct ServiceWorkerRegistration {
    /// A unique identifer.
    id: ServiceWorkerRegistrationId,
    /// <https://w3c.github.io/ServiceWorker/#dfn-active-worker>
    active_worker: Option<ServiceWorker>,
    /// <https://w3c.github.io/ServiceWorker/#dfn-waiting-worker>
    waiting_worker: Option<ServiceWorker>,
    /// <https://w3c.github.io/ServiceWorker/#dfn-installing-worker>
    installing_worker: Option<ServiceWorker>,
    /// A channel to send control message to the worker,
    /// currently only used to signal shutdown.
    control_sender: Option<Sender<ServiceWorkerControlMsg>>,
    /// A handle to join on the worker thread.
    join_handle: Option<JoinHandle<()>>,
    /// A context to request an interrupt.
    context: Option<ContextForRequestInterrupt>,
    /// The closing flag for the worker.
    closing: Option<Arc<AtomicBool>>,
}

impl ServiceWorkerRegistration {
    pub fn new() -> ServiceWorkerRegistration {
        ServiceWorkerRegistration {
            id: ServiceWorkerRegistrationId::new(),
            active_worker: None,
            waiting_worker: None,
            installing_worker: None,
            join_handle: None,
            control_sender: None,
            context: None,
            closing: None,
        }
    }

    fn note_worker_thread(
        &mut self,
        join_handle: JoinHandle<()>,
        control_sender: Sender<ServiceWorkerControlMsg>,
        context: ContextForRequestInterrupt,
        closing: Arc<AtomicBool>,
    ) {
        assert!(self.join_handle.is_none());
        self.join_handle = Some(join_handle);

        assert!(self.control_sender.is_none());
        self.control_sender = Some(control_sender);

        assert!(self.context.is_none());
        self.context = Some(context);

        assert!(self.closing.is_none());
        self.closing = Some(closing);
    }

    /// <https://w3c.github.io/ServiceWorker/#get-newest-worker>
    fn get_newest_worker(&self) -> Option<ServiceWorker> {
        if let Some(worker) = self.active_worker.as_ref() {
            return Some(worker.clone());
        }
        if let Some(worker) = self.waiting_worker.as_ref() {
            return Some(worker.clone());
        }
        if let Some(worker) = self.installing_worker.as_ref() {
            return Some(worker.clone());
        }
        None
    }

    /// <https://w3c.github.io/ServiceWorker/#update-registration-state>
    fn update_registration_state(
        &mut self,
        target: RegistrationUpdateTarget,
        worker: ServiceWorker,
    ) {
        match target {
            RegistrationUpdateTarget::Active => {
                self.active_worker = Some(worker);
            },
            RegistrationUpdateTarget::Waiting => {
                self.waiting_worker = Some(worker);
            },
            RegistrationUpdateTarget::Installing => {
                self.installing_worker = Some(worker);
            },
        }
    }
}

/// A structure managing all registrations and workers for a given origin.
pub struct ServiceWorkerManager {
    /// <https://w3c.github.io/ServiceWorker/#dfn-scope-to-registration-map>
    registrations: HashMap<ServoUrl, ServiceWorkerRegistration>,
    // Will be useful to implement posting a message to a client.
    // See https://github.com/servo/servo/issues/24660
    _constellation_sender: IpcSender<SWManagerMsg>,
    // own sender to send messages here
    own_sender: IpcSender<ServiceWorkerMsg>,
    // receiver to receive messages from constellation
    own_port: Receiver<ServiceWorkerMsg>,
    // to receive resource messages
    resource_receiver: Receiver<CustomResponseMediator>,
}

impl ServiceWorkerManager {
    fn new(
        own_sender: IpcSender<ServiceWorkerMsg>,
        from_constellation_receiver: Receiver<ServiceWorkerMsg>,
        resource_port: Receiver<CustomResponseMediator>,
        constellation_sender: IpcSender<SWManagerMsg>,
    ) -> ServiceWorkerManager {
        // Install a pipeline-namespace in the current thread.
        PipelineNamespace::auto_install();

        ServiceWorkerManager {
            registrations: HashMap::new(),
            own_sender,
            own_port: from_constellation_receiver,
            resource_receiver: resource_port,
            _constellation_sender: constellation_sender,
        }
    }

    pub fn get_matching_scope(&self, load_url: &ServoUrl) -> Option<ServoUrl> {
        for scope in self.registrations.keys() {
            if longest_prefix_match(scope, load_url) {
                return Some(scope.clone());
            }
        }
        None
    }

    fn handle_message(&mut self) {
        while let Ok(message) = self.receive_message() {
            let should_continue = match message {
                Message::FromConstellation(msg) => self.handle_message_from_constellation(msg),
                Message::FromResource(msg) => self.handle_message_from_resource(msg),
            };
            if !should_continue {
                for registration in self.registrations.drain() {
                    // Signal shut-down, and join on the thread.
                    drop(registration);
                }
                break;
            }
        }
    }

    fn handle_message_from_resource(&mut self, mediator: CustomResponseMediator) -> bool {
        if serviceworker_enabled() {
            if let Some(scope) = self.get_matching_scope(&mediator.load_url) {
                if let Some(registration) = self.registrations.get(&scope) {
                    if let Some(ref worker) = registration.active_worker {
                        worker.send_message(ServiceWorkerScriptMsg::Response(mediator));
                        return true;
                    }
                }
            }
        }
        let _ = mediator.response_chan.send(None);
        true
    }

    fn receive_message(&mut self) -> Result<Message, RecvError> {
        select! {
            recv(self.own_port) -> msg => msg.map(Message::FromConstellation),
            recv(self.resource_receiver) -> msg => msg.map(Message::FromResource),
        }
    }

    fn handle_message_from_constellation(&mut self, msg: ServiceWorkerMsg) -> bool {
        match msg {
            ServiceWorkerMsg::Timeout(_scope) => {
                // TODO: https://w3c.github.io/ServiceWorker/#terminate-service-worker
            },
            ServiceWorkerMsg::ForwardDOMMessage(msg, scope_url) => {
                if let Some(registration) = self.registrations.get_mut(&scope_url) {
                    if let Some(ref worker) = registration.active_worker {
                        worker.forward_dom_message(msg);
                    }
                }
            },
            ServiceWorkerMsg::ScheduleJob(job) => match job.job_type {
                JobType::Register => {
                    self.handle_register_job(job);
                },
                JobType::Update => {
                    self.handle_update_job(job);
                },
                JobType::Unregister => {
                    // TODO: https://w3c.github.io/ServiceWorker/#unregister-algorithm
                },
            },
            ServiceWorkerMsg::Exit => return false,
        }
        true
    }

    /// <https://w3c.github.io/ServiceWorker/#register-algorithm>
    fn handle_register_job(&mut self, mut job: Job) {
        if !job.script_url.is_origin_trustworthy() {
            // Step 1.1
            let _ = job
                .client
                .send(JobResult::RejectPromise(JobError::SecurityError));
            return;
        }

        if job.script_url.origin() != job.referrer.origin() ||
            job.scope_url.origin() != job.referrer.origin()
        {
            // Step 2.1
            let _ = job
                .client
                .send(JobResult::RejectPromise(JobError::SecurityError));
            return;
        }

        // Step 4: Get registration.
        if let Some(registration) = self.registrations.get(&job.scope_url) {
            // Step 5, we have a registation.

            // Step 5.1, get newest worker
            let newest_worker = registration.get_newest_worker();

            // step 5.2
            if newest_worker.is_some() {
                // TODO: the various checks of job versus worker.

                // Step 2.1: Run resolve job.
                let client = job.client.clone();
                let _ = client.send(JobResult::ResolvePromise(
                    job,
                    JobResultValue::Registration {
                        id: registration.id,
                        installing_worker: registration
                            .installing_worker
                            .as_ref()
                            .map(|worker| worker.id),
                        waiting_worker: registration
                            .waiting_worker
                            .as_ref()
                            .map(|worker| worker.id),
                        active_worker: registration.active_worker.as_ref().map(|worker| worker.id),
                    },
                ));
            }
        } else {
            // Step 6: we do not have a registration.

            // Step 6.1: Run Set Registration.
            let new_registration = ServiceWorkerRegistration::new();
            self.registrations
                .insert(job.scope_url.clone(), new_registration);

            // Step 7: Schedule update
            job.job_type = JobType::Update;
            let _ = self.own_sender.send(ServiceWorkerMsg::ScheduleJob(job));
        }
    }

    /// <https://w3c.github.io/ServiceWorker/#update>
    fn handle_update_job(&mut self, job: Job) {
        // Step 1: Get registation
        if let Some(registration) = self.registrations.get_mut(&job.scope_url) {
            // Step 3.
            let newest_worker = registration.get_newest_worker();

            // Step 4.
            if let Some(worker) = newest_worker {
                if worker.script_url != job.script_url {
                    let _ = job
                        .client
                        .send(JobResult::RejectPromise(JobError::TypeError));
                    return;
                }
            }

            let scope_things = job
                .scope_things
                .clone()
                .expect("Update job should have scope things.");

            // Very roughly steps 5 to 18.
            // TODO: implement all steps precisely.
            let (new_worker, join_handle, control_sender, context, closing) =
                update_serviceworker(self.own_sender.clone(), job.scope_url.clone(), scope_things);

            // Since we've just started the worker thread, ensure we can shut it down later.
            registration.note_worker_thread(join_handle, control_sender, context, closing);

            // Step 19, run Install.

            // Install: Step 4, run Update Registration State.
            registration
                .update_registration_state(RegistrationUpdateTarget::Installing, new_worker);

            // Install: Step 7, run Resolve Job Promise.
            let client = job.client.clone();
            let _ = client.send(JobResult::ResolvePromise(
                job,
                JobResultValue::Registration {
                    id: registration.id,
                    installing_worker: registration
                        .installing_worker
                        .as_ref()
                        .map(|worker| worker.id),
                    waiting_worker: registration.waiting_worker.as_ref().map(|worker| worker.id),
                    active_worker: registration.active_worker.as_ref().map(|worker| worker.id),
                },
            ));
        } else {
            // Step 2
            let _ = job
                .client
                .send(JobResult::RejectPromise(JobError::TypeError));
        }
    }
}

/// <https://w3c.github.io/ServiceWorker/#update-algorithm>
fn update_serviceworker(
    own_sender: IpcSender<ServiceWorkerMsg>,
    scope_url: ServoUrl,
    scope_things: ScopeThings,
) -> (
    ServiceWorker,
    JoinHandle<()>,
    Sender<ServiceWorkerControlMsg>,
    ContextForRequestInterrupt,
    Arc<AtomicBool>,
) {
    let (sender, receiver) = unbounded();
    let (_devtools_sender, devtools_receiver) = ipc::channel().unwrap();
    let worker_id = ServiceWorkerId::new();

    let (control_sender, control_receiver) = unbounded();
    let (context_sender, context_receiver) = unbounded();
    let closing = Arc::new(AtomicBool::new(false));

    let join_handle = ServiceWorkerGlobalScope::run_serviceworker_scope(
        scope_things.clone(),
        sender.clone(),
        receiver,
        devtools_receiver,
        own_sender,
        scope_url.clone(),
        control_receiver,
        context_sender,
        closing.clone(),
    );

    let context = context_receiver
        .recv()
        .expect("Couldn't receive a context for worker.");

    (
        ServiceWorker::new(scope_things.script_url, sender, worker_id),
        join_handle,
        control_sender,
        context,
        closing,
    )
}

impl ServiceWorkerManagerFactory for ServiceWorkerManager {
    fn create(sw_senders: SWManagerSenders, origin: ImmutableOrigin) {
        let (resource_chan, resource_port) = ipc::channel().unwrap();

        let SWManagerSenders {
            resource_sender,
            own_sender,
            receiver,
            swmanager_sender: constellation_sender,
        } = sw_senders;

        let from_constellation = ROUTER.route_ipc_receiver_to_new_crossbeam_receiver(receiver);
        let resource_port = ROUTER.route_ipc_receiver_to_new_crossbeam_receiver(resource_port);
        let _ = resource_sender.send(CoreResourceMsg::NetworkMediator(resource_chan, origin));
        let swmanager_thread = move || {
            ServiceWorkerManager::new(
                own_sender,
                from_constellation,
                resource_port,
                constellation_sender,
            )
            .handle_message()
        };
        if thread::Builder::new()
            .name("SvcWorkerManager".to_owned())
            .spawn(swmanager_thread)
            .is_err()
        {
            warn!("ServiceWorkerManager thread spawning failed");
        }
    }
}

pub fn serviceworker_enabled() -> bool {
    pref!(dom.serviceworker.enabled)
}

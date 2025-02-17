/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::default::Default;
use std::rc::Rc;

use dom_struct::dom_struct;
use ipc_channel::ipc;
use ipc_channel::router::ROUTER;
use script_traits::{Job, JobError, JobResult, JobResultValue, JobType, ScriptMsg};

use crate::dom::bindings::codegen::Bindings::ServiceWorkerContainerBinding::{
    RegistrationOptions, ServiceWorkerContainerMethods,
};
use crate::dom::bindings::error::Error;
use crate::dom::bindings::refcounted::TrustedPromise;
use crate::dom::bindings::reflector::{reflect_dom_object, DomGlobal};
use crate::dom::bindings::root::{Dom, DomRoot, MutNullableDom};
use crate::dom::bindings::str::USVString;
use crate::dom::client::Client;
use crate::dom::eventtarget::EventTarget;
use crate::dom::globalscope::GlobalScope;
use crate::dom::promise::Promise;
use crate::dom::serviceworker::ServiceWorker;
use crate::dom::serviceworkerregistration::ServiceWorkerRegistration;
use crate::realms::{enter_realm, InRealm};
use crate::script_runtime::CanGc;
use crate::task_source::SendableTaskSource;

#[dom_struct]
pub(crate) struct ServiceWorkerContainer {
    eventtarget: EventTarget,
    controller: MutNullableDom<ServiceWorker>,
    client: Dom<Client>,
}

impl ServiceWorkerContainer {
    fn new_inherited(client: &Client) -> ServiceWorkerContainer {
        ServiceWorkerContainer {
            eventtarget: EventTarget::new_inherited(),
            controller: Default::default(),
            client: Dom::from_ref(client),
        }
    }

    #[cfg_attr(crown, allow(crown::unrooted_must_root))]
    pub(crate) fn new(global: &GlobalScope) -> DomRoot<ServiceWorkerContainer> {
        let client = Client::new(global.as_window());
        let container = ServiceWorkerContainer::new_inherited(&client);
        reflect_dom_object(Box::new(container), global, CanGc::note())
    }
}

impl ServiceWorkerContainerMethods<crate::DomTypeHolder> for ServiceWorkerContainer {
    // https://w3c.github.io/ServiceWorker/#service-worker-container-controller-attribute
    fn GetController(&self) -> Option<DomRoot<ServiceWorker>> {
        self.client.get_controller()
    }

    /// <https://w3c.github.io/ServiceWorker/#dom-serviceworkercontainer-register> - A
    /// and <https://w3c.github.io/ServiceWorker/#start-register> - B
    fn Register(
        &self,
        script_url: USVString,
        options: &RegistrationOptions,
        comp: InRealm,
        can_gc: CanGc,
    ) -> Rc<Promise> {
        // A: Step 2.
        let global = self.client.global();

        // A: Step 1
        let promise = Promise::new_in_current_realm(comp, can_gc);
        let USVString(ref script_url) = script_url;

        // A: Step 3
        let api_base_url = global.api_base_url();
        let script_url = match api_base_url.join(script_url) {
            Ok(url) => url,
            Err(_) => {
                // B: Step 1
                promise.reject_error(Error::Type("Invalid script URL".to_owned()));
                return promise;
            },
        };

        // A: Step 4-5
        let scope = match options.scope {
            Some(ref scope) => {
                let USVString(inner_scope) = scope;
                match api_base_url.join(inner_scope) {
                    Ok(url) => url,
                    Err(_) => {
                        promise.reject_error(Error::Type("Invalid scope URL".to_owned()));
                        return promise;
                    },
                }
            },
            None => script_url.join("./").unwrap(),
        };

        // A: Step 6 -> invoke B.

        // B: Step 3
        match script_url.scheme() {
            "https" | "http" => {},
            _ => {
                promise.reject_error(Error::Type("Only secure origins are allowed".to_owned()));
                return promise;
            },
        }
        // B: Step 4
        if script_url.path().to_ascii_lowercase().contains("%2f") ||
            script_url.path().to_ascii_lowercase().contains("%5c")
        {
            promise.reject_error(Error::Type(
                "Script URL contains forbidden characters".to_owned(),
            ));
            return promise;
        }

        // B: Step 6
        match scope.scheme() {
            "https" | "http" => {},
            _ => {
                promise.reject_error(Error::Type("Only secure origins are allowed".to_owned()));
                return promise;
            },
        }
        // B: Step 7
        if scope.path().to_ascii_lowercase().contains("%2f") ||
            scope.path().to_ascii_lowercase().contains("%5c")
        {
            promise.reject_error(Error::Type(
                "Scope URL contains forbidden characters".to_owned(),
            ));
            return promise;
        }

        // Setup the callback for reject/resolve of the promise,
        // from steps running "in-parallel" from here in the serviceworker manager.
        let mut handler = RegisterJobResultHandler {
            trusted_promise: Some(TrustedPromise::new(promise.clone())),
            task_source: global.task_manager().dom_manipulation_task_source().into(),
        };

        let (job_result_sender, job_result_receiver) = ipc::channel().expect("ipc channel failure");

        ROUTER.add_typed_route(
            job_result_receiver,
            Box::new(move |message| match message {
                Ok(msg) => handler.handle(msg),
                Err(err) => warn!("Error receiving a JobResult: {:?}", err),
            }),
        );

        let scope_things =
            ServiceWorkerRegistration::create_scope_things(&global, script_url.clone());

        // B: Step 8 - 13
        let job = Job::create_job(
            JobType::Register,
            scope,
            script_url,
            job_result_sender,
            self.client.creation_url(),
            Some(scope_things),
        );

        // B: Step 14: schedule job.
        let _ = global
            .script_to_constellation_chan()
            .send(ScriptMsg::ScheduleJob(job));

        // A: Step 7
        promise
    }
}

/// Callback for resolve/reject job promise for Register.
/// <https://w3c.github.io/ServiceWorker/#register>
struct RegisterJobResultHandler {
    trusted_promise: Option<TrustedPromise>,
    task_source: SendableTaskSource,
}

impl RegisterJobResultHandler {
    /// <https://w3c.github.io/ServiceWorker/#reject-job-promise>
    /// <https://w3c.github.io/ServiceWorker/#resolve-job-promise>
    /// Handle a result to either resolve or reject the register job promise.
    pub(crate) fn handle(&mut self, result: JobResult) {
        match result {
            JobResult::RejectPromise(error) => {
                let promise = self
                    .trusted_promise
                    .take()
                    .expect("No promise to resolve for SW Register job.");

                // Step 1
                self.task_source.queue(
                    task!(reject_promise_with_security_error: move || {
                        let promise = promise.root();
                        let _ac = enter_realm(&*promise.global());
                        match error {
                            JobError::TypeError => {
                                promise.reject_error(Error::Type("Failed to register a ServiceWorker".to_string()));
                            },
                            JobError::SecurityError => {
                                promise.reject_error(Error::Security);
                            },
                        }

                    })
                );

                // TODO: step 2, handle equivalent jobs.
            },
            JobResult::ResolvePromise(job, value) => {
                let promise = self
                    .trusted_promise
                    .take()
                    .expect("No promise to resolve for SW Register job.");

                // Step 1
                self.task_source.queue(task!(resolve_promise: move || {
                    let promise = promise.root();
                    let global = promise.global();
                    let _ac = enter_realm(&*global);

                    // Step 1.1
                    let JobResultValue::Registration {
                        id,
                        installing_worker,
                        waiting_worker,
                        active_worker,
                    } = value;

                    // Step 1.2 (Job type is "register").
                    let registration = global.get_serviceworker_registration(
                        &job.script_url,
                        &job.scope_url,
                        id,
                        installing_worker,
                        waiting_worker,
                        active_worker,
                    );

                    // Step 1.4
                    promise.resolve_native(&*registration);
                }));

                // TODO: step 2, handle equivalent jobs.
            },
        }
    }
}

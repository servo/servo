/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::collections::VecDeque;
use std::default::Default;
use std::rc::Rc;

use dom_struct::dom_struct;
use js::context::JSContext;
use js::jsval::UndefinedValue;
use js::realm::CurrentRealm;
use script_bindings::cell::DomRefCell;
use script_bindings::inheritance::Castable;
use script_bindings::reflector::reflect_dom_object_with_cx;
use servo_base::generic_channel::GenericCallback;
use servo_constellation_traits::{
    Job, JobError, JobResult, JobResultValue, JobType, ScriptToConstellationMessage,
    ServiceWorkerAlgorithm, ServiceWorkerAlgorithmResult, ServiceWorkerRegistrationInfo,
};
use servo_url::{ImmutableOrigin, ServoUrl};

use crate::dom::bindings::codegen::Bindings::ServiceWorkerContainerBinding::{
    RegistrationOptions, ServiceWorkerContainerMethods,
};
use crate::dom::bindings::error::Error;
use crate::dom::bindings::refcounted::Trusted;
use crate::dom::bindings::reflector::DomGlobal;
use crate::dom::bindings::root::{DomRoot, MutNullableDom};
use crate::dom::bindings::str::USVString;
use crate::dom::bindings::structuredclone;
use crate::dom::eventtarget::EventTarget;
use crate::dom::globalscope::GlobalScope;
use crate::dom::promise::Promise;
use crate::dom::serviceworker::ServiceWorker;
use crate::dom::serviceworkerregistration::ServiceWorkerRegistration;
use crate::dom::types::MessageEvent;
use crate::script_runtime::CanGc;

#[dom_struct]
pub(crate) struct ServiceWorkerContainer {
    eventtarget: EventTarget,
    controller: MutNullableDom<ServiceWorker>,

    /// Pending results for
    /// <https://w3c.github.io/ServiceWorker/#algorithms>
    #[conditional_malloc_size_of]
    pending_algorithm_results: DomRefCell<VecDeque<Rc<Promise>>>,

    /// Handler of algorithm results.
    #[no_trace]
    callback: DomRefCell<Option<GenericCallback<ServiceWorkerAlgorithmResult>>>,
}

impl ServiceWorkerContainer {
    fn new_inherited() -> ServiceWorkerContainer {
        ServiceWorkerContainer {
            eventtarget: EventTarget::new_inherited(),
            controller: Default::default(),
            pending_algorithm_results: Default::default(),
            callback: Default::default(),
        }
    }

    pub(crate) fn new(cx: &mut JSContext, global: &GlobalScope) -> DomRoot<ServiceWorkerContainer> {
        reflect_dom_object_with_cx(
            Box::new(ServiceWorkerContainer::new_inherited()),
            global,
            cx,
        )
    }

    /// <https://w3c.github.io/ServiceWorker/#reject-job-promise>
    /// <https://w3c.github.io/ServiceWorker/#resolve-job-promise>
    fn handle_job_result(&self, cx: &mut JSContext, result: JobResult, promise: Rc<Promise>) {
        let global = self.global();
        match result {
            // <https://w3c.github.io/ServiceWorker/#reject-job-promise>
            // Step 2.2: Queue a task, on equivalentJob’s client’s responsible event loop
            // using the DOM manipulation task source,
            // to reject equivalentJob’s job promise with a new exception with errorData,
            // in equivalentJob’s client’s Realm.
            // Note: we are in the task already.
            JobResult::RejectPromise(error) => match error {
                JobError::TypeError => {
                    promise.reject_error_with_cx(
                        cx,
                        Error::Type(c"Failed to register a ServiceWorker".to_owned()),
                    );
                },
                JobError::SecurityError => {
                    promise.reject_error_with_cx(cx, Error::Security(None));
                },
            },
            // <https://w3c.github.io/ServiceWorker/#resolve-job-promise>
            JobResult::ResolvePromise(value) => {
                match value {
                    JobResultValue::Unregister(success) => {
                        promise.resolve_native_with_cx(cx, &success);
                    },
                    JobResultValue::Register(value) => {
                        let ServiceWorkerRegistrationInfo {
                            id,
                            installing_worker,
                            waiting_worker,
                            active_worker,
                            storage_key: _,
                            scope_url,
                            script_url,
                        } = value;
                        // Step 2.2: If equivalentJob’s job type is either register or update,
                        // set convertedValue to the result of getting the service worker registration object
                        // that represents value in equivalentJob’s client.
                        let registration = global.get_serviceworker_registration(
                            &script_url,
                            &scope_url,
                            id,
                            installing_worker,
                            waiting_worker,
                            active_worker,
                            CanGc::from_cx(cx),
                        );

                        // TODO Step 2.3: Else, set convertedValue to value, in equivalentJob’s client’s Realm.

                        // Step 2.4: Resolve equivalentJob’s job promise with convertedValue.
                        promise.resolve_native_with_cx(cx, &*registration);
                    },
                }
            },
        }
    }

    /// Continuation of the parallel steps from
    /// <https://w3c.github.io/ServiceWorker/#dom-serviceworkercontainer-getregistration>
    fn handle_match_registration_result(
        &self,
        cx: &mut JSContext,
        registration_info: Option<ServiceWorkerRegistrationInfo>,
        promise: Rc<Promise>,
    ) {
        // Step 8.1 Let registration be the result of running Match Service Worker Registration given storage key and clientURL.
        // Note: the `registration_info` argument is the result from the parallel algorithm run.

        // Step 8.2: If registration is null, resolve promise with undefined and abort these steps.
        let Some(info) = registration_info else {
            promise.resolve_native_with_cx(cx, &());
            return;
        };

        // Step 8.3: Resolve promise with the result of getting the service worker registration object
        // that represents registration in promise’s relevant settings object.
        let registration = self.global().get_serviceworker_registration(
            &info.script_url,
            &info.scope_url,
            info.id,
            info.installing_worker,
            info.waiting_worker,
            info.active_worker,
            CanGc::from_cx(cx),
        );
        promise.resolve_native_with_cx(cx, &*registration);
    }

    fn handle_algorithm_result(&self, cx: &mut JSContext, result: ServiceWorkerAlgorithmResult) {
        match result {
            ServiceWorkerAlgorithmResult::Job(job_result) => {
                let Some(promise) = self.pending_algorithm_results.borrow_mut().pop_front() else {
                    debug_assert!(false, "No pending algorithm result.");
                    return;
                };
                self.handle_job_result(cx, job_result, promise);
            },
            ServiceWorkerAlgorithmResult::MatchServiceWorkerRegistration(registration_info) => {
                let Some(promise) = self.pending_algorithm_results.borrow_mut().pop_front() else {
                    debug_assert!(false, "No pending algorithm result.");
                    return;
                };
                self.handle_match_registration_result(cx, registration_info, promise);
            },
            ServiceWorkerAlgorithmResult::MessageFromWorker {
                message,
                source,
                scope_url,
                script_url,
                origin,
            } => {
                // <https://w3c.github.io/ServiceWorker/#dom-client-postmessage-message-options>
                // Add a task that runs the following steps to destination’s client message queue:
                // Note: we are in the task.
                // Step 4.5.2: Let source be the result of getting the service worker object
                // that represents contextObject’s relevant global object’s service worker in targetClient.
                let global = self.global();

                // Note: spec uses a MesssageEvent, so it's unclear what to do with source.
                // Perhaps an ExtendableMessageEvent should be used instead.
                // See https://github.com/w3c/ServiceWorker/issues/1823
                let _source =
                    global.get_serviceworker(&script_url, &scope_url, source, CanGc::from_cx(cx));

                // Step 4.5.4: Let messageClone be deserializeRecord.[[Deserialized]].
                // Step 4.5.5: Let newPorts be a new frozen array consisting of all MessagePort objects
                // in deserializeRecord.[[TransferredValues]], if any.
                rooted!(&in(cx) let mut message_val = UndefinedValue());
                if let Ok(ports) =
                    structuredclone::read(cx, &global, message, message_val.handle_mut())
                {
                    // Step 4.5.6: Dispatch an event named message at destination, using MessageEvent, with its origin initialized to origin,
                    // the source attribute initialized to source,
                    // the data attribute initialized to messageClone, and the ports attribute initialized to newPorts.
                    MessageEvent::dispatch_jsval(
                        cx,
                        self.upcast(),
                        &global,
                        message_val.handle(),
                        Some(&origin.ascii_serialization()),
                        None,
                        ports,
                    );
                } else {
                    error!("Failed to deserialize message ports in message from service worker.");
                }
            },
        }
    }

    /// Setup the callback to the backend service, if this hasn't been done already.
    fn get_or_setup_callback(
        &self,
        promise: Rc<Promise>,
    ) -> GenericCallback<ServiceWorkerAlgorithmResult> {
        self.pending_algorithm_results
            .borrow_mut()
            .push_back(promise);
        if let Some(cb) = self.callback.borrow_mut().as_ref() {
            return cb.clone();
        }

        let global = self.global();
        let response_listener = Trusted::new(self);

        let task_source = global
            .task_manager()
            .dom_manipulation_task_source()
            .to_sendable();
        let callback = GenericCallback::new(move |message| {
            let response_listener = response_listener.clone();
            let response = match message {
                Ok(inner) => inner,
                Err(err) => {
                    return error!(
                        "Error in Service worker algorithm result handlings {:?}.",
                        err
                    );
                },
            };
            task_source.queue(task!(set_request_result_to_database: move |cx| {
                let container = response_listener.root();
                container.handle_algorithm_result(cx, response)
            }));
        })
        .expect("Could not create callback");

        *self.callback.borrow_mut() = Some(callback.clone());

        callback
    }

    /// Continuation for
    /// <https://w3c.github.io/ServiceWorker/#dom-serviceworkerregistration-unregister>
    pub(crate) fn create_and_schedule_unregister_job(
        &self,
        cx: &mut JSContext,
        storage_key: ImmutableOrigin,
        scope: ServoUrl,
        script_url: ServoUrl,
        promise: Rc<Promise>,
    ) {
        let global = self.global();
        let result_handler = self.get_or_setup_callback(promise);

        // Step 3: Let job be the result of running Create Job with unregister,
        // registration’s storage key, registration’s scope url, null, promise,
        // and this’s relevant settings object.
        let job = Job::create_job(
            JobType::Unregister,
            scope,
            script_url,
            result_handler,
            global.creation_url(),
            None,
            storage_key,
        );

        // Step 4: Invoke Schedule Job with job.
        if global
            .script_to_constellation_chan()
            .send(ScriptToConstellationMessage::ServiceWorkerAlgorithm(
                ServiceWorkerAlgorithm::Unregister(job),
            ))
            .is_err()
        {
            // Note: pop the promise we just pushed, since we will not get a result back to handle it.
            self.pending_algorithm_results.borrow_mut().pop_back();

            debug_assert!(
                false,
                "Failed to send Unregister algorithm message to the constellation."
            );
            self.handle_algorithm_result(
                cx,
                ServiceWorkerAlgorithmResult::Job(JobResult::RejectPromise(JobError::TypeError)),
            );
        }
    }
}

impl ServiceWorkerContainerMethods<crate::DomTypeHolder> for ServiceWorkerContainer {
    /// <https://w3c.github.io/ServiceWorker/#service-worker-container-controller-attribute>
    fn GetController(&self) -> Option<DomRoot<ServiceWorker>> {
        None
    }

    /// <https://w3c.github.io/ServiceWorker/#dom-serviceworkercontainer-register> - A
    /// and <https://w3c.github.io/ServiceWorker/#start-register> - B
    fn Register(
        &self,
        realm: &mut CurrentRealm,
        script_url: USVString,
        options: &RegistrationOptions,
    ) -> Rc<Promise> {
        // A: Step 2.
        let global = self.global();

        // A: Step 1
        let promise = Promise::new_in_realm(realm);
        let USVString(ref script_url) = script_url;

        // A: Step 3
        let api_base_url = global.api_base_url();
        let script_url = match api_base_url.join(script_url) {
            Ok(url) => url,
            Err(_) => {
                // B: Step 1
                promise.reject_error_with_cx(realm, Error::Type(c"Invalid script URL".to_owned()));
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
                        promise.reject_error_with_cx(
                            realm,
                            Error::Type(c"Invalid scope URL".to_owned()),
                        );
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
                promise.reject_error_with_cx(
                    realm,
                    Error::Type(c"Only secure origins are allowed".to_owned()),
                );
                return promise;
            },
        }
        // B: Step 4
        if script_url.path().to_ascii_lowercase().contains("%2f") ||
            script_url.path().to_ascii_lowercase().contains("%5c")
        {
            promise.reject_error_with_cx(
                realm,
                Error::Type(c"Script URL contains forbidden characters".to_owned()),
            );
            return promise;
        }

        // B: Step 6
        match scope.scheme() {
            "https" | "http" => {},
            _ => {
                promise.reject_error_with_cx(
                    realm,
                    Error::Type(c"Only secure origins are allowed".to_owned()),
                );
                return promise;
            },
        }
        // B: Step 7
        if scope.path().to_ascii_lowercase().contains("%2f") ||
            scope.path().to_ascii_lowercase().contains("%5c")
        {
            promise.reject_error_with_cx(
                realm,
                Error::Type(c"Scope URL contains forbidden characters".to_owned()),
            );
            return promise;
        }

        let result_handler = self.get_or_setup_callback(promise.clone());

        let scope_things =
            ServiceWorkerRegistration::create_scope_things(&global, script_url.clone());

        // B: Step 8 - 13

        // Step 10: Let storage key be the result of running obtain a storage key given client.
        let Some(storage_key) = global.obtain_storage_key() else {
            promise.reject_error_with_cx(
                realm,
                Error::Type(c"Failed to obtain a storage key".to_owned()),
            );
            // Note: pop the promise we just pushed, since we will not get a result back to handle it.
            self.pending_algorithm_results.borrow_mut().pop_back();
            return promise;
        };

        let job = Job::create_job(
            JobType::Register,
            scope,
            script_url,
            result_handler,
            global.creation_url(),
            Some(scope_things),
            storage_key,
        );

        // B: Step 14: schedule job.
        if global
            .script_to_constellation_chan()
            .send(ScriptToConstellationMessage::ServiceWorkerAlgorithm(
                ServiceWorkerAlgorithm::StartRegister(job),
            ))
            .is_err()
        {
            // Note: pop the promise we just pushed, since we will not get a result back to handle it.
            self.pending_algorithm_results.borrow_mut().pop_back();
            debug_assert!(
                false,
                "Failed to send StartRegister algorithm message to the constellation."
            );
            promise.reject_error_with_cx(
                realm,
                Error::Type(c"Failed to register a ServiceWorker".to_owned()),
            );
        }

        // A: Step 7
        promise
    }

    /// <https://w3c.github.io/ServiceWorker/#navigator-service-worker-getRegistration>
    fn GetRegistration(&self, realm: &mut CurrentRealm, client_url: USVString) -> Rc<Promise> {
        // Step 1: Let client be this’s service worker client.
        let global = self.global();

        // Step 7: Let promise be a new promise.
        // Note: done here so it can be used to handle failure of the below steps.
        let promise = Promise::new_in_realm(realm);

        // Step 2: Let client storage key be the result of running obtain a storage key given client.
        let Some(storage_key) = global.obtain_storage_key() else {
            promise.reject_error_with_cx(
                realm,
                Error::Type(c"Failed to obtain a storage key".to_owned()),
            );
            return promise;
        };

        // Step 3: Let clientURL be the result of parsing clientURL with this’s relevant settings object’s API base URL.
        let mut client_url = match global.api_base_url().join(&client_url.0) {
            Ok(url) => url,
            Err(_) => {
                // Step 4: If clientURL is failure, return a promise rejected with a TypeError.
                promise.reject_error_with_cx(
                    realm,
                    Error::Type(c"Failed to parse clientURL".to_owned()),
                );
                return promise;
            },
        };

        // Step 5: Set clientURL’s fragment to null.
        client_url.set_fragment(None);

        // Step 6: If the origin of clientURL is not client’s origin, return a promise rejected with a "SecurityError" DOMException.
        if &client_url.origin() != global.origin().immutable() {
            promise.reject_error_with_cx(realm, Error::Security(None));
            return promise;
        }

        let result_handler = self.get_or_setup_callback(promise.clone());

        // Step 8: Run the following substeps in parallel:
        // Note: continues in parallel in the service worker manager,
        // by way of the constellation.
        if global
            .script_to_constellation_chan()
            .send(ScriptToConstellationMessage::ServiceWorkerAlgorithm(
                ServiceWorkerAlgorithm::MatchServiceWorkerRegistration {
                    client_url,
                    storage_key,
                    result_handler,
                },
            ))
            .is_err()
        {
            // Note: pop the promise we just pushed, since we will not get a result back to handle it.
            self.pending_algorithm_results.borrow_mut().pop_back();
            promise.reject_error_with_cx(
                realm,
                Error::Type(c"Failed to send MatchServiceWorkerRegistration algorithm".to_owned()),
            );
        }

        // Step 9: Return promise.
        promise
    }
}

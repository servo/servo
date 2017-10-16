/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::ServiceWorkerContainerBinding::{ServiceWorkerContainerMethods, Wrap};
use dom::bindings::codegen::Bindings::ServiceWorkerContainerBinding::RegistrationOptions;
use dom::bindings::error::Error;
use dom::bindings::reflector::{DomObject, reflect_dom_object};
use dom::bindings::root::{Dom, DomRoot, MutNullableDom};
use dom::bindings::str::USVString;
use dom::client::Client;
use dom::eventtarget::EventTarget;
use dom::globalscope::GlobalScope;
use dom::promise::Promise;
use dom::serviceworker::ServiceWorker;
use dom_struct::dom_struct;
use script_thread::ScriptThread;
use serviceworkerjob::{Job, JobType};
use std::ascii::AsciiExt;
use std::default::Default;
use std::rc::Rc;

#[dom_struct]
pub struct ServiceWorkerContainer {
    eventtarget: EventTarget,
    controller: MutNullableDom<ServiceWorker>,
    client: Dom<Client>
}

impl ServiceWorkerContainer {
    fn new_inherited(client: &Client) -> ServiceWorkerContainer {
        ServiceWorkerContainer {
            eventtarget: EventTarget::new_inherited(),
            controller: Default::default(),
            client: Dom::from_ref(client),
        }
    }

    #[allow(unrooted_must_root)]
    pub fn new(global: &GlobalScope) -> DomRoot<ServiceWorkerContainer> {
        let client = Client::new(&global.as_window());
        let container = ServiceWorkerContainer::new_inherited(&*client);
        reflect_dom_object(Box::new(container), global, Wrap)
    }
}

impl ServiceWorkerContainerMethods for ServiceWorkerContainer {
    // https://w3c.github.io/ServiceWorker/#service-worker-container-controller-attribute
    fn GetController(&self) -> Option<DomRoot<ServiceWorker>> {
        self.client.get_controller()
    }

    #[allow(unrooted_must_root)]
    // https://w3c.github.io/ServiceWorker/#service-worker-container-register-method and - A
    // https://w3c.github.io/ServiceWorker/#start-register-algorithm - B
    fn Register(&self,
                script_url: USVString,
                options: &RegistrationOptions) -> Rc<Promise> {
        // A: Step 1
        let promise = Promise::new(&*self.global());
        let USVString(ref script_url) = script_url;
        let api_base_url = self.global().api_base_url();
        // A: Step 3-5
        let script_url = match api_base_url.join(script_url) {
            Ok(url) => url,
            Err(_) => {
                promise.reject_error(Error::Type("Invalid script URL".to_owned()));
                return promise;
            }
        };
        // B: Step 2
        match script_url.scheme() {
            "https" | "http" => {},
            _ => {
                promise.reject_error(Error::Type("Only secure origins are allowed".to_owned()));
                return promise;
            }
        }
        // B: Step 3
        if script_url.path().to_ascii_lowercase().contains("%2f") ||
        script_url.path().to_ascii_lowercase().contains("%5c") {
            promise.reject_error(Error::Type("Script URL contains forbidden characters".to_owned()));
            return promise;
        }
        // B: Step 4-5
        let scope = match options.scope {
            Some(ref scope) => {
                let &USVString(ref inner_scope) = scope;
                match api_base_url.join(inner_scope) {
                    Ok(url) => url,
                    Err(_) => {
                        promise.reject_error(Error::Type("Invalid scope URL".to_owned()));
                        return promise;
                    }
                }
            },
            None => script_url.join("./").unwrap()
        };
        // B: Step 6
        match scope.scheme() {
            "https" | "http" => {},
            _ => {
                promise.reject_error(Error::Type("Only secure origins are allowed".to_owned()));
                return promise;
            }
        }
        // B: Step 7
        if scope.path().to_ascii_lowercase().contains("%2f") ||
        scope.path().to_ascii_lowercase().contains("%5c") {
            promise.reject_error(Error::Type("Scope URL contains forbidden characters".to_owned()));
            return promise;
        }

        // B: Step 8
        let job = Job::create_job(JobType::Register, scope, script_url, promise.clone(), &*self.client);
        ScriptThread::schedule_job(job);
        promise
    }
}

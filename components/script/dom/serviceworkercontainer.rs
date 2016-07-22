/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::ServiceWorkerContainerBinding::RegistrationOptions;
use dom::bindings::codegen::Bindings::ServiceWorkerContainerBinding::{ServiceWorkerContainerMethods, Wrap};
use dom::bindings::error::{Error, Fallible};
use dom::bindings::global::GlobalRef;
use dom::bindings::js::{JS, MutNullableHeap, Root};
use dom::bindings::reflector::{Reflectable, reflect_dom_object};
use dom::bindings::str::USVString;
use dom::eventtarget::EventTarget;
use dom::serviceworker::ServiceWorker;
use dom::serviceworkerregistration::ServiceWorkerRegistration;
use script_thread::ScriptThread;
use std::ascii::AsciiExt;
use std::default::Default;

#[dom_struct]
pub struct ServiceWorkerContainer {
    eventtarget: EventTarget,
    controller: MutNullableHeap<JS<ServiceWorker>>,
}

impl ServiceWorkerContainer {
    fn new_inherited() -> ServiceWorkerContainer {
        ServiceWorkerContainer {
            eventtarget: EventTarget::new_inherited(),
            controller: Default::default(),
        }
    }

    pub fn new(global: GlobalRef) -> Root<ServiceWorkerContainer> {
        reflect_dom_object(box ServiceWorkerContainer::new_inherited(), global, Wrap)
    }
}

pub trait Controllable {
    fn set_controller(&self, active_worker: &ServiceWorker);
}

impl Controllable for ServiceWorkerContainer {
    fn set_controller(&self, active_worker: &ServiceWorker) {
        self.controller.set(Some(active_worker))
    }
}

impl ServiceWorkerContainerMethods for ServiceWorkerContainer {
    // https://slightlyoff.github.io/ServiceWorker/spec/service_worker/#service-worker-container-controller-attribute
    fn GetController(&self) -> Option<Root<ServiceWorker>> {
        return self.controller.get()
    }

    // https://slightlyoff.github.io/ServiceWorker/spec/service_worker/#service-worker-container-register-method
    fn Register(&self,
                script_url: USVString,
                options: &RegistrationOptions) -> Fallible<Root<ServiceWorkerRegistration>> {
        let USVString(ref script_url) = script_url;
        // Step 3-4
        let script_url = match self.global().r().api_base_url().join(script_url) {
            Ok(url) => url,
            Err(_) => return Err(Error::Type("Invalid script URL".to_owned()))
        };
        // Step 5
        match script_url.scheme() {
            "https" | "http" => {},
            _ => return Err(Error::Type("Only secure origins are allowed".to_owned()))
        }
        // Step 6
        if script_url.path().to_ascii_lowercase().contains("%2f") ||
        script_url.path().to_ascii_lowercase().contains("%5c") {
            return Err(Error::Type("Script URL contains forbidden characters".to_owned()));
        }
        // Step 8-9
        let scope = match options.scope {
            Some(ref scope) => {
                let &USVString(ref inner_scope) = scope;
                match self.global().r().api_base_url().join(inner_scope) {
                    Ok(url) => url,
                    Err(_) => return Err(Error::Type("Invalid scope URL".to_owned()))
                }
            },
            None => script_url.join("./").unwrap()
        };
        // Step 11
        match scope.scheme() {
            "https" | "http" => {},
            _ => return Err(Error::Type("Only secure origins are allowed".to_owned()))
        }
        // Step 12
        if scope.path().to_ascii_lowercase().contains("%2f") ||
        scope.path().to_ascii_lowercase().contains("%5c") {
            return Err(Error::Type("Scope URL contains forbidden characters".to_owned()));
        }

        let scope_str = scope.as_str().to_owned();
        let worker_registration = ServiceWorkerRegistration::new(self.global().r(),
                                                                 script_url,
                                                                 scope_str.clone(),
                                                                 self);
        ScriptThread::set_registration(scope, &*worker_registration, self.global().r().pipeline());
        Ok(worker_registration)
    }
}

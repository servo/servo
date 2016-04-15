/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::cell::DOMRefCell;
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
use std::cmp::Ordering;
use std::collections::HashMap;
use std::default::Default;
use url::Url;

#[dom_struct]
pub struct ServiceWorkerContainer {
    eventtarget: EventTarget,
    controller: MutNullableHeap<JS<ServiceWorker>>,
    #[ignore_heap_size_of = "Defined in std"]
    registration_map: DOMRefCell<HashMap<String, JS<ServiceWorkerRegistration>>>
}

impl ServiceWorkerContainer {
    fn new_inherited() -> ServiceWorkerContainer {
        ServiceWorkerContainer {
            eventtarget: EventTarget::new_inherited(),
            controller: Default::default(),
            registration_map: DOMRefCell::new(HashMap::new()),
        }
    }

    pub fn new(global: GlobalRef) -> Root<ServiceWorkerContainer> {
        reflect_dom_object(box ServiceWorkerContainer::new_inherited(), global, Wrap)
    }

    pub fn set_registration(&self, scope_url: String, registration_ref: Root<ServiceWorkerRegistration>) {
        self.registration_map.borrow_mut().insert(scope_url.clone(), JS::from_rooted(&registration_ref));
    }

    #[allow(unrooted_must_root)]
    pub fn match_registration(&self, client_url: &str) -> Option<Root<ServiceWorkerRegistration>> {
        for scope in self.registration_map.borrow().keys() {
            match scope.cmp(&client_url.to_owned()) {
                Ordering::Equal => return self.get_registration(Url::parse(client_url).unwrap()),
                _ => return None
            }
        }
        return None;
    }

    #[allow(unrooted_must_root)]
    pub fn get_registration(&self, scope_str: Url) -> Option<Root<ServiceWorkerRegistration>> {
        let path_fragment = scope_str.path();
        match self.registration_map.borrow_mut().remove(path_fragment) {
            Some(reg) => return Some(Root::from_ref(&*reg)),
            None => return None
        }
    }
}

pub trait Controllable {
    fn set_controller(&self, active_worker: Root<ServiceWorker>);
}

impl Controllable for ServiceWorkerContainer {
    fn set_controller(&self, active_worker: Root<ServiceWorker>) {
        self.controller.set(Some(&active_worker))
    }
}

impl ServiceWorkerContainerMethods for ServiceWorkerContainer {

// https://slightlyoff.github.io/ServiceWorker/spec/service_worker/#service-worker-container-controller-attribute
    fn GetController(&self) -> Option<Root<ServiceWorker>> {
        return self.controller.get()
    }

    #[allow(unrooted_must_root)]
// https://slightlyoff.github.io/ServiceWorker/spec/service_worker/#service-worker-container-register-method
    fn Register(&self,
                script_url: USVString,
                options: &RegistrationOptions) -> Fallible<Root<ServiceWorkerRegistration>> {

        let USVString(ref script_url) = script_url;
        // Step 3-4
        let script_url = match self.global().r().api_base_url().join(script_url) {
            Ok(url) => url,
            Err(_) => return Err(Error::Syntax),
        };
        // Step 5
        match script_url.scheme() {
            "https" | "http" => {},
            _ => return Err(Error::Type("Only http/https Supported".to_owned()))
        }
        // Step 6
        if script_url.path().to_lowercase().contains("%2f") || script_url.path().to_lowercase().contains("%5c") {
            return Err(Error::Type("Invalid Url Path".to_owned()));
        }
        // Step 8-9
        let scope_url = match options.scope {
            Some(ref scope) => {
                let &USVString(ref inner_scope) = scope;
                match self.global().r().api_base_url().join(inner_scope) {
                    Ok(url) => url.as_str().to_owned(),
                    Err(_) => return Err(Error::Syntax),
                }
            },
            None => {
                script_url.as_str().to_owned()
            }
        };

        let worker_registration = ServiceWorkerRegistration::new(self.global().r(),
                                                                 script_url,
                                                                 scope_url.clone(),
                                                                 self);

        self.set_registration(scope_url.clone(), worker_registration.clone());
        Ok(worker_registration)
    }
}

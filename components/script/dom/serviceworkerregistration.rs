/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::cell::DOMRefCell;
use dom::bindings::codegen::Bindings::ServiceWorkerRegistrationBinding::{ServiceWorkerRegistrationMethods, Wrap};
use dom::bindings::global::GlobalRef;
use dom::bindings::js::{JS, MutNullableHeap, Root};
use dom::bindings::reflector::{Reflectable, reflect_dom_object};
use dom::bindings::str::USVString;
use dom::eventtarget::EventTarget;
use dom::serviceworker::ServiceWorker;

#[dom_struct]
pub struct ServiceWorkerRegistration {
    eventtarget: EventTarget,
    active: MutNullableHeap<JS<ServiceWorker>>,
    installing: MutNullableHeap<JS<ServiceWorker>>,
    waiting: MutNullableHeap<JS<ServiceWorker>>,
    scope: DOMRefCell<String>,
}

impl ServiceWorkerRegistration {
    fn new_inherited(active_sw: Option<&ServiceWorker>, scope: String) -> ServiceWorkerRegistration {
        ServiceWorkerRegistration {
            eventtarget: EventTarget::new_inherited(),
            active: MutNullableHeap::new(active_sw),
            installing: Default::default(),
            waiting: Default::default(),
            scope: DOMRefCell::new(scope)
        }
    }

    pub fn new(global: GlobalRef, active_sw: Option<&ServiceWorker>, scope: String) -> Root<ServiceWorkerRegistration> {
        reflect_dom_object(box ServiceWorkerRegistration::new_inherited(active_sw, scope), global, Wrap)
    }
}

impl ServiceWorkerRegistrationMethods for ServiceWorkerRegistration {

    // https://slightlyoff.github.io/ServiceWorker/spec/service_worker/#service-worker-registration-installing-attribute
    fn GetInstalling(&self) -> Option<Root<ServiceWorker>> {
        self.installing.get()
    }

    // https://slightlyoff.github.io/ServiceWorker/spec/service_worker/#service-worker-registration-active-attribute
    fn GetActive(&self) -> Option<Root<ServiceWorker>> {
        self.active.get()
    }

    // https://slightlyoff.github.io/ServiceWorker/spec/service_worker/#service-worker-registration-waiting-attribute
    fn GetWaiting(&self) -> Option<Root<ServiceWorker>> {
        self.waiting.get()
    }
    // https://slightlyoff.github.io/ServiceWorker/spec/service_worker/#service-worker-registration-scope-attribute
    fn Scope(&self) -> USVString {
        USVString(self.scope.borrow().clone())
    }
}

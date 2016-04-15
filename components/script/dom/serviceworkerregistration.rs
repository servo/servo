/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::cell::DOMRefCell;
use dom::bindings::codegen::Bindings::ServiceWorkerRegistrationBinding::{ServiceWorkerRegistrationMethods, Wrap};
use dom::bindings::global::GlobalRef;
use dom::bindings::js::{JS, MutNullableHeap, Root};
use dom::bindings::reflector::reflect_dom_object;
use dom::bindings::str::USVString;
use dom::eventtarget::EventTarget;
use dom::serviceworker::ServiceWorker;
use dom::serviceworkercontainer::Controllable;
use std::hash::{Hash, Hasher};
use url::Url;

#[dom_struct]
pub struct ServiceWorkerRegistration {
    eventtarget: EventTarget,
    active: MutNullableHeap<JS<ServiceWorker>>,
    installing: MutNullableHeap<JS<ServiceWorker>>,
    waiting: MutNullableHeap<JS<ServiceWorker>>,
    scope: DOMRefCell<String>,
}

impl ServiceWorkerRegistration {
    #[allow(unrooted_must_root)]
    fn new_inherited(active_sw: &ServiceWorker, scope: String) -> ServiceWorkerRegistration {
        ServiceWorkerRegistration {
            eventtarget: EventTarget::new_inherited(),
            active: MutNullableHeap::new(Some(&active_sw)),
            installing: Default::default(),
            waiting: Default::default(),
            scope: DOMRefCell::new(scope)
        }
    }
    #[allow(unrooted_must_root)]
    pub fn new(global: GlobalRef,
               script_url: Url,
               scope: String,
               container: &Controllable) -> Root<ServiceWorkerRegistration> {

        let active_worker = ServiceWorker::init_service_worker(global, script_url).unwrap();
        container.set_controller(active_worker.clone());
        reflect_dom_object(box ServiceWorkerRegistration::new_inherited(&*active_worker, scope), global, Wrap)
    }
}

// Needed to store in registration map
impl Hash for ServiceWorkerRegistration {
  fn hash<H>(&self, state: &mut H) where H: Hasher {
    for i in self.scope.borrow().chars() {
        state.write_u8(i as u8);
    }
    state.finish();
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

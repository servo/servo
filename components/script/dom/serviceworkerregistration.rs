/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::ServiceWorkerBinding::ServiceWorkerState;
use dom::bindings::codegen::Bindings::ServiceWorkerRegistrationBinding::{ServiceWorkerRegistrationMethods, Wrap};
use dom::bindings::js::{JS, Root};
use dom::bindings::reflector::reflect_dom_object;
use dom::bindings::str::USVString;
use dom::eventtarget::EventTarget;
use dom::globalscope::GlobalScope;
use dom::serviceworker::ServiceWorker;
use dom::serviceworkercontainer::Controllable;
use dom::workerglobalscope::prepare_workerscope_init;
use script_traits::{WorkerScriptLoadOrigin, ScopeThings};
use servo_url::ServoUrl;

#[dom_struct]
pub struct ServiceWorkerRegistration {
    eventtarget: EventTarget,
    active: Option<JS<ServiceWorker>>,
    installing: Option<JS<ServiceWorker>>,
    waiting: Option<JS<ServiceWorker>>,
    scope: String
}

impl ServiceWorkerRegistration {
    fn new_inherited(active_sw: &ServiceWorker, scope: ServoUrl) -> ServiceWorkerRegistration {
        ServiceWorkerRegistration {
            eventtarget: EventTarget::new_inherited(),
            active: Some(JS::from_ref(active_sw)),
            installing: None,
            waiting: None,
            scope: scope.as_str().to_owned(),
        }
    }
    #[allow(unrooted_must_root)]
    pub fn new(global: &GlobalScope,
               script_url: ServoUrl,
               scope: ServoUrl,
               container: &Controllable) -> Root<ServiceWorkerRegistration> {
        let active_worker = ServiceWorker::install_serviceworker(global, script_url.clone(), scope.clone(), true);
        active_worker.set_transition_state(ServiceWorkerState::Installed);
        container.set_controller(&*active_worker.clone());
        reflect_dom_object(box ServiceWorkerRegistration::new_inherited(&*active_worker, scope), global, Wrap)
    }

    pub fn get_installed(&self) -> &ServiceWorker {
        self.active.as_ref().unwrap()
    }

    pub fn create_scope_things(global: &GlobalScope, script_url: ServoUrl) -> ScopeThings {
        let worker_load_origin = WorkerScriptLoadOrigin {
            referrer_url: None,
            referrer_policy: None,
            pipeline_id: Some(global.pipeline_id())
        };

        let worker_id = global.get_next_worker_id();
        let devtools_chan = global.devtools_chan().cloned();
        let init = prepare_workerscope_init(global, None);
        ScopeThings {
            script_url: script_url,
            init: init,
            worker_load_origin: worker_load_origin,
            devtools_chan: devtools_chan,
            worker_id: worker_id
        }
    }
}

pub fn longest_prefix_match(stored_scope: &ServoUrl, potential_match: &ServoUrl) -> bool {
    if stored_scope.origin() != potential_match.origin() {
        return false;
    }
    let scope_chars = stored_scope.path().chars();
    let matching_chars = potential_match.path().chars();
    if scope_chars.count() > matching_chars.count() {
        return false;
    }

    stored_scope.path().chars().zip(potential_match.path().chars()).all(|(scope, matched)| scope == matched)
}

impl ServiceWorkerRegistrationMethods for ServiceWorkerRegistration {
    // https://w3c.github.io/ServiceWorker/#service-worker-registration-installing-attribute
    fn GetInstalling(&self) -> Option<Root<ServiceWorker>> {
        self.installing.as_ref().map(|sw| Root::from_ref(&**sw))
    }

    // https://w3c.github.io/ServiceWorker/#service-worker-registration-active-attribute
    fn GetActive(&self) -> Option<Root<ServiceWorker>> {
        self.active.as_ref().map(|sw| Root::from_ref(&**sw))
    }

    // https://w3c.github.io/ServiceWorker/#service-worker-registration-waiting-attribute
    fn GetWaiting(&self) -> Option<Root<ServiceWorker>> {
        self.waiting.as_ref().map(|sw| Root::from_ref(&**sw))
    }

    // https://w3c.github.io/ServiceWorker/#service-worker-registration-scope-attribute
    fn Scope(&self) -> USVString {
        USVString(self.scope.clone())
    }
}

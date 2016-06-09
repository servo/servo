/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::ServiceWorkerBinding::ServiceWorkerState;
use dom::bindings::codegen::Bindings::ServiceWorkerRegistrationBinding::{ServiceWorkerRegistrationMethods, Wrap};
use dom::bindings::global::GlobalRef;
use dom::bindings::js::{JS, Root};
use dom::bindings::reflector::reflect_dom_object;
use dom::bindings::str::USVString;
use dom::eventtarget::EventTarget;
use dom::serviceworker::ServiceWorker;
use dom::serviceworkercontainer::Controllable;
use dom::workerglobalscope::prepare_workerscope_init;
use ipc_channel::ipc;
use script_traits::{WorkerScriptLoadOrigin, ScopeThings};
use url::Url;

#[dom_struct]
pub struct ServiceWorkerRegistration {
    eventtarget: EventTarget,
    active: Option<JS<ServiceWorker>>,
    installing: Option<JS<ServiceWorker>>,
    waiting: Option<JS<ServiceWorker>>,
    scope: String
}

impl ServiceWorkerRegistration {
    fn new_inherited(active_sw: &ServiceWorker, scope: String) -> ServiceWorkerRegistration {
        ServiceWorkerRegistration {
            eventtarget: EventTarget::new_inherited(),
            active: Some(JS::from_ref(active_sw)),
            installing: None,
            waiting: None,
            scope: scope,
        }
    }
    #[allow(unrooted_must_root)]
    pub fn new(global: GlobalRef,
               script_url: Url,
               scope: String,
               container: &Controllable) -> Root<ServiceWorkerRegistration> {
        let active_worker = ServiceWorker::install_serviceworker(global, script_url.clone(), true);
        active_worker.set_transition_state(ServiceWorkerState::Installed);
        container.set_controller(&*active_worker.clone());
        reflect_dom_object(box ServiceWorkerRegistration::new_inherited(&*active_worker, scope), global, Wrap)
    }

    pub fn get_installed(&self) -> &ServiceWorker {
        self.active.as_ref().unwrap()
    }

    pub fn create_scope_things(global: GlobalRef,
                               script_url: Url) -> ScopeThings {
        let (devtools_sender, devtools_receiver) = ipc::channel().unwrap();
        let closing = false;
        let worker_load_origin = WorkerScriptLoadOrigin {
            referrer_url: None,
            referrer_policy: None,
            request_source: global.request_source(),
            pipeline_id: Some(global.pipeline())
        };

        let init = prepare_workerscope_init(global,
            "Service Worker".to_owned(),
            script_url.clone(),
            devtools_sender.clone(),
            closing);

        ScopeThings {
            script_url: script_url.clone(),
            pipeline_id: global.pipeline(),
            init: init,
            worker_load_origin: worker_load_origin,
            devtools_receiver: devtools_receiver,
        }
    }
}

pub fn longest_prefix_match(src: &Url, dest: &Url) -> bool {
    if src.origin() != dest.origin() {
        return false;
    }
    let src_path_vec = src.path().chars().collect::<Vec<_>>();
    let dest_path_vec = dest.path().chars().collect::<Vec<_>>();
    let mut test_path = Vec::new();
    // at each iteration try to match the path until it
    // becomes equal to the stored registration scope
    // and return true when there's a match.
    for i in dest_path_vec.clone() {
        test_path.push(i);
        if test_path == src_path_vec {
            return true;
        }
    }
    false
}

impl ServiceWorkerRegistrationMethods for ServiceWorkerRegistration {
    // https://slightlyoff.github.io/ServiceWorker/spec/service_worker/#service-worker-registration-installing-attribute
    fn GetInstalling(&self) -> Option<Root<ServiceWorker>> {
        self.installing.as_ref().map(|sw| Root::from_ref(&**sw))
    }

    // https://slightlyoff.github.io/ServiceWorker/spec/service_worker/#service-worker-registration-active-attribute
    fn GetActive(&self) -> Option<Root<ServiceWorker>> {
        self.active.as_ref().map(|sw| Root::from_ref(&**sw))
    }

    // https://slightlyoff.github.io/ServiceWorker/spec/service_worker/#service-worker-registration-waiting-attribute
    fn GetWaiting(&self) -> Option<Root<ServiceWorker>> {
        self.waiting.as_ref().map(|sw| Root::from_ref(&**sw))
    }

    // https://slightlyoff.github.io/ServiceWorker/spec/service_worker/#service-worker-registration-scope-attribute
    fn Scope(&self) -> USVString {
        USVString(self.scope.clone())
    }
}

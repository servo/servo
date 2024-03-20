/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::Cell;

use devtools_traits::WorkerId;
use dom_struct::dom_struct;
use msg::constellation_msg::ServiceWorkerRegistrationId;
use script_traits::{ScopeThings, WorkerScriptLoadOrigin};
use servo_url::ServoUrl;
use uuid::Uuid;

use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::ServiceWorkerRegistrationBinding::{
    ServiceWorkerRegistrationMethods, ServiceWorkerUpdateViaCache,
};
use crate::dom::bindings::reflector::{reflect_dom_object, DomObject};
use crate::dom::bindings::root::{Dom, DomRoot, MutNullableDom};
use crate::dom::bindings::str::{ByteString, USVString};
use crate::dom::eventtarget::EventTarget;
use crate::dom::globalscope::GlobalScope;
use crate::dom::navigationpreloadmanager::NavigationPreloadManager;
use crate::dom::serviceworker::ServiceWorker;
use crate::dom::workerglobalscope::prepare_workerscope_init;

#[dom_struct]
pub struct ServiceWorkerRegistration {
    eventtarget: EventTarget,
    active: DomRefCell<Option<Dom<ServiceWorker>>>,
    installing: DomRefCell<Option<Dom<ServiceWorker>>>,
    waiting: DomRefCell<Option<Dom<ServiceWorker>>>,
    navigation_preload: MutNullableDom<NavigationPreloadManager>,
    #[no_trace]
    scope: ServoUrl,
    navigation_preload_enabled: Cell<bool>,
    navigation_preload_header_value: DomRefCell<Option<ByteString>>,
    update_via_cache: ServiceWorkerUpdateViaCache,
    uninstalling: Cell<bool>,
    #[no_trace]
    registration_id: ServiceWorkerRegistrationId,
}

impl ServiceWorkerRegistration {
    fn new_inherited(
        scope: ServoUrl,
        registration_id: ServiceWorkerRegistrationId,
    ) -> ServiceWorkerRegistration {
        ServiceWorkerRegistration {
            eventtarget: EventTarget::new_inherited(),
            active: DomRefCell::new(None),
            installing: DomRefCell::new(None),
            waiting: DomRefCell::new(None),
            navigation_preload: MutNullableDom::new(None),
            scope: scope,
            navigation_preload_enabled: Cell::new(false),
            navigation_preload_header_value: DomRefCell::new(None),
            update_via_cache: ServiceWorkerUpdateViaCache::Imports,
            uninstalling: Cell::new(false),
            registration_id,
        }
    }

    #[allow(crown::unrooted_must_root)]
    pub fn new(
        global: &GlobalScope,
        scope: ServoUrl,
        registration_id: ServiceWorkerRegistrationId,
    ) -> DomRoot<ServiceWorkerRegistration> {
        reflect_dom_object(
            Box::new(ServiceWorkerRegistration::new_inherited(
                scope,
                registration_id,
            )),
            global,
        )
    }

    /// Does this registration have an active worker?
    pub fn is_active(&self) -> bool {
        self.active.borrow().is_some()
    }

    pub fn set_installing(&self, worker: &ServiceWorker) {
        *self.installing.borrow_mut() = Some(Dom::from_ref(worker));
    }

    pub fn get_navigation_preload_header_value(&self) -> Option<ByteString> {
        self.navigation_preload_header_value.borrow().clone()
    }

    pub fn set_navigation_preload_header_value(&self, value: ByteString) {
        let mut header_value = self.navigation_preload_header_value.borrow_mut();
        *header_value = Some(value);
    }

    pub fn get_navigation_preload_enabled(&self) -> bool {
        self.navigation_preload_enabled.get()
    }

    pub fn set_navigation_preload_enabled(&self, flag: bool) {
        self.navigation_preload_enabled.set(flag)
    }

    pub fn get_uninstalling(&self) -> bool {
        self.uninstalling.get()
    }

    pub fn set_uninstalling(&self, flag: bool) {
        self.uninstalling.set(flag)
    }

    pub fn create_scope_things(global: &GlobalScope, script_url: ServoUrl) -> ScopeThings {
        let worker_load_origin = WorkerScriptLoadOrigin {
            referrer_url: None,
            referrer_policy: None,
            pipeline_id: global.pipeline_id(),
        };

        let worker_id = WorkerId(Uuid::new_v4());
        let devtools_chan = global.devtools_chan().cloned();
        let init = prepare_workerscope_init(global, None, None);
        ScopeThings {
            script_url: script_url,
            init: init,
            worker_load_origin: worker_load_origin,
            devtools_chan: devtools_chan,
            worker_id: worker_id,
        }
    }

    // https://w3c.github.io/ServiceWorker/#get-newest-worker-algorithm
    pub fn get_newest_worker(&self) -> Option<DomRoot<ServiceWorker>> {
        let installing = self.installing.borrow();
        let waiting = self.waiting.borrow();
        let active = self.active.borrow();
        installing
            .as_ref()
            .map(|sw| DomRoot::from_ref(&**sw))
            .or_else(|| waiting.as_ref().map(|sw| DomRoot::from_ref(&**sw)))
            .or_else(|| active.as_ref().map(|sw| DomRoot::from_ref(&**sw)))
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

    stored_scope
        .path()
        .chars()
        .zip(potential_match.path().chars())
        .all(|(scope, matched)| scope == matched)
}

impl ServiceWorkerRegistrationMethods for ServiceWorkerRegistration {
    // https://w3c.github.io/ServiceWorker/#service-worker-registration-installing-attribute
    fn GetInstalling(&self) -> Option<DomRoot<ServiceWorker>> {
        self.installing
            .borrow()
            .as_ref()
            .map(|sw| DomRoot::from_ref(&**sw))
    }

    // https://w3c.github.io/ServiceWorker/#service-worker-registration-active-attribute
    fn GetActive(&self) -> Option<DomRoot<ServiceWorker>> {
        self.active
            .borrow()
            .as_ref()
            .map(|sw| DomRoot::from_ref(&**sw))
    }

    // https://w3c.github.io/ServiceWorker/#service-worker-registration-waiting-attribute
    fn GetWaiting(&self) -> Option<DomRoot<ServiceWorker>> {
        self.waiting
            .borrow()
            .as_ref()
            .map(|sw| DomRoot::from_ref(&**sw))
    }

    // https://w3c.github.io/ServiceWorker/#service-worker-registration-scope-attribute
    fn Scope(&self) -> USVString {
        USVString(self.scope.as_str().to_owned())
    }

    // https://w3c.github.io/ServiceWorker/#service-worker-registration-updateviacache
    fn UpdateViaCache(&self) -> ServiceWorkerUpdateViaCache {
        self.update_via_cache
    }

    // https://w3c.github.io/ServiceWorker/#service-worker-registration-navigationpreload
    fn NavigationPreload(&self) -> DomRoot<NavigationPreloadManager> {
        self.navigation_preload
            .or_init(|| NavigationPreloadManager::new(&self.global(), self))
    }
}

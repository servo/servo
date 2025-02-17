/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::Cell;

use base::id::ServiceWorkerRegistrationId;
use devtools_traits::WorkerId;
use dom_struct::dom_struct;
use net_traits::request::Referrer;
use script_traits::{ScopeThings, WorkerScriptLoadOrigin};
use servo_url::ServoUrl;
use uuid::Uuid;

use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::ServiceWorkerRegistrationBinding::{
    ServiceWorkerRegistrationMethods, ServiceWorkerUpdateViaCache,
};
use crate::dom::bindings::reflector::{reflect_dom_object, DomGlobal};
use crate::dom::bindings::root::{Dom, DomRoot, MutNullableDom};
use crate::dom::bindings::str::{ByteString, USVString};
use crate::dom::eventtarget::EventTarget;
use crate::dom::globalscope::GlobalScope;
use crate::dom::navigationpreloadmanager::NavigationPreloadManager;
use crate::dom::serviceworker::ServiceWorker;
use crate::dom::workerglobalscope::prepare_workerscope_init;
use crate::script_runtime::CanGc;

#[dom_struct]
pub(crate) struct ServiceWorkerRegistration {
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
            scope,
            navigation_preload_enabled: Cell::new(false),
            navigation_preload_header_value: DomRefCell::new(None),
            update_via_cache: ServiceWorkerUpdateViaCache::Imports,
            uninstalling: Cell::new(false),
            registration_id,
        }
    }

    #[cfg_attr(crown, allow(crown::unrooted_must_root))]
    pub(crate) fn new(
        global: &GlobalScope,
        scope: ServoUrl,
        registration_id: ServiceWorkerRegistrationId,
        can_gc: CanGc,
    ) -> DomRoot<ServiceWorkerRegistration> {
        reflect_dom_object(
            Box::new(ServiceWorkerRegistration::new_inherited(
                scope,
                registration_id,
            )),
            global,
            can_gc,
        )
    }

    /// Does this registration have an active worker?
    pub(crate) fn is_active(&self) -> bool {
        self.active.borrow().is_some()
    }

    pub(crate) fn set_installing(&self, worker: &ServiceWorker) {
        *self.installing.borrow_mut() = Some(Dom::from_ref(worker));
    }

    pub(crate) fn get_navigation_preload_header_value(&self) -> Option<ByteString> {
        self.navigation_preload_header_value.borrow().clone()
    }

    pub(crate) fn set_navigation_preload_header_value(&self, value: ByteString) {
        let mut header_value = self.navigation_preload_header_value.borrow_mut();
        *header_value = Some(value);
    }

    pub(crate) fn get_navigation_preload_enabled(&self) -> bool {
        self.navigation_preload_enabled.get()
    }

    pub(crate) fn set_navigation_preload_enabled(&self, flag: bool) {
        self.navigation_preload_enabled.set(flag)
    }

    pub(crate) fn get_uninstalling(&self) -> bool {
        self.uninstalling.get()
    }

    pub(crate) fn set_uninstalling(&self, flag: bool) {
        self.uninstalling.set(flag)
    }

    pub(crate) fn create_scope_things(global: &GlobalScope, script_url: ServoUrl) -> ScopeThings {
        let worker_load_origin = WorkerScriptLoadOrigin {
            referrer_url: match global.get_referrer() {
                Referrer::Client(url) => Some(url),
                Referrer::ReferrerUrl(url) => Some(url),
                _ => None,
            },
            referrer_policy: global.get_referrer_policy(),
            pipeline_id: global.pipeline_id(),
        };

        let worker_id = WorkerId(Uuid::new_v4());
        let devtools_chan = global.devtools_chan().cloned();
        let init = prepare_workerscope_init(global, None, None);
        ScopeThings {
            script_url,
            init,
            worker_load_origin,
            devtools_chan,
            worker_id,
        }
    }

    // https://w3c.github.io/ServiceWorker/#get-newest-worker-algorithm
    pub(crate) fn get_newest_worker(&self) -> Option<DomRoot<ServiceWorker>> {
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

pub(crate) fn longest_prefix_match(stored_scope: &ServoUrl, potential_match: &ServoUrl) -> bool {
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

impl ServiceWorkerRegistrationMethods<crate::DomTypeHolder> for ServiceWorkerRegistration {
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
            .or_init(|| NavigationPreloadManager::new(&self.global(), self, CanGc::note()))
    }
}

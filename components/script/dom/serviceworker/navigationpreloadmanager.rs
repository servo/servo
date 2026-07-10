/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::rc::Rc;

use dom_struct::dom_struct;
use js::context::JSContext;
use js::jsval::UndefinedValue;
use js::realm::CurrentRealm;
use script_bindings::reflector::{Reflector, reflect_dom_object_with_cx};

use crate::dom::bindings::codegen::Bindings::NavigationPreloadManagerBinding::{
    NavigationPreloadManagerMethods, NavigationPreloadState,
};
use crate::dom::bindings::reflector::DomGlobal;
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::bindings::str::ByteString;
use crate::dom::domexception::{DOMErrorName, DOMException};
use crate::dom::globalscope::GlobalScope;
use crate::dom::promise::Promise;
use crate::dom::serviceworkerregistration::ServiceWorkerRegistration;

#[dom_struct]
pub(crate) struct NavigationPreloadManager {
    reflector_: Reflector,
    serviceworker_registration: Dom<ServiceWorkerRegistration>,
}

impl NavigationPreloadManager {
    fn new_inherited(registration: &ServiceWorkerRegistration) -> NavigationPreloadManager {
        NavigationPreloadManager {
            reflector_: Reflector::new(),
            serviceworker_registration: Dom::from_ref(registration),
        }
    }

    #[cfg_attr(crown, expect(crown::unrooted_must_root))]
    pub(crate) fn new(
        cx: &mut JSContext,
        global: &GlobalScope,
        registration: &ServiceWorkerRegistration,
    ) -> DomRoot<NavigationPreloadManager> {
        let manager = NavigationPreloadManager::new_inherited(registration);
        reflect_dom_object_with_cx(Box::new(manager), global, cx)
    }
}

impl NavigationPreloadManagerMethods<crate::DomTypeHolder> for NavigationPreloadManager {
    /// <https://w3c.github.io/ServiceWorker/#navigation-preload-manager-enable>
    fn Enable(&self, cx: &mut CurrentRealm) -> Rc<Promise> {
        let promise = Promise::new_in_realm(cx);

        // 2.
        if self.serviceworker_registration.is_active() {
            let exception = DOMException::new(cx, &self.global(), DOMErrorName::InvalidStateError);
            promise.reject_native(cx, &exception);
        } else {
            // 3.
            self.serviceworker_registration
                .set_navigation_preload_enabled(true);

            // 4.
            promise.resolve_native(cx, &UndefinedValue());
        }

        promise
    }

    /// <https://w3c.github.io/ServiceWorker/#navigation-preload-manager-disable>
    fn Disable(&self, cx: &mut CurrentRealm) -> Rc<Promise> {
        let promise = Promise::new_in_realm(cx);

        // 2.
        if self.serviceworker_registration.is_active() {
            let exception = DOMException::new(cx, &self.global(), DOMErrorName::InvalidStateError);
            promise.reject_native(cx, &exception);
        } else {
            // 3.
            self.serviceworker_registration
                .set_navigation_preload_enabled(false);

            // 4.
            promise.resolve_native(cx, &UndefinedValue());
        }

        promise
    }

    /// <https://w3c.github.io/ServiceWorker/#navigation-preload-manager-setheadervalue>
    fn SetHeaderValue(&self, cx: &mut CurrentRealm, value: ByteString) -> Rc<Promise> {
        let promise = Promise::new_in_realm(cx);

        // 2.
        if self.serviceworker_registration.is_active() {
            let exception = DOMException::new(cx, &self.global(), DOMErrorName::InvalidStateError);
            promise.reject_native(cx, &exception);
        } else {
            // 3.
            self.serviceworker_registration
                .set_navigation_preload_header_value(value);

            // 4.
            promise.resolve_native(cx, &UndefinedValue());
        }

        promise
    }

    /// <https://w3c.github.io/ServiceWorker/#navigation-preload-manager-getstate>
    fn GetState(&self, cx: &mut CurrentRealm) -> Rc<Promise> {
        let promise = Promise::new_in_realm(cx);
        // 2.
        let mut state = NavigationPreloadState::empty();

        // 3.
        if self.serviceworker_registration.is_active()
            && self
                .serviceworker_registration
                .get_navigation_preload_enabled()
        {
            state.enabled = true;
        }

        // 4.
        state.headerValue = self
            .serviceworker_registration
            .get_navigation_preload_header_value();

        // 5.
        promise.resolve_native(cx, &state);

        promise
    }
}

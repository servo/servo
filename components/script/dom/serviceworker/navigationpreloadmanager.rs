/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use js::context::JSContext;
use js::realm::CurrentRealm;
use net_traits::trim_http_whitespace;
use script_bindings::reflector::{Reflector, reflect_dom_object_with_cx};

use crate::dom::bindings::codegen::Bindings::NavigationPreloadManagerBinding::{
    NavigationPreloadManagerMethods, NavigationPreloadState,
};
use crate::dom::bindings::error::Error;
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::bindings::str::ByteString;
use crate::dom::globalscope::GlobalScope;
use crate::dom::promise::{Promise, RootedPromise};
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
    fn Enable(&self, cx: &mut CurrentRealm) -> RootedPromise {
        // Step 1. Let promise be a new promise.
        let promise = Promise::new_in_realm_rooted(cx);

        // Step 2.
        // 1. If registration’s active worker is null, then reject promise with an "InvalidStateError" DOMException.
        if !self.serviceworker_registration.is_active() {
            promise.reject_error(
                cx,
                Error::InvalidState(Some("Registration has no active worker".into())),
            );
        } else {
            // 2. Set registration’s navigation preload enabled to true.
            self.serviceworker_registration
                .set_navigation_preload_enabled(true);

            // 3. Resolve promise with undefined.
            promise.resolve_native(cx, &());
        }

        // Step 3. Return promise.
        promise
    }

    /// <https://w3c.github.io/ServiceWorker/#navigation-preload-manager-disable>
    fn Disable(&self, cx: &mut CurrentRealm) -> RootedPromise {
        // Step 1. Let promise be a new promise.
        let promise = Promise::new_in_realm_rooted(cx);

        // Step 2.
        // 1. If registration’s active worker is null, then reject promise with an "InvalidStateError" DOMException.
        if !self.serviceworker_registration.is_active() {
            promise.reject_error(
                cx,
                Error::InvalidState(Some("Registration has no active worker".into())),
            );
        } else {
            // 2. Set registration’s navigation preload enabled to false.
            self.serviceworker_registration
                .set_navigation_preload_enabled(false);

            // 3. Resolve promise with undefined.
            promise.resolve_native(cx, &());
        }

        // Step 3. Return promise.
        promise
    }

    /// <https://w3c.github.io/ServiceWorker/#navigation-preload-manager-setheadervalue>
    fn SetHeaderValue(&self, cx: &mut CurrentRealm, value: ByteString) -> RootedPromise {
        // Step 1. Let value be the result of normalizing value.
        let value = Vec::<u8>::from(value);
        let normalized_value = trim_http_whitespace(&value);

        // Step 2. If value is not a header value, return a promise rejected with a TypeError.
        let promise = Promise::new_in_realm_rooted(cx);
        if normalized_value.contains(&b'\0') {
            promise.reject_error(cx, Error::Type(c"Invalid header value".to_owned()));
            return promise;
        }

        // Step 4.
        // 1. If registration’s active worker is null, then reject promise with an "InvalidStateError" DOMException.
        if !self.serviceworker_registration.is_active() {
            promise.reject_error(
                cx,
                Error::InvalidState(Some("Registration has no active worker".into())),
            );
        } else {
            // 2. Set registration’s navigation preload header value to value.
            self.serviceworker_registration
                .set_navigation_preload_header_value(ByteString::new(normalized_value.to_vec()));

            // 3. Resolve promise with undefined.
            promise.resolve_native(cx, &());
        }

        // Step 5. Return promise.
        promise
    }

    /// <https://w3c.github.io/ServiceWorker/#navigation-preload-manager-getstate>
    fn GetState(&self, cx: &mut CurrentRealm) -> RootedPromise {
        // Step 1. Let promise be a new promise.
        let promise = Promise::new_in_realm_rooted(cx);

        // Step 2.
        // 1. Let state be a new NavigationPreloadState dictionary.
        let mut state = NavigationPreloadState::empty();

        // 2. Set state["enabled"] to registration’s navigation preload enabled.
        if self
            .serviceworker_registration
            .get_navigation_preload_enabled()
        {
            state.enabled = true;
        }

        // 3. Set state["headerValue"] to registration’s navigation preload header value.
        state.headerValue = Some(
            self.serviceworker_registration
                .get_navigation_preload_header_value(),
        );

        // 4. Resolve promise with state.
        promise.resolve_native(cx, &state);

        // Step 3. Return promise.
        promise
    }
}

/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::rc::Rc;

use dom_struct::dom_struct;
use js::context::JSContext;
use js::jsval::UndefinedValue;
use script_bindings::cell::DomRefCell;
use script_bindings::reflector::{Reflector, reflect_dom_object_with_cx};
use script_bindings::root::{Dom, DomRoot};
use servo_url::ServoUrl;

use crate::dom::bindings::codegen::Bindings::CookieStoreBinding::CookieStoreGetOptions;
use crate::dom::bindings::codegen::Bindings::CookieStoreManagerBinding::CookieStoreManagerMethods;
use crate::dom::bindings::error::Error;
use crate::dom::bindings::reflector::DomGlobal;
use crate::dom::bindings::str::USVString;
use crate::dom::cookiestore::CookieStore;
use crate::dom::globalscope::GlobalScope;
use crate::dom::promise::Promise;
use crate::dom::serviceworker::serviceworkerregistration::{
    ServiceWorkerRegistration, longest_prefix_match,
};

/// <https://cookiestore.spec.whatwg.org/#cookie-store-manager-interface>
#[dom_struct]
pub(crate) struct CookieStoreManager {
    reflector_: Reflector,
    // A CookieStoreManager has an associated registration which is a service worker registration.
    serviceworker_registration: Dom<ServiceWorkerRegistration>,
    // Let subscription list be registration's associated cookie change
    // subscription list.
    #[ignore_malloc_size_of = "generated WebIDL dictionary"]
    subscriptions: DomRefCell<Vec<CookieStoreGetOptions>>,
}

impl CookieStoreManager {
    fn new_inherited(registration: &ServiceWorkerRegistration) -> CookieStoreManager {
        // Each ServiceWorkerRegistration has an associated CookieStoreManager
        // object. The CookieStoreManager's registration is equal to the
        // ServiceWorkerRegistration's service worker registration.
        CookieStoreManager {
            reflector_: Reflector::new(),
            serviceworker_registration: Dom::from_ref(registration),
            subscriptions: DomRefCell::new(Vec::new()),
        }
    }

    pub(crate) fn new(
        cx: &mut JSContext,
        global: &GlobalScope,
        registration: &ServiceWorkerRegistration,
    ) -> DomRoot<CookieStoreManager> {
        // The cookies getter steps are to return this's associated
        // CookieStoreManager object.
        reflect_dom_object_with_cx(Box::new(Self::new_inherited(registration)), global, cx)
    }

    /// <https://cookiestore.spec.whatwg.org/#dom-cookiestoremanager-subscribe>
    fn normalize_subscription(
        &self,
        subscription: CookieStoreGetOptions,
    ) -> Result<CookieStoreGetOptions, Error> {
        // Step 4.2.1. Let name be null.
        let name = subscription
            .name
            .as_ref()
            // Step 4.2.2. If entry["name"] exists:
            // Step 4.2.2.1. Set name to entry["name"].
            // Step 4.2.2.2. Normalize name.
            .map(|name| USVString(CookieStore::normalize(name)));

        // Step 4.2.3. Let url be registration's scope URL.
        let mut url = self.serviceworker_registration.scope_url().clone();
        // Step 4.2.4. If entry["url"] exists, then set url to the result of
        // parsing entry["url"] with settings' API base URL.
        if let Some(entry_url) = subscription.url {
            url = ServoUrl::parse_with_base(Some(&self.global().get_url()), &entry_url.0)
                .map_err(|_| Error::Type(c"Invalid cookie subscription URL".to_owned()))?;
        }

        // Step 4.2.5. If url is failure or url does not start with
        // registration's scope URL, reject p with a TypeError and abort these
        // steps.
        if !longest_prefix_match(self.serviceworker_registration.scope_url(), &url) {
            return Err(Error::Type(
                c"Cookie subscription URL is outside the scope".to_owned(),
            ));
        }

        // Step 4.2.6. Let subscription be the cookie change subscription
        // (name, url).
        Ok(CookieStoreGetOptions {
            name,
            url: Some(USVString(url.as_str().to_owned())),
        })
    }
}

impl CookieStoreManagerMethods<crate::DomTypeHolder> for CookieStoreManager {
    /// <https://cookiestore.spec.whatwg.org/#dom-cookiestoremanager-subscribe>
    fn Subscribe(
        &self,
        cx: &mut JSContext,
        subscriptions: Vec<CookieStoreGetOptions>,
    ) -> Rc<Promise> {
        // Step 1. Let settings be this's relevant settings object.
        // Step 2. Let registration be this's registration.
        // Step 3. Let p be a new promise.
        let promise = Promise::new(cx, &self.global());
        // Step 4.1. Let subscription list be registration's associated cookie
        // change subscription list.

        // Step 4.2. For each entry in subscriptions, run these steps.
        for subscription in subscriptions {
            let subscription = match self.normalize_subscription(subscription) {
                Ok(subscription) => subscription,
                Err(error) => {
                    // Step 4.2.5. Reject p with a TypeError and abort these
                    // steps.
                    promise.reject_error(cx, error);
                    return promise;
                },
            };

            let mut current = self.subscriptions.safe_borrow_mut(cx);
            // Step 4.2.7. If subscription list does not already contain
            // subscription, then append subscription to subscription list.
            if !current.iter().any(|existing| {
                existing.name == subscription.name && existing.url == subscription.url
            }) {
                current.push(subscription);
            }
        }

        // Step 4.3. Resolve p with undefined.
        promise.resolve_native(cx, &UndefinedValue());
        // Step 5. Return p.
        promise
    }

    /// <https://cookiestore.spec.whatwg.org/#dom-cookiestoremanager-getsubscriptions>
    fn GetSubscriptions(&self, cx: &mut JSContext) -> Rc<Promise> {
        // Step 1. Let registration be this's registration.
        // Step 2. Let p be a new promise.
        let promise = Promise::new(cx, &self.global());
        // Step 3.1. Let subscriptions be registration's associated cookie
        // change subscription list.
        let subscriptions = self.subscriptions.borrow();
        // Step 3.2. Let result be « ».
        // Step 3.3. For each subscription, append its name and URL to result.
        // Step 3.4. Resolve p with result.
        promise.resolve_native(cx, &*subscriptions);
        // Step 4. Return p.
        promise
    }

    /// <https://cookiestore.spec.whatwg.org/#dom-cookiestoremanager-unsubscribe>
    fn Unsubscribe(
        &self,
        cx: &mut JSContext,
        subscriptions: Vec<CookieStoreGetOptions>,
    ) -> Rc<Promise> {
        // Step 1. Let settings be this's relevant settings object.
        // Step 2. Let registration be this's registration.
        // Step 3. Let p be a new promise.
        let promise = Promise::new(cx, &self.global());
        // Step 4.1. Let subscription list be registration's associated cookie
        // change subscription list.

        // Step 4.2. For each entry in subscriptions, run these steps.
        for subscription in subscriptions {
            let subscription = match self.normalize_subscription(subscription) {
                Ok(subscription) => subscription,
                Err(error) => {
                    // Step 4.2.5. Reject p with a TypeError and abort these
                    // steps.
                    promise.reject_error(cx, error);
                    return promise;
                },
            };

            let mut current = self.subscriptions.safe_borrow_mut(cx);
            // Step 4.2.7. Remove any item from subscription list equal to
            // subscription.
            current.retain(|existing| {
                existing.name != subscription.name || existing.url != subscription.url
            });
        }

        // Step 4.3. Resolve p with undefined.
        promise.resolve_native(cx, &UndefinedValue());
        // Step 5. Return p.
        promise
    }
}

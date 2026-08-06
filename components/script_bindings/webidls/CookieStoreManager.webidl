/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// https://cookiestore.spec.whatwg.org/#cookie-store-manager-interface
[Exposed=(ServiceWorker,Window),
 SecureContext,
 Pref="dom_cookiestore_enabled"]
interface CookieStoreManager {
  Promise<undefined> subscribe(sequence<CookieStoreGetOptions> subscriptions);
  Promise<sequence<CookieStoreGetOptions>> getSubscriptions();
  Promise<undefined> unsubscribe(sequence<CookieStoreGetOptions> subscriptions);
};

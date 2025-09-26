/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// https://cookiestore.spec.whatwg.org/

[Exposed=(ServiceWorker,Window),
 SecureContext,
 Pref="dom_cookiestore_enabled"]
interface CookieStore : EventTarget {
  Promise<CookieListItem?> get(USVString name);
  Promise<CookieListItem?> get(optional CookieStoreGetOptions options = {});

  Promise<CookieList> getAll(USVString name);
  Promise<CookieList> getAll(optional CookieStoreGetOptions options = {});

  Promise<undefined> set(USVString name, USVString value);
  Promise<undefined> set(CookieInit options);

  Promise<undefined> delete(USVString name);
  Promise<undefined> delete(CookieStoreDeleteOptions options);

  // [Exposed=Window]
  // attribute EventHandler onchange;
};

dictionary CookieStoreGetOptions {
  USVString name;
  USVString url;
};

enum CookieSameSite {
  "strict",
  "lax",
  "none"
};

dictionary CookieInit {
  required USVString name;
  required USVString value;
  DOMHighResTimeStamp? expires = null;
  USVString? domain = null;
  USVString path = "/";
  CookieSameSite sameSite = "strict";
  boolean partitioned = false;
};

dictionary CookieStoreDeleteOptions {
  required USVString name;
  USVString? domain = null;
  USVString path = "/";
  boolean partitioned = false;
};

dictionary CookieListItem {
  USVString name;
  USVString value;
};

typedef sequence<CookieListItem> CookieList;

[SecureContext]
partial interface Window {
  [SameObject, Pref="dom_cookiestore_enabled"] readonly attribute CookieStore cookieStore;
};

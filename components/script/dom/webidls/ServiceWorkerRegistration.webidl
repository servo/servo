/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// https://w3c.github.io/ServiceWorker/#serviceworkerregistration-interface
[Pref="dom.serviceworker.enabled", SecureContext, Exposed=(Window,Worker)]
interface ServiceWorkerRegistration : EventTarget {
  readonly attribute ServiceWorker? installing;
  readonly attribute ServiceWorker? waiting;
  readonly attribute ServiceWorker? active;
  [SameObject] readonly attribute NavigationPreloadManager navigationPreload;

  readonly attribute USVString scope;
  readonly attribute ServiceWorkerUpdateViaCache updateViaCache;

  // [NewObject] Promise<void> update();
  // [NewObject] Promise<boolean> unregister();

  // event
  // attribute EventHandler onupdatefound;
};

enum ServiceWorkerUpdateViaCache {
  "imports",
  "all",
  "none"
};

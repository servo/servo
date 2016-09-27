/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// https://w3c.github.io/ServiceWorker/#service-worker-registration-obj
[Pref="dom.serviceworker.enabled", Exposed=(Window,Worker)]
interface ServiceWorkerRegistration : EventTarget {
  [Unforgeable] readonly attribute ServiceWorker? installing;
  [Unforgeable] readonly attribute ServiceWorker? waiting;
  [Unforgeable] readonly attribute ServiceWorker? active;

  readonly attribute USVString scope;

  // [NewObject] Promise<void> update();
  // [NewObject] Promise<boolean> unregister();

  // event
  // attribute EventHandler onupdatefound;
};

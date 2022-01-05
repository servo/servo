/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// https://w3c.github.io/ServiceWorker/#serviceworker-interface
[Pref="dom.serviceworker.enabled", SecureContext, Exposed=(Window,Worker)]
interface ServiceWorker : EventTarget {
  readonly attribute USVString scriptURL;
  readonly attribute ServiceWorkerState state;
  [Throws] undefined postMessage(any message, sequence<object> transfer);
  [Throws] undefined postMessage(any message, optional PostMessageOptions options = {});

  // event
  attribute EventHandler onstatechange;
};

// FIXME: use `includes` instead of `implements` after #22539 is fixed.
ServiceWorker includes AbstractWorker;

enum ServiceWorkerState {
  "installing",
  "installed",
  "activating",
  "activated",
  "redundant"
};

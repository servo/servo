/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// https://w3c.github.io/ServiceWorker/#service-worker-container
[Pref="dom.serviceworker.enabled", Exposed=(Window,Worker)]
interface ServiceWorkerContainer : EventTarget {
  [Unforgeable] readonly attribute ServiceWorker? controller;
  //[SameObject] readonly attribute Promise<ServiceWorkerRegistration> ready;

  [NewObject] Promise<ServiceWorkerRegistration> register(USVString scriptURL, optional RegistrationOptions options);

  //[NewObject] /*Promise<any>*/ any getRegistration(optional USVString clientURL = "");
  //[NewObject] /* Promise */<sequence<ServiceWorkerRegistration>> getRegistrations();


  // events
  //attribute EventHandler oncontrollerchange;
  //attribute EventHandler onerror;
  //attribute EventHandler onmessage; // event.source of message events is ServiceWorker object
};

dictionary RegistrationOptions {
  USVString scope;
  //WorkerType type = "classic";
};

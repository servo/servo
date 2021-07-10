/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// https://w3c.github.io/ServiceWorker/#serviceworkercontainer-interface
[Pref="dom.serviceworker.enabled", Exposed=(Window,Worker)]
interface ServiceWorkerContainer : EventTarget {
  readonly attribute ServiceWorker? controller;
  //readonly attribute Promise<ServiceWorkerRegistration> ready;

  [NewObject] Promise<ServiceWorkerRegistration> register(USVString scriptURL,
                                                          optional RegistrationOptions options = {});

  //[NewObject] Promise<any> getRegistration(optional USVString clientURL = "");
  //[NewObject] Promise<FrozenArray<ServiceWorkerRegistration>> getRegistrations();

  //void startMessages();

  // events
  //attribute EventHandler oncontrollerchange;
  //attribute EventHandler onerror;
  //attribute EventHandler onmessage; // event.source of message events is ServiceWorker object
  //attribute EventHandler onmessageerror;
};

dictionary RegistrationOptions {
  USVString scope;
  WorkerType type = "classic";
  ServiceWorkerUpdateViaCache updateViaCache = "imports";
};

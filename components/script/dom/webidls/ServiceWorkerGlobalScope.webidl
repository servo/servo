/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// https://w3c.github.io/ServiceWorker/#serviceworkerglobalscope

[Global=(Worker,ServiceWorker), Exposed=ServiceWorker,
 Pref="dom.serviceworker.enabled"]
interface ServiceWorkerGlobalScope : WorkerGlobalScope {
  // A container for a list of Client objects that correspond to
  // browsing contexts (or shared workers) that are on the origin of this SW
  //[SameObject] readonly attribute Clients clients;
  //[SameObject] readonly attribute ServiceWorkerRegistration registration;

  //[NewObject] Promise<void> skipWaiting();

  //attribute EventHandler oninstall;
  //attribute EventHandler onactivate;
  //attribute EventHandler onfetch;

  // event
  attribute EventHandler onmessage; // event.source of the message events is Client object
  attribute EventHandler onmessageerror;
};

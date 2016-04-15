/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// https://slightlyoff.github.io/ServiceWorker/spec/service_worker/#service-worker-global-scope

[Global, Pref="dom.serviceworker.enabled"/*=(Worker,ServiceWorker), Exposed=ServiceWorker*/]
interface ServiceWorkerGlobalScope : WorkerGlobalScope {
  // A container for a list of Client objects that correspond to
  // browsing contexts (or shared workers) that are on the origin of this SW
  //[SameObject] readonly attribute Clients clients;
  //[SameObject] readonly attribute ServiceWorkerRegistration registration;

  //[NewObject] Promise<void> skipWaiting();

  //attribute EventHandler oninstall;
  //attribute EventHandler onactivate;
  //attribute EventHandler onfetch;
  //attribute EventHandler onforeignfetch;

  // event
  attribute EventHandler onmessage; // event.source of the message events is Client object
};

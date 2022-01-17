/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// https://html.spec.whatwg.org/multipage/#dedicatedworkerglobalscope
[Global=(Worker,DedicatedWorker), Exposed=DedicatedWorker]
/*sealed*/ interface DedicatedWorkerGlobalScope : WorkerGlobalScope {
  [Throws] undefined postMessage(any message, sequence<object> transfer);
  [Throws] undefined postMessage(any message, optional PostMessageOptions options = {});
  attribute EventHandler onmessage;

  undefined close();
};

/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// https://html.spec.whatwg.org/multipage/#dedicatedworkerglobalscope
[Global=(Worker,DedicatedWorker), Exposed=DedicatedWorker]
interface DedicatedWorkerGlobalScope : WorkerGlobalScope {
  [Replaceable] readonly attribute DOMString name;

  [Throws] undefined postMessage(any message, sequence<object> transfer);
  [Throws] undefined postMessage(any message, optional StructuredSerializeOptions options = {});
  attribute EventHandler onmessage;
  attribute EventHandler onmessageerror;

  undefined close();
};

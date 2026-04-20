/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// https://html.spec.whatwg.org/multipage/#shared-workers-and-the-sharedworker-interface
[Exposed=Window]
interface SharedWorker : EventTarget {
  [Throws] constructor((TrustedScriptURL or USVString) scriptURL, optional (DOMString or SharedWorkerOptions) options = {});

  readonly attribute MessagePort port;
};
SharedWorker includes AbstractWorker;

dictionary SharedWorkerOptions : WorkerOptions {
  boolean extendedLifetime = false;
};

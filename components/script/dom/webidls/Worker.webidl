/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// https://html.spec.whatwg.org/multipage/#abstractworker
[Exposed=(Window,Worker)]
interface mixin AbstractWorker {
    attribute EventHandler onerror;
};

// https://html.spec.whatwg.org/multipage/#worker
[Exposed=(Window,Worker)]
interface Worker : EventTarget {
  [Throws] constructor(USVString scriptURL, optional WorkerOptions options = {});
  undefined terminate();

  [Throws] undefined postMessage(any message, sequence<object> transfer);
  [Throws] undefined postMessage(any message, optional PostMessageOptions options = {});
  attribute EventHandler onmessage;
  attribute EventHandler onmessageerror;
};

dictionary WorkerOptions {
  WorkerType type = "classic";
  RequestCredentials credentials = "same-origin"; // credentials is only used if type is "module"
  DOMString name = "";
};

enum WorkerType { "classic", "module" };

Worker includes AbstractWorker;

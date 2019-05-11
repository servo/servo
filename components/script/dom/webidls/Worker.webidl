/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// https://html.spec.whatwg.org/multipage/#abstractworker
[NoInterfaceObject, Exposed=(Window,Worker)]
interface AbstractWorker {
    attribute EventHandler onerror;
};

// https://html.spec.whatwg.org/multipage/#worker
[Constructor(USVString scriptURL, optional WorkerOptions options), Exposed=(Window,Worker)]
interface Worker : EventTarget {
  void terminate();

  [Throws] void postMessage(any message/*, sequence<object> transfer*/);
  // void postMessage(any message, optional PostMessageOptions options);
  attribute EventHandler onmessage;
  attribute EventHandler onmessageerror;
};

dictionary WorkerOptions {
  WorkerType type = "classic";
  RequestCredentials credentials = "same-origin"; // credentials is only used if type is "module"
  DOMString name = "";
};

enum WorkerType { "classic", "module" };

Worker implements AbstractWorker;

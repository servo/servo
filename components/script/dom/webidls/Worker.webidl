/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// https://html.spec.whatwg.org/multipage/#abstractworker
[NoInterfaceObject, Exposed=(Window,Worker)]
interface AbstractWorker {
    attribute EventHandler onerror;
};

// https://html.spec.whatwg.org/multipage/#worker
[Constructor(DOMString scriptURL), Exposed=(Window,Worker)]
interface Worker : EventTarget {
  void terminate();

[Throws]
void postMessage(any message/*, optional sequence<Transferable> transfer*/);
           attribute EventHandler onmessage;
};
Worker implements AbstractWorker;

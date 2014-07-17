/* -*- Mode: IDL; tab-width: 2; indent-tabs-mode: nil; c-basic-offset: 2 -*- */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// http://www.whatwg.org/html/#abstractworker
[NoInterfaceObject/*, Exposed=Window,Worker*/]
interface AbstractWorker {
  //         attribute EventHandler onerror;
};

// http://www.whatwg.org/html/#worker
[Constructor(DOMString scriptURL)/*, Exposed=Window,Worker*/]
interface Worker : EventTarget {
  //void terminate();

  //void postMessage(any message/*, optional sequence<Transferable> transfer*/);
  //         attribute EventHandler onmessage;
};
Worker implements AbstractWorker;

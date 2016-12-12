/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
/*
 * The origin of this IDL file is:
 * https://html.spec.whatwg.org/multipage/#messageport
 */

[Exposed=(Window,Worker)]
interface MessagePort : EventTarget {
  [Throws] void postMessage(any message, optional sequence<object> transfer /*= []*/);
  void start();
  void close();

  // event handlers
  attribute EventHandler onmessage;
};

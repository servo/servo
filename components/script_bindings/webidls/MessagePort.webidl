/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
/*
 * The origin of this IDL file is:
 * https://html.spec.whatwg.org/multipage/#messageport
 */

[Exposed=(Window,Worker)]
interface MessagePort : EventTarget {
  [Throws] undefined postMessage(any message, sequence<object> transfer);
  [Throws] undefined postMessage(any message, optional StructuredSerializeOptions options = {});
  undefined start();
  undefined close();

  // event handlers
  attribute EventHandler onmessage;
  attribute EventHandler onmessageerror;
  attribute EventHandler onclose;
};

dictionary StructuredSerializeOptions {
  sequence<object> transfer = [];
};

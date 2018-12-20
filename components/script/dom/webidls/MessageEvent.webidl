/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// https://html.spec.whatwg.org/multipage/#messageevent
[Constructor(DOMString type, optional MessageEventInit eventInitDict), Exposed=(Window,Worker)]
interface MessageEvent : Event {
  readonly attribute any data;
  readonly attribute DOMString origin;
  readonly attribute DOMString lastEventId;
  readonly attribute object? source; // FIXME: WindowProxy is not exposed in Worker globals
  //readonly attribute (WindowProxy or MessagePort)? source;
  //readonly attribute MessagePort[]? ports;
};

dictionary MessageEventInit : EventInit {
  any data = null;
  DOMString origin = "";
  DOMString lastEventId = "";
  //DOMString channel;
  Window? source;
  //(WindowProxy or MessagePort)? source;
  //sequence<MessagePort> ports;
};

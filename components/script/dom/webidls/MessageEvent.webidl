/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// https://html.spec.whatwg.org/multipage/#messageevent
[Exposed=(Window,Worker)]
interface MessageEvent : Event {
  [Throws] constructor(DOMString type, optional MessageEventInit eventInitDict = {});
  readonly attribute any data;
  readonly attribute DOMString origin;
  readonly attribute DOMString lastEventId;
  // FIXME(#22617): WindowProxy is not exposed in Worker globals
  readonly attribute object? source;
  //readonly attribute (WindowProxy or MessagePort)? source;
  readonly attribute /*FrozenArray<MessagePort>*/any ports;
};

dictionary MessageEventInit : EventInit {
  any data = null;
  DOMString origin = "";
  DOMString lastEventId = "";
  //DOMString channel;
  Window? source;
  //(WindowProxy or MessagePort)? source;
  sequence<MessagePort> ports;
};

typedef (/*WindowProxy or */MessagePort or ServiceWorker) MessageEventSource;

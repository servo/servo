/* -*- Mode: IDL; tab-width: 2; indent-tabs-mode: nil; c-basic-offset: 2 -*- */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// http://www.whatwg.org/html/#messageevent
[Constructor(DOMString type, optional MessageEventInit eventInitDict)/*, Exposed=Window,Worker*/]
interface MessageEvent : Event {
  readonly attribute any data;
  readonly attribute DOMString origin;
  readonly attribute DOMString lastEventId;
  //readonly attribute (WindowProxy or MessagePort)? source;
  //readonly attribute MessagePort[]? ports;
};

dictionary MessageEventInit : EventInit {
  any data = null;
  DOMString origin = "";
  DOMString lastEventId = "";
  //DOMString channel;
  //(WindowProxy or MessagePort)? source;
  //sequence<MessagePort> ports;
};

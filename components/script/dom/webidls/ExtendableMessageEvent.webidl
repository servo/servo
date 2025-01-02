/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// https://w3c.github.io/ServiceWorker/#extendablemessage-event-section

[Exposed=ServiceWorker,
 Pref="dom.serviceworker.enabled"]
interface ExtendableMessageEvent : ExtendableEvent {
  [Throws] constructor(DOMString type, optional ExtendableMessageEventInit eventInitDict = {});
  readonly attribute any data;
  readonly attribute DOMString origin;
  readonly attribute DOMString lastEventId;
  // [SameObject] readonly attribute (Client or ServiceWorker /*or MessagePort*/)? source;
  readonly attribute /*FrozenArray<MessagePort>*/any ports;
};

dictionary ExtendableMessageEventInit : ExtendableEventInit {
  any data = null;
  DOMString origin = "";
  DOMString lastEventId = "";
  // (Client or ServiceWorker /*or MessagePort*/)? source;
  sequence<MessagePort> ports = [];
};

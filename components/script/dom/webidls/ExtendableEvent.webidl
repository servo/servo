/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// https://w3c.github.io/ServiceWorker/#extendable-event

[Constructor(DOMString type,
 optional ExtendableEventInit eventInitDict),
 Exposed=ServiceWorker,
 Pref="dom.serviceworker.enabled"]
interface ExtendableEvent : Event {
  [Throws] void waitUntil(/*Promise<*/any/*>*/ f);
};

dictionary ExtendableEventInit : EventInit {
  // Defined for the forward compatibility across the derived events
};

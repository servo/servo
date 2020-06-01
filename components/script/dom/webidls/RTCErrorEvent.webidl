/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

 // https://www.w3.org/TR/webrtc/#rtcerrorevent-interface

[Exposed=Window]
interface RTCErrorEvent : Event {
  constructor(DOMString type, RTCErrorEventInit eventInitDict);
  [SameObject] readonly attribute RTCError error;
};

dictionary RTCErrorEventInit : EventInit {
  required RTCError error;
};
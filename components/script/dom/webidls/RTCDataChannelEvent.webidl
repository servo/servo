/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// https://w3c.github.io/webrtc-pc/#dom-rtcdatachannelevent

[Exposed=Window, Pref="dom.webrtc.enabled"]
interface RTCDataChannelEvent : Event {
  constructor(DOMString type, RTCDataChannelEventInit eventInitDict);
  readonly attribute RTCDataChannel channel;
};

dictionary RTCDataChannelEventInit : EventInit {
  required RTCDataChannel channel;
};

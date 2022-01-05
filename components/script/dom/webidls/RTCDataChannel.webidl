/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// https://w3c.github.io/webrtc-pc/#dom-rtcdatachannel

[Exposed=Window, Pref="dom.webrtc.enabled"]
interface RTCDataChannel : EventTarget {
  readonly attribute USVString label;
  readonly attribute boolean ordered;
  readonly attribute unsigned short? maxPacketLifeTime;
  readonly attribute unsigned short? maxRetransmits;
  readonly attribute USVString protocol;
  readonly attribute boolean negotiated;
  readonly attribute unsigned short? id;
  readonly attribute RTCDataChannelState readyState;
  //readonly attribute unsigned long bufferedAmount;
  //attribute unsigned long bufferedAmountLowThreshold;
  attribute EventHandler onopen;
  attribute EventHandler onbufferedamountlow;
  attribute EventHandler onerror;
  attribute EventHandler onclosing;
  attribute EventHandler onclose;
  undefined close();
  attribute EventHandler onmessage;
  [SetterThrows] attribute DOMString binaryType;
  [Throws] undefined send(USVString data);
  [Throws] undefined send(Blob data);
  [Throws] undefined send(ArrayBuffer data);
  [Throws] undefined send(ArrayBufferView data);
};

// https://www.w3.org/TR/webrtc/#dom-rtcdatachannelinit
dictionary RTCDataChannelInit {
  boolean ordered = true;
  unsigned short maxPacketLifeTime;
  unsigned short maxRetransmits;
  USVString protocol = "";
  boolean negotiated = false;
  unsigned short id;
};

// https://www.w3.org/TR/webrtc/#dom-rtcdatachannelstate
enum RTCDataChannelState {
  "connecting",
  "open",
  "closing",
  "closed"
};
